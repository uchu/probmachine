pub struct MasterLimiter {
    delay_buffer: Vec<f32>,
    buffer_size: usize,
    write_pos: usize,
    current_gain: f32,
    attack_coeff: f32,
    release_coeff: f32,
    threshold: f32,
    knee_width: f32,
    lookahead_samples: usize,
}

impl MasterLimiter {
    pub fn new(sample_rate: f32) -> Self {
        let lookahead_ms = 1.5;
        let lookahead_samples = ((sample_rate * lookahead_ms / 1000.0) as usize).max(1);
        let buffer_size = lookahead_samples * 2;

        Self {
            delay_buffer: vec![0.0; buffer_size],
            buffer_size,
            write_pos: 0,
            current_gain: 1.0,
            attack_coeff: 1.0 - (-1.0 / (sample_rate * 0.0001)).exp(),
            release_coeff: 1.0 - (-1.0 / (sample_rate * 0.1)).exp(),
            threshold: 0.99,
            knee_width: 0.1,
            lookahead_samples,
        }
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        let lookahead_ms = 1.5;
        let lookahead_samples = ((sample_rate * lookahead_ms / 1000.0) as usize).max(1);
        let buffer_size = lookahead_samples * 2;

        self.delay_buffer.resize(buffer_size, 0.0);
        self.delay_buffer.fill(0.0);
        self.buffer_size = buffer_size;
        self.write_pos = 0;
        self.current_gain = 1.0;
        self.attack_coeff = 1.0 - (-1.0 / (sample_rate * 0.0001)).exp();
        self.release_coeff = 1.0 - (-1.0 / (sample_rate * 0.1)).exp();
        self.lookahead_samples = lookahead_samples;
    }

    #[inline]
    fn compute_gain_soft_knee(&self, peak: f32) -> f32 {
        if peak <= 0.0 {
            return 1.0;
        }

        let knee_start = self.threshold - self.knee_width;

        if peak <= knee_start {
            1.0
        } else if peak >= self.threshold {
            self.threshold / peak
        } else {
            let t = (peak - knee_start) / self.knee_width;
            let target_gain = self.threshold / peak;
            1.0 - t * t * (1.0 - target_gain)
        }
    }

    pub fn process_block(&mut self, left: &mut [f32], right: &mut [f32]) {
        for i in 0..left.len() {
            self.delay_buffer[self.write_pos] = left[i];
            self.delay_buffer[self.write_pos + 1] = right[i];

            let read_pos = if self.write_pos >= self.lookahead_samples * 2 {
                self.write_pos - self.lookahead_samples * 2
            } else {
                self.buffer_size - (self.lookahead_samples * 2 - self.write_pos)
            };

            let delayed_l = self.delay_buffer[read_pos];
            let delayed_r = self.delay_buffer[read_pos + 1];

            let peak = left[i].abs().max(right[i].abs());
            let target_gain = self.compute_gain_soft_knee(peak);

            let coeff = if target_gain < self.current_gain {
                self.attack_coeff
            } else {
                self.release_coeff
            };
            self.current_gain += (target_gain - self.current_gain) * coeff;

            left[i] = delayed_l * self.current_gain;
            right[i] = delayed_r * self.current_gain;

            self.write_pos = (self.write_pos + 2) % self.buffer_size;
        }
    }
}
