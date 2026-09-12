#![allow(dead_code)]

use std::{cell::Cell, rc::Rc};

use crate::{
    clock::{Clocked, M_CYCLE},
    memory::{
        bus::{Bus, BusController},
        mmu::{MemoryHandler, MemoryRead, MemoryWrite, Mmu},
    },
};

#[derive(Clone, Copy)]
enum OAMTransferState {
    Idle,
    InProgress,
}

pub struct DMAController {
    mmu: Rc<Mmu>,
    bus_controller: Rc<BusController>,
    oam_transfer_state: Cell<OAMTransferState>,
    current_index: Cell<u8>,
    src_addr_reg: Cell<u8>,
}

impl DMAController {
    pub fn new(mmu: Rc<Mmu>, bus_controller: Rc<BusController>) -> Self {
        Self {
            mmu,
            bus_controller,
            oam_transfer_state: Cell::new(OAMTransferState::Idle),
            current_index: Cell::new(0),
            src_addr_reg: Cell::new(0xFF),
        }
    }
}

impl MemoryHandler for DMAController {
    fn read(&self, _: &Mmu, address: u16) -> MemoryRead {
        if address != 0xFF46 {
            unreachable!("Invalid read in DMAController at address 0x{:04X}", address)
        }
        MemoryRead::Replace(self.src_addr_reg.get())
    }

    fn write(&self, _: &Mmu, address: u16, value: u8) -> MemoryWrite {
        if address != 0xFF46 {
            unreachable!(
                "Invalid write in DMAController at address 0x:{:04X}",
                address
            )
        }

        // Release any held bus just in case this is interrupting a running transfer
        if matches!(self.oam_transfer_state.get(), OAMTransferState::InProgress) {
            self.bus_controller
                .release_bus_for((self.src_addr_reg.get() as u16) << 8);
            self.bus_controller.release(Bus::Oam);
        }

        self.src_addr_reg.set(value);
        // Will start on next clock tick, since write ticks happens before pokes
        // meaning the clock has already ticked here, and wil only tick next cycle
        self.oam_transfer_state.set(OAMTransferState::InProgress);
        self.current_index.set(0);
        MemoryWrite::Block
    }
}

impl Clocked for DMAController {
    fn step(&self, elapsed: u16) {
        match self.oam_transfer_state.get() {
            OAMTransferState::Idle => return,
            OAMTransferState::InProgress => {
                for _ in 0..(elapsed / M_CYCLE) {
                    let src_addr: u16 =
                        ((self.src_addr_reg.get() as u16) << 8) | self.current_index.get() as u16;

                    if self.current_index.get() > 0x9F {
                        self.oam_transfer_state.replace(OAMTransferState::Idle);
                        self.bus_controller.release_bus_for(src_addr);
                        self.bus_controller.release(Bus::Oam);
                        return;
                    }

                    let value = self.mmu.peek(src_addr);

                    // Idempotent
                    self.bus_controller.seize(Bus::Oam, 0xFF);
                    self.bus_controller.seize_bus_of(src_addr, value);

                    self.mmu
                        .poke(0xFE00 | self.current_index.get() as u16, value);
                    self.current_index.update(|idx| idx + 1);
                }
            }
        }
    }
}
