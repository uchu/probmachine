use std::f64::consts::PI;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum HpfMode {
    Off,
    Hz35,
    Hz80,
    Hz120,
    Hz220,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum HpfBoost {
    None,
    Medium,
    High,
}

impl HpfBoost {
    pub fn from_index(idx: i32) -> Self {
        match idx {
            1 => HpfBoost::Medium,
            2 => HpfBoost::High,
            _ => HpfBoost::None,
        }
    }

    fn q(&self) -> f64 {
        match self {
            HpfBoost::None => std::f64::consts::FRAC_1_SQRT_2,
            HpfBoost::Medium => 2.0,    // Same as SAW tight filter at full amount
            HpfBoost::High => 4.0,      // Aggressive resonant peak
        }
    }
}

impl HpfMode {
    pub fn from_index(idx: i32) -> Self {
        match idx {
            1 => HpfMode::Hz35,
            2 => HpfMode::Hz80,
            3 => HpfMode::Hz120,
            4 => HpfMode::Hz220,
            _ => HpfMode::Off,
        }
    }

    pub fn cutoff_hz(&self) -> Option<f64> {
        match self {
            HpfMode::Off => None,
            HpfMode::Hz35 => Some(35.0),
            HpfMode::Hz80 => Some(80.0),
            HpfMode::Hz120 => Some(120.0),
            HpfMode::Hz220 => Some(220.0),
        }
    }
}

pub struct MasterHpf {
    ic1eq_l: f64,
    ic2eq_l: f64,
    ic1eq_r: f64,
    ic2eq_r: f64,
    g: f64,
    k: f64,
    a1: f64,
    a2: f64,
    a3: f64,
    mode: HpfMode,
    boost: HpfBoost,
    sample_rate: f64,
}

impl MasterHpf {
    pub fn new(sample_rate: f32) -> Self {
        let mut hpf = Self {
            ic1eq_l: 0.0,
            ic2eq_l: 0.0,
            ic1eq_r: 0.0,
            ic2eq_r: 0.0,
            g: 0.0,
            k: 0.0,
            a1: 0.0,
            a2: 0.0,
            a3: 0.0,
            mode: HpfMode::Off,
            boost: HpfBoost::None,
            sample_rate: sample_rate as f64,
        };
        hpf.update_coefficients();
        hpf
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate as f64;
        self.update_coefficients();
    }

    pub fn set_mode(&mut self, mode: HpfMode) {
        if mode != self.mode {
            self.mode = mode;
            self.update_coefficients();
            if mode == HpfMode::Off {
                self.ic1eq_l = 0.0;
                self.ic2eq_l = 0.0;
                self.ic1eq_r = 0.0;
                self.ic2eq_r = 0.0;
            }
        }
    }

    pub fn set_boost(&mut self, boost: HpfBoost) {
        if boost != self.boost {
            self.boost = boost;
            self.update_coefficients();
        }
    }

    fn update_coefficients(&mut self) {
        if let Some(freq) = self.mode.cutoff_hz() {
            self.g = (PI * freq / self.sample_rate).tan();
            self.k = 1.0 / self.boost.q();
            self.a1 = 1.0 / (1.0 + self.g * (self.g + self.k));
            self.a2 = self.g * self.a1;
            self.a3 = self.g * self.a2;
        }
    }

    #[inline]
    pub fn process_block(&mut self, buffer_l: &mut [f32], buffer_r: &mut [f32]) {
        if self.mode == HpfMode::Off {
            return;
        }

        let a1 = self.a1;
        let a2 = self.a2;
        let a3 = self.a3;
        let k = self.k;

        for (l, r) in buffer_l.iter_mut().zip(buffer_r.iter_mut()) {
            // Left channel SVF highpass
            let input_l = *l as f64;
            let v3_l = input_l - self.ic2eq_l;
            let v1_l = a1 * self.ic1eq_l + a2 * v3_l;
            let v2_l = self.ic2eq_l + a2 * self.ic1eq_l + a3 * v3_l;
            self.ic1eq_l = 2.0 * v1_l - self.ic1eq_l;
            self.ic2eq_l = 2.0 * v2_l - self.ic2eq_l;
            *l = (input_l - k * v1_l - v2_l) as f32;

            // Right channel SVF highpass
            let input_r = *r as f64;
            let v3_r = input_r - self.ic2eq_r;
            let v1_r = a1 * self.ic1eq_r + a2 * v3_r;
            let v2_r = self.ic2eq_r + a2 * self.ic1eq_r + a3 * v3_r;
            self.ic1eq_r = 2.0 * v1_r - self.ic1eq_r;
            self.ic2eq_r = 2.0 * v2_r - self.ic2eq_r;
            *r = (input_r - k * v1_r - v2_r) as f32;
        }
    }

    #[inline]
    pub fn process_mono(&mut self, buffer: &mut [f32]) {
        if self.mode == HpfMode::Off {
            return;
        }

        let a1 = self.a1;
        let a2 = self.a2;
        let a3 = self.a3;
        let k = self.k;

        for sample in buffer.iter_mut() {
            let input = *sample as f64;
            let v3 = input - self.ic2eq_l;
            let v1 = a1 * self.ic1eq_l + a2 * v3;
            let v2 = self.ic2eq_l + a2 * self.ic1eq_l + a3 * v3;
            self.ic1eq_l = 2.0 * v1 - self.ic1eq_l;
            self.ic2eq_l = 2.0 * v2 - self.ic2eq_l;
            *sample = (input - k * v1 - v2) as f32;
        }
    }
}
