#![allow(dead_code)]

use std::{cell::Cell, convert::Infallible, rc::Rc};

use crate::{
    clock::{Cycles, Timeline, M_CYCLE},
    memory::{
        bus::{Bus, BusController, BusOwner},
        mmu::{MemoryHandler, MemoryRead, MemoryWrite, Mmu},
    },
};

#[derive(Clone, Copy)]
enum OAMTransferState {
    Idle,
    InProgress,
}

struct DMAState {
    pub src_addr_reg: Cell<u8>,
    pub requested: Cell<bool>,
}

impl DMAState {
    pub fn new() -> Self {
        Self {
            src_addr_reg: Cell::new(0xFF),
            requested: Cell::new(false),
        }
    }
}

pub struct DMAController {
    mmu: Rc<Mmu>,
    bus_controller: Rc<BusController>,
    state: DMAState,
}

impl DMAController {
    pub fn new(mmu: Rc<Mmu>, bus_controller: Rc<BusController>) -> Self {
        Self {
            mmu,
            bus_controller,
            state: DMAState::new(),
        }
    }

    pub async fn task(this: Rc<DMAController>, timeline: Timeline) -> Infallible {
        loop {
            while !this.state.requested.get() {
                timeline.wait(M_CYCLE as Cycles).await;
            }
            let src = (this.state.src_addr_reg.get() as u16) << 8;
            for offset in 0..0xA0u16 {
                let src_addr = src | offset;
                timeline.wait(M_CYCLE as Cycles).await;
                // resume checkpoint for save states go here
                // ...
                let value = this.mmu.peek(src | offset);

                // Idempotent
                this.bus_controller.seize(Bus::Oam, BusOwner::Dma, 0xFF);
                this.bus_controller
                    .seize_bus_of(src_addr, BusOwner::Dma, value);

                this.mmu.poke(0xFE00 | offset, value);
            }
            this.bus_controller.release_bus_for(src, BusOwner::Dma);
            this.bus_controller.release(Bus::Oam, BusOwner::Dma);
            this.state.requested.set(false);
        }
    }
}

impl MemoryHandler for DMAController {
    fn read(&self, _: &Mmu, address: u16) -> MemoryRead {
        if address != 0xFF46 {
            unreachable!("Invalid read in DMAController at address 0x{:04X}", address)
        }
        MemoryRead::Replace(self.state.src_addr_reg.get())
    }

    fn write(&self, _: &Mmu, address: u16, value: u8) -> MemoryWrite {
        if address != 0xFF46 {
            unreachable!(
                "Invalid write in DMAController at address 0x:{:04X}",
                address
            )
        }

        // Release any held bus just in case this is interrupting a running transfer
        self.bus_controller
            .release_bus_for((self.state.src_addr_reg.get() as u16) << 8, BusOwner::Dma);
        self.bus_controller.release(Bus::Oam, BusOwner::Dma);

        self.state.src_addr_reg.set(value);
        self.state.requested.set(true);
        MemoryWrite::Block
    }
}
