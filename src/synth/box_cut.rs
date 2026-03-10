use std::f64::consts::PI;

const CENTER_HZ: f64 = 400.0;
const Q: f64 = 1.5;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum BoxCutMode {
    Off,
    Low,
    Med,
    High,
}

impl BoxCutMode {
    pub fn from_index(idx: i32) -> Self {
        match idx {
            1 => BoxCutMode::Low,
            2 => BoxCutMode::Med,
            3 => BoxCutMode::High,
            _ => BoxCutMode::Off,
        }
    }

    fn amount(&self) -> f64 {
        match self {
            BoxCutMode::Off => 0.0,
            BoxCutMode::Low => 0.2921,   // -3 dB
            BoxCutMode::Med => 0.4988,   // -6 dB
            BoxCutMode::High => 0.7488,  // -12 dB
        }
    }
}

pub struct BoxCutFilter {
    ic1eq_l: f64,
    ic2eq_l: f64,
    ic1eq_r: f64,
    ic2eq_r: f64,
    g: f64,
    k: f64,
    a1: f64,
    a2: f64,
    a3: f64,
    mode: BoxCutMode,
    sample_rate: f64,
}

impl BoxCutFilter {
    pub fn new(sample_rate: f32) -> Self {
        let mut f = Self {
            ic1eq_l: 0.0,
            ic2eq_l: 0.0,
            ic1eq_r: 0.0,
            ic2eq_r: 0.0,
            g: 0.0,
            k: 0.0,
            a1: 0.0,
            a2: 0.0,
            a3: 0.0,
            mode: BoxCutMode::Off,
            sample_rate: sample_rate as f64,
        };
        f.update_coefficients();
        f
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate as f64;
        self.update_coefficients();
    }

    pub fn set_mode(&mut self, mode: BoxCutMode) {
        if mode != self.mode {
            self.mode = mode;
            if mode == BoxCutMode::Off {
                self.ic1eq_l = 0.0;
                self.ic2eq_l = 0.0;
                self.ic1eq_r = 0.0;
                self.ic2eq_r = 0.0;
            }
        }
    }

    fn update_coefficients(&mut self) {
        self.g = (PI * CENTER_HZ / self.sample_rate).tan();
        self.k = 1.0 / Q;
        self.a1 = 1.0 / (1.0 + self.g * (self.g + self.k));
        self.a2 = self.g * self.a1;
        self.a3 = self.g * self.a2;
    }

    #[inline]
    pub fn process_block(&mut self, buffer_l: &mut [f32], buffer_r: &mut [f32]) {
        if self.mode == BoxCutMode::Off {
            return;
        }

        let a1 = self.a1;
        let a2 = self.a2;
        let a3 = self.a3;
        let k = self.k;
        let amount = self.mode.amount();

        for (l, r) in buffer_l.iter_mut().zip(buffer_r.iter_mut()) {
            let input_l = *l as f64;
            let v3_l = input_l - self.ic2eq_l;
            let v1_l = a1 * self.ic1eq_l + a2 * v3_l;
            let v2_l = self.ic2eq_l + a2 * self.ic1eq_l + a3 * v3_l;
            self.ic1eq_l = 2.0 * v1_l - self.ic1eq_l;
            self.ic2eq_l = 2.0 * v2_l - self.ic2eq_l;
            *l = (input_l - amount * k * v1_l) as f32;

            let input_r = *r as f64;
            let v3_r = input_r - self.ic2eq_r;
            let v1_r = a1 * self.ic1eq_r + a2 * v3_r;
            let v2_r = self.ic2eq_r + a2 * self.ic1eq_r + a3 * v3_r;
            self.ic1eq_r = 2.0 * v1_r - self.ic1eq_r;
            self.ic2eq_r = 2.0 * v2_r - self.ic2eq_r;
            *r = (input_r - amount * k * v1_r) as f32;
        }
    }

    #[inline]
    pub fn process_mono(&mut self, buffer: &mut [f32]) {
        if self.mode == BoxCutMode::Off {
            return;
        }

        let a1 = self.a1;
        let a2 = self.a2;
        let a3 = self.a3;
        let k = self.k;
        let amount = self.mode.amount();

        for sample in buffer.iter_mut() {
            let input = *sample as f64;
            let v3 = input - self.ic2eq_l;
            let v1 = a1 * self.ic1eq_l + a2 * v3;
            let v2 = self.ic2eq_l + a2 * self.ic1eq_l + a3 * v3;
            self.ic1eq_l = 2.0 * v1 - self.ic1eq_l;
            self.ic2eq_l = 2.0 * v2 - self.ic2eq_l;
            *sample = (input - amount * k * v1) as f32;
        }
    }
}
