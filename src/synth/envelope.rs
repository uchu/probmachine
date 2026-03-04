#![allow(clippy::too_many_arguments)]

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EnvelopeStage {
    Idle,
    Attack,
    Decay,
    Sustain,
    Release,
}

pub struct Envelope {
    stage: EnvelopeStage,
    current_value: f64,
    stage_start_value: f64,
    stage_samples: f64,
    stage_counter: f64,
    held: bool,
    was_idle: bool,
    sample_rate: f64,

    attack_ms: f64,
    decay_ms: f64,
    sustain_level: f64,
    release_ms: f64,
    attack_shape: f64,
    decay_shape: f64,
    release_shape: f64,

    attack_ln_k: f64,
    decay_ln_k: f64,
    release_ln_k: f64,

    sustain_smoother: f64,
}

const MIN_STAGE_MS: f64 = 0.5;
const SUSTAIN_SMOOTH_MS: f64 = 3.0;

impl Envelope {
    pub fn new(sample_rate: f64) -> Self {
        Self {
            stage: EnvelopeStage::Idle,
            current_value: 0.0,
            stage_start_value: 0.0,
            stage_samples: 0.0,
            stage_counter: 0.0,
            held: false,
            was_idle: true,
            sample_rate,
            attack_ms: 10.0,
            decay_ms: 100.0,
            sustain_level: 0.7,
            release_ms: 200.0,
            attack_shape: 0.0,
            decay_shape: 0.0,
            release_shape: 0.0,
            attack_ln_k: 0.0,
            decay_ln_k: 0.0,
            release_ln_k: 0.0,
            sustain_smoother: 0.7,
        }
    }

    pub fn is_active(&self) -> bool {
        self.stage != EnvelopeStage::Idle
    }

    pub fn is_held(&self) -> bool {
        self.held
    }

    pub fn trigger(
        &mut self,
        attack_ms: f64, attack_shape: f64,
        decay_ms: f64, decay_shape: f64,
        sustain: f64,
        release_ms: f64, release_shape: f64,
    ) {
        self.was_idle = self.stage == EnvelopeStage::Idle;
        self.stage_start_value = self.current_value;
        self.held = true;

        self.cache_params(attack_ms, attack_shape, decay_ms, decay_shape, sustain, release_ms, release_shape);
        self.enter_attack();
    }

    pub fn trigger_with_dip(
        &mut self,
        attack_ms: f64, attack_shape: f64,
        decay_ms: f64, decay_shape: f64,
        sustain: f64,
        release_ms: f64, release_shape: f64,
        dip: f64,
    ) {
        self.was_idle = self.stage == EnvelopeStage::Idle;
        self.current_value *= 1.0 - dip.clamp(0.0, 1.0);
        self.stage_start_value = self.current_value;
        self.held = true;

        self.cache_params(attack_ms, attack_shape, decay_ms, decay_shape, sustain, release_ms, release_shape);
        self.enter_attack();
    }

    pub fn release(&mut self) {
        self.held = false;
        if self.stage == EnvelopeStage::Idle {
            return;
        }
        self.stage_start_value = self.current_value;
        self.stage = EnvelopeStage::Release;
        self.stage_samples = ms_to_samples(self.release_ms, self.sample_rate);
        self.stage_counter = 0.0;
    }

    pub fn force_off(&mut self) {
        self.stage = EnvelopeStage::Idle;
        self.current_value = 0.0;
        self.stage_start_value = 0.0;
        self.stage_counter = 0.0;
        self.stage_samples = 0.0;
        self.held = false;
        self.sustain_smoother = 0.0;
    }

