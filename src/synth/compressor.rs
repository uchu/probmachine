use std::f64::consts::PI;

const LN10_OVER_20: f64 = std::f64::consts::LN_10 / 20.0;
const LN10_RECIP: f64 = 1.0 / std::f64::consts::LN_10;
const SMOOTHER_TIME_MS: f64 = 5.0;
const GR_NORM_DB: f64 = 20.0;

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
    s1: f64,
    s2: f64,
}

impl BiquadHpf {
    fn new() -> Self {
        Self {
            b0: 1.0, b1: 0.0, b2: 0.0,
            a1: 0.0, a2: 0.0,
            s1: 0.0, s2: 0.0,
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
        self.s1 = 0.0;
        self.s2 = 0.0;
    }

    #[inline]
    fn process(&mut self, x: f64) -> f64 {
        let y = self.b0 * x + self.s1;
        self.s1 = self.b1 * x - self.a1 * y + self.s2;
        self.s2 = self.b2 * x - self.a2 * y;
        y
    }
}

const MAX_LOOKAHEAD_SAMPLES: usize = 256;

const CROSSFADE_LEN: usize = 64;

struct StereoDelayLine {
    buf_l: [f64; MAX_LOOKAHEAD_SAMPLES],
    buf_r: [f64; MAX_LOOKAHEAD_SAMPLES],
    write_pos: usize,
    delay: usize,
    old_delay: usize,
    crossfade_remaining: usize,
}

impl StereoDelayLine {
    fn new() -> Self {
        Self {
            buf_l: [0.0; MAX_LOOKAHEAD_SAMPLES],
            buf_r: [0.0; MAX_LOOKAHEAD_SAMPLES],
            write_pos: 0,
            delay: 0,
            old_delay: 0,
            crossfade_remaining: 0,
        }
    }

    fn set_delay(&mut self, samples: usize) {
        let new_delay = samples.min(MAX_LOOKAHEAD_SAMPLES - 1);
        if new_delay != self.delay {
            self.old_delay = self.delay;
            self.delay = new_delay;
            self.crossfade_remaining = CROSSFADE_LEN;
        }
    }

    fn reset(&mut self) {
        self.buf_l = [0.0; MAX_LOOKAHEAD_SAMPLES];
        self.buf_r = [0.0; MAX_LOOKAHEAD_SAMPLES];
        self.write_pos = 0;
        self.crossfade_remaining = 0;
    }

    #[inline]
    fn read_at(&self, delay: usize) -> (f64, f64) {
        let read_pos = if self.write_pos >= delay {
            self.write_pos - delay
        } else {
            MAX_LOOKAHEAD_SAMPLES + self.write_pos - delay
        };
        (self.buf_l[read_pos], self.buf_r[read_pos])
    }

    #[inline]
    fn process(&mut self, left: f64, right: f64) -> (f64, f64) {
        self.buf_l[self.write_pos] = left;
        self.buf_r[self.write_pos] = right;

        let result = if self.crossfade_remaining > 0 {
            let (new_l, new_r) = self.read_at(self.delay);
            let (old_l, old_r) = self.read_at(self.old_delay);
            self.crossfade_remaining -= 1;
            let t = 1.0 - (self.crossfade_remaining as f64 / CROSSFADE_LEN as f64);
            let t_smooth = t * t * (3.0 - 2.0 * t);
            (
                old_l + (new_l - old_l) * t_smooth,
                old_r + (new_r - old_r) * t_smooth,
            )
        } else {
            self.read_at(self.delay)
        };

        self.write_pos = (self.write_pos + 1) % MAX_LOOKAHEAD_SAMPLES;
        result
    }
}

struct TruePeakDetector {
    prev_l: [f64; 3],
    prev_r: [f64; 3],
}

impl TruePeakDetector {
    fn new() -> Self {
        Self {
            prev_l: [0.0; 3],
            prev_r: [0.0; 3],
        }
    }

