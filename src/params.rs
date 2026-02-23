use nih_plug::prelude::*;
use nih_plug_egui::EguiState;
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BeatMode {
    Straight,
    Triplet,
    Dotted,
}

impl BeatMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            BeatMode::Straight => "S",
            BeatMode::Triplet => "T",
            BeatMode::Dotted => "D",
        }
    }
}

/// Check if a note's value matches the target slider value
/// target: -100 to 100 where negative=weak/short, 0=any, positive=strong/long
/// value: 0.0 to 1.0 (normalized value - strength or length)
///
/// At center (Any): affects ALL notes
/// At Weak/Short (-100): affects only notes with lowest value
/// At Strong/Long (+100): affects only notes with highest value
pub fn target_matches(target: f32, value: f32) -> bool {
    // At center (Any): affects all notes
    if target.abs() < 1.0 {
        return true;
    }

    if target < 0.0 {
        // Targeting weak/short: affects notes with value below threshold
        // At -100: threshold ≈ 0.15 (only very weak/short)
        // At -1: threshold ≈ 0.99 (almost all)
        let threshold = 1.0 - (target.abs() / 100.0) * 0.85;
        value < threshold
    } else {
        // Targeting strong/long: affects notes with value above threshold
        // At +100: threshold ≈ 0.85 (only very strong/long)
        // At +1: threshold ≈ 0.01 (almost all)
        let threshold = (target / 100.0) * 0.85;
        value > threshold
    }
}

/// Legacy function name for backwards compatibility
pub fn strength_target_matches(target: f32, strength: f32) -> bool {
    target_matches(target, strength)
}

/// Generate a random value "up to" the given amount
/// If amount > 0: returns random value in [0, amount]
/// If amount < 0: returns random value in [amount, 0]
/// If amount == 0: returns 0
pub fn random_up_to(amount: f32, rng: &mut impl rand::Rng) -> f32 {
    if amount > 0.0 {
        rng.gen_range(0.0..=amount)
    } else if amount < 0.0 {
        rng.gen_range(amount..=0.0)
    } else {
        0.0
    }
}

#[derive(Params)]
pub struct DeviceParams {
    #[persist = "editor-state"]
    pub editor_state: Arc<EguiState>,

    #[id = "div1_beat1"]
    pub div1_beat1: FloatParam,

    #[id = "div2_beat1"]
    pub div2_beat1: FloatParam,
    #[id = "div2_beat2"]
    pub div2_beat2: FloatParam,

    #[id = "div4_beat1"]
    pub div4_beat1: FloatParam,
    #[id = "div4_beat2"]
    pub div4_beat2: FloatParam,
    #[id = "div4_beat3"]
    pub div4_beat3: FloatParam,
    #[id = "div4_beat4"]
    pub div4_beat4: FloatParam,

    #[id = "div8_beat1"]
    pub div8_beat1: FloatParam,
    #[id = "div8_beat2"]
    pub div8_beat2: FloatParam,
    #[id = "div8_beat3"]
    pub div8_beat3: FloatParam,
    #[id = "div8_beat4"]
    pub div8_beat4: FloatParam,
    #[id = "div8_beat5"]
    pub div8_beat5: FloatParam,
    #[id = "div8_beat6"]
    pub div8_beat6: FloatParam,
    #[id = "div8_beat7"]
    pub div8_beat7: FloatParam,
    #[id = "div8_beat8"]
    pub div8_beat8: FloatParam,

    #[id = "div16_beat1"]
    pub div16_beat1: FloatParam,
    #[id = "div16_beat2"]
    pub div16_beat2: FloatParam,
    #[id = "div16_beat3"]
    pub div16_beat3: FloatParam,
    #[id = "div16_beat4"]
    pub div16_beat4: FloatParam,
    #[id = "div16_beat5"]
    pub div16_beat5: FloatParam,
    #[id = "div16_beat6"]
    pub div16_beat6: FloatParam,
    #[id = "div16_beat7"]
    pub div16_beat7: FloatParam,
    #[id = "div16_beat8"]
    pub div16_beat8: FloatParam,
    #[id = "div16_beat9"]
    pub div16_beat9: FloatParam,
    #[id = "div16_beat10"]
    pub div16_beat10: FloatParam,
    #[id = "div16_beat11"]
    pub div16_beat11: FloatParam,
    #[id = "div16_beat12"]
    pub div16_beat12: FloatParam,
    #[id = "div16_beat13"]
    pub div16_beat13: FloatParam,
    #[id = "div16_beat14"]
    pub div16_beat14: FloatParam,
    #[id = "div16_beat15"]
    pub div16_beat15: FloatParam,
    #[id = "div16_beat16"]
    pub div16_beat16: FloatParam,

    #[id = "div32_beat1"]
    pub div32_beat1: FloatParam,
    #[id = "div32_beat2"]
    pub div32_beat2: FloatParam,
    #[id = "div32_beat3"]
    pub div32_beat3: FloatParam,
    #[id = "div32_beat4"]
    pub div32_beat4: FloatParam,
    #[id = "div32_beat5"]
    pub div32_beat5: FloatParam,
    #[id = "div32_beat6"]
    pub div32_beat6: FloatParam,
    #[id = "div32_beat7"]
    pub div32_beat7: FloatParam,
    #[id = "div32_beat8"]
    pub div32_beat8: FloatParam,
    #[id = "div32_beat9"]
    pub div32_beat9: FloatParam,
    #[id = "div32_beat10"]
    pub div32_beat10: FloatParam,
    #[id = "div32_beat11"]
    pub div32_beat11: FloatParam,
    #[id = "div32_beat12"]
    pub div32_beat12: FloatParam,
    #[id = "div32_beat13"]
    pub div32_beat13: FloatParam,
    #[id = "div32_beat14"]
    pub div32_beat14: FloatParam,
    #[id = "div32_beat15"]
    pub div32_beat15: FloatParam,
    #[id = "div32_beat16"]
    pub div32_beat16: FloatParam,
    #[id = "div32_beat17"]
    pub div32_beat17: FloatParam,
    #[id = "div32_beat18"]
    pub div32_beat18: FloatParam,
    #[id = "div32_beat19"]
    pub div32_beat19: FloatParam,
    #[id = "div32_beat20"]
    pub div32_beat20: FloatParam,
    #[id = "div32_beat21"]
    pub div32_beat21: FloatParam,
    #[id = "div32_beat22"]
    pub div32_beat22: FloatParam,
    #[id = "div32_beat23"]
    pub div32_beat23: FloatParam,
    #[id = "div32_beat24"]
    pub div32_beat24: FloatParam,
    #[id = "div32_beat25"]
    pub div32_beat25: FloatParam,
    #[id = "div32_beat26"]
    pub div32_beat26: FloatParam,
    #[id = "div32_beat27"]
    pub div32_beat27: FloatParam,
    #[id = "div32_beat28"]
    pub div32_beat28: FloatParam,
    #[id = "div32_beat29"]
    pub div32_beat29: FloatParam,
    #[id = "div32_beat30"]
    pub div32_beat30: FloatParam,
    #[id = "div32_beat31"]
    pub div32_beat31: FloatParam,
    #[id = "div32_beat32"]
    pub div32_beat32: FloatParam,

    #[id = "div3t_beat1"]
    pub div3t_beat1: FloatParam,
    #[id = "div3t_beat2"]
    pub div3t_beat2: FloatParam,
    #[id = "div3t_beat3"]
    pub div3t_beat3: FloatParam,

    #[id = "div6t_beat1"]
    pub div6t_beat1: FloatParam,
    #[id = "div6t_beat2"]
    pub div6t_beat2: FloatParam,
    #[id = "div6t_beat3"]
    pub div6t_beat3: FloatParam,
    #[id = "div6t_beat4"]
    pub div6t_beat4: FloatParam,
    #[id = "div6t_beat5"]
    pub div6t_beat5: FloatParam,
    #[id = "div6t_beat6"]
    pub div6t_beat6: FloatParam,

    #[id = "div12t_beat1"]
    pub div12t_beat1: FloatParam,
    #[id = "div12t_beat2"]
    pub div12t_beat2: FloatParam,
    #[id = "div12t_beat3"]
    pub div12t_beat3: FloatParam,
    #[id = "div12t_beat4"]
    pub div12t_beat4: FloatParam,
    #[id = "div12t_beat5"]
    pub div12t_beat5: FloatParam,
    #[id = "div12t_beat6"]
    pub div12t_beat6: FloatParam,
    #[id = "div12t_beat7"]
    pub div12t_beat7: FloatParam,
    #[id = "div12t_beat8"]
    pub div12t_beat8: FloatParam,
    #[id = "div12t_beat9"]
    pub div12t_beat9: FloatParam,
    #[id = "div12t_beat10"]
    pub div12t_beat10: FloatParam,
    #[id = "div12t_beat11"]
    pub div12t_beat11: FloatParam,
    #[id = "div12t_beat12"]
    pub div12t_beat12: FloatParam,

    #[id = "div24t_beat1"]
    pub div24t_beat1: FloatParam,
    #[id = "div24t_beat2"]
    pub div24t_beat2: FloatParam,
    #[id = "div24t_beat3"]
    pub div24t_beat3: FloatParam,
    #[id = "div24t_beat4"]
    pub div24t_beat4: FloatParam,
    #[id = "div24t_beat5"]
    pub div24t_beat5: FloatParam,
    #[id = "div24t_beat6"]
    pub div24t_beat6: FloatParam,
    #[id = "div24t_beat7"]
    pub div24t_beat7: FloatParam,
    #[id = "div24t_beat8"]
    pub div24t_beat8: FloatParam,
    #[id = "div24t_beat9"]
    pub div24t_beat9: FloatParam,
    #[id = "div24t_beat10"]
    pub div24t_beat10: FloatParam,
    #[id = "div24t_beat11"]
    pub div24t_beat11: FloatParam,
    #[id = "div24t_beat12"]
    pub div24t_beat12: FloatParam,
    #[id = "div24t_beat13"]
    pub div24t_beat13: FloatParam,
    #[id = "div24t_beat14"]
    pub div24t_beat14: FloatParam,
    #[id = "div24t_beat15"]
    pub div24t_beat15: FloatParam,
    #[id = "div24t_beat16"]
    pub div24t_beat16: FloatParam,
    #[id = "div24t_beat17"]
    pub div24t_beat17: FloatParam,
    #[id = "div24t_beat18"]
    pub div24t_beat18: FloatParam,
    #[id = "div24t_beat19"]
    pub div24t_beat19: FloatParam,
    #[id = "div24t_beat20"]
    pub div24t_beat20: FloatParam,
    #[id = "div24t_beat21"]
    pub div24t_beat21: FloatParam,
    #[id = "div24t_beat22"]
    pub div24t_beat22: FloatParam,
    #[id = "div24t_beat23"]
    pub div24t_beat23: FloatParam,
    #[id = "div24t_beat24"]
    pub div24t_beat24: FloatParam,

    #[id = "div2d_beat1"]
    pub div2d_beat1: FloatParam,
    #[id = "div2d_beat2"]
    pub div2d_beat2: FloatParam,

    #[id = "div3d_beat1"]
    pub div3d_beat1: FloatParam,
    #[id = "div3d_beat2"]
    pub div3d_beat2: FloatParam,
    #[id = "div3d_beat3"]
    pub div3d_beat3: FloatParam,

    #[id = "div6d_beat1"]
    pub div6d_beat1: FloatParam,
    #[id = "div6d_beat2"]
    pub div6d_beat2: FloatParam,
    #[id = "div6d_beat3"]
    pub div6d_beat3: FloatParam,
    #[id = "div6d_beat4"]
    pub div6d_beat4: FloatParam,
    #[id = "div6d_beat5"]
    pub div6d_beat5: FloatParam,
    #[id = "div6d_beat6"]
    pub div6d_beat6: FloatParam,

    #[id = "div11d_beat1"]
    pub div11d_beat1: FloatParam,
    #[id = "div11d_beat2"]
    pub div11d_beat2: FloatParam,
    #[id = "div11d_beat3"]
    pub div11d_beat3: FloatParam,
    #[id = "div11d_beat4"]
    pub div11d_beat4: FloatParam,
    #[id = "div11d_beat5"]
    pub div11d_beat5: FloatParam,
    #[id = "div11d_beat6"]
    pub div11d_beat6: FloatParam,
    #[id = "div11d_beat7"]
    pub div11d_beat7: FloatParam,
    #[id = "div11d_beat8"]
    pub div11d_beat8: FloatParam,
    #[id = "div11d_beat9"]
    pub div11d_beat9: FloatParam,
    #[id = "div11d_beat10"]
    pub div11d_beat10: FloatParam,
    #[id = "div11d_beat11"]
    pub div11d_beat11: FloatParam,

