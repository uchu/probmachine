#![allow(clippy::too_many_arguments)]

use synfx_dsp::{EnvRetrigADSR, EnvADSRParams};

pub struct Envelope {
    adsr: EnvRetrigADSR,
    params: EnvADSRParams,
    gate: f32,
    retrigger_countdown: u8,
}

impl Envelope {
    pub fn new(sample_rate: f64) -> Self {
        let mut adsr = EnvRetrigADSR::new();
        adsr.set_sample_rate(sample_rate as f32);

        Self {
            adsr,
            params: EnvADSRParams::default(),
            gate: 0.0,
            retrigger_countdown: 0,
        }
    }

    pub fn trigger(&mut self, attack_ms: f64, attack_shape: f64, decay_ms: f64, decay_shape: f64, sustain: f64, release_ms: f64, release_shape: f64) {
        // Enforce minimum 1ms for attack and release to avoid clicks
        self.params = EnvADSRParams {
            attack_ms: attack_ms.max(1.0) as f32,
            attack_shape: attack_shape as f32,
            decay_ms: decay_ms.max(1.0) as f32,
            decay_shape: decay_shape as f32,
            sustain: sustain as f32,
            release_ms: release_ms.max(1.0) as f32,
            release_shape: release_shape as f32,
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

    pub fn next(&mut self) -> f64 {
        if self.retrigger_countdown > 0 {
            if self.retrigger_countdown == 2 {
                self.gate = 0.0;
            } else if self.retrigger_countdown == 1 {
                self.gate = 1.0;
            }
            self.retrigger_countdown -= 1;
        }

        let (env, _) = self.adsr.tick(self.gate, &mut self.params);
        env as f64
    }
}
