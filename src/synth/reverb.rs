use std::f64::consts::PI;

const FDN_SIZE: usize = 8;
const DIFFUSION_STAGES: usize = 4;
const MAX_PRE_DELAY_SAMPLES: usize = 22050;
const EARLY_TAPS: usize = 12;

const BASE_SAMPLE_RATE: f64 = 44100.0;

const FDN_BASE_LENGTHS: [usize; FDN_SIZE] = [1087, 1283, 1543, 1823, 2011, 2281, 2543, 2791];

const DIFFUSION_LENGTHS_L: [usize; DIFFUSION_STAGES] = [142, 107, 379, 277];
const DIFFUSION_LENGTHS_R: [usize; DIFFUSION_STAGES] = [151, 113, 389, 283];

const LFO_BASE_RATES: [f64; FDN_SIZE] = [0.47, 0.61, 0.73, 0.53, 0.67, 0.79, 0.43, 0.59];

const ER_TAP_DELAYS_MS: [f64; EARLY_TAPS] = [
    7.1, 11.3, 15.7, 21.2, 27.8, 33.1, 39.4, 46.7, 53.2, 61.8, 71.3, 79.6,
];

const ER_TAP_GAINS_L: [f64; EARLY_TAPS] = [
    0.84, -0.68, 0.55, -0.50, 0.42, -0.35, 0.30, -0.24, 0.20, -0.16, 0.13, -0.10,
];
const ER_TAP_GAINS_R: [f64; EARLY_TAPS] = [
    -0.65, 0.76, -0.52, 0.60, -0.38, 0.45, -0.27, 0.32, -0.18, 0.22, -0.12, 0.15,
];

const ER_HF_ROLLOFF: [f64; EARLY_TAPS] = [
    1.0, 0.98, 0.96, 0.93, 0.90, 0.87, 0.83, 0.79, 0.75, 0.71, 0.67, 0.63,
];

const TAP_WEIGHTS_L: [f64; FDN_SIZE] = [
    0.4706, 0.3765, -0.2824, 0.4236, -0.3295, 0.2353, 0.4001, -0.2589,
];
const TAP_WEIGHTS_R: [f64; FDN_SIZE] = [
    -0.2824, 0.4236, 0.4706, -0.2589, 0.4001, 0.3295, -0.2353, 0.3765,
];

const DENORMAL_GUARD: f64 = 1e-18;
const ER_CROSS_FEED: f64 = 0.15;
const ER_TO_LATE_FEED: f64 = 0.35;

struct DelayLine {
    buffer: Vec<f64>,
    write_pos: usize,
    length: usize,
}

impl DelayLine {
    fn new(max_length: usize) -> Self {
        Self {
            buffer: vec![0.0; max_length + 64],
            write_pos: 0,
            length: max_length,
        }
    }

    fn set_length(&mut self, length: usize) {
        self.length = length.min(self.buffer.len() - 1).max(1);
    }

    #[inline]
    fn write(&mut self, sample: f64) {
        self.buffer[self.write_pos] = sample;
        self.write_pos += 1;
        if self.write_pos >= self.buffer.len() {
            self.write_pos = 0;
        }
    }

    #[inline]
    fn read(&self, delay: usize) -> f64 {
        let d = delay.min(self.buffer.len() - 1);
        let pos = if self.write_pos >= d + 1 {
            self.write_pos - d - 1
        } else {
            self.buffer.len() + self.write_pos - d - 1
        };
        self.buffer[pos]
    }

    #[inline]
    fn read_interpolated(&self, delay_fractional: f64) -> f64 {
        let d = delay_fractional.max(1.0);
        let d_int = d as usize;
        let frac = d - d_int as f64;

        let ym1 = self.read(d_int - 1);
        let y0 = self.read(d_int);
        let y1 = self.read(d_int + 1);
        let y2 = self.read(d_int + 2);

        let c0 = y0;
        let c1 = 0.5 * (y1 - ym1);
        let c2 = ym1 - 2.5 * y0 + 2.0 * y1 - 0.5 * y2;
        let c3 = 0.5 * (y2 - ym1) + 1.5 * (y0 - y1);
        ((c3 * frac + c2) * frac + c1) * frac + c0
    }

