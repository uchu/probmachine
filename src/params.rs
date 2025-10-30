use nih_plug::prelude::*;
use nih_plug_egui::EguiState;
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BeatMode {
    Straight,
    Triplet,
    Dotted,
}

impl BeatMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            BeatMode::Straight => "S",
            BeatMode::Triplet => "T",
            BeatMode::Dotted => "D",
        }
    }
}

#[derive(Params)]
pub struct DeviceParams {
    #[persist = "editor-state"]
    pub editor_state: Arc<EguiState>,

    #[id = "div1_beat1"]
    pub div1_beat1: FloatParam,

    #[id = "div2_beat1"]
    pub div2_beat1: FloatParam,
    #[id = "div2_beat2"]
    pub div2_beat2: FloatParam,

    #[id = "div4_beat1"]
    pub div4_beat1: FloatParam,
    #[id = "div4_beat2"]
    pub div4_beat2: FloatParam,
    #[id = "div4_beat3"]
    pub div4_beat3: FloatParam,
    #[id = "div4_beat4"]
    pub div4_beat4: FloatParam,

    #[id = "div8_beat1"]
    pub div8_beat1: FloatParam,
    #[id = "div8_beat2"]
    pub div8_beat2: FloatParam,
    #[id = "div8_beat3"]
    pub div8_beat3: FloatParam,
    #[id = "div8_beat4"]
    pub div8_beat4: FloatParam,
    #[id = "div8_beat5"]
    pub div8_beat5: FloatParam,
    #[id = "div8_beat6"]
    pub div8_beat6: FloatParam,
    #[id = "div8_beat7"]
    pub div8_beat7: FloatParam,
    #[id = "div8_beat8"]
    pub div8_beat8: FloatParam,

    #[id = "div16_beat1"]
    pub div16_beat1: FloatParam,
    #[id = "div16_beat2"]
    pub div16_beat2: FloatParam,
    #[id = "div16_beat3"]
    pub div16_beat3: FloatParam,
    #[id = "div16_beat4"]
    pub div16_beat4: FloatParam,
    #[id = "div16_beat5"]
    pub div16_beat5: FloatParam,
    #[id = "div16_beat6"]
    pub div16_beat6: FloatParam,
    #[id = "div16_beat7"]
    pub div16_beat7: FloatParam,
    #[id = "div16_beat8"]
    pub div16_beat8: FloatParam,
    #[id = "div16_beat9"]
    pub div16_beat9: FloatParam,
    #[id = "div16_beat10"]
    pub div16_beat10: FloatParam,
    #[id = "div16_beat11"]
    pub div16_beat11: FloatParam,
    #[id = "div16_beat12"]
    pub div16_beat12: FloatParam,
    #[id = "div16_beat13"]
    pub div16_beat13: FloatParam,
    #[id = "div16_beat14"]
    pub div16_beat14: FloatParam,
    #[id = "div16_beat15"]
    pub div16_beat15: FloatParam,
    #[id = "div16_beat16"]
    pub div16_beat16: FloatParam,

    #[id = "div32_beat1"]
    pub div32_beat1: FloatParam,
    #[id = "div32_beat2"]
    pub div32_beat2: FloatParam,
    #[id = "div32_beat3"]
    pub div32_beat3: FloatParam,
    #[id = "div32_beat4"]
    pub div32_beat4: FloatParam,
    #[id = "div32_beat5"]
    pub div32_beat5: FloatParam,
    #[id = "div32_beat6"]
    pub div32_beat6: FloatParam,
    #[id = "div32_beat7"]
    pub div32_beat7: FloatParam,
    #[id = "div32_beat8"]
    pub div32_beat8: FloatParam,
    #[id = "div32_beat9"]
    pub div32_beat9: FloatParam,
    #[id = "div32_beat10"]
    pub div32_beat10: FloatParam,
    #[id = "div32_beat11"]
    pub div32_beat11: FloatParam,
    #[id = "div32_beat12"]
    pub div32_beat12: FloatParam,
    #[id = "div32_beat13"]
    pub div32_beat13: FloatParam,
    #[id = "div32_beat14"]
    pub div32_beat14: FloatParam,
    #[id = "div32_beat15"]
    pub div32_beat15: FloatParam,
    #[id = "div32_beat16"]
    pub div32_beat16: FloatParam,
    #[id = "div32_beat17"]
    pub div32_beat17: FloatParam,
    #[id = "div32_beat18"]
    pub div32_beat18: FloatParam,
    #[id = "div32_beat19"]
    pub div32_beat19: FloatParam,
    #[id = "div32_beat20"]
    pub div32_beat20: FloatParam,
    #[id = "div32_beat21"]
    pub div32_beat21: FloatParam,
    #[id = "div32_beat22"]
    pub div32_beat22: FloatParam,
    #[id = "div32_beat23"]
    pub div32_beat23: FloatParam,
    #[id = "div32_beat24"]
    pub div32_beat24: FloatParam,
    #[id = "div32_beat25"]
    pub div32_beat25: FloatParam,
    #[id = "div32_beat26"]
    pub div32_beat26: FloatParam,
    #[id = "div32_beat27"]
    pub div32_beat27: FloatParam,
    #[id = "div32_beat28"]
    pub div32_beat28: FloatParam,
    #[id = "div32_beat29"]
    pub div32_beat29: FloatParam,
    #[id = "div32_beat30"]
    pub div32_beat30: FloatParam,
    #[id = "div32_beat31"]
    pub div32_beat31: FloatParam,
    #[id = "div32_beat32"]
    pub div32_beat32: FloatParam,

    #[id = "div3t_beat1"]
    pub div3t_beat1: FloatParam,
    #[id = "div3t_beat2"]
    pub div3t_beat2: FloatParam,
    #[id = "div3t_beat3"]
    pub div3t_beat3: FloatParam,

    #[id = "div6t_beat1"]
    pub div6t_beat1: FloatParam,
    #[id = "div6t_beat2"]
    pub div6t_beat2: FloatParam,
    #[id = "div6t_beat3"]
    pub div6t_beat3: FloatParam,
    #[id = "div6t_beat4"]
    pub div6t_beat4: FloatParam,
    #[id = "div6t_beat5"]
    pub div6t_beat5: FloatParam,
    #[id = "div6t_beat6"]
    pub div6t_beat6: FloatParam,

    #[id = "div12t_beat1"]
    pub div12t_beat1: FloatParam,
    #[id = "div12t_beat2"]
    pub div12t_beat2: FloatParam,
    #[id = "div12t_beat3"]
    pub div12t_beat3: FloatParam,
    #[id = "div12t_beat4"]
    pub div12t_beat4: FloatParam,
    #[id = "div12t_beat5"]
    pub div12t_beat5: FloatParam,
    #[id = "div12t_beat6"]
    pub div12t_beat6: FloatParam,
    #[id = "div12t_beat7"]
    pub div12t_beat7: FloatParam,
    #[id = "div12t_beat8"]
    pub div12t_beat8: FloatParam,
    #[id = "div12t_beat9"]
    pub div12t_beat9: FloatParam,
    #[id = "div12t_beat10"]
    pub div12t_beat10: FloatParam,
    #[id = "div12t_beat11"]
    pub div12t_beat11: FloatParam,
    #[id = "div12t_beat12"]
    pub div12t_beat12: FloatParam,

