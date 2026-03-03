#![allow(clippy::too_many_arguments)]

use synfx_dsp::{EnvRetrigADSR, EnvADSRParams};

pub struct Envelope {
    adsr: EnvRetrigADSR,
    params: EnvADSRParams,
    gate: f32,
    retrigger_countdown: u8,
    last_value: f32,
    gate_held: bool,
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
            last_value: 0.0,
            gate_held: false,
        }
    }

    pub fn is_active(&self) -> bool {
        self.gate > 0.0 || self.retrigger_countdown > 0 || self.last_value > 0.001
    }

    pub fn is_held(&self) -> bool {
        self.gate_held
    }

    pub fn trigger(
        &mut self,
        attack_ms: f64, attack_shape: f64,
        decay_ms: f64, decay_shape: f64,
        sustain: f64,
        release_ms: f64, release_shape: f64,
    ) {
        let is_retrigger = self.gate > 0.0;
        let min_attack = if is_retrigger { 2.0 } else { 1.0 };

        self.update_params_internal(attack_ms, attack_shape, decay_ms, decay_shape, sustain, release_ms, release_shape, min_attack);
        self.gate_held = true;

        if is_retrigger {
            self.retrigger_countdown = 2;
        } else {
            self.gate = 1.0;
        }
    }

    pub fn release(&mut self) {
        self.gate = 0.0;
        self.retrigger_countdown = 0;
        self.gate_held = false;
    }

    pub fn force_off(&mut self) {
        self.gate = 0.0;
        self.retrigger_countdown = 0;
        self.last_value = 0.0;
        self.gate_held = false;
        self.adsr.reset();
    }

    pub fn restart(
        &mut self,
        attack_ms: f64, attack_shape: f64,
        decay_ms: f64, decay_shape: f64,
        sustain: f64,
        release_ms: f64, release_shape: f64,
    ) {
        self.update_params_internal(attack_ms, attack_shape, decay_ms, decay_shape, sustain, release_ms, release_shape, 2.0);
        self.gate_held = true;
        if self.gate > 0.0 {
            self.retrigger_countdown = 2;
        } else {
            self.gate = 1.0;
        }
    }

    pub fn update_params(
        &mut self,
        attack_ms: f64, attack_shape: f64,
        decay_ms: f64, decay_shape: f64,
        sustain: f64,
        release_ms: f64, release_shape: f64,
    ) {
        let min_attack = if self.retrigger_countdown > 0 { 2.0 } else { 1.0 };
        self.update_params_internal(attack_ms, attack_shape, decay_ms, decay_shape, sustain, release_ms, release_shape, min_attack);
    }

    #[allow(dead_code)]
    pub fn set_sample_rate(&mut self, sample_rate: f64) {
        self.adsr.set_sample_rate(sample_rate as f32);
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
        self.last_value = env;
        env as f64
    }

    fn update_params_internal(
        &mut self,
        attack_ms: f64, attack_shape: f64,
        decay_ms: f64, decay_shape: f64,
        sustain: f64,
        release_ms: f64, release_shape: f64,
        min_attack: f64,
    ) {
        self.params = EnvADSRParams {
            attack_ms: attack_ms.max(min_attack) as f32,
            attack_shape: attack_shape as f32,
            decay_ms: decay_ms.max(1.0) as f32,
            decay_shape: decay_shape as f32,
            sustain: sustain as f32,
            release_ms: release_ms.max(1.0) as f32,
            release_shape: release_shape as f32,
        };
    }
}