    #[id = "div22d_beat1"]
    pub div22d_beat1: FloatParam,
    #[id = "div22d_beat2"]
    pub div22d_beat2: FloatParam,
    #[id = "div22d_beat3"]
    pub div22d_beat3: FloatParam,
    #[id = "div22d_beat4"]
    pub div22d_beat4: FloatParam,
    #[id = "div22d_beat5"]
    pub div22d_beat5: FloatParam,
    #[id = "div22d_beat6"]
    pub div22d_beat6: FloatParam,
    #[id = "div22d_beat7"]
    pub div22d_beat7: FloatParam,
    #[id = "div22d_beat8"]
    pub div22d_beat8: FloatParam,
    #[id = "div22d_beat9"]
    pub div22d_beat9: FloatParam,
    #[id = "div22d_beat10"]
    pub div22d_beat10: FloatParam,
    #[id = "div22d_beat11"]
    pub div22d_beat11: FloatParam,
    #[id = "div22d_beat12"]
    pub div22d_beat12: FloatParam,
    #[id = "div22d_beat13"]
    pub div22d_beat13: FloatParam,
    #[id = "div22d_beat14"]
    pub div22d_beat14: FloatParam,
    #[id = "div22d_beat15"]
    pub div22d_beat15: FloatParam,
    #[id = "div22d_beat16"]
    pub div22d_beat16: FloatParam,
    #[id = "div22d_beat17"]
    pub div22d_beat17: FloatParam,
    #[id = "div22d_beat18"]
    pub div22d_beat18: FloatParam,
    #[id = "div22d_beat19"]
    pub div22d_beat19: FloatParam,
    #[id = "div22d_beat20"]
    pub div22d_beat20: FloatParam,
    #[id = "div22d_beat21"]
    pub div22d_beat21: FloatParam,
    #[id = "div22d_beat22"]
    pub div22d_beat22: FloatParam,

    #[id = "synth_osc_d"]
    pub synth_osc_d: FloatParam,
    #[id = "synth_osc_v"]
    pub synth_osc_v: FloatParam,
    #[id = "synth_osc_stereo_v_offset"]
    pub synth_osc_stereo_v_offset: FloatParam,
    #[id = "synth_osc_volume"]
    pub synth_osc_volume: FloatParam,
    #[id = "synth_osc_octave"]
    pub synth_osc_octave: IntParam,
    #[id = "synth_osc_tune"]
    pub synth_osc_tune: IntParam,
    #[id = "synth_osc_fine"]
    pub synth_osc_fine: FloatParam,
    #[id = "synth_osc_fold"]
    pub synth_osc_fold: FloatParam,
    #[id = "synth_osc_stereo_d_offset"]
    pub synth_osc_stereo_d_offset: FloatParam,
    #[id = "synth_vps_shape_type"]
    pub synth_vps_shape_type: IntParam,
    #[id = "synth_vps_shape_amount"]
    pub synth_vps_shape_amount: FloatParam,
    #[id = "synth_vps_phase_mode"]
    pub synth_vps_phase_mode: IntParam,
    #[id = "synth_sub_volume"]
    pub synth_sub_volume: FloatParam,
    #[id = "synth_sub_source"]
    pub synth_sub_source: IntParam,

    #[id = "synth_pll_fm_amount"]
    pub synth_pll_fm_amount: FloatParam,
    #[id = "synth_pll_fm_ratio"]
    pub synth_pll_fm_ratio: IntParam,

    #[id = "synth_pll_track_speed"]
    pub synth_pll_track_speed: FloatParam,
    #[id = "synth_pll_damping"]
    pub synth_pll_damping: FloatParam,
    #[id = "synth_pll_mult"]
    pub synth_pll_mult: IntParam,
    #[id = "synth_pll_colored"]
    pub synth_pll_colored: BoolParam,
    #[id = "synth_pll_mode"]
    pub synth_pll_mode: BoolParam,
    #[id = "synth_pll_mult_slew"]
    pub synth_pll_mult_slew: BoolParam,
    #[id = "synth_pll_ref_octave"]
    pub synth_pll_ref_octave: IntParam,
    #[id = "synth_pll_ref_tune"]
    pub synth_pll_ref_tune: IntParam,
    #[id = "synth_pll_ref_fine"]
    pub synth_pll_ref_fine: FloatParam,
    #[id = "synth_pll_ref_pulse_width"]
    pub synth_pll_ref_pulse_width: FloatParam,
    #[id = "synth_pll_feedback"]
    pub synth_pll_feedback: FloatParam,
    #[id = "synth_pll_influence"]
    pub synth_pll_influence: FloatParam,
    #[id = "synth_pll_volume"]
    pub synth_pll_volume: FloatParam,
    #[id = "synth_pll_stereo_damp_offset"]
    pub synth_pll_stereo_damp_offset: FloatParam,
    #[id = "synth_pll_glide"]
    pub synth_pll_glide: FloatParam,

    // New PLL experimental parameters
    #[id = "synth_pll_retrigger"]
    pub synth_pll_retrigger: FloatParam,
    #[id = "synth_pll_burst_threshold"]
    pub synth_pll_burst_threshold: FloatParam,
    #[id = "synth_pll_burst_amount"]
    pub synth_pll_burst_amount: FloatParam,
    #[id = "synth_pll_loop_saturation"]
    pub synth_pll_loop_saturation: FloatParam,
    #[id = "synth_pll_color_amount"]
    pub synth_pll_color_amount: FloatParam,
    #[id = "synth_pll_edge_sensitivity"]
    pub synth_pll_edge_sensitivity: FloatParam,
    #[id = "synth_pll_range"]
    pub synth_pll_range: FloatParam,
    #[id = "synth_pll_stereo_track_offset"]
    pub synth_pll_stereo_track_offset: FloatParam,
    #[id = "synth_pll_stereo_phase"]
    pub synth_pll_stereo_phase: FloatParam,
    #[id = "synth_pll_cross_feedback"]
    pub synth_pll_cross_feedback: FloatParam,
    #[id = "synth_pll_fm_env_amount"]
    pub synth_pll_fm_env_amount: FloatParam,

    #[id = "synth_ring_mod"]
    pub synth_ring_mod: FloatParam,
    #[id = "synth_wavefold"]
    pub synth_wavefold: FloatParam,

    #[id = "synth_drift_amount"]
    pub synth_drift_amount: FloatParam,
    #[id = "synth_drift_rate"]
    pub synth_drift_rate: FloatParam,
    #[id = "synth_noise_amount"]
    pub synth_noise_amount: FloatParam,
    #[id = "synth_tube_drive"]
    pub synth_tube_drive: FloatParam,
    #[id = "synth_color_distortion_amount"]
    pub synth_color_distortion_amount: FloatParam,
    #[id = "synth_color_distortion_threshold"]
    pub synth_color_distortion_threshold: FloatParam,

    // Bypass switches for CPU profiling
    #[id = "synth_pll_enable"]
    pub synth_pll_enable: BoolParam,
    #[id = "synth_vps_enable"]
    pub synth_vps_enable: BoolParam,
    #[id = "synth_coloration_enable"]
    pub synth_coloration_enable: BoolParam,
    #[id = "synth_reverb_enable"]
    pub synth_reverb_enable: BoolParam,
    #[id = "synth_oversampling_factor"]
    pub synth_oversampling_factor: IntParam,
    #[id = "synth_base_rate"]
    pub synth_base_rate: IntParam,  // 0=Auto, 1=44.1k, 2=88.2k, 3=96k, 4=192k

    #[id = "synth_filter_enable"]
    pub synth_filter_enable: BoolParam,
    #[id = "synth_filter_cutoff"]
    pub synth_filter_cutoff: FloatParam,
    #[id = "synth_filter_resonance"]
    pub synth_filter_resonance: FloatParam,
    #[id = "synth_filter_env_amount"]
    pub synth_filter_env_amount: FloatParam,
    #[id = "synth_filter_drive"]
    pub synth_filter_drive: FloatParam,

    #[id = "global_volume"]
    pub global_volume: FloatParam,

    #[id = "limiter_enable"]
    pub limiter_enable: BoolParam,

    #[id = "synth_vol_attack"]
    pub synth_vol_attack: FloatParam,
    #[id = "synth_vol_attack_shape"]
    pub synth_vol_attack_shape: FloatParam,
    #[id = "synth_vol_decay"]
    pub synth_vol_decay: FloatParam,
    #[id = "synth_vol_decay_shape"]
    pub synth_vol_decay_shape: FloatParam,
    #[id = "synth_vol_sustain"]
    pub synth_vol_sustain: FloatParam,
    #[id = "synth_vol_release"]
    pub synth_vol_release: FloatParam,
    #[id = "synth_vol_release_shape"]
    pub synth_vol_release_shape: FloatParam,

    #[id = "synth_filt_attack"]
    pub synth_filt_attack: FloatParam,
    #[id = "synth_filt_attack_shape"]
    pub synth_filt_attack_shape: FloatParam,
    #[id = "synth_filt_decay"]
    pub synth_filt_decay: FloatParam,
    #[id = "synth_filt_decay_shape"]
    pub synth_filt_decay_shape: FloatParam,
    #[id = "synth_filt_sustain"]
    pub synth_filt_sustain: FloatParam,
    #[id = "synth_filt_release"]
    pub synth_filt_release: FloatParam,
    #[id = "synth_filt_release_shape"]
    pub synth_filt_release_shape: FloatParam,

    #[id = "synth_reverb_mix"]
    pub synth_reverb_mix: FloatParam,
    #[id = "synth_reverb_pre_delay"]
    pub synth_reverb_pre_delay: FloatParam,
    #[id = "synth_reverb_time_scale"]
    pub synth_reverb_time_scale: FloatParam,
    #[id = "synth_reverb_input_hpf"]
    pub synth_reverb_input_hpf: FloatParam,
    #[id = "synth_reverb_input_lpf"]
    pub synth_reverb_input_lpf: FloatParam,
    #[id = "synth_reverb_hpf"]
    pub synth_reverb_hpf: FloatParam,
    #[id = "synth_reverb_lpf"]
    pub synth_reverb_lpf: FloatParam,
    #[id = "synth_reverb_mod_speed"]
    pub synth_reverb_mod_speed: FloatParam,
    #[id = "synth_reverb_mod_depth"]
    pub synth_reverb_mod_depth: FloatParam,
    #[id = "synth_reverb_mod_shape"]
    pub synth_reverb_mod_shape: FloatParam,
    #[id = "synth_reverb_diffusion_mix"]
    pub synth_reverb_diffusion_mix: FloatParam,
    #[id = "synth_reverb_diffusion"]
    pub synth_reverb_diffusion: FloatParam,
    #[id = "synth_reverb_decay"]
    pub synth_reverb_decay: FloatParam,
    #[id = "synth_reverb_ducking"]
    pub synth_reverb_ducking: FloatParam,

    // ===== LFO 1 =====
    #[id = "lfo1_rate"]
    pub lfo1_rate: FloatParam,
    #[id = "lfo1_waveform"]
    pub lfo1_waveform: IntParam,
    #[id = "lfo1_tempo_sync"]
    pub lfo1_tempo_sync: BoolParam,
    #[id = "lfo1_sync_division"]
    pub lfo1_sync_division: IntParam,
    #[id = "lfo1_sync_source"]
    pub lfo1_sync_source: IntParam,
    #[id = "lfo1_phase_mod"]
    pub lfo1_phase_mod: FloatParam,
    #[id = "lfo1_dest1"]
    pub lfo1_dest1: IntParam,
    #[id = "lfo1_amount1"]
    pub lfo1_amount1: FloatParam,
    #[id = "lfo1_dest2"]
    pub lfo1_dest2: IntParam,
    #[id = "lfo1_amount2"]
    pub lfo1_amount2: FloatParam,

