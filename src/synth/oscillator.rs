use synfx_dsp::{VPSOscillator, rand_01};

pub struct Oscillator {
    osc: VPSOscillator,
    sample_rate: f32,
    freq: f32,
    d: f32,
    v: f32,
}

impl Oscillator {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            osc: VPSOscillator::new(rand_01() * 0.25),
            sample_rate,
            freq: 220.0,
            d: 0.5,
            v: 0.5,
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
        self.osc.next(self.freq, israte, d, v_limited)
    }
}
