use std::cell::Cell;
use std::convert::Infallible;
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, Poll, Waker};

/// Length of a machine cycle, the granularity at which the CPU touches the bus.
pub const M_CYCLE: u64 = 4;

/// Absolute cycle count since power-on. Wide enough never to wrap in practice.
pub type Cycles = u64;
type BoxFuture<T> = Pin<Box<dyn Future<Output = T>>>;

/// A task's view of time. Holds the clock's counter, not the `Time` itself:
/// the clock owns the tasks, so an `Rc<Time>` in a task would be a cycle.
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

#[derive(Clone, Default)]
pub struct Pulse(Rc<Cell<u32>>);

impl Pulse {
    pub fn new() -> Self {
        Self(Rc::new(Cell::new(0u32)))
    }

    pub fn raise(&self) {
        self.0.set(self.0.get() + 1);
    }

    #[must_use = "does nothing unless awaited"]
    pub fn next(&self) -> impl Future<Output = ()> + '_ {
        std::future::poll_fn(|_| match self.0.get() {
            0 => Poll::Pending,
            n => {
                self.0.set(n - 1);
                Poll::Ready(())
            }
        })
    }
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

struct Counter {
    now: Rc<Cell<Cycles>>,
    stopped: Cell<bool>,
}

impl Counter {
    fn new() -> Self {
        Self {
            now: Rc::new(Cell::new(0)),
            stopped: Cell::new(false),
        }
    }

    fn advance(&self, n: Cycles) {
        if !self.stopped.get() {
            self.now.set(self.now.get() + n);
        }
    }

    /// A task's cursor into this counter, starting where the counter is now.
    fn timeline(&self) -> Timeline {
        Timeline {
            at: Rc::new(Cell::new(self.now.get())),
            now: self.now.clone(),
        }
    }

    fn now(&self) -> Cycles {
        self.now.get()
    }
    fn stop(&self) {
        self.stopped.set(true);
    }
    fn resume(&self) {
        self.stopped.set(false);
    }
}

/// Dot clock. Never changes rate; the PPU and APU are clocked to it.
pub struct FixedClock(Counter);

impl FixedClock {
    pub fn new() -> Rc<Self> {
        Rc::new(Self(Counter::new()))
    }
    pub fn tick(&self) {
        self.0.advance(1);
    }
    pub fn timeline(&self) -> Timeline {
        self.0.timeline()
    }
    pub fn stop(&self) {
        self.0.stop();
    }
    pub fn resume(&self) {
        self.0.resume();
    }
}

/// CPU clock. Doubles in CGB double speed, and carries the state derived from
/// it: the divider, and the speed switch that changes its own rate.
pub struct CpuClock {
    counter: Counter,
    div_epoch: Cell<Cycles>,
    speed: Cell<Speed>,
    switch_armed: Cell<bool>,
}

impl CpuClock {
    pub fn new() -> Rc<Self> {
        Rc::new(Self {
            counter: Counter::new(),
            div_epoch: Cell::new(0),
            speed: Cell::default(),
            switch_armed: Cell::new(false),
        })
    }

    pub fn tick(&self) {
        self.counter.advance(1 << self.speed.get() as u8);
    }
    pub fn timeline(&self) -> Timeline {
        self.counter.timeline()
    }
    pub fn resume(&self) {
        self.counter.resume();
    }

    /// STOP and the speed-switch stall both land here. Freezing the counter
    /// freezes DIV with it, since DIV is derived from it.
    pub fn stop(&self) {
        self.counter.stop();
        self.reset_div();
    }

    pub fn div(&self) -> u16 {
        self.counter.now().wrapping_sub(self.div_epoch.get()) as u16
    }
    pub fn reset_div(&self) {
        self.div_epoch.set(self.counter.now());
    }

    pub fn arm(&self, value: bool) {
        self.switch_armed.set(value);
    }
    pub fn switch_armed(&self) -> bool {
        self.switch_armed.get()
    }
    pub fn key1(&self) -> u8 {
        0x7E | (self.speed.get() as u8) << 7 | self.switch_armed.get() as u8
    }
    pub fn speed(&self) -> Speed {
        self.speed.get()
    }
    pub fn switch_speed(&self) {
        self.speed.set(match self.speed.get() {
            Speed::Normal => Speed::Double,
            Speed::Double => Speed::Normal,
        });
        self.switch_armed.set(false);
    }
}

struct Executor {
    tasks: Vec<Task>,
}

impl Executor {
    pub fn new() -> Self {
        Self {
            tasks: Vec::with_capacity(10),
        }
    }

    /// Polled once here so the task reaches its first `await` at the current
    /// cycle rather than the next tick's.
    pub fn spawn(&mut self, future: impl Future<Output = Infallible> + 'static) {
        let mut future = Box::pin(future);
        let _ = future
            .as_mut()
            .poll(&mut Context::from_waker(Waker::noop()));
        self.tasks.push(Task(future));
    }

