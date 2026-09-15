use std::cell::{Cell, RefCell};
use std::convert::Infallible;
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, Poll, Waker};

/// Length of a machine cycle, the granularity at which the CPU touches the bus.
pub const M_CYCLE: u16 = 4;

/// Absolute cycle count since power-on. Wide enough never to wrap in practice.
pub type Cycles = u64;
type BoxFuture<T> = Pin<Box<dyn Future<Output = T>>>;

/// A task's view of time. Holds the clock's counter, not the `Clock` itself:
/// the clock owns the tasks, so an `Rc<Clock>` in a task would be a cycle.
/// Cloning shares the position, so a helper stays on its caller's timeline.
#[derive(Clone)]
pub struct Timeline {
    now: Rc<Cell<Cycles>>,
    at: Rc<Cell<Cycles>>,
}

impl Timeline {
    /// Suspend until `n` more cycles pass. Cumulative, not `now + n`: a tick can
    /// carry several M-cycles, and a task must catch up rather than drop the
    /// overshoot.
    pub fn wait(&self, n: Cycles) -> Wait {
        let at = self.at.get().wrapping_add(n);
        self.at.set(at);
        Wait {
            now: self.now.clone(),
            at,
        }
    }

    /// Cycles elapsed on this task's clock since power-on.
    pub fn now(&self) -> Cycles {
        self.now.get()
    }

    /// The cycle this task has advanced itself to. Trails `now` while catching up.
    pub fn position(&self) -> Cycles {
        self.at.get()
    }
}

/// `wait` advances the task's position as a side effect, so dropping one
/// un-awaited skews every later deadline. `Future`'s own `must_use` does not
/// cover a named return type.
#[must_use = "does nothing unless awaited"]
pub struct Wait {
    now: Rc<Cell<Cycles>>,
    at: Cycles,
}

impl Future for Wait {
    type Output = ();
    fn poll(self: Pin<&mut Self>, _: &mut Context) -> Poll<()> {
        if self.now.get() >= self.at {
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }
}

/// Which clock a task runs on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Domain {
    /// Fixed rate, unaffected by double speed. PPU and APU.
    Fixed,
    /// Runs twice as fast in CGB double speed. Timer, serial, OAM DMA.
    Cpu,
}

/// Tasks never complete: `Infallible` makes that checked rather than conventional.
struct Task(BoxFuture<Infallible>);

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Speed {
    #[default]
    Normal = 0,
    Double = 1,
}

impl From<Speed> for u8 {
    fn from(value: Speed) -> Self {
        value as u8
    }
}

/// A device that advances with the system clock. Devices are shared (`Rc`) and
/// keep their own interior mutability, so stepping never needs a mutable borrow
/// and cannot conflict with a concurrent bus access to the same device.
///
/// Sequence-shaped devices belong in a task instead; see `ASYNC_CLOCK.md`.
pub trait Clocked {
    fn step(&self, elapsed_cycles: u16);
}

/// Drives every timed device. The CPU advances it one machine cycle per bus
/// access rather than once per instruction, so devices observe the same
/// intra-instruction timing the hardware does.
///
/// Both models run during the migration. Order is fixed — `Clocked` devices in
/// attach order, then tasks in spawn order — and must stay that way: the
/// savestate scheme needs the core to be bit-deterministic.
pub struct Clock {
    now_cpu: Rc<Cell<Cycles>>,
    now_fixed: Rc<Cell<Cycles>>,
    tasks: RefCell<Vec<Task>>,

    cpu_clock: RefCell<Vec<Rc<dyn Clocked>>>,
    fixed_clock: RefCell<Vec<Rc<dyn Clocked>>>,

    div_epoch: Cell<Cycles>,
    elapsed: Cell<u32>,
    stopped: Cell<bool>,
    speed: Cell<Speed>,
    switch_armed: Cell<bool>,
    is_cgb: bool,
}

impl Clock {
    pub fn new(is_cgb: bool) -> Rc<Self> {
        Rc::new(Self {
            now_cpu: Rc::new(Cell::new(0)),
            now_fixed: Rc::new(Cell::new(0)),
            tasks: RefCell::new(Vec::with_capacity(10)),
            cpu_clock: RefCell::new(Vec::new()),
            fixed_clock: RefCell::new(Vec::new()),
            elapsed: Cell::new(0),
            stopped: Cell::new(false),
            switch_armed: Cell::new(false),
            speed: Cell::default(),
            is_cgb,
            div_epoch: Cell::default(),
        })
    }

