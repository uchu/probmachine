#![allow(dead_code)]

use synfx_dsp::SlewValue;

struct BiquadState {
    x1: f64,
    x2: f64,
    y1: f64,
    y2: f64,
}

impl BiquadState {
    fn new() -> Self {
        Self { x1: 0.0, x2: 0.0, y1: 0.0, y2: 0.0 }
    }

    fn reset(&mut self) {
        self.x1 = 0.0;
        self.x2 = 0.0;
        self.y1 = 0.0;
        self.y2 = 0.0;
    }
}

pub struct FormantFilter {
    sample_rate: f64,

    // 3 bandpass filters for formant frequencies
    bp1_freq: f64,
    bp1_state: BiquadState,

    bp2_freq: f64,
    bp2_state: BiquadState,

    bp3_freq: f64,
    bp3_state: BiquadState,

    // Slew for smooth vowel transitions
    freq1_slew: SlewValue<f64>,
    freq2_slew: SlewValue<f64>,
    freq3_slew: SlewValue<f64>,
}

// Formant frequencies for vowels (F1, F2, F3 in Hz)
// Based on average adult vocal formant data for natural sounding vowels
// F1 correlates with tongue height (low F1 = high tongue)
// F2 correlates with tongue position (high F2 = front vowel)
// F3 adds character and naturalness
const VOWELS: [(f64, f64, f64); 5] = [
    (730.0, 1090.0, 2440.0),   // A (ah) - open back vowel
    (530.0, 1840.0, 2480.0),   // E (eh) - mid front vowel
    (270.0, 2290.0, 3010.0),   // I (ee) - close front vowel
    (570.0, 840.0, 2410.0),    // O (oh) - mid back rounded
    (300.0, 870.0, 2240.0),    // U (oo) - close back rounded
];

impl FormantFilter {
    pub fn new(sample_rate: f64) -> Self {
        let mut freq1_slew = SlewValue::new();
        let mut freq2_slew = SlewValue::new();
        let mut freq3_slew = SlewValue::new();
        freq1_slew.set_sample_rate(sample_rate);
        freq2_slew.set_sample_rate(sample_rate);
        freq3_slew.set_sample_rate(sample_rate);

        Self {
            sample_rate,
            bp1_freq: 800.0,
            bp1_state: BiquadState::new(),
            bp2_freq: 1200.0,
            bp2_state: BiquadState::new(),
            bp3_freq: 2500.0,
            bp3_state: BiquadState::new(),
            freq1_slew,
            freq2_slew,
            freq3_slew,
        }
    }

    fn interpolate_vowel(vowel: f64) -> (f64, f64, f64) {
        let vowel = vowel.clamp(0.0, 1.0);
        let scaled = vowel * 4.0; // 0-4 for 5 vowels
        let idx = (scaled as usize).min(3);
        let frac = scaled - idx as f64;

        let (f1a, f2a, f3a) = VOWELS[idx];
        let (f1b, f2b, f3b) = VOWELS[idx + 1];

        (
            f1a + (f1b - f1a) * frac,
            f2a + (f2b - f2a) * frac,
            f3a + (f3b - f3a) * frac,
        )
    }

    pub fn set_vowel(&mut self, vowel: f64, shift: f64) {
        let (f1, f2, f3) = Self::interpolate_vowel(vowel);

        // Shift formants up or down (in semitones)
        let shift_factor = 2.0_f64.powf(shift);

        self.bp1_freq = self.freq1_slew.next(f1 * shift_factor, 50.0);
        self.bp2_freq = self.freq2_slew.next(f2 * shift_factor, 50.0);
        self.bp3_freq = self.freq3_slew.next(f3 * shift_factor, 50.0);
    }

    pub fn process(&mut self, input: f64) -> f64 {
        // Lower Q values = wider bandwidth = more audible formants
        let q1 = 2.5;  // F1 - widest (lowest formant)
        let q2 = 3.5;  // F2 - medium
        let q3 = 4.5;  // F3 - narrower (highest formant)

        let out1 = process_bandpass(input, self.bp1_freq.clamp(50.0, 10000.0), q1,
                                    self.sample_rate, &mut self.bp1_state);
        let out2 = process_bandpass(input, self.bp2_freq.clamp(50.0, 15000.0), q2,
                                    self.sample_rate, &mut self.bp2_state);
        let out3 = process_bandpass(input, self.bp3_freq.clamp(50.0, 18000.0), q3,
                                    self.sample_rate, &mut self.bp3_state);

        // Mix formants with significant gain to make them clearly audible
        // F1 (lowest) gets highest gain as it carries most energy
        let mixed = out1 * 3.0 + out2 * 2.5 + out3 * 2.0;
        (mixed * 2.0).tanh()
    }
}

fn process_bandpass(input: f64, freq: f64, q: f64, sample_rate: f64, state: &mut BiquadState) -> f64 {
    let omega = 2.0 * std::f64::consts::PI * freq / sample_rate;
    let sin_omega = omega.sin();
    let cos_omega = omega.cos();
    let alpha = sin_omega / (2.0 * q);

    // Bandpass filter coefficients (constant 0 dB peak gain)
    let b0 = alpha;
    let b1 = 0.0;
    let b2 = -alpha;
    let a0 = 1.0 + alpha;
    let a1 = -2.0 * cos_omega;
    let a2 = 1.0 - alpha;

    // Normalize coefficients
    let b0n = b0 / a0;
    let b1n = b1 / a0;
    let b2n = b2 / a0;
    let a1n = a1 / a0;
    let a2n = a2 / a0;

    // Direct Form I biquad: y[n] = b0*x[n] + b1*x[n-1] + b2*x[n-2] - a1*y[n-1] - a2*y[n-2]
    let output = b0n * input + b1n * state.x1 + b2n * state.x2
               - a1n * state.y1 - a2n * state.y2;

    // Update state (input and output history)
    state.x2 = state.x1;
    state.x1 = input;
    state.y2 = state.y1;
    state.y1 = output;

    output
}
