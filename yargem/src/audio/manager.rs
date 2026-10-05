use cpal::{traits::HostTrait, Device, Host};
use yargem_lib::SAMPLE_RATE;

const FRAME_BURST: usize = 70224 / 64; // source frames per run_frame
const JITTER_MARGIN: usize = FRAME_BURST; // raise on crackles
const FALLBACK_CALLBACK_FRAMES: usize = 1024; // until the first callback reports
const CAPACITY: usize = 8192;

pub struct AudioManager {
    host: Host,
    out_device: Option<Device>,
}

impl AudioManager {
    pub fn new() -> Self {
        let host = cpal::default_host();
        let out_device = host.default_output_device();
        Self { host, out_device }
    }

    pub fn devices(&self) -> Result<cpal::Devices, cpal::Error> {
        self.host.devices()
    }

    pub fn set_out_device(&mut self, device: Option<cpal::Device>) {
        self.out_device = device;
    }

    fn target_fill(callback_frames: usize, device_rate: u32) -> usize {
        let callback_burst =
            (callback_frames as f64 * SAMPLE_RATE / device_rate as f64).ceil() as usize;
        (callback_burst + FRAME_BURST + JITTER_MARGIN).min(CAPACITY - FRAME_BURST)
    }
}