    // ===== LFO 2 =====
    #[id = "lfo2_rate"]
    pub lfo2_rate: FloatParam,
    #[id = "lfo2_waveform"]
    pub lfo2_waveform: IntParam,
    #[id = "lfo2_tempo_sync"]
    pub lfo2_tempo_sync: BoolParam,
    #[id = "lfo2_sync_division"]
    pub lfo2_sync_division: IntParam,
    #[id = "lfo2_sync_source"]
    pub lfo2_sync_source: IntParam,
    #[id = "lfo2_phase_mod"]
    pub lfo2_phase_mod: FloatParam,
    #[id = "lfo2_dest1"]
    pub lfo2_dest1: IntParam,
    #[id = "lfo2_amount1"]
    pub lfo2_amount1: FloatParam,
    #[id = "lfo2_dest2"]
    pub lfo2_dest2: IntParam,
    #[id = "lfo2_amount2"]
    pub lfo2_amount2: FloatParam,

    // ===== LFO 3 =====
    #[id = "lfo3_rate"]
    pub lfo3_rate: FloatParam,
    #[id = "lfo3_waveform"]
    pub lfo3_waveform: IntParam,
    #[id = "lfo3_tempo_sync"]
    pub lfo3_tempo_sync: BoolParam,
    #[id = "lfo3_sync_division"]
    pub lfo3_sync_division: IntParam,
    #[id = "lfo3_sync_source"]
    pub lfo3_sync_source: IntParam,
    #[id = "lfo3_phase_mod"]
    pub lfo3_phase_mod: FloatParam,
    #[id = "lfo3_dest1"]
    pub lfo3_dest1: IntParam,
    #[id = "lfo3_amount1"]
    pub lfo3_amount1: FloatParam,
    #[id = "lfo3_dest2"]
    pub lfo3_dest2: IntParam,
    #[id = "lfo3_amount2"]
    pub lfo3_amount2: FloatParam,

    // ===== Mod Sequencer =====
    #[id = "mseq_step_1"]
    pub mseq_step_1: FloatParam,
    #[id = "mseq_step_2"]
    pub mseq_step_2: FloatParam,
    #[id = "mseq_step_3"]
    pub mseq_step_3: FloatParam,
    #[id = "mseq_step_4"]
    pub mseq_step_4: FloatParam,
    #[id = "mseq_step_5"]
    pub mseq_step_5: FloatParam,
    #[id = "mseq_step_6"]
    pub mseq_step_6: FloatParam,
    #[id = "mseq_step_7"]
    pub mseq_step_7: FloatParam,
    #[id = "mseq_step_8"]
    pub mseq_step_8: FloatParam,
    #[id = "mseq_step_9"]
    pub mseq_step_9: FloatParam,
    #[id = "mseq_step_10"]
    pub mseq_step_10: FloatParam,
    #[id = "mseq_step_11"]
    pub mseq_step_11: FloatParam,
    #[id = "mseq_step_12"]
    pub mseq_step_12: FloatParam,
    #[id = "mseq_step_13"]
    pub mseq_step_13: FloatParam,
    #[id = "mseq_step_14"]
    pub mseq_step_14: FloatParam,
    #[id = "mseq_step_15"]
    pub mseq_step_15: FloatParam,
    #[id = "mseq_step_16"]
    pub mseq_step_16: FloatParam,
    #[id = "mseq_ties"]
    pub mseq_ties: IntParam,
    #[id = "mseq_division"]
    pub mseq_division: IntParam,
    #[id = "mseq_slew"]
    pub mseq_slew: FloatParam,
    #[id = "mseq_dest1"]
    pub mseq_dest1: IntParam,
    #[id = "mseq_amount1"]
    pub mseq_amount1: FloatParam,
    #[id = "mseq_dest2"]
    pub mseq_dest2: IntParam,
    #[id = "mseq_amount2"]
    pub mseq_amount2: FloatParam,

    #[id = "note_length_percent"]
    pub note_length_percent: FloatParam,

    #[id = "len_mod_1_target"]
    pub len_mod_1_target: FloatParam,
    #[id = "len_mod_1_amount"]
    pub len_mod_1_amount: FloatParam,
    #[id = "len_mod_1_prob"]
    pub len_mod_1_prob: FloatParam,

    #[id = "len_mod_2_target"]
    pub len_mod_2_target: FloatParam,
    #[id = "len_mod_2_amount"]
    pub len_mod_2_amount: FloatParam,
    #[id = "len_mod_2_prob"]
    pub len_mod_2_prob: FloatParam,

    #[id = "vel_strength_target"]
    pub vel_strength_target: FloatParam,
    #[id = "vel_strength_amount"]
    pub vel_strength_amount: FloatParam,
    #[id = "vel_strength_prob"]
    pub vel_strength_prob: FloatParam,

    #[id = "vel_length_target"]
    pub vel_length_target: FloatParam,
    #[id = "vel_length_amount"]
    pub vel_length_amount: FloatParam,
    #[id = "vel_length_prob"]
    pub vel_length_prob: FloatParam,

    #[id = "pos_mod_1_target"]
    pub pos_mod_1_target: FloatParam,
    #[id = "pos_mod_1_shift"]
    pub pos_mod_1_shift: FloatParam,
    #[id = "pos_mod_1_prob"]
    pub pos_mod_1_prob: FloatParam,

    #[id = "pos_mod_2_target"]
    pub pos_mod_2_target: FloatParam,
    #[id = "pos_mod_2_shift"]
    pub pos_mod_2_shift: FloatParam,
    #[id = "pos_mod_2_prob"]
    pub pos_mod_2_prob: FloatParam,

    #[id = "swing_amount"]
    pub swing_amount: FloatParam,

    #[id = "legato_mode"]
    pub legato_mode: BoolParam,
    #[id = "legato_time"]
    pub legato_time: FloatParam,

    #[id = "sequencer_enable"]
    pub sequencer_enable: BoolParam,
}

impl DeviceParams {
    /// Apply swing to a normalized time position (0.0 to 1.0)
    /// swing_amount: 50 = no swing, 66 = triplet feel, 75 = hard swing
    /// Swing affects the "and" of each beat (8th notes within quarter notes)
    pub fn apply_swing(time: f32, swing_amount: f32) -> f32 {
        if (swing_amount - 50.0).abs() < 0.01 {
            return time;
        }

        let swing_ratio = swing_amount / 100.0; // 0.5 to 0.75

        // Work within each quarter note (0.25 of the bar)
        let quarter_duration = 0.25_f32;
        let quarter_index = (time / quarter_duration).floor();
        let quarter_start = quarter_index * quarter_duration;
        let pos_in_quarter = time - quarter_start;

        // 8th note is half a quarter (0.125)
        let eighth_duration = quarter_duration / 2.0;

        // Check if we're in the second half of the quarter (the "and")
        if pos_in_quarter >= eighth_duration - 0.001 {
            // This is an "and" beat - apply swing
            // Calculate how far into the second 8th we are
            let pos_in_second_eighth = pos_in_quarter - eighth_duration;
            let ratio_in_second_eighth = pos_in_second_eighth / eighth_duration;

            // The second 8th starts at swing_ratio and ends at 1.0 of the quarter
            let swung_eighth_start = quarter_duration * swing_ratio;
            let swung_eighth_duration = quarter_duration - swung_eighth_start;

            quarter_start + swung_eighth_start + ratio_in_second_eighth * swung_eighth_duration
        } else {
            // This is a downbeat - compress into the first part of the quarter
            let ratio_in_first_eighth = pos_in_quarter / eighth_duration;
            let swung_first_duration = quarter_duration * swing_ratio;

            quarter_start + ratio_in_first_eighth * swung_first_duration
        }
    }

    /// Get all length modifiers as (target, amount, probability) tuples
    pub fn get_length_modifiers(&self) -> [(f32, f32, f32); 2] {
        [
            (
                self.len_mod_1_target.value(),
                self.len_mod_1_amount.value(),
                self.len_mod_1_prob.value(),
            ),
            (
                self.len_mod_2_target.value(),
                self.len_mod_2_amount.value(),
                self.len_mod_2_prob.value(),
            ),
        ]
    }

    /// Calculate the length multiplier based on strength and modifiers
    /// Returns the multiplier (1.0 = base length, 2.0 = double length)
    /// Amount is applied as "up to" - random value between 1.0 and the configured multiplier
    pub fn calculate_length_multiplier(&self, strength: f32, rng: &mut impl rand::Rng) -> f32 {
        let modifiers = self.get_length_modifiers();

        let mut candidates: Vec<(f32, f32)> = Vec::new();
        for (target, amount, prob) in modifiers.iter() {
            if strength_target_matches(*target, strength) && *prob > 0.0 {
                candidates.push((*amount, *prob));
            }
        }

        if candidates.is_empty() {
            return 1.0;
        }

        let total_prob: f32 = candidates.iter().map(|(_, p)| p).sum();
        let roll = rng.gen_range(0.0..127.0);

        if roll >= total_prob {
            return 1.0;
        }

        let mut cumulative = 0.0;
        for (amount, prob) in candidates {
            cumulative += prob;
            if roll < cumulative {
                let target_multiplier = amount / 100.0;
                // Apply "up to" logic: random between 1.0 and target multiplier
                if target_multiplier > 1.0 {
                    return rng.gen_range(1.0..=target_multiplier);
                } else if target_multiplier < 1.0 {
                    return rng.gen_range(target_multiplier..=1.0);
                } else {
                    return 1.0;
                }
            }
        }

        1.0
    }

    /// Calculate the velocity for a note based on strength, length, and modifiers
    /// strength: 0.0 to 1.0 (beat strength from grid)
    /// length: 0.0 to 1.0 (normalized beat duration, 0=shortest like 1/32, 1=longest like 1/1)
    /// Returns velocity 1-127
    #[allow(dead_code)]
    pub fn calculate_velocity(&self, strength: f32, length: f32, rng: &mut impl rand::Rng) -> u8 {
        let mut velocity: f32 = 100.0;

        // Strength-based velocity modifier (targets weak/any/strong beats)
        let strength_target = self.vel_strength_target.value();
        let strength_amount = self.vel_strength_amount.value();
        let strength_prob = self.vel_strength_prob.value();

        if target_matches(strength_target, strength) && strength_prob > 0.0 {
            let roll = rng.gen_range(0.0..127.0);
            if roll < strength_prob {
                velocity += strength_amount;
            }
        }

        // Length-based velocity modifier (targets short/any/long notes)
        let length_target = self.vel_length_target.value();
        let length_amount = self.vel_length_amount.value();
        let length_prob = self.vel_length_prob.value();

        if target_matches(length_target, length) && length_prob > 0.0 {
            let roll = rng.gen_range(0.0..127.0);
            if roll < length_prob {
                velocity += length_amount;
            }
        }

        velocity.clamp(1.0, 127.0) as u8
    }

    /// Calculate velocity using relative strength and length for targeting
    /// Amount is applied as "up to" - random value between 0 and amount
    pub fn calculate_velocity_relative(
        &self,
        relative_strength: f32,
        relative_length: f32,
        rng: &mut impl rand::Rng
    ) -> u8 {
        let mut velocity: f32 = 100.0;

        let strength_target = self.vel_strength_target.value();
        let strength_amount = self.vel_strength_amount.value();
        let strength_prob = self.vel_strength_prob.value();

        if target_matches(strength_target, relative_strength) && strength_prob > 0.0 {
            let roll = rng.gen_range(0.0..127.0);
            if roll < strength_prob {
                velocity += random_up_to(strength_amount, rng);
            }
        }

        let length_target = self.vel_length_target.value();
        let length_amount = self.vel_length_amount.value();
        let length_prob = self.vel_length_prob.value();

        if target_matches(length_target, relative_length) && length_prob > 0.0 {
            let roll = rng.gen_range(0.0..127.0);
            if roll < length_prob {
                velocity += random_up_to(length_amount, rng);
            }
        }

        velocity.clamp(1.0, 127.0) as u8
    }

    /// Get position modifiers as (target, shift, probability) tuples
    pub fn get_position_modifiers(&self) -> [(f32, f32, f32); 2] {
        [
            (
                self.pos_mod_1_target.value(),
                self.pos_mod_1_shift.value(),
                self.pos_mod_1_prob.value(),
            ),
            (
                self.pos_mod_2_target.value(),
                self.pos_mod_2_shift.value(),
                self.pos_mod_2_prob.value(),
            ),
        ]
    }

