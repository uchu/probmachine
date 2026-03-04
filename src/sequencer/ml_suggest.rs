use crate::params::{BeatMode, DeviceParams};
use crate::ui::SharedUiState;
use crate::sequencer::scales::{Scale, StabilityPattern};
use rand::Rng;
use std::sync::Arc;

pub const SLOT_COUNT: usize = 152;

const STRONG_POSITIONS: [f32; 4] = [0.0, 0.25, 0.5, 0.75];
const STRONG_THRESHOLD: f32 = 0.03;

#[derive(Clone, Copy, PartialEq)]
pub enum StyleFilter {
    All,
    Straight,
    Offbeat,
}

#[derive(Clone)]
struct DistributionMeta {
    active_slots: u8,
    strong_ratio: f32,
}

pub const DIVISIONS: [(BeatMode, usize); 15] = [
    (BeatMode::Straight, 1),
    (BeatMode::Straight, 2),
    (BeatMode::Straight, 4),
    (BeatMode::Straight, 8),
    (BeatMode::Straight, 16),
    (BeatMode::Straight, 32),
    (BeatMode::Triplet, 3),
    (BeatMode::Triplet, 6),
    (BeatMode::Triplet, 12),
    (BeatMode::Triplet, 24),
    (BeatMode::Dotted, 2),
    (BeatMode::Dotted, 3),
    (BeatMode::Dotted, 6),
    (BeatMode::Dotted, 11),
    (BeatMode::Dotted, 22),
];

pub fn flat_index(mode: BeatMode, beat_count: usize, beat_index: usize) -> usize {
    let mut offset = 0;
    for &(m, c) in &DIVISIONS {
        if m == mode && c == beat_count {
            return offset + beat_index;
        }
        offset += c;
    }
    panic!("Invalid division: {:?} count={}", mode, beat_count);
}

pub fn reverse_flat_index(flat: usize) -> (BeatMode, usize, usize) {
    let mut offset = 0;
    for &(mode, count) in &DIVISIONS {
        if flat < offset + count {
            return (mode, count, flat - offset);
        }
        offset += count;
    }
    panic!("Invalid flat index: {}", flat);
}

struct BeatSlotSpan {
    flat_index: usize,
    start: f32,
    end: f32,
}

fn build_slot_spans() -> Vec<BeatSlotSpan> {
    let mut spans = Vec::with_capacity(SLOT_COUNT);
    let mut flat = 0;
    for &(mode, count) in &DIVISIONS {
        for index in 0..count {
            let (start, end) = DeviceParams::get_beat_time_span(mode, count, index);
            spans.push(BeatSlotSpan { flat_index: flat, start, end });
            flat += 1;
        }
    }
    spans
}

pub fn normalize_beat_constraints(result: &mut [f32; SLOT_COUNT]) {
    let spans = build_slot_spans();

    let mut time_points: Vec<f32> = Vec::with_capacity(SLOT_COUNT * 2);
    for span in &spans {
        if result[span.flat_index] > 0.0 {
            time_points.push(span.start);
            time_points.push(span.end);
        }
    }
    time_points.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    time_points.dedup_by(|a, b| (*a - *b).abs() < 0.00001);

    if time_points.len() < 2 {
        return;
    }

    let mut scale_factors = [1.0f32; SLOT_COUNT];

    for i in 0..time_points.len() - 1 {
        let mid = (time_points[i] + time_points[i + 1]) / 2.0;

        let mut active_indices = Vec::new();
        let mut total = 0.0f32;

        for span in &spans {
            if result[span.flat_index] > 0.0 && mid >= span.start && mid < span.end {
                active_indices.push(span.flat_index);
                total += result[span.flat_index];
            }
        }

        if total > 127.0 {
            let factor = 127.0 / total;
            for &idx in &active_indices {
                scale_factors[idx] = scale_factors[idx].min(factor);
            }
        }
    }

    for i in 0..SLOT_COUNT {
        if scale_factors[i] < 1.0 {
            result[i] *= scale_factors[i];
            if result[i] < 5.0 {
                result[i] = 0.0;
            }
        }
    }
}

#[derive(Clone)]
pub struct BeatSuggestion {
    pub beats: [f32; SLOT_COUNT],
    pub swing: f32,
    pub strength: [f32; 96],
}

