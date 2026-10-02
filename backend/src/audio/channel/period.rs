#[derive(Debug, Default)]
pub struct Period {
    divider: u16,
    period_low: u8,
    period_high: u8,
}

impl Period {
    /// Ticks the channel timer divider an returns true on timer output
    pub fn clock(&mut self) -> bool {
        let previous = self.divider;
        self.divider = (self.divider + 1) & 0x7FF;
        // overflowed if new value is smaller than previous
        if previous > self.divider {
            self.reload();
            true
        } else {
            false
        }
    }

    pub fn div(&self) -> u16 {
        self.divider
    }

    pub fn reload(&mut self) {
        self.divider = ((self.period_high as u16) << 8) | (self.period_low as u16)
    }

    pub fn write_high(&mut self, value: u8) {
        self.period_high = value;
    }

    pub fn write_low(&mut self, value: u8) {
        self.period_low = value;
    }

    pub fn freq(&self) -> u16 {
        (self.period_high as u16) << 8 | self.period_low as u16
    }
}
