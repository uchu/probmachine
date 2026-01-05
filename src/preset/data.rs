use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NotePresetData {
    pub midi_note: u8,
    pub chance: u8,
    pub beat: u8,
    pub beat_length: u8,
}

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

    pub synth_pll_track_speed: f32,
    pub synth_pll_damping: f32,
    pub synth_pll_influence: f32,
    pub synth_pll_mult: i32,
    pub synth_pll_colored: bool,
    pub synth_pll_mode: bool,
    pub synth_pll_ref_octave: i32,
    pub synth_pll_ref_tune: i32,
    pub synth_pll_ref_fine_tune: f32,
    pub synth_pll_ref_pulse_width: f32,
    pub synth_pll_feedback: f32,
    pub synth_pll_volume: f32,
    pub synth_pll_distortion_amount: f32,
    pub synth_pll_stereo_damp_offset: f32,
    pub synth_pll_glide: f32,
    pub synth_pll_fm_amount: f32,
    pub synth_pll_fm_ratio: i32,

    pub synth_osc_octave: i32,
    pub synth_osc_d: f32,
    pub synth_osc_v: f32,
    pub synth_osc_stereo_v_offset: f32,
    pub synth_osc_volume: f32,
    pub synth_distortion_amount: f32,

    pub synth_sub_volume: f32,

    pub synth_formant_mix: f32,
    pub synth_formant_vowel: f32,
    pub synth_formant_shift: f32,

    pub synth_filter_enable: bool,
    pub synth_filter_cutoff: f32,
    pub synth_filter_resonance: f32,
    pub synth_filter_env_amount: f32,
    pub synth_filter_drive: f32,
    pub synth_filter_mode: i32,

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

    pub synth_volume: f32,

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
}

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
                arr[0] = 64;
                arr[24] = 48;
                arr[48] = 48;
                arr[72] = 48;
                arr[12] = 32;
                arr[36] = 32;
                arr[60] = 32;
                arr[84] = 32;
                arr
            },

            root_note: 48,
            notes: vec![],

            synth_pll_track_speed: 0.5,
            synth_pll_damping: 0.3,
            synth_pll_influence: 0.5,
            synth_pll_mult: 0,
            synth_pll_colored: false,
            synth_pll_mode: true,
            synth_pll_ref_octave: 0,
            synth_pll_ref_tune: 0,
            synth_pll_ref_fine_tune: 0.0,
            synth_pll_ref_pulse_width: 0.5,
            synth_pll_feedback: 0.0,
            synth_pll_volume: 0.0,
            synth_pll_distortion_amount: 0.0,
            synth_pll_stereo_damp_offset: 0.0,
            synth_pll_glide: 0.0,
            synth_pll_fm_amount: 0.0,
            synth_pll_fm_ratio: 1,

            synth_osc_octave: 0,
            synth_osc_d: 0.5,
            synth_osc_v: 0.5,
            synth_osc_stereo_v_offset: 0.0,
            synth_osc_volume: 0.0,
            synth_distortion_amount: 0.0,

            synth_sub_volume: 0.0,

            synth_formant_mix: 0.0,
            synth_formant_vowel: 0.0,
            synth_formant_shift: 0.0,

            synth_filter_enable: false,
            synth_filter_cutoff: 1000.0,
            synth_filter_resonance: 0.0,
            synth_filter_env_amount: 0.0,
            synth_filter_drive: 1.0,
            synth_filter_mode: 3,

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

            synth_volume: 0.7,

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
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Preset {
    pub name: String,
    pub data: PresetData,
}

impl Preset {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            data: PresetData::default(),
        }
    }

    pub fn with_data(name: &str, data: PresetData) -> Self {
        Self {
            name: name.to_string(),
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
    pub presets: [Preset; 16],
}

#[allow(dead_code)]
impl PresetBank {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            presets: std::array::from_fn(|i| Preset::new(&format!("Preset {}", i + 1))),
        }
    }
}
