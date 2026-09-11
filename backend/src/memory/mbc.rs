#![allow(unused)]
use std::cell::{Cell, RefCell};

use crate::{is_bit_set, util::bit_operations::addressing_number_of_bits};

use super::mmu::{MemoryHandler, MemoryRead, MemoryWrite};
use core::fmt;

const CGB: u8 = 7;
const CGB_ONLY: u8 = 6;
const ROM_SIZE: usize = 32 * 1024;
const ROM_BANK_SIZE: usize = 16 * 1024;
const RAM_BANK_SIZE: usize = 8 * 1024;
const MAX_ROM_SIZE_NORMAL: usize = 512 * 1024;

trait MemoryBank {
    fn read(&self, address: u16) -> MemoryRead;
    fn write(&mut self, address: u16, value: u8) -> MemoryWrite;
}

enum MbcType {
    MbcNone(MbcNone),
    Mbc1(Mbc1),
    Mbc2(MbcNone),
    Mbc3(MbcNone),
    Mbc5(MbcNone),
    Mbc6(MbcNone),
    Mbc7(MbcNone),
}

impl MbcType {
    pub fn new(code: u8, rom: Vec<u8>, rom_size: usize, ram_size: usize) -> MbcType {
        match code {
            0x00 => MbcType::MbcNone(MbcNone { rom }),
            0x01 => MbcType::Mbc1(Mbc1::new(rom, ram_size)),
            _ => unimplemented!("Mbc type 0x{:02X} not yet implemented", code),
        }
    }
}

impl MemoryBank for MbcType {
    fn read(&self, address: u16) -> MemoryRead {
        match self {
            MbcType::MbcNone(mbc) => mbc.read(address),
            MbcType::Mbc1(mbc) => mbc.read(address),
            MbcType::Mbc2(_) => todo!(),
            MbcType::Mbc3(_) => todo!(),
            MbcType::Mbc5(_) => todo!(),
            MbcType::Mbc6(_) => todo!(),
            MbcType::Mbc7(_) => todo!(),
        }
    }

