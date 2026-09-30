use crate::{
    audio::channel::{period::Period, ChannelCore},
    is_bit_set,
};

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

#[derive(Debug, Default, Clone, Copy)]
pub struct SweepControl(u8);

impl SweepControl {
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

#[derive(Debug, Default)]
pub struct Sweep {}

impl Sweep {
    pub fn write(&mut self, value: u8, core: &mut ChannelCore) {}
    pub fn trigger(&mut self, period: &Period, core: &ChannelCore) {}
}
