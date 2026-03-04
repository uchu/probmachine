#![allow(dead_code)]

use std::cell::RefCell;
use std::f64::consts::{PI, TAU};

struct SplitMix64(u64);

impl SplitMix64 {
    fn new() -> Self {
        let seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0x12345678deadbeef);
        Self(seed)
    }

    #[inline]
    fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e3779b97f4a7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
        z ^ (z >> 31)
    }
}

thread_local! {
    static RNG: RefCell<SplitMix64> = RefCell::new(SplitMix64::new());
}

#[inline]
pub fn rand_01() -> f64 {
    RNG.with(|r| {
        let u = r.borrow_mut().next_u64();
        (u >> 11) as f64 * (1.0 / (1u64 << 53) as f64)
    })
}

#[derive(Debug, Clone, Copy)]
pub struct SlewValue {
    current: f64,
    slew_per_ms: f64,
}

impl SlewValue {
    pub fn new() -> Self {
        Self {
            current: 0.0,
            slew_per_ms: 1000.0 / 44100.0,
        }
    }

    pub fn reset(&mut self) {
        self.current = 0.0;
    }

    pub fn set_sample_rate(&mut self, srate: f64) {
        self.slew_per_ms = 1000.0 / srate;
    }

    pub fn value(&self) -> f64 {
        self.current
    }

    #[inline]
    pub fn next(&mut self, target: f64, slew_ms_per_1: f64) -> f64 {
        if slew_ms_per_1 < 0.11 {
            self.current = target;
        } else {
            let max_delta = self.slew_per_ms / slew_ms_per_1;
            self.current = target
                .min(self.current + max_delta)
                .max(self.current - max_delta);
        }
        self.current
    }
}

impl Default for SlewValue {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Copy, Clone, Default)]
struct BiquadCoefs {
    a1: f64,
    a2: f64,
    b0: f64,
    b1: f64,
    b2: f64,
}

impl BiquadCoefs {
    fn lowpass(sample_rate: f64, q: f64, cutoff: f64) -> Self {
        let f = (cutoff * PI / sample_rate).tan();
        let a0r = 1.0 / (1.0 + f / q + f * f);
        let b0 = f * f * a0r;
        let b1 = 2.0 * b0;
        let b2 = b0;
        let a1 = 2.0 * (f * f - 1.0) * a0r;
        let a2 = (1.0 - f / q + f * f) * a0r;
        Self { a1, a2, b0, b1, b2 }
    }

    fn calc_cascaded_butter_q(order: usize, casc_idx: usize) -> f64 {
        let order = order as f64;
        let casc_idx = casc_idx as f64;
        let b = -2.0 * ((2.0 * casc_idx + order - 1.0) * PI / (2.0 * order)).cos();
        1.0 / b
    }
}

#[derive(Debug, Copy, Clone, Default)]
struct Biquad {
    coefs: BiquadCoefs,
    x1: f64,
    x2: f64,
    y1: f64,
    y2: f64,
}

impl Biquad {
    fn new() -> Self {
        Self::default()
    }

    fn set_coefs(&mut self, coefs: BiquadCoefs) {
        self.coefs = coefs;
    }

    fn reset(&mut self) {
        self.x1 = 0.0;
        self.x2 = 0.0;
        self.y1 = 0.0;
        self.y2 = 0.0;
    }

    #[inline]
    fn tick(&mut self, input: f64) -> f64 {
        let y0 = self.coefs.b0 * input
            + self.coefs.b1 * self.x1
            + self.coefs.b2 * self.x2
            - self.coefs.a1 * self.y1
            - self.coefs.a2 * self.y2;
        self.x2 = self.x1;
        self.x1 = input;
        self.y2 = self.y1;
        self.y1 = y0;
        y0
    }
}

#[derive(Debug, Clone)]
pub struct Oversampling<const N: usize> {
    filters: [Biquad; 4],
    buffer: [f64; N],
}

impl<const N: usize> Oversampling<N> {
    pub fn new() -> Self {
        let mut this = Self {
            filters: [Biquad::new(); 4],
            buffer: [0.0; N],
        };
        this.set_sample_rate(44100.0);
        this
    }

    pub fn reset(&mut self) {
        self.buffer = [0.0; N];
        for filt in &mut self.filters {
            filt.reset();
        }
    }

    pub fn set_sample_rate(&mut self, srate: f64) {
        let cutoff = 0.98 * (0.5 * srate);
        let ovr_srate = (N as f64) * srate;
        let filters_len = self.filters.len();
        for (i, filt) in self.filters.iter_mut().enumerate() {
            let q = BiquadCoefs::calc_cascaded_butter_q(2 * 4, filters_len - i);
            filt.set_coefs(BiquadCoefs::lowpass(ovr_srate, q, cutoff));
        }
    }

    #[inline]
    pub fn resample_buffer(&mut self) -> &mut [f64; N] {
        &mut self.buffer
    }

    #[inline]
    pub fn downsample(&mut self) -> f64 {
        let mut ret = 0.0;
        for s in &mut self.buffer {
            ret = *s;
            for filt in &mut self.filters {
                ret = filt.tick(ret);
            }
        }
        ret
    }
}

#[derive(Debug, Clone)]
pub struct VPSOscillator {
    phase: f64,
}

impl VPSOscillator {
    pub fn new(init_phase: f64) -> Self {
        Self { phase: init_phase }
    }

    pub fn set_phase(&mut self, phase: f64) {
        self.phase = phase;
    }

