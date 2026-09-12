use std::{cell::Cell, rc::Rc};

use crate::clock::{Clock, M_CYCLE};
use crate::memory::mmu::Mmu;

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum Bus {
    External = 0,
    Video = 1,
    Oam = 2,
}

pub struct BusController {
    slots: [Cell<Option<u8>>; 3], // indexed by Bus
}

impl BusController {
    pub fn new() -> Self {
        Self {
            slots: [const { Cell::new(None) }; 3],
        }
    }

    pub fn conflict(&self, address: u16) -> Option<u8> {
        match BusController::bus_for(address) {
            Some(bus) => self.slots[bus as usize].get(),
            None => None,
        }
    }

    fn bus_for(address: u16) -> Option<Bus> {
        match address {
            0xFE00..=0xFE9F => Some(Bus::Oam),
            0x8000..=0x9FFF => Some(Bus::Video),
            0x0000..=0x7FFF | 0xA000..=0xFDFF => Some(Bus::External),
            _ => None,
        }
    }

    pub fn seize(&self, bus: Bus, conflict_value: u8) {
        self.slots[bus as usize].set(Some(conflict_value));
    }

    pub fn seize_bus_of(&self, address: u16, conflict_value: u8) {
        if let Some(bus) = BusController::bus_for(address) {
            self.seize(bus, conflict_value);
        }
    }

    pub fn release(&self, bus: Bus) {
        self.slots[bus as usize].set(None);
    }

    pub fn release_bus_for(&self, address: u16) {
        if let Some(bus) = BusController::bus_for(address) {
            self.release(bus);
        }
    }
}

/// The CPU's view of the bus. A bus access here costs a machine cycle and loses
/// to whoever else is driving that bus.
///
/// Clocked devices hold the `Rc<Mmu>` directly and use `peek`/`poke`, which is
/// what keeps them out of their own arbitration: OAM DMA must not be blocked by
/// the OAM lock it holds itself. `Mmu` deliberately has no ticking accessor, so
/// taking the arbitrated path by accident is not possible.
pub struct CpuBus {
    mmu: Rc<Mmu>,
    bus_controller: Rc<BusController>,
    clock: Rc<Clock>,
}

impl CpuBus {
    pub fn new(mmu: Rc<Mmu>, bus_controller: Rc<BusController>, clock: Rc<Clock>) -> Self {
        Self {
            mmu,
            bus_controller,
            clock,
        }
    }

    pub fn read(&self, address: u16) -> u8 {
        self.clock.tick(M_CYCLE);
        match self.bus_controller.conflict(address) {
            // Whoever owns the bus is driving it; the CPU sees their value.
            Some(conflict) => conflict,
            None => self.mmu.peek(address),
        }
    }

    pub fn write(&self, address: u16, value: u8) {
        self.clock.tick(M_CYCLE);
        if self.bus_controller.conflict(address).is_none() {
            self.mmu.poke(address, value);
        }
    }

    /// Inspect without a bus cycle and without arbitration, for the places the
    /// CPU reads state rather than driving the bus (STOP checking IE/IF/JOYP).
    pub fn peek(&self, address: u16) -> u8 {
        self.mmu.peek(address)
    }
}
