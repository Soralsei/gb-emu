use crate::cpu::interrupt::InterruptRequest;

enum Mode {
    OAM,
    
}

pub struct Ppu {
    interrupt_request: InterruptRequest,
    clock: u32,
}