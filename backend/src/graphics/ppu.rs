#![allow(dead_code)]
use std::cell::{Cell, RefCell};

use crate::clock::Clocked;
use crate::cpu::interrupt::InterruptRequest;
use crate::graphics::registers::PpuRegisters;
use crate::is_bit_set;
use crate::memory::mmu::{MemoryHandler, MemoryRead, MemoryWrite, Mmu};

#[repr(u8)]
enum ObjectPriority {
    BgWindow = 1,
    None = 0,
}

impl From<bool> for ObjectPriority {
    fn from(value: bool) -> Self {
        if value {
            return Self::BgWindow;
        }
        Self::None
    }
}

#[repr(u8)]
enum DMGPalette {
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

#[repr(u8)]
enum CGBBank {
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

struct ObjectFlags(u8);
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

struct ObjectAttribute(u32);

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

#[derive(Clone, Copy)]
enum OamDmaTransferState {
    Idle,
    Queued,
    InProgress,
}

pub struct Ppu {
    registers: RefCell<PpuRegisters>,

    interrupt_request: InterruptRequest,

    /// OAM lives on the PPU die, not in system RAM. `Cell<u8>` is layout- and
    /// cost-identical to `u8`, so OAM DMA can write through the bus handler
    /// while the PPU is mid-scan.
    oam: [Cell<u8>; 160],

    clock: Cell<u16>,
}

impl Ppu {
    fn object(&self, index: usize) -> ObjectAttribute {
        let base = index * 4;
        ObjectAttribute(u32::from_le_bytes([
            self.oam[base].get(),
            self.oam[base + 1].get(),
            self.oam[base + 2].get(),
            self.oam[base + 3].get(),
        ]))
    }
}

impl MemoryHandler for Ppu {
    fn read(&self, _mmu: &Mmu, address: u16) -> MemoryRead {
        match address {
            _ => MemoryRead::Replace(self.registers.borrow().read(address)),
        }
    }

    fn write(&self, _mmu: &Mmu, address: u16, value: u8) -> MemoryWrite {
        self.registers.borrow_mut().write(address, value);
        MemoryWrite::Block
    }
}

impl Clocked for Ppu {
    fn step(&self, elapsed_cycles: u16) {
        self.clock
            .set(self.clock.get().wrapping_add(elapsed_cycles));
        todo!()
    }
}
