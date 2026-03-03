#![allow(clippy::too_many_arguments)]

use synfx_dsp::{Oversampling, SlewValue, apply_distortion};
use super::oscillator::{Oscillator, PolyBlepWrapper, PLLOscillator, SawOscillator, VpsPhaseMode};
use super::envelope::Envelope;
use super::lfo::ModulationValues;
use super::simd::{stereo, stereo_left, stereo_right, stereo_wavefold, OnePoleSlewValue};

struct SawTightFilter {
    ic1eq: f64,
    ic2eq: f64,
    sample_rate: f64,
}

impl SawTightFilter {
    fn new(sample_rate: f64) -> Self {
        Self { ic1eq: 0.0, ic2eq: 0.0, sample_rate }
    }

    fn set_sample_rate(&mut self, sample_rate: f64) {
        self.sample_rate = sample_rate;
    }

    #[allow(dead_code)]
    fn reset(&mut self) {
        self.ic1eq = 0.0;
        self.ic2eq = 0.0;
    }

    #[inline]
    fn process(&mut self, input: f64, amount: f64) -> f64 {
        if amount < 0.001 { return input; }

        let cutoff = 5.0 * (120.0_f64 / 5.0).powf(amount);
        let q = 0.707 + amount * 1.3;

        let g = (std::f64::consts::PI * cutoff / self.sample_rate).tan();
        let k = 1.0 / q;
        let a1 = 1.0 / (1.0 + g * (g + k));
        let a2 = g * a1;
        let a3 = g * a2;

        let v3 = input - self.ic2eq;
        let v1 = a1 * self.ic1eq + a2 * v3;
        let v2 = self.ic2eq + a2 * self.ic1eq + a3 * v3;
        self.ic1eq = 2.0 * v1 - self.ic1eq;
        self.ic2eq = 2.0 * v2 - self.ic2eq;

        input - k * v1 - v2
    }
}

struct SineOscillator {
    phase: f64,
    sample_rate: f64,
}

impl SineOscillator {
    fn new(sample_rate: f64) -> Self {
        Self { phase: 0.0, sample_rate }
    }

