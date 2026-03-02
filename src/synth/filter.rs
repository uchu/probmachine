use super::simd::{Stereo, stereo, stereo_splat, stereo_left, stereo_right, stereo_sin, stereo_tanh, StereoDCBlocker};

pub struct StereoMoogFilter {
    israte: f64,
    b0: Stereo,
    b1: Stereo,
    b2: Stereo,
    b3: Stereo,
    delay: [Stereo; 4],
    dc_blocker: StereoDCBlocker,
}

impl StereoMoogFilter {
    pub fn new(sample_rate: f64) -> Self {
        Self {
            israte: 1.0 / sample_rate,
            b0: stereo_splat(0.0),
            b1: stereo_splat(0.0),
            b2: stereo_splat(0.0),
            b3: stereo_splat(0.0),
            delay: [stereo_splat(0.0); 4],
            dc_blocker: StereoDCBlocker::new(),
        }
    }

    pub fn set_sample_rate(&mut self, sample_rate: f64) {
        self.israte = (1.0 / sample_rate).max(1e-10);
    }

    pub fn reset(&mut self) {
        self.b0 = stereo_splat(0.0);
        self.b1 = stereo_splat(0.0);
        self.b2 = stereo_splat(0.0);
        self.b3 = stereo_splat(0.0);
        self.delay = [stereo_splat(0.0); 4];
        self.dc_blocker.reset();
    }

    #[inline(always)]
    fn apply_drive(input: Stereo, drive: f64) -> Stereo {
        if drive <= 1.0 {
            return input;
        }
        let inv_tanh_drive = 1.0 / drive.tanh();
        let l = (stereo_left(input) * drive).tanh() * inv_tanh_drive;
        let r = (stereo_right(input) * drive).tanh() * inv_tanh_drive;
        stereo(l, r)
    }

    #[inline(always)]
    fn saturate(x: Stereo) -> Stereo {
        stereo_tanh(x)
    }

    #[inline(always)]
    pub fn process_sample(&mut self, input: Stereo, freq_l: f64, freq_r: f64, res: f64, drive: f64) -> Stereo {
        let driven = Self::apply_drive(input, drive);

        let israte_v = stereo_splat(self.israte);
        let freq_v = stereo(freq_l, freq_r);
        let cutoff = stereo_splat(2.0) * freq_v * israte_v;
        let p = cutoff * (stereo_splat(1.8) - stereo_splat(0.8) * cutoff);
        let k = stereo_splat(2.0) * stereo_sin(cutoff * stereo_splat(std::f64::consts::PI * 0.5)) - stereo_splat(1.0);

        let t1 = (stereo_splat(1.0) - p) * stereo_splat(1.386249);
        let t2 = stereo_splat(12.0) + t1 * t1;
        let res_v = stereo_splat(res) * (t2 + stereo_splat(6.0) * t1) / (t2 - stereo_splat(6.0) * t1);

        let x = driven - res_v * self.b3;

        self.b0 = x * p + self.delay[0] * p - k * self.b0;
        self.b0 = Self::saturate(self.b0);

        self.b1 = self.b0 * p + self.delay[1] * p - k * self.b1;
        self.b1 = Self::saturate(self.b1);

        self.b2 = self.b1 * p + self.delay[2] * p - k * self.b2;
        self.b2 = Self::saturate(self.b2);

        self.b3 = self.b2 * p + self.delay[3] * p - k * self.b3;
        self.b3 = Self::saturate(self.b3);

        self.delay[0] = x;
        self.delay[1] = self.b0;
        self.delay[2] = self.b1;
        self.delay[3] = self.b2;

        self.dc_blocker.process(self.b3)
    }
}
