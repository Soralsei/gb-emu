use crate::is_bit_set;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum EnvelopeDirection {
    Decreasing = 0,
    Increasing = 1,
}

impl From<bool> for EnvelopeDirection {
    fn from(value: bool) -> Self {
        if value {
            EnvelopeDirection::Increasing
        } else {
            EnvelopeDirection::Decreasing
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct NRx2(u8);

impl NRx2 {
    pub fn initial_volume(&self) -> u8 {
        self.0 >> 4
    }

    pub fn env_dir(&self) -> EnvelopeDirection {
        is_bit_set!(self.0, 3).into()
    }
}

#[derive(Debug, Default)]
pub struct Envelope {}

impl Envelope {
    pub fn write(&mut self, value: u8) {}
    pub fn trigger(&mut self) {}
}