fn build_strong_mask() -> [bool; SLOT_COUNT] {
    let mut mask = [false; SLOT_COUNT];
    let mut flat = 0;
    for &(mode, count) in &DIVISIONS {
        for index in 0..count {
            let (start, _) = DeviceParams::get_beat_time_span(mode, count, index);
            mask[flat] = STRONG_POSITIONS.iter().any(|&sp| (start - sp).abs() < STRONG_THRESHOLD);
            flat += 1;
        }
    }
    mask
}

fn compute_meta(dist: &[f32; SLOT_COUNT], strong_mask: &[bool; SLOT_COUNT]) -> DistributionMeta {
    let mut active_slots = 0u8;
    let mut strong_sum = 0.0f32;
    let mut total_sum = 0.0f32;
    for i in 0..SLOT_COUNT {
        if dist[i] > 0.0 {
            active_slots += 1;
            total_sum += dist[i];
            if strong_mask[i] {
                strong_sum += dist[i];
            }
        }
    }
    let strong_ratio = if total_sum > 0.0 { strong_sum / total_sum } else { 0.5 };
    DistributionMeta { active_slots, strong_ratio }
}

fn style_matches(style: StyleFilter, strong_ratio: f32) -> bool {
    match style {
        StyleFilter::All => true,
        StyleFilter::Straight => strong_ratio >= 0.55,
        StyleFilter::Offbeat => strong_ratio < 0.45,
    }
}

pub struct BeatSuggester {
    distributions: Vec<[f32; SLOT_COUNT]>,
    strength_grids: Vec<[f32; 96]>,
    swing_values: Vec<f32>,
    meta: Vec<DistributionMeta>,
}

impl BeatSuggester {
    pub fn new() -> Self {
        let data = include_bytes!("beat_data.bin");
        Self::from_data(data)
    }

    pub fn from_data(data: &[u8]) -> Self {
        let (distributions, strength_grids, swing_values) = Self::parse_data(data);
        let strong_mask = build_strong_mask();
        let meta: Vec<DistributionMeta> = distributions.iter()
            .map(|d| compute_meta(d, &strong_mask))
            .collect();
        Self { distributions, strength_grids, swing_values, meta }
    }

    pub fn parse_data(data: &[u8]) -> (Vec<[f32; SLOT_COUNT]>, Vec<[f32; 96]>, Vec<f32>) {
        if data.len() < 9 {
            return (Vec::new(), Vec::new(), Vec::new());
        }

        if &data[0..4] != b"BTDT" {
            return (Vec::new(), Vec::new(), Vec::new());
        }

        let version = data[4];

        let count = u32::from_le_bytes([data[5], data[6], data[7], data[8]]) as usize;
        let dist_size = count * SLOT_COUNT * 4;
        let expected_size = 9 + dist_size;
        if data.len() < expected_size {
            return (Vec::new(), Vec::new(), Vec::new());
        }

        let mut distributions = Vec::with_capacity(count);
        let mut offset = 9;

        for _ in 0..count {
            let mut dist = [0.0f32; SLOT_COUNT];
            for slot in &mut dist {
                let bytes = [data[offset], data[offset + 1], data[offset + 2], data[offset + 3]];
                *slot = f32::from_le_bytes(bytes);
                offset += 4;
            }
            distributions.push(dist);
        }

        let mut strength_grids = Vec::with_capacity(count);
        if version >= 3 {
            let strength_size = count * 96 * 4;
            if data.len() >= expected_size + strength_size {
                for _ in 0..count {
                    let mut grid = [0.0f32; 96];
                    for val in &mut grid {
                        let bytes = [data[offset], data[offset + 1], data[offset + 2], data[offset + 3]];
                        *val = f32::from_le_bytes(bytes);
                        offset += 4;
                    }
                    strength_grids.push(grid);
                }
            } else {
                strength_grids = vec![[0.5f32; 96]; count];
            }
        } else {
            strength_grids = vec![[0.5f32; 96]; count];
        }

        let mut swing_values = vec![50.0f32; count];
        if version >= 2 {
            let swing_size = count * 4;
            if data.len() >= offset + swing_size {
                for swing in &mut swing_values {
                    let bytes = [data[offset], data[offset + 1], data[offset + 2], data[offset + 3]];
                    *swing = f32::from_le_bytes(bytes);
                    offset += 4;
                }
            }
        }

        (distributions, strength_grids, swing_values)
    }