    fn reset(&mut self) {
        self.prev_l = [0.0; 3];
        self.prev_r = [0.0; 3];
    }

    #[inline]
    fn process(&mut self, l: f64, r: f64) -> (f64, f64) {
        let peak_l = catmull_rom_peak(self.prev_l[0], self.prev_l[1], self.prev_l[2], l);
        let peak_r = catmull_rom_peak(self.prev_r[0], self.prev_r[1], self.prev_r[2], r);
        self.prev_l = [self.prev_l[1], self.prev_l[2], l];
        self.prev_r = [self.prev_r[1], self.prev_r[2], r];
        (peak_l, peak_r)
    }
}

#[inline]
fn catmull_rom_peak(p0: f64, p1: f64, p2: f64, p3: f64) -> f64 {
    let mut peak = p1.abs().max(p2.abs());
    let c0 = p1;
    let c1 = 0.5 * (-p0 + p2);
    let c2 = 0.5 * (2.0 * p0 - 5.0 * p1 + 4.0 * p2 - p3);
    let c3 = 0.5 * (-p0 + 3.0 * p1 - 3.0 * p2 + p3);
    for &t in &[0.25, 0.5, 0.75] {
        let v = c0 + t * (c1 + t * (c2 + t * c3));
        peak = peak.max(v.abs());
    }
    peak
}

struct ParamSmoother {
    value: f64,
    coeff: f64,
}

impl ParamSmoother {
    fn new(initial: f64, sample_rate: f64) -> Self {
        Self {
            value: initial,
            coeff: Self::compute_coeff(sample_rate),
        }
    }

    fn compute_coeff(sample_rate: f64) -> f64 {
        (-1.0 / (sample_rate * SMOOTHER_TIME_MS * 0.001)).exp()
    }

    fn set_sample_rate(&mut self, sample_rate: f64) {
        self.coeff = Self::compute_coeff(sample_rate);
    }

    #[inline]
    fn process(&mut self, target: f64) -> f64 {
        self.value = self.coeff * self.value + (1.0 - self.coeff) * target;
        self.value
    }

    fn reset(&mut self, value: f64) {
        self.value = value;
    }
}

pub struct Compressor {
    sample_rate: f64,
    sc_hpf_l: BiquadHpf,
    sc_hpf_r: BiquadHpf,
    sc_hpf_mode: ScHpfMode,
    gain_smooth_db_l: f64,
    gain_smooth_db_r: f64,
    threshold_db: f64,
    ratio: f64,
    attack_ms: f64,
    release_ms: f64,
    makeup_db: f64,
    mix: f64,
    knee_db: f64,
    stereo_link: f64,
    auto_makeup: bool,
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
    true_peak: TruePeakDetector,
    threshold_smooth: ParamSmoother,
    ratio_smooth: ParamSmoother,
    makeup_smooth: ParamSmoother,
    mix_smooth: ParamSmoother,
    knee_smooth: ParamSmoother,
    gr_peak_db: f64,
}

impl Compressor {
    pub fn new(sample_rate: f64) -> Self {
        let mut comp = Self {
            sample_rate,
            sc_hpf_l: BiquadHpf::new(),
            sc_hpf_r: BiquadHpf::new(),
            sc_hpf_mode: ScHpfMode::Off,
            gain_smooth_db_l: 0.0,
            gain_smooth_db_r: 0.0,
            threshold_db: -12.0,
            ratio: 4.0,
            attack_ms: 10.0,
            release_ms: 100.0,
            makeup_db: 0.0,
            mix: 1.0,
            knee_db: 6.0,
            stereo_link: 1.0,
            auto_makeup: false,
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
            true_peak: TruePeakDetector::new(),
            threshold_smooth: ParamSmoother::new(-12.0, sample_rate),
            ratio_smooth: ParamSmoother::new(4.0, sample_rate),
            makeup_smooth: ParamSmoother::new(0.0, sample_rate),
            mix_smooth: ParamSmoother::new(1.0, sample_rate),
            knee_smooth: ParamSmoother::new(6.0, sample_rate),
            gr_peak_db: 0.0,
        };
        comp.update_coeffs();
        comp
    }

