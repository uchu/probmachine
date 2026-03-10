/// Shared state for UI communication with the audio engine
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, AtomicI32, AtomicU8, AtomicU32, AtomicU64, Ordering};
use crate::sequencer::NotePool;
use crate::sequencer::scales::{Scale, StabilityPattern, OctaveRandomization};
use crate::sequencer::styles::StyleConfig;
use crate::sequencer::multi_bar::MultiBarConfig;
use crate::sequencer::melodic_engine::MelodicConfig;
use crate::sequencer::ml_dataset::MlDataset;
use crate::sequencer::BeatLinks;
use crate::preset::PresetManager;
use crate::midi_modes::MidiModeDisplay;
use crate::midi_devices::{MidiDeviceManager, MidiInputQueue, MidiOutputQueue};
use crate::midi_learn::MidiLearnState;

#[derive(Clone)]
pub struct SharedUiState {
    pub note_pool: Arc<Mutex<NotePool>>,
    pub strength_values: Arc<Mutex<Vec<f32>>>,
    pub preset_manager: Arc<Mutex<PresetManager>>,
    pub preset_version: Arc<AtomicU64>,
    pub cpu_load: Arc<AtomicU32>,
    pub output_level: Arc<AtomicU32>,
    pub scale: Arc<Mutex<Scale>>,
    pub stability_pattern: Arc<Mutex<StabilityPattern>>,
    pub octave_randomization: Arc<Mutex<OctaveRandomization>>,
    pub style_config: Arc<Mutex<StyleConfig>>,
    pub multi_bar_config: Arc<Mutex<MultiBarConfig>>,
    pub melodic_config: Arc<Mutex<MelodicConfig>>,
    pub ml_dataset: Arc<Mutex<Arc<MlDataset>>>,
    pub ml_dataset_dirty: Arc<AtomicBool>,
    pub request_dsp_reset: Arc<AtomicBool>,
    pub seq_data_dirty: Arc<AtomicBool>,
    pub midi_mode: Arc<AtomicU8>,
    pub midi_mode_display: Arc<Mutex<MidiModeDisplay>>,
    pub midi_clear_memory: Arc<AtomicBool>,
    pub sample_rate: Arc<AtomicU32>,
    pub limiter_latency_samples: Arc<AtomicU32>,
    pub comp_latency_samples: Arc<AtomicU32>,
    pub comp_gr_db: Arc<AtomicU32>,
    pub midi_device_manager: Arc<Mutex<MidiDeviceManager>>,
    pub midi_device_input_queue: MidiInputQueue,
    pub midi_device_output_queue: MidiOutputQueue,
    pub midi_learn: Arc<MidiLearnState>,
    pub midi_clock_in: Arc<AtomicBool>,
    pub midi_clock_out: Arc<AtomicBool>,
    pub midi_transport_in: Arc<AtomicBool>,
    pub midi_transport_out: Arc<AtomicBool>,
    pub midi_transport_start: Arc<AtomicBool>,
    pub midi_transport_stop: Arc<AtomicBool>,
    pub soft_takeover: Arc<AtomicBool>,
    pub beat_links: Arc<Mutex<BeatLinks>>,
    pub restored_oversampling: Arc<AtomicI32>,
}

