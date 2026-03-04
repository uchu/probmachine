use std::f64::consts::{FRAC_1_SQRT_2, PI};

const MAX_DELAY_SECONDS: f64 = 2.0;
const FEEDBACK_HPF_DEFAULT: f64 = 80.0;
const FEEDBACK_LPF_DEFAULT: f64 = 12000.0;

pub struct PingPongDelay {
    buffer_l: Vec<f64>,
    buffer_r: Vec<f64>,
    buffer_size: usize,
    write_pos: usize,
    sample_rate: f64,
    enabled: bool,

    target_delay_samples: f64,
    delay_samples_slew: f64,
    delay_slew_coeff: f64,

    target_feedback: f64,
    feedback_slew: f64,
    target_mix: f64,
    mix_slew: f64,
    target_spread: f64,
    spread_slew: f64,
    param_slew_coeff: f64,

    tempo_sync: bool,
    sync_division: i32,
    free_time_ms: f64,
    bpm: f64,

    hpf_freq: f64,
    lpf_freq: f64,
    hpf_ic1eq_l: f64,
    hpf_ic2eq_l: f64,
    hpf_ic1eq_r: f64,
    hpf_ic2eq_r: f64,
    hpf_g: f64,
    hpf_k: f64,
    hpf_a1: f64,
    hpf_a2: f64,
    hpf_a3: f64,

    lpf_ic1eq_l: f64,
    lpf_ic2eq_l: f64,
    lpf_ic1eq_r: f64,
    lpf_ic2eq_r: f64,
    lpf_g: f64,
    lpf_k: f64,
    lpf_a1: f64,
    lpf_a2: f64,
    lpf_a3: f64,

}

impl PingPongDelay {
    pub fn new(sample_rate: f32) -> Self {
        let sr = sample_rate as f64;
        let buffer_size = (sr * MAX_DELAY_SECONDS) as usize + 16;
        let param_slew_coeff = 1.0 - (-1.0 / (sr * 0.005)).exp();
        let delay_slew_coeff = 1.0 - (-1.0 / (sr * 0.05)).exp();

        let mut s = Self {
            buffer_l: vec![0.0; buffer_size],
            buffer_r: vec![0.0; buffer_size],
            buffer_size,
            write_pos: 0,
            sample_rate: sr,
            enabled: false,

            target_delay_samples: 0.0,
            delay_samples_slew: 0.0,
            delay_slew_coeff,

            target_feedback: 0.4,
            feedback_slew: 0.4,
            target_mix: 0.0,
            mix_slew: 0.0,
            target_spread: 1.0,
            spread_slew: 1.0,
            param_slew_coeff,

            tempo_sync: true,
            sync_division: 3,
            free_time_ms: 500.0,
            bpm: 120.0,

            hpf_freq: FEEDBACK_HPF_DEFAULT,
            lpf_freq: FEEDBACK_LPF_DEFAULT,
            hpf_ic1eq_l: 0.0,
            hpf_ic2eq_l: 0.0,
            hpf_ic1eq_r: 0.0,
            hpf_ic2eq_r: 0.0,
            hpf_g: 0.0,
            hpf_k: 0.0,
            hpf_a1: 0.0,
            hpf_a2: 0.0,
            hpf_a3: 0.0,

            lpf_ic1eq_l: 0.0,
            lpf_ic2eq_l: 0.0,
            lpf_ic1eq_r: 0.0,
            lpf_ic2eq_r: 0.0,
            lpf_g: 0.0,
            lpf_k: 0.0,
            lpf_a1: 0.0,
            lpf_a2: 0.0,
            lpf_a3: 0.0,

        };
        s.update_hpf_coefficients();
        s.update_lpf_coefficients();
        s
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        let sr = sample_rate as f64;
        self.sample_rate = sr;
        let buffer_size = (sr * MAX_DELAY_SECONDS) as usize + 16;
        self.buffer_l = vec![0.0; buffer_size];
        self.buffer_r = vec![0.0; buffer_size];
        self.buffer_size = buffer_size;
        self.write_pos = 0;
        self.param_slew_coeff = 1.0 - (-1.0 / (sr * 0.005)).exp();
        self.delay_slew_coeff = 1.0 - (-1.0 / (sr * 0.05)).exp();
        self.update_hpf_coefficients();
        self.update_lpf_coefficients();
    }