    pub fn is_available(&self) -> bool {
        !self.distributions.is_empty()
    }

    pub fn distribution_count(&self) -> usize {
        self.distributions.len()
    }

    pub fn max_active_slots(&self) -> u8 {
        self.meta.iter().map(|m| m.active_slots).max().unwrap_or(16)
    }

    pub fn min_active_slots(&self) -> u8 {
        self.meta.iter().map(|m| m.active_slots).min().unwrap_or(1)
    }

    pub fn suggest_filtered(
        &self,
        density: f32,
        min_notes: u8,
        style: StyleFilter,
        rng: &mut impl Rng,
    ) -> BeatSuggestion {
        if self.distributions.is_empty() {
            return BeatSuggestion { beats: [0.0; SLOT_COUNT], swing: 50.0, strength: [0.5; 96] };
        }
        let qualifying: Vec<usize> = self.meta.iter().enumerate()
            .filter(|(_, m)| m.active_slots >= min_notes && style_matches(style, m.strong_ratio))
            .map(|(i, _)| i)
            .collect();
        if qualifying.is_empty() {
            return self.suggest(density, rng);
        }
        let idx = qualifying[rng.gen_range(0..qualifying.len())];
        self.suggest_with_index(density, idx)
    }

    pub fn suggest(
        &self,
        density: f32,
        rng: &mut impl Rng,
    ) -> BeatSuggestion {
        if self.distributions.is_empty() {
            return BeatSuggestion { beats: [0.0; SLOT_COUNT], swing: 50.0, strength: [0.5; 96] };
        }
        let idx = rng.gen_range(0..self.distributions.len());
        self.suggest_with_index(density, idx)
    }

    pub fn suggest_with_index(
        &self,
        density: f32,
        bar_index: usize,
    ) -> BeatSuggestion {
        if self.distributions.is_empty() {
            return BeatSuggestion { beats: [0.0; SLOT_COUNT], swing: 50.0, strength: [0.5; 96] };
        }

        let idx = bar_index % self.distributions.len();
        let base = &self.distributions[idx];
        let swing = self.swing_values.get(idx).copied().unwrap_or(50.0);

        let mut result = [0.0f32; SLOT_COUNT];
        let scale = density;

        for i in 0..SLOT_COUNT {
            let val = (base[i] * scale).clamp(0.0, 127.0);
            result[i] = if val < 5.0 { 0.0 } else { val };
        }

        normalize_beat_constraints(&mut result);

        let strength = self.strength_grids.get(idx)
            .copied()
            .unwrap_or([0.5; 96]);

        BeatSuggestion { beats: result, swing, strength }
    }
}

// --- Pitch Suggestion ---

#[derive(Clone)]
pub struct PitchNoteEntry {
    pub semitone_offset: i8,
    pub chance: u8,
    pub strength_bias: u8,
    pub length_bias: u8,
}

#[derive(Clone)]
pub struct PitchSuggestion {
    pub notes: Vec<PitchNoteEntry>,
}

fn fold_pitch_spread(notes: Vec<PitchNoteEntry>, spread: f32) -> Vec<PitchNoteEntry> {
    if spread >= 1.0 {
        return notes;
    }

    let mut folded_map: std::collections::HashMap<i8, PitchNoteEntry> = std::collections::HashMap::new();

    for entry in notes {
        let interval = ((entry.semitone_offset as i16 % 12) + 12) % 12;
        let octave = (entry.semitone_offset as i16 - interval) / 12;
        let new_octave = (octave as f32 * spread).round() as i16;
        let new_offset = (interval + new_octave * 12).clamp(-48, 48) as i8;

        folded_map.entry(new_offset)
            .and_modify(|existing| {
                existing.chance = existing.chance.max(entry.chance);
                existing.strength_bias = ((existing.strength_bias as u16 + entry.strength_bias as u16) / 2) as u8;
                existing.length_bias = ((existing.length_bias as u16 + entry.length_bias as u16) / 2) as u8;
            })
            .or_insert(PitchNoteEntry {
                semitone_offset: new_offset,
                chance: entry.chance,
                strength_bias: entry.strength_bias,
                length_bias: entry.length_bias,
            });
    }

    let mut result: Vec<PitchNoteEntry> = folded_map.into_values().collect();
    result.sort_by_key(|e| e.semitone_offset);
    result
}

