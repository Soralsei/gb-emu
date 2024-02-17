use std::{cell::RefCell, rc::Rc};

use crate::{
    is_bit_set,
    memory::mmu::{MemoryHandler, MemoryRead, MemoryWrite, Mmu},
};

const VBLANK: u8 = 0;
const LCD: u8 = 1;
const TIMER: u8 = 2;
const SERIAL: u8 = 3;
const JOYPAD: u8 = 4;

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
        self.vblank = is_bit_set!(value, VBLANK);
        self.lcd = is_bit_set!(value, LCD);
        self.timer = is_bit_set!(value, TIMER);
        self.serial = is_bit_set!(value, SERIAL);
        self.joypad = is_bit_set!(value, JOYPAD);
    }
    fn get(&self) -> u8 {
        u8::from(self)
    }
}

impl std::convert::From<u8> for Interrupts {
    fn from(value: u8) -> Self {
        Self {
            vblank: is_bit_set!(value, VBLANK),
            lcd: is_bit_set!(value, LCD),
            timer: is_bit_set!(value, TIMER),
            serial: is_bit_set!(value, SERIAL),
            joypad: is_bit_set!(value, JOYPAD),
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
        #[cfg(feature = "debug")]
        if value {
            println!("VBlank requested");
        }
    }

    pub fn lcd(&mut self, value: bool) {
        self.request.borrow_mut().lcd = value;
        #[cfg(feature = "debug")]
        if value {
            println!("LCD interrupt requested");
        }
    }

    pub fn timer(&mut self, value: bool) {
        self.request.borrow_mut().timer = value;
        #[cfg(feature = "debug")]
        if value {
            println!("Timer interrupt requested");
        }
    }

    pub fn serial(&mut self, value: bool) {
        self.request.borrow_mut().serial = value;
        #[cfg(feature = "debug")]
        if value {
            println!("serial interrupt requested");
        }
    }

    pub fn joypad(&mut self, value: bool) {
        self.request.borrow_mut().joypad = value;
        #[cfg(feature = "debug")]
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
                #[cfg(feature = "debug")]
                println!("IC received weird read to address 0x{:04X}", address);
                MemoryRead::Pass
            }
        }
    }

    fn write(&mut self, _: &Mmu, address: u16, value: u8) -> MemoryWrite {
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
                #[cfg(feature = "debug")]
                println!("IC received weird write to address 0x{:04X}", address);
                MemoryWrite::Pass
            }
        }
    }
}
