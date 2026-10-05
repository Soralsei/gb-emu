use crate::is_bit_set;

const WAVE_DUTY_PATTERN: [u8; 4] = [
    0b00000001, // 12.5%
    0b10000001, // 25%
    0b10000111, // 50%
    0b01111110, // 75%
];

#[derive(Debug, Default)]
pub struct Duty {
    step: u8,
    wave_duty: u8,
}

impl Duty {
    pub fn clock(&mut self) {
        self.step = self.step.wrapping_add(1) & 0x7;
    }

    pub fn output(&self) -> bool {
        is_bit_set!(
            WAVE_DUTY_PATTERN[(self.wave_duty & 0x3) as usize],
            7 - self.step
        )
    }

    pub fn write(&mut self, value: u8) {
        self.wave_duty = value >> 6;
    }

    pub fn wave_duty(&self) -> u8 {
        self.wave_duty
    }
}
