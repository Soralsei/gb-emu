#[derive(Debug, Default)]
pub struct Period {
    counter: u16,
    period_low: u8,
    period_high: u8,
}

impl Period {
    pub fn tick(&mut self) -> bool {
        let previous = self.counter;
        self.counter = (self.counter + 1) & 0x800;
        // overflowed if previous is greater than current
        previous > self.counter
    }

    pub fn value(&self) -> u16 {
        self.counter
    }

    pub fn reload(&mut self) {
        self.counter = ((self.period_high as u16) << 8) | (self.period_low as u16)
    }

    pub fn write_high(&mut self, value: u8) {
        self.period_high = value;
    }

    pub fn write_low(&mut self, value: u8) {
        self.period_low = value;
    }
}
