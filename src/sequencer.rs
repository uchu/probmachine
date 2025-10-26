use crate::params::{BeatMode, DeviceParams};
use rand::Rng;
use nih_plug::prelude::Param;

#[derive(Clone, Debug)]
struct NoteEvent {
    sample_position: usize,
    frequency: f32,
    duration_samples: usize,
}

pub struct Sequencer {
    sample_rate: f32,
    current_bar: Vec<NoteEvent>,
    next_bar: Vec<NoteEvent>,
    bar_position_samples: usize,
    bar_length_samples: usize,
    current_note: Option<(usize, usize)>,
    params_hash: u64,
    tempo_bpm: f32,
}

impl Sequencer {
    pub fn new(sample_rate: f32, tempo_bpm: f32) -> Self {
        let bar_length_samples = Self::calculate_bar_length_samples(sample_rate, tempo_bpm);

        Self {
            sample_rate,
            current_bar: Vec::new(),
            next_bar: Vec::new(),
            bar_position_samples: 0,
            bar_length_samples,
            current_note: None,
            params_hash: 0,
            tempo_bpm,
        }
    }

    fn calculate_bar_length_samples(sample_rate: f32, tempo_bpm: f32) -> usize {
        let seconds_per_beat = 60.0 / tempo_bpm;
        let seconds_per_bar = seconds_per_beat * 4.0;
        (seconds_per_bar * sample_rate) as usize
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

        hasher.finish()
    }

    fn generate_bar(&self, params: &DeviceParams) -> Vec<NoteEvent> {
        let mut events = Vec::new();
        let mut rng = rand::thread_rng();

        let total_samples = self.bar_length_samples;

        let highest_resolution = 32;
        let samples_per_step = total_samples / highest_resolution;

        for step in 0..highest_resolution {
            let step_start_sample = step * samples_per_step;
            let step_time_normalized = step as f32 / highest_resolution as f32;

            let mut candidates: Vec<(BeatMode, usize, usize, f32)> = Vec::new();

            for mode in [BeatMode::Straight, BeatMode::Triplet, BeatMode::Dotted] {
                for (count, _) in DeviceParams::get_divisions_for_mode(mode).iter() {
                    for index in 0..*count {
                        let (start, end) = DeviceParams::get_beat_time_span(mode, *count, index);

                        if step_time_normalized >= start && step_time_normalized < end {
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
                            let (_, end) = DeviceParams::get_beat_time_span(mode, count, index);
                            let duration_normalized = end - step_time_normalized;
                            let duration_samples = (duration_normalized * total_samples as f32) as usize;

                            events.push(NoteEvent {
                                sample_position: step_start_sample,
                                frequency: 220.0,
                                duration_samples,
                            });
                            break;
                        }
                    }
                }
            }
        }

        events
    }

    pub fn update(&mut self, params: &DeviceParams) -> (bool, bool) {
        let new_hash = Self::hash_params(params);
        let params_changed = new_hash != self.params_hash;

        if params_changed {
            self.params_hash = new_hash;
            self.next_bar = self.generate_bar(params);
        }

        let mut should_trigger = false;
        let mut should_release = false;

        if let Some((_start_pos, end_pos)) = self.current_note {
            if self.bar_position_samples >= end_pos {
                should_release = true;
                self.current_note = None;
            }
        }

        for event in &self.current_bar {
            if event.sample_position == self.bar_position_samples {
                should_trigger = true;
                self.current_note = Some((
                    event.sample_position,
                    event.sample_position + event.duration_samples,
                ));
                break;
            }
        }

        self.bar_position_samples += 1;

        if self.bar_position_samples >= self.bar_length_samples {
            self.bar_position_samples = 0;
            self.current_bar = self.next_bar.clone();
            self.next_bar = self.generate_bar(params);
            self.current_note = None;
        }

        (should_trigger, should_release)
    }
}
