use crate::audio::channel::{Channel, ChannelCore};

#[derive(Debug, Default)]
pub struct Wave {
    core: ChannelCore,
    wave_ram: [u8; 0x10], // 0xFF30-0xFF3F
}

impl Wave {
    pub fn clear(&mut self) {}
}

impl Channel for Wave {
    fn core(&self) -> &ChannelCore {
        &self.core
    }

    fn core_mut(&mut self) -> &mut ChannelCore {
        &mut self.core
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

    fn write(&mut self, reg: usize, value: u8, step: u8) {
        todo!()
    }
}
