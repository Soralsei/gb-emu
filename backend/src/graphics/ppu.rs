use crate::cpu::interrupt::InterruptRequest;
use crate::graphics::registers::PpuRegisters;
use crate::memory::mmu::{MemoryHandler, MemoryRead, MemoryWrite, Mmu};

pub struct Ppu {
    registers: PpuRegisters,
    interrupt_request: InterruptRequest,
    clock: u32,
}

impl MemoryHandler for Ppu {
    fn read(&self, mmu: &Mmu, address: u16) -> MemoryRead {
        MemoryRead::Replace(self.registers.read(address))
    }

    fn write(&mut self, mmu: &Mmu, address: u16, value: u8) -> MemoryWrite {
        self.registers.write(address, value);
        MemoryWrite::Block
    }
}
