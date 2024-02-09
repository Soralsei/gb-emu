use super::mmu::{MemoryHandler, MemoryRead, MemoryWrite, Mmu};
use crate::cpu::interrupt::InterruptRequest;

const CYCLES_TO_SEND: u32 = 512 * 8; // 8192Hz clock => 512 cpu cycles * 8 bits

pub struct Serial {
    interrupt_request: InterruptRequest,
    sb: u8,                 // Next byte
    recv: u8,               // Received btye
    transfer_enable: bool,  // true if there is an ongoing or pending transfer
    clock_speed: bool,      // CGB only: false: normal, true: fast
    clock_select: bool,     // false: external clock, true : internal
    clock: u32,             // clock timer
    log: String,
}

impl Serial {
    pub fn new(interrupt_request: InterruptRequest) -> Self {
        Self {
            interrupt_request,
            sb: 0x0,
            transfer_enable: false,
            clock_speed: false,
            clock_select: true,
            clock: 0,
            log: String::with_capacity(150),
            recv: 0,
        }
    }

    pub fn step(&mut self, elapsed_cycles: u16) {
        if !self.transfer_enable {
            return;
        }

        // Master
        if self.clock_select {
            self.clock += elapsed_cycles as u32;
            // Transfer done
            if self.clock >= CYCLES_TO_SEND {
                #[cfg(feature="debug")]
                println!("Serial transfer done");
                self.sb = self.recv;
                self.transfer_enable = false;
                self.interrupt_request.serial(true);
            }
        }
        // Slave
        else {
            todo!("Implement serial transfer for slave");
        }
    }

    fn set_sc(&mut self, value: u8) {
        self.transfer_enable = (value & 0x80) != 0;
        self.clock_speed = (value & 0x02) != 0;
        self.clock_select = (value & 0x01) != 0;
    }

    fn get_sc(&self) -> u8 {
        let mut res = 0;
        res |= (self.transfer_enable as u8) << 7;
        res |= (self.clock_speed as u8) << 1;
        res |= self.clock_select as u8;
        res
    }
}

impl MemoryHandler for Serial {
    fn read(&self, _: &Mmu, address: u16) -> MemoryRead {
        match address {
            0xFF01 => MemoryRead::Replace(self.sb),
            0xFF02 => MemoryRead::Replace(self.get_sc()),
            _ => unreachable!("Invalid serial read : 0x{:04X}", address),
        }
    }

    fn write(&mut self, _: &Mmu, address: u16, value: u8) -> MemoryWrite {
        match address {
            0xFF01 => {
                self.sb = value;
            }
            0xFF02 => {
                self.set_sc(value);
                // TODO : abstract byte sending to a handler (network or other)
                // For now, just log the byte
                if self.transfer_enable {
                    self.log.push(self.sb as char);
                    println!("{}", self.log);
                }
            }
            _ => unreachable!("Invalid serial write : 0x{:04X}", address),
        }
        MemoryWrite::Block
    }
}