    fn set_sample_rate(&mut self, sample_rate: f64) {
        self.sample_rate = sample_rate;
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
    fm_oscillator: SineOscillator,
    saw_oscillator: SawOscillator,

    // ===== Processing =====
    volume_envelope: Envelope,
    oversampling_2x_left: Oversampling<2>,
    oversampling_2x_right: Oversampling<2>,
    oversampling_4x_left: Oversampling<4>,
    oversampling_4x_right: Oversampling<4>,
    oversampling_8x_left: Oversampling<8>,
    oversampling_8x_right: Oversampling<8>,
    oversampling_16x_left: Oversampling<16>,
    oversampling_16x_right: Oversampling<16>,
    oversampling_32x_left: Oversampling<32>,
    oversampling_32x_right: Oversampling<32>,
    oversampling_64x_left: Oversampling<64>,
    oversampling_64x_right: Oversampling<64>,
    oversampling_128x_left: Oversampling<128>,
    oversampling_128x_right: Oversampling<128>,
    saw_os_2x: Oversampling<2>,
    saw_os_4x: Oversampling<4>,
    saw_os_8x: Oversampling<8>,
    saw_os_16x: Oversampling<16>,
    saw_os_32x: Oversampling<32>,
    saw_os_64x: Oversampling<64>,
    saw_os_128x: Oversampling<128>,

    // ===== Global Parameters =====
    base_frequency: f64,
    master_volume: f64,
    daw_sample_rate: f64,
    processing_sample_rate: f64,
    base_rate_option: i32,  // 0=Auto, 1=44.1k, 2=88.2k, 3=96k, 4=192k

    // ===== Bypass Switches =====
    pll_enabled: bool,
    vps_enabled: bool,
    coloration_enabled: bool,

    // Per-oscillator oversampling
    current_pll_os_factor: i32,
    current_saw_os_factor: i32,
    current_vps_os_factor: i32,
    pll_effective_ratio: i32,
    saw_effective_ratio: i32,
    vps_effective_ratio: i32,

    // VPS oversampling buffers
    vps_os_2x_left: Oversampling<2>,
    vps_os_2x_right: Oversampling<2>,
    vps_os_4x_left: Oversampling<4>,
    vps_os_4x_right: Oversampling<4>,
    vps_os_8x_left: Oversampling<8>,
    vps_os_8x_right: Oversampling<8>,
    vps_os_16x_left: Oversampling<16>,
    vps_os_16x_right: Oversampling<16>,
    vps_os_32x_left: Oversampling<32>,
    vps_os_32x_right: Oversampling<32>,
    vps_os_64x_left: Oversampling<64>,
    vps_os_64x_right: Oversampling<64>,
    vps_os_128x_left: Oversampling<128>,
    vps_os_128x_right: Oversampling<128>,

    // ===== Stereo =====
    pll_damping_stereo_offset: f64,

    // ===== VPS Oscillator =====
    vps_octave: i32,
    vps_tune: i32,
    vps_fine: f64,
    vps_fold: f64,
    target_vps_fold: f64,
    vps_d_param: f64,
    vps_v_param: f64,
    vps_volume: f64,
    vps_stereo_v_offset: f64,
    vps_stereo_d_offset: f64,
    vps_shape_type: i32,
    vps_shape_amount: f64,
    vps_phase_mode: VpsPhaseMode,
    prev_ref_phase: f64,

    // ===== Sub Oscillator =====
    sub_volume: f64,

    // ===== Saw Oscillator =====
    saw_enabled: bool,
    saw_octave: i32,
    saw_tune: i32,
    saw_volume: f64,
    saw_fold: f64,
    saw_shape_type: i32,
    saw_shape_amount: f64,
    saw_fold_range: i32,
    saw_tight: f64,
    target_saw_tight: f64,
    saw_tight_slew: OnePoleSlewValue,
    saw_tight_filter: SawTightFilter,

    // ===== PLL =====
    pll_volume: f64,
    pll_track_speed: f64,
    pll_damping: f64,
    pll_multiplier: f64,
    pll_feedback_amount: f64,
    pll_feedback_state: f64,
    pll_mode_is_edge: bool,
    pll_colored: bool,

    // ===== PLL FM =====
    pll_fm_amount: f64,
    pll_fm_ratio_float: f64,
    pll_fm_expand: bool,

    // ===== PLL Reference =====
    pll_ref_octave: i32,
    pll_ref_tune: i32,
    pll_ref_fine: f64,
    pll_ref_pulse_width: f64,

    // ===== PLL Experimental =====
    pll_retrigger: f64,
    pll_burst_threshold: f64,
    pll_burst_amount: f64,
    pll_loop_saturation: f64,
    pll_color_amount: f64,
    pll_edge_sensitivity: f64,
    pll_range: f64,
    pll_stereo_track_offset: f64,
    pll_stereo_phase: f64,
    pll_fm_env_amount: f64,
    pll_precision: bool,
    pll_anti_alias: bool,
    pll_injection_amount: f64,
    pll_injection_x4: bool,
    pll_prev_out_l: f64,
    pll_prev_out_r: f64,

    // ===== Coloration =====
    ring_mod_amount: f64,
    wavefold_amount: f64,
    drift_amount: f64,
    drift_rate: f64,
    drift_phase_l: f64,
    drift_phase_r: f64,
    tube_drive: f64,

    // ===== Volume Envelope =====
    vol_env_attack: f64,
    vol_env_attack_shape: f64,
    vol_env_decay: f64,
    vol_env_decay_shape: f64,
    vol_env_sustain: f64,
    vol_env_release: f64,
    vol_env_release_shape: f64,
    velocity: f64,

    // ===== Slew Limiters =====
    freq_slew: SlewValue<f64>,
    pll_volume_slew: SlewValue<f64>,
    pll_track_slew: SlewValue<f64>,
    pll_damping_slew: SlewValue<f64>,
    pll_influence_slew: SlewValue<f64>,
    pll_feedback_slew: SlewValue<f64>,
    pll_pulse_width_slew: SlewValue<f64>,
    pll_stereo_offset_slew: SlewValue<f64>,
    pll_fm_amount_slew: SlewValue<f64>,
    pll_injection_amount_slew: SlewValue<f64>,
    pll_burst_threshold_slew: SlewValue<f64>,
    pll_burst_amount_slew: SlewValue<f64>,
    pll_color_amount_slew: SlewValue<f64>,
    pll_stereo_track_slew: SlewValue<f64>,
    pll_stereo_phase_slew: SlewValue<f64>,
    pll_fm_env_slew: SlewValue<f64>,
    pll_range_slew: SlewValue<f64>,

    // Coloration slews
    ring_mod_slew: SlewValue<f64>,
    wavefold_slew: SlewValue<f64>,
    drift_amount_slew: SlewValue<f64>,
    drift_rate_slew: SlewValue<f64>,
    tube_drive_slew: SlewValue<f64>,

    // VPS slew limiters
    vps_d_slew: SlewValue<f64>,
    vps_v_slew: SlewValue<f64>,
    vps_volume_slew: SlewValue<f64>,
    vps_stereo_offset_slew: SlewValue<f64>,
    vps_fold_slew: OnePoleSlewValue,
    vps_stereo_d_offset_slew: SlewValue<f64>,
    vps_shape_amount_slew: OnePoleSlewValue,

    // Saw slew limiters
    saw_volume_slew: SlewValue<f64>,
    saw_fold_slew: OnePoleSlewValue,
    saw_shape_amount_slew: OnePoleSlewValue,

    // Sub slew
    sub_volume_slew: SlewValue<f64>,

    // Velocity slew for click-free velocity changes
    velocity_slew: SlewValue<f64>,

    // Master volume slew for click-free volume changes
    master_volume_slew: SlewValue<f64>,
    target_master_volume: f64,

    // ===== Target Values =====
    glide_time_ms: f64,
    legato_mode: bool,
    legato_velocity_lock: bool,
    vca_mode: bool,
    target_frequency: f64,
    target_pll_volume: f64,
    target_pll_track: f64,
    target_pll_damping: f64,
    target_pll_influence: f64,
    target_pll_feedback: f64,
    target_pll_pulse_width: f64,
    target_pll_stereo_offset: f64,
    target_pll_fm_amount: f64,
    target_pll_injection_amount: f64,
    target_pll_burst_threshold: f64,
    target_pll_burst_amount: f64,
    target_pll_color_amount: f64,
    target_pll_stereo_track_offset: f64,
    target_pll_stereo_phase: f64,
    target_pll_fm_env_amount: f64,
    target_pll_range: f64,
    target_ring_mod: f64,
    target_wavefold: f64,
    target_drift_amount: f64,
    target_drift_rate: f64,
    target_tube_drive: f64,
    target_vps_d: f64,
    target_vps_v: f64,
    target_vps_volume: f64,
    target_vps_stereo_offset: f64,
    target_vps_stereo_d_offset: f64,
    target_vps_shape_amount: f64,
    target_sub_volume: f64,
    target_saw_volume: f64,
    target_saw_fold: f64,
    target_saw_shape_amount: f64,
    target_velocity: f64,

    // ===== Modulation Offsets (applied per-sample from LFOs) =====
    mod_pll_damping: f64,
    mod_pll_influence: f64,
    mod_pll_track_speed: f64,
    mod_pll_feedback: f64,
    mod_pll_fm_amount: f64,
    mod_pll_injection_amount: f64,
    mod_pll_mult_slew: f64,
    mod_pll_pulse_width: f64,
    mod_pll_stereo_phase: f64,
    mod_pll_fm_env_amount: f64,
    mod_pll_burst_amount: f64,
    mod_pll_range: f64,
    mod_vps_d: f64,
    mod_vps_v: f64,
    mod_ring_mod: f64,
    mod_wavefold: f64,
    mod_drift_amount: f64,
    mod_tube_drive: f64,
    mod_pll_volume: f64,
    mod_vps_volume: f64,
    mod_sub_volume: f64,
    mod_pll_mult: f64,
    mod_pll_mult_direct: f64,
    mod_vps_shape_amount: f64,
    mod_vps_stereo_d_offset: f64,
    mod_vps_fold: f64,
    mod_vps_stereo_v_offset: f64,
    mod_saw_fold: f64,
    mod_saw_shape_amount: f64,
    mod_saw_volume: f64,

    // Slews for modulation (critical for smooth, crackle-free modulation)
    mod_slew_pll_damping: SlewValue<f64>,
    mod_slew_pll_influence: SlewValue<f64>,
    mod_slew_pll_track: SlewValue<f64>,
    mod_slew_pll_feedback: SlewValue<f64>,
    mod_slew_pll_fm: SlewValue<f64>,
    mod_slew_pll_injection: SlewValue<f64>,
    mod_slew_pll_mult_slew: SlewValue<f64>,
    mod_slew_pll_pw: SlewValue<f64>,
    mod_slew_pll_stereo_phase: SlewValue<f64>,
    mod_slew_pll_fm_env: SlewValue<f64>,
    mod_slew_pll_burst: SlewValue<f64>,
    mod_slew_pll_range: SlewValue<f64>,
    mod_slew_vps_d: SlewValue<f64>,
    mod_slew_vps_v: SlewValue<f64>,
    mod_slew_ring_mod: SlewValue<f64>,
    mod_slew_wavefold: SlewValue<f64>,
    mod_slew_drift: SlewValue<f64>,
    mod_slew_tube: SlewValue<f64>,
    mod_slew_pll_vol: SlewValue<f64>,
    mod_slew_vps_vol: SlewValue<f64>,
    mod_slew_sub_vol: SlewValue<f64>,
    mod_slew_pll_mult: SlewValue<f64>,
    mod_slew_pll_mult_direct: SlewValue<f64>,
    mod_slew_vps_shape_amount: SlewValue<f64>,
    mod_slew_vps_stereo_d_offset: SlewValue<f64>,
    mod_slew_vps_fold: SlewValue<f64>,
    mod_slew_vps_stereo_v_offset: SlewValue<f64>,
    mod_slew_saw_fold: SlewValue<f64>,
    mod_slew_saw_shape_amount: SlewValue<f64>,
    mod_slew_saw_volume: SlewValue<f64>,

    pll_mult_slew_time: f64,
    bpm: f64,
    target_pll_multiplier: f64,
    pll_mult_slew_state: SlewValue<f64>,
}

impl Voice {
    pub fn new(sample_rate: f32) -> Self {
        let mut oversampling_2x_left = Oversampling::<2>::new();
        let mut oversampling_2x_right = Oversampling::<2>::new();
        oversampling_2x_left.set_sample_rate(sample_rate);
        oversampling_2x_right.set_sample_rate(sample_rate);

        let mut oversampling_4x_left = Oversampling::<4>::new();
        let mut oversampling_4x_right = Oversampling::<4>::new();
        oversampling_4x_left.set_sample_rate(sample_rate);
        oversampling_4x_right.set_sample_rate(sample_rate);

        let mut oversampling_8x_left = Oversampling::<8>::new();
        let mut oversampling_8x_right = Oversampling::<8>::new();
        oversampling_8x_left.set_sample_rate(sample_rate);
        oversampling_8x_right.set_sample_rate(sample_rate);

        let mut oversampling_16x_left = Oversampling::<16>::new();
        let mut oversampling_16x_right = Oversampling::<16>::new();
        oversampling_16x_left.set_sample_rate(sample_rate);
        oversampling_16x_right.set_sample_rate(sample_rate);

        let mut oversampling_32x_left = Oversampling::<32>::new();
        let mut oversampling_32x_right = Oversampling::<32>::new();
        oversampling_32x_left.set_sample_rate(sample_rate);
        oversampling_32x_right.set_sample_rate(sample_rate);

        let mut oversampling_64x_left = Oversampling::<64>::new();
        let mut oversampling_64x_right = Oversampling::<64>::new();
        oversampling_64x_left.set_sample_rate(sample_rate);
        oversampling_64x_right.set_sample_rate(sample_rate);

        let mut oversampling_128x_left = Oversampling::<128>::new();
        let mut oversampling_128x_right = Oversampling::<128>::new();
        oversampling_128x_left.set_sample_rate(sample_rate);
        oversampling_128x_right.set_sample_rate(sample_rate);

        let mut saw_os_2x = Oversampling::<2>::new();
        let mut saw_os_4x = Oversampling::<4>::new();
        let mut saw_os_8x = Oversampling::<8>::new();
        let mut saw_os_16x = Oversampling::<16>::new();
        let mut saw_os_32x = Oversampling::<32>::new();
        let mut saw_os_64x = Oversampling::<64>::new();
        let mut saw_os_128x = Oversampling::<128>::new();
        saw_os_2x.set_sample_rate(sample_rate);
        saw_os_4x.set_sample_rate(sample_rate);
        saw_os_8x.set_sample_rate(sample_rate);
        saw_os_16x.set_sample_rate(sample_rate);
        saw_os_32x.set_sample_rate(sample_rate);
        saw_os_64x.set_sample_rate(sample_rate);
        saw_os_128x.set_sample_rate(sample_rate);

        let mut vps_os_2x_left = Oversampling::<2>::new();
        let mut vps_os_2x_right = Oversampling::<2>::new();
        let mut vps_os_4x_left = Oversampling::<4>::new();
        let mut vps_os_4x_right = Oversampling::<4>::new();
        let mut vps_os_8x_left = Oversampling::<8>::new();
        let mut vps_os_8x_right = Oversampling::<8>::new();
        let mut vps_os_16x_left = Oversampling::<16>::new();
        let mut vps_os_16x_right = Oversampling::<16>::new();
        let mut vps_os_32x_left = Oversampling::<32>::new();
        let mut vps_os_32x_right = Oversampling::<32>::new();
        let mut vps_os_64x_left = Oversampling::<64>::new();
        let mut vps_os_64x_right = Oversampling::<64>::new();
        let mut vps_os_128x_left = Oversampling::<128>::new();
        let mut vps_os_128x_right = Oversampling::<128>::new();
        vps_os_2x_left.set_sample_rate(sample_rate);
        vps_os_2x_right.set_sample_rate(sample_rate);
        vps_os_4x_left.set_sample_rate(sample_rate);
        vps_os_4x_right.set_sample_rate(sample_rate);
        vps_os_8x_left.set_sample_rate(sample_rate);
        vps_os_8x_right.set_sample_rate(sample_rate);
        vps_os_16x_left.set_sample_rate(sample_rate);
        vps_os_16x_right.set_sample_rate(sample_rate);
        vps_os_32x_left.set_sample_rate(sample_rate);
        vps_os_32x_right.set_sample_rate(sample_rate);
        vps_os_64x_left.set_sample_rate(sample_rate);
        vps_os_64x_right.set_sample_rate(sample_rate);
        vps_os_128x_left.set_sample_rate(sample_rate);
        vps_os_128x_right.set_sample_rate(sample_rate);

        let sample_rate_f64 = sample_rate as f64;
        // Default to 1x (no oversampling) - processing at DAW rate
        let processing_rate = sample_rate_f64;

        let make_slew = || {
            let mut s = SlewValue::new();
            s.set_sample_rate(sample_rate_f64);
            s
        };
        let make_one_pole_slew = || {
            let mut s = OnePoleSlewValue::new();
            s.set_sample_rate(sample_rate_f64);
            s
        };

        Self {
            vps_oscillator_left: Oscillator::new(sample_rate_f64),
            vps_oscillator_right: Oscillator::new(sample_rate_f64),
            sub_oscillator: SineOscillator::new(sample_rate_f64),
            pll_oscillator_left: PLLOscillator::new(processing_rate),
            pll_oscillator_right: PLLOscillator::new(processing_rate),
            pll_reference_oscillator: PolyBlepWrapper::new(processing_rate),
            fm_oscillator: SineOscillator::new(processing_rate),
            saw_oscillator: SawOscillator::new(sample_rate_f64),

            volume_envelope: Envelope::new(sample_rate_f64),
            oversampling_2x_left,
            oversampling_2x_right,
            oversampling_4x_left,
            oversampling_4x_right,
            oversampling_8x_left,
            oversampling_8x_right,
            oversampling_16x_left,
            oversampling_16x_right,
            oversampling_32x_left,
            oversampling_32x_right,
            oversampling_64x_left,
            oversampling_64x_right,
            oversampling_128x_left,
            oversampling_128x_right,
            saw_os_2x,
            saw_os_4x,
            saw_os_8x,
            saw_os_16x,
            saw_os_32x,
            saw_os_64x,
            saw_os_128x,

            base_frequency: 220.0,
            master_volume: 0.8,
            daw_sample_rate: sample_rate_f64,
            processing_sample_rate: processing_rate,
            base_rate_option: 0,  // Auto (use DAW rate)

            // Bypass switches (all enabled by default)
            pll_enabled: true,
            vps_enabled: true,
            coloration_enabled: true,
            current_pll_os_factor: 0,
            current_saw_os_factor: 0,
            current_vps_os_factor: 0,
            pll_effective_ratio: 1,
            saw_effective_ratio: 1,
            vps_effective_ratio: 1,
            vps_os_2x_left,
            vps_os_2x_right,
            vps_os_4x_left,
            vps_os_4x_right,
            vps_os_8x_left,
            vps_os_8x_right,
            vps_os_16x_left,
            vps_os_16x_right,
            vps_os_32x_left,
            vps_os_32x_right,
            vps_os_64x_left,
            vps_os_64x_right,
            vps_os_128x_left,
            vps_os_128x_right,

            pll_damping_stereo_offset: 0.0,

            vps_octave: 0,
            vps_tune: 0,
            vps_fine: 0.0,
            vps_fold: 0.0,
            target_vps_fold: 0.0,
            vps_d_param: 0.5,
            vps_v_param: 0.5,
            vps_volume: 1.0,
            vps_stereo_v_offset: 0.0,
            vps_stereo_d_offset: 0.0,
            vps_shape_type: 0,
            vps_shape_amount: 0.0,
            vps_phase_mode: VpsPhaseMode::Free,
            prev_ref_phase: 0.0,

            sub_volume: 0.0,

            saw_enabled: false,
            saw_octave: 0,
            saw_tune: 0,
            saw_volume: 0.0,
            saw_fold: 0.0,
            saw_shape_type: 0,
            saw_shape_amount: 0.0,
            saw_fold_range: 0,
            saw_tight: 0.0,
            target_saw_tight: 0.0,
            saw_tight_slew: make_one_pole_slew(),
            saw_tight_filter: SawTightFilter::new(sample_rate_f64),

            pll_volume: 0.0,
            pll_track_speed: 0.5,
            pll_damping: 0.3,
            pll_multiplier: 1.0,
            pll_feedback_amount: 0.0,
            pll_feedback_state: 0.0,
            pll_mode_is_edge: false,
            pll_colored: false,

            pll_fm_amount: 0.0,
            pll_fm_ratio_float: 1.0,
            pll_fm_expand: false,

            pll_ref_octave: 0,
            pll_ref_tune: 0,
            pll_ref_fine: 0.0,
            pll_ref_pulse_width: 0.5,

            // PLL Experimental defaults
            pll_retrigger: 0.05,
            pll_burst_threshold: 0.7,
            pll_burst_amount: 3.3,
            pll_loop_saturation: 100.0,
            pll_color_amount: 0.25,
            pll_edge_sensitivity: 0.02,
            pll_range: 1.0,
            pll_stereo_track_offset: 0.0,
            pll_stereo_phase: 0.0,
            pll_fm_env_amount: 0.0,
            pll_precision: true,
            pll_anti_alias: true,
            pll_injection_amount: 0.0,
            pll_injection_x4: false,
            pll_prev_out_l: 0.0,
            pll_prev_out_r: 0.0,

            // Coloration defaults
            ring_mod_amount: 0.0,
            wavefold_amount: 0.0,
            drift_amount: 0.0,
            drift_rate: 0.3,
            drift_phase_l: 0.0,
            drift_phase_r: 0.33,
            tube_drive: 0.0,

            vol_env_attack: 1.0,
            vol_env_attack_shape: 0.5,
            vol_env_decay: 20.0,
            vol_env_decay_shape: 0.5,
            vol_env_sustain: 1.0,
            vol_env_release: 5.0,
            vol_env_release_shape: 0.5,
            velocity: 1.0,

            freq_slew: make_slew(),
            pll_volume_slew: make_slew(),
            pll_track_slew: make_slew(),
            pll_damping_slew: make_slew(),
            pll_influence_slew: make_slew(),
            pll_feedback_slew: make_slew(),
            pll_pulse_width_slew: make_slew(),
            pll_stereo_offset_slew: make_slew(),
            pll_fm_amount_slew: make_slew(),
            pll_injection_amount_slew: make_slew(),
            pll_burst_threshold_slew: make_slew(),
            pll_burst_amount_slew: make_slew(),
            pll_color_amount_slew: make_slew(),
            pll_stereo_track_slew: make_slew(),
            pll_stereo_phase_slew: make_slew(),
            pll_fm_env_slew: make_slew(),
            pll_range_slew: make_slew(),
            ring_mod_slew: make_slew(),
            wavefold_slew: make_slew(),
            drift_amount_slew: make_slew(),
            drift_rate_slew: make_slew(),
            tube_drive_slew: make_slew(),
            vps_d_slew: make_slew(),
            vps_v_slew: make_slew(),
            vps_volume_slew: make_slew(),
            vps_stereo_offset_slew: make_slew(),
            vps_fold_slew: make_one_pole_slew(),
            vps_stereo_d_offset_slew: make_slew(),
            vps_shape_amount_slew: make_one_pole_slew(),
            saw_volume_slew: make_slew(),
            saw_fold_slew: make_one_pole_slew(),
            saw_shape_amount_slew: make_one_pole_slew(),
            sub_volume_slew: make_slew(),
            velocity_slew: make_slew(),
            master_volume_slew: make_slew(),
            target_master_volume: 0.8,

            glide_time_ms: 0.0,
            legato_mode: false,
            legato_velocity_lock: false,
            vca_mode: false,
            target_frequency: 220.0,
            target_pll_volume: 0.0,
            target_pll_track: 0.5,
            target_pll_damping: 0.3,
            target_pll_influence: 0.5,
            target_pll_feedback: 0.0,
            target_pll_pulse_width: 0.5,
            target_pll_stereo_offset: 0.0,
            target_pll_fm_amount: 0.0,
            target_pll_injection_amount: 0.0,
            target_pll_burst_threshold: 0.7,
            target_pll_burst_amount: 3.3,
            target_pll_color_amount: 0.25,
            target_pll_stereo_track_offset: 0.0,
            target_pll_stereo_phase: 0.0,
            target_pll_fm_env_amount: 0.0,
            target_pll_range: 1.0,
            target_ring_mod: 0.0,
            target_wavefold: 0.0,
            target_drift_amount: 0.0,
            target_drift_rate: 0.3,
            target_tube_drive: 0.0,
            target_vps_d: 0.5,
            target_vps_v: 0.5,
            target_vps_volume: 1.0,
            target_vps_stereo_offset: 0.0,
            target_vps_stereo_d_offset: 0.0,
            target_vps_shape_amount: 0.0,
            target_sub_volume: 0.0,
            target_saw_volume: 0.0,
            target_saw_fold: 0.0,
            target_saw_shape_amount: 0.0,
            target_velocity: 1.0,

            mod_pll_damping: 0.0,
            mod_pll_influence: 0.0,
            mod_pll_track_speed: 0.0,
            mod_pll_feedback: 0.0,
            mod_pll_fm_amount: 0.0,
            mod_pll_injection_amount: 0.0,
            mod_pll_mult_slew: 0.0,
            mod_pll_pulse_width: 0.0,
            mod_pll_stereo_phase: 0.0,
            mod_pll_fm_env_amount: 0.0,
            mod_pll_burst_amount: 0.0,
            mod_pll_range: 0.0,
            mod_vps_d: 0.0,
            mod_vps_v: 0.0,
            mod_ring_mod: 0.0,
            mod_wavefold: 0.0,
            mod_drift_amount: 0.0,
            mod_tube_drive: 0.0,
            mod_pll_volume: 0.0,
            mod_vps_volume: 0.0,
            mod_sub_volume: 0.0,
            mod_pll_mult: 0.0,
            mod_pll_mult_direct: 0.0,
            mod_vps_shape_amount: 0.0,
            mod_vps_stereo_d_offset: 0.0,
            mod_vps_fold: 0.0,
            mod_vps_stereo_v_offset: 0.0,
            mod_saw_fold: 0.0,
            mod_saw_shape_amount: 0.0,
            mod_saw_volume: 0.0,

            mod_slew_pll_damping: make_slew(),
            mod_slew_pll_influence: make_slew(),
            mod_slew_pll_track: make_slew(),
            mod_slew_pll_feedback: make_slew(),
            mod_slew_pll_fm: make_slew(),
            mod_slew_pll_injection: make_slew(),
            mod_slew_pll_mult_slew: make_slew(),
            mod_slew_pll_pw: make_slew(),
            mod_slew_pll_stereo_phase: make_slew(),
            mod_slew_pll_fm_env: make_slew(),
            mod_slew_pll_burst: make_slew(),
            mod_slew_pll_range: make_slew(),
            mod_slew_vps_d: make_slew(),
            mod_slew_vps_v: make_slew(),
            mod_slew_ring_mod: make_slew(),
            mod_slew_wavefold: make_slew(),
            mod_slew_drift: make_slew(),
            mod_slew_tube: make_slew(),
            mod_slew_pll_vol: make_slew(),
            mod_slew_vps_vol: make_slew(),
            mod_slew_sub_vol: make_slew(),
            mod_slew_pll_mult: make_slew(),
            mod_slew_pll_mult_direct: make_slew(),
            mod_slew_vps_shape_amount: make_slew(),
            mod_slew_vps_stereo_d_offset: make_slew(),
            mod_slew_vps_fold: make_slew(),
            mod_slew_vps_stereo_v_offset: make_slew(),
            mod_slew_saw_fold: make_slew(),
            mod_slew_saw_shape_amount: make_slew(),
            mod_slew_saw_volume: make_slew(),

            pll_mult_slew_time: 0.15,
            bpm: 120.0,
            target_pll_multiplier: 1.0,
            pll_mult_slew_state: make_slew(),
        }
    }

