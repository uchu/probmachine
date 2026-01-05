#![allow(clippy::too_many_arguments)]

use synfx_dsp::{Oversampling, f_distort, SlewValue};
use super::oscillator::{Oscillator, PolyBlepWrapper, PLLOscillator};
use super::filter::MoogFilter;
use super::envelope::Envelope;
use super::reverb::StereoReverb;
use super::formant::FormantFilter;
use super::lfo::ModulationValues;

struct SineOscillator {
    phase: f64,
    sample_rate: f64,
}

impl SineOscillator {
    fn new(sample_rate: f64) -> Self {
        Self { phase: 0.0, sample_rate }
    }

    fn next(&mut self, freq: f64) -> f64 {
        let output = (self.phase * std::f64::consts::TAU).sin();
        self.phase += freq / self.sample_rate;
        if self.phase >= 1.0 {
            self.phase -= 1.0;
        }
        output
    }
}

pub struct Voice {
    // ===== Oscillators =====
    vps_oscillator_left: Oscillator,
    vps_oscillator_right: Oscillator,
    sub_oscillator: SineOscillator,
    pll_oscillator_left: PLLOscillator,
    pll_oscillator_right: PLLOscillator,
    pll_reference_oscillator: PolyBlepWrapper,
    fm_oscillator: PolyBlepWrapper,

    // ===== Processing =====
    filter_left: MoogFilter,
    filter_right: MoogFilter,
    formant_filter_left: FormantFilter,
    formant_filter_right: FormantFilter,
    volume_envelope: Envelope,
    filter_envelope: Envelope,
    oversampling_left: Oversampling<4>,
    oversampling_right: Oversampling<4>,

    // ===== Global Parameters =====
    base_frequency: f64,
    master_volume: f64,
    sample_rate: f64,

    // ===== Stereo =====
    pll_damping_stereo_offset: f64,

    // ===== VPS Oscillator =====
    vps_octave: i32,
    vps_d_param: f64,
    vps_v_param: f64,
    vps_volume: f64,
    vps_stereo_v_offset: f64,

    // ===== Sub Oscillator =====
    sub_volume: f64,

    // ===== PLL =====
    pll_volume: f64,
    pll_track_speed: f64,
    pll_damping: f64,
    pll_multiplier: f64,
    pll_feedback_amount: f64,
    pll_feedback_state: f64,
    pll_distortion_amount: f64,
    pll_mode_is_edge: bool,
    pll_colored: bool,

    // ===== PLL FM =====
    pll_fm_amount: f64,
    pll_fm_ratio: i32,

    // ===== PLL Reference =====
    pll_ref_octave: i32,
    pll_ref_tune_semitones: i32,
    pll_ref_fine_tune_cents: f64,
    pll_ref_pulse_width: f64,

    // ===== Formant =====
    formant_mix: f64,
    #[allow(dead_code)]
    formant_vowel: f64,
    #[allow(dead_code)]
    formant_shift: f64,

    // ===== VPS Distortion =====
    distortion_amount: f64,

    // ===== Filter =====
    filter_enabled: bool,
    filter_cutoff: f64,
    filter_resonance: f64,
    filter_envelope_amount: f64,
    filter_drive: f64,
    filter_mode: i32,

    // ===== Volume Envelope =====
    vol_env_attack: f64,
    vol_env_attack_shape: f64,
    vol_env_decay: f64,
    vol_env_decay_shape: f64,
    vol_env_sustain: f64,
    vol_env_release: f64,
    vol_env_release_shape: f64,

    // ===== Filter Envelope =====
    filt_env_attack: f64,
    filt_env_attack_shape: f64,
    filt_env_decay: f64,
    filt_env_decay_shape: f64,
    filt_env_sustain: f64,
    filt_env_release: f64,
    filt_env_release_shape: f64,

    // ===== Reverb =====
    reverb: StereoReverb,

    // ===== Slew Limiters =====
    freq_slew: SlewValue<f64>,
    pll_volume_slew: SlewValue<f64>,
    pll_track_slew: SlewValue<f64>,
    pll_damping_slew: SlewValue<f64>,
    pll_influence_slew: SlewValue<f64>,
    pll_feedback_slew: SlewValue<f64>,
    pll_dist_amount_slew: SlewValue<f64>,
    pll_fine_tune_slew: SlewValue<f64>,
    pll_pulse_width_slew: SlewValue<f64>,
    pll_stereo_offset_slew: SlewValue<f64>,
    pll_fm_amount_slew: SlewValue<f64>,