    pub fn reset(&mut self) {
        self.buffer_l.fill(0.0);
        self.buffer_r.fill(0.0);
        self.write_pos = 0;
        self.hpf_ic1eq_l = 0.0;
        self.hpf_ic2eq_l = 0.0;
        self.hpf_ic1eq_r = 0.0;
        self.hpf_ic2eq_r = 0.0;
        self.lpf_ic1eq_l = 0.0;
        self.lpf_ic2eq_l = 0.0;
        self.lpf_ic1eq_r = 0.0;
        self.lpf_ic2eq_r = 0.0;
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn set_mix(&mut self, mix: f64) {
        self.target_mix = mix;
    }

    pub fn set_feedback(&mut self, feedback: f64) {
        self.target_feedback = feedback.min(0.95);
    }

    pub fn set_tempo_sync(&mut self, sync: bool) {
        self.tempo_sync = sync;
    }

    pub fn set_sync_division(&mut self, division: i32) {
        self.sync_division = division;
    }

    pub fn set_free_time(&mut self, time_ms: f64) {
        self.free_time_ms = time_ms;
    }

    pub fn set_bpm(&mut self, bpm: f64) {
        self.bpm = bpm.max(20.0);
    }

    pub fn set_spread(&mut self, spread: f64) {
        self.target_spread = spread;
    }

    pub fn set_hpf_freq(&mut self, freq: f64) {
        if (freq - self.hpf_freq).abs() > 0.1 {
            self.hpf_freq = freq;
            self.update_hpf_coefficients();
        }
    }

    pub fn set_lpf_freq(&mut self, freq: f64) {
        if (freq - self.lpf_freq).abs() > 0.1 {
            self.lpf_freq = freq;
            self.update_lpf_coefficients();
        }
    }

    fn update_hpf_coefficients(&mut self) {
        let freq = self.hpf_freq.max(20.0).min(self.sample_rate * 0.45);
        self.hpf_g = (PI * freq / self.sample_rate).tan();
        self.hpf_k = 1.0 / FRAC_1_SQRT_2;
        self.hpf_a1 = 1.0 / (1.0 + self.hpf_g * (self.hpf_g + self.hpf_k));
        self.hpf_a2 = self.hpf_g * self.hpf_a1;
        self.hpf_a3 = self.hpf_g * self.hpf_a2;
    }

    fn update_lpf_coefficients(&mut self) {
        let freq = self.lpf_freq.max(20.0).min(self.sample_rate * 0.45);
        self.lpf_g = (PI * freq / self.sample_rate).tan();
        self.lpf_k = 1.0 / FRAC_1_SQRT_2;
        self.lpf_a1 = 1.0 / (1.0 + self.lpf_g * (self.lpf_g + self.lpf_k));
        self.lpf_a2 = self.lpf_g * self.lpf_a1;
        self.lpf_a3 = self.lpf_g * self.lpf_a2;
    }

    fn division_to_beats(division: i32) -> f64 {
        match division {
            0 => 4.0,
            1 => 2.0,
            2 => 3.0,
            3 => 1.0,
            4 => 1.5,
            5 => 0.5,
            6 => 0.75,
            7 => 0.25,
            8 => 0.6667,
            9 => 0.3333,
            _ => 1.0,
        }
    }

    fn compute_delay_samples(&self) -> f64 {
        let delay_ms = if self.tempo_sync {
            let beats = Self::division_to_beats(self.sync_division);
            (60000.0 / self.bpm) * beats
        } else {
            self.free_time_ms
        };
        let max_ms = MAX_DELAY_SECONDS * 1000.0;
        let clamped = delay_ms.max(1.0).min(max_ms);
        clamped * self.sample_rate / 1000.0
    }

    #[inline]
    fn read_interpolated(buffer: &[f64], buffer_size: usize, write_pos: usize, delay_samples: f64) -> f64 {
        let read_pos = write_pos as f64 - delay_samples;
        let read_pos = if read_pos < 0.0 {
            read_pos + buffer_size as f64
        } else {
            read_pos
        };

        let idx0 = read_pos as usize % buffer_size;
        let idx1 = (idx0 + 1) % buffer_size;
        let frac = read_pos - read_pos.floor();

        buffer[idx0] * (1.0 - frac) + buffer[idx1] * frac
    }

    #[inline]
    fn process_hpf_l(&mut self, input: f64) -> f64 {
        let v3 = input - self.hpf_ic2eq_l;
        let v1 = self.hpf_a1 * self.hpf_ic1eq_l + self.hpf_a2 * v3;
        let v2 = self.hpf_ic2eq_l + self.hpf_a2 * self.hpf_ic1eq_l + self.hpf_a3 * v3;
        self.hpf_ic1eq_l = 2.0 * v1 - self.hpf_ic1eq_l;
        self.hpf_ic2eq_l = 2.0 * v2 - self.hpf_ic2eq_l;
        input - self.hpf_k * v1 - v2
    }

    #[inline]
    fn process_hpf_r(&mut self, input: f64) -> f64 {
        let v3 = input - self.hpf_ic2eq_r;
        let v1 = self.hpf_a1 * self.hpf_ic1eq_r + self.hpf_a2 * v3;
        let v2 = self.hpf_ic2eq_r + self.hpf_a2 * self.hpf_ic1eq_r + self.hpf_a3 * v3;
        self.hpf_ic1eq_r = 2.0 * v1 - self.hpf_ic1eq_r;
        self.hpf_ic2eq_r = 2.0 * v2 - self.hpf_ic2eq_r;
        input - self.hpf_k * v1 - v2
    }

    #[inline]
    fn process_lpf_l(&mut self, input: f64) -> f64 {
        let v3 = input - self.lpf_ic2eq_l;
        let v1 = self.lpf_a1 * self.lpf_ic1eq_l + self.lpf_a2 * v3;
        let v2 = self.lpf_ic2eq_l + self.lpf_a2 * self.lpf_ic1eq_l + self.lpf_a3 * v3;
        self.lpf_ic1eq_l = 2.0 * v1 - self.lpf_ic1eq_l;
        self.lpf_ic2eq_l = 2.0 * v2 - self.lpf_ic2eq_l;
        v2
    }

    #[inline]
    fn process_lpf_r(&mut self, input: f64) -> f64 {
        let v3 = input - self.lpf_ic2eq_r;
        let v1 = self.lpf_a1 * self.lpf_ic1eq_r + self.lpf_a2 * v3;
        let v2 = self.lpf_ic2eq_r + self.lpf_a2 * self.lpf_ic1eq_r + self.lpf_a3 * v3;
        self.lpf_ic1eq_r = 2.0 * v1 - self.lpf_ic1eq_r;
        self.lpf_ic2eq_r = 2.0 * v2 - self.lpf_ic2eq_r;
        v2
    }

    pub fn process_block(
        &mut self,
        buffer_l: &mut [f32],
        buffer_r: &mut [f32],
        mod_mix: f64,
        mod_feedback: f64,
    ) {
        if !self.enabled {
            return;
        }

        self.target_delay_samples = self.compute_delay_samples();
        let slew_p = self.param_slew_coeff;
        let slew_d = self.delay_slew_coeff;

        for (l, r) in buffer_l.iter_mut().zip(buffer_r.iter_mut()) {
            self.delay_samples_slew += (self.target_delay_samples - self.delay_samples_slew) * slew_d;
            self.feedback_slew += (self.target_feedback - self.feedback_slew) * slew_p;
            self.mix_slew += (self.target_mix - self.mix_slew) * slew_p;
            self.spread_slew += (self.target_spread - self.spread_slew) * slew_p;

            let mix = (self.mix_slew + mod_mix).clamp(0.0, 1.0);
            let feedback = (self.feedback_slew + mod_feedback).clamp(0.0, 0.95);
            let spread = self.spread_slew.clamp(0.0, 1.0);
            let delay = self.delay_samples_slew.max(1.0);

            let input_l = *l as f64;
            let input_r = *r as f64;

            let delayed_l = Self::read_interpolated(&self.buffer_l, self.buffer_size, self.write_pos, delay);
            let delayed_r = Self::read_interpolated(&self.buffer_r, self.buffer_size, self.write_pos, delay);

            let hp_l = self.process_hpf_l(delayed_l);
            let filtered_l = self.process_lpf_l(hp_l);
            let hp_r = self.process_hpf_r(delayed_r);
            let filtered_r = self.process_lpf_r(hp_r);

            let cross_l = filtered_r * spread + filtered_l * (1.0 - spread);
            let cross_r = filtered_l * spread + filtered_r * (1.0 - spread);

            self.buffer_l[self.write_pos] = input_l + cross_l * feedback;
            self.buffer_r[self.write_pos] = input_r + cross_r * feedback;

            *l = (input_l * (1.0 - mix) + delayed_l * mix) as f32;
            *r = (input_r * (1.0 - mix) + delayed_r * mix) as f32;

            self.write_pos = (self.write_pos + 1) % self.buffer_size;
        }
    }
}
