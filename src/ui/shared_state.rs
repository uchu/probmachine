/// Shared state for UI communication with the audio engine
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicU64, Ordering};
use crate::sequencer::NotePool;
use crate::preset::PresetManager;

#[derive(Clone)]
pub struct SharedUiState {
    pub note_pool: Arc<Mutex<NotePool>>,
    pub strength_values: Arc<Mutex<Vec<f32>>>,
    pub preset_manager: Arc<Mutex<PresetManager>>,
    pub preset_version: Arc<AtomicU64>,
}

impl SharedUiState {
    pub fn new() -> Self {
        let mut manager = PresetManager::new();
        let _ = manager.load_from_file();

        Self {
            note_pool: Arc::new(Mutex::new(NotePool::new())),
            strength_values: Arc::new(Mutex::new(vec![0.0; 96])),
            preset_manager: Arc::new(Mutex::new(manager)),
            preset_version: Arc::new(AtomicU64::new(0)),
        }
    }

    pub fn increment_preset_version(&self) {
        self.preset_version.fetch_add(1, Ordering::SeqCst);
    }

    pub fn get_preset_version(&self) -> u64 {
        self.preset_version.load(Ordering::SeqCst)
    }
}

impl Default for SharedUiState {
    fn default() -> Self {
        Self::new()
    }
}