    fn clear(&mut self) {
        self.buffer.fill(0.0);
        self.write_pos = 0;
    }
}

#[derive(Clone, Copy)]
struct OnePole {
    state: f64,
    coeff: f64,
}

impl OnePole {
    fn new() -> Self {
        Self { state: 0.0, coeff: 0.0 }
    }

    fn set_freq(&mut self, freq: f64, sample_rate: f64) {
        let w = (2.0 * PI * freq / sample_rate).min(PI * 0.99);
        self.coeff = (-w).exp();
    }

    #[inline]
    fn tick_lpf(&mut self, input: f64) -> f64 {
        self.state = input + self.coeff * (self.state - input);
        self.state
    }

    #[inline]
    fn tick_hpf(&mut self, input: f64) -> f64 {
        let lp = input + self.coeff * (self.state - input);
        self.state = lp;
        input - lp
    }

    fn reset(&mut self) {
        self.state = 0.0;
    }
}

struct AllpassDiffuser {
    delay: DelayLine,
    coeff: f64,
}

impl AllpassDiffuser {
    fn new(length: usize) -> Self {
        Self {
            delay: DelayLine::new(length),
            coeff: 0.5,
        }
    }

    fn set_length(&mut self, length: usize) {
        self.delay.set_length(length);
    }

    #[inline]
    fn process(&mut self, input: f64) -> f64 {
        let delayed = self.delay.read(self.delay.length);
        let v = input - self.coeff * delayed;
        self.delay.write(v);
        delayed + self.coeff * v
    }

    fn clear(&mut self) {
        self.delay.clear();
    }
}

struct ModLfo {
    phase: f64,
    rate: f64,
    rw_state: f64,
    rw_target: f64,
    rw_slew: f64,
    rng_state: u64,
}

impl ModLfo {
    fn new(seed: u64) -> Self {
        Self {
            phase: 0.0,
            rate: 0.5,
            rw_state: 0.0,
            rw_target: 0.0,
            rw_slew: 0.001,
            rng_state: seed ^ 0x9e3779b97f4a7c15,
        }
    }

    fn set_rate(&mut self, rate: f64) {
        self.rate = rate;
    }

    #[inline]
    fn next_random(&mut self) -> f64 {
        self.rng_state = self.rng_state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let bits = (self.rng_state >> 33) as f64 / (1u64 << 31) as f64;
        bits - 1.0
    }

    #[inline]
    fn tick(&mut self, israte: f64, shape: f64) -> f64 {
        self.phase += self.rate * israte;
        if self.phase >= 1.0 {
            self.phase -= 1.0;
            self.rw_target = self.next_random();
        }

        let sine = (self.phase * 2.0 * PI).sin();

        self.rw_slew = 0.0005 + 0.003 * self.rate;
        self.rw_state += (self.rw_target - self.rw_state) * self.rw_slew;

        sine * (1.0 - shape) + self.rw_state * shape
    }

    fn reset(&mut self) {
        self.phase = 0.0;
        self.rw_state = 0.0;
        self.rw_target = 0.0;
    }
}

struct DcBlocker {
    x_prev: f64,
    y_prev: f64,
    r: f64,
}

impl DcBlocker {
    fn new(sample_rate: f64) -> Self {
        Self {
            x_prev: 0.0,
            y_prev: 0.0,
            r: Self::calc_coeff(sample_rate),
        }
    }

    fn calc_coeff(sample_rate: f64) -> f64 {
        1.0 - (2.0 * PI * 5.0 / sample_rate)
    }

    fn set_sample_rate(&mut self, sample_rate: f64) {
        self.r = Self::calc_coeff(sample_rate);
    }

    #[inline]
    fn process(&mut self, input: f64) -> f64 {
        let y = input - self.x_prev + self.r * self.y_prev;
        self.x_prev = input;
        self.y_prev = y;
        y
    }

