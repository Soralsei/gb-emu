use crate::audio::channel::{
    duty::Duty, envelope::Envelope, period::Period, sweep::Sweep, Channel, ChannelCore, NRx4,
};

#[derive(Debug, Default)]
pub struct Square {
    core: ChannelCore,
    duty: Duty,
    envelope: Envelope,
    period: Period,
    sweep: Option<Sweep>, // CH1 only
}

impl Channel for Square {
    fn core(&self) -> &ChannelCore {
        &self.core
    }

    fn core_mut(&mut self) -> &mut ChannelCore {
        &mut self.core
    }

    fn read(&self, reg: usize) -> u8 {
        todo!()
    }

    fn write(&mut self, reg: usize, value: u8, step: u8) {
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
                self.core.write_control(control, step);
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

    fn power_off(&mut self, is_cgb: bool) {
        todo!()
    }

    fn output(&self) -> u8 {
        todo!()
    }
}
