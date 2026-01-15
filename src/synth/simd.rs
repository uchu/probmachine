#![allow(dead_code)]

use std::simd::prelude::*;

pub type Stereo = f64x2;
pub type StereoF32 = f32x2;

#[inline(always)]
pub fn stereo(left: f64, right: f64) -> Stereo {
    f64x2::from_array([left, right])
}

#[inline(always)]
pub fn stereo_splat(value: f64) -> Stereo {
    Stereo::splat(value)
}

#[inline(always)]
pub fn stereo_from_mono(value: f64) -> Stereo {
    Stereo::splat(value)
}

#[inline(always)]
pub fn stereo_left(s: Stereo) -> f64 {
    s[0]
}

#[inline(always)]
pub fn stereo_right(s: Stereo) -> f64 {
    s[1]
}

#[inline(always)]
pub fn stereo_to_array(s: Stereo) -> [f64; 2] {
    s.to_array()
}

#[inline(always)]
pub fn stereo_mul(a: Stereo, b: Stereo) -> Stereo {
    a * b
}

#[inline(always)]
pub fn stereo_add(a: Stereo, b: Stereo) -> Stereo {
    a + b
}

#[inline(always)]
pub fn stereo_sub(a: Stereo, b: Stereo) -> Stereo {
    a - b
}

#[inline(always)]
pub fn stereo_scale(s: Stereo, factor: f64) -> Stereo {
    s * Stereo::splat(factor)
}

#[inline(always)]
pub fn stereo_mix(dry: Stereo, wet: Stereo, mix: f64) -> Stereo {
    let mix_v = Stereo::splat(mix);
    let dry_amt = Stereo::splat(1.0) - mix_v;
    dry * dry_amt + wet * mix_v
}

#[inline(always)]
pub fn stereo_clamp(s: Stereo, min: f64, max: f64) -> Stereo {
    s.simd_clamp(Stereo::splat(min), Stereo::splat(max))
}

#[inline(always)]
pub fn stereo_abs(s: Stereo) -> Stereo {
    s.abs()
}

#[inline(always)]
pub fn stereo_min(a: Stereo, b: Stereo) -> Stereo {
    a.simd_min(b)
}

#[inline(always)]
pub fn stereo_max(a: Stereo, b: Stereo) -> Stereo {
    a.simd_max(b)
}

#[inline(always)]
pub fn stereo_sum(s: Stereo) -> f64 {
    s[0] + s[1]
}

#[inline(always)]
pub fn stereo_avg(s: Stereo) -> f64 {
    (s[0] + s[1]) * 0.5
}

#[inline(always)]
pub fn stereo_tanh(s: Stereo) -> Stereo {
    let x = s;
    let x2 = x * x;
    let x3 = x2 * x;
    let x5 = x3 * x2;
    let x7 = x5 * x2;
    let result = x - x3 * Stereo::splat(1.0 / 3.0)
                   + x5 * Stereo::splat(2.0 / 15.0)
                   - x7 * Stereo::splat(17.0 / 315.0);
    stereo_clamp(result, -1.0, 1.0)
}

#[inline(always)]
pub fn stereo_tanh_accurate(s: Stereo) -> Stereo {
    stereo(s[0].tanh(), s[1].tanh())
}

#[inline(always)]
pub fn stereo_sin(s: Stereo) -> Stereo {
    stereo(s[0].sin(), s[1].sin())
}

#[inline(always)]
pub fn stereo_soft_clip(s: Stereo, threshold: f64) -> Stereo {
    let thresh = Stereo::splat(threshold);
    let neg_thresh = Stereo::splat(-threshold);
    let above = s.simd_gt(thresh);
    let below = s.simd_lt(neg_thresh);

    let soft_pos = thresh + (s - thresh) * Stereo::splat(0.5);
    let soft_neg = neg_thresh + (s - neg_thresh) * Stereo::splat(0.5);

    let result = above.select(soft_pos, s);
    below.select(soft_neg, result)
}

#[inline(always)]
pub fn stereo_wavefold(s: Stereo, amount: f64) -> Stereo {
    let fold_gain = 1.0 + amount * 4.0;
    let x = s * Stereo::splat(fold_gain);
    let folded = stereo_sin(x);
    let amt = Stereo::splat(amount);
    let one_minus_amt = Stereo::splat(1.0 - amount);
    s * one_minus_amt + folded * amt
}

#[inline(always)]
pub fn stereo_tube_saturate(s: Stereo, drive: f64) -> Stereo {
    let drive_factor = 1.0 + drive * 3.0;
    let driven = s * Stereo::splat(drive_factor);

    let pos_clip = driven.simd_min(Stereo::splat(1.0));
    let neg_clip = (-driven).simd_min(Stereo::splat(0.8));

    let cubic = |x: Stereo| -> Stereo {
        let x3 = x * x * x;
        (x - x3 * Stereo::splat(1.0 / 3.0)) * Stereo::splat(1.0 / (1.0 - 1.0 / 3.0))
    };

    let pos_result = cubic(pos_clip);
    let neg_result = -cubic(neg_clip) * Stereo::splat(0.9);

    let is_positive = driven.simd_ge(Stereo::splat(0.0));
    let result = is_positive.select(pos_result, neg_result);

    result * Stereo::splat(1.0 - drive * 0.3)
}