    fn reset(&mut self) {
        self.x_prev = 0.0;
        self.y_prev = 0.0;
    }
}

#[inline]
fn soft_clip(x: f64) -> f64 {
    let ax = x.abs();
    if ax <= 1.0 {
        x
    } else {
        x.signum() * (1.0 + (1.0 - 1.0 / ax) * 0.5)
    }
}

pub struct LushReverb {
    sample_rate: f64,
    inv_sample_rate: f64,
    rate_ratio: f64,

    mix: f64,
    pre_delay_samples: f64,
    time_scale: f64,
    decay: f64,
    diffusion: f64,
    diffusion_mix: f64,
    mod_speed: f64,
    mod_depth: f64,
    mod_shape: f64,
    ducking_amount: f64,
    duck_release_ms: f64,
    stereo_width: f64,
    saturation: f64,

    pre_delay_l: DelayLine,
    pre_delay_r: DelayLine,

    er_delay_l: DelayLine,
    er_delay_r: DelayLine,
    er_tap_delays: [usize; EARLY_TAPS],
    er_lpf_l: OnePole,
    er_lpf_r: OnePole,
    er_hf_filters: [OnePole; EARLY_TAPS],

    input_hpf_l: OnePole,
    input_hpf_r: OnePole,
    input_lpf_l: OnePole,
    input_lpf_r: OnePole,

    diffusers_l: [AllpassDiffuser; DIFFUSION_STAGES],
    diffusers_r: [AllpassDiffuser; DIFFUSION_STAGES],

    fdn_delays: [DelayLine; FDN_SIZE],
    fdn_length_targets: [f64; FDN_SIZE],
    fdn_length_smooth: [f64; FDN_SIZE],
    fdn_decay_gains: [f64; FDN_SIZE],

    tank_lpf: [OnePole; FDN_SIZE],
    tank_hpf: [OnePole; FDN_SIZE],

    mod_lfos: [ModLfo; FDN_SIZE],

    dc_block_l: DcBlocker,
    dc_block_r: DcBlocker,

    duck_envelope: f64,

    rhythm_duck_phase: f64,
    rhythm_duck_depth: f64,
    rhythm_duck_freq: f64,
    rhythm_duck_smooth_ms: f64,
    rhythm_duck_smoothed: f64,

    mix_smooth: f64,
    smooth_coeff: f64,
    length_smooth_coeff: f64,
}

