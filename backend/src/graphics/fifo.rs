use crate::graphics::object_attributes::ObjectPriority;

#[derive(Debug, Default)]
pub struct Pixel {
    color: u8,              // 2 bits, value between 0-3
    palette: u8,            // 3 bits, value between 0-7
    sprite_priority: usize, // OAM index on CGB, not used on DMG
    bg_priority: ObjectPriority,
}

#[derive(Debug, Default)]
pub struct FIFOBuffer {
    queue: Option<[Pixel; 8]>,
}

impl FIFOBuffer {
    pub fn clear(&mut self) {
        self.queue = None;
    }

    pub fn push(&mut self, pixels: [Pixel; 8]) {
        self.queue = Some(pixels);
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub enum FetchTarget {
    #[default]
    Background,
    Window,
    // Suspends the shifter until the fetch completes
    Object {
        oam_index: u8,
        tile: u8,
        flags: u8,
        x: u8,
    },
}
