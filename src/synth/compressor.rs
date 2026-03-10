use std::f64::consts::PI;

const LN10_OVER_20: f64 = std::f64::consts::LN_10 / 20.0;
const LN10_RECIP: f64 = 1.0 / std::f64::consts::LN_10;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScHpfMode {
    Off,
    Hz80,
    Hz150,
    Hz250,
}

impl ScHpfMode {
    pub fn from_index(i: i32) -> Self {
        match i {
            1 => Self::Hz80,
            2 => Self::Hz150,
            3 => Self::Hz250,
            _ => Self::Off,
        }
    }

    fn cutoff_hz(&self) -> Option<f64> {
        match self {
            Self::Off => None,
            Self::Hz80 => Some(80.0),
            Self::Hz150 => Some(150.0),
            Self::Hz250 => Some(250.0),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LookaheadMode {
    Off,
    Ms1,
    Ms2_5,
    Ms5,
}

impl LookaheadMode {
    pub fn from_index(i: i32) -> Self {
        match i {
            1 => Self::Ms1,
            2 => Self::Ms2_5,
            3 => Self::Ms5,
            _ => Self::Off,
        }
    }

    pub fn delay_samples(&self, sample_rate: f64) -> usize {
        match self {
            Self::Off => 0,
            Self::Ms1 => (sample_rate * 0.001) as usize,
            Self::Ms2_5 => (sample_rate * 0.0025) as usize,
            Self::Ms5 => (sample_rate * 0.005) as usize,
        }
    }
}

struct BiquadHpf {
    b0: f64,
    b1: f64,
    b2: f64,
    a1: f64,
    a2: f64,
    x1: f64,
    x2: f64,
    y1: f64,
    y2: f64,
}

impl BiquadHpf {
    fn new() -> Self {
        Self {
            b0: 1.0, b1: 0.0, b2: 0.0,
            a1: 0.0, a2: 0.0,
            x1: 0.0, x2: 0.0,
            y1: 0.0, y2: 0.0,
        }
    }

    fn set_cutoff(&mut self, cutoff_hz: f64, sample_rate: f64) {
        let w0 = 2.0 * PI * cutoff_hz / sample_rate;
        let w0_warped = 2.0 * sample_rate * (w0 / 2.0).tan();
        let k = w0_warped / (2.0 * sample_rate);
        let q = std::f64::consts::FRAC_1_SQRT_2;
        let k2 = k * k;
        let norm = 1.0 + k / q + k2;

        self.b0 = 1.0 / norm;
        self.b1 = -2.0 / norm;
        self.b2 = 1.0 / norm;
        self.a1 = 2.0 * (k2 - 1.0) / norm;
        self.a2 = (1.0 - k / q + k2) / norm;
    }

    fn reset(&mut self) {
        self.x1 = 0.0;
        self.x2 = 0.0;
        self.y1 = 0.0;
        self.y2 = 0.0;
    }

    #[inline]
    fn process(&mut self, input: f64) -> f64 {
        let y = self.b0 * input + self.b1 * self.x1 + self.b2 * self.x2
            - self.a1 * self.y1 - self.a2 * self.y2;
        self.x2 = self.x1;
        self.x1 = input;
        self.y2 = self.y1;
        self.y1 = y;
        y
    }
}

const MAX_LOOKAHEAD_SAMPLES: usize = 256;

struct StereoDelayLine {
    buf_l: [f64; MAX_LOOKAHEAD_SAMPLES],
    buf_r: [f64; MAX_LOOKAHEAD_SAMPLES],
    write_pos: usize,
    delay: usize,
}

impl StereoDelayLine {
    fn new() -> Self {
        Self {
            buf_l: [0.0; MAX_LOOKAHEAD_SAMPLES],
            buf_r: [0.0; MAX_LOOKAHEAD_SAMPLES],
            write_pos: 0,
            delay: 0,
        }
    }

    fn set_delay(&mut self, samples: usize) {
        self.delay = samples.min(MAX_LOOKAHEAD_SAMPLES - 1);
    }

    fn reset(&mut self) {
        self.buf_l = [0.0; MAX_LOOKAHEAD_SAMPLES];
        self.buf_r = [0.0; MAX_LOOKAHEAD_SAMPLES];
        self.write_pos = 0;
    }

    #[inline]
    fn process(&mut self, left: f64, right: f64) -> (f64, f64) {
        self.buf_l[self.write_pos] = left;
        self.buf_r[self.write_pos] = right;
        let read_pos = if self.write_pos >= self.delay {
            self.write_pos - self.delay
        } else {
            MAX_LOOKAHEAD_SAMPLES + self.write_pos - self.delay
        };
        self.write_pos = (self.write_pos + 1) % MAX_LOOKAHEAD_SAMPLES;
        (self.buf_l[read_pos], self.buf_r[read_pos])
    }
}

pub struct Compressor {
    sample_rate: f64,
    sc_hpf_l: BiquadHpf,
    sc_hpf_r: BiquadHpf,
    sc_hpf_mode: ScHpfMode,
    gain_smooth_db: f64,
    threshold_db: f64,
    ratio: f64,
    attack_ms: f64,
    release_ms: f64,
    makeup_db: f64,
    mix: f64,
    knee_db: f64,
    attack_coeff: f64,
    release_coeff: f64,
    slow_release_coeff: f64,
    slow_env: f64,
    slow_env_attack_coeff: f64,
    slow_env_release_coeff: f64,
    rms_sq_l: f64,
    rms_sq_r: f64,
    rms_coeff: f64,
    lookahead_mode: LookaheadMode,
    delay_line: StereoDelayLine,
}

impl Compressor {
    pub fn new(sample_rate: f64) -> Self {
        let mut comp = Self {
            sample_rate,
            sc_hpf_l: BiquadHpf::new(),
            sc_hpf_r: BiquadHpf::new(),
            sc_hpf_mode: ScHpfMode::Off,
            gain_smooth_db: 0.0,
            threshold_db: 0.0,
            ratio: 1.0,
            attack_ms: 10.0,
            release_ms: 100.0,
            makeup_db: 0.0,
            mix: 1.0,
            knee_db: 6.0,
            attack_coeff: 0.0,
            release_coeff: 0.0,
            slow_release_coeff: 0.0,
            slow_env: 0.0,
            slow_env_attack_coeff: 0.0,
            slow_env_release_coeff: 0.0,
            rms_sq_l: 0.0,
            rms_sq_r: 0.0,
            rms_coeff: 0.0,
            lookahead_mode: LookaheadMode::Off,
            delay_line: StereoDelayLine::new(),
        };
        comp.update_coeffs();
        comp
    }

    pub fn set_sample_rate(&mut self, sample_rate: f64) {
        self.sample_rate = sample_rate;
        self.sc_hpf_l.reset();
        self.sc_hpf_r.reset();
        self.gain_smooth_db = 0.0;
        self.slow_env = 0.0;
        self.rms_sq_l = 0.0;
        self.rms_sq_r = 0.0;
        self.delay_line.reset();
        self.update_coeffs();
        self.update_lookahead();
        if let Some(hz) = self.sc_hpf_mode.cutoff_hz() {
            self.sc_hpf_l.set_cutoff(hz, sample_rate);
            self.sc_hpf_r.set_cutoff(hz, sample_rate);
        }
    }

    pub fn set_params(
        &mut self,
        threshold_db: f64,
        ratio: f64,
        attack_ms: f64,
        release_ms: f64,
        makeup_db: f64,
        mix: f64,
        sc_hpf: ScHpfMode,
        lookahead: LookaheadMode,
    ) {
        let needs_coeff_update = (self.attack_ms - attack_ms).abs() > 0.01
            || (self.release_ms - release_ms).abs() > 0.01;
        let needs_hpf_update = self.sc_hpf_mode != sc_hpf;
        let needs_lookahead_update = self.lookahead_mode != lookahead;

        self.threshold_db = threshold_db;
        self.ratio = ratio.max(1.0);
        self.attack_ms = attack_ms;
        self.release_ms = release_ms;
        self.makeup_db = makeup_db;
        self.mix = mix;
        self.sc_hpf_mode = sc_hpf;
        self.lookahead_mode = lookahead;

        if needs_coeff_update {
            self.update_coeffs();
        }
        if needs_hpf_update {
            if let Some(hz) = sc_hpf.cutoff_hz() {
                self.sc_hpf_l.set_cutoff(hz, self.sample_rate);
                self.sc_hpf_r.set_cutoff(hz, self.sample_rate);
            }
        }
        if needs_lookahead_update {
            self.update_lookahead();
        }
    }

    fn update_coeffs(&mut self) {
        let sr = self.sample_rate;
        self.attack_coeff = (-1.0 / (sr * self.attack_ms * 0.001)).exp();
        self.release_coeff = (-1.0 / (sr * self.release_ms * 0.001)).exp();
        self.slow_release_coeff = (-1.0 / (sr * self.release_ms * 0.004)).exp();
        self.rms_coeff = (-1.0 / (sr * 0.010)).exp();

        // Adaptive release envelope: ~50ms attack, ~500ms release
        self.slow_env_attack_coeff = (-1.0 / (sr * 0.050)).exp();
        self.slow_env_release_coeff = (-1.0 / (sr * 0.500)).exp();
    }

    fn update_lookahead(&mut self) {
        let samples = self.lookahead_mode.delay_samples(self.sample_rate);
        self.delay_line.set_delay(samples);
    }

    #[inline]
    fn compute_gain_db(&self, input_db: f64) -> f64 {
        let half_knee = self.knee_db * 0.5;
        let thresh = self.threshold_db;

        if input_db <= thresh - half_knee {
            0.0
        } else if input_db >= thresh + half_knee {
            (thresh + (input_db - thresh) / self.ratio) - input_db
        } else {
            let x = input_db - thresh + half_knee;
            (1.0 / self.ratio - 1.0) * x * x / (2.0 * self.knee_db)
        }
    }

    pub fn process_block(
        &mut self,
        left: &mut [f32],
        right: &mut [f32],
    ) {
        debug_assert_eq!(left.len(), right.len());

        let makeup_lin = db_to_lin(self.makeup_db);
        let mix = self.mix;
        let dry_gain = 1.0 - mix;
        let use_hpf = self.sc_hpf_mode != ScHpfMode::Off;
        let use_lookahead = self.lookahead_mode != LookaheadMode::Off;

        for i in 0..left.len() {
            let in_l = left[i] as f64;
            let in_r = right[i] as f64;

            let sc_l = if use_hpf { self.sc_hpf_l.process(in_l) } else { in_l };
            let sc_r = if use_hpf { self.sc_hpf_r.process(in_r) } else { in_r };

            self.rms_sq_l = self.rms_coeff * self.rms_sq_l + (1.0 - self.rms_coeff) * sc_l * sc_l;
            self.rms_sq_r = self.rms_coeff * self.rms_sq_r + (1.0 - self.rms_coeff) * sc_r * sc_r;

            let peak_l = sc_l.abs();
            let peak_r = sc_r.abs();
            let rms_l = self.rms_sq_l.sqrt();
            let rms_r = self.rms_sq_r.sqrt();
            let level_l = peak_l * 0.7 + rms_l * 0.3;
            let level_r = peak_r * 0.7 + rms_r * 0.3;

            let level = level_l.max(level_r).max(1e-10);
            let level_db = lin_to_db(level);

            let target_gain_db = self.compute_gain_db(level_db);

            let coeff = if target_gain_db < self.gain_smooth_db {
                self.attack_coeff
            } else {
                let slow_blend = (1.0 - self.slow_env).max(0.0);
                self.release_coeff * slow_blend + self.slow_release_coeff * (1.0 - slow_blend)
            };
            self.gain_smooth_db = coeff * self.gain_smooth_db + (1.0 - coeff) * target_gain_db;

            let gr = -target_gain_db.min(0.0);
            let env_coeff = if gr > self.slow_env {
                self.slow_env_attack_coeff
            } else {
                self.slow_env_release_coeff
            };
            self.slow_env = env_coeff * self.slow_env + (1.0 - env_coeff) * gr;

            let final_gain = db_to_lin(self.gain_smooth_db) * makeup_lin;

            let (audio_l, audio_r) = if use_lookahead {
                self.delay_line.process(in_l, in_r)
            } else {
                (in_l, in_r)
            };

            left[i] = (audio_l * final_gain * mix + audio_l * dry_gain) as f32;
            right[i] = (audio_r * final_gain * mix + audio_r * dry_gain) as f32;
        }
    }
}

#[inline]
fn lin_to_db(lin: f64) -> f64 {
    20.0 * LN10_RECIP * lin.ln()
}

#[inline]
fn db_to_lin(db: f64) -> f64 {
    (db * LN10_OVER_20).exp()
}
