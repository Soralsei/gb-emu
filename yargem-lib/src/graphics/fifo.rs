use std::array;

use crate::{graphics::attributes::TilePriority, util::containers::CircularBuffer};

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

pub type BgFIFO = CircularBuffer<Pixel, 8>;

#[derive(Debug, Default)]
pub struct ObjFIFO(CircularBuffer<Pixel, 8>);

impl ObjFIFO {
    pub fn pop(&mut self) -> Option<Pixel> {
        self.0.pop()
    }

    pub fn merge(&mut self, row: [Pixel; 8], clip: u8, is_cgb: bool) {
        for (slot_idx, px) in row.into_iter().skip(clip as usize).enumerate() {
            match self.0.get_mut(slot_idx) {
                Some(slot) => {
                    let wins = px.color != 0
                        && (slot.color == 0
                            || (is_cgb && px.sprite_priority < slot.sprite_priority));
                    if wins {
                        *slot = px;
                    }
                }
                None => self
                    .0
                    .push(px)
                    .expect("merge should never exceed fifo capacity"),
            }
        }
    }
}
