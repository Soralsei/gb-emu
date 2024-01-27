// const RAM_SIZE: usize = 0x8000;
// const HRAM_SIZE: usize = 0xFFFF - 0xFF80;

pub struct Mmu {
    // cart: Cart,
    // ppu: Ppu,
    // spu: Spu,
    // timer: Timer,
    // gamepad: Gamepad
    // ram: Box<[u8]>
    // hram: Box<[u8]>
    memory: [u8; 0xffff],
    pub interrupts_enable: u8,
    pub interrupts_flags: u8,
}

impl Mmu {
    pub fn new(/* cart: Cart, ppu: Ppu, spu: Spu, timer: Timer, gamepad: Gamepad */) -> Mmu {
        Mmu {
            /*cart,
            ppu,
            spu,
            timer,
            gamepad,
            ram: vec!([0; RAM_SIZE]).into_boxed_slice(),
            hram: vec!([0; HRAM_SIZE]).into_boxed_slice(),
            */
            memory: [0; 0xffff],
            interrupts_enable: 0,
            interrupts_flags: 0,
        }
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x7FFF => todo!("Implement rom/cart reads"),
            0x8000..=0x9FFF => todo!("Implement vram reads"),
            0xA000..=0xBFFF => todo!("Implement external ram reads"),
            0xC000..=0xCFFF => todo!("Implement rom reads"),
            0xD000..=0xDFFF => todo!("Implement rom reads"),
            0xE000..=0xFDFF => todo!("Implement/ignore echo ram reads"),
            0xFE00..=0xFE9F => todo!("Implement/ignore OAM reads"),
            0xFEA0..=0xFEFF => todo!("Implement/ignore unusable ram reads"),
            0xFF00 => todo!("Implement gamepad read"),
            0xFF01..=0xFF02 => todo!("Implement Serial I/O read"),
            0xFF04..=0xFF07 => todo!("Implement Timer registers read"),
            0xFF10..=0xFF3F => todo!("Implement SPU registers read"),
            0xFF0F => self.interrupts_flags,
            // 0xFE00..=0xfeff | 0xff40..=0xff4b => todo!("Implement Ppu reads"),
            0xFFFF => self.interrupts_enable,
            _ => panic!("[MMU] invalid address 0x{:04x} received", addr),
        }
    }

    pub fn write(&mut self, addr: u16, value: u8) {
        match addr {
            0x0000..=0x7FFF => todo!("Implement rom/cart write"),
            0x8000..=0x9FFF => todo!("Implement vram write"),
            0xA000..=0xBFFF => todo!("Implement external ram write"),
            0xC000..=0xCFFF => todo!("Implement rom write"),
            0xD000..=0xDFFF => todo!("Implement rom write"),
            0xE000..=0xFDFF => todo!("Implement/ignore echo ram write"),
            0xFE00..=0xFE9F => todo!("Implement/ignore OAM write"),
            0xFEA0..=0xFEFF => todo!("Implement/ignore unusable ram write"),
            0xFF00 => todo!("Implement gamepad write"),
            0xFF01..=0xFF02 => todo!("Implement Serial I/O write"),
            0xFF04..=0xFF07 => todo!("Implement Timer registers write"),
            0xFF10..=0xFF3F => todo!("Implement SPU registers write"),
            0xFF0F => self.interrupts_flags,
            // 0xFE00..=0xfeff | 0xff40..=0xff4b => todo!("Implement Ppu reads"),
            0xFFFF => self.interrupts_enable,
            _ => panic!("[MMU] invalid address 0x{:04x} received", addr),
        };
    }
}