pub struct PitchSuggester {
    distributions: Vec<Vec<PitchNoteEntry>>,
}

const PITCH_V1_DIST_SIZE: usize = 37;

impl PitchSuggester {
    pub fn new() -> Self {
        let data = include_bytes!("pitch_data.bin");
        Self::from_data(data)
    }

    pub fn from_data(data: &[u8]) -> Self {
        let distributions = Self::parse_data(data);
        Self { distributions }
    }

    pub fn parse_data(data: &[u8]) -> Vec<Vec<PitchNoteEntry>> {
        if data.len() < 9 || &data[0..4] != b"PTDT" {
            return Vec::new();
        }

        let version = data[4];
        let count = u32::from_le_bytes([data[5], data[6], data[7], data[8]]) as usize;
        let mut offset = 9;

        if version >= 2 {
            Self::parse_v2(data, &mut offset, count)
        } else {
            Self::parse_v1(data, &mut offset, count)
        }
    }

    fn parse_v1(data: &[u8], offset: &mut usize, count: usize) -> Vec<Vec<PitchNoteEntry>> {
        let expected = *offset + count * PITCH_V1_DIST_SIZE;
        if data.len() < expected {
            return Vec::new();
        }

        let mut distributions = Vec::with_capacity(count);
        for _ in 0..count {
            let _root_pc = data[*offset];
            *offset += 1;

            let mut entries = Vec::new();
            let mut chances = [0u8; 12];
            chances.copy_from_slice(&data[*offset..*offset + 12]);
            *offset += 12;
            let mut strength = [64u8; 12];
            strength.copy_from_slice(&data[*offset..*offset + 12]);
            *offset += 12;
            let mut length = [64u8; 12];
            length.copy_from_slice(&data[*offset..*offset + 12]);
            *offset += 12;

            for i in 0..12 {
                if chances[i] > 0 || i == 0 {
                    entries.push(PitchNoteEntry {
                        semitone_offset: i as i8,
                        chance: chances[i],
                        strength_bias: strength[i],
                        length_bias: length[i],
                    });
                }
            }
            distributions.push(entries);
        }
        distributions
    }

    fn parse_v2(data: &[u8], offset: &mut usize, count: usize) -> Vec<Vec<PitchNoteEntry>> {
        let mut distributions = Vec::with_capacity(count);
        for _ in 0..count {
            if *offset + 2 > data.len() { break; }
            let _root_pc = data[*offset];
            let note_count = data[*offset + 1] as usize;
            *offset += 2;

            if *offset + note_count * 4 > data.len() { break; }
            let mut entries = Vec::with_capacity(note_count);
            for _ in 0..note_count {
                entries.push(PitchNoteEntry {
                    semitone_offset: data[*offset] as i8,
                    chance: data[*offset + 1],
                    strength_bias: data[*offset + 2],
                    length_bias: data[*offset + 3],
                });
                *offset += 4;
            }
            distributions.push(entries);
        }
        distributions
    }

    pub fn is_available(&self) -> bool {
        !self.distributions.is_empty()
    }

    pub fn distribution_count(&self) -> usize {
        self.distributions.len()
    }

    pub fn suggest_pitch(
        &self,
        density: f32,
        spread: f32,
        rng: &mut impl Rng,
    ) -> PitchSuggestion {
        if self.distributions.is_empty() {
            return PitchSuggestion { notes: vec![PitchNoteEntry {
                semitone_offset: 0, chance: 0, strength_bias: 64, length_bias: 64,
            }] };
        }
        let idx = rng.gen_range(0..self.distributions.len());
        self.suggest_pitch_with_index(density, spread, idx)
    }

    pub fn suggest_pitch_with_index(
        &self,
        density: f32,
        spread: f32,
        bar_index: usize,
    ) -> PitchSuggestion {
        if self.distributions.is_empty() {
            return PitchSuggestion { notes: Vec::new() };
        }

        let idx = bar_index % self.distributions.len();
        let base = &self.distributions[idx];

        let notes: Vec<PitchNoteEntry> = base.iter()
            .filter_map(|entry| {
                let scaled = (entry.chance as f32 * density).clamp(0.0, 127.0);
                let chance = if entry.semitone_offset == 0 {
                    (scaled as u8).max(32)
                } else {
                    if scaled < 8.0 { return None; }
                    scaled as u8
                };
                Some(PitchNoteEntry {
                    semitone_offset: entry.semitone_offset,
                    chance,
                    strength_bias: entry.strength_bias,
                    length_bias: entry.length_bias,
                })
            })
            .collect();

        PitchSuggestion { notes: fold_pitch_spread(notes, spread) }
    }
}