impl LushReverb {
    pub fn new(sample_rate: f32) -> Self {
        let sr = sample_rate as f64;
        let rate_ratio = sr / BASE_SAMPLE_RATE;
        let smooth_coeff = 1.0 - (-1.0 / (0.02 * sr)).exp();
        let length_smooth_coeff = 1.0 - (-1.0 / (0.05 * sr)).exp();

        let max_fdn_len = ((FDN_BASE_LENGTHS[FDN_SIZE - 1] as f64) * rate_ratio * 1.5) as usize + 128;

        let fdn_delays = std::array::from_fn(|_| DelayLine::new(max_fdn_len));
        let initial_lengths: [f64; FDN_SIZE] = std::array::from_fn(|i| {
            (FDN_BASE_LENGTHS[i] as f64) * rate_ratio
        });

        let diffusers_l = std::array::from_fn(|i| {
            AllpassDiffuser::new(((DIFFUSION_LENGTHS_L[i] as f64) * rate_ratio) as usize)
        });
        let diffusers_r = std::array::from_fn(|i| {
            AllpassDiffuser::new(((DIFFUSION_LENGTHS_R[i] as f64) * rate_ratio) as usize)
        });

        let mod_lfos = std::array::from_fn(|i| ModLfo::new(i as u64 * 7919 + 1));

        let er_max = ((ER_TAP_DELAYS_MS[EARLY_TAPS - 1] * 0.001 * sr) as usize) + 64;
        let er_tap_delays = std::array::from_fn(|i| {
            (ER_TAP_DELAYS_MS[i] * 0.001 * sr) as usize
        });

        Self {
            sample_rate: sr,
            inv_sample_rate: 1.0 / sr,
            rate_ratio,
            mix: 0.0,
            pre_delay_samples: 0.0,
            time_scale: 0.85,
            decay: 0.8,
            diffusion: 0.75,
            diffusion_mix: 0.85,
            mod_speed: 0.3,
            mod_depth: 0.4,
            mod_shape: 0.5,
            ducking_amount: 0.0,
            duck_release_ms: 80.0,
            stereo_width: 1.0,
            saturation: 0.0,
            pre_delay_l: DelayLine::new(((MAX_PRE_DELAY_SAMPLES as f64) * rate_ratio) as usize + 64),
            pre_delay_r: DelayLine::new(((MAX_PRE_DELAY_SAMPLES as f64) * rate_ratio) as usize + 64),
            er_delay_l: DelayLine::new(er_max),
            er_delay_r: DelayLine::new(er_max),
            er_tap_delays,
            er_lpf_l: OnePole::new(),
            er_lpf_r: OnePole::new(),
            er_hf_filters: std::array::from_fn(|_| OnePole::new()),
            input_hpf_l: OnePole::new(),
            input_hpf_r: OnePole::new(),
            input_lpf_l: OnePole::new(),
            input_lpf_r: OnePole::new(),
            diffusers_l,
            diffusers_r,
            fdn_delays,
            fdn_length_targets: initial_lengths,
            fdn_length_smooth: initial_lengths,
            fdn_decay_gains: [0.85; FDN_SIZE],
            tank_lpf: std::array::from_fn(|_| OnePole::new()),
            tank_hpf: std::array::from_fn(|_| OnePole::new()),
            mod_lfos,
            dc_block_l: DcBlocker::new(sr),
            dc_block_r: DcBlocker::new(sr),
            duck_envelope: 0.0,
            rhythm_duck_phase: 0.0,
            rhythm_duck_depth: 0.0,
            rhythm_duck_freq: 0.0,
            rhythm_duck_smooth_ms: 75.0,
            rhythm_duck_smoothed: 1.0,
            mix_smooth: 0.0,
            smooth_coeff,
            length_smooth_coeff,
        }
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        let sr = sample_rate as f64;
        self.sample_rate = sr;
        self.inv_sample_rate = 1.0 / sr;
        self.rate_ratio = sr / BASE_SAMPLE_RATE;
        self.smooth_coeff = 1.0 - (-1.0 / (0.02 * sr)).exp();
        self.length_smooth_coeff = 1.0 - (-1.0 / (0.05 * sr)).exp();
        self.dc_block_l.set_sample_rate(sr);
        self.dc_block_r.set_sample_rate(sr);
        self.reset();
    }

    pub fn reset(&mut self) {
        self.pre_delay_l.clear();
        self.pre_delay_r.clear();
        self.er_delay_l.clear();
        self.er_delay_r.clear();
        self.er_lpf_l.reset();
        self.er_lpf_r.reset();
        for f in &mut self.er_hf_filters { f.reset(); }
        self.input_hpf_l.reset();
        self.input_hpf_r.reset();
        self.input_lpf_l.reset();
        self.input_lpf_r.reset();
        for d in &mut self.diffusers_l { d.clear(); }
        for d in &mut self.diffusers_r { d.clear(); }
        for d in &mut self.fdn_delays { d.clear(); }
        for f in &mut self.tank_lpf { f.reset(); }
        for f in &mut self.tank_hpf { f.reset(); }
        for l in &mut self.mod_lfos { l.reset(); }
        self.dc_block_l.reset();
        self.dc_block_r.reset();
        self.fdn_length_smooth = self.fdn_length_targets;
        self.duck_envelope = 0.0;
        self.rhythm_duck_phase = 0.0;
        self.rhythm_duck_smoothed = 1.0;
        self.mix_smooth = 0.0;
    }

