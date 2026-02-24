mod note_utils;
pub mod scales;

use crate::params::{BeatMode, DeviceParams};
use rand::Rng;
use nih_plug::prelude::Param;
pub use note_utils::{NotePool, midi_to_frequency};
#[allow(unused_imports)]
pub use scales::{Scale, StabilityPattern, OctaveRandomization, OctaveDirection};

#[derive(Clone, Debug)]
struct NoteEvent {
    sample_position: usize,
    frequency: f64,
    duration_samples: usize,
    velocity: u8,
    midi_note: u8,
}

pub struct Sequencer {
    #[allow(dead_code)]
    sample_rate: f64,
    current_bar: Vec<NoteEvent>,
    next_bar: Vec<NoteEvent>,
    bar_position_samples: usize,
    bar_length_samples: usize,
    current_note: Option<(usize, usize)>,
    params_hash: u64,
    tempo_bpm: f64,
    pub note_pool: NotePool,
    pub strength_values: Vec<f32>,
    pub octave_randomization: OctaveRandomization,
    scratch_start_times: Vec<f32>,
    scratch_lost_beats: Vec<(f32, f32)>,
    scratch_candidates: Vec<(BeatMode, usize, usize, f32)>,
    scratch_events: Vec<NoteEvent>,
}

impl Sequencer {
    pub fn new(sample_rate: f64, tempo_bpm: f64) -> Self {
        let bar_length_samples = Self::calculate_bar_length_samples(sample_rate, tempo_bpm);

        // Initialize strength values - all positions start at 0 (neutral)
        // User will configure these through the Strength page
        let strength_values = vec![0.0; 96];

        Self {
            sample_rate,
            current_bar: Vec::with_capacity(64),
            next_bar: Vec::with_capacity(64),
            bar_position_samples: 0,
            bar_length_samples,
            current_note: None,
            params_hash: 0,
            tempo_bpm,
            note_pool: NotePool::new(),
            strength_values,
            octave_randomization: OctaveRandomization::default(),
            scratch_start_times: Vec::with_capacity(128),
            scratch_lost_beats: Vec::with_capacity(64),
            scratch_candidates: Vec::with_capacity(16),
            scratch_events: Vec::with_capacity(64),
        }
    }

    fn calculate_bar_length_samples(sample_rate: f64, tempo_bpm: f64) -> usize {
        let seconds_per_beat = 60.0 / tempo_bpm;
        let seconds_per_bar = seconds_per_beat * 4.0;
        (seconds_per_bar * sample_rate) as usize
    }

    pub fn get_bpm(&self) -> f64 {
        self.tempo_bpm
    }

    pub fn set_bpm(&mut self, bpm: f64) {
        if (bpm - self.tempo_bpm).abs() > 0.01 {
            self.tempo_bpm = bpm;
            self.bar_length_samples = Self::calculate_bar_length_samples(self.sample_rate, bpm);
        }
    }

    #[allow(dead_code)]
    pub fn set_sample_rate(&mut self, sample_rate: f64) {
        if (sample_rate - self.sample_rate).abs() > 0.1 {
            self.sample_rate = sample_rate;
            self.bar_length_samples = Self::calculate_bar_length_samples(sample_rate, self.tempo_bpm);
        }
    }

    pub fn release_current_note(&mut self) {
        self.current_note = None;
    }

    pub fn has_active_note(&self) -> bool {
        self.current_note.is_some()
    }

    pub fn reset(&mut self) {
        self.current_note = None;
        self.bar_position_samples = 0;
        self.current_bar.clear();
        self.next_bar.clear();
        self.params_hash = 0;
    }

    fn hash_params(params: &DeviceParams) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();

        for mode in [BeatMode::Straight, BeatMode::Triplet, BeatMode::Dotted] {
            for (count, _) in DeviceParams::get_divisions_for_mode(mode).iter() {
                for index in 0..*count {
                    let param = params.get_division_param(mode, *count, index);
                    let value = param.modulated_plain_value();
                    let value_bits = value.to_bits();
                    value_bits.hash(&mut hasher);
                }
            }
        }

        // Include swing in the hash so bars regenerate when swing changes
        params.swing_amount.modulated_plain_value().to_bits().hash(&mut hasher);