    // Bypass switch setters
    pub fn set_bypass_switches(
        &mut self,
        pll: bool,
        vps: bool,
        coloration: bool,
        _reverb: bool,
        saw: bool,
    ) {
        self.pll_enabled = pll;
        self.vps_enabled = vps;
        self.saw_enabled = saw;
        self.coloration_enabled = coloration;
    }

    pub fn set_oversampling(&mut self, pll: i32, saw: i32, vps: i32) {
        let mut changed = false;
        if pll != self.current_pll_os_factor {
            self.current_pll_os_factor = pll;
            changed = true;
        }
        if saw != self.current_saw_os_factor {
            self.current_saw_os_factor = saw;
            changed = true;
        }
        if vps != self.current_vps_os_factor {
            self.current_vps_os_factor = vps;
            changed = true;
        }
        if changed {
            self.update_processing_sample_rate();
        }
    }

    pub fn set_base_rate(&mut self, rate_option: i32) {
        if rate_option != self.base_rate_option {
            self.base_rate_option = rate_option;
            self.update_processing_sample_rate();
        }
    }

    #[allow(dead_code)]
    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        let new_rate = sample_rate as f64;
        if (new_rate - self.daw_sample_rate).abs() > 0.1 {
            self.daw_sample_rate = new_rate;

            // Update oversampling objects with new DAW rate
            self.oversampling_2x_left.set_sample_rate(sample_rate);
            self.oversampling_2x_right.set_sample_rate(sample_rate);
            self.oversampling_4x_left.set_sample_rate(sample_rate);
            self.oversampling_4x_right.set_sample_rate(sample_rate);
            self.oversampling_8x_left.set_sample_rate(sample_rate);
            self.oversampling_8x_right.set_sample_rate(sample_rate);
            self.oversampling_16x_left.set_sample_rate(sample_rate);
            self.oversampling_16x_right.set_sample_rate(sample_rate);
            self.oversampling_32x_left.set_sample_rate(sample_rate);
            self.oversampling_32x_right.set_sample_rate(sample_rate);
            self.oversampling_64x_left.set_sample_rate(sample_rate);
            self.oversampling_64x_right.set_sample_rate(sample_rate);
            self.oversampling_128x_left.set_sample_rate(sample_rate);
            self.oversampling_128x_right.set_sample_rate(sample_rate);
            self.saw_os_2x.set_sample_rate(sample_rate);
            self.saw_os_4x.set_sample_rate(sample_rate);
            self.saw_os_8x.set_sample_rate(sample_rate);
            self.saw_os_16x.set_sample_rate(sample_rate);
            self.saw_os_32x.set_sample_rate(sample_rate);
            self.saw_os_64x.set_sample_rate(sample_rate);
            self.saw_os_128x.set_sample_rate(sample_rate);
            self.vps_os_2x_left.set_sample_rate(sample_rate);
            self.vps_os_2x_right.set_sample_rate(sample_rate);
            self.vps_os_4x_left.set_sample_rate(sample_rate);
            self.vps_os_4x_right.set_sample_rate(sample_rate);
            self.vps_os_8x_left.set_sample_rate(sample_rate);
            self.vps_os_8x_right.set_sample_rate(sample_rate);
            self.vps_os_16x_left.set_sample_rate(sample_rate);
            self.vps_os_16x_right.set_sample_rate(sample_rate);
            self.vps_os_32x_left.set_sample_rate(sample_rate);
            self.vps_os_32x_right.set_sample_rate(sample_rate);
            self.vps_os_64x_left.set_sample_rate(sample_rate);
            self.vps_os_64x_right.set_sample_rate(sample_rate);
            self.vps_os_128x_left.set_sample_rate(sample_rate);
            self.vps_os_128x_right.set_sample_rate(sample_rate);
            // Update envelopes with DAW rate (they run at DAW rate, not oversampled)
            self.volume_envelope.set_sample_rate(new_rate);

            // Update all slew limiters
            let update_slew = |s: &mut SlewValue<f64>| s.set_sample_rate(new_rate);
            update_slew(&mut self.freq_slew);
            update_slew(&mut self.pll_volume_slew);
            update_slew(&mut self.pll_track_slew);
            update_slew(&mut self.pll_damping_slew);
            update_slew(&mut self.pll_influence_slew);
            update_slew(&mut self.pll_feedback_slew);
            update_slew(&mut self.pll_pulse_width_slew);
            update_slew(&mut self.pll_stereo_offset_slew);
            update_slew(&mut self.pll_fm_amount_slew);
            update_slew(&mut self.pll_injection_amount_slew);
            update_slew(&mut self.pll_burst_threshold_slew);
            update_slew(&mut self.pll_burst_amount_slew);
            update_slew(&mut self.pll_color_amount_slew);
            update_slew(&mut self.pll_stereo_track_slew);
            update_slew(&mut self.pll_stereo_phase_slew);
            update_slew(&mut self.pll_fm_env_slew);
            update_slew(&mut self.ring_mod_slew);
            update_slew(&mut self.wavefold_slew);
            update_slew(&mut self.drift_amount_slew);
            update_slew(&mut self.drift_rate_slew);
            update_slew(&mut self.tube_drive_slew);
            update_slew(&mut self.pll_range_slew);
            update_slew(&mut self.vps_d_slew);
            update_slew(&mut self.vps_v_slew);
            update_slew(&mut self.vps_volume_slew);
            update_slew(&mut self.vps_stereo_offset_slew);
            self.vps_fold_slew.set_sample_rate(new_rate);
            update_slew(&mut self.vps_stereo_d_offset_slew);
            self.vps_shape_amount_slew.set_sample_rate(new_rate);
            update_slew(&mut self.sub_volume_slew);
            update_slew(&mut self.velocity_slew);
            update_slew(&mut self.master_volume_slew);
            update_slew(&mut self.mod_slew_pll_damping);
            update_slew(&mut self.mod_slew_pll_influence);
            update_slew(&mut self.mod_slew_pll_track);
            update_slew(&mut self.mod_slew_pll_feedback);
            update_slew(&mut self.mod_slew_pll_fm);
            update_slew(&mut self.mod_slew_pll_pw);
            update_slew(&mut self.mod_slew_pll_stereo_phase);
            update_slew(&mut self.mod_slew_pll_fm_env);
            update_slew(&mut self.mod_slew_pll_burst);
            update_slew(&mut self.mod_slew_pll_range);
            update_slew(&mut self.mod_slew_vps_d);
            update_slew(&mut self.mod_slew_vps_v);
            update_slew(&mut self.mod_slew_ring_mod);
            update_slew(&mut self.mod_slew_wavefold);
            update_slew(&mut self.mod_slew_drift);
            update_slew(&mut self.mod_slew_tube);
            update_slew(&mut self.mod_slew_pll_vol);
            update_slew(&mut self.mod_slew_vps_vol);
            update_slew(&mut self.mod_slew_sub_vol);
            update_slew(&mut self.mod_slew_pll_mult);
            update_slew(&mut self.mod_slew_pll_mult_direct);
            update_slew(&mut self.mod_slew_vps_shape_amount);
            update_slew(&mut self.mod_slew_vps_stereo_d_offset);
            update_slew(&mut self.mod_slew_vps_fold);
            update_slew(&mut self.mod_slew_vps_stereo_v_offset);
            update_slew(&mut self.saw_volume_slew);
            self.saw_fold_slew.set_sample_rate(new_rate);
            self.saw_shape_amount_slew.set_sample_rate(new_rate);
            update_slew(&mut self.mod_slew_saw_fold);
            update_slew(&mut self.mod_slew_saw_shape_amount);
            update_slew(&mut self.mod_slew_saw_volume);
            self.saw_tight_slew.set_sample_rate(new_rate);
            self.saw_tight_filter.set_sample_rate(new_rate);
            self.pll_mult_slew_state.set_sample_rate(new_rate);

            // Now update processing sample rate (oscillators, filters, etc.)
            self.update_processing_sample_rate();
        }
    }