    /// Calculate the position shift based on strength and modifiers
    /// Returns the shift as a fraction of beat duration (-0.5 to +0.5)
    /// Shift is applied as "up to" - random value between 0 and the configured shift
    pub fn calculate_position_shift(&self, strength: f32, beat_duration: f32, rng: &mut impl rand::Rng) -> f32 {
        let modifiers = self.get_position_modifiers();

        // Collect matching modifiers
        let mut candidates: Vec<(f32, f32)> = Vec::new(); // (shift, probability)
        for (target, shift, prob) in modifiers.iter() {
            if strength_target_matches(*target, strength) && *prob > 0.0 {
                candidates.push((*shift, *prob));
            }
        }

        if candidates.is_empty() {
            return 0.0;
        }

        let total_prob: f32 = candidates.iter().map(|(_, p)| p).sum();
        let roll = rng.gen_range(0.0..127.0);

        if roll >= total_prob {
            return 0.0; // No modifier applies
        }

        // Pick winner proportionally
        let mut cumulative = 0.0;
        for (shift, prob) in candidates {
            cumulative += prob;
            if roll < cumulative {
                // Apply "up to" logic: random value between 0 and shift
                let actual_shift = random_up_to(shift, rng);
                // Convert shift percentage to actual time offset
                return (actual_shift / 100.0) * beat_duration;
            }
        }

        0.0
    }

    pub fn get_beat_time_span(mode: BeatMode, beat_count: usize, beat_index: usize) -> (f32, f32) {
        match mode {
            BeatMode::Straight => {
                let start = beat_index as f32 / beat_count as f32;
                let end = (beat_index + 1) as f32 / beat_count as f32;
                (start, end.min(1.0))
            }
            BeatMode::Triplet => {
                let start = beat_index as f32 / beat_count as f32;
                let end = (beat_index + 1) as f32 / beat_count as f32;
                (start, end.min(1.0))
            }
            BeatMode::Dotted => {
                let dotted_duration = match beat_count {
                    2 => 24.0 / 32.0,
                    3 => 12.0 / 32.0,
                    6 => 6.0 / 32.0,
                    11 => 3.0 / 32.0,
                    22 => 1.5 / 32.0,
                    _ => panic!("Invalid dotted division: {}", beat_count),
                };
                let start = beat_index as f32 * dotted_duration;
                let end = start + dotted_duration;
                (start, end.min(1.0))
            }
        }
    }

    pub fn time_spans_overlap(span1: (f32, f32), span2: (f32, f32)) -> bool {
        let (start1, end1) = span1;
        let (start2, end2) = span2;
        start1 < end2 && start2 < end1
    }

    pub fn calculate_available_range(
        &self,
        mode: BeatMode,
        beat_count: usize,
        beat_index: usize,
    ) -> f32 {
        let current_span = Self::get_beat_time_span(mode, beat_count, beat_index);

        let mut time_points = vec![current_span.0, current_span.1];

        for other_mode in [BeatMode::Straight, BeatMode::Triplet, BeatMode::Dotted] {
            for (other_count, _) in Self::get_divisions_for_mode(other_mode).iter() {
                if other_mode == mode && *other_count == beat_count {
                    continue;
                }

                for other_index in 0..*other_count {
                    let other_span = Self::get_beat_time_span(other_mode, *other_count, other_index);

                    if Self::time_spans_overlap(current_span, other_span) {
                        if other_span.0 > current_span.0 && other_span.0 < current_span.1 {
                            time_points.push(other_span.0);
                        }
                        if other_span.1 > current_span.0 && other_span.1 < current_span.1 {
                            time_points.push(other_span.1);
                        }
                    }
                }
            }
        }

        time_points.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        time_points.dedup();

        let mut max_constraint = 0.0f32;

        for time_idx in 0..time_points.len().saturating_sub(1) {
            let sample_time = (time_points[time_idx] + time_points[time_idx + 1]) / 2.0;
            let mut constraint_at_point = 0.0f32;

            for other_mode in [BeatMode::Straight, BeatMode::Triplet, BeatMode::Dotted] {
                for (other_count, _) in Self::get_divisions_for_mode(other_mode).iter() {
                    if other_mode == mode && *other_count == beat_count {
                        continue;
                    }

                    for other_index in 0..*other_count {
                        let other_span = Self::get_beat_time_span(other_mode, *other_count, other_index);

                        if sample_time >= other_span.0 && sample_time < other_span.1 {
                            let other_param = self.get_division_param(other_mode, *other_count, other_index);
                            let value = other_param.modulated_plain_value();
                            constraint_at_point += value;
                        }
                    }
                }
            }

            max_constraint = max_constraint.max(constraint_at_point);
        }

        (127.0 - max_constraint).max(0.0)
    }

    pub fn get_division_param(&self, mode: BeatMode, beat_count: usize, beat_index: usize) -> &FloatParam {
        match mode {
            BeatMode::Straight => match beat_count {
                1 => match beat_index {
                    0 => &self.div1_beat1,
                    _ => panic!("Invalid beat index {} for 1/1", beat_index),
                },
                2 => match beat_index {
                    0 => &self.div2_beat1,
                    1 => &self.div2_beat2,
                    _ => panic!("Invalid beat index {} for 1/2", beat_index),
                },
                4 => match beat_index {
                    0 => &self.div4_beat1,
                    1 => &self.div4_beat2,
                    2 => &self.div4_beat3,
                    3 => &self.div4_beat4,
                    _ => panic!("Invalid beat index {} for 1/4", beat_index),
                },
                8 => match beat_index {
                    0 => &self.div8_beat1,
                    1 => &self.div8_beat2,
                    2 => &self.div8_beat3,
                    3 => &self.div8_beat4,
                    4 => &self.div8_beat5,
                    5 => &self.div8_beat6,
                    6 => &self.div8_beat7,
                    7 => &self.div8_beat8,
                    _ => panic!("Invalid beat index {} for 1/8", beat_index),
                },
                16 => match beat_index {
                    0 => &self.div16_beat1,
                    1 => &self.div16_beat2,
                    2 => &self.div16_beat3,
                    3 => &self.div16_beat4,
                    4 => &self.div16_beat5,
                    5 => &self.div16_beat6,
                    6 => &self.div16_beat7,
                    7 => &self.div16_beat8,
                    8 => &self.div16_beat9,
                    9 => &self.div16_beat10,
                    10 => &self.div16_beat11,
                    11 => &self.div16_beat12,
                    12 => &self.div16_beat13,
                    13 => &self.div16_beat14,
                    14 => &self.div16_beat15,
                    15 => &self.div16_beat16,
                    _ => panic!("Invalid beat index {} for 1/16", beat_index),
                },
                32 => match beat_index {
                    0 => &self.div32_beat1,
                    1 => &self.div32_beat2,
                    2 => &self.div32_beat3,
                    3 => &self.div32_beat4,
                    4 => &self.div32_beat5,
                    5 => &self.div32_beat6,
                    6 => &self.div32_beat7,
                    7 => &self.div32_beat8,
                    8 => &self.div32_beat9,
                    9 => &self.div32_beat10,
                    10 => &self.div32_beat11,
                    11 => &self.div32_beat12,
                    12 => &self.div32_beat13,
                    13 => &self.div32_beat14,
                    14 => &self.div32_beat15,
                    15 => &self.div32_beat16,
                    16 => &self.div32_beat17,
                    17 => &self.div32_beat18,
                    18 => &self.div32_beat19,
                    19 => &self.div32_beat20,
                    20 => &self.div32_beat21,
                    21 => &self.div32_beat22,
                    22 => &self.div32_beat23,
                    23 => &self.div32_beat24,
                    24 => &self.div32_beat25,
                    25 => &self.div32_beat26,
                    26 => &self.div32_beat27,
                    27 => &self.div32_beat28,
                    28 => &self.div32_beat29,
                    29 => &self.div32_beat30,
                    30 => &self.div32_beat31,
                    31 => &self.div32_beat32,
                    _ => panic!("Invalid beat index {} for 1/32", beat_index),
                },
                _ => panic!("Invalid beat count {} for Straight mode", beat_count),
            },
            BeatMode::Triplet => match beat_count {
                3 => match beat_index {
                    0 => &self.div3t_beat1,
                    1 => &self.div3t_beat2,
                    2 => &self.div3t_beat3,
                    _ => panic!("Invalid beat index {} for 1/2T", beat_index),
                },
                6 => match beat_index {
                    0 => &self.div6t_beat1,
                    1 => &self.div6t_beat2,
                    2 => &self.div6t_beat3,
                    3 => &self.div6t_beat4,
                    4 => &self.div6t_beat5,
                    5 => &self.div6t_beat6,
                    _ => panic!("Invalid beat index {} for 1/4T", beat_index),
                },
                12 => match beat_index {
                    0 => &self.div12t_beat1,
                    1 => &self.div12t_beat2,
                    2 => &self.div12t_beat3,
                    3 => &self.div12t_beat4,
                    4 => &self.div12t_beat5,
                    5 => &self.div12t_beat6,
                    6 => &self.div12t_beat7,
                    7 => &self.div12t_beat8,
                    8 => &self.div12t_beat9,
                    9 => &self.div12t_beat10,
                    10 => &self.div12t_beat11,
                    11 => &self.div12t_beat12,
                    _ => panic!("Invalid beat index {} for 1/8T", beat_index),
                },
                24 => match beat_index {
                    0 => &self.div24t_beat1,
                    1 => &self.div24t_beat2,
                    2 => &self.div24t_beat3,
                    3 => &self.div24t_beat4,
                    4 => &self.div24t_beat5,
                    5 => &self.div24t_beat6,
                    6 => &self.div24t_beat7,
                    7 => &self.div24t_beat8,
                    8 => &self.div24t_beat9,
                    9 => &self.div24t_beat10,
                    10 => &self.div24t_beat11,
                    11 => &self.div24t_beat12,
                    12 => &self.div24t_beat13,
                    13 => &self.div24t_beat14,
                    14 => &self.div24t_beat15,
                    15 => &self.div24t_beat16,
                    16 => &self.div24t_beat17,
                    17 => &self.div24t_beat18,
                    18 => &self.div24t_beat19,
                    19 => &self.div24t_beat20,
                    20 => &self.div24t_beat21,
                    21 => &self.div24t_beat22,
                    22 => &self.div24t_beat23,
                    23 => &self.div24t_beat24,
                    _ => panic!("Invalid beat index {} for 1/16T", beat_index),
                },
                _ => panic!("Invalid beat count {} for Triplet mode", beat_count),
            },
            BeatMode::Dotted => match beat_count {
                2 => match beat_index {
                    0 => &self.div2d_beat1,
                    1 => &self.div2d_beat2,
                    _ => panic!("Invalid beat index {} for 1/2D", beat_index),
                },
                3 => match beat_index {
                    0 => &self.div3d_beat1,
                    1 => &self.div3d_beat2,
                    2 => &self.div3d_beat3,
                    _ => panic!("Invalid beat index {} for 1/4D", beat_index),
                },
                6 => match beat_index {
                    0 => &self.div6d_beat1,
                    1 => &self.div6d_beat2,
                    2 => &self.div6d_beat3,
                    3 => &self.div6d_beat4,
                    4 => &self.div6d_beat5,
                    5 => &self.div6d_beat6,
                    _ => panic!("Invalid beat index {} for 1/8D", beat_index),
                },
                11 => match beat_index {
                    0 => &self.div11d_beat1,
                    1 => &self.div11d_beat2,
                    2 => &self.div11d_beat3,
                    3 => &self.div11d_beat4,
                    4 => &self.div11d_beat5,
                    5 => &self.div11d_beat6,
                    6 => &self.div11d_beat7,
                    7 => &self.div11d_beat8,
                    8 => &self.div11d_beat9,
                    9 => &self.div11d_beat10,
                    10 => &self.div11d_beat11,
                    _ => panic!("Invalid beat index {} for 1/16D", beat_index),
                },
                22 => match beat_index {
                    0 => &self.div22d_beat1,
                    1 => &self.div22d_beat2,
                    2 => &self.div22d_beat3,
                    3 => &self.div22d_beat4,
                    4 => &self.div22d_beat5,
                    5 => &self.div22d_beat6,
                    6 => &self.div22d_beat7,
                    7 => &self.div22d_beat8,
                    8 => &self.div22d_beat9,
                    9 => &self.div22d_beat10,
                    10 => &self.div22d_beat11,
                    11 => &self.div22d_beat12,
                    12 => &self.div22d_beat13,
                    13 => &self.div22d_beat14,
                    14 => &self.div22d_beat15,
                    15 => &self.div22d_beat16,
                    16 => &self.div22d_beat17,
                    17 => &self.div22d_beat18,
                    18 => &self.div22d_beat19,
                    19 => &self.div22d_beat20,
                    20 => &self.div22d_beat21,
                    21 => &self.div22d_beat22,
                    _ => panic!("Invalid beat index {} for 1/32D", beat_index),
                },
                _ => panic!("Invalid beat count {} for Dotted mode", beat_count),
            },
        }
    }