        hasher.finish()
    }

    /// Get the strength value for a given position in the bar (0.0 to 1.0)
    fn get_strength_at_position(&self, normalized_position: f32) -> f32 {
        // Map normalized position (0.0 to 1.0) to strength grid index (0 to 95)
        let grid_position = (normalized_position * 96.0) as usize;
        let grid_position = grid_position.min(95); // Clamp to valid range

        self.strength_values[grid_position]
    }

    /// Compute min/max strength from the strength grid
    fn get_strength_range(&self) -> (f32, f32) {
        let mut min = f32::MAX;
        let mut max = f32::MIN;
        for &v in &self.strength_values {
            if v < min { min = v; }
            if v > max { max = v; }
        }
        if (max - min).abs() < 0.001 {
            // All values same - return full range so normalization gives 0.5
            (0.0, 1.0)
        } else {
            (min, max)
        }
    }

    /// Compute min/max normalized beat length from enabled beat divisions
    fn get_enabled_length_range(&self, params: &DeviceParams) -> (f32, f32) {
        let mut min_duration = f32::MAX;
        let mut max_duration = f32::MIN;

        for mode in [BeatMode::Straight, BeatMode::Triplet, BeatMode::Dotted] {
            for (count, _) in DeviceParams::get_divisions_for_mode(mode).iter() {
                for index in 0..*count {
                    let param = params.get_division_param(mode, *count, index);
                    let probability = param.modulated_plain_value();

                    if probability > 0.0 {
                        let (start, end) = DeviceParams::get_beat_time_span(mode, *count, index);
                        let duration = end - start;
                        if duration < min_duration { min_duration = duration; }
                        if duration > max_duration { max_duration = duration; }
                    }
                }
            }
        }

        if min_duration == f32::MAX {
            // No enabled beats
            return (0.0, 1.0);
        }

        // Normalize using log scale (same as beat length normalization)
        let min_normalized = ((min_duration.log2() + 5.0) / 5.0).clamp(0.0, 1.0);
        let max_normalized = ((max_duration.log2() + 5.0) / 5.0).clamp(0.0, 1.0);

        if (max_normalized - min_normalized).abs() < 0.001 {
            (0.0, 1.0)
        } else {
            (min_normalized, max_normalized)
        }
    }

    /// Normalize a value to 0-1 range relative to min/max
    fn normalize_to_range(value: f32, min: f32, max: f32) -> f32 {
        if (max - min).abs() < 0.001 {
            0.5 // All values same, return middle
        } else {
            ((value - min) / (max - min)).clamp(0.0, 1.0)
        }
    }

    fn generate_bar_into(&mut self, params: &DeviceParams) {
        self.scratch_events.clear();
        self.scratch_start_times.clear();
        self.scratch_lost_beats.clear();
        let mut rng = rand::thread_rng();

        let total_samples = self.bar_length_samples;

        let strength_range = self.get_strength_range();
        let length_range = self.get_enabled_length_range(params);

        for mode in [BeatMode::Straight, BeatMode::Triplet, BeatMode::Dotted] {
            for (count, _) in DeviceParams::get_divisions_for_mode(mode).iter() {
                for index in 0..*count {
                    let (start, _) = DeviceParams::get_beat_time_span(mode, *count, index);
                    let start_fixed = (start * 1000000.0) as u32;
                    let start_f = start_fixed as f32 / 1000000.0;
                    if !self.scratch_start_times.iter().any(|t| (*t - start_f).abs() < 0.000001) {
                        self.scratch_start_times.push(start_f);
                    }
                }
            }
        }

        self.scratch_start_times.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let mut occupied_until: f32 = 0.0;

        for st_idx in 0..self.scratch_start_times.len() {
            let start_time = self.scratch_start_times[st_idx];
            if start_time < occupied_until - 0.0001 {
                continue;
            }

            self.scratch_candidates.clear();

            for mode in [BeatMode::Straight, BeatMode::Triplet, BeatMode::Dotted] {
                for (count, _) in DeviceParams::get_divisions_for_mode(mode).iter() {
                    for index in 0..*count {
                        let (start, _end) = DeviceParams::get_beat_time_span(mode, *count, index);

                        if (start - start_time).abs() < 0.0001 {
                            let param = params.get_division_param(mode, *count, index);
                            let probability = param.modulated_plain_value();

                            if probability > 0.0 {
                                self.scratch_candidates.push((mode, *count, index, probability));
                            }
                        }
                    }
                }
            }

            if self.scratch_candidates.is_empty() {
                continue;
            }

            let total_probability: f32 = self.scratch_candidates.iter().map(|(_, _, _, p)| p).sum();

            if total_probability > 0.0 {
                let lost_probability: f32 = self.scratch_lost_beats
                    .iter()
                    .filter(|(end_time, _)| *end_time > start_time + 0.0001)
                    .map(|(_, prob)| prob)
                    .sum();
                let remaining_space = (127.0 - lost_probability).max(0.001);

                let random_value = rng.gen_range(0.0..remaining_space);

                if random_value < total_probability {
                    let mut cumulative = 0.0;
                    let mut winner_idx: Option<usize> = None;

                    for idx in 0..self.scratch_candidates.len() {
                        let (mode, count, index, probability) = self.scratch_candidates[idx];
                        cumulative += probability;
                        if random_value < cumulative {
                            let (start, end) = DeviceParams::get_beat_time_span(mode, count, index);
                            let duration_normalized = end - start;

                            let strength = self.get_strength_at_position(start_time);

                            let note_length_percent = params.note_length_percent.modulated_plain_value();
                            let base_multiplier = note_length_percent / 100.0;

                            let length_mod_multiplier = params.calculate_length_multiplier(strength, &mut rng);
                            let final_multiplier = base_multiplier * length_mod_multiplier;

                            let capped_multiplier = final_multiplier.min(2.0);
                            let duration_samples = ((duration_normalized * total_samples as f32) * capped_multiplier) as usize;

                            let position_shift = params.calculate_position_shift(strength, duration_normalized, &mut rng);
                            let shifted_time = (start_time + position_shift).clamp(0.0, 1.0);

                            let swing_amount = params.swing_amount.modulated_plain_value();
                            let swung_start_time = DeviceParams::apply_swing(shifted_time, swing_amount);
                            let sample_position = (swung_start_time * total_samples as f32) as usize;

                            let length_value = (capped_multiplier / 2.0).clamp(0.0, 1.0);

                            let midi_note = self.note_pool.select_midi_note_with_length(strength, length_value, &mut rng)
                                .or(self.note_pool.root_note)
                                .unwrap_or(48);

                            let final_midi_note = if let Some(shift) = self.octave_randomization.should_shift(strength, length_value, &mut rng) {
                                (midi_note as i16 + shift as i16 * 12).clamp(0, 127) as u8
                            } else {
                                midi_note
                            };

                            let frequency = midi_to_frequency(final_midi_note) as f64;

                            let abs_beat_length = ((duration_normalized.log2() + 5.0) / 5.0).clamp(0.0, 1.0);

                            let relative_strength = Self::normalize_to_range(strength, strength_range.0, strength_range.1);
                            let relative_length = Self::normalize_to_range(abs_beat_length, length_range.0, length_range.1);

                            let velocity = params.calculate_velocity_relative(
                                relative_strength,
                                relative_length,
                                &mut rng
                            );

                            self.scratch_events.push(NoteEvent {
                                sample_position,
                                frequency,
                                duration_samples,
                                velocity,
                                midi_note: final_midi_note,
                            });

                            occupied_until = end;
                            winner_idx = Some(idx);
                            break;
                        }
                    }

                    for idx in 0..self.scratch_candidates.len() {
                        if Some(idx) != winner_idx {
                            let (mode, count, index, probability) = self.scratch_candidates[idx];
                            let (_, end) = DeviceParams::get_beat_time_span(mode, count, index);
                            self.scratch_lost_beats.push((end, probability));
                        }
                    }
                } else {
                    for idx in 0..self.scratch_candidates.len() {
                        let (mode, count, index, probability) = self.scratch_candidates[idx];
                        let (_, end) = DeviceParams::get_beat_time_span(mode, count, index);
                        self.scratch_lost_beats.push((end, probability));
                    }
                }
            }
        }
    }

    pub fn update(&mut self, params: &DeviceParams) -> (bool, bool, f64, u8, u8) {
        let new_hash = Self::hash_params(params);
        let params_changed = new_hash != self.params_hash;

        if params_changed {
            self.params_hash = new_hash;
            self.generate_bar_into(params);
            std::mem::swap(&mut self.next_bar, &mut self.scratch_events);
        }

        let mut should_trigger = false;
        let mut should_release = false;
        let mut frequency = 130.81;
        let mut velocity = 100_u8;
        let mut midi_note = 48_u8;

        for event in &self.current_bar {
            if event.sample_position == self.bar_position_samples {
                should_trigger = true;
                frequency = event.frequency;
                velocity = event.velocity;
                midi_note = event.midi_note;
                self.current_note = Some((
                    event.sample_position,
                    event.sample_position + event.duration_samples,
                ));
                break;
            }
        }

        if !should_trigger {
            if let Some((_start_pos, end_pos)) = self.current_note {
                if self.bar_position_samples >= end_pos {
                    should_release = true;
                    self.current_note = None;
                }
            }
        }

        self.bar_position_samples += 1;

        if self.bar_position_samples >= self.bar_length_samples {
            if self.current_note.is_some() {
                should_release = true;
            }
            self.bar_position_samples = 0;
            std::mem::swap(&mut self.current_bar, &mut self.next_bar);
            self.generate_bar_into(params);
            std::mem::swap(&mut self.next_bar, &mut self.scratch_events);
            self.current_note = None;
        }

        (should_trigger, should_release, frequency, velocity, midi_note)
    }
}