/// Bram de Jong waveshaper distortion (same as synfx_dsp::f_distort)
/// f(x,a) = x*(abs(x) + a)/(x^2 + (a-1)*abs(x) + 1)
#[inline(always)]
pub fn stereo_distort_bram(s: Stereo, gain: f64, threshold: f64) -> Stereo {
    let abs_s = s.abs();
    let s2 = s * s;
    let numerator = s * (abs_s + Stereo::splat(threshold));
    let denominator = s2 + (Stereo::splat(threshold - 1.0) * abs_s) + Stereo::splat(1.0);
    Stereo::splat(gain) * (numerator / denominator)
}

#[inline(always)]
pub fn stereo_to_f32(s: Stereo) -> StereoF32 {
    StereoF32::from_array([s[0] as f32, s[1] as f32])
}

#[inline(always)]
pub fn stereo_from_f32(s: StereoF32) -> Stereo {
    Stereo::from_array([s[0] as f64, s[1] as f64])
}

pub struct StereoSlewValue {
    current: Stereo,
    sample_rate: f64,
}

impl StereoSlewValue {
    pub fn new() -> Self {
        Self {
            current: stereo_splat(0.0),
            sample_rate: 44100.0,
        }
    }

    pub fn set_sample_rate(&mut self, sample_rate: f64) {
        self.sample_rate = sample_rate;
    }

    #[inline(always)]
    pub fn next(&mut self, target: Stereo, time_ms: f64) -> Stereo {
        let samples = (time_ms * self.sample_rate / 1000.0).max(1.0);
        let coeff = Stereo::splat(1.0 / samples);
        self.current = self.current + (target - self.current) * coeff;
        self.current
    }

    #[inline(always)]
    pub fn set(&mut self, value: Stereo) {
        self.current = value;
    }

    #[inline(always)]
    pub fn get(&self) -> Stereo {
        self.current
    }
}

impl Default for StereoSlewValue {
    fn default() -> Self {
        Self::new()
    }
}

pub struct StereoOnePoleLPF {
    state: Stereo,
}

impl StereoOnePoleLPF {
    pub fn new() -> Self {
        Self {
            state: stereo_splat(0.0),
        }
    }

    #[inline(always)]
    pub fn process(&mut self, input: Stereo, coeff: f64) -> Stereo {
        let c = Stereo::splat(coeff);
        self.state = self.state + (input - self.state) * c;
        self.state
    }

    pub fn reset(&mut self) {
        self.state = stereo_splat(0.0);
    }
}

impl Default for StereoOnePoleLPF {
    fn default() -> Self {
        Self::new()
    }
}

pub struct StereoDCBlocker {
    x_prev: Stereo,
    y_prev: Stereo,
    coeff: f64,
}

impl StereoDCBlocker {
    pub fn new() -> Self {
        Self {
            x_prev: stereo_splat(0.0),
            y_prev: stereo_splat(0.0),
            coeff: 0.998,
        }
    }

    pub fn set_coeff(&mut self, coeff: f64) {
        self.coeff = coeff.clamp(0.9, 0.9999);
    }

    #[inline(always)]
    pub fn process(&mut self, input: Stereo) -> Stereo {
        let output = input - self.x_prev + self.y_prev * Stereo::splat(self.coeff);
        self.x_prev = input;
        self.y_prev = output;
        output
    }

    pub fn reset(&mut self) {
        self.x_prev = stereo_splat(0.0);
        self.y_prev = stereo_splat(0.0);
    }
}

impl Default for StereoDCBlocker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stereo_basics() {
        let s = stereo(1.0, 2.0);
        assert_eq!(stereo_left(s), 1.0);
        assert_eq!(stereo_right(s), 2.0);
    }

    #[test]
    fn test_stereo_mix() {
        let dry = stereo(1.0, 1.0);
        let wet = stereo(0.0, 0.0);
        let mixed = stereo_mix(dry, wet, 0.5);
        assert!((stereo_left(mixed) - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_stereo_clamp() {
        let s = stereo(-2.0, 2.0);
        let clamped = stereo_clamp(s, -1.0, 1.0);
        assert_eq!(stereo_left(clamped), -1.0);
        assert_eq!(stereo_right(clamped), 1.0);
    }

    #[test]
    fn test_stereo_tanh() {
        let s = stereo(0.0, 0.5);
        let result = stereo_tanh(s);
        assert!((stereo_left(result) - 0.0).abs() < 0.01);
        assert!((stereo_right(result) - 0.46).abs() < 0.1);
    }

    #[test]
    fn test_stereo_conversion() {
        let s = stereo(1.5, -0.5);
        let f32_s = stereo_to_f32(s);
        let back = stereo_from_f32(f32_s);
        assert!((stereo_left(back) - 1.5).abs() < 0.0001);
        assert!((stereo_right(back) + 0.5).abs() < 0.0001);
    }
}
