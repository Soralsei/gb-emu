use std::cell::{Cell, RefCell};
use std::rc::Rc;

/// Length of a machine cycle, the granularity at which the CPU touches the bus.
pub const M_CYCLE: u16 = 4;

/// A device that advances with the system clock.
pub trait Clocked {
    fn step(&mut self, elapsed_cycles: u16);
    fn stop(&mut self) {}
}

/// Drives every timed device. The CPU advances it one machine cycle per bus
/// access rather than once per instruction, so devices observe the same
/// intra-instruction timing the hardware does.
pub struct Clock {
    devices: RefCell<Vec<Rc<RefCell<dyn Clocked>>>>,
    elapsed: Cell<u32>,
    stopped: RefCell<bool>,
}

impl Clock {
    pub fn new() -> Rc<Self> {
        Rc::new(Self {
            devices: RefCell::new(Vec::new()),
            elapsed: Cell::new(0),
            stopped: RefCell::new(false),
        })
    }

    pub fn attach<T: Clocked + 'static>(&self, device: Rc<RefCell<T>>) {
        self.devices.borrow_mut().push(device);
    }

    pub fn tick(&self, elapsed_cycles: u16) {
        if elapsed_cycles == 0 || *self.stopped.borrow() {
            return;
        }
        self.elapsed.set(self.elapsed.get() + elapsed_cycles as u32);
        for device in self.devices.borrow().iter() {
            device.borrow_mut().step(elapsed_cycles);
        }
    }

    /// Cycles ticked since the last `reset`.
    pub fn elapsed(&self) -> u32 {
        self.elapsed.get()
    }

    pub fn reset(&self) {
        self.elapsed.set(0);
    }

    pub fn stop(&self) {
        self.stopped.replace(true);
        for device in self.devices.borrow().iter() {
            device.borrow_mut().stop();
        }
    }

    pub fn resume(&self) {
        self.stopped.replace(false);
    }
}
