use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

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
    /// Indexed by address rather than keyed by it. The map is dense, so a tree
    /// walk per access buys nothing over an indexed load. Costs 1.5MB.
    handlers: RefCell<Box<[Vec<Rc<dyn MemoryHandler>>]>>,
    /// `Cell<u8>` is `repr(transparent)`, so this has the layout and access cost
    /// of `[u8; 0x10000]`. Interior mutability is what lets the bus be shared as
    /// an `Rc<Mmu>`: a clocked device (OAM DMA, the PPU fetcher) only ever holds
    /// `&Mmu`, and still has to be able to drive a write.
    memory: [Cell<u8>; 0x10000],
    pub interrupts_enable: u8,
    pub interrupts_flags: u8,
}

impl Mmu {
    pub fn new() -> Mmu {
        Mmu {
            handlers: RefCell::new(
                (0..0x10000)
                    .map(|_| Vec::with_capacity(2))
                    .collect::<Vec<_>>()
                    .into_boxed_slice(),
            ),
            memory: [const { Cell::new(0) }; 0x10000],
            interrupts_enable: 0,
            interrupts_flags: 0,
        }
    }

    /// Takes `&self` so devices can be registered after the bus is behind an
    /// `Rc`, which they have to be when a device needs a handle to the bus.
    pub fn add_handler(&self, address_range: (u16, u16), handler: Rc<dyn MemoryHandler>) {
        let mut handlers = self.handlers.borrow_mut();
        for address in address_range.0..=address_range.1 {
            handlers[address as usize].push(handler.clone());
        }
    }

    /// Read a byte. The address map only: no machine cycle, no arbitration.
    /// The CPU reaches memory through `CpuBus`, which adds both.
    pub fn peek(&self, addr: u16) -> u8 {
        // Held while handlers run: one may come back through `peek` (the blaarg
        // spy does), and nested shared borrows are fine. Mutating the table from
        // inside dispatch is not — see `add_handler`.
        let handlers = self.handlers.borrow();
        for handler in handlers[addr as usize].iter() {
            match handler.read(self, addr) {
                MemoryRead::Replace(value) => return value,
                MemoryRead::Pass => (),
            }
        }

        match addr {
            // echo ram read
            0xE000..=0xFDFF => self.memory[(addr - 0x2000) as usize].get(),
            // normal ram read
            _ => self.memory[addr as usize].get(),
        }
    }

    /// Write a byte. The address map only: no machine cycle, no arbitration.
    /// This is the path a clocked device uses to drive the bus it owns.
    pub fn poke(&self, addr: u16, value: u8) {
        let outcome = {
            let handlers = self.handlers.borrow();
            let mut outcome = MemoryWrite::Pass;
            for handler in handlers[addr as usize].iter() {
                match handler.write(self, addr, value) {
                    MemoryWrite::Pass => (),
                    decided => {
                        outcome = decided;
                        break;
                    }
                }
            }
            outcome
        };

        let value = match outcome {
            MemoryWrite::Block => return,
            MemoryWrite::Replace(replacement) => replacement,
            MemoryWrite::Pass => value,
        };

        match addr {
            // echo ram write
            0xE000..=0xFDFF => self.memory[(addr - 0x2000) as usize].set(value),
            // normal ram write
            _ => self.memory[addr as usize].set(value),
        }
    }
}
