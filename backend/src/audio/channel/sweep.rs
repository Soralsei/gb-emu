use crate::{
    audio::channel::{period::Period, ChannelCore},
    is_bit_set,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
#[repr(u8)]
pub enum SweepDirection {
    #[default]
    Increasing = 0,
    Decreasing = 1,
}

impl From<bool> for SweepDirection {
    fn from(value: bool) -> Self {
        if value {
            SweepDirection::Decreasing
        } else {
            SweepDirection::Increasing
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct SweepControl(u8);

impl SweepControl {
    // bits 4-6
    pub fn pace(&self) -> u8 {
        (self.0 & 0x70) >> 4
    }

    // bit 3
    pub fn direction(&self) -> SweepDirection {
        is_bit_set!(self.0, 3).into()
    }

    // bit 0-2
    pub fn individual_step(&self) -> u8 {
        self.0 & 0x7
    }
}

#[derive(Debug, Default)]
pub struct Sweep {
    shadow_freq: u16,
    enabled: bool,
    sweep_control: SweepControl,
    counter: u8,
    negated: bool,
}

impl Sweep {
    pub fn clock(&mut self, period: &mut Period, core: &mut ChannelCore) {
        self.counter = self.counter.saturating_sub(1);
        if self.counter != 0 {
            return;
        }
        self.reload();
        if !self.enabled || self.sweep_control.pace() == 0 {
            return;
        }

        let next = self.calculate(core);
        if next <= 0x7FF && self.sweep_control.individual_step() != 0 {
            self.shadow_freq = next;
            period.write_low(next as u8);
            period.write_high((next >> 8) as u8);

            self.calculate(core);
        }
    }

    fn calculate(&mut self, core: &mut ChannelCore) -> u16 {
        let freq_shift = self.shadow_freq >> self.sweep_control.individual_step();
        let next = match self.sweep_control.direction() {
            SweepDirection::Increasing => self.shadow_freq + freq_shift,
            SweepDirection::Decreasing => {
                self.negated = true;
                self.shadow_freq - freq_shift
            }
        };
        if next > 0x7FF {
            core.disable();
        }
        next
    }

    pub fn write(&mut self, value: u8, core: &mut ChannelCore) {
        let prev_direction = self.sweep_control.direction();
        self.sweep_control = SweepControl(value);
        // Obscure behavior:
        // Clearing the sweep negate mode bit in NR10 after at least one sweep calculation
        // has been made using the negate mode since the last trigger causes the channel to
        // be immediately disabled. This prevents you from having the sweep lower the frequency
        // then raise the frequency without a trigger inbetween.
        if prev_direction == SweepDirection::Decreasing
            && self.sweep_control.direction() == SweepDirection::Increasing
            && self.negated
        {
            core.disable();
        }
    }

    fn reload(&mut self) {
        self.counter = match self.sweep_control.pace() {
            0 => 8,
            pace => pace,
        };
    }

    pub fn trigger(&mut self, period: &Period, core: &mut ChannelCore) {
        self.shadow_freq = period.freq();
        self.negated = false;
        self.reload();
        self.enabled = self.sweep_control.pace() != 0 || self.sweep_control.individual_step() != 0;

        if self.sweep_control.individual_step() != 0 {
            self.calculate(core);
        }
    }

    pub fn read(&self) -> u8 {
        self.sweep_control.0
    }
}