    #[inline]
    fn phi_vps(x: f64, v: f64, d: f64) -> f64 {
        let d = d.clamp(1e-9, 1.0 - 1e-9);
        if x < d {
            (v * x) / d
        } else {
            v + ((1.0 - v) * (x - d)) / (1.0 - d)
        }
    }

    #[inline]
    pub fn limit_v(d: f64, v: f64) -> f64 {
        let delta = 0.5 - (d - 0.5).abs();
        if delta < 0.05 {
            let x = (0.05 - delta) * 19.99;
            if d < 0.5 {
                let mm = x * 0.5;
                let max = 1.0 - mm;
                if v > max && v < 1.0 {
                    max
                } else if v >= 1.0 && v < (1.0 + mm) {
                    1.0 + mm
                } else {
                    v
                }
            } else if v < 1.0 {
                v.clamp(x * 0.5, 1.0)
            } else {
                v
            }
        } else {
            v
        }
    }

    #[inline]
    pub fn next(&mut self, freq: f64, israte: f64, d: f64, v: f64) -> f64 {
        let s = -(TAU * Self::phi_vps(self.phase, v, d)).cos();
        self.phase += freq * israte;
        self.phase = self.phase.fract();
        s
    }
}

#[inline]
pub(crate) fn poly_blep_f64(t: f64, dt: f64) -> f64 {
    if t < dt {
        let t = t / dt;
        2.0 * t - t * t - 1.0
    } else if t > 1.0 - dt {
        let t = (t - 1.0) / dt;
        t * t + 2.0 * t + 1.0
    } else {
        0.0
    }
}

#[derive(Debug, Clone)]
pub struct PolyBlepOscillator {
    phase: f64,
    init_phase: f64,
    last_output: f64,
}

impl PolyBlepOscillator {
    pub fn new(init_phase: f64) -> Self {
        Self {
            phase: 0.0,
            last_output: 0.0,
            init_phase,
        }
    }

    pub fn reset(&mut self) {
        self.phase = self.init_phase;
        self.last_output = 0.0;
    }

    pub fn get_phase(&self) -> f64 {
        self.phase
    }

    pub fn set_phase(&mut self, phase: f64) {
        self.phase = phase;
    }

    #[inline]
    pub fn next_sin(&mut self, freq: f64, israte: f64) -> f64 {
        let s = (self.phase * TAU).sin();
        self.phase += freq * israte;
        self.phase = self.phase.fract();
        s
    }

    #[inline]
    pub fn next_saw(&mut self, freq: f64, israte: f64) -> f64 {
        let phase_inc = freq * israte;
        let mut s = 2.0 * self.phase - 1.0;
        s -= poly_blep_f64(self.phase, phase_inc);
        self.phase += phase_inc;
        self.phase = self.phase.fract();
        s
    }

    #[inline]
    pub fn next_tri(&mut self, freq: f64, israte: f64) -> f64 {
        let phase_inc = freq * israte;
        let mut s = if self.phase < 0.5 { 1.0 } else { -1.0 };
        s += poly_blep_f64(self.phase, phase_inc);
        s -= poly_blep_f64((self.phase + 0.5).fract(), phase_inc);
        s = phase_inc * s + (1.0 - phase_inc) * self.last_output;
        self.last_output = s;
        self.phase += phase_inc;
        self.phase = self.phase.fract();
        s * 4.0
    }

    #[inline]
    pub fn next_pulse(&mut self, freq: f64, israte: f64, pw: f64) -> f64 {
        let phase_inc = freq * israte;
        let pw = 0.1 * pw + (1.0 - pw) * 0.5;
        let dc_compensation = (0.5 - pw) * 2.0;
        let mut s = if self.phase < pw { 1.0 } else { -1.0 };
        s += poly_blep_f64(self.phase, phase_inc);
        s -= poly_blep_f64((self.phase + (1.0 - pw)).fract(), phase_inc);
        s += dc_compensation;
        self.phase += phase_inc;
        self.phase = self.phase.fract();
        s
    }

    #[inline]
    pub fn next_pulse_no_dc(&mut self, freq: f64, israte: f64, pw: f64) -> f64 {
        let phase_inc = freq * israte;
        let pw = 0.1 * pw + (1.0 - pw) * 0.5;
        let mut s = if self.phase < pw { 1.0 } else { -1.0 };
        s += poly_blep_f64(self.phase, phase_inc);
        s -= poly_blep_f64((self.phase + (1.0 - pw)).fract(), phase_inc);
        self.phase += phase_inc;
        self.phase = self.phase.fract();
        s
    }
}

#[inline]
pub fn f_distort(gain: f64, threshold: f64, i: f64) -> f64 {
    gain * (i * (i.abs() + threshold) / (i * i + (threshold - 1.0) * i.abs() + 1.0))
}

#[inline]
pub fn f_fold_distort(gain: f64, threshold: f64, i: f64) -> f64 {
    if i >= threshold || i < -threshold {
        gain * (((((i - threshold) % threshold) * 4.0).abs() - threshold * 2.0).abs() - threshold)
    } else {
        gain * i
    }
}

#[inline]
pub fn apply_distortion(s: f64, damt: f64, dist_type: u8) -> f64 {
    match dist_type {
        1 => (damt.clamp(0.01, 1.0) * 100.0 * s).tanh(),
        2 => f_distort(1.0, damt * damt * damt * 1000.0, s),
        3 => {
            let damt = damt.clamp(0.0, 0.99);
            let damt = 1.0 - damt * damt;
            f_fold_distort(1.0, damt, s) * (1.0 / damt)
        }
        _ => s,
    }
}
