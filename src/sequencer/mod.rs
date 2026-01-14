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
    decay_multiplier: f32,
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
    #[allow(dead_code)]
    tempo_bpm: f64,
    pub note_pool: NotePool,
    pub strength_values: Vec<f32>, // 96 positions for strength grid
}

impl Sequencer {
    pub fn new(sample_rate: f64, tempo_bpm: f64) -> Self {
        let bar_length_samples = Self::calculate_bar_length_samples(sample_rate, tempo_bpm);

        // Initialize strength values - all positions start at 0 (neutral)
        // User will configure these through the Strength page
        let strength_values = vec![0.0; 96];

        Self {
            sample_rate,
            current_bar: Vec::new(),
            next_bar: Vec::new(),
            bar_position_samples: 0,
            bar_length_samples,
            current_note: None,
            params_hash: 0,
            tempo_bpm,
            note_pool: NotePool::new(),
            strength_values,
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

    #[allow(dead_code)]
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

    fn generate_bar(&self, params: &DeviceParams) -> Vec<NoteEvent> {
        let mut events = Vec::new();
        let mut rng = rand::thread_rng();

        let total_samples = self.bar_length_samples;

        use std::collections::HashSet;
        let mut unique_start_times: HashSet<(u32, u32)> = HashSet::new();

        for mode in [BeatMode::Straight, BeatMode::Triplet, BeatMode::Dotted] {
            for (count, _) in DeviceParams::get_divisions_for_mode(mode).iter() {
                for index in 0..*count {
                    let (start, _) = DeviceParams::get_beat_time_span(mode, *count, index);
                    let start_fixed = (start * 1000000.0) as u32;
                    unique_start_times.insert((start_fixed, 1000000));
                }
            }
        }

        let mut start_times: Vec<f32> = unique_start_times
            .iter()
            .map(|(num, denom)| *num as f32 / *denom as f32)
            .collect();
        start_times.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        for &start_time in &start_times {
            let mut candidates: Vec<(BeatMode, usize, usize, f32)> = Vec::new();

            for mode in [BeatMode::Straight, BeatMode::Triplet, BeatMode::Dotted] {
                for (count, _) in DeviceParams::get_divisions_for_mode(mode).iter() {
                    for index in 0..*count {
                        let (start, _end) = DeviceParams::get_beat_time_span(mode, *count, index);

                        if (start - start_time).abs() < 0.0001 {
                            let param = params.get_division_param(mode, *count, index);
                            let probability = param.modulated_plain_value();

                            if probability > 0.0 {
                                candidates.push((mode, *count, index, probability));
                            }
                        }
                    }
                }
            }

            if candidates.is_empty() {
                continue;
            }

            let total_probability: f32 = candidates.iter().map(|(_, _, _, p)| p).sum();

            if total_probability > 0.0 {
                let random_value = rng.gen_range(0.0..127.0);

                if random_value < total_probability {
                    let mut cumulative = 0.0;
                    for (mode, count, index, probability) in candidates {
                        cumulative += probability;
                        if random_value < cumulative {
                            let (start, end) = DeviceParams::get_beat_time_span(mode, count, index);
                            let duration_normalized = end - start;

                            // Get the strength value at this position (for note selection and length modifiers)
                            let strength = self.get_strength_at_position(start_time);

                            // Calculate base duration
                            let note_length_percent = params.note_length_percent.modulated_plain_value();
                            let base_multiplier = note_length_percent / 100.0;

                            // Apply length modifiers based on strength
                            let length_mod_multiplier = params.calculate_length_multiplier(strength, &mut rng);
                            let final_multiplier = base_multiplier * length_mod_multiplier;

                            // Cap at 200% of beat duration to allow legato but prevent overflow
                            let capped_multiplier = final_multiplier.min(2.0);
                            let duration_samples = ((duration_normalized * total_samples as f32) * capped_multiplier) as usize;

                            // Apply position modifier (humanization)
                            let position_shift = params.calculate_position_shift(strength, duration_normalized, &mut rng);
                            let shifted_time = (start_time + position_shift).clamp(0.0, 1.0);

                            // Apply swing to the shifted time
                            let swing_amount = params.swing_amount.modulated_plain_value();
                            let swung_start_time = DeviceParams::apply_swing(shifted_time, swing_amount);
                            let sample_position = (swung_start_time * total_samples as f32) as usize;

                            // Calculate length value for note selection (0.0 = short, 0.5 = normal, 1.0 = long)
                            let length_value = (capped_multiplier / 2.0).clamp(0.0, 1.0);

                            // Select a note from the pool based on strength and length
                            let frequency = if let Some(freq) = self.note_pool.select_note_with_length(strength, length_value, &mut rng) {
                                freq as f64
                            } else if let Some(root_midi) = self.note_pool.root_note {
                                // Fallback to root note if no notes selected from pool
                                midi_to_frequency(root_midi) as f64
                            } else {
                                // Fallback to C3 (130.81Hz) only if no root note is set
                                130.81
                            };

                            // Calculate decay multiplier for volume envelope
                            let decay_multiplier = params.calculate_decay_multiplier(strength, &mut rng);

                            events.push(NoteEvent {
                                sample_position,
                                frequency,
                                duration_samples,
                                decay_multiplier,
                            });
                            break;
                        }
                    }
                }
            }
        }

        events
    }

    pub fn update(&mut self, params: &DeviceParams) -> (bool, bool, f64, f32) {
        let new_hash = Self::hash_params(params);
        let params_changed = new_hash != self.params_hash;

        if params_changed {
            self.params_hash = new_hash;
            self.next_bar = self.generate_bar(params);
        }

        let mut should_trigger = false;
        let mut should_release = false;
        let mut frequency = 130.81; // Default frequency (C3)
        let mut decay_multiplier = 1.0_f32;

        for event in &self.current_bar {
            if event.sample_position == self.bar_position_samples {
                should_trigger = true;
                frequency = event.frequency;
                decay_multiplier = event.decay_multiplier;
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
            self.bar_position_samples = 0;
            self.current_bar = self.next_bar.clone();
            self.next_bar = self.generate_bar(params);
            self.current_note = None;
        }

        (should_trigger, should_release, frequency, decay_multiplier)
    }
}