    #[id = "div24t_beat1"]
    pub div24t_beat1: FloatParam,
    #[id = "div24t_beat2"]
    pub div24t_beat2: FloatParam,
    #[id = "div24t_beat3"]
    pub div24t_beat3: FloatParam,
    #[id = "div24t_beat4"]
    pub div24t_beat4: FloatParam,
    #[id = "div24t_beat5"]
    pub div24t_beat5: FloatParam,
    #[id = "div24t_beat6"]
    pub div24t_beat6: FloatParam,
    #[id = "div24t_beat7"]
    pub div24t_beat7: FloatParam,
    #[id = "div24t_beat8"]
    pub div24t_beat8: FloatParam,
    #[id = "div24t_beat9"]
    pub div24t_beat9: FloatParam,
    #[id = "div24t_beat10"]
    pub div24t_beat10: FloatParam,
    #[id = "div24t_beat11"]
    pub div24t_beat11: FloatParam,
    #[id = "div24t_beat12"]
    pub div24t_beat12: FloatParam,
    #[id = "div24t_beat13"]
    pub div24t_beat13: FloatParam,
    #[id = "div24t_beat14"]
    pub div24t_beat14: FloatParam,
    #[id = "div24t_beat15"]
    pub div24t_beat15: FloatParam,
    #[id = "div24t_beat16"]
    pub div24t_beat16: FloatParam,
    #[id = "div24t_beat17"]
    pub div24t_beat17: FloatParam,
    #[id = "div24t_beat18"]
    pub div24t_beat18: FloatParam,
    #[id = "div24t_beat19"]
    pub div24t_beat19: FloatParam,
    #[id = "div24t_beat20"]
    pub div24t_beat20: FloatParam,
    #[id = "div24t_beat21"]
    pub div24t_beat21: FloatParam,
    #[id = "div24t_beat22"]
    pub div24t_beat22: FloatParam,
    #[id = "div24t_beat23"]
    pub div24t_beat23: FloatParam,
    #[id = "div24t_beat24"]
    pub div24t_beat24: FloatParam,

    #[id = "div2d_beat1"]
    pub div2d_beat1: FloatParam,
    #[id = "div2d_beat2"]
    pub div2d_beat2: FloatParam,

    #[id = "div3d_beat1"]
    pub div3d_beat1: FloatParam,
    #[id = "div3d_beat2"]
    pub div3d_beat2: FloatParam,
    #[id = "div3d_beat3"]
    pub div3d_beat3: FloatParam,

    #[id = "div6d_beat1"]
    pub div6d_beat1: FloatParam,
    #[id = "div6d_beat2"]
    pub div6d_beat2: FloatParam,
    #[id = "div6d_beat3"]
    pub div6d_beat3: FloatParam,
    #[id = "div6d_beat4"]
    pub div6d_beat4: FloatParam,
    #[id = "div6d_beat5"]
    pub div6d_beat5: FloatParam,
    #[id = "div6d_beat6"]
    pub div6d_beat6: FloatParam,

    #[id = "div11d_beat1"]
    pub div11d_beat1: FloatParam,
    #[id = "div11d_beat2"]
    pub div11d_beat2: FloatParam,
    #[id = "div11d_beat3"]
    pub div11d_beat3: FloatParam,
    #[id = "div11d_beat4"]
    pub div11d_beat4: FloatParam,
    #[id = "div11d_beat5"]
    pub div11d_beat5: FloatParam,
    #[id = "div11d_beat6"]
    pub div11d_beat6: FloatParam,
    #[id = "div11d_beat7"]
    pub div11d_beat7: FloatParam,
    #[id = "div11d_beat8"]
    pub div11d_beat8: FloatParam,
    #[id = "div11d_beat9"]
    pub div11d_beat9: FloatParam,
    #[id = "div11d_beat10"]
    pub div11d_beat10: FloatParam,
    #[id = "div11d_beat11"]
    pub div11d_beat11: FloatParam,

    #[id = "div22d_beat1"]
    pub div22d_beat1: FloatParam,
    #[id = "div22d_beat2"]
    pub div22d_beat2: FloatParam,
    #[id = "div22d_beat3"]
    pub div22d_beat3: FloatParam,
    #[id = "div22d_beat4"]
    pub div22d_beat4: FloatParam,
    #[id = "div22d_beat5"]
    pub div22d_beat5: FloatParam,
    #[id = "div22d_beat6"]
    pub div22d_beat6: FloatParam,
    #[id = "div22d_beat7"]
    pub div22d_beat7: FloatParam,
    #[id = "div22d_beat8"]
    pub div22d_beat8: FloatParam,
    #[id = "div22d_beat9"]
    pub div22d_beat9: FloatParam,
    #[id = "div22d_beat10"]
    pub div22d_beat10: FloatParam,
    #[id = "div22d_beat11"]
    pub div22d_beat11: FloatParam,
    #[id = "div22d_beat12"]
    pub div22d_beat12: FloatParam,
    #[id = "div22d_beat13"]
    pub div22d_beat13: FloatParam,
    #[id = "div22d_beat14"]
    pub div22d_beat14: FloatParam,
    #[id = "div22d_beat15"]
    pub div22d_beat15: FloatParam,
    #[id = "div22d_beat16"]
    pub div22d_beat16: FloatParam,
    #[id = "div22d_beat17"]
    pub div22d_beat17: FloatParam,
    #[id = "div22d_beat18"]
    pub div22d_beat18: FloatParam,
    #[id = "div22d_beat19"]
    pub div22d_beat19: FloatParam,
    #[id = "div22d_beat20"]
    pub div22d_beat20: FloatParam,
    #[id = "div22d_beat21"]
    pub div22d_beat21: FloatParam,
    #[id = "div22d_beat22"]
    pub div22d_beat22: FloatParam,

    #[id = "synth_osc_d"]
    pub synth_osc_d: FloatParam,
    #[id = "synth_osc_v"]
    pub synth_osc_v: FloatParam,
    #[id = "synth_osc_volume"]
    pub synth_osc_volume: FloatParam,
    #[id = "synth_osc_octave"]
    pub synth_osc_octave: IntParam,
    #[id = "synth_sub_volume"]
    pub synth_sub_volume: FloatParam,
    #[id = "synth_sub_octave"]
    pub synth_sub_octave: IntParam,
    #[id = "synth_sub_shape"]
    pub synth_sub_shape: FloatParam,

    #[id = "synth_polyblep_volume"]
    pub synth_polyblep_volume: FloatParam,
    #[id = "synth_polyblep_pulse_width"]
    pub synth_polyblep_pulse_width: FloatParam,
    #[id = "synth_polyblep_octave"]
    pub synth_polyblep_octave: IntParam,