    /// Attach a device clocked by the CPU: it runs twice as fast in CGB double
    /// speed. Timer, serial and OAM DMA belong here.
    pub fn attach(&self, device: Rc<dyn Clocked>) {
        self.cpu_clock.borrow_mut().push(device);
    }

    /// Attach a device clocked at a fixed rate, unaffected by double speed.
    /// PPU and APU belong here.
    pub fn attach_fixed(&self, device: Rc<dyn Clocked>) {
        self.fixed_clock.borrow_mut().push(device);
    }

    /// Attach a device task: `clock.spawn(Domain::Fixed, |t| ppu(state, t))`.
    ///
    /// Takes a closure, not a future, because the `Timeline` must exist first
    /// and only the clock should mint one. Polled once here so the task reaches
    /// its first `await` at the current cycle rather than the next tick's.
    pub fn spawn<F, Fut>(&self, domain: Domain, task: F)
    where
        F: FnOnce(Timeline) -> Fut,
        Fut: Future<Output = Infallible> + 'static,
    {
        let now = match domain {
            Domain::Cpu => self.now_cpu.clone(),
            Domain::Fixed => self.now_fixed.clone(),
        };
        let timeline = Timeline {
            at: Rc::new(Cell::new(now.get())),
            now,
        };

        let mut future = Box::pin(task(timeline));
        let _ = future
            .as_mut()
            .poll(&mut Context::from_waker(Waker::noop()));

        self.tasks
            .try_borrow_mut()
            .expect("Clock::spawn called from inside a task poll")
            .push(Task(future));
    }

    pub fn tick(&self, elapsed: u16) {
        if elapsed == 0 || self.stopped.get() {
            return;
        }
        let shift = self.speed.get() as u8;
        // `elapsed` counts fixed-domain cycles so a frame budget stays valid
        // across a speed switch.
        self.elapsed
            .set(self.elapsed.get() + (elapsed >> shift) as u32);

        self.now_cpu.set(self.now_cpu.get() + elapsed as Cycles);
        self.now_fixed
            .set(self.now_fixed.get() + (elapsed >> shift) as Cycles);

        for device in self.cpu_clock.borrow().iter() {
            device.step(elapsed);
        }
        for device in self.fixed_clock.borrow().iter() {
            device.step(elapsed >> shift);
        }
        self.poll_tasks();
    }

    /// Advance only the fixed-rate devices. The CPU-clocked ones (DIV, serial,
    /// OAM DMA) sit still, as they do during a speed-switch pause.
    pub fn tick_fixed(&self, elapsed: u16) {
        if elapsed == 0 || self.stopped.get() {
            return;
        }
        self.elapsed.set(self.elapsed.get() + elapsed as u32);
        self.now_fixed.set(self.now_fixed.get() + elapsed as Cycles);

        for device in self.fixed_clock.borrow().iter() {
            device.step(elapsed);
        }
        self.poll_tasks();
    }

    /// Poll every task, in spawn order. No scheduling: a suspended poll costs
    /// about what an early-returning `step` costs.
    fn poll_tasks(&self) {
        // Borrow held across every poll, so a task must not re-enter the clock.
        let mut tasks = self
            .tasks
            .try_borrow_mut()
            .expect("Clock::tick re-entered from inside a task poll");

        let mut cx = Context::from_waker(Waker::noop());
        for task in tasks.iter_mut() {
            let _ = task.0.as_mut().poll(&mut cx);
        }
    }

    /// The 16-bit system counter; DIV (0xFF04) is its high byte. TIMA is a
    /// falling-edge detector on one of its bits, and the APU frame sequencer
    /// will be another. Derived, so STOP and double speed need no handling.
    pub fn div(&self) -> u16 {
        self.now_cpu.get().wrapping_sub(self.div_epoch.get()) as u16
    }

    /// Reset the system divider. Happens on a write to DIV and on STOP,
    /// entering stop mode or not.
    pub fn reset_div(&self) {
        self.div_epoch.set(self.now_cpu.get());
    }

    /// Cycles ticked since the last `reset`.
    pub fn elapsed(&self) -> u32 {
        self.elapsed.get()
    }

    pub fn reset(&self) {
        self.elapsed.set(0);
    }