    pub fn set_params(
        &mut self,
        mix: f64,
        pre_delay_ms: f64,
        time_scale: f64,
        input_hpf: f64,
        input_lpf: f64,
        tank_hpf: f64,
        tank_lpf: f64,
        mod_speed: f64,
        mod_depth: f64,
        mod_shape: f64,
        diffusion_mix: f64,
        diffusion: f64,
        decay: f64,
        ducking: f64,
        duck_release_ms: f64,
        stereo_width: f64,
        saturation: f64,
    ) {
        self.mix = mix;
        self.pre_delay_samples = pre_delay_ms * 0.001 * self.sample_rate;
        self.time_scale = time_scale.clamp(0.0, 1.0);
        self.decay = decay;
        self.diffusion = diffusion;
        self.diffusion_mix = diffusion_mix;
        self.mod_speed = mod_speed;
        self.mod_depth = mod_depth;
        self.mod_shape = mod_shape;
        self.ducking_amount = ducking;
        self.duck_release_ms = duck_release_ms.max(1.0);
        self.stereo_width = stereo_width;
        self.saturation = saturation;

        self.input_hpf_l.set_freq(input_hpf, self.sample_rate);
        self.input_hpf_r.set_freq(input_hpf, self.sample_rate);
        self.input_lpf_l.set_freq(input_lpf, self.sample_rate);
        self.input_lpf_r.set_freq(input_lpf, self.sample_rate);

        let scale = 0.1 + 0.9 * self.time_scale;
        let rt60_seconds = 0.3 + self.decay * self.decay * 29.7;
        for i in 0..FDN_SIZE {
            let len = (FDN_BASE_LENGTHS[i] as f64) * self.rate_ratio * scale;
            self.fdn_length_targets[i] = len;
            let delay_seconds = len * self.inv_sample_rate;
            self.fdn_decay_gains[i] = 10.0_f64.powf(-3.0 * delay_seconds / rt60_seconds);
        }

        for i in 0..DIFFUSION_STAGES {
            self.diffusers_l[i].set_length(((DIFFUSION_LENGTHS_L[i] as f64) * self.rate_ratio * scale) as usize);
            self.diffusers_l[i].coeff = self.diffusion * 0.75;
            self.diffusers_r[i].set_length(((DIFFUSION_LENGTHS_R[i] as f64) * self.rate_ratio * scale) as usize);
            self.diffusers_r[i].coeff = self.diffusion * 0.75;
        }

        for i in 0..FDN_SIZE {
            self.tank_lpf[i].set_freq(tank_lpf, self.sample_rate);
            self.tank_hpf[i].set_freq(tank_hpf, self.sample_rate);
        }

        let er_scale = 0.3 + 0.7 * self.time_scale;
        for i in 0..EARLY_TAPS {
            self.er_tap_delays[i] = ((ER_TAP_DELAYS_MS[i] * 0.001 * self.sample_rate * er_scale) as usize).max(1);
            let tap_freq = tank_lpf * ER_HF_ROLLOFF[i];
            self.er_hf_filters[i].set_freq(tap_freq, self.sample_rate);
        }
        self.er_lpf_l.set_freq(tank_lpf, self.sample_rate);
        self.er_lpf_r.set_freq(tank_lpf, self.sample_rate);

        let base_rate = 0.1 + mod_speed * mod_speed * 3.9;
        for i in 0..FDN_SIZE {
            self.mod_lfos[i].set_rate(base_rate * LFO_BASE_RATES[i]);
        }
    }

    pub fn set_rhythm_duck_params(&mut self, depth: f64, freq_hz: f64, smooth_ms: f64) {
        self.rhythm_duck_depth = depth;
        self.rhythm_duck_freq = freq_hz;
        self.rhythm_duck_smooth_ms = smooth_ms.max(1.0);
    }