    // VPS slew limiters
    vps_d_slew: SlewValue<f64>,
    vps_v_slew: SlewValue<f64>,
    vps_volume_slew: SlewValue<f64>,
    vps_stereo_offset_slew: SlewValue<f64>,
    vps_dist_slew: SlewValue<f64>,

    // Sub slew
    sub_volume_slew: SlewValue<f64>,

    // Formant slew
    formant_mix_slew: SlewValue<f64>,
    formant_vowel_slew: SlewValue<f64>,
    formant_shift_slew: SlewValue<f64>,

    // Filter slew - not used since nih_plug parameters have built-in smoothing
    #[allow(dead_code)]
    filter_cutoff_slew: SlewValue<f64>,
    #[allow(dead_code)]
    filter_resonance_slew: SlewValue<f64>,
    #[allow(dead_code)]
    filter_drive_slew: SlewValue<f64>,

    // ===== Target Values =====
    glide_time_ms: f64,
    target_frequency: f64,
    target_pll_volume: f64,
    target_pll_track: f64,
    target_pll_damping: f64,
    target_pll_influence: f64,
    target_pll_feedback: f64,
    target_pll_dist_amount: f64,
    target_pll_fine_tune: f64,
    target_pll_pulse_width: f64,
    target_pll_stereo_offset: f64,
    target_pll_fm_amount: f64,
    target_vps_d: f64,
    target_vps_v: f64,
    target_vps_volume: f64,
    target_vps_stereo_offset: f64,
    target_vps_dist: f64,
    target_sub_volume: f64,
    target_formant_mix: f64,
    target_formant_vowel: f64,
    target_formant_shift: f64,
    target_filter_cutoff: f64,
    target_filter_resonance: f64,
    target_filter_drive: f64,

    // ===== Modulation Offsets (applied per-sample from LFOs) =====
    mod_pll_damping: f64,
    mod_pll_influence: f64,
    mod_pll_track_speed: f64,
    mod_pll_feedback: f64,
    mod_pll_fm_amount: f64,
    mod_pll_pulse_width: f64,
    mod_vps_d: f64,
    mod_vps_v: f64,
    mod_filter_cutoff: f64,
    mod_filter_resonance: f64,
    mod_filter_drive: f64,
    mod_formant_vowel: f64,
    mod_formant_shift: f64,
    mod_reverb_mix: f64,
    mod_reverb_decay: f64,
    mod_pll_volume: f64,
    mod_vps_volume: f64,
    mod_sub_volume: f64,

    // Slews for modulation (critical for smooth, crackle-free modulation)
    mod_slew_pll_damping: SlewValue<f64>,
    mod_slew_pll_influence: SlewValue<f64>,
    mod_slew_pll_track: SlewValue<f64>,
    mod_slew_pll_feedback: SlewValue<f64>,
    mod_slew_pll_fm: SlewValue<f64>,
    mod_slew_pll_pw: SlewValue<f64>,
    mod_slew_vps_d: SlewValue<f64>,
    mod_slew_vps_v: SlewValue<f64>,
    mod_slew_filter_cut: SlewValue<f64>,
    mod_slew_filter_res: SlewValue<f64>,
    mod_slew_filter_drv: SlewValue<f64>,
    mod_slew_formant_vow: SlewValue<f64>,
    mod_slew_formant_shf: SlewValue<f64>,
    mod_slew_rev_mix: SlewValue<f64>,
    mod_slew_rev_decay: SlewValue<f64>,
    mod_slew_pll_vol: SlewValue<f64>,
    mod_slew_vps_vol: SlewValue<f64>,
    mod_slew_sub_vol: SlewValue<f64>,
}

