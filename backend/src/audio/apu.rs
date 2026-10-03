use std::{cell::RefCell, convert::Infallible, ops::Add, rc::Rc};

use crate::{
    audio::{
        channel::{Channel, Noise, Square, Wave},
        registers::AudioRegisters,
    },
    clock::{Pulse, Timeline},
    is_bit_set,
    memory::mmu::{MemoryHandler, MemoryRead, MemoryWrite, Mmu},
};

/// Runs `$body` once per channel, with `$ch` bound to each concrete field:
/// static dispatch, no `dyn`.
macro_rules! each_channel {
    ($state:expr, $ch:ident => $body:expr) => {{
        {
            let $ch = &mut $state.ch1;
            $body;
        }
        {
            let $ch = &mut $state.ch2;
            $body;
        }
        {
            let $ch = &mut $state.ch3;
            $body;
        }
        {
            let $ch = &mut $state.ch4;
            $body;
        }
    }};
}

#[derive(Debug)]
struct BoxFilterf {
    acc: f32,
    count: usize,
}

impl BoxFilterf {
    pub fn new() -> Self {
        Self { acc: 0.0, count: 0 }
    }

    pub fn add(&mut self, value: f32) {
        self.acc += value;
        self.count += 1;
    }

    pub fn output(&mut self) -> f32 {
        let acc = std::mem::take(&mut self.acc);
        let count = self.count;
        self.count = 0;
        acc / (count as f32)
    }
}

const BASE_FACTOR: f32 = 0.999958;
const BASE_FACTOR_CGB: f32 = 0.998943;

#[derive(Debug)]
struct ApuState {
    is_cgb: bool,
    is_powered: bool,
    registers: AudioRegisters,
    sequencer_step: u8,

    ch1: Square,
    ch2: Square,
    ch3: Wave,
    ch4: Noise,

    high_pass_cap_left: f32,
    high_pass_cap_right: f32,
    charge_factor: f32,

    left_acc: BoxFilterf,
    right_acc: BoxFilterf,

    sample_buff_left: f32,
    sample_buff_right: f32,
}

pub struct Apu {
    state: RefCell<ApuState>,
}

impl Apu {
    pub fn new(is_cgb: bool) -> Self {
        Self {
            state: RefCell::new(ApuState::new(is_cgb)),
        }
    }

    fn split(address: u16) -> (usize, usize) {
        debug_assert!(
            address > 0xFF0F && address < 0xFF24,
            "Apu address split -> channel/reg only valid within 0xFF10-0xFF23, got 0x{:04X}",
            address
        );
        let offset = (address - 0xFF10) as usize;
        (offset / 5, offset % 5)
    }

    /// High pass filter
    fn hpf(signal_in: f32, dac_enabled: bool, cap: &mut f32, discharge_rate: f32) -> f32 {
        let mut out: f32 = 0.0;
        if dac_enabled {
            out = signal_in - *cap;
            *cap = signal_in - out * discharge_rate;
        }
        out
    }

    pub async fn frame_sequencer_task(this: Rc<Self>, div_apu: Pulse) -> Infallible {
        loop {
            div_apu.next().await;
            this.state.borrow_mut().frame_step();
        }
    }

    pub async fn generator_task(this: Rc<Apu>, timeline: Timeline) -> Infallible {
        let mut step_counter = 0;
        loop {
            timeline.wait(2).await;
            this.state.borrow_mut().generator_step();
            if step_counter == 31 {
                this.state.borrow_mut().buffer_sample();
            }
            step_counter = (step_counter + 1) & 0x1F;
        }
    }
}

impl ApuState {
    fn new(is_cgb: bool) -> Self {
        let charge_factor = if is_cgb {
            BASE_FACTOR_CGB.powi(64)
        } else {
            BASE_FACTOR.powi(64)
        };
        Self {
            is_cgb,
            is_powered: false,
            registers: AudioRegisters::default(),
            sequencer_step: 0u8,
            ch1: Square::ch1(),
            ch2: Square::ch2(),
            ch3: Wave::default(),
            ch4: Noise::default(),
            high_pass_cap_left: 0.0,
            high_pass_cap_right: 0.0,
            charge_factor,
            left_acc: BoxFilterf::new(),
            right_acc: BoxFilterf::new(),
            sample_buff_left: 0.0,
            sample_buff_right: 0.0,
        }
    }

    fn read(&self, channel: usize, reg: usize) -> u8 {
        match channel {
            0 => self.ch1.read(reg),
            1 => self.ch2.read(reg),
            2 => self.ch3.read(reg),
            _ => self.ch4.read(reg),
        }
    }

    fn write(&mut self, channel: usize, reg: usize, value: u8) {
        let will_clock_next = self.will_clock_length_next();
        match channel {
            0 => self.ch1.write(reg, value, !will_clock_next),
            1 => self.ch2.write(reg, value, !will_clock_next),
            2 => self.ch3.write(reg, value, !will_clock_next),
            _ => self.ch4.write(reg, value, !will_clock_next),
        }
    }

