pub mod chord_follow;
pub mod accompaniment;
pub mod scale_detect;

use serde::{Deserialize, Serialize};
use crate::midi::ExternalNoteEvent;
use crate::sequencer::NotePool;
use crate::sequencer::scales::Scale;
use chord_follow::ChordFollowState;
use accompaniment::HarmonicMemory;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
pub enum MidiInputMode {
    #[default]
    Passthrough,
    ChordFollow,
    Accompaniment,
}

impl MidiInputMode {
    pub fn all() -> [MidiInputMode; 3] {
        [
            MidiInputMode::Passthrough,
            MidiInputMode::ChordFollow,
            MidiInputMode::Accompaniment,
        ]
    }

    pub fn label(&self) -> &'static str {
        match self {
            MidiInputMode::Passthrough => "Passthrough",
            MidiInputMode::ChordFollow => "Chord Follow",
            MidiInputMode::Accompaniment => "Accompaniment",
        }
    }

    pub fn from_index(i: u8) -> Self {
        match i {
            0 => MidiInputMode::Passthrough,
            1 => MidiInputMode::ChordFollow,
            2 => MidiInputMode::Accompaniment,
            _ => MidiInputMode::Passthrough,
        }
    }

    pub fn to_index(self) -> u8 {
        match self {
            MidiInputMode::Passthrough => 0,
            MidiInputMode::ChordFollow => 1,
            MidiInputMode::Accompaniment => 2,
        }
    }
}

pub enum MidiModeResult {
    Passthrough,
    NotePoolUpdate(NotePool),
    NoChange,
}

pub struct MidiModeProcessor {
    mode: MidiInputMode,
    chord_follow: ChordFollowState,
    accompaniment: HarmonicMemory,
}

impl MidiModeProcessor {
    pub fn new() -> Self {
        Self {
            mode: MidiInputMode::Passthrough,
            chord_follow: ChordFollowState::new(),
            accompaniment: HarmonicMemory::new(),
        }
    }

    pub fn set_mode(&mut self, mode: MidiInputMode) {
        if self.mode != mode {
            self.mode = mode;
            self.chord_follow.clear();
        }
    }

    pub fn process_events(
        &mut self,
        events: &[ExternalNoteEvent],
        bar_index: u64,
        bar_position: f32,
    ) -> MidiModeResult {
        match self.mode {
            MidiInputMode::Passthrough => {
                MidiModeResult::Passthrough
            }
            MidiInputMode::ChordFollow => {
                for event in events {
                    if event.is_note_on {
                        self.chord_follow.note_on(event.note, event.velocity);
                    } else {
                        self.chord_follow.note_off(event.note);
                    }
                }
                if self.chord_follow.is_dirty() {
                    MidiModeResult::NotePoolUpdate(self.chord_follow.build_note_pool())
                } else {
                    MidiModeResult::NoChange
                }
            }
            MidiInputMode::Accompaniment => {
                for event in events {
                    if event.is_note_on {
                        self.accompaniment.process_note(
                            event.note,
                            event.velocity,
                            bar_position,
                            bar_index,
                        );
                    }
                }
                if self.accompaniment.check_bar_boundary(bar_index) {
                    MidiModeResult::NotePoolUpdate(self.accompaniment.build_note_pool())
                } else {
                    MidiModeResult::NoChange
                }
            }
        }
    }

    pub fn clear_accompaniment(&mut self) {
        self.accompaniment.clear();
    }

    pub fn get_display(&self) -> MidiModeDisplay {
        MidiModeDisplay {
            held_notes: self.chord_follow.get_held_notes(),
            detected_key: self.accompaniment.detected_key(),
            bars_analyzed: self.accompaniment.bars_analyzed(),
            confidence: self.accompaniment.key_confidence(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct MidiModeDisplay {
    pub held_notes: Vec<(u8, u8)>,
    pub detected_key: Option<(u8, Scale)>,
    pub bars_analyzed: u32,
    pub confidence: f32,
}

impl Default for MidiModeDisplay {
    fn default() -> Self {
        Self {
            held_notes: Vec::new(),
            detected_key: None,
            bars_analyzed: 0,
            confidence: 0.0,
        }
    }
}
