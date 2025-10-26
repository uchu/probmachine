use synfx_dsp::{EnvRetrigADSR, EnvADSRParams};

pub struct Envelope {
    adsr: EnvRetrigADSR,
    params: EnvADSRParams,
    gate: f32,
    triggered: bool,
}

impl Envelope {
    pub fn new(sample_rate: f32) -> Self {
        let mut adsr = EnvRetrigADSR::new();
        adsr.set_sample_rate(sample_rate);

        Self {
            adsr,
            params: EnvADSRParams::default(),
            gate: 0.0,
            triggered: false,
        }
    }

    pub fn trigger(&mut self, attack_ms: f32, attack_shape: f32, decay_ms: f32, decay_shape: f32, sustain: f32, release_ms: f32, release_shape: f32) {
        self.params = EnvADSRParams {
            attack_ms,
            attack_shape,
            decay_ms,
            decay_shape,
            sustain,
            release_ms,
            release_shape,
        };
        self.gate = 1.0;
        self.triggered = true;
    }

    pub fn next(&mut self) -> f32 {
        let (env, _) = self.adsr.tick(self.gate, &mut self.params);
        if self.triggered {
            self.gate = 0.0;
            self.triggered = false;
        }
        env
    }
}
