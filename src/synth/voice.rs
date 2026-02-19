#![allow(clippy::too_many_arguments)]

use synfx_dsp::{Oversampling, SlewValue};
use super::oscillator::{Oscillator, PolyBlepWrapper, PLLOscillator};
use super::filter::StereoMoogFilter;
use super::envelope::Envelope;
use super::reverb::StereoReverb;
use super::lfo::ModulationValues;
use super::simd::{stereo, stereo_left, stereo_right, stereo_wavefold, stereo_tube_saturate};

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

    // ===== Processing =====
    stereo_filter: StereoMoogFilter,
    volume_envelope: Envelope,
    filter_envelope: Envelope,
    oversampling_2x_left: Oversampling<2>,
    oversampling_2x_right: Oversampling<2>,
    oversampling_4x_left: Oversampling<4>,
    oversampling_4x_right: Oversampling<4>,
    oversampling_8x_left: Oversampling<8>,
    oversampling_8x_right: Oversampling<8>,
    oversampling_16x_left: Oversampling<16>,
    oversampling_16x_right: Oversampling<16>,

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
    reverb_enabled: bool,
    oversampling_factor: i32,
    current_oversampling_factor: i32,
    effective_oversample_ratio: i32,  // Actual ratio for downsampling (includes base rate boost)

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

    // ===== Sub Oscillator =====
    sub_volume: f64,

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
    pll_fm_ratio: i32,

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
    pll_cross_feedback: f64,
    pll_fm_env_amount: f64,
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

    // ===== Filter (Moog Ladder LP) =====
    filter_enabled: bool,
    filter_cutoff: f64,
    filter_resonance: f64,
    filter_envelope_amount: f64,
    filter_drive: f64,

    // ===== Volume Envelope =====
    vol_env_attack: f64,
    vol_env_attack_shape: f64,
    vol_env_decay: f64,
    vol_env_decay_shape: f64,
    vol_env_sustain: f64,
    vol_env_release: f64,
    vol_env_release_shape: f64,
    velocity: f64,

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
    pll_pulse_width_slew: SlewValue<f64>,
    pll_stereo_offset_slew: SlewValue<f64>,
    pll_fm_amount_slew: SlewValue<f64>,
    pll_burst_threshold_slew: SlewValue<f64>,
    pll_burst_amount_slew: SlewValue<f64>,
    pll_color_amount_slew: SlewValue<f64>,
    pll_stereo_track_slew: SlewValue<f64>,
    pll_stereo_phase_slew: SlewValue<f64>,
    pll_cross_feedback_slew: SlewValue<f64>,
    pll_fm_env_slew: SlewValue<f64>,
    pll_range_slew: SlewValue<f64>,

    // Coloration slews
    ring_mod_slew: SlewValue<f64>,
    wavefold_slew: SlewValue<f64>,
    drift_amount_slew: SlewValue<f64>,
    tube_drive_slew: SlewValue<f64>,

    // VPS slew limiters
    vps_d_slew: SlewValue<f64>,
    vps_v_slew: SlewValue<f64>,
    vps_volume_slew: SlewValue<f64>,
    vps_stereo_offset_slew: SlewValue<f64>,
    vps_fold_slew: SlewValue<f64>,

    // Sub slew
    sub_volume_slew: SlewValue<f64>,

    // Velocity slew for click-free velocity changes
    velocity_slew: SlewValue<f64>,

    // Master volume slew for click-free volume changes
    master_volume_slew: SlewValue<f64>,
    target_master_volume: f64,

    // Filter slew - not used since nih_plug parameters have built-in smoothing
    #[allow(dead_code)]
    filter_cutoff_slew: SlewValue<f64>,
    #[allow(dead_code)]
    filter_resonance_slew: SlewValue<f64>,
    #[allow(dead_code)]
    filter_drive_slew: SlewValue<f64>,

    // ===== Target Values =====
    glide_time_ms: f64,
    legato_mode: bool,
    target_frequency: f64,
    target_pll_volume: f64,
    target_pll_track: f64,
    target_pll_damping: f64,
    target_pll_influence: f64,
    target_pll_feedback: f64,
    target_pll_pulse_width: f64,
    target_pll_stereo_offset: f64,
    target_pll_fm_amount: f64,
    target_pll_burst_threshold: f64,
    target_pll_burst_amount: f64,
    target_pll_color_amount: f64,
    target_pll_stereo_track_offset: f64,
    target_pll_stereo_phase: f64,
    target_pll_cross_feedback: f64,
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
    target_sub_volume: f64,
    target_velocity: f64,
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
    mod_pll_stereo_phase: f64,
    mod_pll_cross_feedback: f64,
    mod_pll_fm_env_amount: f64,
    mod_pll_burst_amount: f64,
    mod_pll_range: f64,
    mod_vps_d: f64,
    mod_vps_v: f64,
    mod_filter_cutoff: f64,
    mod_filter_resonance: f64,
    mod_filter_drive: f64,
    mod_ring_mod: f64,
    mod_wavefold: f64,
    mod_drift_amount: f64,
    mod_tube_drive: f64,
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
    mod_slew_pll_stereo_phase: SlewValue<f64>,
    mod_slew_pll_cross_fb: SlewValue<f64>,
    mod_slew_pll_fm_env: SlewValue<f64>,
    mod_slew_pll_burst: SlewValue<f64>,
    mod_slew_pll_range: SlewValue<f64>,
    mod_slew_vps_d: SlewValue<f64>,
    mod_slew_vps_v: SlewValue<f64>,
    mod_slew_filter_cut: SlewValue<f64>,
    mod_slew_filter_res: SlewValue<f64>,
    mod_slew_filter_drv: SlewValue<f64>,
    mod_slew_ring_mod: SlewValue<f64>,
    mod_slew_wavefold: SlewValue<f64>,
    mod_slew_drift: SlewValue<f64>,
    mod_slew_tube: SlewValue<f64>,
    mod_slew_rev_mix: SlewValue<f64>,
    mod_slew_rev_decay: SlewValue<f64>,
    mod_slew_pll_vol: SlewValue<f64>,
    mod_slew_vps_vol: SlewValue<f64>,
    mod_slew_sub_vol: SlewValue<f64>,
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

        let sample_rate_f64 = sample_rate as f64;
        // Default to 1x (no oversampling) - processing at DAW rate
        let processing_rate = sample_rate_f64;

        let make_slew = || {
            let mut s = SlewValue::new();
            s.set_sample_rate(sample_rate_f64);
            s
        };

        Self {
            vps_oscillator_left: Oscillator::new(processing_rate),
            vps_oscillator_right: Oscillator::new(processing_rate),
            sub_oscillator: SineOscillator::new(processing_rate),
            pll_oscillator_left: PLLOscillator::new(processing_rate),
            pll_oscillator_right: PLLOscillator::new(processing_rate),
            pll_reference_oscillator: PolyBlepWrapper::new(processing_rate),
            fm_oscillator: SineOscillator::new(processing_rate),

            stereo_filter: StereoMoogFilter::new(processing_rate),
            volume_envelope: Envelope::new(sample_rate_f64),
            filter_envelope: Envelope::new(sample_rate_f64),
            oversampling_2x_left,
            oversampling_2x_right,
            oversampling_4x_left,
            oversampling_4x_right,
            oversampling_8x_left,
            oversampling_8x_right,
            oversampling_16x_left,
            oversampling_16x_right,

            base_frequency: 220.0,
            master_volume: 0.8,
            daw_sample_rate: sample_rate_f64,
            processing_sample_rate: processing_rate,
            base_rate_option: 0,  // Auto (use DAW rate)

            // Bypass switches (all enabled by default)
            pll_enabled: true,
            vps_enabled: true,
            coloration_enabled: true,
            reverb_enabled: true,
            oversampling_factor: 0,  // 1x (no oversampling)
            current_oversampling_factor: 0,
            effective_oversample_ratio: 1,

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

            sub_volume: 0.0,

            pll_volume: 0.0,
            pll_track_speed: 0.5,
            pll_damping: 0.3,
            pll_multiplier: 1.0,
            pll_feedback_amount: 0.0,
            pll_feedback_state: 0.0,
            pll_mode_is_edge: false,
            pll_colored: false,

            pll_fm_amount: 0.0,
            pll_fm_ratio: 1,

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
            pll_cross_feedback: 0.0,
            pll_fm_env_amount: 0.0,
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

            filter_enabled: true,
            filter_cutoff: 1000.0,
            filter_resonance: 0.0,
            filter_envelope_amount: 0.0,
            filter_drive: 1.0,

            vol_env_attack: 1.0,
            vol_env_attack_shape: 0.5,
            vol_env_decay: 20.0,
            vol_env_decay_shape: 0.5,
            vol_env_sustain: 1.0,
            vol_env_release: 5.0,
            vol_env_release_shape: 0.5,
            velocity: 1.0,

            filt_env_attack: 1.0,
            filt_env_attack_shape: 0.5,
            filt_env_decay: 20.0,
            filt_env_decay_shape: 0.5,
            filt_env_sustain: 1.0,
            filt_env_release: 5.0,
            filt_env_release_shape: 0.5,

            reverb: StereoReverb::new(processing_rate as f32),

            freq_slew: make_slew(),
            pll_volume_slew: make_slew(),
            pll_track_slew: make_slew(),
            pll_damping_slew: make_slew(),
            pll_influence_slew: make_slew(),
            pll_feedback_slew: make_slew(),
            pll_pulse_width_slew: make_slew(),
            pll_stereo_offset_slew: make_slew(),
            pll_fm_amount_slew: make_slew(),
            pll_burst_threshold_slew: make_slew(),
            pll_burst_amount_slew: make_slew(),
            pll_color_amount_slew: make_slew(),
            pll_stereo_track_slew: make_slew(),
            pll_stereo_phase_slew: make_slew(),
            pll_cross_feedback_slew: make_slew(),
            pll_fm_env_slew: make_slew(),
            pll_range_slew: make_slew(),
            ring_mod_slew: make_slew(),
            wavefold_slew: make_slew(),
            drift_amount_slew: make_slew(),
            tube_drive_slew: make_slew(),
            vps_d_slew: make_slew(),
            vps_v_slew: make_slew(),
            vps_volume_slew: make_slew(),
            vps_stereo_offset_slew: make_slew(),
            vps_fold_slew: make_slew(),
            sub_volume_slew: make_slew(),
            velocity_slew: make_slew(),
            master_volume_slew: make_slew(),
            target_master_volume: 0.8,
            filter_cutoff_slew: make_slew(),
            filter_resonance_slew: make_slew(),
            filter_drive_slew: make_slew(),

            glide_time_ms: 0.0,
            legato_mode: false,
            target_frequency: 220.0,
            target_pll_volume: 0.0,
            target_pll_track: 0.5,
            target_pll_damping: 0.3,
            target_pll_influence: 0.5,
            target_pll_feedback: 0.0,
            target_pll_pulse_width: 0.5,
            target_pll_stereo_offset: 0.0,
            target_pll_fm_amount: 0.0,
            target_pll_burst_threshold: 0.7,
            target_pll_burst_amount: 3.3,
            target_pll_color_amount: 0.25,
            target_pll_stereo_track_offset: 0.0,
            target_pll_stereo_phase: 0.0,
            target_pll_cross_feedback: 0.0,
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
            target_sub_volume: 0.0,
            target_velocity: 1.0,
            target_filter_cutoff: 1000.0,
            target_filter_resonance: 0.0,
            target_filter_drive: 1.0,

            mod_pll_damping: 0.0,
            mod_pll_influence: 0.0,
            mod_pll_track_speed: 0.0,
            mod_pll_feedback: 0.0,
            mod_pll_fm_amount: 0.0,
            mod_pll_pulse_width: 0.0,
            mod_pll_stereo_phase: 0.0,
            mod_pll_cross_feedback: 0.0,
            mod_pll_fm_env_amount: 0.0,
            mod_pll_burst_amount: 0.0,
            mod_pll_range: 0.0,
            mod_vps_d: 0.0,
            mod_vps_v: 0.0,
            mod_filter_cutoff: 0.0,
            mod_filter_resonance: 0.0,
            mod_filter_drive: 0.0,
            mod_ring_mod: 0.0,
            mod_wavefold: 0.0,
            mod_drift_amount: 0.0,
            mod_tube_drive: 0.0,
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
            mod_slew_pll_stereo_phase: make_slew(),
            mod_slew_pll_cross_fb: make_slew(),
            mod_slew_pll_fm_env: make_slew(),
            mod_slew_pll_burst: make_slew(),
            mod_slew_pll_range: make_slew(),
            mod_slew_vps_d: make_slew(),
            mod_slew_vps_v: make_slew(),
            mod_slew_filter_cut: make_slew(),
            mod_slew_filter_res: make_slew(),
            mod_slew_filter_drv: make_slew(),
            mod_slew_ring_mod: make_slew(),
            mod_slew_wavefold: make_slew(),
            mod_slew_drift: make_slew(),
            mod_slew_tube: make_slew(),
            mod_slew_rev_mix: make_slew(),
            mod_slew_rev_decay: make_slew(),
            mod_slew_pll_vol: make_slew(),
            mod_slew_vps_vol: make_slew(),
            mod_slew_sub_vol: make_slew(),
        }
    }

    // Bypass switch setters
    pub fn set_bypass_switches(
        &mut self,
        pll: bool,
        vps: bool,
        coloration: bool,
        reverb: bool,
        oversampling_factor: i32,
    ) {
        self.pll_enabled = pll;
        self.vps_enabled = vps;
        self.coloration_enabled = coloration;
        self.reverb_enabled = reverb;
        self.oversampling_factor = oversampling_factor;

        // Update sample rates if oversampling factor changed
        if oversampling_factor != self.current_oversampling_factor {
            self.current_oversampling_factor = oversampling_factor;
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

            // Update envelopes with DAW rate (they run at DAW rate, not oversampled)
            self.volume_envelope.set_sample_rate(new_rate);
            self.filter_envelope.set_sample_rate(new_rate);

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
            update_slew(&mut self.pll_burst_threshold_slew);
            update_slew(&mut self.pll_burst_amount_slew);
            update_slew(&mut self.pll_color_amount_slew);
            update_slew(&mut self.pll_stereo_track_slew);
            update_slew(&mut self.pll_stereo_phase_slew);
            update_slew(&mut self.pll_cross_feedback_slew);
            update_slew(&mut self.pll_fm_env_slew);
            update_slew(&mut self.ring_mod_slew);
            update_slew(&mut self.wavefold_slew);
            update_slew(&mut self.drift_amount_slew);
            update_slew(&mut self.tube_drive_slew);
            update_slew(&mut self.vps_d_slew);
            update_slew(&mut self.vps_v_slew);
            update_slew(&mut self.vps_volume_slew);
            update_slew(&mut self.vps_stereo_offset_slew);
            update_slew(&mut self.vps_fold_slew);
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
            update_slew(&mut self.mod_slew_pll_cross_fb);
            update_slew(&mut self.mod_slew_pll_fm_env);
            update_slew(&mut self.mod_slew_pll_burst);
            update_slew(&mut self.mod_slew_pll_range);
            update_slew(&mut self.mod_slew_vps_d);
            update_slew(&mut self.mod_slew_vps_v);
            update_slew(&mut self.mod_slew_filter_cut);
            update_slew(&mut self.mod_slew_filter_res);
            update_slew(&mut self.mod_slew_filter_drv);
            update_slew(&mut self.mod_slew_ring_mod);
            update_slew(&mut self.mod_slew_wavefold);
            update_slew(&mut self.mod_slew_drift);
            update_slew(&mut self.mod_slew_tube);
            update_slew(&mut self.mod_slew_rev_mix);
            update_slew(&mut self.mod_slew_rev_decay);
            update_slew(&mut self.mod_slew_pll_vol);
            update_slew(&mut self.mod_slew_vps_vol);
            update_slew(&mut self.mod_slew_sub_vol);

            // Now update processing sample rate (oscillators, filters, etc.)
            self.update_processing_sample_rate();
        }
    }

    fn update_processing_sample_rate(&mut self) {
        // Determine base rate: 0=Auto (DAW), 1=44.1k, 2=88.2k, 3=96k, 4=192k
        let base_rate = match self.base_rate_option {
            1 => 44100.0,
            2 => 88200.0,
            3 => 96000.0,
            4 => 192000.0,
            _ => self.daw_sample_rate,  // Auto: use DAW rate
        };

        // Calculate the user-selected oversampling multiplier
        // 0=1x, 1=2x, 2=4x, 3=8x, 4=16x
        let oversample_mult = match self.current_oversampling_factor {
            0 => 1,
            1 => 2,
            2 => 4,
            3 => 8,
            _ => 16,
        };

        // Processing rate = base rate * oversampling factor
        self.processing_sample_rate = base_rate * oversample_mult as f64;

        // Calculate effective oversample ratio for downsampling to DAW rate
        let ratio = (self.processing_sample_rate / self.daw_sample_rate).round() as i32;
        self.effective_oversample_ratio = match ratio {
            r if r <= 1 => 1,
            r if r <= 2 => 2,
            r if r <= 4 => 4,
            r if r <= 8 => 8,
            _ => 16,
        };

        // Update all oscillators
        self.vps_oscillator_left.set_sample_rate(self.processing_sample_rate);
        self.vps_oscillator_right.set_sample_rate(self.processing_sample_rate);
        self.sub_oscillator.set_sample_rate(self.processing_sample_rate);
        self.pll_oscillator_left.set_sample_rate(self.processing_sample_rate);
        self.pll_oscillator_right.set_sample_rate(self.processing_sample_rate);
        self.pll_reference_oscillator.set_sample_rate(self.processing_sample_rate);
        self.fm_oscillator.set_sample_rate(self.processing_sample_rate);

        // Update stereo filter
        self.stereo_filter.set_sample_rate(self.processing_sample_rate);

        // Update reverb
        self.reverb.set_sample_rate(self.processing_sample_rate);
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
        self.pll_multiplier = mult;
        self.target_pll_influence = influence;
        self.pll_mode_is_edge = edge_mode;
        self.pll_colored = colored;
    }

    pub fn set_pll_volume(&mut self, volume: f64) {
        self.target_pll_volume = volume;
    }

    pub fn set_pll_fm_params(&mut self, amount: f64, ratio: i32) {
        self.target_pll_fm_amount = amount;
        self.pll_fm_ratio = ratio;
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

    pub fn set_pll_cross_feedback(&mut self, amount: f64) {
        self.target_pll_cross_feedback = amount;
    }

    pub fn set_pll_fm_env_amount(&mut self, amount: f64) {
        self.target_pll_fm_env_amount = amount;
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

    pub fn set_filter_params(&mut self, enabled: bool, cutoff: f64, resonance: f64, env_amount: f64, drive: f64) {
        self.filter_enabled = enabled;
        self.target_filter_cutoff = cutoff;
        self.target_filter_resonance = resonance;
        self.filter_envelope_amount = env_amount;
        self.target_filter_drive = drive;
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
        self.mod_pll_fm_amount = self.mod_slew_pll_fm.next(mod_values.pll_fm_amount, MOD_SLEW_MS);
        self.mod_pll_cross_feedback = self.mod_slew_pll_cross_fb.next(mod_values.pll_cross_feedback, MOD_SLEW_MS);
        self.mod_pll_burst_amount = self.mod_slew_pll_burst.next(mod_values.pll_burst_amount, MOD_SLEW_MS);
        self.mod_pll_range = self.mod_slew_pll_range.next(mod_values.pll_range, MOD_SLEW_MS);
        self.mod_vps_d = self.mod_slew_vps_d.next(mod_values.vps_d, MOD_SLEW_MS);
        self.mod_vps_v = self.mod_slew_vps_v.next(mod_values.vps_v, MOD_SLEW_MS);
        self.mod_filter_cutoff = self.mod_slew_filter_cut.next(mod_values.filter_cutoff, MOD_SLEW_MS);
        self.mod_filter_resonance = self.mod_slew_filter_res.next(mod_values.filter_resonance, MOD_SLEW_MS);
        self.mod_filter_drive = self.mod_slew_filter_drv.next(mod_values.filter_drive, MOD_SLEW_MS);
        self.mod_drift_amount = self.mod_slew_drift.next(mod_values.drift_amount, MOD_SLEW_MS);
        self.mod_tube_drive = self.mod_slew_tube.next(mod_values.tube_drive, MOD_SLEW_MS);
        self.mod_reverb_mix = self.mod_slew_rev_mix.next(mod_values.reverb_mix, MOD_SLEW_MS);
        self.mod_reverb_decay = self.mod_slew_rev_decay.next(mod_values.reverb_decay, MOD_SLEW_MS);
        self.mod_pll_volume = self.mod_slew_pll_vol.next(mod_values.pll_volume, MOD_SLEW_MS);
        self.mod_vps_volume = self.mod_slew_vps_vol.next(mod_values.vps_volume, MOD_SLEW_MS);
        self.mod_sub_volume = self.mod_slew_sub_vol.next(mod_values.sub_volume, MOD_SLEW_MS);
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
        let is_playing = self.volume_envelope.is_active();

        if self.legato_mode && is_playing {
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
        self.vps_oscillator_left.trigger();
        self.vps_oscillator_right.trigger();
        self.pll_reference_oscillator.reset_phase();
        self.pll_prev_out_l = 0.0;
        self.pll_prev_out_r = 0.0;
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
        self.pll_ref_pulse_width = (self.pll_pulse_width_slew.next(self.target_pll_pulse_width, 20.0) + self.mod_pll_pulse_width).clamp(0.05, 0.95);
        self.pll_damping_stereo_offset = self.pll_stereo_offset_slew.next(self.target_pll_stereo_offset, 60.0);
        self.pll_fm_amount = (self.pll_fm_amount_slew.next(self.target_pll_fm_amount, 20.0) + self.mod_pll_fm_amount).clamp(0.0, 1.0);
        self.pll_burst_threshold = self.pll_burst_threshold_slew.next(self.target_pll_burst_threshold, 20.0);
        self.pll_burst_amount = (self.pll_burst_amount_slew.next(self.target_pll_burst_amount, 20.0) + self.mod_pll_burst_amount * 10.0).clamp(0.0, 10.0);
        self.pll_range = (self.pll_range_slew.next(self.target_pll_range, 20.0) + self.mod_pll_range).clamp(0.0, 1.0);
        self.pll_color_amount = self.pll_color_amount_slew.next(self.target_pll_color_amount, 20.0);
        self.pll_stereo_track_offset = self.pll_stereo_track_slew.next(self.target_pll_stereo_track_offset, 60.0);
        self.pll_stereo_phase = (self.pll_stereo_phase_slew.next(self.target_pll_stereo_phase, 60.0) + self.mod_pll_stereo_phase).clamp(0.0, 1.0);
        self.pll_cross_feedback = (self.pll_cross_feedback_slew.next(self.target_pll_cross_feedback, 20.0) + self.mod_pll_cross_feedback).clamp(0.0, 1.0);
        self.pll_fm_env_amount = (self.pll_fm_env_slew.next(self.target_pll_fm_env_amount, 20.0) + self.mod_pll_fm_env_amount).clamp(0.0, 1.0);

        // Coloration slews + modulation
        self.ring_mod_amount = (self.ring_mod_slew.next(self.target_ring_mod, 20.0) + self.mod_ring_mod).clamp(0.0, 1.0);
        self.wavefold_amount = (self.wavefold_slew.next(self.target_wavefold, 20.0) + self.mod_wavefold).clamp(0.0, 1.0);
        self.drift_amount = (self.drift_amount_slew.next(self.target_drift_amount, 50.0) + self.mod_drift_amount).clamp(0.0, 1.0);
        self.drift_rate = self.target_drift_rate;
        self.tube_drive = (self.tube_drive_slew.next(self.target_tube_drive, 20.0) + self.mod_tube_drive).clamp(0.0, 1.0);

        // VPS slews + modulation
        self.vps_d_param = (self.vps_d_slew.next(self.target_vps_d, 20.0) + self.mod_vps_d).clamp(0.0, 1.0);
        self.vps_v_param = (self.vps_v_slew.next(self.target_vps_v, 20.0) + self.mod_vps_v).clamp(0.0, 1.0);
        self.vps_volume = (self.vps_volume_slew.next(self.target_vps_volume, 20.0) + self.mod_vps_volume).clamp(0.0, 1.0);
        self.vps_stereo_v_offset = self.vps_stereo_offset_slew.next(self.target_vps_stereo_offset, 20.0);
        self.vps_fold = self.vps_fold_slew.next(self.target_vps_fold, 20.0).clamp(0.0, 1.0);

        // Sub slew + modulation
        self.sub_volume = (self.sub_volume_slew.next(self.target_sub_volume, 20.0) + self.mod_sub_volume).clamp(0.0, 1.0);

        // Velocity slew for click-free note transitions (5ms is fast but smooth)
        self.velocity = self.velocity_slew.next(self.target_velocity, 5.0);

        // Master volume slew for click-free volume changes (20ms)
        self.master_volume = self.master_volume_slew.next(self.target_master_volume, 20.0);

        // Filter parameters - cutoff already smoothed by nih_plug parameter smoother
        // Using target values directly to avoid double-smoothing issues
        let cutoff_mod_hz = self.mod_filter_cutoff * 10000.0; // Scale modulation to ±10kHz
        self.filter_cutoff = self.target_filter_cutoff;
        self.filter_resonance = (self.target_filter_resonance + self.mod_filter_resonance).clamp(0.0, 0.99);
        self.filter_drive = (self.target_filter_drive + self.mod_filter_drive * 14.0).clamp(1.0, 15.0); // Range 1-15, scale by 14

        let cutoff = (self.filter_cutoff + filter_env * self.filter_envelope_amount + cutoff_mod_hz)
            .clamp(20.0, 20000.0);

        self.pll_oscillator_left.prepare_block();
        self.pll_oscillator_right.prepare_block();

        // Use effective_oversample_ratio which accounts for base_rate_88k
        let use_oversampling = self.effective_oversample_ratio > 1;
        let iterations = self.effective_oversample_ratio as usize;

        // Get appropriate resample buffers based on effective ratio
        let (buf_l, buf_r): (&mut [f32], &mut [f32]) = match self.effective_oversample_ratio {
            2 => (self.oversampling_2x_left.resample_buffer(), self.oversampling_2x_right.resample_buffer()),
            4 => (self.oversampling_4x_left.resample_buffer(), self.oversampling_4x_right.resample_buffer()),
            8 => (self.oversampling_8x_left.resample_buffer(), self.oversampling_8x_right.resample_buffer()),
            16 => (self.oversampling_16x_left.resample_buffer(), self.oversampling_16x_right.resample_buffer()),
            _ => (self.oversampling_2x_left.resample_buffer(), self.oversampling_2x_right.resample_buffer()),
        };
        let mut feedback = self.pll_feedback_state;
        let mut direct_out_l = 0.0_f64;
        let mut direct_out_r = 0.0_f64;

        for i in 0..iterations {
            let mut mixed_oscillators_l = 0.0_f64;
            let mut mixed_oscillators_r = 0.0_f64;
            let mut vps_out_l = 0.0_f64;
            let mut vps_out_r = 0.0_f64;
            let mut pll_out_final_l = 0.0_f64;
            let mut pll_out_final_r = 0.0_f64;

            // VPS Oscillators (no FM or formant applied)
            if self.vps_enabled && self.vps_volume > 0.001 {
                let tune_mult = 2.0_f64.powf((self.vps_tune as f64 + self.vps_fine) / 12.0);
                let base_freq = self.base_frequency * 2.0_f64.powi(self.vps_octave) * tune_mult;

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

                // Apply fold if enabled
                let (folded_l, folded_r) = if self.vps_fold > 0.001 {
                    let folded = stereo_wavefold(stereo(raw_l, raw_r), self.vps_fold);
                    (stereo_left(folded), stereo_right(folded))
                } else {
                    (raw_l, raw_r)
                };

                vps_out_l = folded_l * self.vps_volume * volume_env;
                vps_out_r = folded_r * self.vps_volume * volume_env;
                mixed_oscillators_l += vps_out_l;
                mixed_oscillators_r += vps_out_r;
            }

            // PLL Oscillators with FM and Formant (both applied only to PLL)
            if self.pll_enabled && self.pll_volume > 0.001 {
                // Drift - slow random frequency modulation for organic sound
                let drift_mod_l = if self.drift_amount > 0.001 {
                    self.drift_phase_l += self.drift_rate * 0.00001;
                    if self.drift_phase_l > 1.0 { self.drift_phase_l -= 1.0; }
                    let drift_lfo = (self.drift_phase_l * std::f64::consts::TAU).sin();
                    drift_lfo * self.drift_amount * 0.02
                } else { 0.0 };
                let drift_mod_r = if self.drift_amount > 0.001 {
                    self.drift_phase_r += self.drift_rate * 0.000011;
                    if self.drift_phase_r > 1.0 { self.drift_phase_r -= 1.0; }
                    let drift_lfo = (self.drift_phase_r * std::f64::consts::TAU).sin();
                    drift_lfo * self.drift_amount * 0.02
                } else { 0.0 };

                let pll_tune_mult = 2.0_f64.powf((self.pll_ref_tune as f64 + self.pll_ref_fine) / 12.0);
                let ref_freq = self.base_frequency * 2.0_f64.powi(self.pll_ref_octave) * pll_tune_mult;

                // FM modulation - modulate the reference frequency (only affects PLL)
                let fm_freq = ref_freq * self.pll_fm_ratio as f64;
                let fm_signal = self.fm_oscillator.next(fm_freq);
                // FM index scaled for musical results, with envelope modulation
                let fm_env_mod = 1.0 + filter_env * self.pll_fm_env_amount;
                let fm_index = self.pll_fm_amount * 4.0 * self.pll_fm_ratio as f64 * fm_env_mod;
                let fm_mod = fm_signal * fm_index * ref_freq;

                // Cross-feedback modulation
                let cross_mod_l = self.pll_prev_out_r * self.pll_cross_feedback * 0.5;
                let cross_mod_r = self.pll_prev_out_l * self.pll_cross_feedback * 0.5;

                let fb_mod = feedback * self.pll_feedback_amount * 5.0;
                let ref_mod_l = ((ref_freq * (1.0 + drift_mod_l) + fm_mod + cross_mod_l * ref_freq) * (1.0 + fb_mod)).clamp(20.0, self.processing_sample_rate * 2.0);
                let ref_mod_r = ((ref_freq * (1.0 + drift_mod_r) + fm_mod + cross_mod_r * ref_freq) * (1.0 + fb_mod)).clamp(20.0, self.processing_sample_rate * 2.0);

                self.pll_reference_oscillator.set_frequency(ref_mod_l);
                let ref_pulse = self.pll_reference_oscillator.next(self.pll_ref_pulse_width);
                let ref_phase = self.pll_reference_oscillator.get_phase();
                // Stereo phase offset for width
                let ref_phase_r = (ref_phase + self.pll_stereo_phase) % 1.0;

                let use_stereo = self.pll_damping_stereo_offset > 0.0001 || self.pll_stereo_track_offset > 0.0001 || self.pll_stereo_phase > 0.0001 || self.pll_cross_feedback > 0.001;

                // Stereo damping offset
                let damp_left = (self.pll_damping - self.pll_damping_stereo_offset).clamp(0.001, 1.0);
                let damp_right = if use_stereo {
                    (self.pll_damping + self.pll_damping_stereo_offset).clamp(0.001, 1.0)
                } else {
                    damp_left
                };

                // Stereo track speed offset
                let track_left = (self.pll_track_speed - self.pll_stereo_track_offset).clamp(0.0, 1.0);
                let track_right = if use_stereo {
                    (self.pll_track_speed + self.pll_stereo_track_offset).clamp(0.0, 1.0)
                } else {
                    track_left
                };

                let mode = if self.pll_mode_is_edge {
                    super::oscillator::PllMode::EdgePFD
                } else {
                    super::oscillator::PllMode::AnalogLikePD
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
                self.pll_oscillator_left.set_params(track_left, damp_left, self.pll_multiplier, slewed_influence, self.pll_colored, mode);
                let pll_raw_l = self.pll_oscillator_left.next(ref_phase, ref_mod_l, ref_pulse);

                let pll_raw_r = if use_stereo {
                    self.pll_oscillator_right.set_experimental_params(
                        self.pll_retrigger,
                        self.pll_burst_threshold,
                        self.pll_burst_amount,
                        self.pll_loop_saturation,
                        self.pll_color_amount,
                        self.pll_edge_sensitivity,
                        self.pll_range,
                    );
                    self.pll_oscillator_right.set_params(track_right, damp_right, self.pll_multiplier, slewed_influence, self.pll_colored, mode);
                    self.pll_oscillator_right.next(ref_phase_r, ref_mod_r, ref_pulse)
                } else {
                    pll_raw_l
                };

                // Store outputs for cross-feedback
                self.pll_prev_out_l = pll_raw_l;
                self.pll_prev_out_r = pll_raw_r;

                let pll_out_l = pll_raw_l;
                let pll_out_r = if use_stereo { pll_raw_r } else { pll_out_l };

                pll_out_final_l = pll_out_l * self.pll_volume * volume_env;
                pll_out_final_r = pll_out_r * self.pll_volume * volume_env;
                mixed_oscillators_l += pll_out_final_l;
                mixed_oscillators_r += pll_out_final_r;

                feedback = feedback * 0.9 + (pll_raw_l + pll_raw_r) * 0.05;
            }

            // Ring modulation (VPS × PLL)
            if self.coloration_enabled && self.ring_mod_amount > 0.001 {
                let ring_l = vps_out_l * pll_out_final_l * 4.0;
                let ring_r = vps_out_r * pll_out_final_r * 4.0;
                mixed_oscillators_l += ring_l * self.ring_mod_amount;
                mixed_oscillators_r += ring_r * self.ring_mod_amount;
            }

            // Wavefolder (SIMD stereo)
            if self.coloration_enabled && self.wavefold_amount > 0.001 {
                let mixed = stereo(mixed_oscillators_l, mixed_oscillators_r);
                let folded = stereo_wavefold(mixed, self.wavefold_amount);
                mixed_oscillators_l = stereo_left(folded);
                mixed_oscillators_r = stereo_right(folded);
            }

            // Tube saturation (SIMD stereo asymmetric soft clipping)
            if self.coloration_enabled && self.tube_drive > 0.001 {
                let mixed = stereo(mixed_oscillators_l, mixed_oscillators_r);
                let saturated = stereo_tube_saturate(mixed, self.tube_drive);
                mixed_oscillators_l = stereo_left(saturated);
                mixed_oscillators_r = stereo_right(saturated);
            }

            buf_l[i] = mixed_oscillators_l as f32;
            buf_r[i] = mixed_oscillators_r as f32;
        }

        // Apply SIMD stereo Moog ladder filter at oversampled rate
        if self.filter_enabled {
            self.stereo_filter.process_buffers(
                &mut buf_l[..iterations],
                &mut buf_r[..iterations],
                cutoff,
                self.filter_resonance,
                self.filter_drive,
            );
        }

        // Copy buffer to direct output when not oversampling
        if !use_oversampling {
            direct_out_l = buf_l[0] as f64;
            direct_out_r = buf_r[0] as f64;
        }

        // Apply reverb at oversampled rate with modulation
        // Apply modulation to reverb mix (0-1 range)
        let reverb_mix_modulated = (self.reverb.mix + self.mod_reverb_mix).clamp(0.0, 1.0);
        // Apply modulation to decay via temporary adjustment
        if self.mod_reverb_decay.abs() > 0.001 {
            self.reverb.apply_decay_mod(self.mod_reverb_decay);
        }
        if self.reverb_enabled && reverb_mix_modulated > 0.0 {
            for i in 0..iterations {
                let dry_l = if use_oversampling { buf_l[i] as f64 } else { direct_out_l };
                let dry_r = if use_oversampling { buf_r[i] as f64 } else { direct_out_r };

                let (wet_l, wet_r) = self.reverb.process(dry_l, dry_r);

                let out_l = (dry_l * (1.0 - reverb_mix_modulated) + wet_l * reverb_mix_modulated) as f32;
                let out_r = (dry_r * (1.0 - reverb_mix_modulated) + wet_r * reverb_mix_modulated) as f32;

                if use_oversampling {
                    buf_l[i] = out_l;
                    buf_r[i] = out_r;
                } else {
                    direct_out_l = out_l as f64;
                    direct_out_r = out_r as f64;
                }
            }
        }

        // Add sub oscillator at oversampled rate (pure sine at -1 octave)
        // Sub is added after filter/reverb so it remains clean and punchy
        if self.sub_volume > 0.001 {
            let sub_freq = self.base_frequency * 0.5; // -1 octave
            for i in 0..iterations {
                let sub_sample = self.sub_oscillator.next(sub_freq) * self.sub_volume * volume_env;
                if use_oversampling {
                    buf_l[i] += sub_sample as f32;
                    buf_r[i] += sub_sample as f32;
                } else {
                    direct_out_l += sub_sample;
                    direct_out_r += sub_sample;
                }
            }
        }

        self.pll_feedback_state = feedback;

        let vel_scale = self.velocity;
        let (final_l, final_r) = match self.effective_oversample_ratio {
            1 => (direct_out_l * self.master_volume * vel_scale, direct_out_r * self.master_volume * vel_scale),
            2 => {
                let downsampled_l = self.oversampling_2x_left.downsample() as f64;
                let downsampled_r = self.oversampling_2x_right.downsample() as f64;
                (downsampled_l * self.master_volume * vel_scale, downsampled_r * self.master_volume * vel_scale)
            }
            4 => {
                let downsampled_l = self.oversampling_4x_left.downsample() as f64;
                let downsampled_r = self.oversampling_4x_right.downsample() as f64;
                (downsampled_l * self.master_volume * vel_scale, downsampled_r * self.master_volume * vel_scale)
            }
            8 => {
                let downsampled_l = self.oversampling_8x_left.downsample() as f64;
                let downsampled_r = self.oversampling_8x_right.downsample() as f64;
                (downsampled_l * self.master_volume * vel_scale, downsampled_r * self.master_volume * vel_scale)
            }
            _ => {
                let downsampled_l = self.oversampling_16x_left.downsample() as f64;
                let downsampled_r = self.oversampling_16x_right.downsample() as f64;
                (downsampled_l * self.master_volume * vel_scale, downsampled_r * self.master_volume * vel_scale)
            }
        };

        (final_l, final_r)
    }
}