    pub fn get_divisions_for_mode(mode: BeatMode) -> &'static [(usize, &'static str)] {
        match mode {
            BeatMode::Straight => &[
                (1, "1/1"),
                (2, "1/2"),
                (4, "1/4"),
                (8, "1/8"),
                (16, "1/16"),
                (32, "1/32"),
            ],
            BeatMode::Triplet => &[
                (3, "1/2"),
                (6, "1/4"),
                (12, "1/8"),
                (24, "1/16"),
            ],
            BeatMode::Dotted => &[
                (2, "1/2"),
                (3, "1/4"),
                (6, "1/8"),
                (11, "1/16"),
                (22, "1/32"),
            ],
        }
    }

    pub fn is_valid_beat_count(mode: BeatMode, beat_count: usize) -> bool {
        Self::get_divisions_for_mode(mode)
            .iter()
            .any(|(count, _)| *count == beat_count)
    }

    pub fn get_default_beat_count(mode: BeatMode) -> usize {
        match mode {
            BeatMode::Straight => 8,
            BeatMode::Triplet => 12,
            BeatMode::Dotted => 6,
        }
    }

    pub fn log_all_values(&self) {
        let mut values = Vec::new();

        for mode in [BeatMode::Straight, BeatMode::Triplet, BeatMode::Dotted] {
            for (count, label) in Self::get_divisions_for_mode(mode).iter() {
                for index in 0..*count {
                    let param = self.get_division_param(mode, *count, index);
                    let value = param.modulated_plain_value();
                    if value > 0.0 {
                        values.push(format!("{}[{}]={:.0}", label, index, value));
                    }
                }
            }
        }

        if !values.is_empty() {
            nih_log!("All set values: {}", values.join(", "));
        } else {
            nih_log!("All set values: (none)");
        }
    }

    fn create_param(name: String, default: f32) -> FloatParam {
        FloatParam::new(name, default, FloatRange::Linear { min: 0.0, max: 127.0 })
    }
}