    pub fn set_sample_rate(&mut self, sample_rate: f64) {
        self.sample_rate = sample_rate;
        self.sc_hpf_l.reset();
        self.sc_hpf_r.reset();
        self.gain_smooth_db_l = 0.0;
        self.gain_smooth_db_r = 0.0;
        self.slow_env = 0.0;
        self.rms_sq_l = 0.0;
        self.rms_sq_r = 0.0;
        self.delay_line.reset();
        self.true_peak.reset();
        self.threshold_smooth.set_sample_rate(sample_rate);
        self.ratio_smooth.set_sample_rate(sample_rate);
        self.makeup_smooth.set_sample_rate(sample_rate);
        self.mix_smooth.set_sample_rate(sample_rate);
        self.knee_smooth.set_sample_rate(sample_rate);
        self.threshold_smooth.reset(self.threshold_db);
        self.ratio_smooth.reset(self.ratio);
        self.makeup_smooth.reset(self.makeup_db);
        self.mix_smooth.reset(self.mix);
        self.knee_smooth.reset(self.knee_db);
        self.gr_peak_db = 0.0;
        self.update_coeffs();
        self.update_lookahead();
        if let Some(hz) = self.sc_hpf_mode.cutoff_hz() {
            self.sc_hpf_l.set_cutoff(hz, sample_rate);
            self.sc_hpf_r.set_cutoff(hz, sample_rate);
        }
    }

    #[allow(clippy::too_many_arguments)]
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
        knee_db: f64,
        stereo_link: f64,
        auto_makeup: bool,
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
        self.knee_db = knee_db;
        self.stereo_link = stereo_link.clamp(0.0, 1.0);
        self.auto_makeup = auto_makeup;

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
        self.slow_env_attack_coeff = (-1.0 / (sr * 0.050)).exp();
        self.slow_env_release_coeff = (-1.0 / (sr * 0.500)).exp();
    }

    fn update_lookahead(&mut self) {
        let samples = self.lookahead_mode.delay_samples(self.sample_rate);
        self.delay_line.set_delay(samples);
    }

    pub fn latency_samples(&self) -> usize {
        self.lookahead_mode.delay_samples(self.sample_rate)
    }

    pub fn gain_reduction_db(&self) -> f64 {
        self.gr_peak_db
    }

