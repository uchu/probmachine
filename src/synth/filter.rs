use synfx_dsp::process_stilson_moog;

pub struct MoogFilter {
    israte: f32,
    b0: f32,
    b1: f32,
    b2: f32,
    b3: f32,
    delay: [f32; 4],
}

impl MoogFilter {
    pub fn new(sample_rate: f64) -> Self {
        Self {
            israte: 1.0 / sample_rate as f32,
            b0: 0.0,
            b1: 0.0,
            b2: 0.0,
            b3: 0.0,
            delay: [0.0; 4],
        }
    }

    pub fn set_sample_rate(&mut self, sample_rate: f64) {
        self.israte = (1.0 / sample_rate as f32).max(1e-7);
    }

    #[allow(dead_code)]
    pub fn reset(&mut self) {
        self.b0 = 0.0;
        self.b1 = 0.0;
        self.b2 = 0.0;
        self.b3 = 0.0;
        self.delay = [0.0; 4];
    }

    #[inline]
    fn apply_drive(input: f32, drive: f32) -> f32 {
        if drive <= 1.0 {
            return input;
        }
        let x = input * drive;
        let saturated = x.tanh();
        let compensation = 1.0 / drive.tanh();
        saturated * compensation
    }

    pub fn process_buffer(&mut self, buffer: &mut [f32], cutoff: f32, resonance: f32, drive: f32) {
        let nyquist = 0.5 / self.israte;
        let max_freq = nyquist * 0.40;
        let freq = cutoff.clamp(20.0, max_freq);
        let res = resonance.clamp(0.0, 0.98);

        for sample in buffer.iter_mut() {
            let driven_input = Self::apply_drive(*sample, drive);
            *sample = process_stilson_moog(
                driven_input,
                freq,
                res,
                self.israte,
                &mut self.b0,
                &mut self.b1,
                &mut self.b2,
                &mut self.b3,
                &mut self.delay,
            );
        }
    }
}