    fn os_factor_to_mult(factor: i32) -> i32 {
        match factor { 0 => 1, 1 => 2, 2 => 4, 3 => 8, 4 => 16, 5 => 32, 6 => 64, _ => 128 }
    }

    fn compute_effective_ratio(processing_rate: f64, daw_rate: f64) -> i32 {
        let ratio = (processing_rate / daw_rate).round() as i32;
        match ratio {
            r if r <= 1 => 1,
            r if r <= 2 => 2,
            r if r <= 4 => 4,
            r if r <= 8 => 8,
            r if r <= 16 => 16,
            r if r <= 32 => 32,
            r if r <= 64 => 64,
            _ => 128,
        }
    }

    fn update_processing_sample_rate(&mut self) {
        let base_rate = match self.base_rate_option {
            1 => 44100.0,
            2 => 88200.0,
            3 => 96000.0,
            4 => 192000.0,
            _ => self.daw_sample_rate,
        };

        let pll_rate = base_rate * Self::os_factor_to_mult(self.current_pll_os_factor) as f64;
        let saw_rate = base_rate * Self::os_factor_to_mult(self.current_saw_os_factor) as f64;
        let vps_rate = base_rate * Self::os_factor_to_mult(self.current_vps_os_factor) as f64;

        self.processing_sample_rate = pll_rate;
        self.pll_effective_ratio = Self::compute_effective_ratio(pll_rate, self.daw_sample_rate);
        self.saw_effective_ratio = Self::compute_effective_ratio(saw_rate, self.daw_sample_rate);
        self.vps_effective_ratio = Self::compute_effective_ratio(vps_rate, self.daw_sample_rate);

        self.pll_oscillator_left.set_sample_rate(pll_rate);
        self.pll_oscillator_right.set_sample_rate(pll_rate);
        self.pll_reference_oscillator.set_sample_rate(pll_rate);
        self.fm_oscillator.set_sample_rate(pll_rate);

        self.saw_oscillator.set_sample_rate(saw_rate);

        self.vps_oscillator_left.set_sample_rate(vps_rate);
        self.vps_oscillator_right.set_sample_rate(vps_rate);

        self.sub_oscillator.set_sample_rate(self.daw_sample_rate);
    }