impl Voice {
    pub fn new(sample_rate: f32) -> Self {
        let mut oversampling_left = Oversampling::<4>::new();
        let mut oversampling_right = Oversampling::<4>::new();
        oversampling_left.set_sample_rate(sample_rate);
        oversampling_right.set_sample_rate(sample_rate);

        let sample_rate_f64 = sample_rate as f64;
        let oversampled_rate = sample_rate_f64 * 4.0;

        let make_slew = || {
            let mut s = SlewValue::new();
            s.set_sample_rate(sample_rate_f64);
            s
        };

        Self {
            vps_oscillator_left: Oscillator::new(oversampled_rate),
            vps_oscillator_right: Oscillator::new(oversampled_rate),
            sub_oscillator: SineOscillator::new(oversampled_rate),
            pll_oscillator_left: PLLOscillator::new(oversampled_rate),
            pll_oscillator_right: PLLOscillator::new(oversampled_rate),
            pll_reference_oscillator: PolyBlepWrapper::new(oversampled_rate),
            fm_oscillator: PolyBlepWrapper::new(oversampled_rate),

            filter_left: MoogFilter::new(oversampled_rate),
            filter_right: MoogFilter::new(oversampled_rate),
            formant_filter_left: FormantFilter::new(oversampled_rate),
            formant_filter_right: FormantFilter::new(oversampled_rate),
            volume_envelope: Envelope::new(sample_rate_f64),
            filter_envelope: Envelope::new(sample_rate_f64),
            oversampling_left,
            oversampling_right,

            base_frequency: 220.0,
            master_volume: 0.8,
            sample_rate: sample_rate_f64,
            pll_damping_stereo_offset: 0.0,

            vps_octave: 0,
            vps_d_param: 0.5,
            vps_v_param: 0.5,
            vps_volume: 1.0,
            vps_stereo_v_offset: 0.0,

            sub_volume: 0.0,

            pll_volume: 0.0,
            pll_track_speed: 0.5,
            pll_damping: 0.3,
            pll_multiplier: 1.0,
            pll_feedback_amount: 0.0,
            pll_feedback_state: 0.0,
            pll_distortion_amount: 0.0,
            pll_mode_is_edge: false,
            pll_colored: false,

            pll_fm_amount: 0.0,
            pll_fm_ratio: 1,

            pll_ref_octave: 0,
            pll_ref_tune_semitones: 0,
            pll_ref_fine_tune_cents: 0.0,
            pll_ref_pulse_width: 0.5,

            formant_mix: 0.0,
            formant_vowel: 0.0,
            formant_shift: 0.0,

            distortion_amount: 0.0,

            filter_enabled: true,
            filter_cutoff: 1000.0,
            filter_resonance: 0.0,
            filter_envelope_amount: 0.0,
            filter_drive: 1.0,
            filter_mode: 3,

            vol_env_attack: 1.0,
            vol_env_attack_shape: 0.5,
            vol_env_decay: 20.0,
            vol_env_decay_shape: 0.5,
            vol_env_sustain: 1.0,
            vol_env_release: 5.0,
            vol_env_release_shape: 0.5,

            filt_env_attack: 1.0,
            filt_env_attack_shape: 0.5,
            filt_env_decay: 20.0,
            filt_env_decay_shape: 0.5,
            filt_env_sustain: 1.0,
            filt_env_release: 5.0,
            filt_env_release_shape: 0.5,

            reverb: StereoReverb::new(oversampled_rate as f32),

            freq_slew: make_slew(),
            pll_volume_slew: make_slew(),
            pll_track_slew: make_slew(),
            pll_damping_slew: make_slew(),
            pll_influence_slew: make_slew(),
            pll_feedback_slew: make_slew(),
            pll_dist_amount_slew: make_slew(),
            pll_fine_tune_slew: make_slew(),
            pll_pulse_width_slew: make_slew(),
            pll_stereo_offset_slew: make_slew(),
            pll_fm_amount_slew: make_slew(),
            vps_d_slew: make_slew(),
            vps_v_slew: make_slew(),
            vps_volume_slew: make_slew(),
            vps_stereo_offset_slew: make_slew(),
            vps_dist_slew: make_slew(),
            sub_volume_slew: make_slew(),
            formant_mix_slew: make_slew(),
            formant_vowel_slew: make_slew(),
            formant_shift_slew: make_slew(),
            filter_cutoff_slew: make_slew(),
            filter_resonance_slew: make_slew(),
            filter_drive_slew: make_slew(),

            glide_time_ms: 0.0,
            target_frequency: 220.0,
            target_pll_volume: 0.0,
            target_pll_track: 0.5,
            target_pll_damping: 0.3,
            target_pll_influence: 0.5,
            target_pll_feedback: 0.0,
            target_pll_dist_amount: 0.0,
            target_pll_fine_tune: 0.0,
            target_pll_pulse_width: 0.5,
            target_pll_stereo_offset: 0.0,
            target_pll_fm_amount: 0.0,
            target_vps_d: 0.5,
            target_vps_v: 0.5,
            target_vps_volume: 1.0,
            target_vps_stereo_offset: 0.0,
            target_vps_dist: 0.0,
            target_sub_volume: 0.0,
            target_formant_mix: 0.0,
            target_formant_vowel: 0.0,
            target_formant_shift: 0.0,
            target_filter_cutoff: 1000.0,
            target_filter_resonance: 0.0,
            target_filter_drive: 1.0,

            mod_pll_damping: 0.0,
            mod_pll_influence: 0.0,
            mod_pll_track_speed: 0.0,
            mod_pll_feedback: 0.0,
            mod_pll_fm_amount: 0.0,
            mod_pll_pulse_width: 0.0,
            mod_vps_d: 0.0,
            mod_vps_v: 0.0,
            mod_filter_cutoff: 0.0,
            mod_filter_resonance: 0.0,
            mod_filter_drive: 0.0,
            mod_formant_vowel: 0.0,
            mod_formant_shift: 0.0,
            mod_reverb_mix: 0.0,
            mod_reverb_decay: 0.0,
            mod_pll_volume: 0.0,
            mod_vps_volume: 0.0,
            mod_sub_volume: 0.0,

            mod_slew_pll_damping: make_slew(),
            mod_slew_pll_influence: make_slew(),
            mod_slew_pll_track: make_slew(),
            mod_slew_pll_feedback: make_slew(),
            mod_slew_pll_fm: make_slew(),
            mod_slew_pll_pw: make_slew(),
            mod_slew_vps_d: make_slew(),
            mod_slew_vps_v: make_slew(),
            mod_slew_filter_cut: make_slew(),
            mod_slew_filter_res: make_slew(),
            mod_slew_filter_drv: make_slew(),
            mod_slew_formant_vow: make_slew(),
            mod_slew_formant_shf: make_slew(),
            mod_slew_rev_mix: make_slew(),
            mod_slew_rev_decay: make_slew(),
            mod_slew_pll_vol: make_slew(),
            mod_slew_vps_vol: make_slew(),
            mod_slew_sub_vol: make_slew(),
        }
    }

