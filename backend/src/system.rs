use std::cell::Ref;
use std::rc::Rc;

use crate::clock::{CpuClock, Time};
use crate::graphics::oam::DMAController;
use crate::graphics::ppu::{Frame, Ppu};
use crate::input::{Button, JoypadHandler};
use crate::memory::bus::{BusController, CpuBus};
use crate::memory::mmu::{MemoryRead, MemoryWrite};

use super::memory::mbc::Mbc;
use super::memory::mmu::MemoryHandler;

use super::cpu::cpu::Cpu;
use super::cpu::interrupt::InterruptController;
use super::cpu::timer::Timer;
#[cfg(feature = "blaarg")]
use super::debug::blaarg_spy::BlaargSpy;
use super::memory::mmu::Mmu;
use super::memory::serial::{LogSink, Serial};

const CYCLES_PER_FRAME: u32 = 70224; // 154 lines * 456 T-cycles

/// KEY1 (0xFF4D). Both meaningful bits — current speed and the armed switch —
/// are clock state, so the handler is a pure view onto `Clock`.
struct SpeedSwitch(Rc<CpuClock>);

impl MemoryHandler for SpeedSwitch {
    fn read(&self, _: &Mmu, _address: u16) -> MemoryRead {
        MemoryRead::Replace(self.0.key1())
    }

    fn write(&self, _: &Mmu, _address: u16, value: u8) -> MemoryWrite {
        self.0.arm(value & 1 != 0);
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
    cpu: Rc<Cpu>,
    ppu: Rc<Ppu>,
    time: Time,
    interrupt_controller: Rc<InterruptController>,
    joypad: Rc<JoypadHandler>,
}

impl System {
    pub fn new(boot_rom: Option<Vec<u8>>, rom: Vec<u8>, is_cgb_override: bool) -> Self {
        let time = Time::new();

        let mbc = Rc::new(Mbc::new(boot_rom, rom));
        let is_cgb = mbc.cartridge().is_cgb_only() || is_cgb_override;

        let interrupt_controller = Rc::new(InterruptController::new());
        let serial = Rc::new(Serial::new(interrupt_controller.request(), LogSink));
        let timer = Rc::new(Timer::new(
            interrupt_controller.request(),
            time.cpu.clone(),
            is_cgb,
        ));
        let joypad = Rc::new(JoypadHandler::new(interrupt_controller.request()));

        // The bus is shared: the CPU drives it, and clocked devices that move
        // bytes themselves (OAM DMA, the PPU fetcher) need a handle to it too.
        let mmu = Rc::new(Mmu::new());
        let bus_controller = Rc::new(BusController::new());

        let cpu = Rc::new(Cpu::new(is_cgb));
        let ppu = Rc::new(Ppu::new(
            interrupt_controller.request(),
            bus_controller.clone(),
            is_cgb,
        ));
        let dma = Rc::new(DMAController::new(mmu.clone(), bus_controller.clone()));

        time.spawn(Ppu::task(ppu.clone(), time.fixed.timeline()));
        time.spawn(Serial::task(serial.clone(), time.cpu.timeline()));
        time.spawn(DMAController::task(dma.clone(), time.cpu.timeline()));
        time.spawn(Timer::task(timer.clone(), time.cpu.timeline()));
        time.spawn(Cpu::task(
            cpu.clone(),
            interrupt_controller.clone(),
            CpuBus::new(mmu.clone(), bus_controller.clone()),
            time.cpu.clone(),
            time.fixed.clone(),
        ));

        #[cfg(feature = "blaarg")]
        {
            println!("Added blaarg debug feature");
            mmu.add_handler((0xA000, 0xBFFF), Rc::new(BlaargSpy()));
        }

        mmu.add_handler((0x0000, 0x7FFF), mbc.clone());
        mmu.add_handler((0xFF50, 0xFF50), mbc.clone());
        mmu.add_handler((0xA000, 0xBFFF), mbc.clone());

        // PPU VRAM
        mmu.add_handler((0x8000, 0x9FFF), ppu.clone());
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
        if is_cgb {
            mmu.add_handler((0xFF4D, 0xFF4D), Rc::new(SpeedSwitch(time.cpu.clone())));
            // TODO: map other IO registers in the CGB
        } else {
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
            ppu,
            time,
            interrupt_controller,
            joypad,
        }
    }

    pub fn step(&mut self) {
        let mark = self.cpu.instr_count();
        while self.cpu.instr_count() == mark {
            self.time.tick();
        }
    }

    pub fn run_frame(&mut self) {
        for _ in 0..(CYCLES_PER_FRAME * 2) {
            self.time.tick();
            if self.ppu.take_frame_ready() {
                break;
            }
        }
    }

    pub fn get_framebuffer(&self) -> Ref<'_, Frame> {
        self.ppu.framebuffer()
    }

    pub fn set_button(&mut self, btn: Button, down: bool) {
        if self.joypad.set(btn, down) {
            self.time.resume();
        }
    }
}