pub fn suggest_linked_filtered(
    beat_suggester: &BeatSuggester,
    pitch_suggester: &PitchSuggester,
    density: f32,
    spread: f32,
    min_notes: u8,
    style: StyleFilter,
    rng: &mut impl Rng,
) -> (BeatSuggestion, PitchSuggestion) {
    let max_index = beat_suggester.distribution_count()
        .min(pitch_suggester.distribution_count());
    if max_index == 0 {
        let beats = beat_suggester.suggest_with_index(density, 0);
        let pitch = pitch_suggester.suggest_pitch_with_index(density, spread, 0);
        return (beats, pitch);
    }

    let qualifying: Vec<usize> = beat_suggester.meta.iter().enumerate()
        .take(max_index)
        .filter(|(_, m)| m.active_slots >= min_notes && style_matches(style, m.strong_ratio))
        .map(|(i, _)| i)
        .collect();

    let bar_index = if qualifying.is_empty() {
        rng.gen_range(0..max_index)
    } else {
        qualifying[rng.gen_range(0..qualifying.len())]
    };

    let beats = beat_suggester.suggest_with_index(density, bar_index);
    let pitch = pitch_suggester.suggest_pitch_with_index(density, spread, bar_index);
    (beats, pitch)
}

pub fn rescale_beat_suggestion(raw: &BeatSuggestion, density: f32) -> BeatSuggestion {
    let mut result = [0.0f32; SLOT_COUNT];
    for i in 0..SLOT_COUNT {
        let val = (raw.beats[i] * density).clamp(0.0, 127.0);
        result[i] = if val < 5.0 { 0.0 } else { val };
    }
    normalize_beat_constraints(&mut result);
    BeatSuggestion { beats: result, swing: raw.swing, strength: raw.strength }
}

pub fn rescale_pitch_suggestion(raw: &PitchSuggestion, density: f32, spread: f32) -> PitchSuggestion {
    let notes: Vec<PitchNoteEntry> = raw.notes.iter()
        .filter_map(|entry| {
            let scaled = (entry.chance as f32 * density).clamp(0.0, 127.0);
            let chance = if entry.semitone_offset == 0 {
                (scaled as u8).max(32)
            } else {
                if scaled < 8.0 { return None; }
                scaled as u8
            };
            Some(PitchNoteEntry {
                semitone_offset: entry.semitone_offset,
                chance,
                strength_bias: entry.strength_bias,
                length_bias: entry.length_bias,
            })
        })
        .collect();
    PitchSuggestion { notes: fold_pitch_spread(notes, spread) }
}

pub fn apply_pitch_suggestion(
    suggestion: &PitchSuggestion,
    root_note: u8,
    ui_state: &Arc<SharedUiState>,
) {
    if let Ok(mut note_pool) = ui_state.note_pool.lock() {
        note_pool.notes.clear();
        note_pool.set_root_note(root_note);

        for entry in &suggestion.notes {
            let midi_note_i16 = root_note as i16 + entry.semitone_offset as i16;
            if midi_note_i16 < 0 || midi_note_i16 > 127 { continue; }
            let midi_note = midi_note_i16 as u8;

            let chance_norm = if entry.semitone_offset == 0 { 1.0 } else { entry.chance as f32 / 127.0 };
            let strength_bias = ((entry.strength_bias as f32 - 64.0) / 63.0).clamp(-1.0, 1.0);
            let length_bias = ((entry.length_bias as f32 - 64.0) / 63.0).clamp(-1.0, 1.0);

            note_pool.set_note_full(midi_note, 0, chance_norm, strength_bias, length_bias);
        }
    }

    if let Ok(mut scale) = ui_state.scale.lock() {
        *scale = Scale::Custom;
    }
    if let Ok(mut pattern) = ui_state.stability_pattern.lock() {
        *pattern = StabilityPattern::Custom;
    }

    ui_state.increment_preset_version();
}
