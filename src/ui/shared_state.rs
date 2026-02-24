/// Shared state for UI communication with the audio engine
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use crate::sequencer::NotePool;
use crate::sequencer::scales::{Scale, StabilityPattern, OctaveRandomization};
use crate::preset::PresetManager;

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
    pub request_dsp_reset: Arc<AtomicBool>,
}

impl SharedUiState {
    pub fn new() -> Self {
        let mut manager = PresetManager::new();
        let _ = manager.load_from_file();
        let _ = manager.load_favorites();

        Self {
            note_pool: Arc::new(Mutex::new(NotePool::new())),
            strength_values: Arc::new(Mutex::new(vec![0.0; 96])),
            preset_manager: Arc::new(Mutex::new(manager)),
            preset_version: Arc::new(AtomicU64::new(0)),
            cpu_load: Arc::new(AtomicU32::new(0)),
            output_level: Arc::new(AtomicU32::new(0)),
            scale: Arc::new(Mutex::new(Scale::default())),
            stability_pattern: Arc::new(Mutex::new(StabilityPattern::default())),
            octave_randomization: Arc::new(Mutex::new(OctaveRandomization::default())),
            request_dsp_reset: Arc::new(AtomicBool::new(false)),
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