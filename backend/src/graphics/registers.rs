use crate::is_bit_set;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum TileArea {
    #[default]
    Low = 0,
    High = 1,
}

impl From<TileArea> for u8 {
    fn from(value: TileArea) -> Self {
        value as u8
    }
}

impl From<bool> for TileArea {
    fn from(value: bool) -> Self {
        if value {
            Self::High
        } else {
            Self::Low
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ObjSize {
    #[default]
    Size8x8 = 0,
    Size8x16 = 1,
}

impl From<ObjSize> for u8 {
    fn from(value: ObjSize) -> Self {
        value as u8
    }
}

impl From<bool> for ObjSize {
    fn from(value: bool) -> Self {
        if value {
            Self::Size8x16
        } else {
            Self::Size8x8
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct LcdControl {
    pub lcd_ppu_enable: bool,           // bit 7
    pub window_tilemap_area: TileArea,  // bit 6
    pub window_enable: bool,            // bit 5
    pub bg_window_tiles_area: TileArea, // bit 4
    pub bg_tilemap_area: TileArea,      // bit 3
    pub obj_size: ObjSize,              // bit 2
    pub obj_enable: bool,               // bit 1
    pub bg_window_enable: bool,         // bit 0, has different meaning in CGB mode
}

impl From<LcdControl> for u8 {
    fn from(value: LcdControl) -> Self {
        (value.lcd_ppu_enable as u8) << 7
            | (value.window_tilemap_area as u8) << 6
            | (value.window_enable as u8) << 5
            | (value.bg_window_tiles_area as u8) << 4
            | (value.bg_tilemap_area as u8) << 3
            | (value.obj_size as u8) << 2
            | (value.obj_enable as u8) << 1
            | (value.bg_window_enable as u8)
    }
}

impl From<u8> for LcdControl {
    fn from(value: u8) -> Self {
        Self {
            lcd_ppu_enable: is_bit_set!(value, 7),
            window_tilemap_area: is_bit_set!(value, 6).into(),
            window_enable: is_bit_set!(value, 5),
            bg_window_tiles_area: is_bit_set!(value, 4).into(),
            bg_tilemap_area: is_bit_set!(value, 3).into(),
            obj_size: is_bit_set!(value, 2).into(),
            obj_enable: is_bit_set!(value, 1),
            bg_window_enable: is_bit_set!(value, 0),
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum PpuMode {
    HBlank = 0, // Mode 0
    VBlank = 1, // Mode 1
    #[default]
    OamScan = 2, // Mode 2
    Drawing = 3, // Mode 3
}

impl From<PpuMode> for u8 {
    fn from(value: PpuMode) -> Self {
        value as u8
    }
}

impl From<u8> for PpuMode {
    fn from(value: u8) -> Self {
        match value & 0b11 {
            0 => PpuMode::HBlank,
            1 => PpuMode::VBlank,
            2 => PpuMode::OamScan,
            _ => PpuMode::Drawing,
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct LcdStat {
    pub lyc_int_select: bool,   // bit 6,   R/W
    pub mode2_int_select: bool, // bit 5,   R/W
    pub mode1_int_select: bool, // bit 4,   R/W
    pub mode0_int_select: bool, // bit 3,   R/W
    pub lyc_ly_eq: bool,        // bit 2,   RO
    pub ppu_mode: PpuMode,      // bit 0-1, RO
}

impl From<LcdStat> for u8 {
    fn from(value: LcdStat) -> Self {
        0x80 // bit 7 is unused and always reads as 1
            | (value.lyc_int_select as u8) << 6
            | (value.mode2_int_select as u8) << 5
            | (value.mode1_int_select as u8) << 4
            | (value.mode0_int_select as u8) << 3
            | (value.lyc_ly_eq as u8) << 2
            | (value.ppu_mode as u8)
    }
}

impl LcdStat {
    pub fn set(&mut self, value: u8) {
        self.lyc_int_select = is_bit_set!(value, 6);
        self.mode2_int_select = is_bit_set!(value, 5);
        self.mode1_int_select = is_bit_set!(value, 4);
        self.mode0_int_select = is_bit_set!(value, 3);
        // lyc_ly_eq and ppu_mode are both read-only
        // they are only set by the PPU, never by the CPU
    }
}

#[derive(Debug, Default)]
pub struct PpuRegisters {
    lcdc: LcdControl,
    ly: u8,
    lyc: u8,
    stat: LcdStat,
    wx: u8,
    wy: u8,
    scy: u8,
    scx: u8,
    bgp: u8,
    obp0: u8,
    obp1: u8,
    oam_dma: u8,
}

impl PpuRegisters {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn read(&self, address: u16) -> u8 {
        match address {
            0xFF40 => self.lcdc.into(),
            0xFF41 => self.ly,
            0xFF42 => self.lyc,
            0xFF43 => self.stat.into(),
            0xFF44 => self.wx,
            0xFF45 => self.wy,
            0xFF46 => self.scy,
            0xFF47 => self.scx,
            0xFF48 => self.bgp,
            0xFF49 => self.obp0,
            0xFF4A => self.obp1,
            0xFF4B => self.oam_dma,
            _ => unreachable!("Ppu register at address 0x{:04X} does not exist", address),
        }
    }

    pub fn write(&mut self, address: u16, value: u8) {
        match address {
            0xFF40 => self.lcdc = value.into(),
            // LY is read-only, the PPU is the only one driving it (see set_ly)
            0xFF41 => {}
            // writing LYC re-evaluates the coincidence flag immediately
            0xFF42 => {
                self.lyc = value;
                self.stat.lyc_ly_eq = self.ly == self.lyc;
            }
            // STAT bits 0-2 are read-only, LcdStat::set only takes bits 3-6
            0xFF43 => self.stat.set(value),
            0xFF44 => self.wx = value,
            0xFF45 => self.wy = value,
            0xFF46 => self.scy = value,
            0xFF47 => self.scx = value,
            0xFF48 => self.bgp = value,
            0xFF49 => self.obp0 = value,
            0xFF4A => self.obp1 = value,
            // the actual OAM transfer is started by the Ppu memory handler
            0xFF4B => self.oam_dma = value,
            _ => unreachable!("Ppu register at address 0x{:04X} does not exist", address),
        }
    }

    /// LY and the read-only STAT bits are driven by the PPU, not by CPU writes.
    pub fn set_ly(&mut self, ly: u8) {
        self.ly = ly;
        self.stat.lyc_ly_eq = self.ly == self.lyc;
    }

    pub fn set_mode(&mut self, mode: PpuMode) {
        self.stat.ppu_mode = mode;
    }

    pub fn lcdc(&self) -> LcdControl {
        self.lcdc
    }

    pub fn stat(&self) -> LcdStat {
        self.stat
    }
}