    pub fn set_pll_stereo_damp_offset(&mut self, offset: f64) {
        self.target_pll_stereo_offset = offset.clamp(0.0, 1.0);
    }

    pub fn set_glide_time(&mut self, time_ms: f64) {
        self.glide_time_ms = time_ms.max(0.0);
    }

    pub fn set_frequency(&mut self, freq: f64, _pll_feedback: f64, feedback_amount: f64) {
        self.target_frequency = freq;
        self.target_pll_feedback = feedback_amount;
    }

    pub fn set_osc_params(&mut self, d: f64, v: f64) {
        self.target_vps_d = d;
        self.target_vps_v = v;
    }

    pub fn set_vps_stereo_v_offset(&mut self, offset: f64) {
        self.target_vps_stereo_offset = offset.clamp(0.0, 1.0);
    }

    pub fn set_osc_volume(&mut self, volume: f64) {
        self.target_vps_volume = volume;
    }

    pub fn set_osc_octave(&mut self, octave: i32) {
        self.vps_octave = octave;
    }

    pub fn set_sub_volume(&mut self, volume: f64) {
        self.target_sub_volume = volume;
    }

    pub fn set_pll_ref_params(&mut self, octave: i32, tune: i32, fine_tune: f64, pulse_width: f64) {
        self.pll_ref_octave = octave;
        self.pll_ref_tune_semitones = tune;
        self.target_pll_fine_tune = fine_tune;
        self.target_pll_pulse_width = pulse_width;
    }

    pub fn set_pll_params(&mut self, track: f64, damp: f64, mult: f64, influence: f64, colored: bool, edge_mode: bool) {
        self.target_pll_track = track;
        self.target_pll_damping = damp;
        self.pll_multiplier = mult;
        self.target_pll_influence = influence;
        self.pll_mode_is_edge = edge_mode;
        self.pll_colored = colored;
    }

    pub fn set_pll_volume(&mut self, volume: f64) {
        self.target_pll_volume = volume;
    }

    pub fn set_pll_distortion(&mut self, amount: f64) {
        self.target_pll_dist_amount = amount;
    }

    pub fn set_pll_fm_params(&mut self, amount: f64, ratio: i32) {
        self.target_pll_fm_amount = amount;
        self.pll_fm_ratio = ratio;
    }

    pub fn set_distortion(&mut self, amount: f64) {
        self.target_vps_dist = amount;
    }

    pub fn set_formant_params(&mut self, mix: f64, vowel: f64, shift: f64) {
        self.target_formant_mix = mix;
        self.target_formant_vowel = vowel;
        self.target_formant_shift = shift;
    }

