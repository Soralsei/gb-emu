use std::cell::{Ref, RefCell, RefMut};
use std::rc::Rc;

use crate::clock::Clock;
use crate::{SCREEN_H, SCREEN_W};

use super::memory::mbc::Mbc;
use super::memory::mmu::MemoryHandler;

use super::cpu::cpu::Cpu;
use super::cpu::interrupt::InterruptController;
use super::cpu::timer::Timer;
#[cfg(feature = "blaarg")]
use super::debug::blaarg_spy::BlaargSpy;
use super::memory::mmu::Mmu;
use super::memory::serial::Serial;

const CYCLES_PER_FRAME: u32 = 70224; // 154 lines * 456 T-cycles

#[derive(Copy, Clone)]
pub enum Button {
    A,
    B,
    Select,
    Start,
    Right,
    Left,
    Up,
    Down,
}

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
            Err(e) => panic!("Recursive read at 0x{:04X}: {}", address, e),
        }
    }

    fn write(&mut self, mmu: &Mmu, address: u16, value: u8) -> crate::memory::mmu::MemoryWrite {
        match self.0.try_borrow_mut() {
            Ok(mut device) => return device.write(mmu, address, value),
            Err(e) => panic!("Recursive write at 0x{:04X}: {}", address, e),
        }
    }
}

pub struct System {
    cpu: Cpu,
    interrupt_controller: Device<InterruptController>,
    // timer: Device<Timer>,
    // serial: Device<Serial>,
    current_frame: Box<[u8; SCREEN_W * SCREEN_H]>,
}

impl System {
    pub fn new(boot_rom: Option<Vec<u8>>, rom: Vec<u8>) -> Self {
        let clock = Clock::new();
        let interrupt_controller = Device::new(InterruptController::new());
        let serial = Device::new(Serial::new(interrupt_controller.borrow().request()));
        let timer = Device::new(Timer::new(interrupt_controller.borrow().request()));

        let mut mmu = Mmu::new(clock.clone());
        let mbc = Device::new(Mbc::new(boot_rom, rom));

        clock.attach(timer.0.clone());
        clock.attach(serial.0.clone());

        #[cfg(feature = "blaarg")]
        {
            println!("Added blaarg debug feature");
            let spy = IoMemoryHandler(Rc::new(RefCell::new(BlaargSpy())));
            mmu.add_handler((0xA000, 0xBFFF), spy);
        }

        mmu.add_handler((0x0000, 0x7fff), mbc.handler());
        mmu.add_handler((0xff50, 0xff50), mbc.handler());
        mmu.add_handler((0xa000, 0xbfff), mbc.handler());

        mmu.add_handler((0xFF01, 0xFF02), serial.handler());
        mmu.add_handler((0xFF04, 0xFF07), timer.handler());

        mmu.add_handler((0xff0f, 0xff0f), interrupt_controller.handler());
        mmu.add_handler((0xffff, 0xffff), interrupt_controller.handler());

        let cpu = Cpu::new(mmu, clock.clone());
        Self {
            cpu,
            interrupt_controller,
            current_frame: Box::new([0; SCREEN_W * SCREEN_H]),
        }
    }

    pub fn step(&mut self) -> usize {
        let mut elapsed = self.cpu.execute_instruction();
        elapsed += self
            .cpu
            .handle_interrupts(self.interrupt_controller.borrow_mut());
        elapsed
    }

    pub fn run_frame(&mut self) {
        static mut COUNTER: usize = 0;
        let mut cycle_budget = CYCLES_PER_FRAME * 2;
        loop {
            cycle_budget = cycle_budget.saturating_sub(self.step() as u32);
            // if self.ppu.borrow().framebuffer() || cycle_budget == 0 {
            if cycle_budget == 0 {
                break;
            }
        }
        // self.current_frame
        // .copy_from_slice(self.ppu.borrow().framebuffer());
        unsafe {
            render_test_pattern(&mut self.current_frame[..], SCREEN_W, COUNTER);
            COUNTER += 1;
        }
    }

    pub fn get_framebuffer(&self) -> &[u8] {
        &self.current_frame[..]
    }

    pub fn set_button(&mut self, b: Button, down: bool) {
        // TODO: Implement joypad device
        // self.joypad.borrow_mut().set(b, down);
    }
}

pub fn render_test_pattern(framebuffer: &mut [u8], width: usize, frame_counter: usize) {
    const NUM_COLORS: usize = 4;

    let bayer: [[f32; 4]; 4] = [
        [0.0 / 16.0, 8.0 / 16.0, 2.0 / 16.0, 10.0 / 16.0],
        [12.0 / 16.0, 4.0 / 16.0, 14.0 / 16.0, 6.0 / 16.0],
        [3.0 / 16.0, 11.0 / 16.0, 1.0 / 16.0, 9.0 / 16.0],
        [15.0 / 16.0, 7.0 / 16.0, 13.0 / 16.0, 5.0 / 16.0],
    ];

    let animation_speed = 3;

    for (i, pixel) in framebuffer.iter_mut().enumerate() {
        let x = i % width;
        let y = i / width;

        let shifted_x = (x + (frame_counter * animation_speed)) % width;

        let t = shifted_x as f32 / (width - 1) as f32;
        let scaled_t = t * (NUM_COLORS - 1) as f32;

        let mut index = scaled_t.floor() as usize;
        let mut fraction = scaled_t - index as f32;

        if index >= NUM_COLORS - 1 {
            index = NUM_COLORS - 2;
            fraction = 1.0;
        }

        let threshold = bayer[y % 4][x % 4];

        let final_index = if fraction > threshold {
            index + 1
        } else {
            index
        };
        *pixel = final_index as u8;
    }
}
