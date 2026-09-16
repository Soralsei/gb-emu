use std::{
    array,
    ops::{Deref, DerefMut},
};

use crate::graphics::attributes::TilePriority;

#[derive(Debug, Default, Clone, Copy)]
pub struct Pixel {
    pub color: u8,           // 2 bits, value between 0-3
    pub palette: u8,         // 3 bits, value between 0-7
    pub sprite_priority: u8, // OAM index on CGB, not used on DMG
    pub bg_priority: TilePriority,
}

impl Pixel {
    pub fn from_bytes(
        low: u8,
        high: u8,
        flip_x: bool,
        palette: u8,
        sprite_priority: u8,
        bg_priority: TilePriority,
    ) -> [Self; 8] {
        array::from_fn(|i| {
            let bit = if flip_x { i } else { 7 - i };
            Pixel {
                color: ((high >> bit) & 1) << 1 | ((low >> bit) & 1),
                palette,
                sprite_priority,
                bg_priority,
            }
        })
    }
}

#[derive(Debug, Default)]
pub struct FIFOBuffer {
    pixels: [Pixel; 8],
    len: u8,
}

impl FIFOBuffer {
    pub fn clear(&mut self) {
        self.len = 0;
    }

    pub fn fill(&mut self, pixels: [Pixel; 8]) {
        self.pixels = pixels;
        self.len = 8;
    }

    pub fn pop(&mut self) -> Option<Pixel> {
        if self.is_empty() {
            return None;
        }
        let pixel = self.pixels[0];
        self.pixels.rotate_left(1);
        self.pixels[7] = Pixel::default();
        self.len = self.len.saturating_sub(1);
        Some(pixel)
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

#[derive(Debug, Default)]
pub struct BgFIFO(FIFOBuffer);

#[derive(Debug, Default)]
pub struct ObjFIFO(FIFOBuffer);

// Implement Deref so immutable methods (like is_empty) work
impl Deref for BgFIFO {
    type Target = FIFOBuffer;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// Implement DerefMut so mutable methods (like fill, clear, pop) work
impl DerefMut for BgFIFO {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

// Do the same for ObjFIFO...
impl Deref for ObjFIFO {
    type Target = FIFOBuffer;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ObjFIFO {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl ObjFIFO {
    pub fn merge(&mut self, row: [Pixel; 8], clip: u8, is_cgb: bool) {
        for i in clip..8 {
            let slot = &mut self.0.pixels[(i - clip) as usize];
            if slot.color == 0
                || (is_cgb
                    && row[i as usize].color != 0
                    && row[i as usize].sprite_priority < slot.sprite_priority)
            {
                *slot = row[i as usize];
            }
        }
        self.0.len = 8;
    }
}
