use crate::is_bit_set;

#[derive(Debug, Default, Clone, Copy)]
pub struct Panning(u8);
impl Panning {
    pub fn left(&self, channel: usize) -> bool {
        is_bit_set!(self.0, channel + 4)
    }
    pub fn right(&self, channel: usize) -> bool {
        is_bit_set!(self.0, channel)
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct MasterVolume(u8);
impl MasterVolume {
    pub fn vin_left(&self) -> bool {
        is_bit_set!(self.0, 7)
    }
    pub fn vin_right(&self) -> bool {
        is_bit_set!(self.0, 3)
    }

    pub fn volume_left(&self) -> u8 {
        (self.0 & 0x70) >> 4
    }
    pub fn volume_right(&self) -> u8 {
        self.0 & 0x07
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct AudioRegisters {
    master_volume: MasterVolume,
    panning: Panning,
}

impl AudioRegisters {
    pub fn read(&self, address: u16) -> u8 {
        match address {
            0xFF25 => self.panning.0,
            _ => self.master_volume.0,
        }
    }

    pub fn write(&mut self, address: u16, value: u8) {
        match address {
            0xFF25 => self.panning = Panning(value),
            _ => self.master_volume = MasterVolume(value),
        }
    }
}
