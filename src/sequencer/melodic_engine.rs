use rand::Rng;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug)]
pub struct MelodicNote {
    pub relative_pitch: i8,
    pub start_time: f32,
    pub duration: f32,
    pub velocity: u8,
}

#[derive(Clone, Debug)]
pub struct MelodicFragment {
    pub root_pitch_class: u8,
    pub notes: Vec<MelodicNote>,
}

pub struct MelodySuggester {
    fragments: Vec<MelodicFragment>,
}

impl MelodySuggester {
    pub fn new() -> Self {
        let data = include_bytes!("melody_data.bin");
        Self::from_data(data)
    }

    pub fn from_data(data: &[u8]) -> Self {
        let fragments = Self::parse_data(data);
        Self { fragments }
    }

    pub fn parse_data(data: &[u8]) -> Vec<MelodicFragment> {
        if data.len() < 9 {
            return Vec::new();
        }

        if &data[0..4] != b"MLDT" {
            return Vec::new();
        }

        let _version = data[4];
        let count = u32::from_le_bytes([data[5], data[6], data[7], data[8]]) as usize;

        let mut fragments = Vec::with_capacity(count);
        let mut pos = 9;

        for _ in 0..count {
            if pos + 2 > data.len() {
                break;
            }

            let root_pitch_class = data[pos];
            let note_count = data[pos + 1] as usize;
            pos += 2;

            let mut notes = Vec::with_capacity(note_count);
            for _ in 0..note_count {
                if pos + 6 > data.len() {
                    break;
                }

                let relative_pitch = data[pos] as i8;
                let start_time_raw = u16::from_le_bytes([data[pos + 1], data[pos + 2]]);
                let duration_raw = u16::from_le_bytes([data[pos + 3], data[pos + 4]]);
                let velocity = data[pos + 5];
                pos += 6;

                notes.push(MelodicNote {
                    relative_pitch,
                    start_time: start_time_raw as f32 / 10000.0,
                    duration: duration_raw as f32 / 10000.0,
                    velocity,
                });
            }

            fragments.push(MelodicFragment {
                root_pitch_class,
                notes,
            });
        }

        fragments
    }

    pub fn is_available(&self) -> bool {
        !self.fragments.is_empty()
    }

    pub fn fragment_count(&self) -> usize {
        self.fragments.len()
    }

    #[allow(dead_code)]
    pub fn get_fragment(&self, index: usize) -> Option<&MelodicFragment> {
        self.fragments.get(index)
    }

    #[allow(dead_code)]
    pub fn random_fragment(&self, rng: &mut impl Rng) -> Option<&MelodicFragment> {
        if self.fragments.is_empty() {
            return None;
        }
        let idx = rng.gen_range(0..self.fragments.len());
        Some(&self.fragments[idx])
    }

    pub fn generate_varied(
        &self,
        fragment_index: usize,
        config: &MelodicConfig,
        target_root: u8,
        rng: &mut impl Rng,
    ) -> Vec<MelodicNote> {
        let fragment = match self.fragments.get(fragment_index) {
            Some(f) => f,
            None => return Vec::new(),
        };

        let root_offset = (target_root % 12) as i8 - fragment.root_pitch_class as i8;

        let mut result = Vec::with_capacity(fragment.notes.len());

        for (i, note) in fragment.notes.iter().enumerate() {
            if config.note_drop_chance > 0.0
                && rng.gen::<f32>() < config.note_drop_chance
                && i > 0
                && note.start_time > 0.05
            {
                continue;
            }

            let mut pitch = note.relative_pitch as i16 + root_offset as i16;

            if config.pitch_variation > 0.0 && rng.gen::<f32>() < config.pitch_variation {
                let shift = if rng.gen::<bool>() { 1 } else { -1 };
                pitch += shift;
            }

            if config.octave_variation > 0.0 && rng.gen::<f32>() < config.octave_variation {
                let shift = if rng.gen::<bool>() { 12 } else { -12 };
                pitch += shift;
            }

            pitch = pitch.clamp(-36, 36);

            let mut start = note.start_time;
            if config.rhythm_variation > 0.0 {
                let max_shift = config.rhythm_variation * 0.05;
                let shift = rng.gen_range(-max_shift..max_shift);
                start = (start + shift).clamp(0.0, 1.0);
            }

            result.push(MelodicNote {
                relative_pitch: pitch as i8,
                start_time: start,
                duration: note.duration,
                velocity: note.velocity,
            });
        }

        result.sort_by(|a, b| a.start_time.partial_cmp(&b.start_time).unwrap());
        result
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MelodicConfig {
    pub enabled: bool,
    pub pitch_variation: f32,
    pub rhythm_variation: f32,
    pub note_drop_chance: f32,
    pub octave_variation: f32,
    pub blend: f32,
    pub fragment_index: Option<usize>,
}

impl Default for MelodicConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            pitch_variation: 0.2,
            rhythm_variation: 0.1,
            note_drop_chance: 0.1,
            octave_variation: 0.05,
            blend: 0.5,
            fragment_index: None,
        }
    }
}

impl MelodicConfig {
    pub fn pick_fragment_index(&self, suggester: &MelodySuggester, rng: &mut impl Rng) -> Option<usize> {
        if !suggester.is_available() {
            return None;
        }
        match self.fragment_index {
            Some(idx) if idx < suggester.fragment_count() => Some(idx),
            _ => Some(rng.gen_range(0..suggester.fragment_count())),
        }
    }
}
