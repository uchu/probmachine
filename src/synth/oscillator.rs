use synfx_dsp::{VPSOscillator, PolyBlepOscillator, rand_01};

#[derive(Clone, Copy, PartialEq)]
pub enum PllMode {
    AnalogLikePD,
    EdgePFD,
}

pub struct Oscillator {
    osc: VPSOscillator,
    sample_rate: f32,
    freq: f32,
    d: f32,
    v: f32,
    phase: f32,
}

impl Oscillator {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            osc: VPSOscillator::new(rand_01() * 0.25),
            sample_rate,
            freq: 220.0,
            d: 0.5,
            v: 0.5,
            phase: 0.0,
        }
    }

    pub fn set_frequency(&mut self, freq: f32) {
        self.freq = freq;
    }

    pub fn set_params(&mut self, d: f32, v: f32) {
        self.d = d;
        self.v = v;
    }

    pub fn next(&mut self, d: f32, v: f32) -> f32 {
        let israte = 1.0 / self.sample_rate;
        let v_limited = VPSOscillator::limit_v(d, v);

        self.phase += self.freq * israte;
        if self.phase >= 1.0 {
            self.phase -= 1.0;
        }

        self.osc.next(self.freq, israte, d, v_limited)
    }

    pub fn get_phase(&self) -> f32 {
        self.phase
    }
}

pub struct PolyBlepWrapper {
    osc: PolyBlepOscillator,
    sample_rate: f32,
    freq: f32,
    phase: f32,
}

impl PolyBlepWrapper {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            osc: PolyBlepOscillator::new(rand_01() * 0.25),
            sample_rate,
            freq: 220.0,
            phase: rand_01() * 0.25,
        }
    }

    pub fn set_frequency(&mut self, freq: f32) {
        self.freq = freq;
    }

    pub fn next(&mut self, pulse_width: f32) -> f32 {
        let israte = 1.0 / self.sample_rate;
        self.phase += self.freq * israte;
        if self.phase >= 1.0 {
            self.phase -= 1.0;
        }
        self.osc.next_pulse(self.freq, israte, pulse_width)
    }

    pub fn next_sin(&mut self) -> f32 {
        let israte = 1.0 / self.sample_rate;
        self.phase += self.freq * israte;
        if self.phase >= 1.0 {
            self.phase -= 1.0;
        }
        self.osc.next_sin(self.freq, israte)
    }

    pub fn get_phase(&self) -> f32 {
        self.phase
    }
}

pub struct PLLOscillator {
    phase: f32,
    integrator: f32,
    filtered_error: f32,
    jitter_state: f32,
    base_freq: f32,
    track_gain: f32,
    damping: f32,
    mult: f32,
    desired_mult: f32,
    range_coeff: f32,
    ki_multiplier: f32,
    colored: bool,
    color_x: f32,
    sample_rate: f32,

    mode: PllMode,
    last_ref_phase: f32,
    last_pll_phase: f32,
    last_ref_rising_smpl: i64,
    last_pll_rising_smpl: i64,
    last_ref_falling_smpl: i64,
    last_pll_falling_smpl: i64,
    sample_counter: i64,
    pfd_state: f32,

    cached_alpha: f32,
    cached_kp: f32,
    cached_ki: f32,
}

impl PLLOscillator {
    pub fn new(sample_rate: f32) -> Self {
        let mut pll = Self {
            phase: 0.0,
            integrator: 0.0,
            filtered_error: 0.0,
            jitter_state: 0.0,
            base_freq: 110.0,
            track_gain: 3.0,
            damping: 0.2,
            mult: 1.0,
            desired_mult: 1.0,
            range_coeff: 1.0,
            ki_multiplier: 10000.0,
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
        };
        pll.prepare_block();
        pll
    }

    pub fn set_frequency(&mut self, freq: f32) {
        self.base_freq = freq;
    }

    pub fn set_params(&mut self, track: f32, damp: f32, mult: f32, range: f32, colored: bool, mode: PllMode) {
        self.track_gain = track;
        self.damping = damp;
        self.desired_mult = mult;
        self.range_coeff = range;
        self.colored = colored;
        self.mode = mode;
    }

    pub fn set_ki_multiplier(&mut self, ki_mult: f32) {
        self.ki_multiplier = ki_mult;
    }

