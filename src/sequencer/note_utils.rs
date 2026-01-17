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
    pub octave_offset: i8,  // -1, 0, or 1 relative to root octave
    pub chance: f32,        // 0.0 to 1.0
    pub strength_bias: f32, // -1.0 (weak) to 1.0 (strong)
    pub length_bias: f32,   // -1.0 (short) to 1.0 (long)
}

#[allow(dead_code)]
impl NoteSelection {
    pub fn new(midi_note: u8, chance: f32, strength_bias: f32) -> Self {
        Self {
            midi_note,
            octave_offset: 0,
            chance: chance.clamp(0.0, 1.0),
            strength_bias: strength_bias.clamp(-1.0, 1.0),
            length_bias: 0.0,
        }
    }

    pub fn new_full(midi_note: u8, octave_offset: i8, chance: f32, strength_bias: f32, length_bias: f32) -> Self {
        Self {
            midi_note,
            octave_offset: octave_offset.clamp(-1, 1),
            chance: chance.clamp(0.0, 1.0),
            strength_bias: strength_bias.clamp(-1.0, 1.0),
            length_bias: length_bias.clamp(-1.0, 1.0),
        }
    }

    pub fn effective_midi_note(&self) -> u8 {
        let base = self.midi_note as i16;
        let shifted = base + (self.octave_offset as i16 * 12);
        shifted.clamp(0, 127) as u8
    }

    #[allow(dead_code)]
    pub fn frequency(&self) -> f32 {
        midi_to_frequency(self.effective_midi_note())
    }
}

/// Collection of note selections for the sequencer
#[derive(Debug, Clone)]
pub struct NotePool {
    pub notes: Vec<NoteSelection>,
    pub root_note: Option<u8>,
}

impl NotePool {
    pub fn new() -> Self {
        Self {
            notes: Vec::new(),
            root_note: None,
        }
    }

    pub fn set_note(&mut self, midi_note: u8, chance: f32, strength_bias: f32) {
        self.set_note_full(midi_note, 0, chance, strength_bias, 0.0);
    }

    pub fn set_note_full(&mut self, midi_note: u8, octave_offset: i8, chance: f32, strength_bias: f32, length_bias: f32) {
        if let Some(existing) = self.notes.iter_mut().find(|n| n.midi_note == midi_note && n.octave_offset == octave_offset) {
            existing.chance = chance.clamp(0.0, 1.0);
            existing.strength_bias = strength_bias.clamp(-1.0, 1.0);
            existing.length_bias = length_bias.clamp(-1.0, 1.0);
        } else {
            self.notes.push(NoteSelection::new_full(midi_note, octave_offset, chance, strength_bias, length_bias));
        }
    }

    #[allow(dead_code)]
    pub fn remove_note(&mut self, midi_note: u8) {
        self.notes.retain(|n| n.midi_note != midi_note);
    }

    #[allow(dead_code)]
    pub fn remove_note_with_octave(&mut self, midi_note: u8, octave_offset: i8) {
        self.notes.retain(|n| !(n.midi_note == midi_note && n.octave_offset == octave_offset));
    }

    pub fn set_root_note(&mut self, midi_note: u8) {
        self.root_note = Some(midi_note);
        self.set_note(midi_note, 1.0, 0.0);
    }

    #[allow(dead_code)]
    pub fn select_note(&self, strength_value: f32, rng: &mut impl rand::Rng) -> Option<f32> {
        self.select_note_with_length(strength_value, 0.5, rng)
    }

    pub fn select_note_with_length(&self, strength_value: f32, length_value: f32, rng: &mut impl rand::Rng) -> Option<f32> {
        self.select_midi_note_with_length(strength_value, length_value, rng)
            .map(midi_to_frequency)
    }

    pub fn select_midi_note_with_length(&self, strength_value: f32, length_value: f32, rng: &mut impl rand::Rng) -> Option<u8> {
        let mut weighted_notes = Vec::new();

        for note in &self.notes {
            let strength_modifier = self.calculate_bias_modifier(note.strength_bias, strength_value);
            let length_modifier = self.calculate_bias_modifier(note.length_bias, length_value);

            let effective_chance = note.chance * strength_modifier * length_modifier;
            if effective_chance > 0.0 {
                weighted_notes.push((note.effective_midi_note(), effective_chance));
            }
        }

        if weighted_notes.is_empty() {
            return None;
        }

        let total_weight: f32 = weighted_notes.iter().map(|(_, w)| w).sum();
        let mut random_value = rng.gen::<f32>() * total_weight;

        for (midi_note, weight) in &weighted_notes {
            random_value -= weight;
            if random_value <= 0.0 {
                return Some(*midi_note);
            }
        }

        Some(weighted_notes.last()?.0)
    }

    fn calculate_bias_modifier(&self, bias: f32, value: f32) -> f32 {
        if bias.abs() < 0.01 {
            return 1.0;
        }

        if bias > 0.0 {
            1.0 + (bias * value)
        } else {
            1.0 + (-bias * (1.0 - value))
        }
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