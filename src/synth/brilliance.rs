use std::f64::consts::PI;

const CUTOFF_HZ: f64 = 4500.0;
const Q: f64 = 0.5;

pub struct BrillianceFilter {
    ic1eq_l: f64,
    ic2eq_l: f64,
    ic1eq_r: f64,
    ic2eq_r: f64,
    g: f64,
    k: f64,
    a1: f64,
    a2: f64,
    a3: f64,
    amount: f64,
    drive: f64,
    sample_rate: f64,
}

impl BrillianceFilter {
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
            amount: 0.0,
            drive: 0.0,
            sample_rate: sample_rate as f64,
        };
        f.update_coefficients();
        f
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate as f64;
        self.update_coefficients();
    }

    pub fn set_amount(&mut self, amount: f64) {
        self.amount = amount;
    }

    pub fn set_drive(&mut self, drive: f64) {
        self.drive = drive;
    }

    fn update_coefficients(&mut self) {
        self.g = (PI * CUTOFF_HZ / self.sample_rate).tan();
        self.k = 1.0 / Q;
        self.a1 = 1.0 / (1.0 + self.g * (self.g + self.k));
        self.a2 = self.g * self.a1;
        self.a3 = self.g * self.a2;
    }

    #[inline]
    pub fn process_block(&mut self, buffer_l: &mut [f32], buffer_r: &mut [f32]) {
        if self.amount < 0.001 {
            return;
        }

        let a1 = self.a1;
        let a2 = self.a2;
        let a3 = self.a3;
        let k = self.k;
        let amount = self.amount;
        let drive_gain = 1.0 + self.drive * 5.0;

        for (l, r) in buffer_l.iter_mut().zip(buffer_r.iter_mut()) {
            // Left: SVF highpass to extract highs
            let input_l = *l as f64;
            let v3_l = input_l - self.ic2eq_l;
            let v1_l = a1 * self.ic1eq_l + a2 * v3_l;
            let v2_l = self.ic2eq_l + a2 * self.ic1eq_l + a3 * v3_l;
            self.ic1eq_l = 2.0 * v1_l - self.ic1eq_l;
            self.ic2eq_l = 2.0 * v2_l - self.ic2eq_l;
            let hp_l = input_l - k * v1_l - v2_l;

            // Right: SVF highpass to extract highs
            let input_r = *r as f64;
            let v3_r = input_r - self.ic2eq_r;
            let v1_r = a1 * self.ic1eq_r + a2 * v3_r;
            let v2_r = self.ic2eq_r + a2 * self.ic1eq_r + a3 * v3_r;
            self.ic1eq_r = 2.0 * v1_r - self.ic1eq_r;
            self.ic2eq_r = 2.0 * v2_r - self.ic2eq_r;
            let hp_r = input_r - k * v1_r - v2_r;

            // Saturate highs (drive=0: clean shelf, drive>0: harmonic exciter)
            let excited_l = (hp_l * drive_gain).tanh();
            let excited_r = (hp_r * drive_gain).tanh();

            // Mix: high shelf = input + amount * processed_highs
            *l = (input_l + amount * excited_l) as f32;
            *r = (input_r + amount * excited_r) as f32;
        }
    }
}
