use crate::audio::channel::{core_accessors, envelope::Envelope, Channel, ChannelCore};

#[derive(Debug)]
pub struct Wave {
    core: ChannelCore,
    envelope: Envelope,
    wave_ram: [u8; 0x10], // 0xFF30-0xFF3F
}

impl Default for Wave {
    fn default() -> Self {
        Self {
            core: ChannelCore::new(256),
            envelope: Envelope::default(),
            wave_ram: [0; 0x10],
        }
    }
}

impl Wave {
    pub fn clear(&mut self) {}
}

impl Channel for Wave {
    const READ_MASK: [u8; 5] = [0x7F, 0xFF, 0x9F, 0xFF, 0xBF];

    fn read_raw(&self, reg: usize) -> u8 {
        todo!()
    }

    fn write(&mut self, reg: usize, value: u8, step: u8) {
        todo!()
    }

    fn fresh(&self) -> Self {
        Self::default()
    }

    fn clock(&mut self) {
        todo!()
    }

    fn output(&self) -> u8 {
        todo!()
    }

    fn clock_envelope(&mut self) {
        self.envelope.clock();
    }

    core_accessors!();
}
