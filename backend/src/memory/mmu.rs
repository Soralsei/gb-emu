use std::{cell::RefCell, collections::BTreeMap, rc::Rc};

use crate::clock::{Clock, M_CYCLE};

pub enum MemoryRead {
    Replace(u8),
    Pass,
}

pub enum MemoryWrite {
    Replace(u8),
    Pass,
    Block,
}

pub trait MemoryHandler {
    fn read(&self, mmu: &Mmu, address: u16) -> MemoryRead;
    fn write(&mut self, mmu: &Mmu, address: u16, value: u8) -> MemoryWrite;
}

#[allow(unused)]
pub struct Mmu {
    pub handlers: BTreeMap<u16, Vec<Rc<RefCell<dyn MemoryHandler>>>>,
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

    pub fn add_handler<T: MemoryHandler + 'static>(
        &mut self,
        address_range: (u16, u16),
        handler: T,
    ) {
        let handler = Rc::new(RefCell::new(handler));
        for address in address_range.0..=address_range.1 {
            if self.handlers.contains_key(&address) {
                match self.handlers.get_mut(&address) {
                    Some(handler_list) => handler_list.push(handler.clone()),
                    None => {}
                }
            } else {
                self.handlers.insert(address, vec![handler.clone()]);
            }
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
        match self.handlers.get(&addr) {
            Some(handlers) => {
                for handler in handlers {
                    match handler.borrow().read(self, addr) {
                        MemoryRead::Replace(value) => return value,
                        MemoryRead::Pass => {}
                    }
                }
            }
            None => {
                // #[cfg(feature="debug")]
                // println!("[MMU] No explicit handler for address 0x{:04x}", addr);
            }
        };

        match addr {
            // echo ram read
            0xE000..=0xFDFF => self.memory[(addr - 0x2000) as usize],
            // normal ram read
            _ => self.memory[addr as usize],
        }
    }

    /// Bus write, as performed by the CPU: costs a machine cycle.
    pub fn write(&mut self, addr: u16, value: u8) {
        self.clock.tick(M_CYCLE);
        match self.handlers.get(&addr) {
            Some(handlers) => {
                for handler in handlers {
                    match handler.borrow_mut().write(self, addr, value) {
                        MemoryWrite::Replace(v) => {
                            self.memory[addr as usize] = v;
                            return;
                        }
                        MemoryWrite::Pass => (),
                        MemoryWrite::Block => return,
                    }
                }
            }
            None => (),
            // None => println!("[MMU] No explicit handler for address 0x{:04x}", addr),
        };

        match addr {
            // echo ram write
            0xE000..=0xFDFF => self.memory[(addr - 0x2000) as usize] = value,
            // normal ram write
            _ => self.memory[addr as usize] = value,
        }
    }
}
