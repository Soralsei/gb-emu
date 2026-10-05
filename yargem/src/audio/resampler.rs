#[derive(Clone, Copy, Default)]
pub struct StereoFrame {
    pub left: f32,
    pub right: f32,
}

pub struct LinearResampler {
    step: f64,
    frac: f64,
    prev: StereoFrame,
    next: StereoFrame,
}

pub trait StereoResampler {
    fn resample(&mut self, pull: impl FnMut() -> Option<StereoFrame>) -> StereoFrame;
}

impl LinearResampler {
    pub fn new(src_rate: u32, tgt_rate: u32) -> Self {
        Self {
            step: src_rate as f64 / tgt_rate as f64,
            frac: 1.0, // pull a real frame on first call
            prev: StereoFrame::default(),
            next: StereoFrame::default(),
        }
    }

    fn advance(&mut self, pull: &mut impl FnMut() -> Option<StereoFrame>) {
        self.prev = self.next;
        if let Some(frame) = pull() {
            self.next = frame;
        }
    }
}

impl StereoResampler for LinearResampler {
    fn resample(&mut self, mut pull: impl FnMut() -> Option<StereoFrame>) -> StereoFrame {
        let crossed = self.frac.floor() as usize;
        for _ in 0..crossed {
            self.advance(&mut pull);
        }
        self.frac -= crossed as f64;
        let t = self.frac as f32;
        self.frac += self.step;

        let left = self.prev.left + t * (self.next.left - self.prev.left);
        let right = self.prev.right + t * (self.next.right - self.prev.right);

        StereoFrame { left, right }
    }
}
