/// Shared state for UI communication with the audio engine
use std::sync::{Arc, Mutex};
use crate::sequencer::NotePool;

#[derive(Clone)]
pub struct SharedUiState {
    pub note_pool: Arc<Mutex<NotePool>>,
    pub strength_values: Arc<Mutex<Vec<f32>>>,
}

impl SharedUiState {
    pub fn new() -> Self {
        Self {
            note_pool: Arc::new(Mutex::new(NotePool::new())),
            strength_values: Arc::new(Mutex::new(vec![0.0; 96])),
        }
    }
}

impl Default for SharedUiState {
    fn default() -> Self {
        Self::new()
    }
}