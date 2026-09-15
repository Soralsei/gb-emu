use crate::is_bit_set;

#[derive(Debug, Clone, Copy, Default)]
#[repr(u8)]
pub enum ObjectPriority {
    #[default]
    None = 0,
    BgWindow = 1,
}

impl From<bool> for ObjectPriority {
    fn from(value: bool) -> Self {
        if value {
            return Self::BgWindow;
        }
        Self::None
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum DMGPalette {
    OBP0 = 0,
    OBP1 = 1,
}

impl From<bool> for DMGPalette {
    fn from(value: bool) -> Self {
        if value {
            return Self::OBP1;
        }
        Self::OBP0
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum CGBBank {
    Bank0 = 0,
    Bank1 = 1,
}

impl From<bool> for CGBBank {
    fn from(value: bool) -> Self {
        if value {
            return Self::Bank1;
        }
        Self::Bank0
    }
}

pub struct ObjectFlags(u8);
impl ObjectFlags {
    pub fn priority(&self) -> ObjectPriority {
        is_bit_set!(self.0, 7).into()
    }

    pub fn y_flip(&self) -> bool {
        is_bit_set!(self.0, 6)
    }

    pub fn x_flip(&self) -> bool {
        is_bit_set!(self.0, 5)
    }

    pub fn dmg_palette(&self) -> DMGPalette {
        is_bit_set!(self.0, 4).into()
    }

    pub fn bank(&self) -> CGBBank {
        is_bit_set!(self.0, 3).into()
    }

    pub fn cgb_palette(&self) -> u8 {
        self.0 & 0x07
    }
}

pub struct ObjectAttribute(pub u32);

impl ObjectAttribute {
    // Byte 0
    pub fn y(&self) -> u8 {
        (self.0 & 0xFF) as u8
    }

    // Byte 1
    pub fn x(&self) -> u8 {
        ((self.0 >> 8) & 0xFF) as u8
    }

    // Byte 2
    pub fn tile_index(&self) -> u8 {
        ((self.0 >> 16) & 0xFF) as u8
    }

    // Byte 3
    pub fn flags(&self) -> ObjectFlags {
        ObjectFlags(((self.0 >> 24) & 0xFF) as u8)
    }
}
