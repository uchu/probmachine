#![allow(clippy::too_many_arguments)]

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EnvelopeStage {
    Idle,
    Dip,
    Attack,
    Hold,
    Decay,
    Sustain,
    Release,
    QuickFade,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EnvelopeLoopMode {
    OneShot,
    LoopAHD,
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
    hold_ms: f64,
    decay_ms: f64,
    sustain_level: f64,
    release_ms: f64,
    attack_shape: f64,
    decay_shape: f64,
    release_shape: f64,

    attack_ln_k: f64,
    decay_ln_k: f64,
    release_ln_k: f64,

    attack_s_curve: bool,
    decay_s_curve: bool,
    release_s_curve: bool,

    loop_mode: EnvelopeLoopMode,

    sustain_smoother: f64,
    dip_target: f64,
}

const MIN_STAGE_MS: f64 = 0.1;
const MIN_RELEASE_MS: f64 = 3.0;
const SUSTAIN_SMOOTH_MS: f64 = 3.0;
const DIP_MS: f64 = 2.0;
const QUICK_FADE_MS: f64 = 2.0;

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
            hold_ms: 0.0,
            decay_ms: 100.0,
            sustain_level: 0.7,
            release_ms: 200.0,
            attack_shape: 0.0,
            decay_shape: 0.0,
            release_shape: 0.0,
            attack_ln_k: 0.0,
            decay_ln_k: 0.0,
            release_ln_k: 0.0,
            attack_s_curve: false,
            decay_s_curve: false,
            release_s_curve: false,
            loop_mode: EnvelopeLoopMode::OneShot,
            sustain_smoother: 0.7,
            dip_target: 0.0,
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
        hold_ms: f64,
        decay_ms: f64, decay_shape: f64,
        sustain: f64,
        release_ms: f64, release_shape: f64,
        loop_mode: EnvelopeLoopMode,
        attack_s: bool, decay_s: bool, release_s: bool,
    ) {
        self.was_idle = self.stage == EnvelopeStage::Idle;
        self.stage_start_value = self.current_value;
        self.held = true;

        self.cache_params(attack_ms, attack_shape, hold_ms, decay_ms, decay_shape, sustain, release_ms, release_shape, loop_mode, attack_s, decay_s, release_s);
        self.enter_attack();
    }

    pub fn trigger_with_dip(
        &mut self,
        attack_ms: f64, attack_shape: f64,
        hold_ms: f64,
        decay_ms: f64, decay_shape: f64,
        sustain: f64,
        release_ms: f64, release_shape: f64,
        loop_mode: EnvelopeLoopMode,
        dip: f64,
        attack_s: bool, decay_s: bool, release_s: bool,
    ) {
        self.was_idle = self.stage == EnvelopeStage::Idle;
        self.held = true;
        self.cache_params(attack_ms, attack_shape, hold_ms, decay_ms, decay_shape, sustain, release_ms, release_shape, loop_mode, attack_s, decay_s, release_s);

        let dip_clamped = dip.clamp(0.0, 1.0);
        if dip_clamped < 0.001 {
            self.stage_start_value = self.current_value;
            self.enter_attack();
        } else {
            self.stage_start_value = self.current_value;
            self.dip_target = self.current_value * (1.0 - dip_clamped);
            self.stage = EnvelopeStage::Dip;
            self.stage_samples = ms_to_samples(DIP_MS, self.sample_rate);
            self.stage_counter = 0.0;
        }
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
        if self.current_value > 0.0001 {
            self.stage_start_value = self.current_value;
            self.stage = EnvelopeStage::QuickFade;
            self.stage_samples = ms_to_samples(QUICK_FADE_MS, self.sample_rate);
            self.stage_counter = 0.0;
            self.held = false;
        } else {
            self.stage = EnvelopeStage::Idle;
            self.current_value = 0.0;
            self.stage_start_value = 0.0;
            self.stage_counter = 0.0;
            self.stage_samples = 0.0;
            self.held = false;
            self.sustain_smoother = 0.0;
            self.dip_target = 0.0;
        }
    }

    pub fn update_params(
        &mut self,
        attack_ms: f64, attack_shape: f64,
        hold_ms: f64,
        decay_ms: f64, decay_shape: f64,
        sustain: f64,
        release_ms: f64, release_shape: f64,
        loop_mode: EnvelopeLoopMode,
        attack_s: bool, decay_s: bool, release_s: bool,
    ) {
        let old_attack_ms = self.attack_ms;
        let old_hold_ms = self.hold_ms;
        let old_decay_ms = self.decay_ms;
        let old_release_ms = self.release_ms;

        self.cache_params(attack_ms, attack_shape, hold_ms, decay_ms, decay_shape, sustain, release_ms, release_shape, loop_mode, attack_s, decay_s, release_s);

        match self.stage {
            EnvelopeStage::Attack => {
                rescale_stage(self, old_attack_ms, self.attack_ms);
            }
            EnvelopeStage::Hold => {
                rescale_stage(self, old_hold_ms, self.hold_ms);
            }
            EnvelopeStage::Decay => {
                rescale_stage(self, old_decay_ms, self.decay_ms);
            }
            EnvelopeStage::Release => {
                rescale_stage(self, old_release_ms, self.release_ms);
            }
            _ => {}
        }
    }

    #[inline]
    pub fn next(&mut self) -> f64 {
        match self.stage {
            EnvelopeStage::Idle => 0.0,
            EnvelopeStage::Dip => {
                self.stage_counter += 1.0;
                let phase = (self.stage_counter / self.stage_samples).min(1.0);
                self.current_value = self.stage_start_value + (self.dip_target - self.stage_start_value) * phase;

                if self.stage_counter >= self.stage_samples {
                    self.current_value = self.dip_target;
                    self.stage_start_value = self.dip_target;
                    self.enter_attack();
                }
                self.current_value
            }
            EnvelopeStage::Attack => {
                self.stage_counter += 1.0;
                let phase = (self.stage_counter / self.stage_samples).min(1.0);
                let shaped = shape_phase(phase, self.attack_ln_k, self.attack_s_curve);
                self.current_value = self.stage_start_value + (1.0 - self.stage_start_value) * shaped;

                if self.stage_counter >= self.stage_samples {
                    self.current_value = 1.0;
                    self.stage_start_value = 1.0;
                    self.enter_hold();
                }
                self.current_value
            }
            EnvelopeStage::Hold => {
                self.stage_counter += 1.0;
                if self.stage_counter >= self.stage_samples {
                    self.enter_decay();
                }
                self.current_value
            }
            EnvelopeStage::Decay => {
                self.stage_counter += 1.0;
                let phase = (self.stage_counter / self.stage_samples).min(1.0);
                let shaped = shape_phase(phase, self.decay_ln_k, self.decay_s_curve);
                self.current_value = self.stage_start_value + (self.sustain_level - self.stage_start_value) * shaped;

                if self.stage_counter >= self.stage_samples {
                    if self.loop_mode == EnvelopeLoopMode::LoopAHD && self.held {
                        self.stage_start_value = self.current_value;
                        self.enter_attack();
                    } else {
                        self.current_value = self.sustain_level;
                        self.sustain_smoother = self.sustain_level;
                        self.stage = EnvelopeStage::Sustain;
                    }
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
                let shaped = shape_phase(phase, self.release_ln_k, self.release_s_curve);
                self.current_value = self.stage_start_value * (1.0 - shaped);

                if self.stage_counter >= self.stage_samples {
                    self.current_value = 0.0;
                    self.stage = EnvelopeStage::Idle;
                }
                self.current_value
            }
            EnvelopeStage::QuickFade => {
                self.stage_counter += 1.0;
                let phase = (self.stage_counter / self.stage_samples).min(1.0);
                self.current_value = self.stage_start_value * (1.0 - phase);

                if self.stage_counter >= self.stage_samples {
                    self.current_value = 0.0;
                    self.stage = EnvelopeStage::Idle;
                    self.sustain_smoother = 0.0;
                    self.dip_target = 0.0;
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

    fn enter_hold(&mut self) {
        if self.hold_ms < MIN_STAGE_MS {
            self.enter_decay();
        } else {
            self.stage = EnvelopeStage::Hold;
            self.stage_samples = ms_to_samples(self.hold_ms, self.sample_rate);
            self.stage_counter = 0.0;
        }
    }

    fn enter_decay(&mut self) {
        self.stage_start_value = self.current_value;
        self.stage = EnvelopeStage::Decay;
        self.stage_samples = ms_to_samples(self.decay_ms, self.sample_rate);
        self.stage_counter = 0.0;
    }

    fn cache_params(
        &mut self,
        attack_ms: f64, attack_shape: f64,
        hold_ms: f64,
        decay_ms: f64, decay_shape: f64,
        sustain: f64,
        release_ms: f64, release_shape: f64,
        loop_mode: EnvelopeLoopMode,
        attack_s: bool, decay_s: bool, release_s: bool,
    ) {
        self.attack_ms = attack_ms.max(MIN_STAGE_MS);
        self.hold_ms = hold_ms.max(0.0);
        self.decay_ms = decay_ms.max(MIN_STAGE_MS);
        self.sustain_level = sustain.clamp(0.0, 1.0);
        self.release_ms = release_ms.max(MIN_RELEASE_MS);
        self.attack_shape = attack_shape.clamp(-1.0, 1.0);
        self.decay_shape = decay_shape.clamp(-1.0, 1.0);
        self.release_shape = release_shape.clamp(-1.0, 1.0);
        self.attack_s_curve = attack_s;
        self.decay_s_curve = decay_s;
        self.release_s_curve = release_s;
        self.attack_ln_k = if attack_s {
            compute_ln_k(self.attack_shape.max(0.0) * 2.0)
        } else {
            compute_ln_k(self.attack_shape)
        };
        self.decay_ln_k = if decay_s {
            compute_ln_k(self.decay_shape.max(0.0) * 2.0)
        } else {
            compute_ln_k(self.decay_shape)
        };
        self.release_ln_k = if release_s {
            compute_ln_k(self.release_shape.max(0.0) * 2.0)
        } else {
            compute_ln_k(self.release_shape)
        };
        self.loop_mode = loop_mode;
    }
}

fn rescale_stage(env: &mut Envelope, old_ms: f64, new_ms: f64) {
    let new_samples = ms_to_samples(new_ms, env.sample_rate);
    if old_ms > MIN_STAGE_MS {
        let old_samples = ms_to_samples(old_ms, env.sample_rate);
        if old_samples > 0.0 {
            env.stage_counter = (env.stage_counter / old_samples) * new_samples;
        }
    }
    env.stage_samples = new_samples;
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

#[inline]
fn apply_s_curve(phase: f64, ln_k: f64) -> f64 {
    if ln_k.abs() < 0.001 {
        return phase;
    }
    let k = ln_k.abs();
    let denom = k.exp_m1();
    if ln_k > 0.0 {
        if phase <= 0.5 {
            let t = phase * 2.0;
            0.5 * (t * k).exp_m1() / denom
        } else {
            let t = (1.0 - phase) * 2.0;
            1.0 - 0.5 * (t * k).exp_m1() / denom
        }
    } else {
        if phase <= 0.5 {
            let t = phase * 2.0;
            0.5 * (1.0 - ((1.0 - t) * k).exp_m1() / denom)
        } else {
            let t = (1.0 - phase) * 2.0;
            1.0 - 0.5 * (1.0 - ((1.0 - t) * k).exp_m1() / denom)
        }
    }
}

#[inline]
fn shape_phase(phase: f64, ln_k: f64, s_curve: bool) -> f64 {
    if s_curve {
        apply_s_curve(phase, ln_k)
    } else {
        apply_curve(phase, ln_k)
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
    const OS: EnvelopeLoopMode = EnvelopeLoopMode::OneShot;

    fn make_env() -> Envelope {
        Envelope::new(SR)
    }

    fn run_samples(env: &mut Envelope, n: usize) -> Vec<f64> {
        (0..n).map(|_| env.next()).collect()
    }

    fn trig(env: &mut Envelope, a: f64, as_: f64, h: f64, d: f64, ds: f64, s: f64, r: f64, rs: f64) {
        env.trigger(a, as_, h, d, ds, s, r, rs, OS, false, false, false);
    }

    fn upd(env: &mut Envelope, a: f64, as_: f64, h: f64, d: f64, ds: f64, s: f64, r: f64, rs: f64) {
        env.update_params(a, as_, h, d, ds, s, r, rs, OS, false, false, false);
    }

    #[test]
    fn fresh_trigger_from_idle() {
        let mut env = make_env();
        assert_eq!(env.stage, EnvelopeStage::Idle);

        trig(&mut env, 10.0, 0.0, 0.0, 100.0, 0.0, 0.7, 200.0, 0.0);
        assert_eq!(env.stage, EnvelopeStage::Attack);
        assert!(env.was_idle);

        let samples = run_samples(&mut env, 44100);
        let last = *samples.last().unwrap();
        assert!(last > 0.0, "envelope should produce output");
    }

    #[test]
    fn retrigger_during_attack_is_continuous() {
        let mut env = make_env();
        trig(&mut env, 50.0, 0.0, 0.0, 100.0, 0.0, 0.7, 200.0, 0.0);

        run_samples(&mut env, 500);
        let val_before = env.current_value;
        assert!(val_before > 0.0 && val_before < 1.0);

        trig(&mut env, 50.0, 0.0, 0.0, 100.0, 0.0, 0.7, 200.0, 0.0);
        let val_after = env.next();

        assert!((val_after - val_before).abs() < 0.05, "retrigger must be continuous: before={val_before}, after={val_after}");
        assert!(!env.was_idle);
    }

    #[test]
    fn retrigger_during_decay_is_continuous() {
        let mut env = make_env();
        trig(&mut env, 5.0, 0.0, 0.0, 500.0, 0.0, 0.5, 200.0, 0.0);
        run_samples(&mut env, 1000);

        assert_eq!(env.stage, EnvelopeStage::Decay);
        let val_before = env.current_value;

        trig(&mut env, 5.0, 0.0, 0.0, 500.0, 0.0, 0.5, 200.0, 0.0);
        let val_after = env.next();
        assert!((val_after - val_before).abs() < 0.05);
    }

    #[test]
    fn retrigger_during_release_is_continuous() {
        let mut env = make_env();
        trig(&mut env, 5.0, 0.0, 0.0, 50.0, 0.0, 0.7, 500.0, 0.0);
        run_samples(&mut env, 5000);
        env.release();
        run_samples(&mut env, 2000);

        assert_eq!(env.stage, EnvelopeStage::Release);
        let val_before = env.current_value;
        assert!(val_before > 0.0);

        trig(&mut env, 5.0, 0.0, 0.0, 50.0, 0.0, 0.7, 500.0, 0.0);
        let val_after = env.next();
        assert!((val_after - val_before).abs() < 0.05);
    }

    #[test]
    fn release_during_attack() {
        let mut env = make_env();
        trig(&mut env, 50.0, 0.0, 0.0, 100.0, 0.0, 0.7, 200.0, 0.0);
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
        trig(&mut env, 1.0, 0.0, 0.0, 1.0, 0.0, 0.7, 200.0, 0.0);
        run_samples(&mut env, 500);
        assert_eq!(env.stage, EnvelopeStage::Sustain);

        upd(&mut env, 1.0, 0.0, 0.0, 1.0, 0.0, 0.3, 200.0, 0.0);

        let samples = run_samples(&mut env, 500);
        let last = *samples.last().unwrap();
        assert!((last - 0.3).abs() < 0.05, "sustain should smoothly follow: {last}");
    }

    #[test]
    fn time_change_during_active_stage() {
        let mut env = make_env();
        trig(&mut env, 100.0, 0.0, 0.0, 100.0, 0.0, 0.7, 200.0, 0.0);
        run_samples(&mut env, 1000);
        assert_eq!(env.stage, EnvelopeStage::Attack);

        let val_before = env.current_value;
        upd(&mut env, 200.0, 0.0, 0.0, 100.0, 0.0, 0.7, 200.0, 0.0);
        let val_after = env.next();

        assert!((val_after - val_before).abs() < 0.05, "time change must not cause discontinuity");
    }

    #[test]
    fn curve_shapes() {
        let mut linear = make_env();
        trig(&mut linear, 100.0, 0.0, 0.0, 100.0, 0.0, 0.5, 100.0, 0.0);
        let lin_samples = run_samples(&mut linear, 2205);

        let mut log_env = make_env();
        trig(&mut log_env, 100.0, -1.0, 0.0, 100.0, 0.0, 0.5, 100.0, 0.0);
        let log_samples = run_samples(&mut log_env, 2205);

        let mut exp_env = make_env();
        trig(&mut exp_env, 100.0, 1.0, 0.0, 100.0, 0.0, 0.5, 100.0, 0.0);
        let exp_samples = run_samples(&mut exp_env, 2205);

        let mid = 1000;
        assert!(log_samples[mid] > lin_samples[mid], "log curve should be above linear at midpoint");
        assert!(exp_samples[mid] < lin_samples[mid], "exp curve should be below linear at midpoint");
    }

    #[test]
    fn trigger_with_dip_behavior() {
        let mut env = make_env();
        trig(&mut env, 5.0, 0.0, 0.0, 50.0, 0.0, 0.7, 200.0, 0.0);
        run_samples(&mut env, 5000);

        let val_before = env.current_value;
        assert!((val_before - 0.7).abs() < 0.05);

        env.trigger_with_dip(50.0, 0.0, 0.0, 50.0, 0.0, 0.7, 200.0, 0.0, OS, 0.5, false, false, false);
        assert_eq!(env.stage, EnvelopeStage::Dip);

        let dip_samples_count = (DIP_MS * 0.001 * SR) as usize;
        let dip_samples = run_samples(&mut env, dip_samples_count + 1);
        let dip_end = *dip_samples.last().unwrap();
        let expected = val_before * 0.5;
        assert!((dip_end - expected).abs() < 0.1, "dip should reach target: got={dip_end}, expected={expected}");
        assert_eq!(env.stage, EnvelopeStage::Attack);
    }

    #[test]
    fn trigger_with_dip_is_smooth() {
        let mut env = make_env();
        trig(&mut env, 5.0, 0.0, 0.0, 50.0, 0.0, 0.7, 200.0, 0.0);
        run_samples(&mut env, 5000);

        let val_before = env.current_value;
        env.trigger_with_dip(5.0, 0.0, 0.0, 50.0, 0.0, 0.7, 200.0, 0.0, OS, 0.8, false, false, false);

        let mut prev = val_before;
        for _ in 0..500 {
            let v = env.next();
            let delta = (v - prev).abs();
            assert!(delta < 0.02, "envelope jump too large: {delta} (prev={prev}, cur={v})");
            prev = v;
        }
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
        trig(&mut env, 0.0, 0.0, 0.0, 100.0, 0.0, 0.7, 200.0, 0.0);
        assert_eq!(env.stage, EnvelopeStage::Attack);

        let samples = run_samples(&mut env, 100);
        assert!(samples.iter().any(|&v| v > 0.9), "minimum attack should still reach peak quickly");
    }

    #[test]
    fn full_adsr_cycle_reaches_zero() {
        let mut env = make_env();
        trig(&mut env, 10.0, 0.0, 0.0, 50.0, 0.0, 0.7, 100.0, 0.0);
        run_samples(&mut env, 10000);
        env.release();
        let samples = run_samples(&mut env, 44100);
        assert_eq!(env.stage, EnvelopeStage::Idle);
        assert!(*samples.last().unwrap() < 0.0001);
    }

    #[test]
    fn release_floor_prevents_sub_3ms() {
        let mut env = make_env();
        trig(&mut env, 1.0, 0.0, 0.0, 1.0, 0.0, 1.0, 0.1, 0.0);
        run_samples(&mut env, 500);
        assert_eq!(env.stage, EnvelopeStage::Sustain);

        env.release();
        assert_eq!(env.stage, EnvelopeStage::Release);

        let min_samples = (MIN_RELEASE_MS * 0.001 * SR) as usize;
        let half = min_samples / 2;
        let samples = run_samples(&mut env, half);
        assert!(samples.last().unwrap() > &0.4,
            "at midpoint of 3ms floor the envelope should still be significant: got {}",
            samples.last().unwrap());

        assert_eq!(env.stage, EnvelopeStage::Release, "should still be in release at midpoint");

        let rest = run_samples(&mut env, min_samples + 500);
        assert!(*rest.last().unwrap() < 0.0001, "should reach zero after floor time");
    }

    #[test]
    fn force_off_uses_quick_fade() {
        let mut env = make_env();
        trig(&mut env, 1.0, 0.0, 0.0, 1.0, 0.0, 0.8, 200.0, 0.0);
        run_samples(&mut env, 500);

        let val_before = env.current_value;
        assert!(val_before > 0.1);

        env.force_off();
        assert_eq!(env.stage, EnvelopeStage::QuickFade);
        assert!(env.is_active());

        let mut prev = val_before;
        let fade_samples = (QUICK_FADE_MS * 0.001 * SR) as usize + 5;
        for _ in 0..fade_samples {
            let v = env.next();
            assert!(v <= prev + 0.001, "quick fade must be monotonically decreasing");
            prev = v;
        }
        assert_eq!(env.stage, EnvelopeStage::Idle);
        assert!(!env.is_active());
    }

    #[test]
    fn force_off_when_already_silent() {
        let mut env = make_env();
        assert_eq!(env.current_value, 0.0);

        env.force_off();
        assert_eq!(env.stage, EnvelopeStage::Idle);
        assert_eq!(env.current_value, 0.0);
    }

    #[test]
    fn retrigger_during_quick_fade_is_continuous() {
        let mut env = make_env();
        trig(&mut env, 1.0, 0.0, 0.0, 1.0, 0.0, 0.8, 200.0, 0.0);
        run_samples(&mut env, 500);

        env.force_off();
        assert_eq!(env.stage, EnvelopeStage::QuickFade);
        run_samples(&mut env, 20);

        let val_before = env.current_value;
        assert!(val_before > 0.0);

        trig(&mut env, 10.0, 0.0, 0.0, 100.0, 0.0, 0.7, 200.0, 0.0);
        assert_eq!(env.stage, EnvelopeStage::Attack);

        let val_after = env.next();
        assert!((val_after - val_before).abs() < 0.05,
            "retrigger from quick fade must be continuous: before={val_before}, after={val_after}");
    }

    #[test]
    fn attack_allows_sub_half_ms() {
        let mut env = make_env();
        trig(&mut env, 0.1, 0.0, 0.0, 100.0, 0.0, 0.7, 200.0, 0.0);
        assert_eq!(env.stage, EnvelopeStage::Attack);

        let samples = run_samples(&mut env, 20);
        assert!(samples.iter().any(|&v| v > 0.9), "0.1ms attack should reach peak very quickly");
    }

    #[test]
    fn hold_stage_delays_decay() {
        let mut env = make_env();
        trig(&mut env, 1.0, 0.0, 50.0, 100.0, 0.0, 0.5, 200.0, 0.0);
        run_samples(&mut env, 50);
        assert_eq!(env.stage, EnvelopeStage::Hold);
        assert!((env.current_value - 1.0).abs() < 0.001, "hold should keep peak value");

        let hold_samples = (50.0 * 0.001 * SR) as usize;
        run_samples(&mut env, hold_samples);
        assert_eq!(env.stage, EnvelopeStage::Decay);
    }

    #[test]
    fn zero_hold_skips_to_decay() {
        let mut env = make_env();
        trig(&mut env, 1.0, 0.0, 0.0, 100.0, 0.0, 0.5, 200.0, 0.0);
        run_samples(&mut env, 50);
        assert!(env.stage == EnvelopeStage::Decay || env.stage == EnvelopeStage::Sustain);
    }

    #[test]
    fn loop_ahd_cycles_while_held() {
        let mut env = make_env();
        env.trigger(5.0, 0.0, 10.0, 20.0, 0.0, 0.0, 200.0, 0.0, EnvelopeLoopMode::LoopAHD, false, false, false);

        let mut peaks = 0;
        let mut prev = 0.0_f64;
        let mut rising = true;
        for _ in 0..44100 {
            let v = env.next();
            if rising && v < prev {
                peaks += 1;
                rising = false;
            } else if !rising && v > prev {
                rising = true;
            }
            prev = v;
        }
        assert!(peaks >= 3, "loop should cycle multiple times: got {peaks} peaks");
    }

    #[test]
    fn loop_stops_on_release() {
        let mut env = make_env();
        env.trigger(5.0, 0.0, 0.0, 20.0, 0.0, 0.0, 100.0, 0.0, EnvelopeLoopMode::LoopAHD, false, false, false);
        run_samples(&mut env, 2000);

        env.release();
        assert_eq!(env.stage, EnvelopeStage::Release);

        let samples = run_samples(&mut env, 44100);
        assert!(*samples.last().unwrap() < 0.001);
    }
}
