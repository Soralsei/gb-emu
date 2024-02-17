#![allow(unused)]
use crate::is_bit_set;

use super::mmu::{MemoryHandler, MemoryRead, MemoryWrite};
use core::fmt;

const CGB: u8 = 7;
const CGB_ONLY: u8 = 6;

trait MemoryBank {
    fn read(&self, address: u16) -> MemoryRead;
    fn write(&self, address: u16, value: u8) -> MemoryWrite;
}

enum MbcType {
    MbcNone(MbcNone),
    Mbc1(MbcNone),
    Mbc2(MbcNone),
    Mbc3(MbcNone),
    Mbc5(MbcNone),
    Mbc6(MbcNone),
    Mbc7(MbcNone),
}

impl MbcType {
    pub fn new(code: u8, rom: Vec<u8>) -> MbcType {
        match code {
            0x00 => MbcType::MbcNone(MbcNone::new(rom)),
            _ => unimplemented!("Mbc type not yet implemented"),
        }
    }
}

impl MemoryBank for MbcType {
    fn read(&self, address: u16) -> MemoryRead {
        match self {
            MbcType::MbcNone(mbc) => mbc.read(address),
            MbcType::Mbc1(_) => todo!(),
            MbcType::Mbc2(_) => todo!(),
            MbcType::Mbc3(_) => todo!(),
            MbcType::Mbc5(_) => todo!(),
            MbcType::Mbc6(_) => todo!(),
            MbcType::Mbc7(_) => todo!(),
        }
    }

    fn write(&self, address: u16, value: u8) -> MemoryWrite {
        match self {
            MbcType::MbcNone(mbc) => mbc.write(address, value),
            MbcType::Mbc1(_) => todo!(),
            MbcType::Mbc2(_) => todo!(),
            MbcType::Mbc3(_) => todo!(),
            MbcType::Mbc5(_) => todo!(),
            MbcType::Mbc6(_) => todo!(),
            MbcType::Mbc7(_) => todo!(),
        }
    }
}

impl fmt::Display for MbcType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let display_name = match self {
            MbcType::MbcNone(_) => "None",
            MbcType::Mbc1(_) => "MBC1",
            MbcType::Mbc2(_) => "MBC2",
            MbcType::Mbc3(_) => "MBC3",
            MbcType::Mbc5(_) => "MBC5",
            MbcType::Mbc6(_) => "MBC6",
            MbcType::Mbc7(_) => "MBC7",
        };
        write!(f, "{}", display_name)
    }
}

struct MbcNone {
    rom: Vec<u8>,
}

impl MbcNone {
    pub fn new(rom: Vec<u8>) -> Self {
        Self { rom }
    }
}

impl MemoryBank for MbcNone {
    fn read(&self, address: u16) -> MemoryRead {
        if address <= 0x7FFF {
            return MemoryRead::Replace(self.rom[address as usize]);
        }
        MemoryRead::Pass
    }

    fn write(&self, address: u16, value: u8) -> MemoryWrite {
        match address {
            0..=0x7FFF => {
                return MemoryWrite::Block;
            }
            0xA000..=0xBFFF => return MemoryWrite::Pass,
            _ => unreachable!("Invalid memory write at address 0x{:04X}", address),
        }
    }
}

fn decode_string(data: &[u8]) -> String {
    String::from("")
}

struct Cartridge {
    title: String,
    cgb: bool,
    cgb_only: bool,
    mbc: MbcType,
    rom_size: u8,
    ram_size: u8,
}

fn checksum(rom: &[u8]) {
    println!("Should verify cartridge checksum");
}

impl Cartridge {
    pub fn new(rom: Vec<u8>) -> Self {
        let title = decode_string(&rom[0x134..=0x142]);
        let mbc_type = rom[0x147];
        let rom_size = rom[0x148];
        let ram_size = rom[0x149];
        Self {
            title,
            cgb: is_bit_set!(rom[0x143], CGB),
            cgb_only: is_bit_set!(rom[0x143], CGB_ONLY),
            mbc: MbcType::new(mbc_type, rom),
            rom_size,
            ram_size,
        }
    }
}

impl MemoryHandler for Cartridge {
    fn read(&self, mmu: &super::mmu::Mmu, address: u16) -> MemoryRead {
        self.mbc.read(address)
    }

    fn write(&mut self, mmu: &super::mmu::Mmu, address: u16, value: u8) -> MemoryWrite {
        self.mbc.write(address, value)
    }
}

impl fmt::Display for Cartridge {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ram_size = match self.ram_size {
            0x0 => "None",
            0x1 => "Unused",
            0x2 => "8 KiB",
            0x3 => "32 KiB",
            0x4 => "128 KiB",
            0x5 => "64 KiB",
            _ => "Unknown",
        };
        write!(
            f,
            "Cartridge {{
            Title : {},
            MBC: {},
            CGB : {} | CGB only : {},
            RAM size : {},
            ROM size : {} KiB
        }}",
            self.title,
            self.mbc,
            self.cgb,
            self.cgb_only,
            ram_size,
            32 * (1 << self.rom_size)
        )
    }
}

pub struct Mbc {
    cart: Cartridge,
    boot_rom: Vec<u8>,
    boot_rom_enabled: bool,
}

impl Mbc {
    pub fn new(boot_rom: Option<Vec<u8>>, rom: Vec<u8>) -> Self {
        let cart = Cartridge::new(rom);
        println!("{}", cart);

        match boot_rom {
            Some(boot_rom) => Self {
                cart,
                boot_rom: boot_rom,
                boot_rom_enabled: true,
            },
            None => Self {
                cart,
                boot_rom: Vec::with_capacity(0),
                boot_rom_enabled: false,
            },
        }
    }

    #[inline]
    fn in_boot_rom(&self, address: u16) -> bool {
        address < 0x100 || (self.boot_rom.len() == 0x900 && address >= 200 && address < 0x900)
    }
}

impl MemoryHandler for Mbc {
    fn read(&self, mmu: &super::mmu::Mmu, address: u16) -> MemoryRead {
        if self.boot_rom_enabled && self.in_boot_rom(address) {
            return MemoryRead::Replace(self.boot_rom[address as usize]);
        }
        self.cart.read(mmu, address)
    }

    fn write(&mut self, mmu: &super::mmu::Mmu, address: u16, value: u8) -> MemoryWrite {
        if self.boot_rom_enabled && self.in_boot_rom(address) {
            eprintln!("Write to boot rom detected ?!");
            return MemoryWrite::Block;
        } else if address == 0xFF50 {
            self.boot_rom_enabled = false;
            return MemoryWrite::Block;
        }
        let write = self.cart.write(mmu, address, value);
        write
    }
}