    pub fn update_params(
        &mut self,
        attack_ms: f64, attack_shape: f64,
        decay_ms: f64, decay_shape: f64,
        sustain: f64,
        release_ms: f64, release_shape: f64,
    ) {
        let old_attack_ms = self.attack_ms;
        let old_decay_ms = self.decay_ms;
        let old_release_ms = self.release_ms;

        self.cache_params(attack_ms, attack_shape, decay_ms, decay_shape, sustain, release_ms, release_shape);

        match self.stage {
            EnvelopeStage::Attack => {
                let new_samples = ms_to_samples(self.attack_ms, self.sample_rate);
                if old_attack_ms > MIN_STAGE_MS {
                    let old_samples = ms_to_samples(old_attack_ms, self.sample_rate);
                    if old_samples > 0.0 {
                        self.stage_counter = (self.stage_counter / old_samples) * new_samples;
                    }
                }
                self.stage_samples = new_samples;
            }
            EnvelopeStage::Decay => {
                let new_samples = ms_to_samples(self.decay_ms, self.sample_rate);
                if old_decay_ms > MIN_STAGE_MS {
                    let old_samples = ms_to_samples(old_decay_ms, self.sample_rate);
                    if old_samples > 0.0 {
                        self.stage_counter = (self.stage_counter / old_samples) * new_samples;
                    }
                }
                self.stage_samples = new_samples;
            }
            EnvelopeStage::Release => {
                let new_samples = ms_to_samples(self.release_ms, self.sample_rate);
                if old_release_ms > MIN_STAGE_MS {
                    let old_samples = ms_to_samples(old_release_ms, self.sample_rate);
                    if old_samples > 0.0 {
                        self.stage_counter = (self.stage_counter / old_samples) * new_samples;
                    }
                }
                self.stage_samples = new_samples;
            }
            _ => {}
        }
    }

    #[inline]
    pub fn next(&mut self) -> f64 {
        match self.stage {
            EnvelopeStage::Idle => 0.0,
            EnvelopeStage::Attack => {
                self.stage_counter += 1.0;
                let phase = (self.stage_counter / self.stage_samples).min(1.0);
                let shaped = apply_curve(phase, self.attack_ln_k);
                self.current_value = self.stage_start_value + (1.0 - self.stage_start_value) * shaped;

                if self.stage_counter >= self.stage_samples {
                    self.current_value = 1.0;
                    self.stage_start_value = 1.0;
                    self.stage = EnvelopeStage::Decay;
                    self.stage_samples = ms_to_samples(self.decay_ms, self.sample_rate);
                    self.stage_counter = 0.0;
                }
                self.current_value
            }
            EnvelopeStage::Decay => {
                self.stage_counter += 1.0;
                let phase = (self.stage_counter / self.stage_samples).min(1.0);
                let shaped = apply_curve(phase, self.decay_ln_k);
                self.current_value = self.stage_start_value + (self.sustain_level - self.stage_start_value) * shaped;

                if self.stage_counter >= self.stage_samples {
                    self.current_value = self.sustain_level;
                    self.sustain_smoother = self.sustain_level;
                    self.stage = EnvelopeStage::Sustain;
                }
                self.current_value
            }
            EnvelopeStage::Sustain => {
                let coeff = (-1.0 / (SUSTAIN_SMOOTH_MS * 0.001 * self.sample_rate)).exp();
                self.sustain_smoother = self.sustain_level + coeff * (self.sustain_smoother - self.sustain_level);
                self.current_value = self.sustain_smoother;
                self.current_value
            }
            EnvelopeStage::Release => {
                self.stage_counter += 1.0;
                let phase = (self.stage_counter / self.stage_samples).min(1.0);
                let shaped = apply_curve(phase, self.release_ln_k);
                self.current_value = self.stage_start_value * (1.0 - shaped);

                if self.stage_counter >= self.stage_samples {
                    self.current_value = 0.0;
                    self.stage = EnvelopeStage::Idle;
                }
                self.current_value
            }
        }
    }

    #[allow(dead_code)]
    pub fn set_sample_rate(&mut self, sample_rate: f64) {
        self.sample_rate = sample_rate;
    }

    fn enter_attack(&mut self) {
        self.stage = EnvelopeStage::Attack;
        self.stage_samples = ms_to_samples(self.attack_ms, self.sample_rate);
        self.stage_counter = 0.0;
    }

    fn cache_params(
        &mut self,
        attack_ms: f64, attack_shape: f64,
        decay_ms: f64, decay_shape: f64,
        sustain: f64,
        release_ms: f64, release_shape: f64,
    ) {
        self.attack_ms = attack_ms.max(MIN_STAGE_MS);
        self.decay_ms = decay_ms.max(MIN_STAGE_MS);
        self.sustain_level = sustain.clamp(0.0, 1.0);
        self.release_ms = release_ms.max(MIN_STAGE_MS);
        self.attack_shape = attack_shape.clamp(-1.0, 1.0);
        self.decay_shape = decay_shape.clamp(-1.0, 1.0);
        self.release_shape = release_shape.clamp(-1.0, 1.0);
        self.attack_ln_k = compute_ln_k(self.attack_shape);
        self.decay_ln_k = compute_ln_k(self.decay_shape);
        self.release_ln_k = compute_ln_k(self.release_shape);
    }
}

