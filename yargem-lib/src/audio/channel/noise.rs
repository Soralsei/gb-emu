use crate::{
    audio::channel::{core_accessors, envelope::Envelope, Channel, ChannelCore, NRx4},
    is_bit_set,
};

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum LfsrWidth {
    Wide = 0,
    Narrow = 1,
}

impl From<bool> for LfsrWidth {
    fn from(value: bool) -> Self {
        if value {
            LfsrWidth::Narrow
        } else {
            LfsrWidth::Wide
        }
    }
}

/// NR43 : controls frequency of amplitude switching in noise generator
#[derive(Debug, Default, Clone, Copy)]
pub struct RandomControl(u8);

impl RandomControl {
    pub fn clock_shift(&self) -> u8 {
        (self.0 & 0xF0) >> 4
    }
    pub fn lfsr_width(&self) -> LfsrWidth {
        is_bit_set!(self.0, 3).into()
    }
    pub fn div(&self) -> u8 {
        self.0 & 0x07
    }
}

#[derive(Debug)]
pub struct Noise {
    core: ChannelCore<64>,
    envelope: Envelope,
    random_control: RandomControl,
    shift_register: u16,
    counter: u32,
}

impl Noise {
    fn timer_period(&self) -> u32 {
        let shift = self.random_control.clock_shift();
        let div = self.random_control.div() as u32;
        // expressed in terms of clock calls instead of dots (divided by 2 => 1 call every 2 dots)
        (if div == 0 { 4 } else { 8 * div }) << shift
    }
}

impl Default for Noise {
    fn default() -> Self {
        Self {
            core: ChannelCore::new(),
            envelope: Envelope::default(),
            random_control: RandomControl::default(),
            shift_register: 0u16,
            counter: 0u32,
        }
    }
}

impl Channel<64> for Noise {
    const READ_MASK: [u8; 5] = [0xFF, 0xFF, 0x00, 0x00, 0xBF];

    fn read_raw(&self, reg: usize) -> u8 {
        debug_assert!(
            reg <= 4,
            "[Noise::read_raw] reg value should never be > 4, got {}",
            reg
        );
        match reg {
            1 => 0,
            2 => self.envelope.read(),
            3 => self.random_control.0,
            4 => self.core.read_control(),
            _ => 0,
        }
    }

    fn write(&mut self, reg: usize, value: u8, do_clock_length: bool) {
        debug_assert!(
            reg <= 4,
            "[Noise::write] reg value should never be > 4, got {}",
            reg
        );
        match reg {
            1 => self.core.length.load(value),
            2 => {
                self.envelope.write(value);
                self.core.set_dac(value & 0xF8 != 0);
            }
            3 => self.random_control = RandomControl(value),
            4 => {
                let control = NRx4(value);
                self.core.write_control(control, do_clock_length);
                if control.trigger() {
                    self.envelope.trigger();
                    self.shift_register = 0;
                    self.counter = self.timer_period();
                }
            }
            _ => { /* unused */ }
        }
    }

    fn fresh(&self) -> Self {
        Self::default()
    }

    fn clock(&mut self) {
        if !self.enabled() {
            return;
        }
        let shift = self.random_control.clock_shift();
        if shift >= 14 {
            return;
        }
        self.counter = self.counter.saturating_sub(1);
        if self.counter != 0 {
            return;
        }
        self.counter = self.timer_period();

        let feedback_bit = !(self.shift_register ^ (self.shift_register >> 1)) & 1;

        // Copy !(LFSR_0 ^ LFSR_1) to bit 15 of the shift register
        self.shift_register = feedback_bit << 15 | (self.shift_register & !0x8000);

        // if in narrow mode, also copy to bit 7 of the shift register
        if matches!(self.random_control.lfsr_width(), LfsrWidth::Narrow) {
            self.shift_register = (self.shift_register & !0x0080) | (feedback_bit << 7);
        }

        self.shift_register >>= 1;
    }

    fn output(&self) -> u8 {
        if self.enabled() && is_bit_set!(self.shift_register, 0) {
            self.envelope.volume()
        } else {
            0
        }
    }

    fn clock_envelope(&mut self) {
        self.envelope.clock();
    }

    core_accessors!(64);
}
