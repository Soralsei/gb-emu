#![allow(dead_code)]
use crate::is_bit_set;

pub trait Channel {
    fn core(&self) -> &ChannelCore;
    fn core_mut(&mut self) -> &mut ChannelCore;
    fn read(&self, reg: usize) -> u8;
    fn write(&mut self, reg: usize, value: u8, step: u8); // step: NRx4 quirks
    fn power_off(&mut self, is_cgb: bool);
    fn output(&self) -> u8;

    fn enabled(&self) -> bool {
        self.core().enabled
    }
    fn write_length(&mut self, value: u8) {
        self.core_mut().length.load(value)
    }
    fn clock_length(&mut self) {
        self.core_mut().clock_length()
    }
}

#[derive(Debug, Default)]
struct Length {
    counter: u16,
    enabled: bool,
    max: u16,
}

impl Length {
    pub fn load(&mut self, value: u8) {
        self.counter = self.max - (value as u16 & (self.max - 1));
    }

    pub fn clock(&self) -> bool {
        self.counter > 0
    }
}

#[derive(Debug, Default)]
pub struct ChannelCore {
    enabled: bool,
    dac: bool,
    length: Length,
}

impl ChannelCore {
    pub fn set_dac(&mut self, on: bool) {
        self.dac = on;
        self.enabled &= on;
    }

    pub fn disable(&mut self) {
        self.enabled = false;
    }

    pub fn clock_length(&mut self) {
        if self.length.clock() {
            self.enabled = false;
        }
    }

    /// NRx4 bits 6–7. True if triggered; the channel then runs its own part.
    pub fn write_control(&mut self, value: u8, step: u8) -> bool {
        /* length enable + quirk, trigger */
        true
    }

    pub fn read_control(&self) -> u8 {
        0xBF | (self.length.enabled as u8) << 6
    }
}

#[derive(Debug, Default)]
struct Duty {}

impl Duty {
    pub fn write(&mut self, value: u8) {}
}

#[derive(Debug, Default)]
struct Envelope {}

impl Envelope {
    pub fn write(&mut self, value: u8) {}
    pub fn trigger(&mut self) {}
}

#[derive(Debug, Default)]
struct Period {}

impl Period {
    pub fn reload(&mut self) {}
    pub fn write_high(&mut self, value: u8) {}
    pub fn write_low(&mut self, value: u8) {}
}

#[derive(Debug, Default)]
struct Sweep {}

impl Sweep {
    pub fn write(&mut self, value: u8, core: &mut ChannelCore) {}
    pub fn trigger(&mut self, period: &Period, core: &ChannelCore) {}
}

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
        todo!()
    }

    fn core_mut(&mut self) -> &mut ChannelCore {
        todo!()
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
                self.period.write_high(value); // before trigger: trigger uses the new period
                if self.core.write_control(value, step) {
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

    fn enabled(&self) -> bool {
        todo!()
    }

    fn write_length(&mut self, length: u8) {
        todo!()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum SweepDirection {
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
pub struct Wave {
    enabled: bool,
    wave_ram: [u8; 0x10], // 0xFF30-0xFF3F
}

impl Wave {
    pub fn clear(&mut self) {}
}

impl Channel for Wave {
    fn core(&self) -> &ChannelCore {
        todo!()
    }

    fn core_mut(&mut self) -> &mut ChannelCore {
        todo!()
    }

    fn read(&self, reg: usize) -> u8 {
        todo!()
    }

    fn power_off(&mut self, is_cgb: bool) {
        todo!()
    }

    fn output(&self) -> u8 {
        todo!()
    }

    fn enabled(&self) -> bool {
        todo!()
    }

    fn write_length(&mut self, length: u8) {
        todo!()
    }

    fn write(&mut self, reg: usize, value: u8, step: u8) {
        todo!()
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum LfsrWidth {
    Wide = 0,
    Narrow = 1,
}

impl From<bool> for LfsrWidth {
    fn from(value: bool) -> Self {
        if value {
            LfsrWidth::Narrow
        } else {
            LfsrWidth::Wide
        }
    }
}

/// NR43 : controls frequency of amplitude switching in noise generator
#[derive(Debug, Default, Clone, Copy)]
pub struct RandomControl(u8);

impl RandomControl {
    pub fn clock_shift(&self) -> u8 {
        (self.0 & 0xF0) >> 4
    }
    pub fn lfsr_width(&self) -> LfsrWidth {
        is_bit_set!(self.0, 3).into()
    }
    pub fn div(&self) -> u8 {
        self.0 & 0x07
    }
}

#[derive(Debug, Default)]
pub struct Noise {
    enabled: bool,
    random_control: RandomControl,
}

impl Channel for Noise {
    fn core(&self) -> &ChannelCore {
        todo!()
    }

    fn core_mut(&mut self) -> &mut ChannelCore {
        todo!()
    }

    fn read(&self, reg: usize) -> u8 {
        todo!()
    }

    fn write(&mut self, reg: usize, value: u8, step: u8) {
        todo!()
    }

    fn power_off(&mut self, is_cgb: bool) {
        todo!()
    }

    fn output(&self) -> u8 {
        todo!()
    }

    fn enabled(&self) -> bool {
        todo!()
    }

    fn write_length(&mut self, length: u8) {
        todo!()
    }
}