fn ms_to_samples(ms: f64, sample_rate: f64) -> f64 {
    (ms * 0.001 * sample_rate).max(1.0)
}

fn compute_ln_k(shape: f64) -> f64 {
    if shape.abs() < 0.01 {
        return 0.0;
    }
    let k = 1.0 + shape.abs() * 9.0;
    let ln_k = k.ln();
    if shape > 0.0 { ln_k } else { -ln_k }
}

#[inline]
fn apply_curve(phase: f64, ln_k: f64) -> f64 {
    if ln_k.abs() < 0.001 {
        return phase;
    }
    if ln_k > 0.0 {
        (phase * ln_k).exp_m1() / ln_k.exp_m1()
    } else {
        let pos_k = -ln_k;
        1.0 - ((1.0 - phase) * pos_k).exp_m1() / pos_k.exp_m1()
    }
}

pub struct TailEnvelope {
    value: f64,
    decay_coeff: f64,
    active: bool,
}

impl TailEnvelope {
    pub fn new() -> Self {
        Self {
            value: 0.0,
            decay_coeff: 0.999,
            active: false,
        }
    }

    pub fn trigger_tail(&mut self, time_ms: f64, amount: f64, sample_rate: f64) {
        let samples = (time_ms * 0.001 * sample_rate).max(1.0);
        self.decay_coeff = 0.001_f64.powf(1.0 / samples);
        self.value = amount.clamp(0.0, 1.0);
        self.active = true;
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    #[inline]
    pub fn next(&mut self) -> f64 {
        if !self.active {
            return 0.0;
        }
        self.value *= self.decay_coeff;
        if self.value < 0.0001 {
            self.value = 0.0;
            self.active = false;
        }
        self.value
    }

    #[allow(dead_code)]
    pub fn reset(&mut self) {
        self.value = 0.0;
        self.active = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SR: f64 = 44100.0;

    fn make_env() -> Envelope {
        Envelope::new(SR)
    }

    fn run_samples(env: &mut Envelope, n: usize) -> Vec<f64> {
        (0..n).map(|_| env.next()).collect()
    }

    #[test]
    fn fresh_trigger_from_idle() {
        let mut env = make_env();
        assert_eq!(env.stage, EnvelopeStage::Idle);

        env.trigger(10.0, 0.0, 100.0, 0.0, 0.7, 200.0, 0.0);
        assert_eq!(env.stage, EnvelopeStage::Attack);
        assert!(env.was_idle);

        let samples = run_samples(&mut env, 44100);
        let last = *samples.last().unwrap();
        assert!(last > 0.0, "envelope should produce output");
    }

    #[test]
    fn retrigger_during_attack_is_continuous() {
        let mut env = make_env();
        env.trigger(50.0, 0.0, 100.0, 0.0, 0.7, 200.0, 0.0);

        run_samples(&mut env, 500);
        let val_before = env.current_value;
        assert!(val_before > 0.0 && val_before < 1.0);

        env.trigger(50.0, 0.0, 100.0, 0.0, 0.7, 200.0, 0.0);
        let val_after = env.next();

        assert!((val_after - val_before).abs() < 0.05, "retrigger must be continuous: before={val_before}, after={val_after}");
        assert!(!env.was_idle);
    }

    #[test]
    fn retrigger_during_decay_is_continuous() {
        let mut env = make_env();
        env.trigger(5.0, 0.0, 500.0, 0.0, 0.5, 200.0, 0.0);
        run_samples(&mut env, 1000);

        assert_eq!(env.stage, EnvelopeStage::Decay);
        let val_before = env.current_value;

        env.trigger(5.0, 0.0, 500.0, 0.0, 0.5, 200.0, 0.0);
        let val_after = env.next();
        assert!((val_after - val_before).abs() < 0.05);
    }

    #[test]
    fn retrigger_during_release_is_continuous() {
        let mut env = make_env();
        env.trigger(5.0, 0.0, 50.0, 0.0, 0.7, 500.0, 0.0);
        run_samples(&mut env, 5000);
        env.release();
        run_samples(&mut env, 2000);

        assert_eq!(env.stage, EnvelopeStage::Release);
        let val_before = env.current_value;
        assert!(val_before > 0.0);

        env.trigger(5.0, 0.0, 50.0, 0.0, 0.7, 500.0, 0.0);
        let val_after = env.next();
        assert!((val_after - val_before).abs() < 0.05);
    }

    #[test]
    fn release_during_attack() {
        let mut env = make_env();
        env.trigger(50.0, 0.0, 100.0, 0.0, 0.7, 200.0, 0.0);
        run_samples(&mut env, 500);

        let val_before = env.current_value;
        env.release();
        assert_eq!(env.stage, EnvelopeStage::Release);

        let val_after = env.next();
        assert!((val_after - val_before).abs() < 0.05);

        let samples = run_samples(&mut env, 44100);
        assert!(*samples.last().unwrap() < 0.001);
    }

    #[test]
    fn sustain_level_change_during_sustain() {
        let mut env = make_env();
        env.trigger(1.0, 0.0, 1.0, 0.0, 0.7, 200.0, 0.0);
        run_samples(&mut env, 500);
        assert_eq!(env.stage, EnvelopeStage::Sustain);

        env.update_params(1.0, 0.0, 1.0, 0.0, 0.3, 200.0, 0.0);

        let samples = run_samples(&mut env, 500);
        let last = *samples.last().unwrap();
        assert!((last - 0.3).abs() < 0.05, "sustain should smoothly follow: {last}");
    }

    #[test]
    fn time_change_during_active_stage() {
        let mut env = make_env();
        env.trigger(100.0, 0.0, 100.0, 0.0, 0.7, 200.0, 0.0);
        run_samples(&mut env, 1000);
        assert_eq!(env.stage, EnvelopeStage::Attack);

        let val_before = env.current_value;
        env.update_params(200.0, 0.0, 100.0, 0.0, 0.7, 200.0, 0.0);
        let val_after = env.next();

        assert!((val_after - val_before).abs() < 0.05, "time change must not cause discontinuity");
    }

    #[test]
    fn curve_shapes() {
        let mut linear = make_env();
        linear.trigger(100.0, 0.0, 100.0, 0.0, 0.5, 100.0, 0.0);
        let lin_samples = run_samples(&mut linear, 2205);

        let mut log_env = make_env();
        log_env.trigger(100.0, -1.0, 100.0, 0.0, 0.5, 100.0, 0.0);
        let log_samples = run_samples(&mut log_env, 2205);

        let mut exp_env = make_env();
        exp_env.trigger(100.0, 1.0, 100.0, 0.0, 0.5, 100.0, 0.0);
        let exp_samples = run_samples(&mut exp_env, 2205);

        let mid = 1000;
        assert!(log_samples[mid] > lin_samples[mid], "log curve should be above linear at midpoint");
        assert!(exp_samples[mid] < lin_samples[mid], "exp curve should be below linear at midpoint");
    }

    #[test]
    fn trigger_with_dip_behavior() {
        let mut env = make_env();
        env.trigger(5.0, 0.0, 50.0, 0.0, 0.7, 200.0, 0.0);
        run_samples(&mut env, 5000);

        let val_before = env.current_value;
        assert!((val_before - 0.7).abs() < 0.05);

        env.trigger_with_dip(5.0, 0.0, 50.0, 0.0, 0.7, 200.0, 0.0, 0.5);
        let val_after = env.current_value;
        assert!((val_after - val_before * 0.5).abs() < 0.05, "dip should halve: before={val_before}, after={val_after}");
    }

    #[test]
    fn tail_envelope_decay() {
        let mut tail = TailEnvelope::new();
        tail.trigger_tail(500.0, 1.0, SR);
        assert!(tail.is_active());

        let mut prev = tail.value;
        for _ in 0..22050 {
            let v = tail.next();
            assert!(v <= prev);
            prev = v;
        }

        for _ in 0..44100 {
            tail.next();
        }
        assert!(!tail.is_active(), "tail should decay to silence");
    }

    #[test]
    fn zero_attack_clamps_to_minimum() {
        let mut env = make_env();
        env.trigger(0.0, 0.0, 100.0, 0.0, 0.7, 200.0, 0.0);
        assert_eq!(env.stage, EnvelopeStage::Attack);

        let samples = run_samples(&mut env, 100);
        assert!(samples.iter().any(|&v| v > 0.9), "minimum attack should still reach peak quickly");
    }

    #[test]
    fn full_adsr_cycle_reaches_zero() {
        let mut env = make_env();
        env.trigger(10.0, 0.0, 50.0, 0.0, 0.7, 100.0, 0.0);
        run_samples(&mut env, 10000);
        env.release();
        let samples = run_samples(&mut env, 44100);
        assert_eq!(env.stage, EnvelopeStage::Idle);
        assert!(*samples.last().unwrap() < 0.0001);
    }
}