impl Default for DeviceParams {
    fn default() -> Self {
        Self {
            editor_state: EguiState::from_size(1280, 720),

            div1_beat1: Self::create_param("1/1 Beat 1".to_string(), 0.0),

            div2_beat1: Self::create_param("1/2 Beat 1".to_string(), 0.0),
            div2_beat2: Self::create_param("1/2 Beat 2".to_string(), 0.0),

            div4_beat1: Self::create_param("1/4 Beat 1".to_string(), 0.0),
            div4_beat2: Self::create_param("1/4 Beat 2".to_string(), 0.0),
            div4_beat3: Self::create_param("1/4 Beat 3".to_string(), 0.0),
            div4_beat4: Self::create_param("1/4 Beat 4".to_string(), 0.0),

            div8_beat1: Self::create_param("1/8 Beat 1".to_string(), 0.0),
            div8_beat2: Self::create_param("1/8 Beat 2".to_string(), 0.0),
            div8_beat3: Self::create_param("1/8 Beat 3".to_string(), 0.0),
            div8_beat4: Self::create_param("1/8 Beat 4".to_string(), 0.0),
            div8_beat5: Self::create_param("1/8 Beat 5".to_string(), 0.0),
            div8_beat6: Self::create_param("1/8 Beat 6".to_string(), 0.0),
            div8_beat7: Self::create_param("1/8 Beat 7".to_string(), 0.0),
            div8_beat8: Self::create_param("1/8 Beat 8".to_string(), 0.0),

            div16_beat1: Self::create_param("1/16 Beat 1".to_string(), 0.0),
            div16_beat2: Self::create_param("1/16 Beat 2".to_string(), 0.0),
            div16_beat3: Self::create_param("1/16 Beat 3".to_string(), 0.0),
            div16_beat4: Self::create_param("1/16 Beat 4".to_string(), 0.0),
            div16_beat5: Self::create_param("1/16 Beat 5".to_string(), 0.0),
            div16_beat6: Self::create_param("1/16 Beat 6".to_string(), 0.0),
            div16_beat7: Self::create_param("1/16 Beat 7".to_string(), 0.0),
            div16_beat8: Self::create_param("1/16 Beat 8".to_string(), 0.0),
            div16_beat9: Self::create_param("1/16 Beat 9".to_string(), 0.0),
            div16_beat10: Self::create_param("1/16 Beat 10".to_string(), 0.0),
            div16_beat11: Self::create_param("1/16 Beat 11".to_string(), 0.0),
            div16_beat12: Self::create_param("1/16 Beat 12".to_string(), 0.0),
            div16_beat13: Self::create_param("1/16 Beat 13".to_string(), 0.0),
            div16_beat14: Self::create_param("1/16 Beat 14".to_string(), 0.0),
            div16_beat15: Self::create_param("1/16 Beat 15".to_string(), 0.0),
            div16_beat16: Self::create_param("1/16 Beat 16".to_string(), 0.0),

            div32_beat1: Self::create_param("1/32 Beat 1".to_string(), 0.0),
            div32_beat2: Self::create_param("1/32 Beat 2".to_string(), 0.0),
            div32_beat3: Self::create_param("1/32 Beat 3".to_string(), 0.0),
            div32_beat4: Self::create_param("1/32 Beat 4".to_string(), 0.0),
            div32_beat5: Self::create_param("1/32 Beat 5".to_string(), 0.0),
            div32_beat6: Self::create_param("1/32 Beat 6".to_string(), 0.0),
            div32_beat7: Self::create_param("1/32 Beat 7".to_string(), 0.0),
            div32_beat8: Self::create_param("1/32 Beat 8".to_string(), 0.0),
            div32_beat9: Self::create_param("1/32 Beat 9".to_string(), 0.0),
            div32_beat10: Self::create_param("1/32 Beat 10".to_string(), 0.0),
            div32_beat11: Self::create_param("1/32 Beat 11".to_string(), 0.0),
            div32_beat12: Self::create_param("1/32 Beat 12".to_string(), 0.0),
            div32_beat13: Self::create_param("1/32 Beat 13".to_string(), 0.0),
            div32_beat14: Self::create_param("1/32 Beat 14".to_string(), 0.0),
            div32_beat15: Self::create_param("1/32 Beat 15".to_string(), 0.0),
            div32_beat16: Self::create_param("1/32 Beat 16".to_string(), 0.0),
            div32_beat17: Self::create_param("1/32 Beat 17".to_string(), 0.0),
            div32_beat18: Self::create_param("1/32 Beat 18".to_string(), 0.0),
            div32_beat19: Self::create_param("1/32 Beat 19".to_string(), 0.0),
            div32_beat20: Self::create_param("1/32 Beat 20".to_string(), 0.0),
            div32_beat21: Self::create_param("1/32 Beat 21".to_string(), 0.0),
            div32_beat22: Self::create_param("1/32 Beat 22".to_string(), 0.0),
            div32_beat23: Self::create_param("1/32 Beat 23".to_string(), 0.0),
            div32_beat24: Self::create_param("1/32 Beat 24".to_string(), 0.0),
            div32_beat25: Self::create_param("1/32 Beat 25".to_string(), 0.0),
            div32_beat26: Self::create_param("1/32 Beat 26".to_string(), 0.0),
            div32_beat27: Self::create_param("1/32 Beat 27".to_string(), 0.0),
            div32_beat28: Self::create_param("1/32 Beat 28".to_string(), 0.0),
            div32_beat29: Self::create_param("1/32 Beat 29".to_string(), 0.0),
            div32_beat30: Self::create_param("1/32 Beat 30".to_string(), 0.0),
            div32_beat31: Self::create_param("1/32 Beat 31".to_string(), 0.0),
            div32_beat32: Self::create_param("1/32 Beat 32".to_string(), 0.0),

            div3t_beat1: Self::create_param("1/2T Beat 1".to_string(), 0.0),
            div3t_beat2: Self::create_param("1/2T Beat 2".to_string(), 0.0),
            div3t_beat3: Self::create_param("1/2T Beat 3".to_string(), 0.0),

            div6t_beat1: Self::create_param("1/4T Beat 1".to_string(), 0.0),
            div6t_beat2: Self::create_param("1/4T Beat 2".to_string(), 0.0),
            div6t_beat3: Self::create_param("1/4T Beat 3".to_string(), 0.0),
            div6t_beat4: Self::create_param("1/4T Beat 4".to_string(), 0.0),
            div6t_beat5: Self::create_param("1/4T Beat 5".to_string(), 0.0),
            div6t_beat6: Self::create_param("1/4T Beat 6".to_string(), 0.0),

            div12t_beat1: Self::create_param("1/8T Beat 1".to_string(), 0.0),
            div12t_beat2: Self::create_param("1/8T Beat 2".to_string(), 0.0),
            div12t_beat3: Self::create_param("1/8T Beat 3".to_string(), 0.0),
            div12t_beat4: Self::create_param("1/8T Beat 4".to_string(), 0.0),
            div12t_beat5: Self::create_param("1/8T Beat 5".to_string(), 0.0),
            div12t_beat6: Self::create_param("1/8T Beat 6".to_string(), 0.0),
            div12t_beat7: Self::create_param("1/8T Beat 7".to_string(), 0.0),
            div12t_beat8: Self::create_param("1/8T Beat 8".to_string(), 0.0),
            div12t_beat9: Self::create_param("1/8T Beat 9".to_string(), 0.0),
            div12t_beat10: Self::create_param("1/8T Beat 10".to_string(), 0.0),
            div12t_beat11: Self::create_param("1/8T Beat 11".to_string(), 0.0),
            div12t_beat12: Self::create_param("1/8T Beat 12".to_string(), 0.0),

            div24t_beat1: Self::create_param("1/16T Beat 1".to_string(), 0.0),
            div24t_beat2: Self::create_param("1/16T Beat 2".to_string(), 0.0),
            div24t_beat3: Self::create_param("1/16T Beat 3".to_string(), 0.0),
            div24t_beat4: Self::create_param("1/16T Beat 4".to_string(), 0.0),
            div24t_beat5: Self::create_param("1/16T Beat 5".to_string(), 0.0),
            div24t_beat6: Self::create_param("1/16T Beat 6".to_string(), 0.0),
            div24t_beat7: Self::create_param("1/16T Beat 7".to_string(), 0.0),
            div24t_beat8: Self::create_param("1/16T Beat 8".to_string(), 0.0),
            div24t_beat9: Self::create_param("1/16T Beat 9".to_string(), 0.0),
            div24t_beat10: Self::create_param("1/16T Beat 10".to_string(), 0.0),
            div24t_beat11: Self::create_param("1/16T Beat 11".to_string(), 0.0),
            div24t_beat12: Self::create_param("1/16T Beat 12".to_string(), 0.0),
            div24t_beat13: Self::create_param("1/16T Beat 13".to_string(), 0.0),
            div24t_beat14: Self::create_param("1/16T Beat 14".to_string(), 0.0),
            div24t_beat15: Self::create_param("1/16T Beat 15".to_string(), 0.0),
            div24t_beat16: Self::create_param("1/16T Beat 16".to_string(), 0.0),
            div24t_beat17: Self::create_param("1/16T Beat 17".to_string(), 0.0),
            div24t_beat18: Self::create_param("1/16T Beat 18".to_string(), 0.0),
            div24t_beat19: Self::create_param("1/16T Beat 19".to_string(), 0.0),
            div24t_beat20: Self::create_param("1/16T Beat 20".to_string(), 0.0),
            div24t_beat21: Self::create_param("1/16T Beat 21".to_string(), 0.0),
            div24t_beat22: Self::create_param("1/16T Beat 22".to_string(), 0.0),
            div24t_beat23: Self::create_param("1/16T Beat 23".to_string(), 0.0),
            div24t_beat24: Self::create_param("1/16T Beat 24".to_string(), 0.0),

            div2d_beat1: Self::create_param("1/2D Beat 1".to_string(), 0.0),
            div2d_beat2: Self::create_param("1/2D Beat 2".to_string(), 0.0),

            div3d_beat1: Self::create_param("1/4D Beat 1".to_string(), 0.0),
            div3d_beat2: Self::create_param("1/4D Beat 2".to_string(), 0.0),
            div3d_beat3: Self::create_param("1/4D Beat 3".to_string(), 0.0),

            div6d_beat1: Self::create_param("1/8D Beat 1".to_string(), 0.0),
            div6d_beat2: Self::create_param("1/8D Beat 2".to_string(), 0.0),
            div6d_beat3: Self::create_param("1/8D Beat 3".to_string(), 0.0),
            div6d_beat4: Self::create_param("1/8D Beat 4".to_string(), 0.0),
            div6d_beat5: Self::create_param("1/8D Beat 5".to_string(), 0.0),
            div6d_beat6: Self::create_param("1/8D Beat 6".to_string(), 0.0),

            div11d_beat1: Self::create_param("1/16D Beat 1".to_string(), 0.0),
            div11d_beat2: Self::create_param("1/16D Beat 2".to_string(), 0.0),
            div11d_beat3: Self::create_param("1/16D Beat 3".to_string(), 0.0),
            div11d_beat4: Self::create_param("1/16D Beat 4".to_string(), 0.0),
            div11d_beat5: Self::create_param("1/16D Beat 5".to_string(), 0.0),
            div11d_beat6: Self::create_param("1/16D Beat 6".to_string(), 0.0),
            div11d_beat7: Self::create_param("1/16D Beat 7".to_string(), 0.0),
            div11d_beat8: Self::create_param("1/16D Beat 8".to_string(), 0.0),
            div11d_beat9: Self::create_param("1/16D Beat 9".to_string(), 0.0),
            div11d_beat10: Self::create_param("1/16D Beat 10".to_string(), 0.0),
            div11d_beat11: Self::create_param("1/16D Beat 11".to_string(), 0.0),

            div22d_beat1: Self::create_param("1/32D Beat 1".to_string(), 0.0),
            div22d_beat2: Self::create_param("1/32D Beat 2".to_string(), 0.0),
            div22d_beat3: Self::create_param("1/32D Beat 3".to_string(), 0.0),
            div22d_beat4: Self::create_param("1/32D Beat 4".to_string(), 0.0),
            div22d_beat5: Self::create_param("1/32D Beat 5".to_string(), 0.0),
            div22d_beat6: Self::create_param("1/32D Beat 6".to_string(), 0.0),
            div22d_beat7: Self::create_param("1/32D Beat 7".to_string(), 0.0),
            div22d_beat8: Self::create_param("1/32D Beat 8".to_string(), 0.0),
            div22d_beat9: Self::create_param("1/32D Beat 9".to_string(), 0.0),
            div22d_beat10: Self::create_param("1/32D Beat 10".to_string(), 0.0),
            div22d_beat11: Self::create_param("1/32D Beat 11".to_string(), 0.0),
            div22d_beat12: Self::create_param("1/32D Beat 12".to_string(), 0.0),
            div22d_beat13: Self::create_param("1/32D Beat 13".to_string(), 0.0),
            div22d_beat14: Self::create_param("1/32D Beat 14".to_string(), 0.0),
            div22d_beat15: Self::create_param("1/32D Beat 15".to_string(), 0.0),
            div22d_beat16: Self::create_param("1/32D Beat 16".to_string(), 0.0),
            div22d_beat17: Self::create_param("1/32D Beat 17".to_string(), 0.0),
            div22d_beat18: Self::create_param("1/32D Beat 18".to_string(), 0.0),
            div22d_beat19: Self::create_param("1/32D Beat 19".to_string(), 0.0),
            div22d_beat20: Self::create_param("1/32D Beat 20".to_string(), 0.0),
            div22d_beat21: Self::create_param("1/32D Beat 21".to_string(), 0.0),
            div22d_beat22: Self::create_param("1/32D Beat 22".to_string(), 0.0),

            synth_osc_d: FloatParam::new(
                "Oscillator D".to_string(),
                0.5,
                FloatRange::Linear { min: 0.0, max: 1.0 }
            ).with_smoother(SmoothingStyle::Linear(50.0)),
            synth_osc_v: FloatParam::new(
                "Oscillator V".to_string(),
                0.5,
                FloatRange::Linear { min: 0.0, max: 1.0 }
            ).with_smoother(SmoothingStyle::Linear(50.0)),
            synth_osc_stereo_v_offset: FloatParam::new(
                "VPS Stereo V Δ".to_string(),
                0.0,
                FloatRange::Linear { min: 0.0, max: 0.3 }
            ).with_smoother(SmoothingStyle::Linear(50.0)),
            synth_osc_volume: FloatParam::new(
                "VPS Volume".to_string(),
                1.0,
                FloatRange::Linear { min: 0.0, max: 1.0 }
            ).with_smoother(SmoothingStyle::Linear(50.0)),
            synth_osc_octave: IntParam::new(
                "VPS Octave".to_string(),
                0,
                IntRange::Linear { min: -3, max: 3 }
            ),
            synth_osc_tune: IntParam::new(
                "VPS Tune".to_string(),
                0,
                IntRange::Linear { min: -12, max: 12 }
            ),
            synth_osc_fine: FloatParam::new(
                "VPS Fine".to_string(),
                0.0,
                FloatRange::Linear { min: -1.0, max: 1.0 }
            ).with_smoother(SmoothingStyle::Linear(20.0)),
            synth_osc_fold: FloatParam::new(
                "VPS Fold".to_string(),
                0.0,
                FloatRange::Linear { min: 0.0, max: 1.0 }
            ).with_smoother(SmoothingStyle::Linear(20.0)),
            synth_osc_stereo_d_offset: FloatParam::new(
                "VPS Stereo D Δ".to_string(),
                0.0,
                FloatRange::Linear { min: 0.0, max: 0.3 }
            ).with_smoother(SmoothingStyle::Linear(50.0)),
            synth_vps_shape_type: IntParam::new(
                "VPS Shape Type".to_string(),
                0,
                IntRange::Linear { min: 0, max: 1 }
            ),
            synth_vps_shape_amount: FloatParam::new(
                "VPS Shape Amount".to_string(),
                0.0,
                FloatRange::Linear { min: 0.0, max: 1.0 }
            ).with_smoother(SmoothingStyle::Linear(20.0)),
            synth_vps_phase_mode: IntParam::new(
                "VPS Phase Mode".to_string(),
                0,
                IntRange::Linear { min: 0, max: 1 }
            ),
            synth_sub_volume: FloatParam::new(
                "Sub Volume".to_string(),
                0.0,
                FloatRange::Linear { min: 0.0, max: 1.0 }
            ).with_smoother(SmoothingStyle::Linear(50.0)),
            synth_sub_source: IntParam::new(
                "Sub Source".to_string(),
                0,
                IntRange::Linear { min: 0, max: 1 }
            ),

            synth_pll_fm_amount: FloatParam::new(
                "PLL FM Amount".to_string(),
                0.0,
                FloatRange::Linear { min: 0.0, max: 0.2 }
            ).with_smoother(SmoothingStyle::Linear(50.0)),
            synth_pll_fm_ratio: IntParam::new(
                "PLL FM Ratio".to_string(),
                1,
                IntRange::Linear { min: 1, max: 8 }
            ),

            synth_pll_track_speed: FloatParam::new(
                "PLL Track Speed".to_string(),
                0.5,
                FloatRange::Linear { min: 0.0, max: 1.0 }
            ).with_smoother(SmoothingStyle::Linear(50.0)),
            synth_pll_damping: FloatParam::new(
                "PLL Damping".to_string(),
                0.3,
                FloatRange::Linear { min: 0.0, max: 1.0 }
            ).with_smoother(SmoothingStyle::Linear(50.0)),
            synth_pll_mult: IntParam::new(
                "PLL Multiplier".to_string(),
                0,
                IntRange::Linear { min: 0, max: 6 }
            ),
            synth_pll_colored: BoolParam::new(
                "PLL Colored".to_string(),
                false
            ),
            synth_pll_mode: BoolParam::new(
                "PLL Edge Mode".to_string(),
                true
            ),
            synth_pll_mult_slew: BoolParam::new(
                "PLL Mult Slew".to_string(),
                true
            ),
            synth_pll_ref_octave: IntParam::new(
                "PLL Ref Octave".to_string(),
                0,
                IntRange::Linear { min: -3, max: 3 }
            ),
            synth_pll_ref_tune: IntParam::new(
                "PLL Ref Tune".to_string(),
                0,
                IntRange::Linear { min: -12, max: 12 }
            ),
            synth_pll_ref_fine: FloatParam::new(
                "PLL Ref Fine".to_string(),
                0.0,
                FloatRange::Linear { min: -1.0, max: 1.0 }
            ).with_smoother(SmoothingStyle::Linear(20.0)),
            synth_pll_ref_pulse_width: FloatParam::new(
                "PLL Ref Pulse Width".to_string(),
                0.5,
                FloatRange::Linear { min: 0.01, max: 0.99 }
            ).with_smoother(SmoothingStyle::Linear(50.0)),
            synth_pll_feedback: FloatParam::new(
                "PLL Feedback".to_string(),
                0.0,
                FloatRange::Linear { min: 0.0, max: 1.0 }
            ).with_smoother(SmoothingStyle::Linear(50.0)),
            synth_pll_influence: FloatParam::new(
                "PLL Influence".to_string(),
                0.5,
                FloatRange::Linear { min: 0.0, max: 1.0 }
            ).with_smoother(SmoothingStyle::Linear(50.0)),
            synth_pll_volume: FloatParam::new(
                "PLL Volume".to_string(),
                0.0,
                FloatRange::Linear { min: 0.0, max: 1.0 }
            ).with_smoother(SmoothingStyle::Linear(50.0)),
            synth_pll_stereo_damp_offset: FloatParam::new(
                "PLL Stereo Damp Δ".to_string(),
                0.0,
                FloatRange::Linear { min: 0.0, max: 0.5 }
            ).with_smoother(SmoothingStyle::Linear(50.0)),
            synth_pll_glide: FloatParam::new(
                "PLL Glide".to_string(),
                0.0,
                FloatRange::Skewed { min: 0.0, max: 2000.0, factor: 0.3 }
            ).with_smoother(SmoothingStyle::Linear(50.0)),

            // New PLL experimental parameters
            synth_pll_retrigger: FloatParam::new(
                "PLL Retrigger".to_string(),
                1.0,
                FloatRange::Linear { min: 0.0, max: 1.0 }
            ).with_smoother(SmoothingStyle::Linear(20.0)),
            synth_pll_burst_threshold: FloatParam::new(
                "PLL Burst Threshold".to_string(),
                0.7,
                FloatRange::Linear { min: 0.0, max: 1.0 }
            ).with_smoother(SmoothingStyle::Linear(50.0)),
            synth_pll_burst_amount: FloatParam::new(
                "PLL Burst Amount".to_string(),
                3.3,
                FloatRange::Linear { min: 0.0, max: 10.0 }
            ).with_smoother(SmoothingStyle::Linear(50.0)),
            synth_pll_loop_saturation: FloatParam::new(
                "PLL Loop Limit".to_string(),
                100.0,
                FloatRange::Skewed { min: 1.0, max: 500.0, factor: 0.5 }
            ).with_smoother(SmoothingStyle::Linear(50.0)),
            synth_pll_color_amount: FloatParam::new(
                "PLL Color Amount".to_string(),
                0.25,
                FloatRange::Linear { min: 0.0, max: 1.0 }
            ).with_smoother(SmoothingStyle::Linear(50.0)),
            synth_pll_edge_sensitivity: FloatParam::new(
                "PLL Edge Sensitivity".to_string(),
                0.02,
                FloatRange::Skewed { min: 0.001, max: 0.2, factor: 0.5 }
            ).with_smoother(SmoothingStyle::Linear(50.0)),
            synth_pll_range: FloatParam::new(
                "PLL Range".to_string(),
                1.0,
                FloatRange::Linear { min: 0.0, max: 1.0 }
            ).with_smoother(SmoothingStyle::Linear(50.0)),
            synth_pll_stereo_track_offset: FloatParam::new(
                "PLL Stereo Track Δ".to_string(),
                0.0,
                FloatRange::Linear { min: 0.0, max: 0.5 }
            ).with_smoother(SmoothingStyle::Linear(50.0)),
            synth_pll_stereo_phase: FloatParam::new(
                "PLL Stereo Phase".to_string(),
                0.0,
                FloatRange::Linear { min: 0.0, max: 0.5 }
            ).with_smoother(SmoothingStyle::Linear(50.0)),
            synth_pll_cross_feedback: FloatParam::new(
                "PLL Cross Feedback".to_string(),
                0.0,
                FloatRange::Linear { min: 0.0, max: 1.0 }
            ).with_smoother(SmoothingStyle::Linear(50.0)),
            synth_pll_fm_env_amount: FloatParam::new(
                "PLL FM Env Amount".to_string(),
                0.0,
                FloatRange::Linear { min: -1.0, max: 1.0 }
            ).with_smoother(SmoothingStyle::Linear(50.0)),

            synth_ring_mod: FloatParam::new(
                "Ring Mod".to_string(),
                0.0,
                FloatRange::Linear { min: 0.0, max: 1.0 }
            ).with_smoother(SmoothingStyle::Linear(50.0)),
            synth_wavefold: FloatParam::new(
                "Wavefold".to_string(),
                0.0,
                FloatRange::Linear { min: 0.0, max: 1.0 }
            ).with_smoother(SmoothingStyle::Linear(50.0)),

            synth_drift_amount: FloatParam::new(
                "Drift Amount".to_string(),
                0.0,
                FloatRange::Linear { min: 0.0, max: 1.0 }
            ).with_smoother(SmoothingStyle::Linear(50.0)),
            synth_drift_rate: FloatParam::new(
                "Drift Rate".to_string(),
                0.3,
                FloatRange::Skewed { min: 0.01, max: 5.0, factor: 0.5 }
            ).with_smoother(SmoothingStyle::Linear(50.0)),
            synth_noise_amount: FloatParam::new(
                "Noise Amount".to_string(),
                0.0,
                FloatRange::Linear { min: 0.0, max: 1.0 }
            ).with_smoother(SmoothingStyle::Linear(50.0)),
            synth_tube_drive: FloatParam::new(
                "Tube Drive".to_string(),
                0.0,
                FloatRange::Linear { min: 0.0, max: 1.0 }
            ).with_smoother(SmoothingStyle::Linear(50.0)),
            synth_color_distortion_amount: FloatParam::new(
                "Color Distortion Amount".to_string(),
                0.0,
                FloatRange::Linear { min: 0.0, max: 1.0 }
            ).with_smoother(SmoothingStyle::Linear(50.0)),
            synth_color_distortion_threshold: FloatParam::new(
                "Color Distortion Threshold".to_string(),
                0.7,
                FloatRange::Linear { min: 0.1, max: 1.0 }
            ).with_smoother(SmoothingStyle::Linear(50.0)),

            // Bypass switches (all enabled by default)
            synth_pll_enable: BoolParam::new("PLL Enable".to_string(), true),
            synth_vps_enable: BoolParam::new("VPS Enable".to_string(), true),
            synth_coloration_enable: BoolParam::new("Coloration Enable".to_string(), true),
            synth_reverb_enable: BoolParam::new("Reverb Enable".to_string(), true),
            synth_oversampling_factor: IntParam::new(
                "Oversampling Factor",
                0,  // Default to 1x (index 0: 1x=0, 2x=1, 4x=2, 8x=3, 16x=4)
                IntRange::Linear { min: 0, max: 4 },
            ),
            synth_base_rate: IntParam::new(
                "Base Sample Rate",
                0,  // 0=Auto, 1=44.1k, 2=88.2k, 3=96k, 4=192k
                IntRange::Linear { min: 0, max: 4 },
            ),

            synth_filter_enable: BoolParam::new(
                "Filter Enable".to_string(),
                true
            ),
            synth_filter_cutoff: FloatParam::new(
                "Filter Cutoff".to_string(),
                1000.0,
                FloatRange::Skewed { min: 20.0, max: 20000.0, factor: 0.3 }
            ).with_smoother(SmoothingStyle::Linear(50.0)),
            synth_filter_resonance: FloatParam::new(
                "Filter Resonance".to_string(),
                0.0,
                FloatRange::Linear { min: 0.0, max: 0.99 }
            ).with_smoother(SmoothingStyle::Linear(50.0)),
            synth_filter_env_amount: FloatParam::new(
                "Filter Env Amount".to_string(),
                0.0,
                FloatRange::Linear { min: -5000.0, max: 5000.0 }
            ).with_smoother(SmoothingStyle::Linear(50.0)),
            synth_filter_drive: FloatParam::new(
                "Filter Drive".to_string(),
                1.0,
                FloatRange::Linear { min: 1.0, max: 15.849 }
            ).with_smoother(SmoothingStyle::Linear(50.0)),

            global_volume: FloatParam::new(
                "Global Volume".to_string(),
                0.5,
                FloatRange::Linear { min: 0.0, max: 1.0 }
            ).with_smoother(SmoothingStyle::Linear(50.0)),

            limiter_enable: BoolParam::new("Limiter".to_string(), true),

            synth_vol_attack: FloatParam::new(
                "Vol Attack".to_string(),
                10.0,
                FloatRange::Linear { min: 1.0, max: 1000.0 }
            ).with_smoother(SmoothingStyle::Linear(50.0)),
            synth_vol_attack_shape: FloatParam::new(
                "Vol Attack Shape".to_string(),
                0.5,
                FloatRange::Linear { min: 0.0, max: 1.0 }
            ).with_smoother(SmoothingStyle::Linear(50.0)),
            synth_vol_decay: FloatParam::new(
                "Vol Decay".to_string(),
                100.0,
                FloatRange::Linear { min: 1.0, max: 1000.0 }
            ).with_smoother(SmoothingStyle::Linear(50.0)),
            synth_vol_decay_shape: FloatParam::new(
                "Vol Decay Shape".to_string(),
                0.5,
                FloatRange::Linear { min: 0.0, max: 1.0 }
            ).with_smoother(SmoothingStyle::Linear(50.0)),
            synth_vol_sustain: FloatParam::new(
                "Vol Sustain".to_string(),
                0.7,
                FloatRange::Linear { min: 0.0, max: 1.0 }
            ).with_smoother(SmoothingStyle::Linear(50.0)),
            synth_vol_release: FloatParam::new(
                "Vol Release".to_string(),
                200.0,
                FloatRange::Linear { min: 1.0, max: 1000.0 }
            ).with_smoother(SmoothingStyle::Linear(50.0)),
            synth_vol_release_shape: FloatParam::new(
                "Vol Release Shape".to_string(),
                0.5,
                FloatRange::Linear { min: 0.0, max: 1.0 }
            ).with_smoother(SmoothingStyle::Linear(50.0)),

            synth_filt_attack: FloatParam::new(
                "Filt Attack".to_string(),
                10.0,
                FloatRange::Linear { min: 1.0, max: 1000.0 }
            ).with_smoother(SmoothingStyle::Linear(50.0)),
            synth_filt_attack_shape: FloatParam::new(
                "Filt Attack Shape".to_string(),
                0.5,
                FloatRange::Linear { min: 0.0, max: 1.0 }
            ).with_smoother(SmoothingStyle::Linear(50.0)),
            synth_filt_decay: FloatParam::new(
                "Filt Decay".to_string(),
                100.0,
                FloatRange::Linear { min: 1.0, max: 1000.0 }
            ).with_smoother(SmoothingStyle::Linear(50.0)),
            synth_filt_decay_shape: FloatParam::new(
                "Filt Decay Shape".to_string(),
                0.5,
                FloatRange::Linear { min: 0.0, max: 1.0 }
            ).with_smoother(SmoothingStyle::Linear(50.0)),
            synth_filt_sustain: FloatParam::new(
                "Filt Sustain".to_string(),
                0.5,
                FloatRange::Linear { min: 0.0, max: 1.0 }
            ).with_smoother(SmoothingStyle::Linear(50.0)),
            synth_filt_release: FloatParam::new(
                "Filt Release".to_string(),
                200.0,
                FloatRange::Linear { min: 1.0, max: 1000.0 }
            ).with_smoother(SmoothingStyle::Linear(50.0)),
            synth_filt_release_shape: FloatParam::new(
                "Filt Release Shape".to_string(),
                0.5,
                FloatRange::Linear { min: 0.0, max: 1.0 }
            ).with_smoother(SmoothingStyle::Linear(50.0)),

            synth_reverb_mix: FloatParam::new(
                "Reverb Dry/Wet".to_string(),
                0.0,
                FloatRange::Linear { min: 0.0, max: 1.0 }
            ).with_smoother(SmoothingStyle::Linear(50.0)),
            synth_reverb_pre_delay: FloatParam::new(
                "Reverb Pre-Delay".to_string(),
                50.0,
                FloatRange::Linear { min: 0.0, max: 500.0 }
            ).with_smoother(SmoothingStyle::Linear(50.0)),
            synth_reverb_time_scale: FloatParam::new(
                "Reverb Size".to_string(),
                0.85,
                FloatRange::Linear { min: 0.0, max: 1.0 }
            ).with_smoother(SmoothingStyle::Linear(50.0)),
            synth_reverb_input_hpf: FloatParam::new(
                "Reverb Input HPF".to_string(),
                20.0,
                FloatRange::Skewed { min: 20.0, max: 22000.0, factor: 0.25 }
            ).with_smoother(SmoothingStyle::Linear(50.0)),
            synth_reverb_input_lpf: FloatParam::new(
                "Reverb Input LPF".to_string(),
                18000.0,
                FloatRange::Skewed { min: 20.0, max: 22000.0, factor: 0.25 }
            ).with_smoother(SmoothingStyle::Linear(50.0)),
            synth_reverb_hpf: FloatParam::new(
                "Reverb HPF".to_string(),
                100.0,
                FloatRange::Skewed { min: 20.0, max: 22000.0, factor: 0.25 }
            ).with_smoother(SmoothingStyle::Linear(50.0)),
            synth_reverb_lpf: FloatParam::new(
                "Reverb LPF".to_string(),
                14000.0,
                FloatRange::Skewed { min: 20.0, max: 22000.0, factor: 0.25 }
            ).with_smoother(SmoothingStyle::Linear(50.0)),
            synth_reverb_mod_speed: FloatParam::new(
                "Reverb Mod Speed".to_string(),
                0.3,
                FloatRange::Linear { min: 0.0, max: 1.0 }
            ).with_smoother(SmoothingStyle::Linear(50.0)),
            synth_reverb_mod_depth: FloatParam::new(
                "Reverb Mod Depth".to_string(),
                0.4,
                FloatRange::Linear { min: 0.0, max: 1.0 }
            ).with_smoother(SmoothingStyle::Linear(50.0)),
            synth_reverb_mod_shape: FloatParam::new(
                "Reverb Mod Shape".to_string(),
                0.5,
                FloatRange::Linear { min: 0.0, max: 1.0 }
            ).with_smoother(SmoothingStyle::Linear(50.0)),
            synth_reverb_diffusion_mix: FloatParam::new(
                "Reverb Diffusion Mix".to_string(),
                0.85,
                FloatRange::Linear { min: 0.0, max: 1.0 }
            ).with_smoother(SmoothingStyle::Linear(50.0)),
            synth_reverb_diffusion: FloatParam::new(
                "Reverb Diffusion".to_string(),
                0.75,
                FloatRange::Linear { min: 0.0, max: 1.0 }
            ).with_smoother(SmoothingStyle::Linear(50.0)),
            synth_reverb_decay: FloatParam::new(
                "Reverb Decay".to_string(),
                0.8,
                FloatRange::Linear { min: 0.0, max: 1.0 }
            ).with_smoother(SmoothingStyle::Linear(50.0)),
            synth_reverb_ducking: FloatParam::new(
                "Reverb Ducking".to_string(),
                0.0,
                FloatRange::Linear { min: 0.0, max: 1.0 }
            ).with_smoother(SmoothingStyle::Linear(50.0)),

            // LFO 1
            lfo1_rate: FloatParam::new(
                "LFO 1 Rate".to_string(),
                1.0,
                FloatRange::Skewed { min: 0.01, max: 50.0, factor: 0.3 }
            ).with_smoother(SmoothingStyle::Linear(20.0)),
            lfo1_waveform: IntParam::new("LFO 1 Waveform".to_string(), 0, IntRange::Linear { min: 0, max: 4 }),
            lfo1_tempo_sync: BoolParam::new("LFO 1 Tempo Sync".to_string(), false),
            lfo1_sync_division: IntParam::new("LFO 1 Division".to_string(), 2, IntRange::Linear { min: 0, max: 15 }),
            lfo1_sync_source: IntParam::new("LFO 1 Sync Source".to_string(), -1, IntRange::Linear { min: -1, max: 2 }),
            lfo1_phase_mod: FloatParam::new(
                "LFO 1 Phase Mod".to_string(),
                0.0,
                FloatRange::Linear { min: 0.0, max: 1.0 }
            ).with_smoother(SmoothingStyle::Linear(20.0)),
            lfo1_dest1: IntParam::new("LFO 1 Dest 1".to_string(), 0, IntRange::Linear { min: 0, max: 30 }),
            lfo1_amount1: FloatParam::new(
                "LFO 1 Amount 1".to_string(),
                0.0,
                FloatRange::Linear { min: -1.0, max: 1.0 }
            ).with_smoother(SmoothingStyle::Linear(20.0)),
            lfo1_dest2: IntParam::new("LFO 1 Dest 2".to_string(), 0, IntRange::Linear { min: 0, max: 30 }),
            lfo1_amount2: FloatParam::new(
                "LFO 1 Amount 2".to_string(),
                0.0,
                FloatRange::Linear { min: -1.0, max: 1.0 }
            ).with_smoother(SmoothingStyle::Linear(20.0)),

            // LFO 2
            lfo2_rate: FloatParam::new(
                "LFO 2 Rate".to_string(),
                1.0,
                FloatRange::Skewed { min: 0.01, max: 50.0, factor: 0.3 }
            ).with_smoother(SmoothingStyle::Linear(20.0)),
            lfo2_waveform: IntParam::new("LFO 2 Waveform".to_string(), 0, IntRange::Linear { min: 0, max: 4 }),
            lfo2_tempo_sync: BoolParam::new("LFO 2 Tempo Sync".to_string(), false),
            lfo2_sync_division: IntParam::new("LFO 2 Division".to_string(), 3, IntRange::Linear { min: 0, max: 15 }),
            lfo2_sync_source: IntParam::new("LFO 2 Sync Source".to_string(), -1, IntRange::Linear { min: -1, max: 2 }),
            lfo2_phase_mod: FloatParam::new(
                "LFO 2 Phase Mod".to_string(),
                0.0,
                FloatRange::Linear { min: 0.0, max: 1.0 }
            ).with_smoother(SmoothingStyle::Linear(20.0)),
            lfo2_dest1: IntParam::new("LFO 2 Dest 1".to_string(), 0, IntRange::Linear { min: 0, max: 30 }),
            lfo2_amount1: FloatParam::new(
                "LFO 2 Amount 1".to_string(),
                0.0,
                FloatRange::Linear { min: -1.0, max: 1.0 }
            ).with_smoother(SmoothingStyle::Linear(20.0)),
            lfo2_dest2: IntParam::new("LFO 2 Dest 2".to_string(), 0, IntRange::Linear { min: 0, max: 30 }),
            lfo2_amount2: FloatParam::new(
                "LFO 2 Amount 2".to_string(),
                0.0,
                FloatRange::Linear { min: -1.0, max: 1.0 }
            ).with_smoother(SmoothingStyle::Linear(20.0)),

            // LFO 3
            lfo3_rate: FloatParam::new(
                "LFO 3 Rate".to_string(),
                1.0,
                FloatRange::Skewed { min: 0.01, max: 50.0, factor: 0.3 }
            ).with_smoother(SmoothingStyle::Linear(20.0)),
            lfo3_waveform: IntParam::new("LFO 3 Waveform".to_string(), 0, IntRange::Linear { min: 0, max: 4 }),
            lfo3_tempo_sync: BoolParam::new("LFO 3 Tempo Sync".to_string(), false),
            lfo3_sync_division: IntParam::new("LFO 3 Division".to_string(), 0, IntRange::Linear { min: 0, max: 15 }),
            lfo3_sync_source: IntParam::new("LFO 3 Sync Source".to_string(), -1, IntRange::Linear { min: -1, max: 2 }),
            lfo3_phase_mod: FloatParam::new(
                "LFO 3 Phase Mod".to_string(),
                0.0,
                FloatRange::Linear { min: 0.0, max: 1.0 }
            ).with_smoother(SmoothingStyle::Linear(20.0)),
            lfo3_dest1: IntParam::new("LFO 3 Dest 1".to_string(), 0, IntRange::Linear { min: 0, max: 30 }),
            lfo3_amount1: FloatParam::new(
                "LFO 3 Amount 1".to_string(),
                0.0,
                FloatRange::Linear { min: -1.0, max: 1.0 }
            ).with_smoother(SmoothingStyle::Linear(20.0)),
            lfo3_dest2: IntParam::new("LFO 3 Dest 2".to_string(), 0, IntRange::Linear { min: 0, max: 30 }),
            lfo3_amount2: FloatParam::new(
                "LFO 3 Amount 2".to_string(),
                0.0,
                FloatRange::Linear { min: -1.0, max: 1.0 }
            ).with_smoother(SmoothingStyle::Linear(20.0)),

            mseq_step_1: FloatParam::new("MSeq Step 1", 0.0, FloatRange::Linear { min: -1.0, max: 1.0 }),
            mseq_step_2: FloatParam::new("MSeq Step 2", 0.0, FloatRange::Linear { min: -1.0, max: 1.0 }),
            mseq_step_3: FloatParam::new("MSeq Step 3", 0.0, FloatRange::Linear { min: -1.0, max: 1.0 }),
            mseq_step_4: FloatParam::new("MSeq Step 4", 0.0, FloatRange::Linear { min: -1.0, max: 1.0 }),
            mseq_step_5: FloatParam::new("MSeq Step 5", 0.0, FloatRange::Linear { min: -1.0, max: 1.0 }),
            mseq_step_6: FloatParam::new("MSeq Step 6", 0.0, FloatRange::Linear { min: -1.0, max: 1.0 }),
            mseq_step_7: FloatParam::new("MSeq Step 7", 0.0, FloatRange::Linear { min: -1.0, max: 1.0 }),
            mseq_step_8: FloatParam::new("MSeq Step 8", 0.0, FloatRange::Linear { min: -1.0, max: 1.0 }),
            mseq_step_9: FloatParam::new("MSeq Step 9", 0.0, FloatRange::Linear { min: -1.0, max: 1.0 }),
            mseq_step_10: FloatParam::new("MSeq Step 10", 0.0, FloatRange::Linear { min: -1.0, max: 1.0 }),
            mseq_step_11: FloatParam::new("MSeq Step 11", 0.0, FloatRange::Linear { min: -1.0, max: 1.0 }),
            mseq_step_12: FloatParam::new("MSeq Step 12", 0.0, FloatRange::Linear { min: -1.0, max: 1.0 }),
            mseq_step_13: FloatParam::new("MSeq Step 13", 0.0, FloatRange::Linear { min: -1.0, max: 1.0 }),
            mseq_step_14: FloatParam::new("MSeq Step 14", 0.0, FloatRange::Linear { min: -1.0, max: 1.0 }),
            mseq_step_15: FloatParam::new("MSeq Step 15", 0.0, FloatRange::Linear { min: -1.0, max: 1.0 }),
            mseq_step_16: FloatParam::new("MSeq Step 16", 0.0, FloatRange::Linear { min: -1.0, max: 1.0 }),
            mseq_ties: IntParam::new("MSeq Ties", 0, IntRange::Linear { min: 0, max: 65535 }),
            mseq_division: IntParam::new("MSeq Division", 3, IntRange::Linear { min: 0, max: 15 }),
            mseq_slew: FloatParam::new(
                "MSeq Slew", 5.0, FloatRange::Linear { min: 0.0, max: 200.0 }
            ).with_smoother(SmoothingStyle::Linear(20.0)),
            mseq_dest1: IntParam::new("MSeq Dest 1", 0, IntRange::Linear { min: 0, max: 30 }),
            mseq_amount1: FloatParam::new(
                "MSeq Amount 1", 0.0, FloatRange::Linear { min: -1.0, max: 1.0 }
            ).with_smoother(SmoothingStyle::Linear(20.0)),
            mseq_dest2: IntParam::new("MSeq Dest 2", 0, IntRange::Linear { min: 0, max: 30 }),
            mseq_amount2: FloatParam::new(
                "MSeq Amount 2", 0.0, FloatRange::Linear { min: -1.0, max: 1.0 }
            ).with_smoother(SmoothingStyle::Linear(20.0)),

            note_length_percent: FloatParam::new(
                "Note Length %".to_string(),
                95.0,
                FloatRange::Linear { min: 1.0, max: 200.0 }
            ),

            len_mod_1_target: FloatParam::new(
                "Len Mod 1 Target",
                -75.0,
                FloatRange::Linear { min: -100.0, max: 100.0 }
            ),
            len_mod_1_amount: FloatParam::new(
                "Len Mod 1 Amount",
                100.0,
                FloatRange::Linear { min: 0.0, max: 200.0 }
            ),
            len_mod_1_prob: FloatParam::new(
                "Len Mod 1 Prob",
                0.0,
                FloatRange::Linear { min: 0.0, max: 127.0 }
            ),

            len_mod_2_target: FloatParam::new(
                "Len Mod 2 Target",
                75.0,
                FloatRange::Linear { min: -100.0, max: 100.0 }
            ),
            len_mod_2_amount: FloatParam::new(
                "Len Mod 2 Amount",
                100.0,
                FloatRange::Linear { min: 0.0, max: 200.0 }
            ),
            len_mod_2_prob: FloatParam::new(
                "Len Mod 2 Prob",
                0.0,
                FloatRange::Linear { min: 0.0, max: 127.0 }
            ),

            vel_strength_target: FloatParam::new(
                "Vel Strength Target",
                0.0,
                FloatRange::Linear { min: -100.0, max: 100.0 }
            ),
            vel_strength_amount: FloatParam::new(
                "Vel Strength Amount",
                0.0,
                FloatRange::Linear { min: -99.0, max: 27.0 }
            ),
            vel_strength_prob: FloatParam::new(
                "Vel Strength Prob",
                0.0,
                FloatRange::Linear { min: 0.0, max: 127.0 }
            ),

            vel_length_target: FloatParam::new(
                "Vel Length Target",
                0.0,
                FloatRange::Linear { min: -100.0, max: 100.0 }
            ),
            vel_length_amount: FloatParam::new(
                "Vel Length Amount",
                0.0,
                FloatRange::Linear { min: -99.0, max: 27.0 }
            ),
            vel_length_prob: FloatParam::new(
                "Vel Length Prob",
                0.0,
                FloatRange::Linear { min: 0.0, max: 127.0 }
            ),

            pos_mod_1_target: FloatParam::new(
                "Pos Mod 1 Target",
                -75.0,
                FloatRange::Linear { min: -100.0, max: 100.0 }
            ),
            pos_mod_1_shift: FloatParam::new(
                "Pos Mod 1 Shift",
                0.0,
                FloatRange::Linear { min: -50.0, max: 50.0 }
            ),
            pos_mod_1_prob: FloatParam::new(
                "Pos Mod 1 Prob",
                0.0,
                FloatRange::Linear { min: 0.0, max: 127.0 }
            ),

            pos_mod_2_target: FloatParam::new(
                "Pos Mod 2 Target",
                75.0,
                FloatRange::Linear { min: -100.0, max: 100.0 }
            ),
            pos_mod_2_shift: FloatParam::new(
                "Pos Mod 2 Shift",
                0.0,
                FloatRange::Linear { min: -50.0, max: 50.0 }
            ),
            pos_mod_2_prob: FloatParam::new(
                "Pos Mod 2 Prob",
                0.0,
                FloatRange::Linear { min: 0.0, max: 127.0 }
            ),

            swing_amount: FloatParam::new(
                "Swing".to_string(),
                50.0,
                FloatRange::Linear { min: 50.0, max: 75.0 }
            ),

            legato_mode: BoolParam::new("Legato", false),
            legato_time: FloatParam::new(
                "Legato Time",
                50.0,
                FloatRange::Skewed { min: 1.0, max: 500.0, factor: 0.4 }
            ).with_unit(" ms"),

            sequencer_enable: BoolParam::new("Sequencer Enable", false),
        }
    }
}