    #[inline]
    fn hadamard_mix(input: &[f64; FDN_SIZE]) -> [f64; FDN_SIZE] {
        let scale = 1.0 / (FDN_SIZE as f64).sqrt();

        let a0 = input[0] + input[1];
        let a1 = input[0] - input[1];
        let a2 = input[2] + input[3];
        let a3 = input[2] - input[3];
        let a4 = input[4] + input[5];
        let a5 = input[4] - input[5];
        let a6 = input[6] + input[7];
        let a7 = input[6] - input[7];

        let b0 = a0 + a2;
        let b1 = a1 + a3;
        let b2 = a0 - a2;
        let b3 = a1 - a3;
        let b4 = a4 + a6;
        let b5 = a5 + a7;
        let b6 = a4 - a6;
        let b7 = a5 - a7;

        [
            (b0 + b4) * scale,
            (b1 + b5) * scale,
            (b2 + b6) * scale,
            (b3 + b7) * scale,
            (b0 - b4) * scale,
            (b1 - b5) * scale,
            (b2 - b6) * scale,
            (b3 - b7) * scale,
        ]
    }

    #[inline]
    fn apply_saturation(sample: f64, amount: f64) -> f64 {
        if amount < 0.001 {
            return sample;
        }
        let clean = sample;
        let driven = sample - sample * sample * sample / 3.0;
        clean + amount * (driven - clean)
    }

