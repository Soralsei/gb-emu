use super::interrupt::InterruptRequest;
use crate::{
    clock::{Clocked, M_CYCLE},
    is_bit_set,
    memory::mmu::{MemoryHandler, MemoryRead, MemoryWrite},
};
const TAC_ENABLE: u8 = 2;
const TIMA_PERIODS: [u16; 4] = [1024, 16, 64, 256];

pub struct Timer {
    interrupt_request: InterruptRequest,
    counter: u16, // DIV register is its high byte
    tima: u8,     // address 0xFF05
    tma: u8,      // address 0xFF06
    tac: u8,      // address 0xFF07
    overflowed: bool,
    is_cgb: bool,
}

impl Timer {
    pub fn new(interrupt_request: InterruptRequest, is_cgb: bool) -> Self {
        Self {
            interrupt_request,
            counter: 0,
            tima: 0,
            tma: 0,
            tac: 0,
            overflowed: false,
            is_cgb,
        }
    }

    fn timer_enabled(&self) -> bool {
        is_bit_set!(self.tac, TAC_ENABLE)
    }

    fn is_selected_clock_bit_set(&self) -> bool {
        let bit = TIMA_PERIODS[(self.tac & 0b11) as usize] >> 1;
        self.counter & bit != 0
    }

    fn state_change(&mut self, counter: u16, tac: u8) {
        let previous_active = self.timer_enabled();
        let previous_selected_set = self.is_selected_clock_bit_set();

        self.counter = counter;
        self.tac = tac;

        let current_active = self.timer_enabled();
        let current_selected_set = self.is_selected_clock_bit_set();

        // Changing which bit of the system counter is selected (by changing the “Clock select”
        // bits of TAC) from a bit currently set to another that is currently unset, will send
        // a “Timer tick” pulse.
        let selected_edge_falling =
            previous_selected_set && !current_selected_set && current_active && previous_active;
        // On monochrome consoles, disabling the timer if the currently selected bit is set, will
        // send a “Timer tick” once. This does not happen on Color models.
        let dmg_set_disabled =
            !self.is_cgb && current_selected_set && !current_active && previous_active;
        // On CGB, a write to TAC that enables the timer while the current selected counter bit is
        // set will sometimes send a "Timer tick" (not always, depends on the individual console,
        // here, we do it but not required)
        let cgb_set_enabled =
            self.is_cgb && current_selected_set && !previous_active && current_active;

        if selected_edge_falling || dmg_set_disabled || cgb_set_enabled {
            let (tima, overflow) = self.tima.overflowing_add(1);
            self.tima = tima;
            self.overflowed |= overflow;
        }
    }
}

impl MemoryHandler for Timer {
    fn read(&self, _: &crate::memory::mmu::Mmu, address: u16) -> crate::memory::mmu::MemoryRead {
        match address {
            0xFF04 => MemoryRead::Replace((self.counter >> 8) as u8),
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
                self.state_change(0, self.tac);
                return MemoryWrite::Block;
            }
            0xFF05 => {
                self.tima = value;
                self.overflowed = false;
            }
            0xFF06 => self.tma = value,
            0xFF07 => {
                self.state_change(self.counter, value & 0b111);
            }
            _ => {}
        }
        MemoryWrite::Pass
    }
}

impl Clocked for Timer {
    fn step(&mut self, elapsed_cycles: u16) {
        for _ in 0..elapsed_cycles / M_CYCLE {
            if self.overflowed {
                self.tima = self.tma;
                self.overflowed = false;
                self.interrupt_request.timer(true);
            }
            self.state_change(self.counter.wrapping_add(M_CYCLE), self.tac);
        }
    }

    fn stop(&mut self) {
        self.counter = 0;
    }
}
