use crate::sequencer::NotePool;
use crate::sequencer::scales::Scale;
use super::scale_detect;

const MEMORY_CAPACITY: usize = 256;
const MAX_NOTES_PER_BAR: usize = 128;
const HISTOGRAM_DECAY: f32 = 0.995;
const HYSTERESIS_THRESHOLD: f32 = 0.15;

#[derive(Clone)]
struct BarHarmonics {
    bar_index: u64,
    pitch_histogram: [f32; 12],
    confidence: f32,
    listen_count: u32,
}

pub struct HarmonicMemory {
    bars: Vec<Option<BarHarmonics>>,
    global_histogram: [f32; 12],
    detected_key: Option<(u8, Scale)>,
    key_confidence: f32,
    current_bar_notes: Vec<(u8, u8, f32)>,
    current_bar_index: u64,
    last_bar_index: u64,
    bars_analyzed: u32,
    current_chord_root: Option<u8>,
}

impl HarmonicMemory {
    pub fn new() -> Self {
        Self {
            bars: vec![None; MEMORY_CAPACITY],
            global_histogram: [0.0; 12],
            detected_key: None,
            key_confidence: 0.0,
            current_bar_notes: Vec::with_capacity(MAX_NOTES_PER_BAR),
            current_bar_index: 0,
            last_bar_index: u64::MAX,
            bars_analyzed: 0,
            current_chord_root: None,
        }
    }

    pub fn process_note(&mut self, note: u8, velocity: u8, bar_position: f32, bar_index: u64) {
        if bar_index != self.current_bar_index {
            if self.last_bar_index != u64::MAX {
                self.analyze_completed_bar();
            }
            self.current_bar_index = bar_index;
            self.current_bar_notes.clear();
        }

        if self.current_bar_notes.len() < MAX_NOTES_PER_BAR {
            let pc = note % 12;
            self.current_bar_notes.push((pc, velocity, bar_position));

            if bar_position < 0.125 || (bar_position > 0.45 && bar_position < 0.55) {
                self.current_chord_root = Some(pc);
            }
        }

        self.last_bar_index = bar_index;
    }

    pub fn check_bar_boundary(&mut self, bar_index: u64) -> bool {
        if bar_index != self.current_bar_index && self.last_bar_index != u64::MAX {
            self.analyze_completed_bar();
            self.current_bar_index = bar_index;
            self.current_bar_notes.clear();
            return true;
        }
        false
    }

    fn analyze_completed_bar(&mut self) {
        if self.current_bar_notes.is_empty() {
            return;
        }

        let mut bar_histogram = [0.0f32; 12];
        for &(pc, vel, _) in &self.current_bar_notes {
            bar_histogram[pc as usize] += vel as f32 / 127.0;
        }

        let slot = self.current_bar_index as usize % MEMORY_CAPACITY;
        if let Some(existing) = &mut self.bars[slot] {
            if existing.bar_index == self.current_bar_index {
                for (i, &bh) in bar_histogram.iter().enumerate() {
                    existing.pitch_histogram[i] =
                        existing.pitch_histogram[i] * 0.7 + bh * 0.3;
                }
                existing.listen_count += 1;
                existing.confidence = (existing.confidence + 0.1).min(1.0);
            } else {
                self.bars[slot] = Some(BarHarmonics {
                    bar_index: self.current_bar_index,
                    pitch_histogram: bar_histogram,
                    confidence: 0.3,
                    listen_count: 1,
                });
            }
        } else {
            self.bars[slot] = Some(BarHarmonics {
                bar_index: self.current_bar_index,
                pitch_histogram: bar_histogram,
                confidence: 0.3,
                listen_count: 1,
            });
        }

        for (i, &bh) in bar_histogram.iter().enumerate() {
            self.global_histogram[i] *= HISTOGRAM_DECAY;
            self.global_histogram[i] += bh;
        }

        self.detect_global_key();
        self.bars_analyzed += 1;
    }

    fn detect_global_key(&mut self) {
        if let Some((root, scale, score)) = scale_detect::detect_key(&self.global_histogram) {
            if let Some((_, _)) = self.detected_key {
                if score > self.key_confidence + HYSTERESIS_THRESHOLD {
                    self.detected_key = Some((root, scale));
                    self.key_confidence = score;
                }
            } else {
                self.detected_key = Some((root, scale));
                self.key_confidence = score;
            }
        }
    }

    pub fn build_note_pool(&self) -> NotePool {
        let mut pool = NotePool::new();

        let (key_root, scale) = match self.detected_key {
            Some((r, s)) => (r, s),
            None => {
                let root_midi = 48u8;
                pool.set_root_note(root_midi);
                pool.set_note(root_midi, 1.0, 0.0);
                pool.set_note(root_midi + 7, 0.5, 0.0);
                return pool;
            }
        };

        let chord_root = self.current_chord_root.unwrap_or(key_root);
        let root_midi = 48 + chord_root;
        pool.set_root_note(root_midi);

        let confidence_factor = self.key_confidence.clamp(0.0, 1.0);

        for &interval in scale.intervals() {
            let midi_note = root_midi + interval;
            if midi_note > 127 {
                continue;
            }

            let base_chance = scale.base_chance_for_interval(interval) as f32 / 127.0;

            let chord_boost = if self.note_in_current_chord(interval, chord_root, key_root, &scale)
            {
                1.5
            } else {
                1.0
            };

            let chance = (base_chance * chord_boost * confidence_factor).min(1.0);
            let strength_bias = if interval == 0 { 0.0 } else { -0.15 };

            pool.set_note(midi_note, chance, strength_bias);
        }

        pool
    }

    fn note_in_current_chord(
        &self,
        interval: u8,
        chord_root: u8,
        key_root: u8,
        scale: &Scale,
    ) -> bool {
        let chord_interval = (chord_root + 12 - key_root) % 12;
        let quality = scale_detect::infer_chord_quality(chord_root, key_root, scale);

        let chord_intervals: &[u8] = match quality {
            scale_detect::ChordQuality::Major => &[0, 4, 7],
            scale_detect::ChordQuality::Minor => &[0, 3, 7],
            scale_detect::ChordQuality::Diminished => &[0, 3, 6],
            scale_detect::ChordQuality::Power => &[0, 7],
        };

        let note_from_chord_root = (interval + 12 - chord_interval) % 12;
        chord_intervals.contains(&note_from_chord_root)
    }

    pub fn clear(&mut self) {
        self.bars = vec![None; MEMORY_CAPACITY];
        self.global_histogram = [0.0; 12];
        self.detected_key = None;
        self.key_confidence = 0.0;
        self.current_bar_notes.clear();
        self.current_bar_index = 0;
        self.last_bar_index = u64::MAX;
        self.bars_analyzed = 0;
        self.current_chord_root = None;
    }

    pub fn detected_key(&self) -> Option<(u8, Scale)> {
        self.detected_key
    }

    pub fn bars_analyzed(&self) -> u32 {
        self.bars_analyzed
    }

    pub fn key_confidence(&self) -> f32 {
        self.key_confidence
    }
}
