use std::cell::RefCell;

use super::mmu::{MemoryHandler, MemoryRead, MemoryWrite, Mmu};
use crate::{clock::Clocked, cpu::interrupt::InterruptRequest, is_bit_set};

const CYCLES_TO_SEND: u32 = 512 * 8; // 8192Hz clock => 512 cpu cycles * 8 bits
const CLOCK_SELECT: u8 = 0;
const CLOCK_SPEED: u8 = 1;
const TRANSFER_ENABLE: u8 = 7;

/// Like the timer, serial is only reached by its own `step` and its own
/// register handler, so one cell at the boundary keeps the logic on `&mut self`.
pub struct Serial {
    state: RefCell<SerialState>,
}

impl Serial {
    pub fn new(interrupt_request: InterruptRequest) -> Self {
        Self {
            state: RefCell::new(SerialState::new(interrupt_request)),
        }
    }
}

struct SerialState {
    interrupt_request: InterruptRequest,
    send_byte: u8,         // Next byte
    recv_byte: u8,         // Received btye
    transfer_enable: bool, // true if there is an ongoing or pending transfer
    clock_speed: bool,     // CGB only: false: normal, true: fast
    clock_select: bool,    // false: external clock, true : internal
    clock: u32,            // clock timer
    log: String,
}

impl SerialState {
    fn new(interrupt_request: InterruptRequest) -> Self {
        Self {
            interrupt_request,
            send_byte: 0x0,
            transfer_enable: false,
            clock_speed: false,
            clock_select: true,
            clock: 0,
            log: String::with_capacity(150),
            recv_byte: 0,
        }
    }

    fn set_sc(&mut self, value: u8) {
        self.transfer_enable = is_bit_set!(value, TRANSFER_ENABLE);
        self.clock_speed = is_bit_set!(value, CLOCK_SPEED);
        self.clock_select = is_bit_set!(value, CLOCK_SELECT);
    }

    fn get_sc(&self) -> u8 {
        let mut res = 0;
        res |= (self.transfer_enable as u8) << TRANSFER_ENABLE;
        res |= (self.clock_speed as u8) << CLOCK_SPEED;
        res |= (self.clock_select as u8) << CLOCK_SELECT;
        res
    }
}

impl MemoryHandler for Serial {
    fn read(&self, _: &Mmu, address: u16) -> MemoryRead {
        self.state.borrow().read(address)
    }

    fn write(&self, _: &Mmu, address: u16, value: u8) -> MemoryWrite {
        self.state.borrow_mut().write(address, value)
    }
}

impl Clocked for Serial {
    fn step(&self, elapsed_cycles: u16) {
        self.state.borrow_mut().step(elapsed_cycles);
    }
}

impl SerialState {
    fn read(&self, address: u16) -> MemoryRead {
        match address {
            0xFF01 => MemoryRead::Replace(self.send_byte),
            0xFF02 => MemoryRead::Replace(self.get_sc()),
            _ => unreachable!("Invalid serial read : 0x{:04X}", address),
        }
    }

    fn write(&mut self, address: u16, value: u8) -> MemoryWrite {
        match address {
            0xFF01 => {
                self.send_byte = value;
            }
            0xFF02 => {
                self.set_sc(value);
                // TODO : abstract byte sending to a handler (network or other)
                // For now, just log the byte
                if self.transfer_enable {
                    self.log.push(self.send_byte as char);
                    println!("{}", self.log);
                }
            }
            _ => unreachable!("Invalid serial write : 0x{:04X}", address),
        }
        MemoryWrite::Block
    }

    fn step(&mut self, elapsed_cycles: u16) {
        if !self.transfer_enable {
            return;
        }

        // Master
        if self.clock_select {
            self.clock += elapsed_cycles as u32;
            // Transfer done
            if self.clock >= CYCLES_TO_SEND {
                self.send_byte = self.recv_byte;
                self.transfer_enable = false;
                self.interrupt_request.serial(true);
                self.clock = 0;
            }
        }
        // Slave
        else {
            todo!("Implement serial transfer for slave");
        }
    }
}