    #[id = "synth_pll_track_speed"]
    pub synth_pll_track_speed: FloatParam,
    #[id = "synth_pll_damping"]
    pub synth_pll_damping: FloatParam,
    #[id = "synth_pll_range"]
    pub synth_pll_range: FloatParam,
    #[id = "synth_pll_mult"]
    pub synth_pll_mult: IntParam,
    #[id = "synth_pll_colored"]
    pub synth_pll_colored: BoolParam,
    #[id = "synth_pll_mode"]
    pub synth_pll_mode: BoolParam,
    #[id = "synth_pll_ref_octave"]
    pub synth_pll_ref_octave: IntParam,
    #[id = "synth_pll_ref_tune"]
    pub synth_pll_ref_tune: IntParam,
    #[id = "synth_pll_ref_fine_tune"]
    pub synth_pll_ref_fine_tune: FloatParam,
    #[id = "synth_pll_ref_pulse_width"]
    pub synth_pll_ref_pulse_width: FloatParam,
    #[id = "synth_pll_feedback"]
    pub synth_pll_feedback: FloatParam,
    #[id = "synth_pll_ki_multiplier"]
    pub synth_pll_ki_multiplier: FloatParam,
    #[id = "synth_pll_volume"]
    pub synth_pll_volume: FloatParam,
    #[id = "synth_pll_distortion_amount"]
    pub synth_pll_distortion_amount: FloatParam,
    #[id = "synth_pll_distortion_threshold"]
    pub synth_pll_distortion_threshold: FloatParam,
    #[id = "synth_pll_stereo_damp_offset"]
    pub synth_pll_stereo_damp_offset: FloatParam,

    #[id = "synth_distortion_amount"]
    pub synth_distortion_amount: FloatParam,
    #[id = "synth_distortion_threshold"]
    pub synth_distortion_threshold: FloatParam,

    #[id = "synth_filter_cutoff"]
    pub synth_filter_cutoff: FloatParam,
    #[id = "synth_filter_resonance"]
    pub synth_filter_resonance: FloatParam,
    #[id = "synth_filter_env_amount"]
    pub synth_filter_env_amount: FloatParam,
    #[id = "synth_filter_drive"]
    pub synth_filter_drive: FloatParam,
    #[id = "synth_filter_mode"]
    pub synth_filter_mode: IntParam,

    #[id = "synth_volume"]
    pub synth_volume: FloatParam,

    #[id = "synth_vol_attack"]
    pub synth_vol_attack: FloatParam,
    #[id = "synth_vol_attack_shape"]
    pub synth_vol_attack_shape: FloatParam,
    #[id = "synth_vol_decay"]
    pub synth_vol_decay: FloatParam,
    #[id = "synth_vol_decay_shape"]
    pub synth_vol_decay_shape: FloatParam,
    #[id = "synth_vol_sustain"]
    pub synth_vol_sustain: FloatParam,
    #[id = "synth_vol_release"]
    pub synth_vol_release: FloatParam,
    #[id = "synth_vol_release_shape"]
    pub synth_vol_release_shape: FloatParam,

    #[id = "synth_filt_attack"]
    pub synth_filt_attack: FloatParam,
    #[id = "synth_filt_attack_shape"]
    pub synth_filt_attack_shape: FloatParam,
    #[id = "synth_filt_decay"]
    pub synth_filt_decay: FloatParam,
    #[id = "synth_filt_decay_shape"]
    pub synth_filt_decay_shape: FloatParam,
    #[id = "synth_filt_sustain"]
    pub synth_filt_sustain: FloatParam,
    #[id = "synth_filt_release"]
    pub synth_filt_release: FloatParam,
    #[id = "synth_filt_release_shape"]
    pub synth_filt_release_shape: FloatParam,

    #[id = "note_length_percent"]
    pub note_length_percent: FloatParam,
}

impl DeviceParams {
    pub fn get_beat_time_span(mode: BeatMode, beat_count: usize, beat_index: usize) -> (f32, f32) {
        match mode {
            BeatMode::Straight => {
                let start = beat_index as f32 / beat_count as f32;
                let end = (beat_index + 1) as f32 / beat_count as f32;
                (start, end.min(1.0))
            }
            BeatMode::Triplet => {
                let start = beat_index as f32 / beat_count as f32;
                let end = (beat_index + 1) as f32 / beat_count as f32;
                (start, end.min(1.0))
            }
            BeatMode::Dotted => {
                let dotted_duration = match beat_count {
                    2 => 24.0 / 32.0,
                    3 => 12.0 / 32.0,
                    6 => 6.0 / 32.0,
                    11 => 3.0 / 32.0,
                    22 => 1.5 / 32.0,
                    _ => panic!("Invalid dotted division: {}", beat_count),
                };
                let start = beat_index as f32 * dotted_duration;
                let end = start + dotted_duration;
                (start, end.min(1.0))
            }
        }
    }

    pub fn time_spans_overlap(span1: (f32, f32), span2: (f32, f32)) -> bool {
        let (start1, end1) = span1;
        let (start2, end2) = span2;
        start1 < end2 && start2 < end1
    }

    pub fn calculate_available_range(
        &self,
        mode: BeatMode,
        beat_count: usize,
        beat_index: usize,
    ) -> f32 {
        let current_span = Self::get_beat_time_span(mode, beat_count, beat_index);

        let mut time_points = vec![current_span.0, current_span.1];

        for other_mode in [BeatMode::Straight, BeatMode::Triplet, BeatMode::Dotted] {
            for (other_count, _) in Self::get_divisions_for_mode(other_mode).iter() {
                if other_mode == mode && *other_count == beat_count {
                    continue;
                }

                for other_index in 0..*other_count {
                    let other_span = Self::get_beat_time_span(other_mode, *other_count, other_index);

                    if Self::time_spans_overlap(current_span, other_span) {
                        if other_span.0 > current_span.0 && other_span.0 < current_span.1 {
                            time_points.push(other_span.0);
                        }
                        if other_span.1 > current_span.0 && other_span.1 < current_span.1 {
                            time_points.push(other_span.1);
                        }
                    }
                }
            }
        }

        time_points.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        time_points.dedup();

        let mut max_constraint = 0.0f32;

        for time_idx in 0..time_points.len().saturating_sub(1) {
            let sample_time = (time_points[time_idx] + time_points[time_idx + 1]) / 2.0;
            let mut constraint_at_point = 0.0f32;

            for other_mode in [BeatMode::Straight, BeatMode::Triplet, BeatMode::Dotted] {
                for (other_count, _) in Self::get_divisions_for_mode(other_mode).iter() {
                    if other_mode == mode && *other_count == beat_count {
                        continue;
                    }

                    for other_index in 0..*other_count {
                        let other_span = Self::get_beat_time_span(other_mode, *other_count, other_index);

                        if sample_time >= other_span.0 && sample_time < other_span.1 {
                            let other_param = self.get_division_param(other_mode, *other_count, other_index);
                            let value = other_param.modulated_plain_value();
                            constraint_at_point += value;
                        }
                    }
                }
            }

            max_constraint = max_constraint.max(constraint_at_point);
        }

        let available = (127.0 - max_constraint).max(0.0);
        available
    }

