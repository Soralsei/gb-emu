use super::interrupt::InterruptRequest;
use crate::memory::mmu::{MemoryHandler, MemoryRead, MemoryWrite};

const DIV_CLOCKS: u16 = 256;
const CLOCKS: [u16; 4] = [
    1024,
    16,
    64,
    256,
];

pub struct Timer {
    interrupt_request: InterruptRequest,
    div: u8,  // address 0xFF04
    tima: u8, // address 0xFF05
    tma: u8,  // address 0xFF06
    tac: u8,  // address 0xFF07
    div_clocks: u16,
    tima_clocks: u32,
    overflowed: bool,
}

impl Timer {
    pub fn new(interrupt_request: InterruptRequest) -> Self {
        Self {
            interrupt_request,
            div: 0,
            tima: 0,
            tma: 0,
            tac: 0,
            div_clocks: 0,
            tima_clocks: 0,
            overflowed: false,
        }
    }
    pub fn step(&mut self, elapsed_cycles: u16) {
        // Handle divider timer
        self.div_clocks += elapsed_cycles;
        if self.div_clocks >= DIV_CLOCKS {
            self.div = self.div.wrapping_add(1);
            self.div_clocks -= DIV_CLOCKS;
        }
        if self.overflowed {
            self.tima = self.tma;
            self.overflowed = false;
            self.interrupt_request.timer(true);
        }
        // Check if TIMA is enabled
        if self.tac & 0b100 == 0 {
            return;
        }

        self.tima_clocks += elapsed_cycles as u32;
        let frequency = CLOCKS[(self.tac & 0b011) as usize] as u32;
        while self.tima_clocks >= frequency{
            let (tima, overflowed) = self.tima.overflowing_add(1);
            self.tima = tima;
            self.overflowed |= overflowed;
            self.tima_clocks -= frequency;
        }
    }
}

impl MemoryHandler for Timer {
    fn read(&self, _: &crate::memory::mmu::Mmu, address: u16) -> crate::memory::mmu::MemoryRead {
        match address {
            0xFF04 => MemoryRead::Replace(self.div),
            0xFF05 => MemoryRead::Replace(self.tima),
            0xFF06 => MemoryRead::Replace(self.tma),
            0xFF07 => MemoryRead::Replace(self.tac),
            _ => MemoryRead::Pass,
        }
    }

    fn write(
        &mut self,
        _: &crate::memory::mmu::Mmu,
        address: u16,
        value: u8,
    ) -> crate::memory::mmu::MemoryWrite {
        match address {
            0xFF04 => {
                self.div = 0;
                return MemoryWrite::Replace(0);
            },
            0xFF05 => self.tima = value,
            0xFF06 => self.tma = value,
            0xFF07 => {
                let old = self.tac;
                self.tac = value & 0b111;
                if self.tac & 0b100 != 0 && old & 0b100 != 0 {
                    self.tima_clocks = 0;
                }
            },
            _ => {}
        }
        MemoryWrite::Pass
    }
}
