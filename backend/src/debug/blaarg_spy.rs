use crate::memory::mmu::{MemoryHandler, MemoryRead, MemoryWrite, Mmu};
pub struct BlaargSpy();

impl MemoryHandler for BlaargSpy {
    fn read(&self, _mmu: &crate::memory::mmu::Mmu, _address: u16) -> MemoryRead {
        MemoryRead::Pass
    }

    fn write(&mut self, mmu: &Mmu, address: u16, value: u8) -> MemoryWrite {
        if address == 0xA000 {
            let previous = mmu.read(address);
            if previous == 0x80 {
                let mut result = String::with_capacity(100);
                let mut value = value;
                let mut addr = address;
                while value != 0 {
                    result.push(value as char);
                    addr += 1;
                    value = mmu.read(addr);
                }
                println!("Test result: {}", result);
            }
        }
        MemoryWrite::Pass
    }
}
