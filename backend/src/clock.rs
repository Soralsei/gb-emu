use std::cell::{Cell, RefCell};
use std::rc::Rc;

/// Length of a machine cycle, the granularity at which the CPU touches the bus.
pub const M_CYCLE: u16 = 4;

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
pub trait Clocked {
    fn step(&self, elapsed_cycles: u16);
    /// The system divider was reset. Happens on a write to DIV and on STOP,
    /// entering stop mode or not.
    fn reset_div(&self) {}
}

/// Drives every timed device. The CPU advances it one machine cycle per bus
/// access rather than once per instruction, so devices observe the same
/// intra-instruction timing the hardware does.
pub struct Clock {
    cpu_clock: RefCell<Vec<Rc<dyn Clocked>>>,
    fixed_clock: RefCell<Vec<Rc<dyn Clocked>>>,
    elapsed: Cell<u32>,
    stopped: Cell<bool>,
    speed: Cell<Speed>,
    switch_armed: Cell<bool>,
    is_cgb: bool,
}

impl Clock {
    pub fn new(is_cgb: bool) -> Rc<Self> {
        Rc::new(Self {
            cpu_clock: RefCell::new(Vec::new()),
            fixed_clock: RefCell::new(Vec::new()),
            elapsed: Cell::new(0),
            stopped: Cell::new(false),
            switch_armed: Cell::new(false),
            speed: Default::default(),
            is_cgb,
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

    pub fn tick(&self, elapsed_cycles: u16) {
        if elapsed_cycles == 0 || self.stopped.get() {
            return;
        }
        let shift = self.speed.get() as u8;
        // `elapsed` counts fixed-domain cycles so a frame budget stays valid
        // across a speed switch.
        self.elapsed
            .set(self.elapsed.get() + (elapsed_cycles >> shift) as u32);
        self.step_cpu(elapsed_cycles);
        self.step_fixed(elapsed_cycles, shift);
        for device in self.fixed_clock.borrow().iter() {
            device.step(elapsed_cycles >> shift);
        }
    }

    fn step_cpu(&self, elapsed_cycles: u16) {
        for device in self.cpu_clock.borrow().iter() {
            device.step(elapsed_cycles);
        }
    }

    fn step_fixed(&self, elapsed_cycles: u16, shift: u8) {
        for device in self.fixed_clock.borrow().iter() {
            device.step(elapsed_cycles >> shift);
        }
    }

    /// Advance only the fixed-rate devices. The CPU-clocked ones (DIV, serial,
    /// OAM DMA) sit still, as they do during a speed-switch pause.
    pub fn tick_fixed(&self, elapsed_cycles: u16) {
        if elapsed_cycles == 0 || self.stopped.get() {
            return;
        }
        self.elapsed.set(self.elapsed.get() + elapsed_cycles as u32);
        for device in self.fixed_clock.borrow().iter() {
            device.step(elapsed_cycles);
        }
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

    /// Reset the system divider. DIV lives on a CPU-clocked device, so the
    /// fixed-rate ones have nothing to do here.
    pub fn reset_div(&self) {
        for device in self.cpu_clock.borrow().iter() {
            device.reset_div();
        }
    }
}