    fn read_nr52(&self) -> u8 {
        0b01110000
            | (self.is_powered as u8) << 7
            | (self.ch4.enabled() as u8) << 3
            | (self.ch3.enabled() as u8) << 2
            | (self.ch2.enabled() as u8) << 1
            | (self.ch1.enabled() as u8)
    }

    fn write_nr52(&mut self, value: u8) {
        let power = is_bit_set!(value, 7);
        match (self.is_powered, power) {
            (true, false) => self.power_off(),
            (false, true) => self.power_on(),
            _ => {}
        }
        self.is_powered = power;
        // bits 0-3 are read-only and depend only on individual channel enabled state
    }

    fn power_on(&mut self) {
        self.sequencer_step = 0;
    }

    fn power_off(&mut self) {
        self.registers = AudioRegisters::default();

        let is_cgb = self.is_cgb;
        each_channel!(self, ch => ch.power_off(is_cgb));
    }

    fn will_clock_length_next(&self) -> bool {
        // next step will clock if it is even
        self.sequencer_step % 2 == 0
    }

    pub fn frame_step(&mut self) {
        if !self.is_powered {
            return;
        }
        let step = self.sequencer_step;
        // even steps
        if step % 2 == 0 {
            each_channel!(self, ch => ch.clock_length());
        }

        // steps 2 and 6
        if step == 2 || step == 6 {
            each_channel!(self, ch => ch.clock_sweep());
        }

        // last step
        if step == 7 {
            each_channel!(self, ch => ch.clock_envelope());
        }

        self.sequencer_step = (step + 1) & 0x7;
    }

    // Reads the channel analog and accumulates then in sample_buff_left/right
    fn mix(&mut self) {
        let panning = self.registers.panning();
        let master_volume = self.registers.master_volume();
        let outputs = [
            self.ch1.analog_output(),
            self.ch2.analog_output(),
            self.ch3.analog_output(),
            self.ch4.analog_output(),
        ];

        // TODO: Add VIN mixing

        let left = outputs
            .iter()
            .enumerate()
            .map(|(i, output)| if panning.left(i) { *output } else { 0.0 })
            .sum::<f32>()
            * (master_volume.volume_left() + 1) as f32;
        let right = outputs
            .iter()
            .enumerate()
            .map(|(i, output)| if panning.right(i) { *output } else { 0.0 })
            .sum::<f32>()
            * (master_volume.volume_right() + 1) as f32;

        self.left_acc.add(left);
        self.right_acc.add(right);
    }

    pub fn generator_step(&mut self) {
        each_channel!(self, ch => ch.clock());
        self.mix();
    }

    pub fn buffer_sample(&mut self) {
        let any_dac = self.ch1.dac_enabled()
            || self.ch2.dac_enabled()
            || self.ch3.dac_enabled()
            || self.ch4.dac_enabled();

        // Linear up to the filter, so scaling here equals scaling every tick.
        let left = self.left_acc.output() / 32.0;
        let right = self.right_acc.output() / 32.0;

        // Clamp after the filter: its input is already within ±1, the
        // overshoot on a DC step happens inside it.
        self.sample_buff_left = Apu::hpf(
            left,
            any_dac,
            &mut self.high_pass_cap_left,
            self.charge_factor,
        )
        .clamp(-1.0, 1.0);
        self.sample_buff_right = Apu::hpf(
            right,
            any_dac,
            &mut self.high_pass_cap_right,
            self.charge_factor,
        )
        .clamp(-1.0, 1.0);
    }
}

impl MemoryHandler for Apu {
    fn read(&self, _: &Mmu, address: u16) -> MemoryRead {
        match address {
            0xFF26 => MemoryRead::Replace(self.state.borrow().read_nr52()),
            0xFF10..=0xFF23 => {
                let (channel, reg) = Apu::split(address);
                MemoryRead::Replace(self.state.borrow().read(channel, reg))
            }
            0xFF27..=0xFF2F => MemoryRead::Replace(0xFF),
            0xFF30..=0xFF3F => MemoryRead::Replace(self.state.borrow().ch3.read_wave(address)),
            _ => MemoryRead::Replace(self.state.borrow().registers.read(address)),
        }
    }

    fn write(&self, _: &Mmu, address: u16, value: u8) -> MemoryWrite {
        let mut state = self.state.borrow_mut();
        match (state.is_powered, address) {
            (_, 0xFF26) => {
                state.write_nr52(value);
            }
            (_, 0xFF30..=0xFF3F) => state.ch3.write_wave(address, value),
            (powered, 0xFF10..=0xFF23) => {
                let (channel, reg) = Apu::split(address);
                if powered || (!state.is_cgb && reg == 1) {
                    state.write(channel, reg, value);
                }
            }
            (true, 0xFF24..=0xFF25) => {
                state.registers.write(address, value);
            }
            _ => {}
        }
        MemoryWrite::Block
    }
}
