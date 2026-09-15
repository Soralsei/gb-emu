#![allow(dead_code)]
use std::cell::{Cell, Ref, RefCell, RefMut};
use std::convert::Infallible;
use std::rc::Rc;

use crate::clock::Timeline;
use crate::cpu::interrupt::InterruptRequest;
use crate::graphics::fifo::FIFOBuffer;
use crate::graphics::object_attributes::ObjectAttribute;
use crate::graphics::registers::{PpuMode, PpuRegisters, TileArea};
use crate::memory::mmu::{MemoryHandler, MemoryRead, MemoryWrite, Mmu};
use crate::{SCREEN_H, SCREEN_W};

const OAM_SCAN_DOTS: u16 = 2;

struct ScannedObject {
    attributes: ObjectAttribute,
    // Index into OAM
    index: usize,
}

pub type Frame = [u8; SCREEN_W * SCREEN_H];

pub struct Ppu {
    registers: RefCell<PpuRegisters>,
    vram: [[Cell<u8>; 0x2000]; 2],

    interrupt_request: InterruptRequest,

    /// OAM lives on the PPU die, not in system RAM. `Cell<u8>` is layout and
    /// cost-identical to `u8`, so OAM DMA can write through the bus handler
    /// while the PPU is mid-scan.
    oam: [Cell<u8>; 160],

    front: RefCell<Box<Frame>>,
    back: RefCell<Box<Frame>>,
    frame_ready: Cell<bool>,

    window_y_reached: Cell<bool>,
    window_ly: Cell<u8>,
}

impl Ppu {
    pub fn new(interrupt_request: InterruptRequest) -> Self {
        Self {
            registers: RefCell::default(),
            interrupt_request,
            vram: core::array::from_fn(|_| [const { Cell::new(0) }; 0x2000]),
            oam: [const { Cell::new(0) }; 160],
            front: RefCell::new(Box::new([0u8; SCREEN_W * SCREEN_H])),
            back: RefCell::new(Box::new([0u8; SCREEN_W * SCREEN_H])),
            frame_ready: Cell::new(false),
            window_y_reached: Cell::new(false),
            window_ly: Cell::new(0),
        }
    }

    pub async fn task(this: Rc<Self>, timeline: Timeline) -> Infallible {
        loop {
            this.window_y_reached.set(false);
            for ly in 0..144 {
                // Mode 2: OAM scan
                {
                    let mut registers = this.registers.borrow_mut();
                    registers.set_ly(ly);
                    registers.set_mode(PpuMode::OamScan);
                }
                // async 80 dots
                let objects = this.oam_scan(ly, &timeline).await;

                // Mode 3: pixel draw
                {
                    let mut registers = this.registers.borrow_mut();
                    registers.set_mode(PpuMode::Drawing);
                    // latch wy reached for the rest of the frame
                    this.window_y_reached
                        .set(this.window_y_reached.get() | (ly == registers.wy()));
                }
                let dots = this.draw_line(ly, &objects, &timeline).await;

                // Mode 0: HBlank
                {
                    let mut registers = this.registers.borrow_mut();
                    registers.set_mode(PpuMode::HBlank);
                }
                timeline.wait(376 - dots).await;
            }

            // Mode 1: VBlank
            this.registers.borrow_mut().set_mode(PpuMode::VBlank);
            this.present_frame();
            this.interrupt_request.vblank(true);
            for ly in 144..154 {
                this.registers.borrow_mut().set_ly(ly);
                timeline.wait(456).await;
            }
        }
    }

    // Will be useful for the rendering passes
    fn object(&self, index: usize) -> ObjectAttribute {
        let base = index * 4;
        ObjectAttribute(u32::from_le_bytes([
            self.oam[base].get(),
            self.oam[base + 1].get(),
            self.oam[base + 2].get(),
            self.oam[base + 3].get(),
        ]))
    }

