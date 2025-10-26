use synfx_dsp::process_stilson_moog;

pub struct MoogFilter {
    sample_rate: f32,
    israte: f32,
    b0: f32,
    b1: f32,
    b2: f32,
    b3: f32,
    delay: [f32; 4],
}

impl MoogFilter {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            sample_rate,
            israte: 1.0 / sample_rate,
            b0: 0.0,
            b1: 0.0,
            b2: 0.0,
            b3: 0.0,
            delay: [0.0; 4],
        }
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        self.israte = 1.0 / sample_rate;
    }

    pub fn process(&mut self, input: f32, cutoff: f32, resonance: f32) -> f32 {
        let res = resonance.clamp(0.0, 0.99);
        process_stilson_moog(
            input,
            cutoff,
            res,
            self.israte,
            &mut self.b0,
            &mut self.b1,
            &mut self.b2,
            &mut self.b3,
            &mut self.delay,
        )
    }

    pub fn reset(&mut self) {
        self.b0 = 0.0;
        self.b1 = 0.0;
        self.b2 = 0.0;
        self.b3 = 0.0;
        self.delay = [0.0; 4];
    }
}
