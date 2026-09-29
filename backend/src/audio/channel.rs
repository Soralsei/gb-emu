use crate::is_bit_set;

pub trait Channel {
    fn read(&self, reg: u8) -> u8;
    fn write(&mut self, reg: u8, value: u8);
    fn write_length(&mut self, length: u8);
    fn poweroff(&mut self);
    fn enabled(&self) -> bool;
    fn output(&self) -> u8;
}

pub struct Square {}

impl Channel for Square {
    fn read(&self, reg: u8) -> u8 {
        todo!()
    }

    fn write(&mut self, reg: u8, value: u8) {
        todo!()
    }

    fn write_length(&mut self, length: u8) {
        todo!()
    }

    fn poweroff(&mut self) {
        todo!()
    }

    fn enabled(&self) -> bool {
        todo!()
    }

    fn output(&self) -> u8 {
        todo!()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum SweepDirection {
    Increasing = 0,
    Decreasing = 1,
}

impl From<bool> for SweepDirection {
    fn from(value: bool) -> Self {
        if value {
            SweepDirection::Decreasing
        } else {
            SweepDirection::Increasing
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Sweep(u8);

impl Sweep {
    // bits 4-6
    pub fn pace(&self) -> u8 {
        (self.0 & 0x70) >> 4
    }

    // bit 3
    pub fn direction(&self) -> SweepDirection {
        is_bit_set!(self.0, 3).into()
    }

    // bit 0-2
    pub fn individual_step(&self) -> u8 {
        self.0 & 0x7
    }
}

pub struct SquareSweep {
    inner: Square,
    sweep: Sweep, // NR10
}

impl Channel for SquareSweep {
    fn read(&self, reg: u8) -> u8 {
        match reg {
            0x10 => self.sweep.0,
            _ => self.inner.read(reg),
        }
    }

    fn write(&mut self, reg: u8, value: u8) {
        todo!()
    }

    fn write_length(&mut self, length: u8) {
        todo!()
    }

    fn poweroff(&mut self) {
        todo!()
    }

    fn enabled(&self) -> bool {
        todo!()
    }

    fn output(&self) -> u8 {
        todo!()
    }
}

pub struct Wave {
    enabled: bool,
    wave_ram: [u8; 0x10], // 0xFF30-0xFF3F
}

impl Channel for Wave {
    fn read(&self, reg: u8) -> u8 {
        todo!()
    }

    fn write(&mut self, reg: u8, value: u8) {
        todo!()
    }

    fn write_length(&mut self, length: u8) {
        todo!()
    }

    fn poweroff(&mut self) {
        todo!()
    }

    fn enabled(&self) -> bool {
        todo!()
    }

    fn output(&self) -> u8 {
        todo!()
    }
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

impl Channel for Noise {
    fn read(&self, reg: u8) -> u8 {
        todo!()
    }

    fn write(&mut self, reg: u8, value: u8) {
        todo!()
    }

    fn write_length(&mut self, length: u8) {
        todo!()
    }

    fn poweroff(&mut self) {
        todo!()
    }

    fn enabled(&self) -> bool {
        todo!()
    }

    fn output(&self) -> u8 {
        todo!()
    }
}
