use std::cell::RefCell;

use crate::{
    audio::{
        channel::{Channel, Noise, Square, SquareSweep, Wave},
        registers::AudioRegisters,
    },
    is_bit_set,
    memory::mmu::{MemoryHandler, MemoryRead, MemoryWrite, Mmu},
};

struct ApuState {
    is_cgb: bool,
    is_powered: bool,
    registers: AudioRegisters,
    ch4: Noise,
    ch3: Wave,
    ch2: Square,
    ch1: SquareSweep,
}

pub struct Apu {
    state: RefCell<ApuState>,
}

impl ApuState {
    fn channel(&self, chan: usize) -> &dyn Channel {
        match chan {
            0 => &self.ch1,
            1 => &self.ch2,
            2 => &self.ch3,
            _ => &self.ch4,
        }
    }

    fn read_nr52(&self) -> u8 {
        0b01110000
            | (self.is_powered as u8) << 7
            | (self.ch4.enabled() as u8) << 3
            | (self.ch3.enabled() as u8) << 2
            | (self.ch2.enabled() as u8) << 1
            | (self.ch1.enabled() as u8)
    }

    fn write_nr52(&mut self, value: u8) {
        self.is_powered = is_bit_set!(value, 7);
        // bits 0-3 are read-only and depend only on individual channel enabled state
    }
}

impl MemoryHandler for Apu {
    fn read(&self, _: &Mmu, address: u16) -> MemoryRead {
        match address {
            0xFF26 => MemoryRead::Replace(self.state.borrow().read_nr52()),
            _ => MemoryRead::Replace(self.state.borrow().registers.read(address)),
        }
    }

    fn write(&self, _: &Mmu, address: u16, value: u8) -> MemoryWrite {
        match address {
            0xFF26 => {
                // TODO: reset all other APU registers
                // ...
                self.state.borrow_mut().write_nr52(value);
            }
            _ => {
                self.state.borrow_mut().registers.write(address, value);
            }
        }
        MemoryWrite::Block
    }
}
