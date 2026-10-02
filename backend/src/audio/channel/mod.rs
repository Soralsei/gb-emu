#![allow(dead_code)]
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

macro_rules! core_accessors {
    () => {
        fn core(&self) -> &$crate::audio::channel::ChannelCore {
            &self.core
        }
        fn core_mut(&mut self) -> &mut $crate::audio::channel::ChannelCore {
            &mut self.core
        }
    };
}
pub(crate) use core_accessors;

/// Not object-safe (`READ_MASK`): the APU holds each channel by its concrete
/// type, so every call is static.
pub trait Channel: Sized {
    /// Bits that read back as 1 (unused or write-only), NRx0..NRx4.
    const READ_MASK: [u8; 5];

    fn core(&self) -> &ChannelCore;
    fn core_mut(&mut self) -> &mut ChannelCore;

    /// The register's stored bits only; `read` applies `READ_MASK`.
    fn read_raw(&self, reg: usize) -> u8;
    fn write(&mut self, reg: usize, value: u8, step: u8); // step: NRx4 quirks

    fn read(&self, reg: usize) -> u8 {
        self.read_raw(reg) | Self::READ_MASK[reg]
    }

    fn fresh(&self) -> Self;

    fn power_off(&mut self, is_cgb: bool) {
        let fresh = self.fresh();
        let old = std::mem::replace(self, fresh);
        self.core_mut().keep_through_power_off(old.core(), is_cgb);
    }

    fn clock(&mut self);
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
    fn clock_sweep(&mut self) {} // CH1 overrides
    fn clock_envelope(&mut self) {} // CH1, CH2, CH4 override
}

#[derive(Debug)]
struct Length {
    counter: u16,
    enabled: bool,
    max: u16,
}

impl Length {
    fn new(max: u16) -> Self {
        Self {
            counter: 0,
            enabled: false,
            max,
        }
    }

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
        if self.enabled && self.counter > 0 {
            self.counter = self.counter.saturating_sub(1);
            self.counter == 0
        } else {
            false
        }
    }
}

#[derive(Debug)]
pub struct ChannelCore {
    enabled: bool,
    dac: bool,
    length: Length,
}

impl ChannelCore {
    /// `length_max` is 64, or 256 for CH3. No `Default`: a zero max would make
    /// `Length::load` underflow.
    pub fn new(length_max: u16) -> Self {
        Self {
            enabled: false,
            dac: false,
            length: Length::new(length_max),
        }
    }

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

    /// NRx4 bits 6–7.
    pub fn write_control(&mut self, control: NRx4, step: u8) {
        self.length.enable(control.length_enable());
        if control.trigger() {
            self.enabled = self.dac;
            self.length.trigger();
        }
    }

    pub fn read_control(&self) -> u8 {
        (self.length.enabled as u8) << 6
    }

    pub fn keep_through_power_off(&mut self, old: &Self, is_cgb: bool) {
        if !is_cgb {
            self.length.counter = old.length.counter;
        }
    }
}