    pub fn get_division_param(&self, mode: BeatMode, beat_count: usize, beat_index: usize) -> &FloatParam {
        match mode {
            BeatMode::Straight => match beat_count {
                1 => match beat_index {
                    0 => &self.div1_beat1,
                    _ => panic!("Invalid beat index {} for 1/1", beat_index),
                },
                2 => match beat_index {
                    0 => &self.div2_beat1,
                    1 => &self.div2_beat2,
                    _ => panic!("Invalid beat index {} for 1/2", beat_index),
                },
                4 => match beat_index {
                    0 => &self.div4_beat1,
                    1 => &self.div4_beat2,
                    2 => &self.div4_beat3,
                    3 => &self.div4_beat4,
                    _ => panic!("Invalid beat index {} for 1/4", beat_index),
                },
                8 => match beat_index {
                    0 => &self.div8_beat1,
                    1 => &self.div8_beat2,
                    2 => &self.div8_beat3,
                    3 => &self.div8_beat4,
                    4 => &self.div8_beat5,
                    5 => &self.div8_beat6,
                    6 => &self.div8_beat7,
                    7 => &self.div8_beat8,
                    _ => panic!("Invalid beat index {} for 1/8", beat_index),
                },
                16 => match beat_index {
                    0 => &self.div16_beat1,
                    1 => &self.div16_beat2,
                    2 => &self.div16_beat3,
                    3 => &self.div16_beat4,
                    4 => &self.div16_beat5,
                    5 => &self.div16_beat6,
                    6 => &self.div16_beat7,
                    7 => &self.div16_beat8,
                    8 => &self.div16_beat9,
                    9 => &self.div16_beat10,
                    10 => &self.div16_beat11,
                    11 => &self.div16_beat12,
                    12 => &self.div16_beat13,
                    13 => &self.div16_beat14,
                    14 => &self.div16_beat15,
                    15 => &self.div16_beat16,
                    _ => panic!("Invalid beat index {} for 1/16", beat_index),
                },
                32 => match beat_index {
                    0 => &self.div32_beat1,
                    1 => &self.div32_beat2,
                    2 => &self.div32_beat3,
                    3 => &self.div32_beat4,
                    4 => &self.div32_beat5,
                    5 => &self.div32_beat6,
                    6 => &self.div32_beat7,
                    7 => &self.div32_beat8,
                    8 => &self.div32_beat9,
                    9 => &self.div32_beat10,
                    10 => &self.div32_beat11,
                    11 => &self.div32_beat12,
                    12 => &self.div32_beat13,
                    13 => &self.div32_beat14,
                    14 => &self.div32_beat15,
                    15 => &self.div32_beat16,
                    16 => &self.div32_beat17,
                    17 => &self.div32_beat18,
                    18 => &self.div32_beat19,
                    19 => &self.div32_beat20,
                    20 => &self.div32_beat21,
                    21 => &self.div32_beat22,
                    22 => &self.div32_beat23,
                    23 => &self.div32_beat24,
                    24 => &self.div32_beat25,
                    25 => &self.div32_beat26,
                    26 => &self.div32_beat27,
                    27 => &self.div32_beat28,
                    28 => &self.div32_beat29,
                    29 => &self.div32_beat30,
                    30 => &self.div32_beat31,
                    31 => &self.div32_beat32,
                    _ => panic!("Invalid beat index {} for 1/32", beat_index),
                },
                _ => panic!("Invalid beat count {} for Straight mode", beat_count),
            },
            BeatMode::Triplet => match beat_count {
                3 => match beat_index {
                    0 => &self.div3t_beat1,
                    1 => &self.div3t_beat2,
                    2 => &self.div3t_beat3,
                    _ => panic!("Invalid beat index {} for 1/2T", beat_index),
                },
                6 => match beat_index {
                    0 => &self.div6t_beat1,
                    1 => &self.div6t_beat2,
                    2 => &self.div6t_beat3,
                    3 => &self.div6t_beat4,
                    4 => &self.div6t_beat5,
                    5 => &self.div6t_beat6,
                    _ => panic!("Invalid beat index {} for 1/4T", beat_index),
                },
                12 => match beat_index {
                    0 => &self.div12t_beat1,
                    1 => &self.div12t_beat2,
                    2 => &self.div12t_beat3,
                    3 => &self.div12t_beat4,
                    4 => &self.div12t_beat5,
                    5 => &self.div12t_beat6,
                    6 => &self.div12t_beat7,
                    7 => &self.div12t_beat8,
                    8 => &self.div12t_beat9,
                    9 => &self.div12t_beat10,
                    10 => &self.div12t_beat11,
                    11 => &self.div12t_beat12,
                    _ => panic!("Invalid beat index {} for 1/8T", beat_index),
                },
                24 => match beat_index {
                    0 => &self.div24t_beat1,
                    1 => &self.div24t_beat2,
                    2 => &self.div24t_beat3,
                    3 => &self.div24t_beat4,
                    4 => &self.div24t_beat5,
                    5 => &self.div24t_beat6,
                    6 => &self.div24t_beat7,
                    7 => &self.div24t_beat8,
                    8 => &self.div24t_beat9,
                    9 => &self.div24t_beat10,
                    10 => &self.div24t_beat11,
                    11 => &self.div24t_beat12,
                    12 => &self.div24t_beat13,
                    13 => &self.div24t_beat14,
                    14 => &self.div24t_beat15,
                    15 => &self.div24t_beat16,
                    16 => &self.div24t_beat17,
                    17 => &self.div24t_beat18,
                    18 => &self.div24t_beat19,
                    19 => &self.div24t_beat20,
                    20 => &self.div24t_beat21,
                    21 => &self.div24t_beat22,
                    22 => &self.div24t_beat23,
                    23 => &self.div24t_beat24,
                    _ => panic!("Invalid beat index {} for 1/16T", beat_index),
                },
                _ => panic!("Invalid beat count {} for Triplet mode", beat_count),
            },
            BeatMode::Dotted => match beat_count {
                2 => match beat_index {
                    0 => &self.div2d_beat1,
                    1 => &self.div2d_beat2,
                    _ => panic!("Invalid beat index {} for 1/2D", beat_index),
                },
                3 => match beat_index {
                    0 => &self.div3d_beat1,
                    1 => &self.div3d_beat2,
                    2 => &self.div3d_beat3,
                    _ => panic!("Invalid beat index {} for 1/4D", beat_index),
                },
                6 => match beat_index {
                    0 => &self.div6d_beat1,
                    1 => &self.div6d_beat2,
                    2 => &self.div6d_beat3,
                    3 => &self.div6d_beat4,
                    4 => &self.div6d_beat5,
                    5 => &self.div6d_beat6,
                    _ => panic!("Invalid beat index {} for 1/8D", beat_index),
                },
                11 => match beat_index {
                    0 => &self.div11d_beat1,
                    1 => &self.div11d_beat2,
                    2 => &self.div11d_beat3,
                    3 => &self.div11d_beat4,
                    4 => &self.div11d_beat5,
                    5 => &self.div11d_beat6,
                    6 => &self.div11d_beat7,
                    7 => &self.div11d_beat8,
                    8 => &self.div11d_beat9,
                    9 => &self.div11d_beat10,
                    10 => &self.div11d_beat11,
                    _ => panic!("Invalid beat index {} for 1/16D", beat_index),
                },
                22 => match beat_index {
                    0 => &self.div22d_beat1,
                    1 => &self.div22d_beat2,
                    2 => &self.div22d_beat3,
                    3 => &self.div22d_beat4,
                    4 => &self.div22d_beat5,
                    5 => &self.div22d_beat6,
                    6 => &self.div22d_beat7,
                    7 => &self.div22d_beat8,
                    8 => &self.div22d_beat9,
                    9 => &self.div22d_beat10,
                    10 => &self.div22d_beat11,
                    11 => &self.div22d_beat12,
                    12 => &self.div22d_beat13,
                    13 => &self.div22d_beat14,
                    14 => &self.div22d_beat15,
                    15 => &self.div22d_beat16,
                    16 => &self.div22d_beat17,
                    17 => &self.div22d_beat18,
                    18 => &self.div22d_beat19,
                    19 => &self.div22d_beat20,
                    20 => &self.div22d_beat21,
                    21 => &self.div22d_beat22,
                    _ => panic!("Invalid beat index {} for 1/32D", beat_index),
                },
                _ => panic!("Invalid beat count {} for Dotted mode", beat_count),
            },
        }
    }

