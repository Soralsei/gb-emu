use crate::{
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

#[derive(Debug, Default)]
pub struct Noise {
    core: ChannelCore,
    random_control: RandomControl,
}

impl Channel for Noise {
    fn core(&self) -> &ChannelCore {
        &self.core
    }

    fn core_mut(&mut self) -> &mut ChannelCore {
        &mut self.core
    }

    fn read(&self, reg: usize) -> u8 {
        todo!()
    }

    fn write(&mut self, reg: usize, value: u8, step: u8) {
        todo!()
    }

    fn power_off(&mut self, is_cgb: bool) {
        todo!()
    }

    fn output(&self) -> u8 {
        todo!()
    }
}