    pub fn process_block(
        &mut self,
        left: &mut [f32],
        right: &mut [f32],
    ) {
        debug_assert_eq!(left.len(), right.len());

        let use_hpf = self.sc_hpf_mode != ScHpfMode::Off;
        let use_lookahead = self.lookahead_mode != LookaheadMode::Off;
        let link = self.stereo_link;
        let mut block_gr_peak = 0.0_f64;

        for i in 0..left.len() {
            let in_l = left[i] as f64;
            let in_r = right[i] as f64;

            let sc_l = if use_hpf { self.sc_hpf_l.process(in_l) } else { in_l };
            let sc_r = if use_hpf { self.sc_hpf_r.process(in_r) } else { in_r };

            let (tp_l, tp_r) = self.true_peak.process(sc_l, sc_r);
            let peak_l = sc_l.abs().max(tp_l);
            let peak_r = sc_r.abs().max(tp_r);

            self.rms_sq_l = self.rms_coeff * self.rms_sq_l + (1.0 - self.rms_coeff) * sc_l * sc_l;
            self.rms_sq_r = self.rms_coeff * self.rms_sq_r + (1.0 - self.rms_coeff) * sc_r * sc_r;
            let rms_l = self.rms_sq_l.sqrt();
            let rms_r = self.rms_sq_r.sqrt();

            let level_l = peak_l * 0.7 + rms_l * 0.3;
            let level_r = peak_r * 0.7 + rms_r * 0.3;

            let mono_level = level_l.max(level_r);
            let detect_l = (mono_level * link + level_l * (1.0 - link)).max(1e-10);
            let detect_r = (mono_level * link + level_r * (1.0 - link)).max(1e-10);

            let threshold = self.threshold_smooth.process(self.threshold_db);
            let ratio = self.ratio_smooth.process(self.ratio);
            let makeup = self.makeup_smooth.process(self.makeup_db);
            let mix = self.mix_smooth.process(self.mix);
            let knee = self.knee_smooth.process(self.knee_db);
            let dry_gain = 1.0 - mix;

            let level_db_l = lin_to_db(detect_l);
            let level_db_r = lin_to_db(detect_r);
            let target_gain_db_l = compute_gain_db(level_db_l, threshold, ratio, knee);
            let target_gain_db_r = compute_gain_db(level_db_r, threshold, ratio, knee);

            let slow_blend = (1.0 - self.slow_env / GR_NORM_DB).clamp(0.0, 1.0);

            let coeff_l = if target_gain_db_l < self.gain_smooth_db_l {
                self.attack_coeff
            } else {
                self.release_coeff * slow_blend + self.slow_release_coeff * (1.0 - slow_blend)
            };
            self.gain_smooth_db_l = coeff_l * self.gain_smooth_db_l + (1.0 - coeff_l) * target_gain_db_l;

            let coeff_r = if target_gain_db_r < self.gain_smooth_db_r {
                self.attack_coeff
            } else {
                self.release_coeff * slow_blend + self.slow_release_coeff * (1.0 - slow_blend)
            };
            self.gain_smooth_db_r = coeff_r * self.gain_smooth_db_r + (1.0 - coeff_r) * target_gain_db_r;

            let gr = -(self.gain_smooth_db_l.min(self.gain_smooth_db_r)).min(0.0);
            let env_coeff = if gr > self.slow_env {
                self.slow_env_attack_coeff
            } else {
                self.slow_env_release_coeff
            };
            self.slow_env = env_coeff * self.slow_env + (1.0 - env_coeff) * gr;

            let auto_makeup_db = if self.auto_makeup {
                -threshold * (1.0 - 1.0 / ratio.max(1.001)) * 0.5
            } else {
                0.0
            };
            let effective_makeup_lin = db_to_lin(makeup + auto_makeup_db);

            let final_gain_l = db_to_lin(self.gain_smooth_db_l) * effective_makeup_lin;
            let final_gain_r = db_to_lin(self.gain_smooth_db_r) * effective_makeup_lin;

            let (audio_l, audio_r) = if use_lookahead {
                self.delay_line.process(in_l, in_r)
            } else {
                (in_l, in_r)
            };

            left[i] = (audio_l * final_gain_l * mix + audio_l * dry_gain) as f32;
            right[i] = (audio_r * final_gain_r * mix + audio_r * dry_gain) as f32;

            block_gr_peak = block_gr_peak.max(gr);
        }

        self.gr_peak_db = block_gr_peak;
    }
}

#[inline]
fn compute_gain_db(input_db: f64, threshold_db: f64, ratio: f64, knee_db: f64) -> f64 {
    if knee_db < 0.01 {
        if input_db <= threshold_db {
            0.0
        } else {
            (threshold_db + (input_db - threshold_db) / ratio) - input_db
        }
    } else {
        let half_knee = knee_db * 0.5;
        if input_db <= threshold_db - half_knee {
            0.0
        } else if input_db >= threshold_db + half_knee {
            (threshold_db + (input_db - threshold_db) / ratio) - input_db
        } else {
            let x = input_db - threshold_db + half_knee;
            (1.0 / ratio - 1.0) * x * x / (2.0 * knee_db)
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