    pub fn set_filter_params(&mut self, enabled: bool, cutoff: f64, resonance: f64, env_amount: f64, drive: f64, mode: i32) {
        self.filter_enabled = enabled;
        self.target_filter_cutoff = cutoff;
        self.target_filter_resonance = resonance;
        self.filter_envelope_amount = env_amount;
        self.target_filter_drive = drive;
        self.filter_mode = mode;
    }

    pub fn set_reverb_params(
        &mut self,
        mix: f64,
        pre_delay_ms: f64,
        time_scale: f64,
        input_hpf_hz: f64,
        input_lpf_hz: f64,
        reverb_hpf_hz: f64,
        reverb_lpf_hz: f64,
        mod_speed: f64,
        mod_depth: f64,
        mod_shape: f64,
        input_diffusion_mix: f64,
        diffusion: f64,
        decay: f64,
        ducking: f64,
    ) {
        self.reverb.set_params(
            mix,
            pre_delay_ms,
            time_scale,
            input_hpf_hz,
            input_lpf_hz,
            reverb_hpf_hz,
            reverb_lpf_hz,
            mod_speed,
            mod_depth,
            mod_shape,
            input_diffusion_mix,
            diffusion,
            decay,
            ducking,
        );
    }

    pub fn apply_modulation(&mut self, mod_values: &ModulationValues) {
        // Apply slew to all modulation values for smooth, crackle-free modulation
        // Using fast slew (5ms) for responsive modulation while avoiding artifacts
        const MOD_SLEW_MS: f64 = 5.0;

        self.mod_pll_damping = self.mod_slew_pll_damping.next(mod_values.pll_damping, MOD_SLEW_MS);
        self.mod_pll_influence = self.mod_slew_pll_influence.next(mod_values.pll_influence, MOD_SLEW_MS);
        self.mod_pll_track_speed = self.mod_slew_pll_track.next(mod_values.pll_track_speed, MOD_SLEW_MS);
        self.mod_pll_feedback = self.mod_slew_pll_feedback.next(mod_values.pll_feedback, MOD_SLEW_MS);
        self.mod_pll_fm_amount = self.mod_slew_pll_fm.next(mod_values.pll_fm_amount, MOD_SLEW_MS);
        self.mod_pll_pulse_width = self.mod_slew_pll_pw.next(mod_values.pll_pulse_width, MOD_SLEW_MS);
        self.mod_vps_d = self.mod_slew_vps_d.next(mod_values.vps_d, MOD_SLEW_MS);
        self.mod_vps_v = self.mod_slew_vps_v.next(mod_values.vps_v, MOD_SLEW_MS);
        self.mod_filter_cutoff = self.mod_slew_filter_cut.next(mod_values.filter_cutoff, MOD_SLEW_MS);
        self.mod_filter_resonance = self.mod_slew_filter_res.next(mod_values.filter_resonance, MOD_SLEW_MS);
        self.mod_filter_drive = self.mod_slew_filter_drv.next(mod_values.filter_drive, MOD_SLEW_MS);
        self.mod_formant_vowel = self.mod_slew_formant_vow.next(mod_values.formant_vowel, MOD_SLEW_MS);
        self.mod_formant_shift = self.mod_slew_formant_shf.next(mod_values.formant_shift, MOD_SLEW_MS);
        self.mod_reverb_mix = self.mod_slew_rev_mix.next(mod_values.reverb_mix, MOD_SLEW_MS);
        self.mod_reverb_decay = self.mod_slew_rev_decay.next(mod_values.reverb_decay, MOD_SLEW_MS);
        self.mod_pll_volume = self.mod_slew_pll_vol.next(mod_values.pll_volume, MOD_SLEW_MS);
        self.mod_vps_volume = self.mod_slew_vps_vol.next(mod_values.vps_volume, MOD_SLEW_MS);
        self.mod_sub_volume = self.mod_slew_sub_vol.next(mod_values.sub_volume, MOD_SLEW_MS);
    }

    pub fn set_volume(&mut self, volume: f64) {
        self.master_volume = volume;
    }

    pub fn set_volume_envelope(
        &mut self,
        attack: f64,
        attack_shape: f64,
        decay: f64,
        decay_shape: f64,
        sustain: f64,
        release: f64,
        release_shape: f64,
    ) {
        self.vol_env_attack = attack;
        self.vol_env_attack_shape = attack_shape;
        self.vol_env_decay = decay;
        self.vol_env_decay_shape = decay_shape;
        self.vol_env_sustain = sustain;
        self.vol_env_release = release;
        self.vol_env_release_shape = release_shape;
    }

