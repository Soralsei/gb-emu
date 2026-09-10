use crate::{
    cpu::interrupt::InterruptRequest,
    memory::mmu::{MemoryHandler, MemoryRead, MemoryWrite},
};

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

enum Group {
    Buttons,
    Dpad,
}

impl Button {
    fn line(self) -> (Group, u8) {
        match self {
            Button::Start => (Group::Buttons, 0x08),
            Button::Select => (Group::Buttons, 0x04),
            Button::B => (Group::Buttons, 0x02),
            Button::A => (Group::Buttons, 0x01),
            Button::Down => (Group::Dpad, 0x08),
            Button::Up => (Group::Dpad, 0x04),
            Button::Left => (Group::Dpad, 0x02),
            Button::Right => (Group::Dpad, 0x01),
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Joypad {
    select_bits: u8,     // bits 5-4 as written
    buttons_pressed: u8, // bit 3..0 = Start, Select, B, A   (1 = pressed)
    dpad_pressed: u8,    // bit 3..0 = Down, Up, Left, Right
}

impl Joypad {
    pub fn select(&mut self, value: u8) {
        // ignore bits other than 5 and 4
        self.select_bits = value & 0x30;
    }

    pub fn get(&self) -> u8 {
        let mut inputs = 0x0F;
        if self.select_bits & 0x20 == 0 {
            inputs &= !self.buttons_pressed;
        }
        if self.select_bits & 0x10 == 0 {
            inputs &= !self.dpad_pressed;
        }
        0xC0 | self.select_bits | (inputs & 0x0F)
    }

    pub fn set(&mut self, button: Button, down: bool) {
        let (group, btn) = button.line();

        let btn_group = match group {
            Group::Buttons => &mut self.buttons_pressed,
            Group::Dpad => &mut self.dpad_pressed,
        };

        if down {
            *btn_group |= btn;
        } else {
            *btn_group &= !btn;
        }
    }
}

pub struct JoypadHandler {
    interrupt_request: InterruptRequest,
    joypad: Joypad,
}

impl JoypadHandler {
    pub fn new(interrupt_request: InterruptRequest) -> Self {
        Self {
            interrupt_request,
            joypad: Default::default(),
        }
    }

    pub fn set(&mut self, button: Button, down: bool) -> bool {
        self.state_change(|joypad| joypad.set(button, down))
    }

    fn state_change(&mut self, change: impl FnOnce(&mut Joypad)) -> bool {
        let before = self.joypad.get() & 0x0F;
        change(&mut self.joypad);
        let after = self.joypad.get() & 0x0F;
        let fell = before & !after != 0;
        if fell {
            self.interrupt_request.joypad(true);
        }
        fell
    }
}

impl MemoryHandler for JoypadHandler {
    fn read(&self, _: &crate::memory::mmu::Mmu, address: u16) -> crate::memory::mmu::MemoryRead {
        if address == 0xFF00 {
            return MemoryRead::Replace(self.joypad.get());
        }
        unreachable!("Invalid read in JoypadHandler at address 0x{:04X}", address)
    }

    fn write(
        &mut self,
        _: &crate::memory::mmu::Mmu,
        address: u16,
        value: u8,
    ) -> crate::memory::mmu::MemoryWrite {
        if address == 0xFF00 {
            let _ = self.state_change(|joypad| joypad.select(value));
            return MemoryWrite::Block;
        }
        unreachable!(
            "Invalid write in JoypadHandler at address 0x{:04X}, value 0x{:02X}",
            address, value
        )
    }
}
