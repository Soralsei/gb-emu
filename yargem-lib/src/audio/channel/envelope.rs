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

    pub fn pace(&self) -> u8 {
        self.0 & 0x07
    }
}

#[derive(Debug, Default)]
pub struct Envelope {
    volume: u8,
    counter: u8,
    stopped: bool,
    control: NRx2,
}

impl Envelope {
    pub fn write(&mut self, value: u8) {
        self.control = NRx2(value);
    }

    pub fn read(&self) -> u8 {
        self.control.0
    }

    pub fn clock(&mut self) {
        if self.control.pace() == 0 || self.stopped {
            return;
        }
        self.counter = self.counter.saturating_sub(1);
        if self.counter > 0 {
            return;
        }

        self.counter = match self.control.pace() {
            0 => 8,
            pace => pace,
        };

        self.volume = match self.control.env_dir() {
            EnvelopeDirection::Decreasing => self.volume.saturating_sub(1), // capped at 0
            EnvelopeDirection::Increasing => (self.volume + 1).min(15),     // capped at 15
        };

        self.stopped |= self.volume == 15 || self.volume == 0;
    }

    pub fn trigger(&mut self) {
        self.volume = self.control.initial_volume();
        self.counter = match self.control.pace() {
            0 => 8,
            pace => pace,
        };
        self.stopped = false;
    }

    pub fn volume(&self) -> u8 {
        self.volume
    }
}