    /// Every task, in spawn order, unconditionally. Order is savestate contract.
    pub fn poll(&mut self) {
        let mut cx = Context::from_waker(Waker::noop());
        for task in self.tasks.iter_mut() {
            let _ = task.0.as_mut().poll(&mut cx);
        }
    }
}

pub struct Time {
    pub cpu: Rc<CpuClock>,
    pub fixed: Rc<FixedClock>,
    exec: Executor,
}

impl Time {
    pub fn new() -> Self {
        Self {
            cpu: CpuClock::new(),
            fixed: FixedClock::new(),
            exec: Executor::new(),
        }
    }

    pub fn tick(&mut self) {
        self.fixed.tick();
        self.cpu.tick();
        self.exec.poll();
    }

    pub fn spawn(&mut self, future: impl Future<Output = Infallible> + 'static) {
        self.exec.spawn(future);
    }

    pub fn resume(&self) {
        self.cpu.resume();
        self.fixed.resume();
    }

    pub fn stop(&self) {
        self.cpu.stop();
        self.fixed.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    async fn ticker(t: Timeline, period: Cycles, log: Rc<RefCell<Vec<Cycles>>>) -> Infallible {
        loop {
            t.wait(period).await;
            log.borrow_mut().push(t.position());
        }
    }

    fn clock_tick(clock: &mut Time, ticks: u32) {
        for _ in 0..ticks {
            clock.tick();
        }
    }

    fn spawn_ticker(
        clock: &mut Time,
        timeline: Timeline,
        period: Cycles,
    ) -> Rc<RefCell<Vec<Cycles>>> {
        let log: Rc<RefCell<Vec<Cycles>>> = Rc::new(RefCell::new(Vec::new()));
        let handle = log.clone();
        clock.spawn(ticker(timeline, period, handle));
        log
    }

    #[test]
    fn does_not_wake_before_its_deadline() {
        let mut clock = Time::new();
        let timeline = clock.fixed.timeline();
        let log = spawn_ticker(&mut clock, timeline, 4);

        clock_tick(&mut clock, 2);
        assert!(log.borrow().is_empty());

        clock_tick(&mut clock, 2);
        assert_eq!(*log.borrow(), vec![4]);
    }

    #[test]
    fn catches_up_inside_one_poll() {
        let mut clock = Time::new();
        let timeline = clock.fixed.timeline();
        let log = spawn_ticker(&mut clock, timeline, 4);

        // Three periods in one tick: three wakes, on multiples of the period.
        clock_tick(&mut clock, 12);
        assert_eq!(*log.borrow(), vec![4, 8, 12]);
    }

    #[test]
    fn double_speed_splits_the_domains() {
        let mut clock = Time::new();
        let t_cpu = clock.cpu.timeline();
        let t_fixed = clock.fixed.timeline();
        let cpu = spawn_ticker(&mut clock, t_cpu, 4);
        let fixed = spawn_ticker(&mut clock, t_fixed, 4);

        clock.cpu.switch_speed();

        clock_tick(&mut clock, 2);
        assert_eq!(*cpu.borrow(), vec![4]);
        assert!(fixed.borrow().is_empty(), "fixed domain ran at CPU rate");

        clock_tick(&mut clock, 2);
        assert_eq!(*cpu.borrow(), vec![4, 8]);
        assert_eq!(*fixed.borrow(), vec![4]);
    }

    #[test]
    fn poll_order_is_spawn_order() {
        let mut clock = Time::new();
        let order: Rc<RefCell<Vec<u8>>> = Rc::new(RefCell::new(Vec::new()));

        for id in 0..3u8 {
            let order = order.clone();
            let t = clock.fixed.timeline();
            clock.spawn(async move {
                loop {
                    t.wait(4).await;
                    order.borrow_mut().push(id);
                }
            });
        }

        clock_tick(&mut clock, 4);
        assert_eq!(*order.borrow(), vec![0, 1, 2]);
    }

    #[test]
    fn div_is_derived_and_resettable() {
        let mut clock = Time::new();

        clock_tick(&mut clock, 0x120);
        assert_eq!(clock.cpu.div(), 0x120);

        clock.cpu.reset_div();
        assert_eq!(clock.cpu.div(), 0);

        clock_tick(&mut clock, 8);
        assert_eq!(clock.cpu.div(), 8);
    }

    #[test]
    fn stopped_clock_advances_nothing() {
        let mut clock = Time::new();
        let t_cpu = clock.cpu.timeline();
        let log = spawn_ticker(&mut clock, t_cpu, 4);

        clock.cpu.stop();
        clock_tick(&mut clock, 16);
        assert!(log.borrow().is_empty());

        clock.cpu.resume();
        clock_tick(&mut clock, 4);
        assert_eq!(*log.borrow(), vec![4]);
    }
}