    pub fn process_block(
        &mut self,
        main_l: &mut [f32],
        main_r: &mut [f32],
        send_l: &[f32],
        send_r: &[f32],
    ) {
        if self.mix < 0.0001 && self.mix_smooth < 0.0001 {
            return;
        }

        let israte = self.inv_sample_rate;
        let mod_depth_samples = self.mod_depth * self.mod_depth * 12.0 * self.rate_ratio;
        let duck_attack = (-1.0 / (0.001 * self.sample_rate)).exp();
        let duck_release = (-1.0 / (self.duck_release_ms * 0.001 * self.sample_rate)).exp();
        let rhythm_coeff = if self.rhythm_duck_depth > 0.001 {
            (-1.0 / (self.rhythm_duck_smooth_ms * 0.001 * self.sample_rate)).exp()
        } else { 0.0 };
        let rhythm_phase_inc = self.rhythm_duck_freq * self.inv_sample_rate;
        let sc = self.smooth_coeff;
        let lsc = self.length_smooth_coeff;
        let sat = self.saturation;
        let width = self.stereo_width;

        for i in 0..main_l.len().min(send_l.len()) {
            self.mix_smooth += (self.mix - self.mix_smooth) * sc;

            for ch in 0..FDN_SIZE {
                self.fdn_length_smooth[ch] += (self.fdn_length_targets[ch] - self.fdn_length_smooth[ch]) * lsc;
            }

            let dry_l = main_l[i] as f64;
            let dry_r = main_r[i] as f64;
            let in_l = send_l[i] as f64;
            let in_r = send_r[i] as f64;

            let dry_power = dry_l * dry_l + dry_r * dry_r;
            let duck_coeff = if dry_power > self.duck_envelope { duck_attack } else { duck_release };
            self.duck_envelope = dry_power + duck_coeff * (self.duck_envelope - dry_power);
            let duck_gain = 1.0 - self.ducking_amount * (self.duck_envelope * 16.0).min(1.0);

            let rhythm_gain = if self.rhythm_duck_depth > 0.001 {
                self.rhythm_duck_phase += rhythm_phase_inc;
                if self.rhythm_duck_phase >= 1.0 { self.rhythm_duck_phase -= 1.0; }
                let cos_val = (self.rhythm_duck_phase * 2.0 * PI).cos();
                let target = 0.5 + 0.5 * cos_val;
                self.rhythm_duck_smoothed = target + rhythm_coeff * (self.rhythm_duck_smoothed - target);
                1.0 - self.rhythm_duck_depth * (1.0 - self.rhythm_duck_smoothed)
            } else { 1.0 };

            self.pre_delay_l.write(in_l);
            self.pre_delay_r.write(in_r);
            let pre_l = self.pre_delay_l.read_interpolated(self.pre_delay_samples);
            let pre_r = self.pre_delay_r.read_interpolated(self.pre_delay_samples);

            self.er_delay_l.write(pre_l);
            self.er_delay_r.write(pre_r);
            let mut er_l = 0.0;
            let mut er_r = 0.0;
            for t in 0..EARLY_TAPS {
                let tap_l = self.er_delay_l.read(self.er_tap_delays[t]);
                let tap_r = self.er_delay_r.read(self.er_tap_delays[t]);
                let filtered_l = self.er_hf_filters[t].tick_lpf(tap_l + tap_r * ER_CROSS_FEED);
                er_l += filtered_l * ER_TAP_GAINS_L[t];
                er_r += self.er_hf_filters[t].tick_lpf(tap_r + tap_l * ER_CROSS_FEED) * ER_TAP_GAINS_R[t];
            }
            er_l = self.er_lpf_l.tick_lpf(er_l);
            er_r = self.er_lpf_r.tick_lpf(er_r);

            let filt_l = self.input_lpf_l.tick_lpf(self.input_hpf_l.tick_hpf(pre_l));
            let filt_r = self.input_lpf_r.tick_lpf(self.input_hpf_r.tick_hpf(pre_r));

            let mut diff_l = filt_l;
            let mut diff_r = filt_r;
            for stage in &mut self.diffusers_l {
                diff_l = stage.process(diff_l);
            }
            for stage in &mut self.diffusers_r {
                diff_r = stage.process(diff_r);
            }

            let diffused_l = filt_l + self.diffusion_mix * (diff_l - filt_l);
            let diffused_r = filt_r + self.diffusion_mix * (diff_r - filt_r);

            let tank_in_l = diffused_l + er_l * ER_TO_LATE_FEED;
            let tank_in_r = diffused_r + er_r * ER_TO_LATE_FEED;

            let mut fdn_read = [0.0f64; FDN_SIZE];
            for ch in 0..FDN_SIZE {
                let mod_offset = self.mod_lfos[ch].tick(israte, self.mod_shape) * mod_depth_samples;
                let read_delay = self.fdn_length_smooth[ch] + mod_offset;
                fdn_read[ch] = self.fdn_delays[ch].read_interpolated(read_delay.max(1.0));
            }

            let mixed = Self::hadamard_mix(&fdn_read);

            for ch in 0..FDN_SIZE {
                let filtered = self.tank_hpf[ch].tick_hpf(self.tank_lpf[ch].tick_lpf(mixed[ch]));
                let with_gain = filtered * self.fdn_decay_gains[ch];
                let saturated = Self::apply_saturation(with_gain, sat);
                let input_inject = if ch < 4 {
                    tank_in_l * [1.0, -1.0, 1.0, -1.0][ch]
                } else {
                    tank_in_r * [1.0, -1.0, 1.0, -1.0][ch - 4]
                };
                let dn = if ch & 1 == 0 { DENORMAL_GUARD } else { -DENORMAL_GUARD };
                self.fdn_delays[ch].write(saturated + input_inject * 0.5 + dn);
            }

            let mut late_l = 0.0;
            let mut late_r = 0.0;
            for ch in 0..FDN_SIZE {
                late_l += fdn_read[ch] * TAP_WEIGHTS_L[ch];
                late_r += fdn_read[ch] * TAP_WEIGHTS_R[ch];
            }

            let wet_l = er_l * 0.5 + late_l;
            let wet_r = er_r * 0.5 + late_r;

            let mid = (wet_l + wet_r) * 0.5;
            let side = (wet_l - wet_r) * 0.5;
            let width_l = mid + side * width;
            let width_r = mid - side * width;

            let out_l = self.dc_block_l.process(width_l) * duck_gain * rhythm_gain;
            let out_r = self.dc_block_r.process(width_r) * duck_gain * rhythm_gain;

            let out_l = soft_clip(out_l);
            let out_r = soft_clip(out_r);

            let mix = self.mix_smooth;
            main_l[i] = (dry_l + mix * out_l) as f32;
            main_r[i] = (dry_r + mix * out_r) as f32;
        }
    }
}
