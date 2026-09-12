use std::rc::Rc;

use crate::clock::Clock;
use crate::graphics::oam::DMAController;
use crate::graphics::ppu::Ppu;
use crate::input::{Button, JoypadHandler};
use crate::memory::bus::{BusController, CpuBus};
use crate::memory::mmu::{MemoryRead, MemoryWrite};
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

/// KEY1 (0xFF4D). Both meaningful bits — current speed and the armed switch —
/// are clock state, so the handler is a pure view onto `Clock`.
struct SpeedSwitch(Rc<Clock>);

impl MemoryHandler for SpeedSwitch {
    fn read(&self, _mmu: &Mmu, _address: u16) -> MemoryRead {
        MemoryRead::Replace(self.0.key1())
    }

    fn write(&self, _mmu: &Mmu, _address: u16, value: u8) -> MemoryWrite {
        self.0.arm(value & 1 != 0);
        // The handler is authoritative: keep the raw backing store out of it,
        // otherwise a stale byte shadows `Clock::key1`.
        MemoryWrite::Block
    }
}

struct Unmapped;

impl MemoryHandler for Unmapped {
    fn read(&self, _: &Mmu, _: u16) -> MemoryRead {
        MemoryRead::Replace(0xFF)
    }
    fn write(&self, _: &Mmu, _: u16, _: u8) -> MemoryWrite {
        MemoryWrite::Block
    }
}

pub struct System {
    cpu: Cpu,
    clock: Rc<Clock>,
    interrupt_controller: Rc<InterruptController>,
    joypad: Rc<JoypadHandler>,
    current_frame: Box<[u8; SCREEN_W * SCREEN_H]>,
}

impl System {
    pub fn new(boot_rom: Option<Vec<u8>>, rom: Vec<u8>, is_cgb_override: bool) -> Self {
        let mbc = Rc::new(Mbc::new(boot_rom, rom));
        let is_cgb = mbc.cartridge().is_cgb_only() || is_cgb_override;

        let clock = Clock::new(is_cgb);

        let interrupt_controller = Rc::new(InterruptController::new());
        let serial = Rc::new(Serial::new(interrupt_controller.request()));
        let timer = Rc::new(Timer::new(interrupt_controller.request(), is_cgb));
        let joypad = Rc::new(JoypadHandler::new(interrupt_controller.request()));

        // The bus is shared: the CPU drives it, and clocked devices that move
        // bytes themselves (OAM DMA, the PPU fetcher) need a handle to it too.
        let mmu = Rc::new(Mmu::new());
        let bus_controller = Rc::new(BusController::new());

        let cpu = Cpu::new(
            CpuBus::new(mmu.clone(), bus_controller.clone(), clock.clone()),
            clock.clone(),
            is_cgb,
        );
        let ppu = Rc::new(Ppu::new(interrupt_controller.request()));
        let dma = Rc::new(DMAController::new(mmu.clone(), bus_controller.clone()));

        clock.attach(timer.clone());
        clock.attach(serial.clone());
        clock.attach(dma.clone());

        clock.attach_fixed(ppu.clone());
        //clock.attach_fixed(apu.clone())

        #[cfg(feature = "blaarg")]
        {
            println!("Added blaarg debug feature");
            mmu.add_handler((0xA000, 0xBFFF), Rc::new(BlaargSpy()));
        }

        mmu.add_handler((0x0000, 0x7FFF), mbc.clone());
        mmu.add_handler((0xFF50, 0xFF50), mbc.clone());
        mmu.add_handler((0xA000, 0xBFFF), mbc.clone());

        // OAM
        mmu.add_handler((0xFE00, 0xFE9F), ppu.clone());
        // Ppu registers other than OAM DMA
        mmu.add_handler((0xFF40, 0xFF45), ppu.clone());
        // OAM DMA register
        mmu.add_handler((0xFF46, 0xFF46), dma.clone());
        // Ppu rest of registers
        mmu.add_handler((0xFF47, 0xFF4B), ppu.clone());

        mmu.add_handler((0xFF00, 0xFF00), joypad.clone());

        mmu.add_handler((0xFF01, 0xFF02), serial.clone());
        mmu.add_handler((0xFF04, 0xFF07), timer.clone());

        mmu.add_handler((0xFF0F, 0xFF0F), interrupt_controller.clone());
        mmu.add_handler((0xFF4D, 0xFF4D), Rc::new(SpeedSwitch(clock.clone())));
        if !is_cgb {
            let unmapped = Rc::new(Unmapped);
            mmu.add_handler((0xFF4D, 0xFF4D), unmapped.clone()); // KEY1
            mmu.add_handler((0xFF4F, 0xFF4F), unmapped.clone()); // VBK
            mmu.add_handler((0xFF51, 0xFF55), unmapped.clone()); // HDMA1-5
            mmu.add_handler((0xFF68, 0xFF6B), unmapped.clone()); // BCPS/BCPD/OCPS/OCPD
            mmu.add_handler((0xFF70, 0xFF70), unmapped);
        }
        mmu.add_handler((0xFFFF, 0xFFFF), interrupt_controller.clone());

        Self {
            cpu,
            clock,
            interrupt_controller,
            joypad,
            current_frame: Box::new([0; SCREEN_W * SCREEN_H]),
        }
    }

    pub fn step(&mut self) -> usize {
        let mut elapsed = self.cpu.execute_instruction();
        elapsed += self.cpu.handle_interrupts(&self.interrupt_controller);
        elapsed
    }

    pub fn run_frame(&mut self) {
        static mut COUNTER: usize = 0;
        let mut cycle_budget = CYCLES_PER_FRAME * 2;
        loop {
            cycle_budget = cycle_budget.saturating_sub(self.step() as u32);
            // STOP mode burns no cycles, so the budget would never drain. Hand
            // the frame back so the frontend can poll the joypad that ends it.
            if self.cpu.is_stopped() {
                break;
            }
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

    pub fn set_button(&mut self, btn: Button, down: bool) {
        if self.joypad.set(btn, down) {
            self.clock.resume();
            self.cpu.resume();
        }
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
