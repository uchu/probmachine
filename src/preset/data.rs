use serde::{Deserialize, Serialize};
use crate::sequencer::scales::{Scale, StabilityPattern, OctaveDirection};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NotePresetData {
    pub midi_note: u8,
    pub chance: u8,
    pub beat: u8,
    pub beat_length: u8,
    #[serde(default)]
    pub octave_offset: i8,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OctaveRandomizationPresetData {
    #[serde(default)]
    pub chance: u8,
    #[serde(default = "default_pref")]
    pub strength_pref: u8,
    #[serde(default = "default_pref")]
    pub length_pref: u8,
    #[serde(default)]
    pub direction: OctaveDirection,
}

impl Default for OctaveRandomizationPresetData {
    fn default() -> Self {
        Self {
            chance: 0,
            strength_pref: 64,
            length_pref: 64,
            direction: OctaveDirection::Both,
        }
    }
}

fn default_pref() -> u8 { 64 }

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PresetData {
    pub straight_1_1: [f32; 1],
    pub straight_1_2: [f32; 2],
    pub straight_1_4: [f32; 4],
    pub straight_1_8: [f32; 8],
    pub straight_1_16: [f32; 16],
    pub straight_1_32: [f32; 32],

    pub triplet_1_2t: [f32; 3],
    pub triplet_1_4t: [f32; 6],
    pub triplet_1_8t: [f32; 12],
    pub triplet_1_16t: [f32; 24],

    pub dotted_1_2d: [f32; 2],
    pub dotted_1_4d: [f32; 3],
    pub dotted_1_8d: [f32; 6],
    pub dotted_1_16d: [f32; 11],
    pub dotted_1_32d: [f32; 22],

    pub strength_values: Vec<u8>,

    pub root_note: u8,
    pub notes: Vec<NotePresetData>,

    #[serde(default)]
    pub scale: Scale,
    #[serde(default)]
    pub stability_pattern: StabilityPattern,
    #[serde(default)]
    pub octave_randomization: OctaveRandomizationPresetData,

    pub synth_pll_track_speed: f32,
    pub synth_pll_damping: f32,
    pub synth_pll_influence: f32,
    pub synth_pll_mult: i32,
    pub synth_pll_colored: bool,
    pub synth_pll_mode: bool,
    pub synth_pll_ref_octave: i32,
    #[serde(default)]
    pub synth_pll_ref_tune: i32,
    #[serde(default)]
    pub synth_pll_ref_fine: f32,
    pub synth_pll_ref_pulse_width: f32,
    pub synth_pll_feedback: f32,
    pub synth_pll_volume: f32,
    pub synth_pll_stereo_damp_offset: f32,
    pub synth_pll_glide: f32,
    pub synth_pll_fm_amount: f32,
    pub synth_pll_fm_ratio: i32,
    #[serde(default)]
    pub synth_pll_retrigger: f32,
    #[serde(default = "default_burst_threshold")]
    pub synth_pll_burst_threshold: f32,
    #[serde(default)]
    pub synth_pll_burst_amount: f32,
    #[serde(default = "default_loop_saturation")]
    pub synth_pll_loop_saturation: f32,
    #[serde(default)]
    pub synth_pll_color_amount: f32,
    #[serde(default = "default_edge_sensitivity")]
    pub synth_pll_edge_sensitivity: f32,
    #[serde(default = "default_pll_range")]
    pub synth_pll_range: f32,
    #[serde(default)]
    pub synth_pll_stereo_track_offset: f32,
    #[serde(default)]
    pub synth_pll_stereo_phase: f32,
    #[serde(default)]
    pub synth_pll_cross_feedback: f32,
    #[serde(default)]
    pub synth_pll_fm_env_amount: f32,
    #[serde(default = "default_true")]
    pub synth_pll_enable: bool,
    #[serde(default = "default_true")]
    pub synth_pll_mult_slew: bool,

    pub synth_osc_octave: i32,
    #[serde(default)]
    pub synth_osc_tune: i32,
    #[serde(default)]
    pub synth_osc_fine: f32,
    #[serde(default)]
    pub synth_osc_fold: f32,
    pub synth_osc_d: f32,
    pub synth_osc_v: f32,
    pub synth_osc_stereo_v_offset: f32,
    #[serde(default)]
    pub synth_osc_stereo_d_offset: f32,
    #[serde(default)]
    pub synth_vps_shape_type: i32,
    #[serde(default)]
    pub synth_vps_shape_amount: f32,
    #[serde(default)]
    pub synth_vps_phase_mode: i32,
    pub synth_osc_volume: f32,

    pub synth_sub_volume: f32,
    #[serde(default)]
    pub synth_sub_source: i32,

    pub synth_filter_enable: bool,
    pub synth_filter_cutoff: f32,
    pub synth_filter_resonance: f32,
    pub synth_filter_env_amount: f32,
    pub synth_filter_drive: f32,

    pub synth_vol_attack: f32,
    pub synth_vol_decay: f32,
    pub synth_vol_sustain: f32,
    pub synth_vol_release: f32,

    pub synth_filt_attack: f32,
    pub synth_filt_decay: f32,
    pub synth_filt_sustain: f32,
    pub synth_filt_release: f32,

    pub synth_reverb_mix: f32,
    pub synth_reverb_time_scale: f32,
    pub synth_reverb_decay: f32,
    pub synth_reverb_diffusion: f32,
    pub synth_reverb_pre_delay: f32,
    pub synth_reverb_mod_depth: f32,
    pub synth_reverb_hpf: f32,
    pub synth_reverb_lpf: f32,
    pub synth_reverb_ducking: f32,
    #[serde(default = "default_reverb_input_hpf")]
    pub synth_reverb_input_hpf: f32,
    #[serde(default = "default_reverb_input_lpf")]
    pub synth_reverb_input_lpf: f32,
    #[serde(default = "default_reverb_mod_shape")]
    pub synth_reverb_mod_shape: f32,

    #[serde(default)]
    pub lfo1_rate: f32,
    #[serde(default)]
    pub lfo1_waveform: i32,
    #[serde(default)]
    pub lfo1_tempo_sync: bool,
    #[serde(default)]
    pub lfo1_sync_division: i32,
    #[serde(default)]
    pub lfo1_sync_source: i32,
    #[serde(default)]
    pub lfo1_phase_mod: f32,
    #[serde(default)]
    pub lfo1_dest1: i32,
    #[serde(default)]
    pub lfo1_amount1: f32,
    #[serde(default)]
    pub lfo1_dest2: i32,
    #[serde(default)]
    pub lfo1_amount2: f32,

    #[serde(default)]
    pub lfo2_rate: f32,
    #[serde(default)]
    pub lfo2_waveform: i32,
    #[serde(default)]
    pub lfo2_tempo_sync: bool,
    #[serde(default)]
    pub lfo2_sync_division: i32,
    #[serde(default)]
    pub lfo2_sync_source: i32,
    #[serde(default)]
    pub lfo2_phase_mod: f32,
    #[serde(default)]
    pub lfo2_dest1: i32,
    #[serde(default)]
    pub lfo2_amount1: f32,
    #[serde(default)]
    pub lfo2_dest2: i32,
    #[serde(default)]
    pub lfo2_amount2: f32,

    #[serde(default)]
    pub lfo3_rate: f32,
    #[serde(default)]
    pub lfo3_waveform: i32,
    #[serde(default)]
    pub lfo3_tempo_sync: bool,
    #[serde(default)]
    pub lfo3_sync_division: i32,
    #[serde(default)]
    pub lfo3_sync_source: i32,
    #[serde(default)]
    pub lfo3_phase_mod: f32,
    #[serde(default)]
    pub lfo3_dest1: i32,
    #[serde(default)]
    pub lfo3_amount1: f32,
    #[serde(default)]
    pub lfo3_dest2: i32,
    #[serde(default)]
    pub lfo3_amount2: f32,

    #[serde(default = "default_swing")]
    pub swing_amount: f32,
    #[serde(default = "default_note_length")]
    pub note_length_percent: f32,

    #[serde(default)]
    pub legato_mode: bool,
    #[serde(default = "default_legato_time")]
    pub legato_time: f32,

    #[serde(default)]
    pub len_mod_1_target: f32,
    #[serde(default = "default_mod_amount")]
    pub len_mod_1_amount: f32,
    #[serde(default)]
    pub len_mod_1_prob: f32,

    #[serde(default)]
    pub len_mod_2_target: f32,
    #[serde(default = "default_mod_amount")]
    pub len_mod_2_amount: f32,
    #[serde(default)]
    pub len_mod_2_prob: f32,

    #[serde(default)]
    pub vel_strength_target: f32,
    #[serde(default)]
    pub vel_strength_amount: f32,
    #[serde(default)]
    pub vel_strength_prob: f32,

    #[serde(default)]
    pub vel_length_target: f32,
    #[serde(default)]
    pub vel_length_amount: f32,
    #[serde(default)]
    pub vel_length_prob: f32,

    #[serde(default)]
    pub pos_mod_1_target: f32,
    #[serde(default)]
    pub pos_mod_1_shift: f32,
    #[serde(default)]
    pub pos_mod_1_prob: f32,

    #[serde(default)]
    pub pos_mod_2_target: f32,
    #[serde(default)]
    pub pos_mod_2_shift: f32,
    #[serde(default)]
    pub pos_mod_2_prob: f32,

    #[serde(default)]
    pub synth_ring_mod: f32,
    #[serde(default)]
    pub synth_wavefold: f32,
    #[serde(default)]
    pub synth_drift_amount: f32,
    #[serde(default = "default_drift_rate")]
    pub synth_drift_rate: f32,
    #[serde(default)]
    pub synth_noise_amount: f32,
    #[serde(default = "default_tube_drive")]
    pub synth_tube_drive: f32,
    #[serde(default)]
    pub synth_color_distortion_amount: f32,
    #[serde(default = "default_distortion_threshold")]
    pub synth_color_distortion_threshold: f32,
    #[serde(default = "default_true")]
    pub synth_vps_enable: bool,
    #[serde(default = "default_true")]
    pub synth_coloration_enable: bool,
    #[serde(default = "default_true")]
    pub synth_reverb_enable: bool,

    #[serde(default)]
    pub mseq_steps: Vec<f32>,
    #[serde(default)]
    pub mseq_ties: i32,
    #[serde(default = "default_mseq_division")]
    pub mseq_division: i32,
    #[serde(default = "default_mseq_slew")]
    pub mseq_slew: f32,
    #[serde(default)]
    pub mseq_dest1: i32,
    #[serde(default)]
    pub mseq_amount1: f32,
    #[serde(default)]
    pub mseq_dest2: i32,
    #[serde(default)]
    pub mseq_amount2: f32,
}

fn default_swing() -> f32 { 50.0 }
fn default_legato_time() -> f32 { 50.0 }
fn default_drift_rate() -> f32 { 0.5 }
fn default_tube_drive() -> f32 { 0.0 }
fn default_distortion_threshold() -> f32 { 0.7 }
fn default_true() -> bool { true }
fn default_note_length() -> f32 { 95.0 }
fn default_mod_amount() -> f32 { 100.0 }
fn default_burst_threshold() -> f32 { 0.7 }
fn default_loop_saturation() -> f32 { 100.0 }
fn default_edge_sensitivity() -> f32 { 0.02 }
fn default_pll_range() -> f32 { 1.0 }
fn default_reverb_input_hpf() -> f32 { 20.0 }
fn default_reverb_input_lpf() -> f32 { 18000.0 }
fn default_reverb_mod_shape() -> f32 { 0.5 }
fn default_mseq_division() -> i32 { 3 }
fn default_mseq_slew() -> f32 { 5.0 }

impl Default for PresetData {
    fn default() -> Self {
        Self {
            straight_1_1: [0.0],
            straight_1_2: [0.0; 2],
            straight_1_4: [0.0; 4],
            straight_1_8: [0.0; 8],
            straight_1_16: [0.0; 16],
            straight_1_32: [0.0; 32],

            triplet_1_2t: [0.0; 3],
            triplet_1_4t: [0.0; 6],
            triplet_1_8t: [0.0; 12],
            triplet_1_16t: [0.0; 24],

            dotted_1_2d: [0.0; 2],
            dotted_1_4d: [0.0; 3],
            dotted_1_8d: [0.0; 6],
            dotted_1_16d: [0.0; 11],
            dotted_1_32d: [0.0; 22],

            strength_values: {
                let mut arr = vec![0u8; 96];
                arr[0] = 100;
                arr[24] = 75;
                arr[48] = 75;
                arr[72] = 75;
                arr[12] = 50;
                arr[36] = 50;
                arr[60] = 50;
                arr[84] = 50;
                arr
            },

            root_note: 48,
            notes: vec![],

            scale: Scale::default(),
            stability_pattern: StabilityPattern::default(),
            octave_randomization: OctaveRandomizationPresetData::default(),

            synth_pll_track_speed: 0.5,
            synth_pll_damping: 0.3,
            synth_pll_influence: 0.5,
            synth_pll_mult: 0,
            synth_pll_colored: false,
            synth_pll_mode: true,
            synth_pll_ref_octave: 0,
            synth_pll_ref_tune: 0,
            synth_pll_ref_fine: 0.0,
            synth_pll_ref_pulse_width: 0.5,
            synth_pll_feedback: 0.0,
            synth_pll_volume: 0.0,
            synth_pll_stereo_damp_offset: 0.0,
            synth_pll_glide: 0.0,
            synth_pll_fm_amount: 0.0,
            synth_pll_fm_ratio: 1,
            synth_pll_retrigger: 1.0,
            synth_pll_burst_threshold: 0.7,
            synth_pll_burst_amount: 0.0,
            synth_pll_loop_saturation: 1.0,
            synth_pll_color_amount: 0.0,
            synth_pll_edge_sensitivity: 0.5,
            synth_pll_range: 1.0,
            synth_pll_stereo_track_offset: 0.0,
            synth_pll_stereo_phase: 0.0,
            synth_pll_cross_feedback: 0.0,
            synth_pll_fm_env_amount: 0.0,
            synth_pll_enable: true,

            synth_osc_octave: 0,
            synth_osc_tune: 0,
            synth_osc_fine: 0.0,
            synth_osc_fold: 0.0,
            synth_osc_d: 0.5,
            synth_osc_v: 0.5,
            synth_osc_stereo_v_offset: 0.0,
            synth_osc_stereo_d_offset: 0.0,
            synth_vps_shape_type: 0,
            synth_vps_shape_amount: 0.0,
            synth_vps_phase_mode: 0,
            synth_osc_volume: 0.0,

            synth_sub_volume: 0.0,
            synth_sub_source: 0,

            synth_filter_enable: false,
            synth_filter_cutoff: 1000.0,
            synth_filter_resonance: 0.0,
            synth_filter_env_amount: 0.0,
            synth_filter_drive: 1.0,

            synth_vol_attack: 10.0,
            synth_vol_decay: 100.0,
            synth_vol_sustain: 0.7,
            synth_vol_release: 200.0,

            synth_filt_attack: 10.0,
            synth_filt_decay: 100.0,
            synth_filt_sustain: 0.5,
            synth_filt_release: 200.0,

            synth_reverb_mix: 0.0,
            synth_reverb_time_scale: 0.5,
            synth_reverb_decay: 0.5,
            synth_reverb_diffusion: 0.7,
            synth_reverb_pre_delay: 20.0,
            synth_reverb_mod_depth: 0.2,
            synth_reverb_hpf: 100.0,
            synth_reverb_lpf: 8000.0,
            synth_reverb_ducking: 0.0,
            synth_reverb_input_hpf: 20.0,
            synth_reverb_input_lpf: 18000.0,
            synth_reverb_mod_shape: 0.5,

            lfo1_rate: 1.0,
            lfo1_waveform: 0,
            lfo1_tempo_sync: false,
            lfo1_sync_division: 2,
            lfo1_sync_source: -1,
            lfo1_phase_mod: 0.0,
            lfo1_dest1: 0,
            lfo1_amount1: 0.0,
            lfo1_dest2: 0,
            lfo1_amount2: 0.0,

            lfo2_rate: 1.0,
            lfo2_waveform: 0,
            lfo2_tempo_sync: false,
            lfo2_sync_division: 2,
            lfo2_sync_source: -1,
            lfo2_phase_mod: 0.0,
            lfo2_dest1: 0,
            lfo2_amount1: 0.0,
            lfo2_dest2: 0,
            lfo2_amount2: 0.0,

            lfo3_rate: 1.0,
            lfo3_waveform: 0,
            lfo3_tempo_sync: false,
            lfo3_sync_division: 2,
            lfo3_sync_source: -1,
            lfo3_phase_mod: 0.0,
            lfo3_dest1: 0,
            lfo3_amount1: 0.0,
            lfo3_dest2: 0,
            lfo3_amount2: 0.0,

            swing_amount: 50.0,
            note_length_percent: 95.0,

            legato_mode: false,
            legato_time: 50.0,

            len_mod_1_target: 0.0,
            len_mod_1_amount: 100.0,
            len_mod_1_prob: 0.0,

            len_mod_2_target: 0.0,
            len_mod_2_amount: 100.0,
            len_mod_2_prob: 0.0,

            vel_strength_target: 0.0,
            vel_strength_amount: 0.0,
            vel_strength_prob: 0.0,

            vel_length_target: 0.0,
            vel_length_amount: 0.0,
            vel_length_prob: 0.0,

            pos_mod_1_target: 0.0,
            pos_mod_1_shift: 0.0,
            pos_mod_1_prob: 0.0,

            pos_mod_2_target: 0.0,
            pos_mod_2_shift: 0.0,
            pos_mod_2_prob: 0.0,

            synth_ring_mod: 0.0,
            synth_wavefold: 0.0,
            synth_drift_amount: 0.0,
            synth_drift_rate: 0.5,
            synth_noise_amount: 0.0,
            synth_tube_drive: 1.0,
            synth_color_distortion_amount: 0.0,
            synth_color_distortion_threshold: 0.7,
            synth_vps_enable: true,
            synth_coloration_enable: true,
            synth_reverb_enable: true,
            synth_pll_mult_slew: true,

            mseq_steps: vec![0.0; 16],
            mseq_ties: 0,
            mseq_division: 3,
            mseq_slew: 5.0,
            mseq_dest1: 0,
            mseq_amount1: 0.0,
            mseq_dest2: 0,
            mseq_amount2: 0.0,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Preset {
    pub name: String,
    #[serde(default = "default_author")]
    pub author: String,
    #[serde(default)]
    pub description: String,
    pub data: PresetData,
}

fn default_author() -> String {
    "Factory".to_string()
}

impl Preset {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            author: "User".to_string(),
            description: String::new(),
            data: PresetData::default(),
        }
    }

    pub fn new_factory(name: &str) -> Self {
        Self {
            name: name.to_string(),
            author: "Factory".to_string(),
            description: String::new(),
            data: PresetData::default(),
        }
    }

    #[allow(dead_code)]
    pub fn with_data(name: &str, data: PresetData) -> Self {
        Self {
            name: name.to_string(),
            author: "User".to_string(),
            description: String::new(),
            data,
        }
    }

    pub fn with_author_and_description(name: &str, author: &str, description: &str, data: PresetData) -> Self {
        Self {
            name: name.to_string(),
            author: author.to_string(),
            description: description.chars().take(256).collect(),
            data,
        }
    }
}

impl Default for Preset {
    fn default() -> Self {
        Self::new("Init")
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PresetBank {
    pub name: String,
    pub presets: [Preset; 32],
}

#[allow(dead_code)]
impl PresetBank {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            presets: std::array::from_fn(|i| Preset::new(&format!("User {}", i + 1))),
        }
    }

    pub fn new_factory(name: &str) -> Self {
        Self {
            name: name.to_string(),
            presets: std::array::from_fn(|i| Preset::new_factory(&format!("Preset {}", i + 1))),
        }
    }
}