    pub fn prepare_block(&mut self) {
        self.mult += (self.desired_mult - self.mult) * 0.05;

        let a = (0.02 + 0.35 * self.track_gain.sqrt()).min(0.985);
        self.cached_alpha = a;

        let mult_penalty = 1.0 / self.mult.max(1.0);
        let damp_curve = 1.0 - (self.damping * 0.9).tanh();
        self.cached_kp = self.track_gain * 80.0 * (1.0 + 1.5 * (1.0 - self.damping)) * self.range_coeff * mult_penalty;
        self.cached_ki = damp_curve * self.track_gain * self.ki_multiplier * self.range_coeff * mult_penalty;

        self.color_x = if self.colored {
            (self.color_x + 0.05).min(1.0)
        } else {
            (self.color_x - 0.05).max(0.0)
        };
    }

    pub fn trigger(&mut self) {
        use synfx_dsp::rand_01;
        self.phase = rand_01() * 0.2;
        self.integrator = 0.0;
        self.filtered_error = 0.0;
        self.jitter_state = 0.0;
    }

    fn wrap_pi(x: f32) -> f32 {
        use std::f32::consts::PI;
        let two_pi = PI * 2.0;
        let y = x - two_pi * (x / two_pi).floor();
        if y > PI { y - two_pi } else { y }
    }

    fn detect_edges(_prev: f32, cur: f32, up_th: f32, dn_th: f32, was_high: bool)
        -> (bool /*rising*/, bool /*falling*/, bool /*is_high now*/) {
        if !was_high && cur >= up_th {
            (true, false, true)
        } else if was_high && cur <= dn_th {
            (false, true, false)
        } else {
            (false, false, was_high)
        }
    }

    fn next_pfd(&mut self, ref_sig: f32) -> f32 {
        self.sample_counter += 1;
        let pll_sig = ((self.phase * 2.0 * std::f32::consts::PI).sin()).signum();

        let rising_threshold = 0.02;
        let falling_threshold = -0.02;

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

        let dt_rising = (self.last_ref_rising_smpl - self.last_pll_rising_smpl) as f32;
        let dt_falling = (self.last_ref_falling_smpl - self.last_pll_falling_smpl) as f32;
        let dt = (dt_rising + dt_falling) * 0.5;

        let effective_freq = (self.base_freq * self.mult).max(1.0);
        let base_period = (self.sample_rate / effective_freq).max(1.0);
        let new_pfd_value = (dt / base_period).tanh();
        self.pfd_state = 0.8 * self.pfd_state + 0.2 * new_pfd_value;
        self.pfd_state
    }

    pub fn next(&mut self, input_phase: f32, input_freq: f32, ref_pulse: f32) -> f32 {
        use std::f32::consts::PI;
        use synfx_dsp::rand_01;

        self.base_freq = input_freq;

        let raw_noise = (rand_01() - 0.5) * 2.0;
        let j_alpha = 0.005;
        self.jitter_state = self.jitter_state * (1.0 - j_alpha) + raw_noise * j_alpha;
        let jitter = self.jitter_state * 0.01;

        let phase_error = match self.mode {
            PllMode::AnalogLikePD => {
                let diff = Self::wrap_pi(input_phase * 2.0 * PI - self.phase * 2.0 * PI);
                ((diff / PI) + jitter).tanh()
            }
            PllMode::EdgePFD => {
                self.next_pfd(ref_pulse)
            }
        };

        self.filtered_error = self.filtered_error * (1.0 - self.cached_alpha) + phase_error * self.cached_alpha;

        self.integrator = self.integrator * 0.9999 + self.filtered_error * (self.cached_ki / self.sample_rate);
        self.integrator = self.integrator.clamp(-2000.0, 2000.0);

        let correction = self.cached_kp * self.filtered_error + self.integrator;

        let target_freq = self.base_freq * self.mult + correction;

        let freq_jitter = (rand_01() - 0.5) * 0.002 * self.base_freq;
        let nyquist = 0.48 * self.sample_rate;
        let freq_control = (target_freq + freq_jitter).clamp(20.0, nyquist);

        self.phase += freq_control / self.sample_rate;
        if self.phase >= 1.0 {
            self.phase -= 1.0;
        }

        let clean = (self.phase * 2.0 * PI).sin();
        let colored = ((self.phase * self.mult) * 2.0 * PI).sin();
        let saturated = colored + 0.3 * colored.powi(3);
        let colored_out = saturated.clamp(-1.0, 1.0);

        clean * (1.0 - self.color_x) + colored_out * self.color_x
    }
}