    fn write(&mut self, address: u16, value: u8) -> MemoryWrite {
        match self {
            MbcType::MbcNone(mbc) => mbc.write(address, value),
            MbcType::Mbc1(mbc) => mbc.write(address, value),
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

impl MemoryBank for MbcNone {
    fn read(&self, address: u16) -> MemoryRead {
        if address <= 0x7FFF {
            return MemoryRead::Replace(self.rom[address as usize]);
        }
        MemoryRead::Pass
    }

    fn write(&mut self, address: u16, value: u8) -> MemoryWrite {
        match address {
            0..=0x7FFF => MemoryWrite::Block,
            0xA000..=0xBFFF => MemoryWrite::Pass,
            _ => unreachable!("Invalid memory write at address 0x{:04X}", address),
        }
    }
}

struct Mbc1 {
    rom: Vec<u8>,
    ram: Vec<u8>,
    // Registers
    ram_enable: bool, // written at 0x
    rom_bank_number: u8,
    ram_bank_number: u8,
    advanced_mode: bool,
}

impl Mbc1 {
    pub fn new(rom: Vec<u8>, ram_size: usize) -> Self {
        Self {
            rom,
            ram: vec![0u8; ram_size],
            ram_enable: false,
            rom_bank_number: 0x00,
            ram_bank_number: 0x00,
            advanced_mode: false,
        }
    }

    fn maybe_one_bank(bank_number: u8) -> u8 {
        if bank_number & 0b11111 == 0x00 {
            0x01
        } else {
            bank_number
        }
    }

    fn has_ram(&self) -> bool {
        self.ram.len() > 0
    }

    fn rom_needs_extended_banking(&self) -> bool {
        self.rom.len() > MAX_ROM_SIZE_NORMAL
    }

    fn get_bank_num_mask(&self) -> usize {
        let num_banks: usize = (self.rom.len() as f32 / ROM_BANK_SIZE as f32).ceil() as usize;
        let num_bits = addressing_number_of_bits(num_banks);
        // Should never occur, but just to be safe
        // avoids overflow
        let usize_bits = size_of::<usize>() * 8;
        if (num_bits >= usize_bits) {
            return !0usize;
        }
        (1 << num_bits) - 1
    }
}

impl MemoryBank for Mbc1 {
    fn read(&self, address: u16) -> MemoryRead {
        match address {
            0x0000..=0x3FFF => {
                // Should be 0b0[ram_bank_number & 0b11]00000
                // ex: ram_bank_number = 0b10 => bank_number = 0b01000000
                // bank_number can only ever be in the set = {0x00, 0x20, 0x40, 0x60}
                let bank_number: usize = if self.advanced_mode && self.rom_needs_extended_banking()
                {
                    (self.ram_bank_number & 0b11) << 5
                } else {
                    0
                } as usize;
                // bits 20 and 19 => bank_number, bits 18-14 => 0, bits 13-0 => address <= 0x3FFF
                let bank_addr: usize = bank_number * ROM_BANK_SIZE | (address as usize);
                MemoryRead::Replace(self.rom[bank_addr])
            }
            0x4000..=0x7FFF => {
                let bank_mask = self.get_bank_num_mask() as u8;
                let corrected_bank1 = Mbc1::maybe_one_bank(self.rom_bank_number) & bank_mask;
                let bank2_number: u8 = if self.rom_needs_extended_banking() {
                    (self.ram_bank_number & 0b11) << 5
                } else {
                    0
                };

                let mut bank_number = bank2_number | corrected_bank1;
                // Get the address inside the selected bank
                // strictly equivalent to bank_number * ROM_BANK_SIZE + (address - 0x3FFF)
                let bank_addr: usize =
                    bank_number as usize * ROM_BANK_SIZE | (address & 0x3FFF) as usize;

                MemoryRead::Replace(self.rom[bank_addr])
            }
            0xA000..=0xBFFF => {
                // Reads from disabled ram return open-bus values
                if !self.ram_enable {
                    return MemoryRead::Replace(0xFF);
                }
                if self.rom_needs_extended_banking() {
                    return MemoryRead::Replace(self.ram[address as usize]);
                }
                let bank = if self.advanced_mode {
                    self.ram_bank_number & 0b11
                } else {
                    0
                };
                // Same here, get address inside ram bank
                // equivalent to bank_number * RAM_BANK_SIZE + (address - 0xA000)
                let ram_addr = bank as usize * RAM_BANK_SIZE | (address & 0xA000) as usize;
                MemoryRead::Replace(self.ram[ram_addr])
            }
            _ => unreachable!("Invalid memory write at address 0x{:04X}", address),
        }
    }

    fn write(&mut self, address: u16, value: u8) -> MemoryWrite {
        match address {
            0x0000..=0x1FFF => {
                self.ram_enable = value & 0b1111 == 0xA;
                MemoryWrite::Block
            }
            0x2000..=0x3FFF => {
                self.rom_bank_number = value & 0b11111;
                eprintln!(
                    "Writing value 0x{:02X} => 0x{:02X} to BANK1 register",
                    value, self.rom_bank_number
                );
                MemoryWrite::Block
            }
            0x4000..=0x5FFF => {
                self.ram_bank_number = value & 0b11;
                return MemoryWrite::Block;
            }
            0x6000..=0x7FFF => {
                self.advanced_mode = value & 0b1 != 0;
                MemoryWrite::Block
            }
            0xA000..=0xBFFF => todo!(),
            _ => unreachable!("Invalid memory write at address 0x{:04X}", address),
        }
    }
}

pub struct Cartridge {
    title: String,
    cgb: bool,
    cgb_only: bool,
    /// Banking registers are the only mutable cartridge state, and nothing
    /// re-enters the cartridge, so one cell here keeps `MemoryBank` on `&mut self`.
    mbc: RefCell<MbcType>,
    rom_size: usize,
    ram_size: usize,
}

fn checksum(rom: &[u8]) {
    println!("Should verify cartridge checksum");
}

impl Cartridge {
    pub fn new(rom: Vec<u8>) -> Self {
        let title = String::from_utf8_lossy(&rom[0x134..=0x142]);
        let mbc_type = rom[0x147];
        let rom_size = ROM_SIZE * (1 << rom[0x148]);
        let ram_size = match rom[0x149] {
            0x02 => RAM_BANK_SIZE,
            0x03 => RAM_BANK_SIZE * 4,
            0x04 => RAM_BANK_SIZE * 16,
            0x05 => RAM_BANK_SIZE * 8,
            _ => 0,
        };
        Self {
            title: title.to_string(),
            cgb: is_bit_set!(rom[0x143], CGB),
            cgb_only: is_bit_set!(rom[0x143], CGB_ONLY),
            mbc: RefCell::new(MbcType::new(mbc_type, rom, rom_size, ram_size)),
            rom_size,
            ram_size,
        }
    }

    pub fn is_cgb_only(&self) -> bool {
        self.cgb_only
    }
}

impl MemoryHandler for Cartridge {
    fn read(&self, mmu: &super::mmu::Mmu, address: u16) -> MemoryRead {
        self.mbc.borrow().read(address)
    }

    fn write(&self, mmu: &super::mmu::Mmu, address: u16, value: u8) -> MemoryWrite {
        self.mbc.borrow_mut().write(address, value)
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
            self.mbc.borrow(),
            self.cgb,
            self.cgb_only,
            ram_size,
            self.rom_size
        )
    }
}

pub struct Mbc {
    cart: Cartridge,
    boot_rom: Vec<u8>,
    boot_rom_enabled: Cell<bool>,
}

impl Mbc {
    pub fn new(boot_rom: Option<Vec<u8>>, rom: Vec<u8>) -> Self {
        let cart = Cartridge::new(rom);
        println!("{}", cart);

        match boot_rom {
            Some(boot_rom) => Self {
                cart,
                boot_rom: boot_rom,
                boot_rom_enabled: Cell::new(true),
            },
            None => Self {
                cart,
                boot_rom: Vec::with_capacity(0),
                boot_rom_enabled: Cell::new(false),
            },
        }
    }

    pub fn cartridge(&self) -> &Cartridge {
        &self.cart
    }

    #[inline]
    fn in_boot_rom(&self, address: u16) -> bool {
        address < 0x100 || (self.boot_rom.len() == 0x900 && address >= 0x200 && address < 0x900)
    }
}

impl MemoryHandler for Mbc {
    fn read(&self, mmu: &super::mmu::Mmu, address: u16) -> MemoryRead {
        if self.boot_rom_enabled.get() && self.in_boot_rom(address) {
            return MemoryRead::Replace(self.boot_rom[address as usize]);
        }
        self.cart.read(mmu, address)
    }

    fn write(&self, mmu: &super::mmu::Mmu, address: u16, value: u8) -> MemoryWrite {
        if self.boot_rom_enabled.get() && self.in_boot_rom(address) {
            eprintln!("Write to boot rom detected ?!");
            return MemoryWrite::Block;
        } else if address == 0xFF50 {
            self.boot_rom_enabled.set(false);
            return MemoryWrite::Block;
        }
        self.cart.write(mmu, address, value)
    }
}
