#[derive(Debug, Default)]
pub struct Duty {
    step: u16,
    wave_duty: u8,
}

impl Duty {
    pub fn write(&mut self, value: u8) {
        self.wave_duty = (value & 0xC0) >> 6;
    }
}
