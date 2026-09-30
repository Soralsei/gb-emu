use crate::is_bit_set;

mod duty;
mod envelope;
mod noise;
mod period;
mod square;
mod sweep;
mod wave;

pub use noise::Noise;
pub use square::Square;
pub use wave::Wave;

#[derive(Debug, Default, Clone, Copy)]
pub struct NRx4(u8);

impl NRx4 {
    pub fn trigger(&self) -> bool {
        is_bit_set!(self.0, 7)
    }

    pub fn length_enable(&self) -> bool {
        is_bit_set!(self.0, 6)
    }

    pub fn period(&self) -> u8 {
        self.0 & 0x7
    }
}

pub trait Channel {
    fn core(&self) -> &ChannelCore;
    fn core_mut(&mut self) -> &mut ChannelCore;
    fn read(&self, reg: usize) -> u8;
    fn write(&mut self, reg: usize, value: u8, step: u8); // step: NRx4 quirks
    fn power_off(&mut self, is_cgb: bool);
    fn output(&self) -> u8;

    fn enabled(&self) -> bool {
        self.core().enabled
    }
    fn write_length(&mut self, value: u8) {
        self.core_mut().length.load(value)
    }
    fn clock_length(&mut self) {
        self.core_mut().clock_length()
    }
}

#[derive(Debug, Default)]
struct Length {
    counter: u16,
    enabled: bool,
    max: u16,
}

impl Length {
    pub fn trigger(&mut self) {
        if self.counter == 0 {
            self.counter = self.max;
        }
    }

    pub fn enable(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn load(&mut self, value: u8) {
        self.counter = self.max - (value as u16 & (self.max - 1));
    }

    pub fn clock(&mut self) -> bool {
        if self.enabled {
            self.counter = self.counter.saturating_sub(1);
            self.counter == 0
        } else {
            false
        }
    }
}

#[derive(Debug, Default)]
pub struct ChannelCore {
    enabled: bool,
    dac: bool,
    length: Length,
}

impl ChannelCore {
    pub fn set_dac(&mut self, on: bool) {
        self.dac = on;
        self.enabled &= on;
    }

    pub fn disable(&mut self) {
        self.enabled = false;
    }

    pub fn clock_length(&mut self) {
        if self.length.clock() {
            self.enabled = false;
        }
    }

    /// NRx4 bits 6–7. True if triggered; the channel then runs its own part.
    pub fn write_control(&mut self, control: NRx4, step: u8) {
        self.length.enable(control.length_enable());
        if control.trigger() {
            self.enabled = self.dac;
            self.length.trigger();
        }
    }

    pub fn read_control(&self) -> u8 {
        0xBF | (self.length.enabled as u8) << 6
    }
}
