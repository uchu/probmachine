use std::f64::consts::PI;

const K: f64 = std::f64::consts::SQRT_2;
const MIN_CROSSOVER_HZ: f64 = 20.0;

#[derive(Clone, Copy)]
struct SvfState {
    ic1eq: f64,
    ic2eq: f64,
}

impl SvfState {
    fn new() -> Self {
        Self { ic1eq: 0.0, ic2eq: 0.0 }
    }

    fn reset(&mut self) {
        self.ic1eq = 0.0;
        self.ic2eq = 0.0;
    }

    #[inline]
    fn tick_lowpass(&mut self, input: f64, a1: f64, a2: f64, a3: f64) -> f64 {
        let v3 = input - self.ic2eq;
        let v1 = a1 * self.ic1eq + a2 * v3;
        let v2 = self.ic2eq + a2 * self.ic1eq + a3 * v3;
        self.ic1eq = 2.0 * v1 - self.ic1eq;
        self.ic2eq = 2.0 * v2 - self.ic2eq;
        v2
    }

    #[inline]
    fn tick_highpass(&mut self, input: f64, a1: f64, a2: f64, a3: f64) -> f64 {
        let v3 = input - self.ic2eq;
        let v1 = a1 * self.ic1eq + a2 * v3;
        let v2 = self.ic2eq + a2 * self.ic1eq + a3 * v3;
        self.ic1eq = 2.0 * v1 - self.ic1eq;
        self.ic2eq = 2.0 * v2 - self.ic2eq;
        input - K * v1 - v2
    }
}

pub struct StereoControl {
    lpf1_l: SvfState,
    lpf2_l: SvfState,
    lpf1_r: SvfState,
    lpf2_r: SvfState,
    hpf1_l: SvfState,
    hpf2_l: SvfState,
    hpf1_r: SvfState,
    hpf2_r: SvfState,

    a1: f64,
    a2: f64,
    a3: f64,

    crossover_hz: f64,
    width: f64,
    width_slew: f64,
    slew_coeff: f64,
    sample_rate: f64,
}

impl StereoControl {
    pub fn new(sample_rate: f32) -> Self {
        let sr = sample_rate as f64;
        let slew_coeff = 1.0 - (-1.0 / (sr * 0.005)).exp();
        Self {
            lpf1_l: SvfState::new(),
            lpf2_l: SvfState::new(),
            lpf1_r: SvfState::new(),
            lpf2_r: SvfState::new(),
            hpf1_l: SvfState::new(),
            hpf2_l: SvfState::new(),
            hpf1_r: SvfState::new(),
            hpf2_r: SvfState::new(),
            a1: 0.0,
            a2: 0.0,
            a3: 0.0,
            crossover_hz: 0.0,
            width: 1.0,
            width_slew: 1.0,
            slew_coeff,
            sample_rate: sr,
        }
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate as f64;
        self.slew_coeff = 1.0 - (-1.0 / (self.sample_rate * 0.005)).exp();
        if self.crossover_hz >= MIN_CROSSOVER_HZ {
            self.update_coefficients();
        }
    }

    pub fn set_crossover_hz(&mut self, hz: f64) {
        if (hz - self.crossover_hz).abs() > 0.01 {
            self.crossover_hz = hz;
            if hz < MIN_CROSSOVER_HZ {
                self.reset_states();
            } else {
                self.update_coefficients();
            }
        }
    }

    pub fn set_width(&mut self, width: f64) {
        self.width = width;
    }

    fn update_coefficients(&mut self) {
        let g = (PI * self.crossover_hz / self.sample_rate).tan();
        self.a1 = 1.0 / (1.0 + g * (g + K));
        self.a2 = g * self.a1;
        self.a3 = g * self.a2;
    }

    fn reset_states(&mut self) {
        self.lpf1_l.reset();
        self.lpf2_l.reset();
        self.lpf1_r.reset();
        self.lpf2_r.reset();
        self.hpf1_l.reset();
        self.hpf2_l.reset();
        self.hpf1_r.reset();
        self.hpf2_r.reset();
    }

    #[inline]
    pub fn process_block(&mut self, buffer_l: &mut [f32], buffer_r: &mut [f32]) {
        let mono_bass_on = self.crossover_hz >= MIN_CROSSOVER_HZ;
        let width_differs = (self.width - 1.0).abs() > 0.001 || (self.width_slew - 1.0).abs() > 0.001;

        if !mono_bass_on && !width_differs {
            return;
        }

        let a1 = self.a1;
        let a2 = self.a2;
        let a3 = self.a3;

        if mono_bass_on {
            for (l, r) in buffer_l.iter_mut().zip(buffer_r.iter_mut()) {
                self.width_slew += (self.width - self.width_slew) * self.slew_coeff;

                let in_l = *l as f64;
                let in_r = *r as f64;

                let lp1_l = self.lpf1_l.tick_lowpass(in_l, a1, a2, a3);
                let low_l = self.lpf2_l.tick_lowpass(lp1_l, a1, a2, a3);

                let lp1_r = self.lpf1_r.tick_lowpass(in_r, a1, a2, a3);
                let low_r = self.lpf2_r.tick_lowpass(lp1_r, a1, a2, a3);

                let hp1_l = self.hpf1_l.tick_highpass(in_l, a1, a2, a3);
                let high_l = self.hpf2_l.tick_highpass(hp1_l, a1, a2, a3);

                let hp1_r = self.hpf1_r.tick_highpass(in_r, a1, a2, a3);
                let high_r = self.hpf2_r.tick_highpass(hp1_r, a1, a2, a3);

                let low_mono = (low_l + low_r) * 0.5;

                let mid = (high_l + high_r) * 0.5;
                let side = (high_l - high_r) * 0.5;
                let scaled_side = side * self.width_slew;
                let out_high_l = mid + scaled_side;
                let out_high_r = mid - scaled_side;

                *l = (low_mono + out_high_l) as f32;
                *r = (low_mono + out_high_r) as f32;
            }
        } else {
            for (l, r) in buffer_l.iter_mut().zip(buffer_r.iter_mut()) {
                self.width_slew += (self.width - self.width_slew) * self.slew_coeff;

                let in_l = *l as f64;
                let in_r = *r as f64;

                let mid = (in_l + in_r) * 0.5;
                let side = (in_l - in_r) * 0.5;
                let scaled_side = side * self.width_slew;

                *l = (mid + scaled_side) as f32;
                *r = (mid - scaled_side) as f32;
            }
        }
    }
}