    fn scanline_mut(&self, ly: u8) -> RefMut<'_, [u8]> {
        let start = ly as usize * SCREEN_W;
        // returns a new RefMut from the passed RefMut mpped with the closure
        // Here it returns a RefMut of a scanline from the back frame buffer
        RefMut::map(self.back.borrow_mut(), |back| {
            &mut back[start..start + SCREEN_W]
        })
    }

    fn present_frame(&self) {
        std::mem::swap(&mut self.back.borrow_mut(), &mut self.front.borrow_mut());
        self.frame_ready.set(true);
    }

    pub fn framebuffer(&self) -> Ref<'_, Frame> {
        // Derefs the inner &Box<Frame> to a Box<Frame>, then derefs that Box<Frame> into a
        // Frame and return a reference to it
        Ref::map(self.front.borrow(), |front| &**front)
    }

    pub fn take_frame_ready(&self) -> bool {
        self.frame_ready.replace(false)
    }

    async fn oam_scan(&self, ly: u8, timeline: &Timeline) -> Vec<ScannedObject> {
        let mut selected = Vec::with_capacity(10);
        for i in 0..40 {
            timeline.wait(2).await;
            if selected.len() >= 10 {
                continue;
            }
            let obj = self.object(i);
            let top = obj.y();
            // if this overflows, object isn't visible anyway (y >= 160)
            let (bottom, overflowed) =
                top.overflowing_add(self.registers.borrow().lcdc().obj_size.as_height());
            // LY's 0 is OBJ.y's 16
            let offset_ly = ly + 16;
            // OBJ has pixel on LY
            if !overflowed && (top..bottom).contains(&offset_ly) {
                selected.push(ScannedObject {
                    attributes: obj,
                    index: i,
                });
            }
        }
        selected
    }

    async fn draw_line(&self, ly: u8, _objects: &Vec<ScannedObject>, timeline: &Timeline) -> u64 {
        let mut bg_fifo = FIFOBuffer::default();
        let _obj_fifo = FIFOBuffer::default();

        let lx = 0;
        let mut fetcher_x = 0;

        // let bg_y = ly.wrapping_add(self.registers.borrow().scy());
        // let tile_y = bg_y / 8;
        // let fine_y = bg_y & 0x07;

        let mut window = false;

        loop {
            if !window && self.window_should_start(lx) {
                window = true;
                fetcher_x = 0;
                bg_fifo.clear();
                timeline.wait(6).await;
                continue;
            }
            let (tile_x, tile_y, base_addr) = self.get_tile(window, fetcher_x, ly);
            let tile_addr = base_addr + (tile_y * 32) as u16 + tile_x as u16;

            fetcher_x += 1;
            timeline.wait(2).await;

            let _tile_data = self.get_tile_data(window, tile_addr);
        }
        0
    }

    fn window_should_start(&self, lx: u8) -> bool {
        self.window_y_reached.get()
            && lx >= self.registers.borrow().wx().saturating_sub(7)
            && self.registers.borrow().lcdc().window_enable
    }

    fn get_tile(&self, window: bool, fetcher_x: u8, ly: u8) -> (u8, u8, u16) {
        if window {
            let base_addr = matches!(
                self.registers.borrow().lcdc().window_tilemap_area,
                TileArea::High
            )
            .then_some(0x9C00)
            .unwrap_or(0x9800);
            (fetcher_x, self.window_ly.get() / 8, base_addr)
        } else {
            let base_addr = matches!(
                self.registers.borrow().lcdc().bg_tilemap_area,
                TileArea::High
            )
            .then_some(0x9C00)
            .unwrap_or(0x9800);
            let tile_x = ((self.registers.borrow().scx() / 8) + fetcher_x) & 0x1F;
            let tile_y = ly.wrapping_add(self.registers.borrow().scy()) / 8;
            (tile_x, tile_y, base_addr)
        }
    }

    fn get_tile_data(&self, _window: bool, _tile_addr: u16) -> u8 {
        0
    }
}

impl MemoryHandler for Ppu {
    fn read(&self, _mmu: &Mmu, address: u16) -> MemoryRead {
        let registers = self.registers.borrow();
        match address {
            0x8000..=0x9FFF => {
                let bank = (registers.vram_bank() & 1) as usize;
                let offset = (address - 0x8000) as usize;
                MemoryRead::Replace(self.vram[bank][offset].get())
            }
            0xFE00..=0xFE9F => MemoryRead::Replace(self.oam[(address & 0xFF) as usize].get()),
            _ => MemoryRead::Replace(registers.read(address)),
        }
    }

    fn write(&self, _mmu: &Mmu, address: u16, value: u8) -> MemoryWrite {
        match address {
            0x8000..=0x9FFF => {
                let bank = (self.registers.borrow().vram_bank() & 1) as usize;
                let offset = (address - 0x8000) as usize;
                self.vram[bank][offset].set(value);
            }
            0xFE00..=0xFE9F => self.oam[(address & 0xFF) as usize].set(value),
            _ => self.registers.borrow_mut().write(address, value),
        };
        MemoryWrite::Block
    }
}