    pub fn set_filter_envelope(
        &mut self,
        attack: f64,
        attack_shape: f64,
        decay: f64,
        decay_shape: f64,
        sustain: f64,
        release: f64,
        release_shape: f64,
    ) {
        self.filt_env_attack = attack;
        self.filt_env_attack_shape = attack_shape;
        self.filt_env_decay = decay;
        self.filt_env_decay_shape = decay_shape;
        self.filt_env_sustain = sustain;
        self.filt_env_release = release;
        self.filt_env_release_shape = release_shape;
    }

    pub fn trigger(&mut self) {
        self.volume_envelope.trigger(
            self.vol_env_attack,
            self.vol_env_attack_shape,
            self.vol_env_decay,
            self.vol_env_decay_shape,
            self.vol_env_sustain,
            self.vol_env_release,
            self.vol_env_release_shape,
        );

        self.filter_envelope.trigger(
            self.filt_env_attack,
            self.filt_env_attack_shape,
            self.filt_env_decay,
            self.filt_env_decay_shape,
            self.filt_env_sustain,
            self.filt_env_release,
            self.filt_env_release_shape,
        );

        self.pll_oscillator_left.trigger();
        self.pll_oscillator_right.trigger();
    }

    pub fn release(&mut self) {
        self.volume_envelope.release();
        self.filter_envelope.release();
    }

