use std::cell::RefCell;

use crate::{
    audio::{
        channel::{Channel, Noise, Square, Wave},
        registers::AudioRegisters,
    },
    is_bit_set,
    memory::mmu::{MemoryHandler, MemoryRead, MemoryWrite, Mmu},
};

/// Runs `$body` once per channel, with `$ch` bound to each concrete field:
/// static dispatch, no `dyn`.
macro_rules! each_channel {
    ($state:expr, $ch:ident => $body:expr) => {{
        {
            let $ch = &mut $state.ch1;
            $body;
        }
        {
            let $ch = &mut $state.ch2;
            $body;
        }
        {
            let $ch = &mut $state.ch3;
            $body;
        }
        {
            let $ch = &mut $state.ch4;
            $body;
        }
    }};
}

#[derive(Debug)]
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
    pub fn new(is_cgb: bool) -> Self {
        Self {
            state: RefCell::new(ApuState::new(is_cgb)),
        }
    }

    fn split(address: u16) -> (usize, usize) {
        debug_assert!(
            address > 0xFF0F && address < 0xFF24,
            "Apu address split -> channel/reg only valid within 0xFF10-0xFF23, got 0x{:04X}",
            address
        );
        let offset = (address - 0xFF10) as usize;
        (offset / 5, offset % 5)
    }
}

impl ApuState {
    fn new(is_cgb: bool) -> Self {
        Self {
            is_cgb,
            is_powered: false,
            registers: AudioRegisters::default(),
            ch4: Noise::default(),
            ch3: Wave::default(),
            ch2: Square::ch2(),
            ch1: Square::ch1(),
        }
    }

    fn read(&self, channel: usize, reg: usize) -> u8 {
        match channel {
            0 => self.ch1.read(reg),
            1 => self.ch2.read(reg),
            2 => self.ch3.read(reg),
            _ => self.ch4.read(reg),
        }
    }

    fn write(&mut self, channel: usize, reg: usize, value: u8) {
        match channel {
            0 => self.ch1.write(reg, value, 0),
            1 => self.ch2.write(reg, value, 0),
            2 => self.ch3.write(reg, value, 0),
            _ => self.ch4.write(reg, value, 0),
        }
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
        each_channel!(self, ch => ch.power_off(is_cgb));
    }
}

impl MemoryHandler for Apu {
    fn read(&self, _: &Mmu, address: u16) -> MemoryRead {
        match address {
            0xFF26 => MemoryRead::Replace(self.state.borrow().read_nr52()),

            0xFF10..=0xFF23 => {
                let (channel, reg) = Apu::split(address);
                MemoryRead::Replace(self.state.borrow().read(channel, reg))
            }
            _ => MemoryRead::Replace(self.state.borrow().registers.read(address)),
        }
    }

    fn write(&self, _: &Mmu, address: u16, value: u8) -> MemoryWrite {
        match address {
            0xFF26 => {
                self.state.borrow_mut().write_nr52(value);
            }
            0xFF10..=0xFF23 => {
                let (channel, reg) = Apu::split(address);
                self.state.borrow_mut().write(channel, reg, value);
            }
            _ => {
                self.state.borrow_mut().registers.write(address, value);
            }
        }
        MemoryWrite::Block
    }
}
