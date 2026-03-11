#![allow(dead_code)]

use std::f64::consts::PI;
use super::dsp::Oversampling;

const DENORMAL_GUARD: f64 = 1e-18;

pub struct FilterParams {
    pub cutoff: f64,
    pub resonance: f64,
    pub drive: f64,
    pub boost: i32,
    pub stereo_sep: f64,
    pub mode: u8,
    pub key_track_hz: f64,
    pub key_track_amount: f64,
    pub sat_type: u8,
    pub morph: f64,
    pub filter_fm: f64,
    pub feedback: f64,
    pub bass_lock: f64,
    pub pole_spread: f64,
    pub res_character: f64,
    pub res_tilt: f64,
    pub cutoff_slew: f64,
    pub poles: usize,
    pub env_mod: f64,
}

#[inline]
fn fast_tanh(x: f64) -> f64 {
    let x = x.clamp(-3.0, 3.0);
    let x2 = x * x;
    x * (27.0 + x2) / (27.0 + 9.0 * x2)
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum FilterMode {
    Lp24,
    Lp12,
    Bp12,
    Hp24,
}

impl FilterMode {
    pub fn from_index(idx: u8) -> Self {
        match idx {
            1 => FilterMode::Lp12,
            2 => FilterMode::Bp12,
            3 => FilterMode::Hp24,
            _ => FilterMode::Lp24,
        }
    }
}

#[inline]
fn saturate(x: f64, sat_type: u8, character: f64) -> f64 {
    let hardness = 1.0 + character * 3.0;
    let xd = x * hardness;
    let base = match sat_type {
        0 => fast_tanh(xd),
        1 => {
            let ax = xd.abs();
            xd / (1.0 + ax) + 0.1 * xd * xd * xd.signum() / (1.0 + ax * ax)
        }
        2 => {
            let t = fast_tanh(xd);
            t + 0.15 * t * (1.0 - t * t)
        }
        _ => fast_tanh(xd),
    };
    base / hardness
}

const MORPH_COEFS_4: [[f64; 5]; 5] = [
    [0.0, 0.0, 0.0, 0.0, 1.0],      // LP24: stage[3]
    [0.0, 0.0, 1.0, 0.0, 0.0],      // LP12: stage[1]
    [0.0, 0.0, 1.0, 0.0, -1.0],     // BP12: stage[1] - stage[3]
    [1.0, -2.0, 2.0, 0.0, 0.0],     // Notch
    [1.0, -4.0, 6.0, -4.0, 1.0],    // HP24
];

const MORPH_COEFS_8: [[f64; 9]; 5] = [
    [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0],                  // LP48
    [0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0],                  // LP24
    [0.0, 0.0, 0.0, 0.0, 1.0, -4.0, 6.0, -4.0, 1.0],                // BP24
    [1.0, -4.0, 6.0, -4.0, 2.0, 0.0, 0.0, 0.0, 0.0],                // Notch24
    [1.0, -8.0, 28.0, -56.0, 70.0, -56.0, 28.0, -8.0, 1.0],         // HP48
];

const SPREAD_OFFSETS_4: [f64; 4] = [-0.15, -0.05, 0.05, 0.15];
const SPREAD_OFFSETS_8: [f64; 8] = [-0.30, -0.20, -0.10, -0.03, 0.03, 0.10, 0.20, 0.30];

struct DCBlocker {
    x_prev: f64,
    y_prev: f64,
    r: f64,
}

impl DCBlocker {
    fn new(sample_rate: f64) -> Self {
        Self {
            x_prev: 0.0,
            y_prev: 0.0,
            r: Self::calc_r(sample_rate),
        }
    }

    fn calc_r(sample_rate: f64) -> f64 {
        1.0 - (std::f64::consts::TAU * 10.0 / sample_rate)
    }

    fn set_sample_rate(&mut self, sample_rate: f64) {
        self.r = Self::calc_r(sample_rate);
    }

    fn reset(&mut self) {
        self.x_prev = 0.0;
        self.y_prev = 0.0;
    }

    #[inline]
    fn process(&mut self, x: f64) -> f64 {
        let y = x - self.x_prev + self.r * self.y_prev;
        self.x_prev = x;
        self.y_prev = y;
        y
    }
}

// 4x polyphase FIR upsampler
// 64-tap prototype: windowed-sinc, Kaiser beta=7.857 (~80dB stopband)
// fc = 0.125 normalized to oversampled rate (original Nyquist)
// Passband: flat to ~17kHz (-0.1dB), -1dB at 19kHz, -6dB at 22.05kHz
// Image rejection: -89dB at first image band, -110dB at higher images
// Latency: 8 samples (half the polyphase tap count)
const UPSAMPLER_TAPS: usize = 16;

#[rustfmt::skip]
const POLYPHASE_COEFFS: [[f64; UPSAMPLER_TAPS]; 4] = [
    [
        -4.13504431798155137e-05,  4.63205214875369242e-04,
        -1.85877378948577556e-03,  5.24819856256699118e-03,
        -1.22192698719886753e-02,  2.55655402218176488e-02,
        -5.26176492399370588e-02,  1.33030136078201799e-01,
         9.73590634571654068e-01, -1.00428991795570341e-01,
         4.36787275572379924e-02, -2.13796598605423827e-02,
         1.00215111201251983e-02, -4.13950210933667490e-03,
         1.37076116360402146e-03, -2.96536469618463659e-04,
    ],
    [
        -2.29224181883271315e-04,  1.66273579156384251e-03,
        -5.96030432467265815e-03,  1.58756969782466312e-02,
        -3.57295530251428919e-02,  7.36992144213142880e-02,
        -1.54552023545920125e-01,  4.59760363795160221e-01,
         7.77708744973157562e-01, -1.91087375764497081e-01,
         8.80376897502389844e-02, -4.30363825631882788e-02,
         1.96873345312422003e-02, -7.77736698241237191e-03,
         2.38111626168156657e-03, -4.27647025312490893e-04,
    ],
    [
        -4.27647025312490893e-04,  2.38111626168156657e-03,
        -7.77736698241237191e-03,  1.96873345312422003e-02,
        -4.30363825631882788e-02,  8.80376897502389844e-02,
        -1.91087375764497081e-01,  7.77708744973157562e-01,
         4.59760363795160221e-01, -1.54552023545920125e-01,
         7.36992144213142880e-02, -3.57295530251428919e-02,
         1.58756969782466312e-02, -5.96030432467265815e-03,
         1.66273579156384251e-03, -2.29224181883271315e-04,
    ],
    [
        -2.96536469618463659e-04,  1.37076116360402146e-03,
        -4.13950210933667490e-03,  1.00215111201251983e-02,
        -2.13796598605423827e-02,  4.36787275572379924e-02,
        -1.00428991795570341e-01,  9.73590634571654068e-01,
         1.33030136078201799e-01, -5.26176492399370588e-02,
         2.55655402218176488e-02, -1.22192698719886753e-02,
         5.24819856256699118e-03, -1.85877378948577556e-03,
         4.63205214875369242e-04, -4.13504431798155137e-05,
    ],
];

struct PolyphaseUpsampler {
    delay: [f64; UPSAMPLER_TAPS],
}

impl PolyphaseUpsampler {
    fn new() -> Self {
        Self { delay: [0.0; UPSAMPLER_TAPS] }
    }

    fn reset(&mut self) {
        self.delay = [0.0; UPSAMPLER_TAPS];
    }

    #[inline]
    fn process(&mut self, input: f64) -> [f64; 4] {
        // Shift delay line and insert new sample at the front
        let d = &mut self.delay;
        d.copy_within(0..UPSAMPLER_TAPS - 1, 1);
        d[0] = input;

        let mut out = [0.0f64; 4];
        for phase in 0..4 {
            let c = &POLYPHASE_COEFFS[phase];
            let mut sum = 0.0;
            // Unrolled inner product: 16 taps
            sum += c[ 0] * d[ 0]; sum += c[ 1] * d[ 1];
            sum += c[ 2] * d[ 2]; sum += c[ 3] * d[ 3];
            sum += c[ 4] * d[ 4]; sum += c[ 5] * d[ 5];
            sum += c[ 6] * d[ 6]; sum += c[ 7] * d[ 7];
            sum += c[ 8] * d[ 8]; sum += c[ 9] * d[ 9];
            sum += c[10] * d[10]; sum += c[11] * d[11];
            sum += c[12] * d[12]; sum += c[13] * d[13];
            sum += c[14] * d[14]; sum += c[15] * d[15];
            out[phase] = sum;
        }
        out
    }
}

struct LadderState {
    stage: [f64; 8],
}

impl LadderState {
    fn new() -> Self {
        Self { stage: [0.0; 8] }
    }

    fn reset(&mut self) {
        self.stage = [0.0; 8];
    }

    #[inline]
    fn process(
        &mut self,
        input: f64,
        g: &[f64; 8],
        resonance: f64,
        drive: f64,
        boost: f64,
        mode: FilterMode,
        sat_type: u8,
        morph: f64,
        res_character: f64,
        active_stages: usize,
    ) -> f64 {
        let k_max = if active_stages == 8 { 1.884 } else { 4.0 };
        let feedback_compensation = resonance * k_max * (1.0 - 0.15 * resonance * resonance);
        let drive_gain = (1.0 + drive * 3.0) * boost;
        let fb_idx = active_stages - 1;

        let input_driven = saturate(input * drive_gain + DENORMAL_GUARD, sat_type, res_character);
        let feedback_sample = saturate(
            self.stage[fb_idx] + DENORMAL_GUARD,
            sat_type,
            res_character,
        );

        let mut g_total = 1.0_f64;
        for gi in g.iter().take(active_stages) {
            g_total *= gi / (1.0 + gi);
        }
        let u = (input_driven - feedback_compensation * feedback_sample)
            / (1.0 + feedback_compensation * g_total);

        let mut x = u;
        for (i, gi) in g.iter().enumerate().take(active_stages) {
            let s = self.stage[i] + DENORMAL_GUARD;
            let sat_x = saturate(x, sat_type, res_character);
            let sat_s = saturate(s, sat_type, res_character);
            let v = (sat_x - sat_s) / (1.0 / gi + 1.0);
            self.stage[i] = s + 2.0 * v;
            x = s + v;
        }

        if active_stages == 8 {
            if morph > 0.001 {
                let pos = morph * 4.0;
                let idx = (pos as usize).min(3);
                let frac = pos - idx as f64;
                let c0 = &MORPH_COEFS_8[idx];
                let c1 = &MORPH_COEFS_8[idx + 1];
                (c0[0] + frac * (c1[0] - c0[0])) * u
                    + (c0[1] + frac * (c1[1] - c0[1])) * self.stage[0]
                    + (c0[2] + frac * (c1[2] - c0[2])) * self.stage[1]
                    + (c0[3] + frac * (c1[3] - c0[3])) * self.stage[2]
                    + (c0[4] + frac * (c1[4] - c0[4])) * self.stage[3]
                    + (c0[5] + frac * (c1[5] - c0[5])) * self.stage[4]
                    + (c0[6] + frac * (c1[6] - c0[6])) * self.stage[5]
                    + (c0[7] + frac * (c1[7] - c0[7])) * self.stage[6]
                    + (c0[8] + frac * (c1[8] - c0[8])) * self.stage[7]
            } else {
                self.stage[7]
            }
        } else if morph > 0.001 {
            let pos = morph * 4.0;
            let idx = (pos as usize).min(3);
            let frac = pos - idx as f64;
            let c0 = &MORPH_COEFS_4[idx];
            let c1 = &MORPH_COEFS_4[idx + 1];
            let c = [
                c0[0] + frac * (c1[0] - c0[0]),
                c0[1] + frac * (c1[1] - c0[1]),
                c0[2] + frac * (c1[2] - c0[2]),
                c0[3] + frac * (c1[3] - c0[3]),
                c0[4] + frac * (c1[4] - c0[4]),
            ];
            c[0] * u
                + c[1] * self.stage[0]
                + c[2] * self.stage[1]
                + c[3] * self.stage[2]
                + c[4] * self.stage[3]
        } else {
            match mode {
                FilterMode::Lp24 => self.stage[3],
                FilterMode::Lp12 => self.stage[1],
                FilterMode::Bp12 => self.stage[1] - self.stage[3],
                FilterMode::Hp24 => {
                    u - 4.0 * self.stage[0] + 6.0 * self.stage[1]
                        - 4.0 * self.stage[2] + self.stage[3]
                }
            }
        }
    }
}

pub struct LadderFilter {
    left: LadderState,
    right: LadderState,
    sample_rate: f64,
    cutoff_hz: f64,
    resonance: f64,
    drive: f64,
    stereo_sep: f64,
    mode: FilterMode,
    key_track_hz: f64,
    key_track_amount: f64,
    env_mod: f64,
    boost: f64,
    sat_type: u8,
    morph: f64,
    filter_fm: f64,
    feedback_amount: f64,
    bass_lock: f64,
    pole_spread: f64,
    res_character: f64,
    res_tilt: f64,
    cutoff_slew_amount: f64,
    cutoff_slew_coef: f64,
    prev_cutoff_l: f64,
    prev_cutoff_r: f64,
    feedback_state_l: f64,
    feedback_state_r: f64,
    bass_state_l: f64,
    bass_state_r: f64,
    active_stages: usize,
    up_left: PolyphaseUpsampler,
    up_right: PolyphaseUpsampler,
    os_left: Oversampling<4>,
    os_right: Oversampling<4>,
    dc_left: DCBlocker,
    dc_right: DCBlocker,
}

impl LadderFilter {
    pub fn new(sample_rate: f32) -> Self {
        let sr = sample_rate as f64;
        let mut os_left = Oversampling::<4>::new();
        let mut os_right = Oversampling::<4>::new();
        os_left.set_sample_rate(sr);
        os_right.set_sample_rate(sr);

        Self {
            left: LadderState::new(),
            right: LadderState::new(),
            sample_rate: sr,
            cutoff_hz: 20000.0,
            resonance: 0.0,
            drive: 0.0,
            stereo_sep: 0.0,
            mode: FilterMode::Lp24,
            key_track_hz: 440.0,
            key_track_amount: 0.0,
            env_mod: 0.0,
            boost: 1.0,
            sat_type: 0,
            morph: 0.0,
            filter_fm: 0.0,
            feedback_amount: 0.0,
            bass_lock: 0.0,
            pole_spread: 0.0,
            res_character: 0.0,
            res_tilt: 0.0,
            cutoff_slew_amount: 0.0,
            cutoff_slew_coef: 1.0,
            prev_cutoff_l: 20000.0,
            prev_cutoff_r: 20000.0,
            feedback_state_l: 0.0,
            feedback_state_r: 0.0,
            bass_state_l: 0.0,
            bass_state_r: 0.0,
            active_stages: 4,
            up_left: PolyphaseUpsampler::new(),
            up_right: PolyphaseUpsampler::new(),
            os_left,
            os_right,
            dc_left: DCBlocker::new(sr),
            dc_right: DCBlocker::new(sr),
        }
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate as f64;
        self.up_left.reset();
        self.up_right.reset();
        self.os_left.set_sample_rate(self.sample_rate);
        self.os_right.set_sample_rate(self.sample_rate);
        self.dc_left.set_sample_rate(self.sample_rate);
        self.dc_right.set_sample_rate(self.sample_rate);
        self.cutoff_slew_coef = Self::calc_slew_coef(self.cutoff_slew_amount, self.sample_rate);
    }

    fn calc_slew_coef(amount: f64, sample_rate: f64) -> f64 {
        if amount > 0.001 {
            let slew_ms = 0.1 + amount * 200.0;
            1.0 - (-1.0 / (slew_ms * 0.001 * sample_rate)).exp()
        } else {
            1.0
        }
    }

    pub fn reset(&mut self) {
        self.left.reset();
        self.right.reset();
        self.feedback_state_l = 0.0;
        self.feedback_state_r = 0.0;
        self.bass_state_l = 0.0;
        self.bass_state_r = 0.0;
        self.up_left.reset();
        self.up_right.reset();
        self.os_left.reset();
        self.os_right.reset();
        self.dc_left.reset();
        self.dc_right.reset();
    }

    pub fn set_params(&mut self, p: &FilterParams) {
        self.cutoff_hz = p.cutoff;
        self.resonance = p.resonance.clamp(0.0, 1.05);
        self.drive = p.drive.clamp(0.0, 1.0);
        self.boost = match p.boost {
            1 => 3.981_071_705_534_969_5,
            2 => 15.848_931_924_611_134,
            3 => 251.188_643_150_958,
            _ => 1.0,
        };
        self.stereo_sep = p.stereo_sep.clamp(0.0, 0.50);
        self.mode = FilterMode::from_index(p.mode);
        self.key_track_hz = p.key_track_hz;
        self.key_track_amount = p.key_track_amount.clamp(0.0, 1.0);
        self.sat_type = p.sat_type.min(2);
        self.morph = p.morph.clamp(0.0, 1.0);
        self.filter_fm = p.filter_fm.clamp(0.0, 1.0);
        self.feedback_amount = p.feedback.clamp(-1.0, 1.0);
        self.bass_lock = p.bass_lock.clamp(0.0, 1.0);
        self.pole_spread = p.pole_spread.clamp(0.0, 1.0);
        self.res_character = p.res_character.clamp(0.0, 1.0);
        self.res_tilt = p.res_tilt.clamp(-1.0, 1.0);
        self.active_stages = if p.poles == 1 { 8 } else { 4 };
        self.env_mod = p.env_mod.clamp(-1.0, 1.0);
        let new_slew = p.cutoff_slew.clamp(0.0, 1.0);
        if (new_slew - self.cutoff_slew_amount).abs() > 1e-6 {
            self.cutoff_slew_amount = new_slew;
            self.cutoff_slew_coef = Self::calc_slew_coef(new_slew, self.sample_rate);
        }
    }

    #[inline]
    fn compute_cutoff(&self, base_hz: f64) -> f64 {
        const MIN_ST: f64 = -54.0;
        const MAX_ST: f64 = 65.513;

        let key_offset = if self.key_track_amount > 0.0 {
            let ratio = self.key_track_hz / 440.0;
            let semitones = 12.0 * ratio.log2();
            semitones * self.key_track_amount
        } else {
            0.0
        };

        let base_st = (base_hz / 440.0).log2() * 12.0 + key_offset;

        let modulated_st = if self.env_mod > 0.0 {
            base_st + self.env_mod * (MAX_ST - base_st)
        } else if self.env_mod < 0.0 {
            base_st + self.env_mod * (base_st - MIN_ST)
        } else {
            base_st
        };

        440.0 * (modulated_st / 12.0).exp2()
    }

    #[inline]
    pub fn process(&mut self, left: f64, right: f64) -> (f64, f64) {
        let os_rate = self.sample_rate * 4.0;
        let nyquist = os_rate * 0.45;

        let base_cutoff = self.compute_cutoff(self.cutoff_hz).clamp(20.0, nyquist);

        let fm_mod = if self.filter_fm > 0.001 {
            (left + right) * 0.5 * self.filter_fm
        } else {
            0.0
        };
        let fm_cutoff = base_cutoff * (fm_mod * 2.0).exp2();

        let sep_octaves = self.stereo_sep;
        let cutoff_l = (fm_cutoff * (-sep_octaves).exp2()).clamp(20.0, nyquist);
        let cutoff_r = (fm_cutoff * sep_octaves.exp2()).clamp(20.0, nyquist);

        self.prev_cutoff_l += self.cutoff_slew_coef * (cutoff_l - self.prev_cutoff_l);
        self.prev_cutoff_r += self.cutoff_slew_coef * (cutoff_r - self.prev_cutoff_r);
        let final_cutoff_l = self.prev_cutoff_l.clamp(20.0, nyquist);
        let final_cutoff_r = self.prev_cutoff_r.clamp(20.0, nyquist);

        let g_l_base = (PI * final_cutoff_l / os_rate).tan();
        let g_r_base = (PI * final_cutoff_r / os_rate).tan();

        let n = self.active_stages;
        let mut g_l = [0.0f64; 8];
        let mut g_r = [0.0f64; 8];

        if self.pole_spread > 0.001 {
            let offsets = if n == 8 { &SPREAD_OFFSETS_8[..] } else { &SPREAD_OFFSETS_4[..] };
            for i in 0..n {
                g_l[i] = g_l_base * (offsets[i] * self.pole_spread).exp2();
                g_r[i] = g_r_base * (offsets[i] * self.pole_spread).exp2();
            }
        } else {
            for i in 0..n {
                g_l[i] = g_l_base;
                g_r[i] = g_r_base;
            }
        }

        let res_l = (self.resonance + self.res_tilt * 0.5).clamp(0.0, 1.05);
        let res_r = (self.resonance - self.res_tilt * 0.5).clamp(0.0, 1.05);

        let fb_l = self.feedback_state_l * self.feedback_amount;
        let fb_r = self.feedback_state_r * self.feedback_amount;
        let input_l = left + fast_tanh(fb_l);
        let input_r = right + fast_tanh(fb_r);

        let mut buf_l = self.up_left.process(input_l);
        let mut buf_r = self.up_right.process(input_r);

        for s in &mut buf_l {
            *s = self.left.process(
                *s, &g_l, res_l, self.drive, self.boost, self.mode,
                self.sat_type, self.morph, self.res_character, n,
            );
        }
        for s in &mut buf_r {
            *s = self.right.process(
                *s, &g_r, res_r, self.drive, self.boost, self.mode,
                self.sat_type, self.morph, self.res_character, n,
            );
        }

        self.os_left.resample_buffer().copy_from_slice(&buf_l);
        let mut out_l = self.os_left.downsample();
        self.os_right.resample_buffer().copy_from_slice(&buf_r);
        let mut out_r = self.os_right.downsample();

        if !out_l.is_finite() {
            out_l = 0.0;
            self.left.reset();
        }
        if !out_r.is_finite() {
            out_r = 0.0;
            self.right.reset();
        }

        self.feedback_state_l = fast_tanh(out_l);
        self.feedback_state_r = fast_tanh(out_r);

        let res_comp = 1.0 + self.resonance * 1.5;
        out_l *= res_comp;
        out_r *= res_comp;

        let (out_l, out_r) = if self.bass_lock > 0.001 {
            let bass_coef = (std::f64::consts::TAU * 120.0 / self.sample_rate).min(0.5);
            self.bass_state_l += bass_coef * (left - self.bass_state_l);
            self.bass_state_r += bass_coef * (right - self.bass_state_r);
            let res_factor = (self.resonance * 1.5).min(1.0);
            let res_curve = res_factor * res_factor;
            let comp_l = self.bass_state_l * self.bass_lock * res_curve * 2.0;
            let comp_r = self.bass_state_r * self.bass_lock * res_curve * 2.0;
            (out_l + comp_l, out_r + comp_r)
        } else {
            (out_l, out_r)
        };

        let out_l = self.dc_left.process(out_l);
        let out_r = self.dc_right.process(out_r);

        (out_l, out_r)
    }
}
