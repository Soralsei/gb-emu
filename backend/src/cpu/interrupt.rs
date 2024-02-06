use std::{cell::RefCell, rc::Rc};

use crate::memory::mmu::{Mmu, MemoryHandler, MemoryRead, MemoryWrite};

const VBLANK: u8 = 0x1;
const LCD: u8 = 0x2;
const TIMER: u8 = 0x4;
const SERIAL: u8 = 0x8;
const JOYPAD: u8 = 0x10;

#[derive(Default)]
pub struct Interrupts {
    vblank: bool,
    lcd: bool,
    timer: bool,
    serial: bool,
    joypad: bool,
}

impl Interrupts {
    fn set(&mut self, value: u8) {
        self.vblank = value & VBLANK != 0;
        self.lcd = value & LCD != 0;
        self.timer = value & TIMER != 0;
        self.serial = value & SERIAL != 0;
        self.joypad = value & JOYPAD != 0;
    }
    fn get(&self) -> u8 {
        u8::from(self)
    }
}

impl std::convert::From<u8> for Interrupts {
    fn from(value: u8) -> Self {
        Self {
            vblank: value & VBLANK != 0,
            lcd: value & LCD != 0,
            timer: value & TIMER != 0,
            serial: value & SERIAL != 0,
            joypad: value & JOYPAD != 0,
        }
    }
}

impl std::convert::From<&Interrupts> for u8 {
    fn from(value: &Interrupts) -> Self {
        let mut res = 0;
        res |= (value.vblank as u8) << 0;
        res |= (value.lcd as u8) << 1;
        res |= (value.timer as u8) << 2;
        res |= (value.serial as u8) << 3;
        res |= (value.joypad as u8) << 4;
        res
    }
}

pub struct InterruptRequest {
    request: Rc<RefCell<Interrupts>>,
}

impl InterruptRequest {
    pub fn new(request: Rc<RefCell<Interrupts>>) -> InterruptRequest {
        Self { request }
    }

    pub fn vblank(&mut self, value: bool) {
        self.request.borrow_mut().vblank = value;
        if value {
            println!("VBlank requested");
        }
    }

    pub fn lcd(&mut self, value: bool) {
        self.request.borrow_mut().lcd = value;
        if value {
            println!("LCD interrupt requested");
        }
    }

    pub fn timer(&mut self, value: bool) {
        self.request.borrow_mut().timer = value;
        if value {
            println!("Timer interrupt requested");
        }
    }

    pub fn serial(&mut self, value: bool) {
        self.request.borrow_mut().serial = value;
        if value {
            println!("serial interrupt requested");
        }
    }

    pub fn joypad(&mut self, value: bool) {
        self.request.borrow_mut().joypad = value;
        if value {
            println!("Joypad interrupt requested");
        }
    }
}

pub struct InterruptController {
    enable: Rc<RefCell<Interrupts>>,
    flags: Rc<RefCell<Interrupts>>,
}

impl InterruptController {
    pub fn new() -> Self {
        Self {
            enable: Rc::new(RefCell::new(Interrupts::default())),
            flags: Rc::new(RefCell::new(Interrupts::default())),
        }
    }

    pub fn request(&self) -> InterruptRequest {
        InterruptRequest::new(self.flags.clone())
    }

    pub fn peek(&self) -> Option<u8> {
        self.check(false)
    }

    pub fn consume(&self) -> Option<u8> {
        self.check(true)
    }

    fn check(&self, consume: bool) -> Option<u8> {
        let enable = self.enable.borrow();
        let mut request = self.flags.borrow_mut();
        if enable.vblank && request.vblank {
            request.vblank = !consume;
            return Some(0x40);
        } else if enable.lcd && request.lcd {
            request.lcd = !consume;
            return Some(0x48);
        } else if enable.timer && request.timer {
            request.timer = !consume;
            return Some(0x50);
        } else if enable.serial && request.serial {
            request.serial = !consume;
            return Some(0x58);
        } else if enable.joypad && request.joypad {
            request.joypad = !consume;
            return Some(0x60);
        }
        None
    }
}

impl MemoryHandler for InterruptController {
    fn read(&self, _: &Mmu, address: u16) -> MemoryRead {
        match address {
            0xffff => MemoryRead::Replace(self.enable.borrow().get()),
            0xff0f => MemoryRead::Replace(self.flags.borrow().get()),
            _ => {
                println!("IC received weird write to address 0x{:04X}", address);
                MemoryRead::Pass
            }
        }
    }

    fn write(
        &mut self,
        _: &Mmu,
        address: u16,
        value: u8,
    ) -> MemoryWrite {
        match address {
            0xffff => {
                self.enable.borrow_mut().set(value);
                MemoryWrite::Block
            }
            0xff0f => {
                self.flags.borrow_mut().set(value);
                MemoryWrite::Block
            }
            _ => {
                println!("IC received weird write to address 0x{:04X}", address);
                MemoryWrite::Pass
            }
        }
    }
}
