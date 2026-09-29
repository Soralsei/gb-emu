use crate::is_bit_set;

pub trait Channel {
    fn read(&self, address: u16) -> u8;
    fn write(&mut self, address: u16, value: u8);
    fn write_length(&mut self, length: u8);
    fn poweroff(&mut self);
    fn enabled(&self) -> bool;
    fn output(&self) -> u8;
}

pub struct Square {}

pub struct Wave {
    enabled: bool,
    wave_ram: [u8; 0x10], // 0xFF30-0xFF3F
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum LSFRWidth {
    Wide = 0,
    Narrow = 1,
}

impl From<bool> for LSFRWidth {
    fn from(value: bool) -> Self {
        if value {
            LSFRWidth::Narrow
        } else {
            LSFRWidth::Wide
        }
    }
}

/// NR43 : controls frequency of amplitude switching in noise generator
#[derive(Debug, Clone, Copy)]
pub struct RandomControl(u8);

impl RandomControl {
    pub fn clock_shift(&self) -> u8 {
        (self.0 & 0xF0) >> 4
    }
    pub fn lfsr_width(&self) -> LSFRWidth {
        is_bit_set!(self.0, 3).into()
    }
    pub fn div(&self) -> u8 {
        self.0 & 0x07
    }
}

pub struct Noise {
    enabled: bool,
    random_control: RandomControl,
}