    pub fn set_pll_stereo_damp_offset(&mut self, offset: f64) {
        self.target_pll_stereo_offset = offset.clamp(0.0, 1.0);
    }

    pub fn set_glide_time(&mut self, time_ms: f64) {
        self.glide_time_ms = time_ms.max(0.0);
    }

    pub fn set_legato_mode(&mut self, enabled: bool) {
        self.legato_mode = enabled;
    }

    pub fn set_legato_velocity_lock(&mut self, enabled: bool) {
        self.legato_velocity_lock = enabled;
    }

    pub fn set_vca_mode(&mut self, enabled: bool) {
        self.vca_mode = enabled;
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

    pub fn set_vps_stereo_d_offset(&mut self, offset: f64) {
        self.target_vps_stereo_d_offset = offset.clamp(0.0, 1.0);
    }

    pub fn set_vps_shape(&mut self, shape_type: i32, amount: f64) {
        self.vps_shape_type = shape_type;
        self.target_vps_shape_amount = amount;
    }

    pub fn set_vps_phase_mode(&mut self, mode: i32) {
        self.vps_phase_mode = match mode {
            1 => VpsPhaseMode::PllSync,
            _ => VpsPhaseMode::Free,
        };
    }


    pub fn set_osc_volume(&mut self, volume: f64) {
        self.target_vps_volume = volume;
    }

    pub fn set_osc_octave(&mut self, octave: i32) {
        self.vps_octave = octave;
    }

    pub fn set_osc_tune(&mut self, tune: i32, fine: f64) {
        self.vps_tune = tune;
        self.vps_fine = fine;
    }

    pub fn set_osc_fold(&mut self, fold: f64) {
        self.target_vps_fold = fold;
    }

    pub fn set_sub_volume(&mut self, volume: f64) {
        self.target_sub_volume = volume;
    }

    pub fn set_saw_volume(&mut self, volume: f64) {
        self.target_saw_volume = volume;
    }

    pub fn set_saw_octave(&mut self, octave: i32) {
        self.saw_octave = octave;
    }

    pub fn set_saw_tune(&mut self, tune: i32) {
        self.saw_tune = tune;
    }

    pub fn set_saw_shape(&mut self, shape_type: i32, shape_amount: f64) {
        self.saw_shape_type = shape_type;
        self.target_saw_shape_amount = shape_amount;
    }

    pub fn set_saw_fold(&mut self, fold: f64) {
        self.target_saw_fold = fold;
    }

    pub fn set_saw_tight(&mut self, tight: f64) {
        self.target_saw_tight = tight;
    }

    pub fn set_saw_fold_range(&mut self, range: i32) {
        self.saw_fold_range = range;
    }

    pub fn set_bpm(&mut self, bpm: f64) {
        self.bpm = bpm.max(1.0);
    }

    pub fn set_pll_ref_params(&mut self, octave: i32, pulse_width: f64) {
        self.pll_ref_octave = octave;
        self.target_pll_pulse_width = pulse_width;
    }

    pub fn set_pll_ref_tune(&mut self, tune: i32, fine: f64) {
        self.pll_ref_tune = tune;
        self.pll_ref_fine = fine;
    }

    pub fn set_pll_params(&mut self, track: f64, damp: f64, mult: f64, influence: f64, colored: bool, edge_mode: bool) {
        self.target_pll_track = track;
        self.target_pll_damping = damp;
        self.target_pll_multiplier = mult;
        self.target_pll_influence = influence;
        self.pll_mode_is_edge = edge_mode;
        self.pll_colored = colored;
    }

    pub fn set_pll_mult_slew_time(&mut self, time: f64) {
        self.pll_mult_slew_time = time;
    }

    pub fn set_pll_volume(&mut self, volume: f64) {
        self.target_pll_volume = volume;
    }

    pub fn set_pll_fm_params(&mut self, amount: f64, ratio_float: f64, expand: bool) {
        self.target_pll_fm_amount = amount;
        self.pll_fm_ratio_float = ratio_float;
        self.pll_fm_expand = expand;
    }

    pub fn set_pll_experimental_params(
        &mut self,
        retrigger: f64,
        burst_threshold: f64,
        burst_amount: f64,
        loop_saturation: f64,
        color_amount: f64,
        edge_sensitivity: f64,
        range: f64,
        stereo_track_offset: f64,
    ) {
        self.pll_retrigger = retrigger;
        self.target_pll_burst_threshold = burst_threshold;
        self.target_pll_burst_amount = burst_amount;
        self.pll_loop_saturation = loop_saturation;
        self.target_pll_color_amount = color_amount;
        self.pll_edge_sensitivity = edge_sensitivity;
        self.target_pll_range = range;
        self.target_pll_stereo_track_offset = stereo_track_offset;
    }

    pub fn set_pll_stereo_phase(&mut self, phase: f64) {
        self.target_pll_stereo_phase = phase;
    }


    pub fn set_pll_fm_env_amount(&mut self, amount: f64) {
        self.target_pll_fm_env_amount = amount;
    }

    pub fn set_pll_precision(&mut self, precision: bool) {
        self.pll_precision = precision;
    }

    pub fn set_pll_advanced_params(
        &mut self,
        anti_alias: bool,
        injection_amount: f64,
        injection_x4: bool,
    ) {
        self.pll_anti_alias = anti_alias;
        self.target_pll_injection_amount = injection_amount;
        self.pll_injection_x4 = injection_x4;
    }

    pub fn set_coloration_params(
        &mut self,
        ring_mod: f64,
        wavefold: f64,
        drift_amount: f64,
        drift_rate: f64,
        tube: f64,
    ) {
        self.target_ring_mod = ring_mod;
        self.target_wavefold = wavefold;
        self.target_drift_amount = drift_amount;
        self.target_drift_rate = drift_rate;
        self.target_tube_drive = tube;
    }

    pub fn apply_modulation(&mut self, mod_values: &ModulationValues) {
        // Apply slew to all modulation values for smooth, crackle-free modulation
        // Using fast slew (5ms) for responsive modulation while avoiding artifacts
        const MOD_SLEW_MS: f64 = 5.0;

        self.mod_pll_damping = self.mod_slew_pll_damping.next(mod_values.pll_damping, MOD_SLEW_MS);
        self.mod_pll_influence = self.mod_slew_pll_influence.next(mod_values.pll_influence, MOD_SLEW_MS);
        self.mod_pll_track_speed = self.mod_slew_pll_track.next(mod_values.pll_track_speed, MOD_SLEW_MS);
        self.mod_pll_fm_amount = self.mod_slew_pll_fm.next(mod_values.pll_fm_amount, MOD_SLEW_MS);
        self.mod_pll_injection_amount = self.mod_slew_pll_injection.next(mod_values.pll_injection_amount, MOD_SLEW_MS);
        self.mod_pll_mult_slew = self.mod_slew_pll_mult_slew.next(mod_values.pll_mult_slew, MOD_SLEW_MS);
        self.mod_pll_burst_amount = self.mod_slew_pll_burst.next(mod_values.pll_burst_amount, MOD_SLEW_MS);
        self.mod_pll_range = self.mod_slew_pll_range.next(mod_values.pll_range, MOD_SLEW_MS);
        self.mod_vps_d = self.mod_slew_vps_d.next(mod_values.vps_d, MOD_SLEW_MS);
        self.mod_vps_v = self.mod_slew_vps_v.next(mod_values.vps_v, MOD_SLEW_MS);
        self.mod_drift_amount = self.mod_slew_drift.next(mod_values.drift_amount, MOD_SLEW_MS);
        self.mod_tube_drive = self.mod_slew_tube.next(mod_values.tube_drive, MOD_SLEW_MS);
        self.mod_pll_volume = self.mod_slew_pll_vol.next(mod_values.pll_volume, MOD_SLEW_MS);
        self.mod_vps_volume = self.mod_slew_vps_vol.next(mod_values.vps_volume, MOD_SLEW_MS);
        self.mod_sub_volume = self.mod_slew_sub_vol.next(mod_values.sub_volume, MOD_SLEW_MS);
        self.mod_pll_mult = self.mod_slew_pll_mult.next(mod_values.pll_mult, MOD_SLEW_MS);
        self.mod_pll_mult_direct = self.mod_slew_pll_mult_direct.next(mod_values.pll_mult_direct, MOD_SLEW_MS);
        self.mod_vps_shape_amount = self.mod_slew_vps_shape_amount.next(mod_values.vps_shape_amount, MOD_SLEW_MS);
        self.mod_vps_stereo_d_offset = self.mod_slew_vps_stereo_d_offset.next(mod_values.vps_stereo_d_offset, MOD_SLEW_MS);
        self.mod_vps_fold = self.mod_slew_vps_fold.next(mod_values.vps_fold, MOD_SLEW_MS);
        self.mod_vps_stereo_v_offset = self.mod_slew_vps_stereo_v_offset.next(mod_values.vps_stereo_v_offset, MOD_SLEW_MS);
        self.mod_saw_fold = self.mod_slew_saw_fold.next(mod_values.saw_fold, MOD_SLEW_MS);
        self.mod_saw_shape_amount = self.mod_slew_saw_shape_amount.next(mod_values.saw_shape_amount, MOD_SLEW_MS);
        self.mod_saw_volume = self.mod_slew_saw_volume.next(mod_values.saw_volume, MOD_SLEW_MS);
    }

    pub fn set_volume(&mut self, volume: f64) {
        self.target_master_volume = volume;
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

    pub fn set_velocity(&mut self, velocity: u8) {
        self.target_velocity = velocity as f64 / 127.0;
    }

    pub fn trigger(&mut self) {
        if self.vca_mode {
            return;
        }

        if self.legato_mode && self.volume_envelope.is_held() {
            if self.legato_velocity_lock {
                self.target_velocity = self.velocity;
            }
            return;
        }

        self.volume_envelope.trigger(
            self.vol_env_attack,
            self.vol_env_attack_shape,
            self.vol_env_decay,
            self.vol_env_decay_shape,
            self.vol_env_sustain,
            self.vol_env_release,
            self.vol_env_release_shape,
        );

        self.reset_oscillator_phases();
    }

    pub fn trigger_articulated(&mut self) {
        self.volume_envelope.restart(
            self.vol_env_attack,
            self.vol_env_attack_shape,
            self.vol_env_decay,
            self.vol_env_decay_shape,
            self.vol_env_sustain,
            self.vol_env_release,
            self.vol_env_release_shape,
        );

        self.reset_oscillator_phases();
    }

    fn reset_oscillator_phases(&mut self) {
        self.pll_oscillator_left.trigger();
        self.pll_oscillator_right.trigger();
        match self.vps_phase_mode {
            VpsPhaseMode::PllSync => {
                self.vps_oscillator_left.hard_reset();
                self.vps_oscillator_right.hard_reset();
            }
            VpsPhaseMode::Free => {
                self.vps_oscillator_left.trigger();
                self.vps_oscillator_right.trigger();
            }
        }
        self.pll_reference_oscillator.reset_phase();
        self.saw_oscillator.trigger();
        self.prev_ref_phase = 0.0;
        self.pll_prev_out_l = 0.0;
        self.pll_prev_out_r = 0.0;
    }

    pub fn release(&mut self) {
        if self.vca_mode {
            return;
        }
        self.volume_envelope.release();
    }

    pub fn stop(&mut self) {
        if self.vca_mode {
            return;
        }
        self.volume_envelope.release();
    }

    pub fn reset(&mut self) {
        self.volume_envelope.force_off();

        self.pll_feedback_state = 0.0;
        self.pll_prev_out_l = 0.0;
        self.pll_prev_out_r = 0.0;
        self.prev_ref_phase = 0.0;

        self.drift_phase_l = 0.0;
        self.drift_phase_r = 0.33;
    }

    pub fn process(&mut self, _pll_feedback: f64) -> (f64, f64, f64) {
        let volume_env = if self.vca_mode {
            1.0
        } else {
            self.volume_envelope.update_params(
                self.vol_env_attack,
                self.vol_env_attack_shape,
                self.vol_env_decay,
                self.vol_env_decay_shape,
                self.vol_env_sustain,
                self.vol_env_release,
                self.vol_env_release_shape,
            );
            self.volume_envelope.next()
        };

        let glide_ms = if self.glide_time_ms > 0.5 { self.glide_time_ms } else { 0.5 };
        let target_log2 = self.target_frequency.max(1.0).log2();
        let slewed_log2 = self.freq_slew.next(target_log2, glide_ms);
        self.base_frequency = (2.0_f64).powf(slewed_log2);

        // PLL slews + modulation
        self.pll_volume = (self.pll_volume_slew.next(self.target_pll_volume, 20.0) + self.mod_pll_volume).clamp(0.0, 1.0);
        self.pll_track_speed = (self.pll_track_slew.next(self.target_pll_track, 20.0) + self.mod_pll_track_speed).clamp(0.0, 1.0);
        self.pll_damping = (self.pll_damping_slew.next(self.target_pll_damping, 20.0) + self.mod_pll_damping).clamp(0.0, 1.0);
        let slewed_influence = (self.pll_influence_slew.next(self.target_pll_influence, 20.0) + self.mod_pll_influence).clamp(0.0, 1.0);
        self.pll_feedback_amount = (self.pll_feedback_slew.next(self.target_pll_feedback, 200.0) + self.mod_pll_feedback).clamp(0.0, 1.0);
        self.pll_ref_pulse_width = (self.pll_pulse_width_slew.next(self.target_pll_pulse_width, 20.0) + self.mod_pll_pulse_width).clamp(0.05, 0.95);
        self.pll_damping_stereo_offset = self.pll_stereo_offset_slew.next(self.target_pll_stereo_offset, 60.0);
        self.pll_fm_amount = (self.pll_fm_amount_slew.next(self.target_pll_fm_amount, 20.0) + self.mod_pll_fm_amount).clamp(0.0, 1.0);
        self.pll_injection_amount = (self.pll_injection_amount_slew.next(self.target_pll_injection_amount, 20.0) + self.mod_pll_injection_amount).clamp(0.0, 1.0);
        self.pll_burst_threshold = self.pll_burst_threshold_slew.next(self.target_pll_burst_threshold, 20.0);
        self.pll_burst_amount = (self.pll_burst_amount_slew.next(self.target_pll_burst_amount, 20.0) + self.mod_pll_burst_amount * 10.0).clamp(0.0, 10.0);
        self.pll_range = (self.pll_range_slew.next(self.target_pll_range, 20.0) + self.mod_pll_range).clamp(0.0, 1.0);
        self.pll_color_amount = self.pll_color_amount_slew.next(self.target_pll_color_amount, 20.0);
        self.pll_stereo_track_offset = self.pll_stereo_track_slew.next(self.target_pll_stereo_track_offset, 60.0);
        self.pll_stereo_phase = (self.pll_stereo_phase_slew.next(self.target_pll_stereo_phase, 60.0) + self.mod_pll_stereo_phase).clamp(0.0, 1.0);
        self.pll_fm_env_amount = (self.pll_fm_env_slew.next(self.target_pll_fm_env_amount, 20.0) + self.mod_pll_fm_env_amount).clamp(0.0, 1.0);

        // Coloration slews + modulation
        self.ring_mod_amount = (self.ring_mod_slew.next(self.target_ring_mod, 20.0) + self.mod_ring_mod).clamp(0.0, 1.0);
        self.wavefold_amount = (self.wavefold_slew.next(self.target_wavefold, 20.0) + self.mod_wavefold).clamp(0.0, 1.0);
        self.drift_amount = (self.drift_amount_slew.next(self.target_drift_amount, 50.0) + self.mod_drift_amount).clamp(0.0, 1.0);
        self.drift_rate = self.drift_rate_slew.next(self.target_drift_rate, 20.0);
        self.tube_drive = (self.tube_drive_slew.next(self.target_tube_drive, 20.0) + self.mod_tube_drive).clamp(0.0, 1.0);

        // VPS slews + modulation
        self.vps_d_param = (self.vps_d_slew.next(self.target_vps_d, 20.0) + self.mod_vps_d).clamp(0.0, 1.0);
        self.vps_v_param = (self.vps_v_slew.next(self.target_vps_v, 20.0) + self.mod_vps_v).clamp(0.0, 1.0);
        self.vps_volume = (self.vps_volume_slew.next(self.target_vps_volume, 20.0) + self.mod_vps_volume).clamp(0.0, 1.0);
        self.vps_stereo_v_offset = (self.vps_stereo_offset_slew.next(self.target_vps_stereo_offset, 20.0) + self.mod_vps_stereo_v_offset * 0.3).clamp(0.0, 0.5);
        self.vps_stereo_d_offset = (self.vps_stereo_d_offset_slew.next(self.target_vps_stereo_d_offset, 20.0) + self.mod_vps_stereo_d_offset * 0.3).clamp(0.0, 0.5);
        self.vps_fold = (self.vps_fold_slew.next(self.target_vps_fold, 50.0) + self.mod_vps_fold).clamp(0.0, 1.0);
        self.vps_shape_amount = (self.vps_shape_amount_slew.next(self.target_vps_shape_amount, 50.0) + self.mod_vps_shape_amount * 0.5).clamp(0.0, 0.5);

        // Saw slews + modulation
        self.saw_volume = (self.saw_volume_slew.next(self.target_saw_volume, 20.0) + self.mod_saw_volume).clamp(0.0, 1.0);
        self.saw_fold = (self.saw_fold_slew.next(self.target_saw_fold, 50.0) + self.mod_saw_fold).clamp(0.0, 1.0);
        self.saw_shape_amount = (self.saw_shape_amount_slew.next(self.target_saw_shape_amount, 50.0) + self.mod_saw_shape_amount).clamp(0.0, 1.0);
        self.saw_tight = self.saw_tight_slew.next(self.target_saw_tight, 50.0).clamp(0.0, 1.0);

        // Sub slew + modulation
        self.sub_volume = (self.sub_volume_slew.next(self.target_sub_volume, 20.0) + self.mod_sub_volume).clamp(0.0, 1.0);

        // PLL multiplier: discrete modulation shifts the step, then quantize
        let mult_steps: [f64; 7] = [1.0, 2.0, 4.0, 8.0, 16.0, 32.0, 64.0];
        let base_index = match self.target_pll_multiplier as i32 {
            1 => 0, 2 => 1, 4 => 2, 8 => 3, 16 => 4, 32 => 5, 64 => 6, _ => 0,
        };
        let mod_offset = (self.mod_pll_mult * 6.0).round() as i32;
        let target_index = (base_index + mod_offset).clamp(0, 6) as usize;
        let target_discrete_mult = mult_steps[target_index];

        let beat_ms = 60000.0 / self.bpm;
        let slew_time = (self.pll_mult_slew_time + self.mod_pll_mult_slew).clamp(0.0, 1.0);
        let slew_beats = 0.0625 + slew_time * slew_time * 7.9375;
        let slew_ms = beat_ms * slew_beats;
        self.pll_multiplier = self.pll_mult_slew_state.next(target_discrete_mult, slew_ms);

        // Direct modulation: continuous offset from LFOs
        self.pll_multiplier = (self.pll_multiplier + self.mod_pll_mult_direct * 63.0).clamp(1.0, 64.0);

        // Velocity slew for click-free note transitions (5ms is fast but smooth)
        self.velocity = self.velocity_slew.next(self.target_velocity, 5.0);

        // Master volume slew for click-free volume changes (20ms)
        self.master_volume = self.master_volume_slew.next(self.target_master_volume, 20.0);

        if !self.vca_mode && !self.volume_envelope.is_active() {
            return (0.0, 0.0, 0.0);
        }

        // ===== PLL OVERSAMPLED BLOCK =====
        let mut pll_sample_l = 0.0_f64;
        let mut pll_sample_r = 0.0_f64;
        let mut ref_phase_wrapped = false;
        let mut feedback = self.pll_feedback_state;

        if self.pll_enabled {
            self.pll_oscillator_left.prepare_block();
            self.pll_oscillator_right.prepare_block();

            let mode = if self.pll_mode_is_edge {
                super::oscillator::PllMode::EdgePFD
            } else {
                super::oscillator::PllMode::AnalogLikePD
            };
            let effective_mult = self.pll_multiplier;
            let use_stereo_pll = self.pll_damping_stereo_offset > 0.0001 || self.pll_stereo_track_offset > 0.0001 || self.pll_stereo_phase > 0.0001;

            let damp_left = (self.pll_damping - self.pll_damping_stereo_offset).clamp(0.001, 1.0);
            let damp_right = if use_stereo_pll {
                (self.pll_damping + self.pll_damping_stereo_offset).clamp(0.001, 1.0)
            } else {
                damp_left
            };
            let track_left = (self.pll_track_speed - self.pll_stereo_track_offset).clamp(0.0, 1.0);
            let track_right = if use_stereo_pll {
                (self.pll_track_speed + self.pll_stereo_track_offset).clamp(0.0, 1.0)
            } else {
                track_left
            };

            self.pll_oscillator_left.set_experimental_params(
                self.pll_retrigger,
                self.pll_burst_threshold,
                self.pll_burst_amount,
                self.pll_loop_saturation,
                self.pll_color_amount,
                self.pll_edge_sensitivity,
                self.pll_range,
            );
            self.pll_oscillator_left.set_precision(self.pll_precision);
            self.pll_oscillator_left.set_anti_alias(self.pll_anti_alias);
            self.pll_oscillator_left.set_injection_amount(self.pll_injection_amount);
            self.pll_oscillator_left.set_injection_mult(self.pll_injection_x4);
            self.pll_oscillator_left.set_params(track_left, damp_left, effective_mult, slewed_influence, self.pll_colored, mode);

            if use_stereo_pll {
                self.pll_oscillator_right.set_experimental_params(
                    self.pll_retrigger,
                    self.pll_burst_threshold,
                    self.pll_burst_amount,
                    self.pll_loop_saturation,
                    self.pll_color_amount,
                    self.pll_edge_sensitivity,
                    self.pll_range,
                );
                self.pll_oscillator_right.set_precision(self.pll_precision);
                self.pll_oscillator_right.set_anti_alias(self.pll_anti_alias);
                self.pll_oscillator_right.set_injection_amount(self.pll_injection_amount);
                self.pll_oscillator_right.set_injection_mult(self.pll_injection_x4);
                self.pll_oscillator_right.set_params(track_right, damp_right, effective_mult, slewed_influence, self.pll_colored, mode);
            }

            let iterations = self.pll_effective_ratio as usize;
            let use_oversampling = self.pll_effective_ratio > 1;

            let (buf_l, buf_r): (&mut [f32], &mut [f32]) = match self.pll_effective_ratio {
                2 => (self.oversampling_2x_left.resample_buffer(), self.oversampling_2x_right.resample_buffer()),
                4 => (self.oversampling_4x_left.resample_buffer(), self.oversampling_4x_right.resample_buffer()),
                8 => (self.oversampling_8x_left.resample_buffer(), self.oversampling_8x_right.resample_buffer()),
                16 => (self.oversampling_16x_left.resample_buffer(), self.oversampling_16x_right.resample_buffer()),
                32 => (self.oversampling_32x_left.resample_buffer(), self.oversampling_32x_right.resample_buffer()),
                64 => (self.oversampling_64x_left.resample_buffer(), self.oversampling_64x_right.resample_buffer()),
                128 => (self.oversampling_128x_left.resample_buffer(), self.oversampling_128x_right.resample_buffer()),
                _ => (self.oversampling_2x_left.resample_buffer(), self.oversampling_2x_right.resample_buffer()),
            };

            let pll_tune_mult = 2.0_f64.powf((self.pll_ref_tune as f64 + self.pll_ref_fine) / 12.0);
            let ref_freq = self.base_frequency * 2.0_f64.powi(self.pll_ref_octave) * pll_tune_mult;
            let fm_ratio = self.pll_fm_ratio_float;
            let effective_fm_amount = if self.pll_fm_expand { self.pll_fm_amount } else { self.pll_fm_amount * 0.2 };
            let fm_env_mod = 1.0;

            for i in 0..iterations {
                let drift_mod_l = if self.drift_amount > 0.001 {
                    self.drift_phase_l += self.drift_rate * 0.00001;
                    if self.drift_phase_l > 1.0 { self.drift_phase_l -= 1.0; }
                    (self.drift_phase_l * std::f64::consts::TAU).sin() * self.drift_amount * 0.02
                } else { 0.0 };
                let drift_mod_r = if self.drift_amount > 0.001 {
                    self.drift_phase_r += self.drift_rate * 0.000011;
                    if self.drift_phase_r > 1.0 { self.drift_phase_r -= 1.0; }
                    (self.drift_phase_r * std::f64::consts::TAU).sin() * self.drift_amount * 0.02
                } else { 0.0 };

                let fm_freq = ref_freq * fm_ratio;
                let fm_signal = self.fm_oscillator.next(fm_freq);
                let fm_index = effective_fm_amount * 4.0 * fm_ratio * fm_env_mod;
                let fm_mod = fm_signal * fm_index * ref_freq;

                let fb_mod = feedback * self.pll_feedback_amount * 5.0;
                let ref_mod_l = ((ref_freq * (1.0 + drift_mod_l) + fm_mod) * (1.0 + fb_mod)).clamp(20.0, self.processing_sample_rate * 2.0);
                let ref_mod_r = ((ref_freq * (1.0 + drift_mod_r) + fm_mod) * (1.0 + fb_mod)).clamp(20.0, self.processing_sample_rate * 2.0);

                self.pll_reference_oscillator.set_frequency(ref_mod_l);
                let ref_pulse = self.pll_reference_oscillator.next(self.pll_ref_pulse_width);
                let ref_phase = self.pll_reference_oscillator.get_phase();

                if ref_phase < self.prev_ref_phase {
                    ref_phase_wrapped = true;
                }
                self.prev_ref_phase = ref_phase;

                let ref_phase_r = (ref_phase + self.pll_stereo_phase) % 1.0;

                let pll_raw_l = self.pll_oscillator_left.next(ref_phase, ref_mod_l, ref_pulse);
                let pll_raw_r = if use_stereo_pll {
                    self.pll_oscillator_right.next(ref_phase_r, ref_mod_r, ref_pulse)
                } else {
                    pll_raw_l
                };

                self.pll_prev_out_l = pll_raw_l;
                self.pll_prev_out_r = pll_raw_r;

                feedback = feedback * 0.9 + (pll_raw_l + pll_raw_r) * 0.05;

                buf_l[i] = pll_raw_l as f32;
                buf_r[i] = if use_stereo_pll { pll_raw_r as f32 } else { pll_raw_l as f32 };
            }

            // Downsample PLL output
            if use_oversampling {
                pll_sample_l = match self.pll_effective_ratio {
                    2 => self.oversampling_2x_left.downsample() as f64,
                    4 => self.oversampling_4x_left.downsample() as f64,
                    8 => self.oversampling_8x_left.downsample() as f64,
                    16 => self.oversampling_16x_left.downsample() as f64,
                    32 => self.oversampling_32x_left.downsample() as f64,
                    64 => self.oversampling_64x_left.downsample() as f64,
                    _ => self.oversampling_128x_left.downsample() as f64,
                };
                pll_sample_r = match self.pll_effective_ratio {
                    2 => self.oversampling_2x_right.downsample() as f64,
                    4 => self.oversampling_4x_right.downsample() as f64,
                    8 => self.oversampling_8x_right.downsample() as f64,
                    16 => self.oversampling_16x_right.downsample() as f64,
                    32 => self.oversampling_32x_right.downsample() as f64,
                    64 => self.oversampling_64x_right.downsample() as f64,
                    _ => self.oversampling_128x_right.downsample() as f64,
                };
            } else {
                pll_sample_l = buf_l[0] as f64;
                pll_sample_r = buf_r[0] as f64;
            }
        }

        self.pll_feedback_state = feedback;

        // ===== VPS OVERSAMPLED =====
        let mut vps_out_l = 0.0_f64;
        let mut vps_out_r = 0.0_f64;

        if self.vps_enabled {
            if ref_phase_wrapped && self.vps_phase_mode == VpsPhaseMode::PllSync {
                self.vps_oscillator_left.sync_reset();
                self.vps_oscillator_right.sync_reset();
            }

            let tune_mult = 2.0_f64.powf((self.vps_tune as f64 + self.vps_fine) / 12.0);
            let base_freq = self.base_frequency * 2.0_f64.powi(self.vps_octave) * tune_mult;

            let use_stereo_v = self.vps_stereo_v_offset > 0.0001;
            let use_stereo_d = self.vps_stereo_d_offset > 0.0001;
            let use_stereo = use_stereo_v || use_stereo_d;

            let v_left = if use_stereo_v { (self.vps_v_param - self.vps_stereo_v_offset).clamp(0.0, 1.0) } else { self.vps_v_param };
            let v_right = if use_stereo_v { (self.vps_v_param + self.vps_stereo_v_offset).clamp(0.0, 1.0) } else { self.vps_v_param };
            let d_left = if use_stereo_d { (self.vps_d_param - self.vps_stereo_d_offset).clamp(0.0, 1.0) } else { self.vps_d_param };
            let d_right = if use_stereo_d { (self.vps_d_param + self.vps_stereo_d_offset).clamp(0.0, 1.0) } else { self.vps_d_param };

            self.vps_oscillator_left.set_frequency(base_freq);
            self.vps_oscillator_right.set_frequency(base_freq);

            let vps_iterations = self.vps_effective_ratio as usize;
            let vps_use_oversampling = self.vps_effective_ratio > 1;
            let dist_type = (self.vps_shape_type + 1) as u8;
            let shape_amt = self.vps_shape_amount as f32;

            let (vbuf_l, vbuf_r): (&mut [f32], &mut [f32]) = match self.vps_effective_ratio {
                2 => (self.vps_os_2x_left.resample_buffer(), self.vps_os_2x_right.resample_buffer()),
                4 => (self.vps_os_4x_left.resample_buffer(), self.vps_os_4x_right.resample_buffer()),
                8 => (self.vps_os_8x_left.resample_buffer(), self.vps_os_8x_right.resample_buffer()),
                16 => (self.vps_os_16x_left.resample_buffer(), self.vps_os_16x_right.resample_buffer()),
                32 => (self.vps_os_32x_left.resample_buffer(), self.vps_os_32x_right.resample_buffer()),
                64 => (self.vps_os_64x_left.resample_buffer(), self.vps_os_64x_right.resample_buffer()),
                128 => (self.vps_os_128x_left.resample_buffer(), self.vps_os_128x_right.resample_buffer()),
                _ => (self.vps_os_2x_left.resample_buffer(), self.vps_os_2x_right.resample_buffer()),
            };

            for i in 0..vps_iterations {
                let raw_l = self.vps_oscillator_left.next(d_left, v_left);
                let raw_r = if use_stereo { self.vps_oscillator_right.next(d_right, v_right) } else { raw_l };

                let (shaped_l, shaped_r) = if self.vps_shape_amount > 0.001 {
                    (
                        apply_distortion(raw_l as f32, shape_amt, dist_type) as f64,
                        apply_distortion(raw_r as f32, shape_amt, dist_type) as f64,
                    )
                } else {
                    (raw_l, raw_r)
                };

                let (folded_l, folded_r) = if self.vps_fold > 0.001 {
                    let folded = stereo_wavefold(stereo(shaped_l, shaped_r), self.vps_fold);
                    (stereo_left(folded), stereo_right(folded))
                } else {
                    (shaped_l, shaped_r)
                };

                vbuf_l[i] = folded_l as f32;
                vbuf_r[i] = folded_r as f32;
            }

            let (ds_l, ds_r) = if vps_use_oversampling {
                let l = match self.vps_effective_ratio {
                    2 => self.vps_os_2x_left.downsample() as f64,
                    4 => self.vps_os_4x_left.downsample() as f64,
                    8 => self.vps_os_8x_left.downsample() as f64,
                    16 => self.vps_os_16x_left.downsample() as f64,
                    32 => self.vps_os_32x_left.downsample() as f64,
                    64 => self.vps_os_64x_left.downsample() as f64,
                    _ => self.vps_os_128x_left.downsample() as f64,
                };
                let r = match self.vps_effective_ratio {
                    2 => self.vps_os_2x_right.downsample() as f64,
                    4 => self.vps_os_4x_right.downsample() as f64,
                    8 => self.vps_os_8x_right.downsample() as f64,
                    16 => self.vps_os_16x_right.downsample() as f64,
                    32 => self.vps_os_32x_right.downsample() as f64,
                    64 => self.vps_os_64x_right.downsample() as f64,
                    _ => self.vps_os_128x_right.downsample() as f64,
                };
                (l, r)
            } else {
                (vbuf_l[0] as f64, vbuf_r[0] as f64)
            };

            vps_out_l = ds_l * self.vps_volume * volume_env;
            vps_out_r = ds_r * self.vps_volume * volume_env;
        }

        // ===== SAW OVERSAMPLED =====
        let mut saw_out = 0.0_f64;
        if self.saw_enabled && self.saw_volume > 0.001 {
            let tune_mult = 2.0_f64.powf(self.saw_tune as f64 / 12.0);
            let saw_freq = self.base_frequency * 2.0_f64.powi(self.saw_octave) * tune_mult;
            self.saw_oscillator.set_frequency(saw_freq);

            let iterations = self.saw_effective_ratio as usize;
            let use_oversampling = self.saw_effective_ratio > 1;
            let dist_type = (self.saw_shape_type + 1) as u8;
            let shape_amt = self.saw_shape_amount as f32;

            let buf: &mut [f32] = match self.saw_effective_ratio {
                2 => self.saw_os_2x.resample_buffer(),
                4 => self.saw_os_4x.resample_buffer(),
                8 => self.saw_os_8x.resample_buffer(),
                16 => self.saw_os_16x.resample_buffer(),
                32 => self.saw_os_32x.resample_buffer(),
                64 => self.saw_os_64x.resample_buffer(),
                128 => self.saw_os_128x.resample_buffer(),
                _ => self.saw_os_2x.resample_buffer(),
            };

            for sample in buf.iter_mut().take(iterations) {
                let raw = self.saw_oscillator.next();

                let shaped = if self.saw_shape_amount > 0.001 {
                    apply_distortion(raw as f32, shape_amt, dist_type) as f64
                } else {
                    raw
                };

                let folded = if self.saw_fold > 0.001 {
                    let fold_gain = 1.0 + self.saw_fold * 4.0;
                    let x = shaped * fold_gain;
                    let f = if self.saw_fold_range == 1 {
                        (x * std::f64::consts::PI).sin()
                    } else {
                        x.sin()
                    };
                    shaped * (1.0 - self.saw_fold) + f * self.saw_fold
                } else {
                    shaped
                };

                *sample = folded as f32;
            }

            let sample = if use_oversampling {
                match self.saw_effective_ratio {
                    2 => self.saw_os_2x.downsample() as f64,
                    4 => self.saw_os_4x.downsample() as f64,
                    8 => self.saw_os_8x.downsample() as f64,
                    16 => self.saw_os_16x.downsample() as f64,
                    32 => self.saw_os_32x.downsample() as f64,
                    64 => self.saw_os_64x.downsample() as f64,
                    _ => self.saw_os_128x.downsample() as f64,
                }
            } else {
                buf[0] as f64
            };

            let tightened = self.saw_tight_filter.process(sample, self.saw_tight);
            saw_out = tightened * self.saw_volume * volume_env;
        }

        // ===== MIX AT DAW RATE =====
        let pll_out_final_l = pll_sample_l * self.pll_volume * volume_env;
        let pll_out_final_r = pll_sample_r * self.pll_volume * volume_env;

        let mixed_l = vps_out_l + pll_out_final_l + saw_out;
        let mixed_r = vps_out_r + pll_out_final_r + saw_out;

        // ===== SUB AT DAW RATE (single sample, returned separately for HPF routing) =====
        let sub_sample = if self.sub_volume > 0.001 {
            let sub_freq = self.base_frequency * 0.5;
            self.sub_oscillator.next(sub_freq) * self.sub_volume * volume_env
        } else {
            0.0
        };

        // ===== OUTPUT =====
        let vel_scale = self.velocity;
        let final_l = mixed_l * self.master_volume * vel_scale;
        let final_r = mixed_r * self.master_volume * vel_scale;
        let final_sub = sub_sample * self.master_volume * vel_scale;

        (final_l, final_r, final_sub)
    }
}
