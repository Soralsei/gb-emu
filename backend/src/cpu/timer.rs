use std::{cell::RefCell, rc::Rc};

use super::interrupt::InterruptRequest;
use crate::{
    clock::{Clock, Clocked, M_CYCLE},
    is_bit_set,
    memory::mmu::{MemoryHandler, MemoryRead, MemoryWrite},
};
const TAC_ENABLE: u8 = 2;
const TIMA_PERIODS: [u16; 4] = [1024, 16, 64, 256];

/// The timer is only ever touched by its own `step` and its own register
/// handler, which never nest, so one cell at the device boundary is enough and
/// all of the logic below keeps plain `&mut self`.
pub struct Timer {
    state: RefCell<TimerState>,
}

impl Timer {
    pub fn new(interrupt_request: InterruptRequest, clock: Rc<Clock>, is_cgb: bool) -> Self {
        Self {
            state: RefCell::new(TimerState {
                clock,
                interrupt_request,
                tima: 0,
                tma: 0,
                tac: 0,
                overflowed: false,
                is_cgb,
            }),
        }
    }
}

struct TimerState {
    clock: Rc<Clock>,
    interrupt_request: InterruptRequest,
    tima: u8, // address 0xFF05
    tma: u8,  // address 0xFF06
    tac: u8,  // address 0xFF07
    overflowed: bool,
    is_cgb: bool,
}

impl TimerState {
    fn enabled(tac: u8) -> bool {
        is_bit_set!(tac, TAC_ENABLE)
    }

    fn selected_bit(counter: u16, tac: u8) -> bool {
        counter & (TIMA_PERIODS[(tac & 0b11) as usize] >> 1) != 0
    }

    /// One M-cycle of counter advance: the pending TMA reload, then a falling
    /// edge of the selected bit between `counter - M_CYCLE` and `counter`.
    fn advance_to(&mut self, counter: u16) {
        if self.overflowed {
            self.tima = self.tma;
            self.overflowed = false;
            self.interrupt_request.timer(true);
        }
        self.state_change(counter.wrapping_sub(M_CYCLE), counter, self.tac);
    }

    /// Sample the selected bit either side of a change to the counter or TAC.
    /// The counter is passed in rather than read twice from the clock: a write
    /// changes it under the detector, so both samples must be explicit.
    fn state_change(&mut self, previous_counter: u16, next_counter: u16, next_tac: u8) {
        let previous_active = Self::enabled(self.tac);
        let previous_selected_set = Self::selected_bit(previous_counter, self.tac);

        self.tac = next_tac;

        let current_active = Self::enabled(self.tac);
        let current_selected_set = Self::selected_bit(next_counter, self.tac);

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
        self.state.borrow().read(address)
    }

    fn write(
        &self,
        _: &crate::memory::mmu::Mmu,
        address: u16,
        value: u8,
    ) -> crate::memory::mmu::MemoryWrite {
        self.state.borrow_mut().write(address, value)
    }
}

impl Clocked for Timer {
    fn step(&self, elapsed_cycles: u16) {
        let mut state = self.state.borrow_mut();
        // The window this tick covers, derived rather than stored: a stored
        // previous sample would go stale on a DIV reset, which is the
        // notification the clock no longer broadcasts.
        let end = state.clock.div();
        let start = end.wrapping_sub(elapsed_cycles);
        for k in (M_CYCLE..=elapsed_cycles).step_by(M_CYCLE as usize) {
            state.advance_to(start.wrapping_add(k));
        }
    }
}

impl TimerState {
    fn read(&self, address: u16) -> MemoryRead {
        match address {
            0xFF04 => MemoryRead::Replace((self.clock.div() >> 8) as u8),
            0xFF05 => MemoryRead::Replace(self.tima),
            0xFF06 => MemoryRead::Replace(self.tma),
            0xFF07 => MemoryRead::Replace(self.tac),
            _ => MemoryRead::Pass,
        }
    }

    fn write(&mut self, address: u16, value: u8) -> MemoryWrite {
        match address {
            0xFF04 => {
                let previous = self.clock.div();
                self.clock.reset_div();
                self.state_change(previous, 0, self.tac);
                return MemoryWrite::Block;
            }
            0xFF05 => {
                self.tima = value;
                self.overflowed = false;
            }
            0xFF06 => self.tma = value,
            0xFF07 => {
                let counter = self.clock.div();
                self.state_change(counter, counter, value & 0b111);
            }
            _ => {}
        }
        MemoryWrite::Pass
    }
}
