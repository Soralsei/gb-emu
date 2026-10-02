use crate::{
    audio::channel::core_accessors,
    audio::channel::{Channel, ChannelCore},
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
    core: ChannelCore,
    random_control: RandomControl,
}

impl Default for Noise {
    fn default() -> Self {
        Self {
            core: ChannelCore::new(64),
            random_control: RandomControl::default(),
        }
    }
}

impl Channel for Noise {
    const READ_MASK: [u8; 5] = [0xFF, 0xFF, 0x00, 0x00, 0xBF];

    fn read_raw(&self, reg: usize) -> u8 {
        todo!()
    }

    fn write(&mut self, reg: usize, value: u8, step: u8) {
        todo!()
    }

    fn fresh(&self) -> Self {
        Self::default()
    }

    fn clock(&mut self) {
        todo!()
    }

    fn output(&self) -> u8 {
        todo!()
    }

    core_accessors!();
}
