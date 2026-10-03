use crate::{
    audio::channel::core_accessors,
    audio::channel::{
        duty::Duty, envelope::Envelope, period::Period, sweep::Sweep, Channel, ChannelCore, NRx4,
    },
};

#[derive(Debug)]
pub struct Square {
    core: ChannelCore<64>,
    duty: Duty,
    envelope: Envelope,
    period: Period<2>,
    sweep: Option<Sweep>, // CH1 only
}

impl Square {
    /// CH1: the pulse channel with a sweep unit.
    pub fn ch1() -> Self {
        Self {
            sweep: Some(Sweep::default()),
            ..Self::ch2()
        }
    }

    /// CH2: no sweep, so NR20 reads $FF.
    pub fn ch2() -> Self {
        Self {
            core: ChannelCore::new(),
            duty: Duty::default(),
            envelope: Envelope::default(),
            period: Period::default(),
            sweep: None,
        }
    }
}

impl Channel<64> for Square {
    // One table serves both: CH2 has no NR20, and `read_raw` returns $FF for it.
    const READ_MASK: [u8; 5] = [0x80, 0x3F, 0x00, 0xFF, 0xBF];

    fn read_raw(&self, reg: usize) -> u8 {
        debug_assert!(
            reg <= 4,
            "[Square::read_raw] reg value should never be > 4, got {}",
            reg
        );
        match reg {
            0 => self.sweep.as_ref().map_or(0xFF, |sweep| sweep.read()),
            1 => self.duty.wave_duty() << 6,
            2 => self.envelope.read(),
            3 => 0, // write-only
            _ => self.core.read_control(),
        }
    }

    fn write(&mut self, reg: usize, value: u8, do_clock_length: bool) {
        debug_assert!(
            reg <= 4,
            "[Square::write] reg value should never be > 4, got {}",
            reg
        );
        match reg {
            0 => {
                if let Some(sweep) = &mut self.sweep {
                    sweep.write(value, &mut self.core)
                }
            }
            1 => {
                self.duty.write(value);
                self.core.length.load(value)
            }
            2 => {
                self.envelope.write(value);
                self.core.set_dac(value & 0xF8 != 0)
            }
            3 => self.period.write_low(value),
            _ => {
                let control = NRx4(value);
                // before trigger: trigger uses the new period
                self.period.write_high(control.period());
                self.core.write_control(control, do_clock_length);
                if control.trigger() {
                    self.envelope.trigger();
                    self.period.reload();
                    if let Some(sweep) = &mut self.sweep {
                        sweep.trigger(&self.period, &mut self.core);
                    }
                }
            }
        }
    }

    fn fresh(&self) -> Self {
        if self.sweep.is_some() {
            Self::ch1()
        } else {
            Self::ch2()
        }
    }

    fn clock(&mut self) {
        if self.enabled() && self.period.clock() {
            self.duty.clock();
        }
    }

    fn output(&self) -> u8 {
        if self.enabled() && self.duty.output() {
            self.envelope.volume()
        } else {
            0
        }
    }

    fn clock_sweep(&mut self) {
        if let Some(sweep) = &mut self.sweep {
            sweep.clock(&mut self.period, &mut self.core);
        }
    }

    fn clock_envelope(&mut self) {
        self.envelope.clock();
    }

    core_accessors!(64);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_unwritten_square_reads_its_mask() {
        let ch1 = Square::ch1();
        let regs: Vec<u8> = (0..5).map(|reg| ch1.read(reg)).collect();
        assert_eq!(regs, [0x80, 0x3F, 0x00, 0xFF, 0xBF]);
        assert_eq!(Square::ch2().read(0), 0xFF, "CH2 has no NR20");
    }

    #[test]
    fn readable_bits_come_back_through_the_mask() {
        let mut ch2 = Square::ch2();
        ch2.write(1, 0x80, false); // duty 2; the length bits are write-only
        ch2.write(4, 0x40, false); // length enable, no trigger
        assert_eq!(ch2.read(1), 0xBF);
        assert_eq!(ch2.read(4), 0xFF);
    }
}
