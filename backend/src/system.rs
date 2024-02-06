use std::cell::{Ref, RefCell, RefMut};
use std::rc::Rc;

use crate::memory::mmu::{MemoryRead, MemoryWrite};

use super::memory::mbc::Mbc;
use super::memory::mmu::MemoryHandler;

use super::cpu::cpu::Cpu;
use super::cpu::interrupt::InterruptController;
use super::cpu::timer::Timer;
use super::memory::mmu::Mmu;
use super::memory::serial::Serial;
#[cfg(feature="blaarg")]
use super::debug::blaarg_spy::BlaargSpy;

#[derive(Clone)]
struct IoMemoryHandler<T>(Rc<RefCell<T>>);
struct Device<T>(Rc<RefCell<T>>);

impl<T> Device<T> {
    pub fn new(dev: T) -> Self {
        Self(Rc::new(RefCell::new(dev)))
    }

    pub fn borrow<'a>(&'a self) -> Ref<'a, T> {
        self.0.borrow()
    }

    pub fn borrow_mut<'a>(&'a self) -> RefMut<'a, T> {
        self.0.borrow_mut()
    }
}

impl<T: MemoryHandler> Device<T> {
    pub fn handler(&self) -> IoMemoryHandler<T> {
        IoMemoryHandler(self.0.clone())
    }
}

impl<T: MemoryHandler> MemoryHandler for IoMemoryHandler<T> {
    fn read(&self, mmu: &Mmu, address: u16) -> crate::memory::mmu::MemoryRead {
        match self.0.try_borrow_mut() {
            Ok(device) => return device.read(mmu, address),
            Err(e) => eprintln!("Recursive write at 0x{:04X}: {}", address, e),
        }
        MemoryRead::Pass
    }

    fn write(&mut self, mmu: &Mmu, address: u16, value: u8) -> crate::memory::mmu::MemoryWrite {
        match self.0.try_borrow_mut() {
            Ok(mut device) => return device.write(mmu, address, value),
            Err(e) => eprintln!("Recursive write at 0x{:04X}: {}", address, e),
        }
        MemoryWrite::Block
    }
}

pub struct System{
    cpu: Cpu,
    interrupt_controller: Device<InterruptController>,
    timer: Device<Timer>,
    serial: Device<Serial>,
}

impl System {
    pub fn new(boot_rom: Option<Vec<u8>>, rom: Vec<u8>) -> Self {
        let interrupt_controller = Device::new(InterruptController::new());
        let serial =  Device::new(Serial::new(interrupt_controller.borrow().request()));
        let timer = Device::new(Timer::new(interrupt_controller.borrow().request()));

        let mut mmu = Mmu::new();
        let mbc = Device::new(Mbc::new(boot_rom, rom));

        #[cfg(feature="blaarg")]
        {
            println!("Added blaarg debug feature");
            let spy = IoMemoryHandler(Rc::new(RefCell::new(BlaargSpy())));
            mmu.add_handler((0xA000, 0xBFFF), spy);
        }
        
        mmu.add_handler((0x0000, 0x7fff), mbc.handler());
        mmu.add_handler((0xff50, 0xff50), mbc.handler());
        mmu.add_handler((0xa000, 0xbfff), mbc.handler());

        mmu.add_handler((0xff0f, 0xff0f), interrupt_controller.handler());
        mmu.add_handler((0xffff, 0xffff), interrupt_controller.handler());

        mmu.add_handler((0xFF01, 0xFF02), serial.handler());
        mmu.add_handler((0xFF04, 0xFF07), timer.handler());
        let cpu = Cpu::new(mmu);
        Self {
            cpu,
            interrupt_controller,
            timer,
            serial,
        }
    }

    pub fn step(&mut self) {
        let elapsed = self.cpu.step(self.interrupt_controller.borrow_mut());
        self.timer.borrow_mut().step(elapsed);
        self.serial.borrow_mut().step(elapsed);
    }
}