    pub fn get_divisions_for_mode(mode: BeatMode) -> &'static [(usize, &'static str)] {
        match mode {
            BeatMode::Straight => &[
                (1, "1/1"),
                (2, "1/2"),
                (4, "1/4"),
                (8, "1/8"),
                (16, "1/16"),
                (32, "1/32"),
            ],
            BeatMode::Triplet => &[
                (3, "1/2"),
                (6, "1/4"),
                (12, "1/8"),
                (24, "1/16"),
            ],
            BeatMode::Dotted => &[
                (2, "1/2"),
                (3, "1/4"),
                (6, "1/8"),
                (11, "1/16"),
                (22, "1/32"),
            ],
        }
    }

    pub fn is_valid_beat_count(mode: BeatMode, beat_count: usize) -> bool {
        Self::get_divisions_for_mode(mode)
            .iter()
            .any(|(count, _)| *count == beat_count)
    }

    pub fn get_default_beat_count(mode: BeatMode) -> usize {
        match mode {
            BeatMode::Straight => 8,
            BeatMode::Triplet => 12,
            BeatMode::Dotted => 6,
        }
    }

    pub fn log_all_values(&self) {
        let mut values = Vec::new();

        for mode in [BeatMode::Straight, BeatMode::Triplet, BeatMode::Dotted] {
            for (count, label) in Self::get_divisions_for_mode(mode).iter() {
                for index in 0..*count {
                    let param = self.get_division_param(mode, *count, index);
                    let value = param.modulated_plain_value();
                    if value > 0.0 {
                        values.push(format!("{}[{}]={:.0}", label, index, value));
                    }
                }
            }
        }

        if !values.is_empty() {
            nih_log!("All set values: {}", values.join(", "));
        } else {
            nih_log!("All set values: (none)");
        }
    }

    fn create_param(name: String, default: f32) -> FloatParam {
        FloatParam::new(name, default, FloatRange::Linear { min: 0.0, max: 127.0 })
            .with_smoother(SmoothingStyle::Linear(50.0))
    }
}

