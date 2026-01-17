#![allow(dead_code)]

use synfx_dsp::{VPSOscillator, PolyBlepOscillator, rand_01};

#[derive(Clone, Copy, PartialEq)]
pub enum PllMode {
    AnalogLikePD,
    EdgePFD,
}

pub struct Oscillator {
    osc: VPSOscillator,
    sample_rate: f64,
    freq: f64,
    d: f64,
    v: f64,
    phase: f64,
}

impl Oscillator {
    pub fn new(sample_rate: f64) -> Self {
        Self {
            osc: VPSOscillator::new(rand_01() * 0.25),
            sample_rate,
            freq: 220.0,
            d: 0.5,
            v: 0.5,
            phase: 0.0,
        }
    }

    pub fn set_sample_rate(&mut self, sample_rate: f64) {
        self.sample_rate = sample_rate;
    }

    pub fn set_frequency(&mut self, freq: f64) {
        self.freq = freq;
    }

    pub fn set_params(&mut self, d: f64, v: f64) {
        self.d = d;
        self.v = v;
    }

    pub fn trigger(&mut self) {
        // Randomize phase slightly on trigger to avoid consistent DC offset clicks
        self.phase = rand_01() as f64 * 0.25;
    }

    pub fn next(&mut self, d: f64, v: f64) -> f64 {
        let israte = 1.0 / self.sample_rate;
        let d_f32 = d as f32;
        let v_f32 = v as f32;
        // limit_v() prevents DC offset by blending out problematic d/v combinations
        let v_limited = VPSOscillator::limit_v(d_f32, v_f32);

        self.phase += self.freq * israte;
        if self.phase >= 1.0 {
            self.phase -= 1.0;
        }

        self.osc.next(self.freq as f32, israte as f32, d_f32, v_limited) as f64
    }
}

pub struct PolyBlepWrapper {
    osc: PolyBlepOscillator,
    sample_rate: f64,
    freq: f64,
    phase: f64,
}

impl PolyBlepWrapper {
    pub fn new(sample_rate: f64) -> Self {
        Self {
            osc: PolyBlepOscillator::new(rand_01() * 0.25),
            sample_rate,
            freq: 220.0,
            phase: rand_01() as f64 * 0.25,
        }
    }

    pub fn set_sample_rate(&mut self, sample_rate: f64) {
        self.sample_rate = sample_rate;
    }

    pub fn set_frequency(&mut self, freq: f64) {
        self.freq = freq;
    }

    pub fn next(&mut self, pulse_width: f64) -> f64 {
        let israte = 1.0 / self.sample_rate;
        self.phase += self.freq * israte;
        if self.phase >= 1.0 {
            self.phase -= 1.0;
        }
        self.osc.next_pulse(self.freq as f32, israte as f32, pulse_width as f32) as f64
    }

    pub fn next_sin(&mut self) -> f64 {
        let israte = 1.0 / self.sample_rate;
        self.phase += self.freq * israte;
        if self.phase >= 1.0 {
            self.phase -= 1.0;
        }
        self.osc.next_sin(self.freq as f32, israte as f32) as f64
    }

    pub fn get_phase(&self) -> f64 {
        self.phase
    }

    pub fn reset_phase(&mut self) {
        self.phase = 0.0;
    }
}

pub struct PLLOscillator {
    phase: f64,
    integrator: f64,
    filtered_error: f64,
    base_freq: f64,
    track_speed: f64,
    damping: f64,
    influence: f64,
    mult: f64,
    desired_mult: f64,
    colored: bool,
    color_x: f64,
    sample_rate: f64,

    mode: PllMode,
    last_ref_phase: f64,
    last_pll_phase: f64,
    last_ref_rising_smpl: i64,
    last_pll_rising_smpl: i64,
    last_ref_falling_smpl: i64,
    last_pll_falling_smpl: i64,
    sample_counter: i64,
    pfd_state: f64,

    cached_alpha: f64,
    cached_kp: f64,
    cached_ki: f64,

    overtrack_state: f64,
    freq_slew_state: f64,
    phase_delta: f64,

    retrigger_amount: f64,
    burst_threshold: f64,
    burst_amount: f64,
    loop_saturation: f64,
    color_amount: f64,
    edge_sensitivity: f64,

    range: f64,
    mult_crossfade: f64,
    prev_mult_phase: f64,
}

