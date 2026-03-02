use crate::sequencer::scales::Scale;

const CANDIDATE_SCALES: [Scale; 9] = [
    Scale::Major,
    Scale::Minor,
    Scale::Dorian,
    Scale::Mixolydian,
    Scale::PentatonicMajor,
    Scale::PentatonicMinor,
    Scale::Blues,
    Scale::HarmonicMinor,
    Scale::Phrygian,
];

pub fn score_scale(root: u8, scale: &Scale, histogram: &[f32; 12]) -> f32 {
    let intervals = scale.intervals();
    let total: f32 = histogram.iter().sum();
    if total < 0.001 {
        return 0.0;
    }

    let mut in_scale = 0.0f32;
    let mut out_of_scale = 0.0f32;

    for pc in 0..12u8 {
        let rotated = (pc + 12 - root) % 12;
        let weight = histogram[pc as usize] / total;
        if intervals.contains(&rotated) {
            in_scale += weight;
        } else {
            out_of_scale += weight;
        }
    }

    in_scale - out_of_scale * 0.5
}

pub fn detect_key(histogram: &[f32; 12]) -> Option<(u8, Scale, f32)> {
    let total: f32 = histogram.iter().sum();
    if total < 2.0 {
        return None;
    }

    let mut best_score = 0.0f32;
    let mut best_root = 0u8;
    let mut best_scale = Scale::Major;

    for root in 0..12u8 {
        for scale in &CANDIDATE_SCALES {
            let score = score_scale(root, scale, histogram);
            if score > best_score {
                best_score = score;
                best_root = root;
                best_scale = *scale;
            }
        }
    }

    if best_score > 0.3 {
        Some((best_root, best_scale, best_score))
    } else {
        None
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ChordQuality {
    Major,
    Minor,
    Diminished,
    Power,
}

pub fn infer_chord_quality(bass_pc: u8, key_root: u8, scale: &Scale) -> ChordQuality {
    let degree = (bass_pc + 12 - key_root) % 12;
    match scale {
        Scale::Major => match degree {
            0 | 5 | 7 => ChordQuality::Major,
            2 | 4 | 9 => ChordQuality::Minor,
            11 => ChordQuality::Diminished,
            _ => ChordQuality::Major,
        },
        Scale::Minor | Scale::Dorian | Scale::Phrygian => match degree {
            0 | 3 | 7 => ChordQuality::Minor,
            5 | 8 | 10 => ChordQuality::Major,
            2 => ChordQuality::Diminished,
            _ => ChordQuality::Minor,
        },
        _ => ChordQuality::Power,
    }
}