impl SharedUiState {
    pub fn new() -> Self {
        let mut preset_manager = PresetManager::new();
        let _ = preset_manager.load_from_file();
        let _ = preset_manager.load_favorites();

        let mut midi_mgr = MidiDeviceManager::new();
        let cfg = midi_mgr.load_config();
        midi_mgr.refresh_devices();
        midi_mgr.reconnect_saved_devices();
        midi_mgr.auto_select_if_single();

        let input_queue = midi_mgr.input_queue();
        let output_queue = midi_mgr.output_queue();
        let restored_midi_mode = cfg.midi_mode;
        let midi_learn = Arc::new(MidiLearnState::with_mappings(
            cfg.midi_learn_mappings_data(),
            cfg.selector_cc,
            cfg.value_cc,
        ));

        Self {
            note_pool: Arc::new(Mutex::new(NotePool::new())),
            strength_values: Arc::new(Mutex::new(vec![0.0; 96])),
            preset_manager: Arc::new(Mutex::new(preset_manager)),
            preset_version: Arc::new(AtomicU64::new(0)),
            cpu_load: Arc::new(AtomicU32::new(0)),
            output_level: Arc::new(AtomicU32::new(0)),
            scale: Arc::new(Mutex::new(Scale::default())),
            stability_pattern: Arc::new(Mutex::new(StabilityPattern::default())),
            octave_randomization: Arc::new(Mutex::new(OctaveRandomization::default())),
            style_config: Arc::new(Mutex::new(StyleConfig::default())),
            multi_bar_config: Arc::new(Mutex::new(MultiBarConfig::default())),
            melodic_config: Arc::new(Mutex::new(MelodicConfig::default())),
            ml_dataset: Arc::new(Mutex::new(Arc::new(MlDataset::builtin()))),
            ml_dataset_dirty: Arc::new(AtomicBool::new(true)),
            request_dsp_reset: Arc::new(AtomicBool::new(false)),
            seq_data_dirty: Arc::new(AtomicBool::new(true)),
            midi_mode: Arc::new(AtomicU8::new(restored_midi_mode)),
            midi_mode_display: Arc::new(Mutex::new(MidiModeDisplay::default())),
            midi_clear_memory: Arc::new(AtomicBool::new(false)),
            sample_rate: Arc::new(AtomicU32::new(44100)),
            limiter_latency_samples: Arc::new(AtomicU32::new(0)),
            comp_latency_samples: Arc::new(AtomicU32::new(0)),
            comp_gr_db: Arc::new(AtomicU32::new(0)),
            midi_device_manager: Arc::new(Mutex::new(midi_mgr)),
            midi_device_input_queue: input_queue,
            midi_device_output_queue: output_queue,
            midi_learn,
            midi_clock_in: Arc::new(AtomicBool::new(cfg.midi_clock_in)),
            midi_clock_out: Arc::new(AtomicBool::new(cfg.midi_clock_out)),
            midi_transport_in: Arc::new(AtomicBool::new(cfg.midi_transport_in)),
            midi_transport_out: Arc::new(AtomicBool::new(cfg.midi_transport_out)),
            midi_transport_start: Arc::new(AtomicBool::new(false)),
            midi_transport_stop: Arc::new(AtomicBool::new(false)),
            soft_takeover: Arc::new(AtomicBool::new(cfg.soft_takeover)),
            beat_links: Arc::new(Mutex::new(BeatLinks::new())),
            restored_oversampling: Arc::new(AtomicI32::new(cfg.oversampling)),
        }
    }

    pub fn increment_preset_version(&self) {
        self.preset_version.fetch_add(1, Ordering::SeqCst);
    }

    pub fn get_preset_version(&self) -> u64 {
        self.preset_version.load(Ordering::SeqCst)
    }

    pub fn set_cpu_load(&self, load_percent: f32) {
        let fixed = (load_percent * 100.0).clamp(0.0, 10000.0) as u32;
        self.cpu_load.store(fixed, Ordering::Relaxed);
    }

    pub fn get_cpu_load(&self) -> f32 {
        self.cpu_load.load(Ordering::Relaxed) as f32 / 100.0
    }

    pub fn set_output_level(&self, level: f32) {
        let fixed = (level * 1000.0).clamp(0.0, 1000.0) as u32;
        self.output_level.store(fixed, Ordering::Relaxed);
    }

    pub fn get_output_level(&self) -> f32 {
        self.output_level.load(Ordering::Relaxed) as f32 / 1000.0
    }

    pub fn mark_seq_dirty(&self) {
        self.seq_data_dirty.store(true, Ordering::Release);
    }

    pub fn take_seq_dirty(&self) -> bool {
        self.seq_data_dirty.swap(false, Ordering::AcqRel)
    }

    pub fn request_dsp_reset(&self) {
        self.request_dsp_reset.store(true, Ordering::SeqCst);
    }

    pub fn take_dsp_reset_request(&self) -> bool {
        self.request_dsp_reset.swap(false, Ordering::SeqCst)
    }
}

impl Default for SharedUiState {
    fn default() -> Self {
        Self::new()
    }
}