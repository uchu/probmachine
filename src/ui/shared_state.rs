/// Shared state for UI communication with the audio engine
use std::sync::{Arc, Mutex};
use crate::sequencer::NotePool;
use crate::preset::PresetManager;

#[derive(Clone)]
pub struct SharedUiState {
    pub note_pool: Arc<Mutex<NotePool>>,
    pub strength_values: Arc<Mutex<Vec<f32>>>,
    pub preset_manager: Arc<Mutex<PresetManager>>,
}

impl SharedUiState {
    pub fn new() -> Self {
        let mut manager = PresetManager::new();
        let _ = manager.load_from_file();

        Self {
            note_pool: Arc::new(Mutex::new(NotePool::new())),
            strength_values: Arc::new(Mutex::new(vec![0.0; 96])),
            preset_manager: Arc::new(Mutex::new(manager)),
        }
    }
}

impl Default for SharedUiState {
    fn default() -> Self {
        Self::new()
    }
}