impl PLLOscillator {
    pub fn new(sample_rate: f64) -> Self {
        let mut pll = Self {
            phase: 0.0,
            integrator: 0.0,
            filtered_error: 0.0,
            base_freq: 110.0,
            track_speed: 0.5,
            damping: 0.3,
            influence: 0.5,
            mult: 1.0,
            desired_mult: 1.0,
            colored: false,
            color_x: 0.0,
            sample_rate,

            mode: PllMode::AnalogLikePD,
            last_ref_phase: 0.0,
            last_pll_phase: 0.0,
            last_ref_rising_smpl: 0,
            last_pll_rising_smpl: 0,
            last_ref_falling_smpl: 0,
            last_pll_falling_smpl: 0,
            sample_counter: 0,
            pfd_state: 0.0,

            cached_alpha: 0.0,
            cached_kp: 0.0,
            cached_ki: 0.0,

            overtrack_state: 0.0,
            freq_slew_state: 0.0,
            phase_delta: 0.0,

            retrigger_amount: 0.05,
            burst_threshold: 0.7,
            burst_amount: 3.3,
            loop_saturation: 1000.0,
            color_amount: 0.25,
            edge_sensitivity: 0.02,

            range: 1.0,
            mult_crossfade: 1.0,
            prev_mult_phase: 0.0,
        };
        pll.prepare_block();
        pll
    }

    pub fn set_sample_rate(&mut self, sample_rate: f64) {
        self.sample_rate = sample_rate;
        self.prepare_block();
    }

    pub fn set_params(&mut self, track: f64, damp: f64, mult: f64, influence: f64, colored: bool, mode: PllMode) {
        self.track_speed = track.clamp(0.0, 1.0);
        self.damping = damp.clamp(0.0, 1.0);
        self.desired_mult = mult;
        self.influence = influence.clamp(0.0, 1.0);
        self.colored = colored;
        self.mode = mode;
    }

    pub fn set_experimental_params(
        &mut self,
        retrigger: f64,
        burst_threshold: f64,
        burst_amount: f64,
        loop_saturation: f64,
        color_amount: f64,
        edge_sensitivity: f64,
        range: f64,
    ) {
        self.retrigger_amount = retrigger.clamp(0.0, 1.0);
        self.burst_threshold = burst_threshold.clamp(0.0, 1.0);
        self.burst_amount = burst_amount.clamp(0.0, 10.0);
        self.loop_saturation = loop_saturation.clamp(1.0, 500.0);
        self.color_amount = color_amount.clamp(0.0, 1.0);
        self.edge_sensitivity = edge_sensitivity.clamp(0.001, 0.2);
        self.range = range.clamp(0.0, 1.0);
    }

    #[allow(dead_code)]
    pub fn get_phase_delta(&self) -> f64 {
        self.phase_delta
    }

    pub fn prepare_block(&mut self) {
        let old_mult = self.mult;
        self.mult += (self.desired_mult - self.mult) * 0.02;
        if (self.mult - old_mult).abs() > 0.01 {
            self.prev_mult_phase = self.phase * old_mult;
            self.mult_crossfade = 0.0;
        }
        if self.mult_crossfade < 1.0 {
            self.mult_crossfade = (self.mult_crossfade + 0.02).min(1.0);
        }

        let track_exp = (self.track_speed * 6.0 - 3.0).exp();
        self.cached_alpha = (track_exp / (1.0 + track_exp)).clamp(0.001, 0.995);

        let range_scale = (self.range * self.range * self.range).max(0.0001);
        let mult_penalty = 1.0 / self.mult.sqrt().max(1.0);
        let influence_scaled = 0.1 + self.influence * 9.9;

        let damp_factor = 1.0 - self.damping * 0.95;
        self.cached_kp = influence_scaled * 50.0 * damp_factor * mult_penalty * range_scale;
        self.cached_ki = influence_scaled * 5000.0 * damp_factor * mult_penalty * range_scale;

        self.cached_alpha *= range_scale.sqrt();

        self.color_x = if self.colored {
            (self.color_x + 0.02).min(1.0)
        } else {
            (self.color_x - 0.02).max(0.0)
        };
    }

    pub fn trigger(&mut self) {
        self.integrator = 0.0;
        self.filtered_error = 0.0;
        self.pfd_state = 0.0;
        self.overtrack_state = 0.0;
        self.phase = 0.0;
    }

    fn wrap_pi(x: f64) -> f64 {
        use std::f64::consts::PI;
        let two_pi = PI * 2.0;
        let y = x - two_pi * (x / two_pi).floor();
        if y > PI { y - two_pi } else { y }
    }

    fn detect_edges(_prev: f64, cur: f64, up_th: f64, dn_th: f64, was_high: bool)
        -> (bool /*rising*/, bool /*falling*/, bool /*is_high now*/) {
        if !was_high && cur >= up_th {
            (true, false, true)
        } else if was_high && cur <= dn_th {
            (false, true, false)
        } else {
            (false, false, was_high)
        }
    }