impl Default for DeviceParams {
    fn default() -> Self {
        Self {
            editor_state: EguiState::from_size(800, 480),

            div1_beat1: Self::create_param("1/1 Beat 1".to_string(), 0.0),

            div2_beat1: Self::create_param("1/2 Beat 1".to_string(), 0.0),
            div2_beat2: Self::create_param("1/2 Beat 2".to_string(), 0.0),

            div4_beat1: Self::create_param("1/4 Beat 1".to_string(), 0.0),
            div4_beat2: Self::create_param("1/4 Beat 2".to_string(), 0.0),
            div4_beat3: Self::create_param("1/4 Beat 3".to_string(), 0.0),
            div4_beat4: Self::create_param("1/4 Beat 4".to_string(), 0.0),

            div8_beat1: Self::create_param("1/8 Beat 1".to_string(), 0.0),
            div8_beat2: Self::create_param("1/8 Beat 2".to_string(), 0.0),
            div8_beat3: Self::create_param("1/8 Beat 3".to_string(), 0.0),
            div8_beat4: Self::create_param("1/8 Beat 4".to_string(), 0.0),
            div8_beat5: Self::create_param("1/8 Beat 5".to_string(), 0.0),
            div8_beat6: Self::create_param("1/8 Beat 6".to_string(), 0.0),
            div8_beat7: Self::create_param("1/8 Beat 7".to_string(), 0.0),
            div8_beat8: Self::create_param("1/8 Beat 8".to_string(), 0.0),

            div16_beat1: Self::create_param("1/16 Beat 1".to_string(), 0.0),
            div16_beat2: Self::create_param("1/16 Beat 2".to_string(), 0.0),
            div16_beat3: Self::create_param("1/16 Beat 3".to_string(), 0.0),
            div16_beat4: Self::create_param("1/16 Beat 4".to_string(), 0.0),
            div16_beat5: Self::create_param("1/16 Beat 5".to_string(), 0.0),
            div16_beat6: Self::create_param("1/16 Beat 6".to_string(), 0.0),
            div16_beat7: Self::create_param("1/16 Beat 7".to_string(), 0.0),
            div16_beat8: Self::create_param("1/16 Beat 8".to_string(), 0.0),
            div16_beat9: Self::create_param("1/16 Beat 9".to_string(), 0.0),
            div16_beat10: Self::create_param("1/16 Beat 10".to_string(), 0.0),
            div16_beat11: Self::create_param("1/16 Beat 11".to_string(), 0.0),
            div16_beat12: Self::create_param("1/16 Beat 12".to_string(), 0.0),
            div16_beat13: Self::create_param("1/16 Beat 13".to_string(), 0.0),
            div16_beat14: Self::create_param("1/16 Beat 14".to_string(), 0.0),
            div16_beat15: Self::create_param("1/16 Beat 15".to_string(), 0.0),
            div16_beat16: Self::create_param("1/16 Beat 16".to_string(), 0.0),

            div32_beat1: Self::create_param("1/32 Beat 1".to_string(), 0.0),
            div32_beat2: Self::create_param("1/32 Beat 2".to_string(), 0.0),
            div32_beat3: Self::create_param("1/32 Beat 3".to_string(), 0.0),
            div32_beat4: Self::create_param("1/32 Beat 4".to_string(), 0.0),
            div32_beat5: Self::create_param("1/32 Beat 5".to_string(), 0.0),
            div32_beat6: Self::create_param("1/32 Beat 6".to_string(), 0.0),
            div32_beat7: Self::create_param("1/32 Beat 7".to_string(), 0.0),
            div32_beat8: Self::create_param("1/32 Beat 8".to_string(), 0.0),
            div32_beat9: Self::create_param("1/32 Beat 9".to_string(), 0.0),
            div32_beat10: Self::create_param("1/32 Beat 10".to_string(), 0.0),
            div32_beat11: Self::create_param("1/32 Beat 11".to_string(), 0.0),
            div32_beat12: Self::create_param("1/32 Beat 12".to_string(), 0.0),
            div32_beat13: Self::create_param("1/32 Beat 13".to_string(), 0.0),
            div32_beat14: Self::create_param("1/32 Beat 14".to_string(), 0.0),
            div32_beat15: Self::create_param("1/32 Beat 15".to_string(), 0.0),
            div32_beat16: Self::create_param("1/32 Beat 16".to_string(), 0.0),
            div32_beat17: Self::create_param("1/32 Beat 17".to_string(), 0.0),
            div32_beat18: Self::create_param("1/32 Beat 18".to_string(), 0.0),
            div32_beat19: Self::create_param("1/32 Beat 19".to_string(), 0.0),
            div32_beat20: Self::create_param("1/32 Beat 20".to_string(), 0.0),
            div32_beat21: Self::create_param("1/32 Beat 21".to_string(), 0.0),
            div32_beat22: Self::create_param("1/32 Beat 22".to_string(), 0.0),
            div32_beat23: Self::create_param("1/32 Beat 23".to_string(), 0.0),
            div32_beat24: Self::create_param("1/32 Beat 24".to_string(), 0.0),
            div32_beat25: Self::create_param("1/32 Beat 25".to_string(), 0.0),
            div32_beat26: Self::create_param("1/32 Beat 26".to_string(), 0.0),
            div32_beat27: Self::create_param("1/32 Beat 27".to_string(), 0.0),
            div32_beat28: Self::create_param("1/32 Beat 28".to_string(), 0.0),
            div32_beat29: Self::create_param("1/32 Beat 29".to_string(), 0.0),
            div32_beat30: Self::create_param("1/32 Beat 30".to_string(), 0.0),
            div32_beat31: Self::create_param("1/32 Beat 31".to_string(), 0.0),
            div32_beat32: Self::create_param("1/32 Beat 32".to_string(), 0.0),

            div3t_beat1: Self::create_param("1/2T Beat 1".to_string(), 0.0),
            div3t_beat2: Self::create_param("1/2T Beat 2".to_string(), 0.0),
            div3t_beat3: Self::create_param("1/2T Beat 3".to_string(), 0.0),

            div6t_beat1: Self::create_param("1/4T Beat 1".to_string(), 0.0),
            div6t_beat2: Self::create_param("1/4T Beat 2".to_string(), 0.0),
            div6t_beat3: Self::create_param("1/4T Beat 3".to_string(), 0.0),
            div6t_beat4: Self::create_param("1/4T Beat 4".to_string(), 0.0),
            div6t_beat5: Self::create_param("1/4T Beat 5".to_string(), 0.0),
            div6t_beat6: Self::create_param("1/4T Beat 6".to_string(), 0.0),

            div12t_beat1: Self::create_param("1/8T Beat 1".to_string(), 0.0),
            div12t_beat2: Self::create_param("1/8T Beat 2".to_string(), 0.0),
            div12t_beat3: Self::create_param("1/8T Beat 3".to_string(), 0.0),
            div12t_beat4: Self::create_param("1/8T Beat 4".to_string(), 0.0),
            div12t_beat5: Self::create_param("1/8T Beat 5".to_string(), 0.0),
            div12t_beat6: Self::create_param("1/8T Beat 6".to_string(), 0.0),
            div12t_beat7: Self::create_param("1/8T Beat 7".to_string(), 0.0),
            div12t_beat8: Self::create_param("1/8T Beat 8".to_string(), 0.0),
            div12t_beat9: Self::create_param("1/8T Beat 9".to_string(), 0.0),
            div12t_beat10: Self::create_param("1/8T Beat 10".to_string(), 0.0),
            div12t_beat11: Self::create_param("1/8T Beat 11".to_string(), 0.0),
            div12t_beat12: Self::create_param("1/8T Beat 12".to_string(), 0.0),

            div24t_beat1: Self::create_param("1/16T Beat 1".to_string(), 0.0),
            div24t_beat2: Self::create_param("1/16T Beat 2".to_string(), 0.0),
            div24t_beat3: Self::create_param("1/16T Beat 3".to_string(), 0.0),
            div24t_beat4: Self::create_param("1/16T Beat 4".to_string(), 0.0),
            div24t_beat5: Self::create_param("1/16T Beat 5".to_string(), 0.0),
            div24t_beat6: Self::create_param("1/16T Beat 6".to_string(), 0.0),
            div24t_beat7: Self::create_param("1/16T Beat 7".to_string(), 0.0),
            div24t_beat8: Self::create_param("1/16T Beat 8".to_string(), 0.0),
            div24t_beat9: Self::create_param("1/16T Beat 9".to_string(), 0.0),
            div24t_beat10: Self::create_param("1/16T Beat 10".to_string(), 0.0),
            div24t_beat11: Self::create_param("1/16T Beat 11".to_string(), 0.0),
            div24t_beat12: Self::create_param("1/16T Beat 12".to_string(), 0.0),
            div24t_beat13: Self::create_param("1/16T Beat 13".to_string(), 0.0),
            div24t_beat14: Self::create_param("1/16T Beat 14".to_string(), 0.0),
            div24t_beat15: Self::create_param("1/16T Beat 15".to_string(), 0.0),
            div24t_beat16: Self::create_param("1/16T Beat 16".to_string(), 0.0),
            div24t_beat17: Self::create_param("1/16T Beat 17".to_string(), 0.0),
            div24t_beat18: Self::create_param("1/16T Beat 18".to_string(), 0.0),
            div24t_beat19: Self::create_param("1/16T Beat 19".to_string(), 0.0),
            div24t_beat20: Self::create_param("1/16T Beat 20".to_string(), 0.0),
            div24t_beat21: Self::create_param("1/16T Beat 21".to_string(), 0.0),
            div24t_beat22: Self::create_param("1/16T Beat 22".to_string(), 0.0),
            div24t_beat23: Self::create_param("1/16T Beat 23".to_string(), 0.0),
            div24t_beat24: Self::create_param("1/16T Beat 24".to_string(), 0.0),

            div2d_beat1: Self::create_param("1/2D Beat 1".to_string(), 0.0),
            div2d_beat2: Self::create_param("1/2D Beat 2".to_string(), 0.0),

            div3d_beat1: Self::create_param("1/4D Beat 1".to_string(), 0.0),
            div3d_beat2: Self::create_param("1/4D Beat 2".to_string(), 0.0),
            div3d_beat3: Self::create_param("1/4D Beat 3".to_string(), 0.0),

            div6d_beat1: Self::create_param("1/8D Beat 1".to_string(), 0.0),
            div6d_beat2: Self::create_param("1/8D Beat 2".to_string(), 0.0),
            div6d_beat3: Self::create_param("1/8D Beat 3".to_string(), 0.0),
            div6d_beat4: Self::create_param("1/8D Beat 4".to_string(), 0.0),
            div6d_beat5: Self::create_param("1/8D Beat 5".to_string(), 0.0),
            div6d_beat6: Self::create_param("1/8D Beat 6".to_string(), 0.0),

            div11d_beat1: Self::create_param("1/16D Beat 1".to_string(), 0.0),
            div11d_beat2: Self::create_param("1/16D Beat 2".to_string(), 0.0),
            div11d_beat3: Self::create_param("1/16D Beat 3".to_string(), 0.0),
            div11d_beat4: Self::create_param("1/16D Beat 4".to_string(), 0.0),
            div11d_beat5: Self::create_param("1/16D Beat 5".to_string(), 0.0),
            div11d_beat6: Self::create_param("1/16D Beat 6".to_string(), 0.0),
            div11d_beat7: Self::create_param("1/16D Beat 7".to_string(), 0.0),
            div11d_beat8: Self::create_param("1/16D Beat 8".to_string(), 0.0),
            div11d_beat9: Self::create_param("1/16D Beat 9".to_string(), 0.0),
            div11d_beat10: Self::create_param("1/16D Beat 10".to_string(), 0.0),
            div11d_beat11: Self::create_param("1/16D Beat 11".to_string(), 0.0),

            div22d_beat1: Self::create_param("1/32D Beat 1".to_string(), 0.0),
            div22d_beat2: Self::create_param("1/32D Beat 2".to_string(), 0.0),
            div22d_beat3: Self::create_param("1/32D Beat 3".to_string(), 0.0),
            div22d_beat4: Self::create_param("1/32D Beat 4".to_string(), 0.0),
            div22d_beat5: Self::create_param("1/32D Beat 5".to_string(), 0.0),
            div22d_beat6: Self::create_param("1/32D Beat 6".to_string(), 0.0),
            div22d_beat7: Self::create_param("1/32D Beat 7".to_string(), 0.0),
            div22d_beat8: Self::create_param("1/32D Beat 8".to_string(), 0.0),
            div22d_beat9: Self::create_param("1/32D Beat 9".to_string(), 0.0),
            div22d_beat10: Self::create_param("1/32D Beat 10".to_string(), 0.0),
            div22d_beat11: Self::create_param("1/32D Beat 11".to_string(), 0.0),
            div22d_beat12: Self::create_param("1/32D Beat 12".to_string(), 0.0),
            div22d_beat13: Self::create_param("1/32D Beat 13".to_string(), 0.0),
            div22d_beat14: Self::create_param("1/32D Beat 14".to_string(), 0.0),
            div22d_beat15: Self::create_param("1/32D Beat 15".to_string(), 0.0),
            div22d_beat16: Self::create_param("1/32D Beat 16".to_string(), 0.0),
            div22d_beat17: Self::create_param("1/32D Beat 17".to_string(), 0.0),
            div22d_beat18: Self::create_param("1/32D Beat 18".to_string(), 0.0),
            div22d_beat19: Self::create_param("1/32D Beat 19".to_string(), 0.0),
            div22d_beat20: Self::create_param("1/32D Beat 20".to_string(), 0.0),
            div22d_beat21: Self::create_param("1/32D Beat 21".to_string(), 0.0),
            div22d_beat22: Self::create_param("1/32D Beat 22".to_string(), 0.0),

            synth_osc_d: FloatParam::new(
                "Oscillator D".to_string(),
                0.5,
                FloatRange::Linear { min: 0.0, max: 1.0 }
            ).with_smoother(SmoothingStyle::Linear(50.0)),
            synth_osc_v: FloatParam::new(
                "Oscillator V".to_string(),
                0.5,
                FloatRange::Linear { min: 0.0, max: 1.0 }
            ).with_smoother(SmoothingStyle::Linear(50.0)),
            synth_osc_volume: FloatParam::new(
                "VPS Volume".to_string(),
                1.0,
                FloatRange::Linear { min: 0.0, max: 1.0 }
            ).with_smoother(SmoothingStyle::Linear(50.0)),
            synth_osc_octave: IntParam::new(
                "VPS Octave".to_string(),
                0,
                IntRange::Linear { min: -2, max: 2 }
            ),
            synth_sub_volume: FloatParam::new(
                "Sub Volume".to_string(),
                0.0,
                FloatRange::Linear { min: 0.0, max: 1.0 }
            ).with_smoother(SmoothingStyle::Linear(50.0)),
            synth_sub_octave: IntParam::new(
                "Sub Octave".to_string(),
                -1,
                IntRange::Linear { min: -2, max: 2 }
            ),
            synth_sub_shape: FloatParam::new(
                "Sub Shape".to_string(),
                0.0,
                FloatRange::Linear { min: 0.0, max: 1.0 }
            ).with_smoother(SmoothingStyle::Linear(50.0)),

            synth_polyblep_volume: FloatParam::new(
                "PolyBlep Volume".to_string(),
                0.0,
                FloatRange::Linear { min: 0.0, max: 1.0 }
            ).with_smoother(SmoothingStyle::Linear(50.0)),
            synth_polyblep_pulse_width: FloatParam::new(
                "PolyBlep Pulse Width".to_string(),
                0.5,
                FloatRange::Linear { min: 0.0, max: 1.0 }
            ).with_smoother(SmoothingStyle::Linear(50.0)),
            synth_polyblep_octave: IntParam::new(
                "PolyBlep Octave".to_string(),
                0,
                IntRange::Linear { min: -2, max: 2 }
            ),

            synth_pll_track_speed: FloatParam::new(
                "PLL Track Speed".to_string(),
                3.0,
                FloatRange::Skewed { min: 0.2, max: 10.0, factor: 0.3 }
            ).with_smoother(SmoothingStyle::Linear(50.0)),
            synth_pll_damping: FloatParam::new(
                "PLL Damping".to_string(),
                0.2,
                FloatRange::Linear { min: 0.0, max: 1.0 }
            ).with_smoother(SmoothingStyle::Linear(50.0)),
            synth_pll_range: FloatParam::new(
                "PLL Range".to_string(),
                1.0,
                FloatRange::Skewed { min: 0.1, max: 100.0, factor: 0.3 }
            ).with_smoother(SmoothingStyle::Linear(50.0)),
            synth_pll_mult: IntParam::new(
                "PLL Multiplier".to_string(),
                0,
                IntRange::Linear { min: 0, max: 4 }
            ),
            synth_pll_colored: BoolParam::new(
                "PLL Colored".to_string(),
                false
            ),
            synth_pll_mode: BoolParam::new(
                "PLL Edge Mode".to_string(),
                true
            ),
            synth_pll_ref_octave: IntParam::new(
                "PLL Ref Octave".to_string(),
                0,
                IntRange::Linear { min: -2, max: 2 }
            ),
            synth_pll_ref_tune: IntParam::new(
                "PLL Ref Tune".to_string(),
                0,
                IntRange::Linear { min: -12, max: 12 }
            ),
            synth_pll_ref_fine_tune: FloatParam::new(
                "PLL Ref Fine Tune".to_string(),
                0.0,
                FloatRange::Linear { min: -1.0, max: 1.0 }
            ).with_smoother(SmoothingStyle::Linear(50.0)),
            synth_pll_ref_pulse_width: FloatParam::new(
                "PLL Ref Pulse Width".to_string(),
                0.5,
                FloatRange::Linear { min: 0.01, max: 0.99 }
            ).with_smoother(SmoothingStyle::Linear(50.0)),
            synth_pll_feedback: FloatParam::new(
                "PLL Feedback".to_string(),
                0.0,
                FloatRange::Linear { min: 0.0, max: 0.1 }
            ).with_smoother(SmoothingStyle::Linear(50.0)),
            synth_pll_ki_multiplier: FloatParam::new(
                "PLL Ki Multiplier".to_string(),
                10000.0,
                FloatRange::Skewed { min: 1.0, max: 50000.0, factor: 0.3 }
            ).with_smoother(SmoothingStyle::Linear(50.0)),
            synth_pll_volume: FloatParam::new(
                "PLL Volume".to_string(),
                0.0,
                FloatRange::Linear { min: 0.0, max: 1.0 }
            ).with_smoother(SmoothingStyle::Linear(50.0)),
            synth_pll_distortion_amount: FloatParam::new(
                "PLL Distortion Amount".to_string(),
                0.0,
                FloatRange::Linear { min: 0.0, max: 1.0 }
            ).with_smoother(SmoothingStyle::Linear(50.0)),
            synth_pll_distortion_threshold: FloatParam::new(
                "PLL Distortion Threshold".to_string(),
                0.9,
                FloatRange::Linear { min: 0.0, max: 1.0 }
            ).with_smoother(SmoothingStyle::Linear(50.0)),
            synth_pll_stereo_damp_offset: FloatParam::new(
                "PLL Stereo Damp Δ".to_string(),
                0.0,
                FloatRange::Linear { min: 0.0, max: 0.3 }
            ).with_smoother(SmoothingStyle::Linear(50.0)),

            synth_distortion_amount: FloatParam::new(
                "Distortion Amount".to_string(),
                0.0,
                FloatRange::Linear { min: 0.0, max: 1.0 }
            ).with_smoother(SmoothingStyle::Linear(50.0)),
            synth_distortion_threshold: FloatParam::new(
                "Distortion Threshold".to_string(),
                0.9,
                FloatRange::Linear { min: 0.0, max: 1.0 }
            ).with_smoother(SmoothingStyle::Linear(50.0)),

            synth_filter_cutoff: FloatParam::new(
                "Filter Cutoff".to_string(),
                1000.0,
                FloatRange::Skewed { min: 20.0, max: 20000.0, factor: 0.3 }
            ).with_smoother(SmoothingStyle::Linear(50.0)),
            synth_filter_resonance: FloatParam::new(
                "Filter Resonance".to_string(),
                0.0,
                FloatRange::Linear { min: 0.0, max: 0.99 }
            ).with_smoother(SmoothingStyle::Linear(50.0)),
            synth_filter_env_amount: FloatParam::new(
                "Filter Env Amount".to_string(),
                0.0,
                FloatRange::Linear { min: -5000.0, max: 5000.0 }
            ).with_smoother(SmoothingStyle::Linear(50.0)),
            synth_filter_drive: FloatParam::new(
                "Filter Drive".to_string(),
                1.0,
                FloatRange::Linear { min: 1.0, max: 15.849 }
            ).with_smoother(SmoothingStyle::Linear(50.0)),
            synth_filter_mode: IntParam::new(
                "Filter Mode".to_string(),
                3,
                IntRange::Linear { min: 0, max: 10 }
            ),

            synth_volume: FloatParam::new(
                "Volume".to_string(),
                0.8,
                FloatRange::Linear { min: 0.0, max: 1.0 }
            ).with_smoother(SmoothingStyle::Linear(50.0)),

            synth_vol_attack: FloatParam::new(
                "Vol Attack".to_string(),
                10.0,
                FloatRange::Linear { min: 0.0, max: 1000.0 }
            ).with_smoother(SmoothingStyle::Linear(50.0)),
            synth_vol_attack_shape: FloatParam::new(
                "Vol Attack Shape".to_string(),
                0.5,
                FloatRange::Linear { min: 0.0, max: 1.0 }
            ).with_smoother(SmoothingStyle::Linear(50.0)),
            synth_vol_decay: FloatParam::new(
                "Vol Decay".to_string(),
                100.0,
                FloatRange::Linear { min: 0.0, max: 1000.0 }
            ).with_smoother(SmoothingStyle::Linear(50.0)),
            synth_vol_decay_shape: FloatParam::new(
                "Vol Decay Shape".to_string(),
                0.5,
                FloatRange::Linear { min: 0.0, max: 1.0 }
            ).with_smoother(SmoothingStyle::Linear(50.0)),
            synth_vol_sustain: FloatParam::new(
                "Vol Sustain".to_string(),
                0.7,
                FloatRange::Linear { min: 0.0, max: 1.0 }
            ).with_smoother(SmoothingStyle::Linear(50.0)),
            synth_vol_release: FloatParam::new(
                "Vol Release".to_string(),
                200.0,
                FloatRange::Linear { min: 0.0, max: 1000.0 }
            ).with_smoother(SmoothingStyle::Linear(50.0)),
            synth_vol_release_shape: FloatParam::new(
                "Vol Release Shape".to_string(),
                0.5,
                FloatRange::Linear { min: 0.0, max: 1.0 }
            ).with_smoother(SmoothingStyle::Linear(50.0)),

            synth_filt_attack: FloatParam::new(
                "Filt Attack".to_string(),
                10.0,
                FloatRange::Linear { min: 0.0, max: 1000.0 }
            ).with_smoother(SmoothingStyle::Linear(50.0)),
            synth_filt_attack_shape: FloatParam::new(
                "Filt Attack Shape".to_string(),
                0.5,
                FloatRange::Linear { min: 0.0, max: 1.0 }
            ).with_smoother(SmoothingStyle::Linear(50.0)),
            synth_filt_decay: FloatParam::new(
                "Filt Decay".to_string(),
                100.0,
                FloatRange::Linear { min: 0.0, max: 1000.0 }
            ).with_smoother(SmoothingStyle::Linear(50.0)),
            synth_filt_decay_shape: FloatParam::new(
                "Filt Decay Shape".to_string(),
                0.5,
                FloatRange::Linear { min: 0.0, max: 1.0 }
            ).with_smoother(SmoothingStyle::Linear(50.0)),
            synth_filt_sustain: FloatParam::new(
                "Filt Sustain".to_string(),
                0.5,
                FloatRange::Linear { min: 0.0, max: 1.0 }
            ).with_smoother(SmoothingStyle::Linear(50.0)),
            synth_filt_release: FloatParam::new(
                "Filt Release".to_string(),
                200.0,
                FloatRange::Linear { min: 0.0, max: 1000.0 }
            ).with_smoother(SmoothingStyle::Linear(50.0)),
            synth_filt_release_shape: FloatParam::new(
                "Filt Release Shape".to_string(),
                0.5,
                FloatRange::Linear { min: 0.0, max: 1.0 }
            ).with_smoother(SmoothingStyle::Linear(50.0)),

            note_length_percent: FloatParam::new(
                "Note Length %".to_string(),
                100.0,
                FloatRange::Linear { min: 0.0, max: 200.0 }
            ).with_smoother(SmoothingStyle::Linear(50.0)),
        }
    }
}
