use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, AtomicU8, AtomicU32, Ordering};
use std::sync::Mutex;

pub const SOUND_PARAMS: &[&str] = &[
    // PLL
    "synth_pll_ref_octave", "synth_pll_ref_tune", "synth_pll_mult",
    "synth_pll_mult_slew_time", "synth_pll_track_speed", "synth_pll_damping",
    "synth_pll_influence", "synth_pll_stereo_damp_offset", "synth_pll_burst_amount",
    "synth_pll_color_amount", "synth_pll_range", "synth_pll_fm_amount",
    "synth_pll_fm_ratio_float", "synth_pll_injection_amount", "synth_tube_drive",
    "synth_drift_amount", "synth_drift_rate", "synth_pll_volume",
    // SUB
    "synth_sub_volume",
    // SAW
    "synth_saw_octave", "synth_saw_tune", "synth_saw_fold", "synth_saw_tight",
    "synth_saw_shape_type", "synth_saw_shape_amount", "synth_saw_volume",
    // FILTER
    "synth_filter_cutoff", "synth_filter_resonance", "synth_filter_drive",
    "synth_filter_key_track", "synth_filter_env_amount", "synth_filter_stereo_sep",
    "synth_filter_enable",
    // VPS
    "synth_osc_octave", "synth_osc_tune", "synth_osc_d", "synth_osc_v",
    "synth_osc_stereo_v_offset", "synth_osc_stereo_d_offset", "synth_osc_fold",
    "synth_vps_shape_amount", "synth_osc_volume",
    // Toggles
    "synth_pll_enable", "synth_pll_colored", "synth_pll_mode",
    "synth_pll_precision", "synth_pll_injection_x4", "synth_pll_fm_expand",
    "synth_saw_enable", "synth_vps_enable",
];

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CcMapping {
    pub cc_number: u8,
    pub param_id: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct MidiLearnMappings {
    pub mappings: Vec<CcMapping>,
}

impl MidiLearnMappings {
    pub fn add(&mut self, cc: u8, param_id: String) {
        self.mappings.retain(|m| m.cc_number != cc && m.param_id != param_id);
        self.mappings.push(CcMapping { cc_number: cc, param_id });
    }

    pub fn remove_last(&mut self) -> bool {
        self.mappings.pop().is_some()
    }

    pub fn clear(&mut self) {
        self.mappings.clear();
    }

    pub fn find_by_param(&self, param_id: &str) -> Option<u8> {
        self.mappings.iter()
            .find(|m| m.param_id == param_id)
            .map(|m| m.cc_number)
    }

    pub fn len(&self) -> usize {
        self.mappings.len()
    }
}

pub struct MidiLearnState {
    pub learn_active: AtomicBool,
    pub awaiting_param: Mutex<Option<String>>,
    pub mappings: Mutex<MidiLearnMappings>,
    pub cc_values: [AtomicU32; 128],
    pub cc_changed: [AtomicBool; 128],
    pub selector_cc: AtomicU8,
    pub value_cc: AtomicU8,
    pub selected_param_idx: AtomicU8,
    pub learn_mode: AtomicU8, // 0=none, 1=learning selector, 2=learning value
    pub value_cc_picked_up: AtomicBool,
}

impl MidiLearnState {
    pub fn with_mappings(mappings: MidiLearnMappings, selector_cc: Option<u8>, value_cc: Option<u8>) -> Self {
        Self {
            learn_active: AtomicBool::new(false),
            awaiting_param: Mutex::new(None),
            mappings: Mutex::new(mappings),
            cc_values: std::array::from_fn(|_| AtomicU32::new(0)),
            cc_changed: std::array::from_fn(|_| AtomicBool::new(false)),
            selector_cc: AtomicU8::new(selector_cc.unwrap_or(255)),
            value_cc: AtomicU8::new(value_cc.unwrap_or(255)),
            selected_param_idx: AtomicU8::new(0),
            learn_mode: AtomicU8::new(0),
            value_cc_picked_up: AtomicBool::new(false),
        }
    }

    pub fn store_cc(&self, cc: u8, value: f32) {
        let idx = cc as usize;
        if idx < 128 {
            self.cc_values[idx].store(value.to_bits(), Ordering::Relaxed);
            self.cc_changed[idx].store(true, Ordering::Release);
        }
    }

    pub fn read_cc(&self, cc: u8) -> f32 {
        f32::from_bits(self.cc_values[cc as usize].load(Ordering::Relaxed))
    }

    pub fn take_changed(&self, cc: u8) -> bool {
        self.cc_changed[cc as usize].swap(false, Ordering::AcqRel)
    }
}
