use super::simd::{Stereo, stereo, stereo_splat, stereo_left, stereo_right};

pub struct StereoMoogFilter {
    israte: f64,
    b0: Stereo,
    b1: Stereo,
    b2: Stereo,
    b3: Stereo,
    delay: [Stereo; 4],
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
        }
    }

    pub fn set_sample_rate(&mut self, sample_rate: f64) {
        self.israte = (1.0 / sample_rate).max(1e-10);
    }

    #[allow(dead_code)]
    pub fn reset(&mut self) {
        self.b0 = stereo_splat(0.0);
        self.b1 = stereo_splat(0.0);
        self.b2 = stereo_splat(0.0);
        self.b3 = stereo_splat(0.0);
        self.delay = [stereo_splat(0.0); 4];
    }

    #[inline(always)]
    fn apply_drive_simd(input: Stereo, drive: f64) -> Stereo {
        if drive <= 1.0 {
            return input;
        }
        let l = stereo_left(input);
        let r = stereo_right(input);
        let driven_l = (l * drive).tanh() / drive.tanh();
        let driven_r = (r * drive).tanh() / drive.tanh();
        stereo(driven_l, driven_r)
    }

    #[inline(always)]
    pub fn process_sample(&mut self, input: Stereo, freq: f64, res: f64, drive: f64) -> Stereo {
        let driven = Self::apply_drive_simd(input, drive);

        let cutoff = 2.0 * freq * self.israte;
        let p = cutoff * (1.8 - 0.8 * cutoff);
        let k = 2.0 * (cutoff * std::f64::consts::PI * 0.5).sin() - 1.0;

        let t1 = (1.0 - p) * 1.386249;
        let t2 = 12.0 + t1 * t1;
        let res_scaled = res * (t2 + 6.0 * t1) / (t2 - 6.0 * t1);

        let p_v = stereo_splat(p);
        let k_v = stereo_splat(k);
        let res_v = stereo_splat(res_scaled);

        let x = driven - res_v * self.b3;

        self.b0 = x * p_v + self.delay[0] * p_v - k_v * self.b0;
        self.b1 = self.b0 * p_v + self.delay[1] * p_v - k_v * self.b1;
        self.b2 = self.b1 * p_v + self.delay[2] * p_v - k_v * self.b2;
        self.b3 = self.b2 * p_v + self.delay[3] * p_v - k_v * self.b3;

        self.b3 = self.b3 - (self.b3 * self.b3 * self.b3) * stereo_splat(0.166667);

        self.delay[0] = x;
        self.delay[1] = self.b0;
        self.delay[2] = self.b1;
        self.delay[3] = self.b2;

        self.b3
    }

    #[allow(dead_code)]
    pub fn process_buffers(
        &mut self,
        buffer_l: &mut [f32],
        buffer_r: &mut [f32],
        cutoff: f64,
        resonance: f64,
        drive: f64,
    ) {
        let nyquist = 0.5 / self.israte;
        let max_freq = nyquist * 0.40;
        let freq = cutoff.clamp(20.0, max_freq);
        let res = resonance.clamp(0.0, 0.98);

        let len = buffer_l.len().min(buffer_r.len());
        for i in 0..len {
            let input = stereo(buffer_l[i] as f64, buffer_r[i] as f64);
            let output = self.process_sample(input, freq, res, drive);
            buffer_l[i] = stereo_left(output) as f32;
            buffer_r[i] = stereo_right(output) as f32;
        }
    }
}
