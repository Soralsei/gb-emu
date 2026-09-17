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

    /// A concurrent branch: shares the clock, owns its own position. `clone`
    /// is for helpers that run *on* the caller's timeline; `fork` is for
    /// sub-tasks that advance beside it.
    pub fn fork(&self) -> Timeline {
        Timeline {
            now: self.now.clone(),
            at: Rc::new(Cell::new(self.at.get())),
        }
    }

    /// Absorb a branch's progress back into this timeline.
    pub fn join(&self, branch: &Timeline) {
        self.at.set(self.at.get().max(branch.at.get()));
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

#[derive(Debug)]
struct ClockState {
    pub now_cpu: Rc<Cell<Cycles>>,
    pub now_fixed: Rc<Cell<Cycles>>,
    pub div_epoch: Cell<Cycles>,
    pub stopped: Cell<bool>,
    pub speed: Cell<Speed>,
    pub switch_armed: Cell<bool>,
}

impl ClockState {
    pub fn new() -> Self {
        Self {
            now_cpu: Rc::new(Cell::new(0)),
            now_fixed: Rc::new(Cell::new(0)),
            div_epoch: Cell::default(),
            stopped: Cell::new(false),
            switch_armed: Cell::new(false),
            speed: Cell::default(),
        }
    }
}

pub struct ClockControl(Rc<ClockState>);
impl ClockControl {
    pub fn stopped(&self) -> bool {
        self.0.stopped.get()
    }

    pub fn switch_armed(&self) -> bool {
        self.0.switch_armed.get()
    }

    pub fn speed(&self) -> Speed {
        self.0.speed.get()
    }

    pub fn arm(&self, value: bool) {
        self.0.switch_armed.set(value);
    }

    pub fn switch_speed(&self) {
        let speed = match self.0.speed.get() {
            Speed::Normal => Speed::Double,
            Speed::Double => Speed::Normal,
        };
        self.0.speed.set(speed);
        self.0.switch_armed.set(false);
    }

    pub fn key1(&self) -> u8 {
        0x7E | (self.0.speed.get() as u8) << 7 | self.0.switch_armed.get() as u8
    }

    pub fn stop(&self) {
        self.0.stopped.set(true);
        self.reset_div();
    }

    pub fn resume(&self) {
        self.0.stopped.set(false);
    }

    pub fn reset_div(&self) {
        self.0.div_epoch.set(self.0.now_cpu.get());
    }

    /// The 16-bit system counter; DIV (0xFF04) is its high byte. TIMA is a
    /// falling-edge detector on one of its bits, and the APU frame sequencer
    /// will be another. Derived, so STOP and double speed need no handling.
    pub fn div(&self) -> u16 {
        self.0.now_cpu.get().wrapping_sub(self.0.div_epoch.get()) as u16
    }
}

/// Drives every timed device. The CPU advances it one machine cycle per bus
/// access rather than once per instruction, so devices observe the same
/// intra-instruction timing the hardware does.
///
/// Both models run during the migration. Order is fixed — `Clocked` devices in
/// attach order, then tasks in spawn order — and must stay that way: the
/// savestate scheme needs the core to be bit-deterministic.
pub struct Clock {
    elapsed: Cell<Cycles>,
    tasks: RefCell<Vec<Task>>,
    state: Rc<ClockState>,
}

impl Clock {
    pub fn new() -> Rc<Self> {
        Rc::new(Self {
            elapsed: Cell::new(0),
            tasks: RefCell::new(Vec::with_capacity(10)),
            state: Rc::new(ClockState::new()),
        })
    }

    pub fn clock_control(&self) -> ClockControl {
        ClockControl(self.state.clone())
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
            Domain::Cpu => self.state.now_cpu.clone(),
            Domain::Fixed => self.state.now_fixed.clone(),
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

    pub fn tick(&self) {
        if self.state.stopped.get() {
            return;
        }
        let shift = self.state.speed.get() as u8;
        self.state
            .now_cpu
            .set(self.state.now_cpu.get() + (1 << shift));
        self.state.now_fixed.set(self.state.now_fixed.get() + 1);

        self.poll_tasks();
    }

    pub fn tick_fixed(&self) {
        if self.state.stopped.get() {
            return;
        }
        self.state.now_fixed.set(self.state.now_fixed.get() + 1);
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

    pub fn elapsed(&self) -> Cycles {
        self.elapsed.get()
    }

    pub fn resume(&self) {
        self.state.stopped.set(false);
    }

    pub fn reset(&self) {
        self.elapsed.set(0);
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

    fn clock_tick(clock: &Clock, ticks: u32) {
        for _ in 0..ticks {
            clock.tick();
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
        let clock = Clock::new();
        let log = spawn_ticker(&clock, Domain::Fixed, 4);

        clock_tick(&clock, 2);
        assert!(log.borrow().is_empty());

        clock_tick(&clock, 2);
        assert_eq!(*log.borrow(), vec![4]);
    }

    #[test]
    fn catches_up_inside_one_poll() {
        let clock = Clock::new();
        let log = spawn_ticker(&clock, Domain::Fixed, 4);

        // Three periods in one tick: three wakes, on multiples of the period.
        clock_tick(&clock, 12);
        assert_eq!(*log.borrow(), vec![4, 8, 12]);
    }

    #[test]
    fn double_speed_splits_the_domains() {
        let clock = Clock::new();
        let cpu = spawn_ticker(&clock, Domain::Cpu, 4);
        let fixed = spawn_ticker(&clock, Domain::Fixed, 4);
        let control = clock.clock_control();
        control.switch_speed();

        clock_tick(&clock, 2);
        assert_eq!(*cpu.borrow(), vec![4]);
        assert!(fixed.borrow().is_empty(), "fixed domain ran at CPU rate");

        clock_tick(&clock, 2);
        assert_eq!(*cpu.borrow(), vec![4, 8]);
        assert_eq!(*fixed.borrow(), vec![4]);
    }

    #[test]
    fn poll_order_is_spawn_order() {
        let clock = Clock::new();
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

        clock_tick(&clock, 4);
        assert_eq!(*order.borrow(), vec![0, 1, 2]);
    }

    #[test]
    fn div_is_derived_and_resettable() {
        let clock = Clock::new();
        let control = clock.clock_control();

        clock_tick(&clock, 0x120);
        assert_eq!(control.div(), 0x120);

        control.reset_div();
        assert_eq!(control.div(), 0);

        clock_tick(&clock, 8);
        assert_eq!(control.div(), 8);
    }

    #[test]
    fn stopped_clock_advances_nothing() {
        let clock = Clock::new();
        let log = spawn_ticker(&clock, Domain::Fixed, 4);
        let control = clock.clock_control();

        control.stop();
        clock_tick(&clock, 16);
        assert!(log.borrow().is_empty());

        control.resume();
        clock_tick(&clock, 4);
        assert_eq!(*log.borrow(), vec![4]);
    }
}
