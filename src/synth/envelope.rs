use synfx_dsp::{EnvRetrigADSR, EnvADSRParams};

pub struct Envelope {
    adsr: EnvRetrigADSR,
    params: EnvADSRParams,
    gate: f32,
    retrigger_countdown: u8,
}

impl Envelope {
    pub fn new(sample_rate: f32) -> Self {
        let mut adsr = EnvRetrigADSR::new();
        adsr.set_sample_rate(sample_rate);

        Self {
            adsr,
            params: EnvADSRParams::default(),
            gate: 0.0,
            retrigger_countdown: 0,
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

        if self.gate > 0.0 {
            self.retrigger_countdown = 2;
        } else {
            self.gate = 1.0;
        }
    }

    pub fn release(&mut self) {
        self.gate = 0.0;
        self.retrigger_countdown = 0;
    }

    pub fn next(&mut self) -> f32 {
        if self.retrigger_countdown > 0 {
            if self.retrigger_countdown == 2 {
                self.gate = 0.0;
            } else if self.retrigger_countdown == 1 {
                self.gate = 1.0;
            }
            self.retrigger_countdown -= 1;
        }

        let (env, _) = self.adsr.tick(self.gate, &mut self.params);
        env
    }
}
