use std::cell::RefCell;

use crate::{
    audio::{
        channel::{Channel, Noise, Square, Wave},
        registers::AudioRegisters,
    },
    is_bit_set,
    memory::mmu::{MemoryHandler, MemoryRead, MemoryWrite, Mmu},
};

#[derive(Debug, Default)]
struct ApuState {
    is_cgb: bool,
    is_powered: bool,
    registers: AudioRegisters,
    ch4: Noise,
    ch3: Wave,
    ch2: Square,
    ch1: Square,
}

pub struct Apu {
    state: RefCell<ApuState>,
}

impl Apu {
    pub fn new() -> Self {
        Self {
            state: RefCell::default(),
        }
    }
}

impl ApuState {
    fn channel(&self, chan: usize) -> &dyn Channel {
        match chan {
            0 => &self.ch1,
            1 => &self.ch2,
            2 => &self.ch3,
            _ => &self.ch4,
        }
    }

    fn channels_mut(&mut self) -> [&mut dyn Channel; 4] {
        [&mut self.ch1, &mut self.ch2, &mut self.ch3, &mut self.ch4]
    }

    fn read_nr52(&self) -> u8 {
        0b01110000
            | (self.is_powered as u8) << 7
            | (self.ch4.enabled() as u8) << 3
            | (self.ch3.enabled() as u8) << 2
            | (self.ch2.enabled() as u8) << 1
            | (self.ch1.enabled() as u8)
    }

    fn write_nr52(&mut self, value: u8) {
        let power = is_bit_set!(value, 7);
        match (self.is_powered, power) {
            (true, false) => self.power_off(),
            (false, true) => self.power_on(),
            _ => {}
        }
        self.is_powered = power;
        // bits 0-3 are read-only and depend only on individual channel enabled state
    }

    fn power_on(&mut self) {
        self.ch3.clear()
    }

    fn power_off(&mut self) {
        let is_cgb = self.is_cgb;
        for ch in self.channels_mut() {
            ch.power_off(is_cgb);
        }
    }
}

impl MemoryHandler for Apu {
    fn read(&self, _: &Mmu, address: u16) -> MemoryRead {
        match address {
            0xFF26 => MemoryRead::Replace(self.state.borrow().read_nr52()),
            _ => MemoryRead::Replace(self.state.borrow().registers.read(address)),
        }
    }

    fn write(&self, _: &Mmu, address: u16, value: u8) -> MemoryWrite {
        match address {
            0xFF26 => {
                // TODO: reset all other APU registers
                // ...
                self.state.borrow_mut().write_nr52(value);
            }
            _ => {
                self.state.borrow_mut().registers.write(address, value);
            }
        }
        MemoryWrite::Block
    }
}