    /// Freeze every device until `resume`. Entered by a STOP with no speed
    /// switch armed, which only a joypad press leaves. A speed switch resets
    /// the dividers the same way but must not freeze the clock, so it calls
    /// `reset_div` alone.
    pub fn stop(&self) {
        self.stopped.replace(true);
        self.reset_div();
    }

    pub fn resume(&self) {
        self.stopped.replace(false);
    }

    pub fn speed(&self) -> Speed {
        self.speed.get()
    }

    pub fn switch_armed(&self) -> bool {
        self.switch_armed.get()
    }

    pub fn arm(&self, on: bool) {
        self.switch_armed.set(on && self.is_cgb);
    }

    /// KEY1 (0xFF4D): bit 7 is the current speed, bit 0 the armed switch.
    /// Unmapped on monochrome models.
    pub fn key1(&self) -> u8 {
        if !self.is_cgb {
            return 0xFF;
        }
        0x7E | (self.speed() as u8) << 7 | self.switch_armed() as u8
    }

    pub fn switch_speed(&self) {
        let speed = match self.speed.get() {
            Speed::Normal => Speed::Double,
            Speed::Double => Speed::Normal,
        };
        self.speed.set(speed);
        self.switch_armed.set(false);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn ticker(t: Timeline, period: Cycles, log: Rc<RefCell<Vec<Cycles>>>) -> Infallible {
        loop {
            t.wait(period).await;
            log.borrow_mut().push(t.position());
        }
    }

    fn spawn_ticker(clock: &Clock, domain: Domain, period: Cycles) -> Rc<RefCell<Vec<Cycles>>> {
        let log: Rc<RefCell<Vec<Cycles>>> = Rc::new(RefCell::new(Vec::new()));
        let handle = log.clone();
        clock.spawn(domain, move |t| ticker(t, period, handle));
        log
    }

    #[test]
    fn does_not_wake_before_its_deadline() {
        let clock = Clock::new(false);
        let log = spawn_ticker(&clock, Domain::Fixed, 4);

        clock.tick(2);
        assert!(log.borrow().is_empty());

        clock.tick(2);
        assert_eq!(*log.borrow(), vec![4]);
    }

    #[test]
    fn catches_up_inside_one_poll() {
        let clock = Clock::new(false);
        let log = spawn_ticker(&clock, Domain::Fixed, 4);

        // Three periods in one tick: three wakes, on multiples of the period.
        clock.tick(12);
        assert_eq!(*log.borrow(), vec![4, 8, 12]);
    }

    #[test]
    fn double_speed_splits_the_domains() {
        let clock = Clock::new(true);
        let cpu = spawn_ticker(&clock, Domain::Cpu, 4);
        let fixed = spawn_ticker(&clock, Domain::Fixed, 4);
        clock.switch_speed();

        clock.tick(4);
        assert_eq!(*cpu.borrow(), vec![4]);
        assert!(fixed.borrow().is_empty(), "fixed domain ran at CPU rate");

        clock.tick(4);
        assert_eq!(*cpu.borrow(), vec![4, 8]);
        assert_eq!(*fixed.borrow(), vec![4]);
    }

    #[test]
    fn poll_order_is_spawn_order() {
        let clock = Clock::new(false);
        let order: Rc<RefCell<Vec<u8>>> = Rc::new(RefCell::new(Vec::new()));

        for id in 0..3u8 {
            let order = order.clone();
            clock.spawn(Domain::Fixed, move |t| async move {
                loop {
                    t.wait(4).await;
                    order.borrow_mut().push(id);
                }
            });
        }

        clock.tick(4);
        assert_eq!(*order.borrow(), vec![0, 1, 2]);
    }

    #[test]
    fn div_is_derived_and_resettable() {
        let clock = Clock::new(false);
        clock.tick(0x120);
        assert_eq!(clock.div(), 0x120);

        clock.reset_div();
        assert_eq!(clock.div(), 0);

        clock.tick(8);
        assert_eq!(clock.div(), 8);
    }

    #[test]
    fn stopped_clock_advances_nothing() {
        let clock = Clock::new(false);
        let log = spawn_ticker(&clock, Domain::Fixed, 4);

        clock.stop();
        clock.tick(16);
        assert!(log.borrow().is_empty());

        clock.resume();
        clock.tick(4);
        assert_eq!(*log.borrow(), vec![4]);
    }
}
