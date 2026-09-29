use std::cell::RefCell;

use crate::{
    audio::{
        channel::{Noise, Square, Wave},
        registers::AudioRegisters,
    },
    memory::mmu::{MemoryHandler, MemoryRead, MemoryWrite, Mmu},
};

struct ApuState {
    is_cgb: bool,
    is_powered: bool,
    registers: AudioRegisters,
    ch4: Noise,
    ch3: Wave,
    ch2: Square,
    ch1: Square,
}

pub struct Apu {
    state: RefCell<ApuState>,
}

impl MemoryHandler for Apu {
    fn read(&self, _: &Mmu, address: u16) -> MemoryRead {
        MemoryRead::Replace(self.state.borrow().registers.read(address))
    }

    fn write(&self, _: &Mmu, address: u16, value: u8) -> MemoryWrite {
        self.state.borrow_mut().registers.write(address, value);
        MemoryWrite::Block
    }
}
