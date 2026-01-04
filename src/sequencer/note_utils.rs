//! Utilities for converting between MIDI notes and frequencies

/// Convert MIDI note number to frequency in Hz
/// A4 (MIDI note 69) = 440Hz
pub fn midi_to_frequency(midi_note: u8) -> f32 {
    440.0 * 2.0_f32.powf((midi_note as f32 - 69.0) / 12.0)
}

/// Convert note name (e.g., "C4") to MIDI note number
#[allow(dead_code)]
pub fn note_name_to_midi(note_name: &str) -> Option<u8> {
    let mut chars = note_name.chars();
    let note_char = chars.next()?;

    // Check for sharp/flat
    let mut modifier = 0i8;
    let mut octave_chars = chars.as_str();

    if let Some(next) = chars.next() {
        match next {
            '#' => {
                modifier = 1;
                octave_chars = chars.as_str();
            }
            'b' => {
                modifier = -1;
                octave_chars = chars.as_str();
            }
            _ => {
                // Not a modifier, must be part of octave
                octave_chars = &note_name[1..];
            }
        }
    }

    let octave: i8 = octave_chars.parse().ok()?;

    let base_note = match note_char {
        'C' => 0,
        'D' => 2,
        'E' => 4,
        'F' => 5,
        'G' => 7,
        'A' => 9,
        'B' => 11,
        _ => return None,
    };

    let midi_note = (octave + 1) as i16 * 12 + base_note as i16 + modifier as i16;

    if !(0..=127).contains(&midi_note) {
        return None;
    }

    Some(midi_note as u8)
}

/// Get note name from MIDI note number
#[allow(dead_code)]
pub fn midi_to_note_name(midi_note: u8) -> String {
    let note_names = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];
    let octave = (midi_note / 12) as i32 - 1;
    let note_index = (midi_note % 12) as usize;
    format!("{}{}", note_names[note_index], octave)
}

/// Structure to hold note selection data
#[derive(Debug, Clone)]
pub struct NoteSelection {
    pub midi_note: u8,
    pub chance: f32,        // 0.0 to 1.0
    pub strength_bias: f32, // -1.0 (weak) to 1.0 (strong)
}

impl NoteSelection {
    pub fn new(midi_note: u8, chance: f32, strength_bias: f32) -> Self {
        Self {
            midi_note,
            chance: chance.clamp(0.0, 1.0),
            strength_bias: strength_bias.clamp(-1.0, 1.0),
        }
    }

    #[allow(dead_code)]
    pub fn frequency(&self) -> f32 {
        midi_to_frequency(self.midi_note)
    }
}

/// Collection of note selections for the sequencer
#[derive(Debug, Clone)]
pub struct NotePool {
    pub notes: Vec<NoteSelection>,
    pub root_note: Option<u8>, // MIDI note number of root
}

impl NotePool {
    pub fn new() -> Self {
        Self {
            notes: Vec::new(),
            root_note: None,
        }
    }

    /// Add or update a note in the pool
    pub fn set_note(&mut self, midi_note: u8, chance: f32, strength_bias: f32) {
        if let Some(existing) = self.notes.iter_mut().find(|n| n.midi_note == midi_note) {
            existing.chance = chance.clamp(0.0, 1.0);
            existing.strength_bias = strength_bias.clamp(-1.0, 1.0);
        } else {
            self.notes.push(NoteSelection::new(midi_note, chance, strength_bias));
        }
    }

    /// Remove a note from the pool
    #[allow(dead_code)]
    pub fn remove_note(&mut self, midi_note: u8) {
        self.notes.retain(|n| n.midi_note != midi_note);
    }

    /// Set the root note and ensure it's in the pool with 100% chance
    pub fn set_root_note(&mut self, midi_note: u8) {
        self.root_note = Some(midi_note);
        // Ensure root note is in the pool with 100% chance and no strength bias
        self.set_note(midi_note, 1.0, 0.0);
    }

    /// Select a note based on chance and strength
    /// strength_value: 0.0 (weakest) to 1.0 (strongest)
    pub fn select_note(&self, strength_value: f32, rng: &mut impl rand::Rng) -> Option<f32> {
        // Calculate effective probability for each note based on strength
        let mut weighted_notes = Vec::new();

        for note in &self.notes {
            // Calculate strength modifier
            // If note prefers strong beats (bias > 0) and beat is strong, increase probability
            // If note prefers weak beats (bias < 0) and beat is weak, increase probability
            let strength_modifier = if note.strength_bias > 0.0 {
                // Note prefers strong beats
                let alignment = strength_value; // 0.0 to 1.0
                1.0 + (note.strength_bias * alignment)
            } else if note.strength_bias < 0.0 {
                // Note prefers weak beats
                let alignment = 1.0 - strength_value; // Inverted: weak beats have high alignment
                1.0 + (-note.strength_bias * alignment)
            } else {
                // No preference
                1.0
            };

            let effective_chance = note.chance * strength_modifier;
            if effective_chance > 0.0 {
                weighted_notes.push((note.midi_note, effective_chance));
            }
        }

        if weighted_notes.is_empty() {
            return None;
        }

        // Calculate total weight
        let total_weight: f32 = weighted_notes.iter().map(|(_, w)| w).sum();

        // Random selection
        let mut random_value = rng.gen::<f32>() * total_weight;

        for (midi_note, weight) in &weighted_notes {
            random_value -= weight;
            if random_value <= 0.0 {
                return Some(midi_to_frequency(*midi_note));
            }
        }

        // Fallback to last note (shouldn't happen)
        Some(midi_to_frequency(weighted_notes.last()?.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_midi_to_frequency() {
        // A4 should be 440Hz
        assert!((midi_to_frequency(69) - 440.0).abs() < 0.01);

        // C4 should be ~261.63Hz
        assert!((midi_to_frequency(60) - 261.63).abs() < 0.1);

        // A3 should be 220Hz
        assert!((midi_to_frequency(57) - 220.0).abs() < 0.1);

        // C3 should be ~130.81Hz (our default root note)
        assert!((midi_to_frequency(48) - 130.81).abs() < 0.1);
    }

    #[test]
    fn test_note_name_conversion() {
        assert_eq!(note_name_to_midi("A4"), Some(69));
        assert_eq!(note_name_to_midi("C4"), Some(60));
        assert_eq!(note_name_to_midi("C#4"), Some(61));
        assert_eq!(note_name_to_midi("Bb3"), Some(58));

        assert_eq!(midi_to_note_name(69), "A4");
        assert_eq!(midi_to_note_name(60), "C4");
        assert_eq!(midi_to_note_name(61), "C#4");
    }

    #[test]
    fn test_octave_mapping() {
        // Test C notes across octaves
        assert_eq!(note_name_to_midi("C0"), Some(12));
        assert_eq!(note_name_to_midi("C1"), Some(24));
        assert_eq!(note_name_to_midi("C2"), Some(36));
        assert_eq!(note_name_to_midi("C3"), Some(48)); // Default root note should be this
        assert_eq!(note_name_to_midi("C4"), Some(60)); // Middle C
        assert_eq!(note_name_to_midi("C5"), Some(72));

        // Test reverse conversion
        assert_eq!(midi_to_note_name(12), "C0");
        assert_eq!(midi_to_note_name(24), "C1");
        assert_eq!(midi_to_note_name(36), "C2");
        assert_eq!(midi_to_note_name(48), "C3");
        assert_eq!(midi_to_note_name(60), "C4");
        assert_eq!(midi_to_note_name(72), "C5");
    }
}