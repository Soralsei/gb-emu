use std::{collections::BTreeMap, rc::Rc};

use crate::clock::{Clock, M_CYCLE};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryRead {
    Replace(u8),
    Pass,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryWrite {
    Replace(u8),
    Pass,
    Block,
}

/// A memory-mapped device. Handlers are shared (`Rc`) and keep their own
/// interior mutability, so a bus access never needs a mutable borrow of the
/// device and cannot conflict with that device being stepped by the clock.
pub trait MemoryHandler {
    fn read(&self, mmu: &Mmu, address: u16) -> MemoryRead;
    fn write(&self, mmu: &Mmu, address: u16, value: u8) -> MemoryWrite;
}

#[allow(unused)]
pub struct Mmu {
    pub handlers: BTreeMap<u16, Vec<Rc<dyn MemoryHandler>>>,
    clock: Rc<Clock>,
    memory: [u8; 0x10000],
    pub interrupts_enable: u8,
    pub interrupts_flags: u8,
}

impl Mmu {
    pub fn new(clock: Rc<Clock>) -> Mmu {
        Mmu {
            handlers: BTreeMap::new(),
            clock,
            memory: [0; 0x10000],
            interrupts_enable: 0,
            interrupts_flags: 0,
        }
    }

    pub fn add_handler(&mut self, address_range: (u16, u16), handler: Rc<dyn MemoryHandler>) {
        for address in address_range.0..=address_range.1 {
            self.handlers
                .entry(address)
                .or_default()
                .push(handler.clone());
        }
    }

    /// Bus read, as performed by the CPU: costs a machine cycle.
    pub fn read(&self, addr: u16) -> u8 {
        self.clock.tick(M_CYCLE);
        self.peek(addr)
    }

    /// Observe memory without advancing the clock. For handlers and debug
    /// tooling, which are not the CPU driving the bus.
    pub fn peek(&self, addr: u16) -> u8 {
        if let Some(handlers) = self.handlers.get(&addr) {
            for handler in handlers {
                match handler.read(self, addr) {
                    MemoryRead::Replace(value) => return value,
                    MemoryRead::Pass => (),
                }
            }
        }

        match addr {
            // echo ram read
            0xE000..=0xFDFF => self.memory[(addr - 0x2000) as usize],
            // normal ram read
            _ => self.memory[addr as usize],
        }
    }

    /// Write memory without advancing the clock. For handlers and debug
    /// tooling, which are not the CPU driving the bus
    pub fn poke(&mut self, addr: u16, value: u8) {
        // Resolve the handler chain first: handlers only ever see `&Mmu`, so the
        // backing store is written after their borrow of `self` has ended.
        let outcome = match self.handlers.get(&addr) {
            Some(handlers) => {
                let mut outcome = MemoryWrite::Pass;
                for handler in handlers {
                    match handler.write(self, addr, value) {
                        MemoryWrite::Pass => (),
                        decided => {
                            outcome = decided;
                            break;
                        }
                    }
                }
                outcome
            }
            None => MemoryWrite::Pass,
        };

        let value = match outcome {
            MemoryWrite::Block => return,
            MemoryWrite::Replace(replacement) => replacement,
            MemoryWrite::Pass => value,
        };

        match addr {
            // echo ram write
            0xE000..=0xFDFF => self.memory[(addr - 0x2000) as usize] = value,
            // normal ram write
            _ => self.memory[addr as usize] = value,
        }
    }

    /// Bus write, as performed by the CPU: costs a machine cycle.
    pub fn write(&mut self, addr: u16, value: u8) {
        self.clock.tick(M_CYCLE);
        self.poke(addr, value);
    }
}