    fn next_pfd(&mut self, ref_sig: f64) -> f64 {
        self.sample_counter += 1;
        let pll_sig = ((self.phase * 2.0 * std::f64::consts::PI).sin()).signum();

        let rising_threshold = self.edge_sensitivity;
        let falling_threshold = -self.edge_sensitivity;

        let (ref_rising, ref_falling, ref_high) = Self::detect_edges(
            self.last_ref_phase, ref_sig, rising_threshold, falling_threshold, self.last_ref_phase > 0.0
        );
        let (pll_rising, pll_falling, pll_high) = Self::detect_edges(
            self.last_pll_phase, pll_sig, rising_threshold, falling_threshold, self.last_pll_phase > 0.0
        );

        if ref_rising { self.last_ref_rising_smpl = self.sample_counter; }
        if ref_falling { self.last_ref_falling_smpl = self.sample_counter; }
        if pll_rising { self.last_pll_rising_smpl = self.sample_counter; }
        if pll_falling { self.last_pll_falling_smpl = self.sample_counter; }

        self.last_ref_phase = if ref_high { 1.0 } else { -1.0 };
        self.last_pll_phase = if pll_high { 1.0 } else { -1.0 };

        let dt_rising = (self.last_ref_rising_smpl - self.last_pll_rising_smpl) as f64;
        let dt_falling = (self.last_ref_falling_smpl - self.last_pll_falling_smpl) as f64;
        let dt = (dt_rising + dt_falling) * 0.5;

        let effective_freq = (self.base_freq * self.mult).max(1.0);
        let base_period = (self.sample_rate / effective_freq).max(1.0);
        let new_pfd_value = (dt / base_period).tanh();
        self.pfd_state = self.pfd_state * 0.85 + new_pfd_value * 0.15;
        self.pfd_state
    }

    pub fn next(&mut self, input_phase: f64, input_freq: f64, ref_pulse: f64) -> f64 {
        use std::f64::consts::PI;

        self.base_freq = input_freq;

        let phase_error = match self.mode {
            PllMode::AnalogLikePD => {
                let diff = Self::wrap_pi(input_phase * 2.0 * PI - self.phase * 2.0 * PI);
                (diff / PI).tanh()
            }
            PllMode::EdgePFD => {
                self.next_pfd(ref_pulse)
            }
        };

        self.phase_delta = phase_error;

        self.filtered_error = self.filtered_error * (1.0 - self.cached_alpha) + phase_error * self.cached_alpha;

        let integrator_decay = 0.9999 - self.damping * 0.0098;
        self.integrator = self.integrator * integrator_decay + self.filtered_error * (self.cached_ki / self.sample_rate);
        self.integrator = self.integrator.clamp(-self.loop_saturation, self.loop_saturation);

        let overtrack_amount = (self.track_speed - self.burst_threshold).max(0.0) * self.burst_amount;
        let overtrack_resonance = 1.0 + overtrack_amount * (1.0 - self.damping);
        let correction = (self.cached_kp * self.filtered_error + self.integrator) * overtrack_resonance;

        let target_freq = self.base_freq * self.mult;
        let corrected_freq = target_freq + correction;

        if overtrack_amount > 0.01 {
            let overshoot = (correction / self.base_freq.max(20.0)).abs();
            self.overtrack_state = self.overtrack_state * 0.99 + overshoot * 0.01;
            let burst = self.overtrack_state * overtrack_amount * 100.0 * (1.0 - self.damping);
            let burst_freq = corrected_freq + burst * (if self.filtered_error > 0.0 { 1.0 } else { -1.0 });
            let nyquist = 0.48 * self.sample_rate;
            self.phase += burst_freq.clamp(20.0, nyquist) / self.sample_rate;
        } else {
            let nyquist = 0.48 * self.sample_rate;
            self.phase += corrected_freq.clamp(20.0, nyquist) / self.sample_rate;
        }

        if self.phase >= 1.0 {
            self.phase -= 1.0;
        } else if self.phase < 0.0 {
            self.phase += 1.0;
        }

        let clean = (self.phase * 2.0 * PI).sin();

        let colored_phase = self.phase * self.mult;
        let prev_colored = (self.prev_mult_phase * 2.0 * PI).sin();
        let curr_colored = (colored_phase * 2.0 * PI).sin();
        let colored = prev_colored * (1.0 - self.mult_crossfade) + curr_colored * self.mult_crossfade;

        let saturated = colored + self.color_amount * colored.powi(3);
        let colored_out = saturated.clamp(-1.0, 1.0);

        self.prev_mult_phase += self.base_freq * self.desired_mult / self.sample_rate;
        if self.prev_mult_phase >= 1.0 {
            self.prev_mult_phase -= 1.0;
        }

        let raw_output = clean * (1.0 - self.color_x) + colored_out * self.color_x;
        raw_output.clamp(-1.0, 1.0)
    }
}
