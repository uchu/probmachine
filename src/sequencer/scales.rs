use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize, Default)]
pub enum Scale {
    #[default]
    Major,
    Minor,
    Dorian,
    Phrygian,
    Lydian,
    Mixolydian,
    Locrian,
    HarmonicMinor,
    MelodicMinor,
    PentatonicMajor,
    PentatonicMinor,
    Blues,
    WholeTone,
    Chromatic,
    Japanese,
    Arabic,
    Hungarian,
}

impl Scale {
    pub fn all() -> &'static [Scale] {
        &[
            Scale::Major,
            Scale::Minor,
            Scale::Dorian,
            Scale::Phrygian,
            Scale::Lydian,
            Scale::Mixolydian,
            Scale::Locrian,
            Scale::HarmonicMinor,
            Scale::MelodicMinor,
            Scale::PentatonicMajor,
            Scale::PentatonicMinor,
            Scale::Blues,
            Scale::WholeTone,
            Scale::Chromatic,
            Scale::Japanese,
            Scale::Arabic,
            Scale::Hungarian,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            Scale::Major => "Major",
            Scale::Minor => "Minor",
            Scale::Dorian => "Dorian",
            Scale::Phrygian => "Phrygian",
            Scale::Lydian => "Lydian",
            Scale::Mixolydian => "Mixolydian",
            Scale::Locrian => "Locrian",
            Scale::HarmonicMinor => "Harmonic Minor",
            Scale::MelodicMinor => "Melodic Minor",
            Scale::PentatonicMajor => "Pentatonic Major",
            Scale::PentatonicMinor => "Pentatonic Minor",
            Scale::Blues => "Blues",
            Scale::WholeTone => "Whole Tone",
            Scale::Chromatic => "Chromatic",
            Scale::Japanese => "Japanese",
            Scale::Arabic => "Arabic",
            Scale::Hungarian => "Hungarian",
        }
    }

    pub fn intervals(&self) -> &'static [u8] {
        match self {
            Scale::Major => &[0, 2, 4, 5, 7, 9, 11],
            Scale::Minor => &[0, 2, 3, 5, 7, 8, 10],
            Scale::Dorian => &[0, 2, 3, 5, 7, 9, 10],
            Scale::Phrygian => &[0, 1, 3, 5, 7, 8, 10],
            Scale::Lydian => &[0, 2, 4, 6, 7, 9, 11],
            Scale::Mixolydian => &[0, 2, 4, 5, 7, 9, 10],
            Scale::Locrian => &[0, 1, 3, 5, 6, 8, 10],
            Scale::HarmonicMinor => &[0, 2, 3, 5, 7, 8, 11],
            Scale::MelodicMinor => &[0, 2, 3, 5, 7, 9, 11],
            Scale::PentatonicMajor => &[0, 2, 4, 7, 9],
            Scale::PentatonicMinor => &[0, 3, 5, 7, 10],
            Scale::Blues => &[0, 3, 5, 6, 7, 10],
            Scale::WholeTone => &[0, 2, 4, 6, 8, 10],
            Scale::Chromatic => &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11],
            Scale::Japanese => &[0, 1, 5, 7, 8],
            Scale::Arabic => &[0, 1, 4, 5, 7, 8, 10],
            Scale::Hungarian => &[0, 2, 3, 6, 7, 8, 11],
        }
    }

    pub fn degree_for_interval(&self, interval: u8) -> Option<u8> {
        self.intervals().iter().position(|&i| i == interval).map(|p| p as u8 + 1)
    }

    pub fn base_chance_for_degree(&self, degree: u8) -> u8 {
        match self {
            Scale::PentatonicMajor | Scale::PentatonicMinor => {
                match degree {
                    1 => 127,
                    2 | 3 => 100,
                    4 | 5 => 90,
                    _ => 80,
                }
            }
            Scale::Blues => {
                match degree {
                    1 => 127,
                    5 => 100,
                    2 | 3 => 85,
                    4 => 75, // Blue note
                    _ => 70,
                }
            }
            Scale::Chromatic => 70,
            _ => {
                match degree {
                    1 => 127, // Root
                    5 => 100, // Fifth
                    4 => 90,  // Fourth
                    3 => 80,  // Third
                    6 => 60,  // Sixth
                    2 => 45,  // Second
                    7 => 40,  // Seventh
                    _ => 50,
                }
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize, Default)]
pub enum StabilityPattern {
    #[default]
    Traditional,
    JazzMelodic,
    Ambient,
    BassHeavy,
    Melodic,
    Tension,
    Even,
    Pentatonic,
}

impl StabilityPattern {
    pub fn all() -> &'static [StabilityPattern] {
        &[
            StabilityPattern::Traditional,
            StabilityPattern::JazzMelodic,
            StabilityPattern::Ambient,
            StabilityPattern::BassHeavy,
            StabilityPattern::Melodic,
            StabilityPattern::Tension,
            StabilityPattern::Even,
            StabilityPattern::Pentatonic,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            StabilityPattern::Traditional => "Traditional",
            StabilityPattern::JazzMelodic => "Jazz Melodic",
            StabilityPattern::Ambient => "Ambient",
            StabilityPattern::BassHeavy => "Bass Heavy",
            StabilityPattern::Melodic => "Melodic",
            StabilityPattern::Tension => "Tension",
            StabilityPattern::Even => "Even",
            StabilityPattern::Pentatonic => "Pentatonic",
        }
    }

    pub fn get_stability_settings(&self, degree: u8) -> NoteStabilitySettings {
        match self {
            StabilityPattern::Traditional => match degree {
                1 => NoteStabilitySettings::new(64, 64, vec![-1, 0, 1]),
                5 => NoteStabilitySettings::new(110, 100, vec![-1, 0]),
                4 => NoteStabilitySettings::new(100, 90, vec![0]),
                3 => NoteStabilitySettings::new(80, 64, vec![0]),
                6 => NoteStabilitySettings::new(70, 50, vec![0, 1]),
                2 => NoteStabilitySettings::new(30, 30, vec![0]),
                7 => NoteStabilitySettings::new(20, 20, vec![0, 1]),
                _ => NoteStabilitySettings::new(64, 64, vec![0]),
            },
            StabilityPattern::JazzMelodic => match degree {
                1 => NoteStabilitySettings::new(64, 64, vec![0]),
                5 => NoteStabilitySettings::new(100, 90, vec![0]),
                3 => NoteStabilitySettings::new(95, 70, vec![0, 1]),
                7 => NoteStabilitySettings::new(90, 80, vec![0]),
                6 => NoteStabilitySettings::new(70, 60, vec![0]),
                2 => NoteStabilitySettings::new(64, 40, vec![0, 1]),
                4 => NoteStabilitySettings::new(40, 30, vec![0]),
                _ => NoteStabilitySettings::new(64, 64, vec![0]),
            },
            StabilityPattern::Ambient => match degree {
                1 => NoteStabilitySettings::new(120, 120, vec![-1, 0]),
                5 => NoteStabilitySettings::new(115, 115, vec![-1, 0]),
                4 => NoteStabilitySettings::new(100, 100, vec![0]),
                _ => NoteStabilitySettings::new(30, 30, vec![0]),
            },
            StabilityPattern::BassHeavy => match degree {
                1 => NoteStabilitySettings::new(120, 110, vec![-1, 0]),
                5 => NoteStabilitySettings::new(110, 100, vec![-1]),
                4 => NoteStabilitySettings::new(90, 70, vec![-1, 0]),
                3 => NoteStabilitySettings::new(64, 60, vec![0]),
                _ => NoteStabilitySettings::new(30, 30, vec![0]),
            },
            StabilityPattern::Melodic => match degree {
                1 => NoteStabilitySettings::new(64, 64, vec![0, 1]),
                3 => NoteStabilitySettings::new(100, 90, vec![0, 1]),
                5 => NoteStabilitySettings::new(95, 85, vec![0, 1]),
                6 => NoteStabilitySettings::new(75, 70, vec![0, 1]),
                2 => NoteStabilitySettings::new(64, 60, vec![0, 1]),
                7 => NoteStabilitySettings::new(70, 50, vec![0, 1]),
                4 => NoteStabilitySettings::new(50, 40, vec![0]),
                _ => NoteStabilitySettings::new(64, 64, vec![0]),
            },
            StabilityPattern::Tension => match degree {
                1 => NoteStabilitySettings::new(64, 64, vec![0]),
                7 => NoteStabilitySettings::new(90, 80, vec![0, 1]),
                2 => NoteStabilitySettings::new(85, 70, vec![0]),
                6 => NoteStabilitySettings::new(75, 65, vec![0]),
                3 => NoteStabilitySettings::new(70, 60, vec![0]),
                5 => NoteStabilitySettings::new(50, 50, vec![0]),
                4 => NoteStabilitySettings::new(45, 40, vec![0]),
                _ => NoteStabilitySettings::new(64, 64, vec![0]),
            },
            StabilityPattern::Even => NoteStabilitySettings::new(64, 64, vec![0]),
            StabilityPattern::Pentatonic => match degree {
                1 => NoteStabilitySettings::new(64, 64, vec![-1, 0, 1]),
                5 => NoteStabilitySettings::new(100, 90, vec![-1, 0]),
                3 => NoteStabilitySettings::new(95, 75, vec![0, 1]),
                _ => NoteStabilitySettings::new(80, 65, vec![0]),
            },
        }
    }
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct NoteStabilitySettings {
    pub strength_pref: u8,
    pub length_pref: u8,
    pub octaves: Vec<i8>,
}

impl NoteStabilitySettings {
    pub fn new(strength_pref: u8, length_pref: u8, octaves: Vec<i8>) -> Self {
        Self {
            strength_pref,
            length_pref,
            octaves,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize, Default)]
pub enum OctaveDirection {
    Down,
    #[default]
    Both,
    Up,
}

impl OctaveDirection {
    pub fn all() -> &'static [OctaveDirection] {
        &[OctaveDirection::Down, OctaveDirection::Both, OctaveDirection::Up]
    }

    pub fn name(&self) -> &'static str {
        match self {
            OctaveDirection::Down => "Down",
            OctaveDirection::Both => "Both",
            OctaveDirection::Up => "Up",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OctaveRandomization {
    pub chance: u8,
    pub strength_pref: u8,
    pub length_pref: u8,
    pub direction: OctaveDirection,
}

impl Default for OctaveRandomization {
    fn default() -> Self {
        Self {
            chance: 0,
            strength_pref: 64,
            length_pref: 64,
            direction: OctaveDirection::Both,
        }
    }
}

#[allow(dead_code)]
impl OctaveRandomization {
    pub fn should_shift<R: rand::Rng>(&self, beat_strength: f32, note_length: f32, rng: &mut R) -> Option<i8> {
        if self.chance == 0 {
            return None;
        }

        let chance_roll: f32 = rng.gen();
        if chance_roll > (self.chance as f32 / 127.0) {
            return None;
        }

        let strength_match = self.calculate_match(beat_strength, self.strength_pref);
        let strength_roll: f32 = rng.gen();
        if strength_roll > strength_match {
            return None;
        }

        let length_match = self.calculate_match(note_length, self.length_pref);
        let length_roll: f32 = rng.gen();
        if length_roll > length_match {
            return None;
        }

        let shift = match self.direction {
            OctaveDirection::Down => -1,
            OctaveDirection::Up => 1,
            OctaveDirection::Both => if rng.gen() { 1 } else { -1 },
        };

        Some(shift)
    }

    fn calculate_match(&self, value: f32, pref: u8) -> f32 {
        if pref == 64 {
            return 1.0;
        }

        let pref_norm = (pref as f32 - 64.0) / 63.0;
        let match_value = 1.0 + pref_norm * (value - 0.5) * 2.0;
        match_value.clamp(0.1, 2.0) / 2.0
    }
}