    pub fn process(&mut self, _pll_feedback: f64) -> (f64, f64) {
        let volume_env = self.volume_envelope.next();
        let filter_env = self.filter_envelope.next();

        // Slew all continuous parameters
        let glide_ms = if self.glide_time_ms > 0.5 { self.glide_time_ms } else { 0.5 };
        self.base_frequency = self.freq_slew.next(self.target_frequency, glide_ms / 500.0);

        // PLL slews + modulation
        self.pll_volume = (self.pll_volume_slew.next(self.target_pll_volume, 20.0) + self.mod_pll_volume).clamp(0.0, 1.0);
        self.pll_track_speed = (self.pll_track_slew.next(self.target_pll_track, 20.0) + self.mod_pll_track_speed).clamp(0.0, 1.0);
        self.pll_damping = (self.pll_damping_slew.next(self.target_pll_damping, 20.0) + self.mod_pll_damping).clamp(0.0, 1.0);
        let slewed_influence = (self.pll_influence_slew.next(self.target_pll_influence, 20.0) + self.mod_pll_influence).clamp(0.0, 1.0);
        self.pll_feedback_amount = (self.pll_feedback_slew.next(self.target_pll_feedback, 200.0) + self.mod_pll_feedback).clamp(0.0, 1.0);
        self.pll_distortion_amount = self.pll_dist_amount_slew.next(self.target_pll_dist_amount, 20.0);
        self.pll_ref_fine_tune_cents = self.pll_fine_tune_slew.next(self.target_pll_fine_tune, 10.0);
        self.pll_ref_pulse_width = (self.pll_pulse_width_slew.next(self.target_pll_pulse_width, 20.0) + self.mod_pll_pulse_width).clamp(0.05, 0.95);
        self.pll_damping_stereo_offset = self.pll_stereo_offset_slew.next(self.target_pll_stereo_offset, 60.0);
        self.pll_fm_amount = (self.pll_fm_amount_slew.next(self.target_pll_fm_amount, 20.0) + self.mod_pll_fm_amount).clamp(0.0, 1.0);

        // VPS slews + modulation
        self.vps_d_param = (self.vps_d_slew.next(self.target_vps_d, 20.0) + self.mod_vps_d).clamp(0.0, 1.0);
        self.vps_v_param = (self.vps_v_slew.next(self.target_vps_v, 20.0) + self.mod_vps_v).clamp(0.0, 1.0);
        self.vps_volume = (self.vps_volume_slew.next(self.target_vps_volume, 20.0) + self.mod_vps_volume).clamp(0.0, 1.0);
        self.vps_stereo_v_offset = self.vps_stereo_offset_slew.next(self.target_vps_stereo_offset, 20.0);
        self.distortion_amount = self.vps_dist_slew.next(self.target_vps_dist, 20.0);

        // Sub slew + modulation
        self.sub_volume = (self.sub_volume_slew.next(self.target_sub_volume, 20.0) + self.mod_sub_volume).clamp(0.0, 1.0);

        // Formant slews + modulation
        self.formant_mix = self.formant_mix_slew.next(self.target_formant_mix, 20.0);
        let formant_vowel_mod = (self.formant_vowel_slew.next(self.target_formant_vowel, 30.0) + self.mod_formant_vowel).clamp(0.0, 1.0);
        let formant_shift_mod = (self.formant_shift_slew.next(self.target_formant_shift, 20.0) + self.mod_formant_shift).clamp(-2.0, 2.0);

        // Filter parameters - cutoff already smoothed by nih_plug parameter smoother
        // Using target values directly to avoid double-smoothing issues
        let cutoff_mod_hz = self.mod_filter_cutoff * 10000.0; // Scale modulation to ±10kHz
        self.filter_cutoff = self.target_filter_cutoff;
        self.filter_resonance = (self.target_filter_resonance + self.mod_filter_resonance).clamp(0.0, 0.99);
        self.filter_drive = (self.target_filter_drive + self.mod_filter_drive).clamp(1.0, 15.0);

        let cutoff = (self.filter_cutoff + filter_env * self.filter_envelope_amount + cutoff_mod_hz)
            .clamp(20.0, 20000.0);

        self.pll_oscillator_left.prepare_block();
        self.pll_oscillator_right.prepare_block();

        // Update formant filters with modulated vowel and shift
        self.formant_filter_left.set_vowel(formant_vowel_mod, formant_shift_mod);
        self.formant_filter_right.set_vowel(formant_vowel_mod, formant_shift_mod);

        let buf_l = self.oversampling_left.resample_buffer();
        let buf_r = self.oversampling_right.resample_buffer();
        let mut feedback = self.pll_feedback_state;

        for i in 0..4 {
            let mut mixed_oscillators_l = 0.0_f64;
            let mut mixed_oscillators_r = 0.0_f64;

            // VPS Oscillators (no FM or formant applied)
            if self.vps_volume > 0.001 {
                let base_freq = self.base_frequency * 2.0_f64.powi(self.vps_octave);

                let use_stereo = self.vps_stereo_v_offset > 0.0001;

                let v_left = if use_stereo {
                    (self.vps_v_param - self.vps_stereo_v_offset).clamp(0.0, 1.0)
                } else {
                    self.vps_v_param
                };
                let v_right = if use_stereo {
                    (self.vps_v_param + self.vps_stereo_v_offset).clamp(0.0, 1.0)
                } else {
                    self.vps_v_param
                };

                self.vps_oscillator_left.set_frequency(base_freq);
                let raw_l = self.vps_oscillator_left.next(self.vps_d_param, v_left);

                self.vps_oscillator_right.set_frequency(base_freq);
                let raw_r = if use_stereo {
                    self.vps_oscillator_right.next(self.vps_d_param, v_right)
                } else {
                    raw_l
                };

                // Distortion with fixed threshold 0.7
                let distortion_gain = 1.0 + self.distortion_amount * 49.0;
                let left = f_distort(distortion_gain as f32, 0.7, raw_l as f32) as f64;
                let right = f_distort(distortion_gain as f32, 0.7, raw_r as f32) as f64;

                mixed_oscillators_l += left * self.vps_volume * volume_env;
                mixed_oscillators_r += right * self.vps_volume * volume_env;
            }

            // PLL Oscillators with FM and Formant (both applied only to PLL)
            if self.pll_volume > 0.001 {
                let tune_oct = (self.pll_ref_tune_semitones as f64 + self.pll_ref_fine_tune_cents) / 12.0;
                let ref_freq = self.base_frequency * 2.0_f64.powi(self.pll_ref_octave) * 2.0_f64.powf(tune_oct);

                // FM modulation - modulate the reference frequency (only affects PLL)
                let fm_freq = ref_freq * self.pll_fm_ratio as f64;
                self.fm_oscillator.set_frequency(fm_freq);
                let fm_signal = self.fm_oscillator.next_sin();
                // FM index scaled for musical results
                let fm_index = self.pll_fm_amount * 4.0 * self.pll_fm_ratio as f64;
                let fm_mod = fm_signal * fm_index * ref_freq;

                let fb_mod = feedback * self.pll_feedback_amount * 5.0;
                let ref_mod = ((ref_freq + fm_mod) * (1.0 + fb_mod)).clamp(20.0, self.sample_rate * 2.0);

                self.pll_reference_oscillator.set_frequency(ref_mod);
                let ref_pulse = self.pll_reference_oscillator.next(self.pll_ref_pulse_width);
                let ref_phase = self.pll_reference_oscillator.get_phase();

                let use_stereo = self.pll_damping_stereo_offset > 0.0001;

                let damp_left = (self.pll_damping - self.pll_damping_stereo_offset).clamp(0.001, 1.0);
                let damp_right = if use_stereo {
                    (self.pll_damping + self.pll_damping_stereo_offset).clamp(0.001, 1.0)
                } else {
                    damp_left
                };

                let mode = if self.pll_mode_is_edge {
                    super::oscillator::PllMode::EdgePFD
                } else {
                    super::oscillator::PllMode::AnalogLikePD
                };

                self.pll_oscillator_left.set_params(self.pll_track_speed, damp_left, self.pll_multiplier, slewed_influence, self.pll_colored, mode);
                let pll_raw_l = self.pll_oscillator_left.next(ref_phase, ref_mod, ref_pulse);

                let pll_raw_r = if use_stereo {
                    self.pll_oscillator_right.set_params(self.pll_track_speed, damp_right, self.pll_multiplier, slewed_influence, self.pll_colored, mode);
                    self.pll_oscillator_right.next(ref_phase, ref_mod, ref_pulse)
                } else {
                    pll_raw_l
                };

                // PLL Distortion with fixed threshold 0.7
                let pll_dist_gain = 1.0 + self.pll_distortion_amount * 49.0;
                let mut pll_out_l = f_distort(pll_dist_gain as f32, 0.7, pll_raw_l as f32) as f64;
                let mut pll_out_r = if use_stereo {
                    f_distort(pll_dist_gain as f32, 0.7, pll_raw_r as f32) as f64
                } else {
                    pll_out_l
                };

                // Apply formant filter ONLY to PLL output
                if self.formant_mix > 0.001 {
                    let formant_l = self.formant_filter_left.process(pll_out_l);
                    let formant_r = self.formant_filter_right.process(pll_out_r);
                    pll_out_l = pll_out_l * (1.0 - self.formant_mix) + formant_l * self.formant_mix;
                    pll_out_r = pll_out_r * (1.0 - self.formant_mix) + formant_r * self.formant_mix;
                }

                mixed_oscillators_l += pll_out_l * self.pll_volume * volume_env;
                mixed_oscillators_r += pll_out_r * self.pll_volume * volume_env;

                feedback = feedback * 0.9 + (pll_raw_l + pll_raw_r) * 0.05;
            }

            buf_l[i] = mixed_oscillators_l as f32;
            buf_r[i] = mixed_oscillators_r as f32;
        }

        // Apply filter at oversampled rate (always process when enabled to avoid crackling)
        if self.filter_enabled {
            self.filter_left.process_buffer(
                unsafe { &mut *(buf_l.as_mut_ptr() as *mut [f32; 4]) },
                cutoff as f32,
                self.filter_resonance as f32,
                self.filter_drive as f32,
                self.filter_mode,
            );
            self.filter_right.process_buffer(
                unsafe { &mut *(buf_r.as_mut_ptr() as *mut [f32; 4]) },
                cutoff as f32,
                self.filter_resonance as f32,
                self.filter_drive as f32,
                self.filter_mode,
            );
        }

        // Apply reverb at oversampled rate
        if self.reverb.mix > 0.0 {
            for i in 0..4 {
                let dry_l = buf_l[i] as f64;
                let dry_r = buf_r[i] as f64;

                let (wet_l, wet_r) = self.reverb.process(dry_l, dry_r);

                buf_l[i] = (dry_l * (1.0 - self.reverb.mix) + wet_l * self.reverb.mix) as f32;
                buf_r[i] = (dry_r * (1.0 - self.reverb.mix) + wet_r * self.reverb.mix) as f32;
            }
        }

        // Add sub oscillator at oversampled rate (pure sine at -1 octave)
        // Sub is added after filter/reverb so it remains clean and punchy
        if self.sub_volume > 0.001 {
            let sub_freq = self.base_frequency * 0.5; // -1 octave
            for i in 0..4 {
                let sub_sample = self.sub_oscillator.next(sub_freq) * self.sub_volume * volume_env;
                buf_l[i] += sub_sample as f32;
                buf_r[i] += sub_sample as f32;
            }
        }

        self.pll_feedback_state = feedback;

        // Downsample the complete signal
        let downsampled_l = self.oversampling_left.downsample() as f64;
        let downsampled_r = self.oversampling_right.downsample() as f64;

        // Apply master volume
        let final_l = downsampled_l * self.master_volume;
        let final_r = downsampled_r * self.master_volume;

        (final_l, final_r)
    }
}
