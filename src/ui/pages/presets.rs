#![allow(clippy::field_reassign_with_default)]

use std::sync::Arc;
use egui_taffy::TuiBuilderLogic;
use nih_plug_egui::egui::{self, Color32};
use crate::params::DeviceParams;
use crate::ui::SharedUiState;
use crate::preset::manager::Bank;
use nih_plug::prelude::*;

#[derive(Clone, PartialEq)]
struct PresetPageState {
    current_bank: Bank,
    selected_preset: usize,
    editing_name: bool,
    name_buffer: String,
    status_message: Option<(String, std::time::Instant)>,
}

impl Default for PresetPageState {
    fn default() -> Self {
        Self {
            current_bank: Bank::A,
            selected_preset: 0,
            editing_name: false,
            name_buffer: String::new(),
            status_message: None,
        }
    }
}

pub fn render(
    tui: &mut egui_taffy::Tui,
    params: &Arc<DeviceParams>,
    setter: &ParamSetter,
    ui_state: &Arc<SharedUiState>,
) {
    let state_id = egui::Id::new("preset_page_state");

    tui.ui(|ui| {
        ui.add_space(12.0);
        ui.heading(egui::RichText::new("    Presets").size(14.0));
        ui.add_space(8.0);
    });

    tui.ui(|ui| {
        let mut state = ui.ctx().data_mut(|d| d.get_temp::<PresetPageState>(state_id).unwrap_or_default());

        if let Some((_, time)) = &state.status_message {
            if time.elapsed().as_secs() > 3 {
                state.status_message = None;
            }
        }

        egui::Frame::default()
            .fill(ui.visuals().extreme_bg_color)
            .inner_margin(15.0)
            .stroke(egui::Stroke::new(1.0, ui.visuals().window_stroke.color))
            .corner_radius(15.0)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Bank:").size(12.0));
                    ui.add_space(10.0);

                    for bank in Bank::all() {
                        let is_selected = state.current_bank == bank;
                        let button = egui::Button::new(
                            egui::RichText::new(bank.label()).size(14.0).strong()
                        )
                        .min_size(egui::vec2(40.0, 28.0))
                        .fill(if is_selected {
                            Color32::from_rgb(60, 100, 160)
                        } else {
                            Color32::from_rgb(50, 50, 50)
                        });

                        if ui.add(button).clicked() {
                            state.current_bank = bank;
                            state.editing_name = false;
                        }
                        ui.add_space(5.0);
                    }

                    ui.add_space(20.0);

                    if let Some((msg, _)) = &state.status_message {
                        ui.label(egui::RichText::new(msg).size(11.0).color(Color32::from_rgb(100, 200, 100)));
                    }
                });

                ui.add_space(15.0);

                if let Ok(manager) = ui_state.preset_manager.lock() {
                    let bank = manager.get_bank(state.current_bank);

                    ui.horizontal_wrapped(|ui| {
                        for i in 0..16 {
                            let _preset = &bank.presets[i];
                            let is_selected = state.selected_preset == i;
                            let is_current = manager.current_bank() == state.current_bank
                                && manager.current_preset_index() == i;

                            let fill_color = if is_current {
                                Color32::from_rgb(80, 140, 80)
                            } else if is_selected {
                                Color32::from_rgb(70, 100, 140)
                            } else {
                                Color32::from_rgb(45, 45, 45)
                            };

                            let button = egui::Button::new(
                                egui::RichText::new(format!("{:02}", i + 1)).size(11.0)
                            )
                            .min_size(egui::vec2(36.0, 36.0))
                            .fill(fill_color);

                            if ui.add(button).clicked() {
                                state.selected_preset = i;
                                state.editing_name = false;
                            }

                            if (i + 1) % 8 == 0 && i < 15 {
                                ui.end_row();
                                ui.add_space(5.0);
                            }
                        }
                    });

                    ui.add_space(15.0);

                    ui.separator();
                    ui.add_space(10.0);

                    let preset = &bank.presets[state.selected_preset];

                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("Name:").size(12.0));
                        ui.add_space(5.0);

                        if state.editing_name {
                            let response = ui.add(
                                egui::TextEdit::singleline(&mut state.name_buffer)
                                    .desired_width(200.0)
                                    .font(egui::TextStyle::Body)
                            );

                            if response.lost_focus() || ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                                state.editing_name = false;
                            }
                        } else {
                            ui.label(egui::RichText::new(&preset.name).size(14.0).strong());
                        }
                    });

                    ui.add_space(15.0);

                    ui.horizontal(|ui| {
                        let load_btn = egui::Button::new(
                            egui::RichText::new("Load").size(12.0)
                        ).min_size(egui::vec2(70.0, 28.0));

                        let save_btn = egui::Button::new(
                            egui::RichText::new("Save").size(12.0)
                        ).min_size(egui::vec2(70.0, 28.0));

                        let rename_btn = egui::Button::new(
                            egui::RichText::new("Rename").size(12.0)
                        ).min_size(egui::vec2(70.0, 28.0));

                        let save_file_btn = egui::Button::new(
                            egui::RichText::new("Save to File").size(12.0)
                        ).min_size(egui::vec2(90.0, 28.0));

                        let reset_btn = egui::Button::new(
                            egui::RichText::new("Reset All").size(12.0)
                        ).min_size(egui::vec2(80.0, 28.0))
                        .fill(Color32::from_rgb(100, 50, 50));

                        drop(manager);

                        if ui.add(load_btn).clicked() {
                            if let Ok(mut mgr) = ui_state.preset_manager.lock() {
                                mgr.set_current_bank(state.current_bank);
                                mgr.set_current_preset(state.selected_preset);
                                let preset = mgr.get_current_preset().clone();
                                drop(mgr);
                                load_preset_to_params(&preset.data, params, setter, ui_state);
                                state.status_message = Some((format!("Loaded: {}", preset.name), std::time::Instant::now()));
                            }
                        }

                        ui.add_space(5.0);

                        if ui.add(save_btn).clicked() {
                            let data = save_params_to_preset_data(params, ui_state);
                            if let Ok(mut mgr) = ui_state.preset_manager.lock() {
                                let current_name = mgr.get_preset(state.current_bank, state.selected_preset)
                                    .map(|p| p.name.clone())
                                    .unwrap_or_else(|| format!("Preset {}", state.selected_preset + 1));
                                let preset = crate::preset::Preset::with_data(&current_name, data);
                                mgr.save_to_slot(state.current_bank, state.selected_preset, preset);
                                state.status_message = Some(("Saved!".to_string(), std::time::Instant::now()));
                            }
                        }

                        ui.add_space(5.0);

                        if ui.add(rename_btn).clicked() {
                            if let Ok(mgr) = ui_state.preset_manager.lock() {
                                if let Some(preset) = mgr.get_preset(state.current_bank, state.selected_preset) {
                                    state.name_buffer = preset.name.clone();
                                    state.editing_name = true;
                                }
                            }
                        }

                        ui.add_space(15.0);

                        if ui.add(save_file_btn).clicked() {
                            if let Ok(mut mgr) = ui_state.preset_manager.lock() {
                                match mgr.save_to_file() {
                                    Ok(_) => {
                                        state.status_message = Some(("Saved to file!".to_string(), std::time::Instant::now()));
                                    }
                                    Err(e) => {
                                        state.status_message = Some((format!("Error: {}", e), std::time::Instant::now()));
                                    }
                                }
                            }
                        }

                        ui.add_space(15.0);

                        if ui.add(reset_btn).clicked() {
                            if let Ok(mut mgr) = ui_state.preset_manager.lock() {
                                mgr.reset_to_defaults();
                                state.status_message = Some(("Reset to defaults!".to_string(), std::time::Instant::now()));
                            }
                        }
                    });

                    if state.editing_name && !state.name_buffer.is_empty() {
                        if let Ok(mut mgr) = ui_state.preset_manager.lock() {
                            mgr.rename_preset(state.current_bank, state.selected_preset, &state.name_buffer);
                        }
                    }
                }
            });

        ui.ctx().data_mut(|d| d.insert_temp(state_id, state));
    });

    tui.ui(|ui| {
        ui.add_space(20.0);

        egui::Frame::default()
            .fill(ui.visuals().extreme_bg_color)
            .inner_margin(15.0)
            .stroke(egui::Stroke::new(1.0, ui.visuals().window_stroke.color))
            .corner_radius(15.0)
            .show(ui, |ui| {
                ui.label(egui::RichText::new("Preset Banks:").size(11.0).strong());
                ui.add_space(5.0);
                ui.label(egui::RichText::new("A - Basic Patterns (4/4, 8th, Shuffle, Breakbeat...)").size(10.0).color(Color32::GRAY));
                ui.label(egui::RichText::new("B - Electronic Genres (Techno, House, Acid, Trance...)").size(10.0).color(Color32::GRAY));
                ui.label(egui::RichText::new("C - Experimental (Glitch, IDM, Euclidean, Generative...)").size(10.0).color(Color32::GRAY));
                ui.label(egui::RichText::new("D - Ambient (Drones, Pads, Sparse, Atmospheric...)").size(10.0).color(Color32::GRAY));
            });
    });
}

fn load_preset_to_params(
    data: &crate::preset::PresetData,
    params: &Arc<DeviceParams>,
    setter: &ParamSetter,
    ui_state: &Arc<SharedUiState>,
) {
    for &v in data.straight_1_1.iter() {
        setter.set_parameter(&params.div1_beat1, v);
    }

    setter.set_parameter(&params.div2_beat1, data.straight_1_2[0]);
    setter.set_parameter(&params.div2_beat2, data.straight_1_2[1]);

    setter.set_parameter(&params.div4_beat1, data.straight_1_4[0]);
    setter.set_parameter(&params.div4_beat2, data.straight_1_4[1]);
    setter.set_parameter(&params.div4_beat3, data.straight_1_4[2]);
    setter.set_parameter(&params.div4_beat4, data.straight_1_4[3]);

    for (i, &v) in data.straight_1_8.iter().enumerate() {
        match i {
            0 => setter.set_parameter(&params.div8_beat1, v),
            1 => setter.set_parameter(&params.div8_beat2, v),
            2 => setter.set_parameter(&params.div8_beat3, v),
            3 => setter.set_parameter(&params.div8_beat4, v),
            4 => setter.set_parameter(&params.div8_beat5, v),
            5 => setter.set_parameter(&params.div8_beat6, v),
            6 => setter.set_parameter(&params.div8_beat7, v),
            7 => setter.set_parameter(&params.div8_beat8, v),
            _ => {}
        }
    }

    for (i, &v) in data.straight_1_16.iter().enumerate() {
        match i {
            0 => setter.set_parameter(&params.div16_beat1, v),
            1 => setter.set_parameter(&params.div16_beat2, v),
            2 => setter.set_parameter(&params.div16_beat3, v),
            3 => setter.set_parameter(&params.div16_beat4, v),
            4 => setter.set_parameter(&params.div16_beat5, v),
            5 => setter.set_parameter(&params.div16_beat6, v),
            6 => setter.set_parameter(&params.div16_beat7, v),
            7 => setter.set_parameter(&params.div16_beat8, v),
            8 => setter.set_parameter(&params.div16_beat9, v),
            9 => setter.set_parameter(&params.div16_beat10, v),
            10 => setter.set_parameter(&params.div16_beat11, v),
            11 => setter.set_parameter(&params.div16_beat12, v),
            12 => setter.set_parameter(&params.div16_beat13, v),
            13 => setter.set_parameter(&params.div16_beat14, v),
            14 => setter.set_parameter(&params.div16_beat15, v),
            15 => setter.set_parameter(&params.div16_beat16, v),
            _ => {}
        }
    }

    for (i, &v) in data.straight_1_32.iter().enumerate() {
        match i {
            0 => setter.set_parameter(&params.div32_beat1, v),
            1 => setter.set_parameter(&params.div32_beat2, v),
            2 => setter.set_parameter(&params.div32_beat3, v),
            3 => setter.set_parameter(&params.div32_beat4, v),
            4 => setter.set_parameter(&params.div32_beat5, v),
            5 => setter.set_parameter(&params.div32_beat6, v),
            6 => setter.set_parameter(&params.div32_beat7, v),
            7 => setter.set_parameter(&params.div32_beat8, v),
            8 => setter.set_parameter(&params.div32_beat9, v),
            9 => setter.set_parameter(&params.div32_beat10, v),
            10 => setter.set_parameter(&params.div32_beat11, v),
            11 => setter.set_parameter(&params.div32_beat12, v),
            12 => setter.set_parameter(&params.div32_beat13, v),
            13 => setter.set_parameter(&params.div32_beat14, v),
            14 => setter.set_parameter(&params.div32_beat15, v),
            15 => setter.set_parameter(&params.div32_beat16, v),
            16 => setter.set_parameter(&params.div32_beat17, v),
            17 => setter.set_parameter(&params.div32_beat18, v),
            18 => setter.set_parameter(&params.div32_beat19, v),
            19 => setter.set_parameter(&params.div32_beat20, v),
            20 => setter.set_parameter(&params.div32_beat21, v),
            21 => setter.set_parameter(&params.div32_beat22, v),
            22 => setter.set_parameter(&params.div32_beat23, v),
            23 => setter.set_parameter(&params.div32_beat24, v),
            24 => setter.set_parameter(&params.div32_beat25, v),
            25 => setter.set_parameter(&params.div32_beat26, v),
            26 => setter.set_parameter(&params.div32_beat27, v),
            27 => setter.set_parameter(&params.div32_beat28, v),
            28 => setter.set_parameter(&params.div32_beat29, v),
            29 => setter.set_parameter(&params.div32_beat30, v),
            30 => setter.set_parameter(&params.div32_beat31, v),
            31 => setter.set_parameter(&params.div32_beat32, v),
            _ => {}
        }
    }

    setter.set_parameter(&params.synth_pll_track_speed, data.synth_pll_track_speed);
    setter.set_parameter(&params.synth_pll_damping, data.synth_pll_damping);
    setter.set_parameter(&params.synth_pll_influence, data.synth_pll_influence);
    setter.set_parameter(&params.synth_pll_mult, data.synth_pll_mult);
    setter.set_parameter(&params.synth_pll_colored, data.synth_pll_colored);
    setter.set_parameter(&params.synth_pll_mode, data.synth_pll_mode);
    setter.set_parameter(&params.synth_pll_ref_octave, data.synth_pll_ref_octave);
    setter.set_parameter(&params.synth_pll_ref_tune, data.synth_pll_ref_tune);
    setter.set_parameter(&params.synth_pll_ref_fine_tune, data.synth_pll_ref_fine_tune);
    setter.set_parameter(&params.synth_pll_ref_pulse_width, data.synth_pll_ref_pulse_width);
    setter.set_parameter(&params.synth_pll_feedback, data.synth_pll_feedback);
    setter.set_parameter(&params.synth_pll_volume, data.synth_pll_volume);
    setter.set_parameter(&params.synth_pll_distortion_amount, data.synth_pll_distortion_amount);
    setter.set_parameter(&params.synth_pll_distortion_threshold, data.synth_pll_distortion_threshold);
    setter.set_parameter(&params.synth_pll_stereo_damp_offset, data.synth_pll_stereo_damp_offset);
    setter.set_parameter(&params.synth_pll_glide, data.synth_pll_glide);

    setter.set_parameter(&params.synth_osc_octave, data.synth_osc_octave);
    setter.set_parameter(&params.synth_osc_d, data.synth_osc_d);
    setter.set_parameter(&params.synth_osc_v, data.synth_osc_v);
    setter.set_parameter(&params.synth_osc_stereo_v_offset, data.synth_osc_stereo_v_offset);
    setter.set_parameter(&params.synth_osc_volume, data.synth_osc_volume);
    setter.set_parameter(&params.synth_distortion_amount, data.synth_distortion_amount);
    setter.set_parameter(&params.synth_distortion_threshold, data.synth_distortion_threshold);

    setter.set_parameter(&params.synth_sub_octave, data.synth_sub_octave);
    setter.set_parameter(&params.synth_sub_shape, data.synth_sub_shape);
    setter.set_parameter(&params.synth_sub_volume, data.synth_sub_volume);

    setter.set_parameter(&params.synth_polyblep_octave, data.synth_polyblep_octave);
    setter.set_parameter(&params.synth_polyblep_pulse_width, data.synth_polyblep_pulse_width);
    setter.set_parameter(&params.synth_polyblep_stereo_width, data.synth_polyblep_stereo_width);
    setter.set_parameter(&params.synth_polyblep_volume, data.synth_polyblep_volume);

    setter.set_parameter(&params.synth_filter_enable, data.synth_filter_enable);
    setter.set_parameter(&params.synth_filter_cutoff, data.synth_filter_cutoff);
    setter.set_parameter(&params.synth_filter_resonance, data.synth_filter_resonance);
    setter.set_parameter(&params.synth_filter_env_amount, data.synth_filter_env_amount);
    setter.set_parameter(&params.synth_filter_drive, data.synth_filter_drive);
    setter.set_parameter(&params.synth_filter_mode, data.synth_filter_mode);

    setter.set_parameter(&params.synth_vol_attack, data.synth_vol_attack);
    setter.set_parameter(&params.synth_vol_decay, data.synth_vol_decay);
    setter.set_parameter(&params.synth_vol_sustain, data.synth_vol_sustain);
    setter.set_parameter(&params.synth_vol_release, data.synth_vol_release);

    setter.set_parameter(&params.synth_filt_attack, data.synth_filt_attack);
    setter.set_parameter(&params.synth_filt_decay, data.synth_filt_decay);
    setter.set_parameter(&params.synth_filt_sustain, data.synth_filt_sustain);
    setter.set_parameter(&params.synth_filt_release, data.synth_filt_release);

    setter.set_parameter(&params.synth_reverb_mix, data.synth_reverb_mix);
    setter.set_parameter(&params.synth_reverb_time_scale, data.synth_reverb_time_scale);
    setter.set_parameter(&params.synth_reverb_decay, data.synth_reverb_decay);
    setter.set_parameter(&params.synth_reverb_diffusion, data.synth_reverb_diffusion);
    setter.set_parameter(&params.synth_reverb_pre_delay, data.synth_reverb_pre_delay);
    setter.set_parameter(&params.synth_reverb_mod_depth, data.synth_reverb_mod_depth);
    setter.set_parameter(&params.synth_reverb_hpf, data.synth_reverb_hpf);
    setter.set_parameter(&params.synth_reverb_lpf, data.synth_reverb_lpf);

    setter.set_parameter(&params.synth_volume, data.synth_volume);

    if let Ok(mut strength_values) = ui_state.strength_values.lock() {
        for (i, &v) in data.strength_values.iter().enumerate() {
            if i < strength_values.len() {
                strength_values[i] = v as f32 / 64.0;
            }
        }
    }

    if let Ok(mut note_pool) = ui_state.note_pool.lock() {
        note_pool.notes.clear();
        note_pool.set_root_note(data.root_note);

        for note_data in &data.notes {
            let chance = note_data.chance as f32 / 127.0;
            let strength_bias = (note_data.beat as f32 - 64.0) / 63.0;
            note_pool.set_note(note_data.midi_note, chance, strength_bias);
        }
    }
}

fn save_params_to_preset_data(
    params: &Arc<DeviceParams>,
    ui_state: &Arc<SharedUiState>,
) -> crate::preset::PresetData {
    let mut data = crate::preset::PresetData::default();

    data.straight_1_1 = [params.div1_beat1.modulated_plain_value()];

    data.straight_1_2 = [
        params.div2_beat1.modulated_plain_value(),
        params.div2_beat2.modulated_plain_value(),
    ];

    data.straight_1_4 = [
        params.div4_beat1.modulated_plain_value(),
        params.div4_beat2.modulated_plain_value(),
        params.div4_beat3.modulated_plain_value(),
        params.div4_beat4.modulated_plain_value(),
    ];

    data.straight_1_8 = [
        params.div8_beat1.modulated_plain_value(),
        params.div8_beat2.modulated_plain_value(),
        params.div8_beat3.modulated_plain_value(),
        params.div8_beat4.modulated_plain_value(),
        params.div8_beat5.modulated_plain_value(),
        params.div8_beat6.modulated_plain_value(),
        params.div8_beat7.modulated_plain_value(),
        params.div8_beat8.modulated_plain_value(),
    ];

    data.straight_1_16 = [
        params.div16_beat1.modulated_plain_value(),
        params.div16_beat2.modulated_plain_value(),
        params.div16_beat3.modulated_plain_value(),
        params.div16_beat4.modulated_plain_value(),
        params.div16_beat5.modulated_plain_value(),
        params.div16_beat6.modulated_plain_value(),
        params.div16_beat7.modulated_plain_value(),
        params.div16_beat8.modulated_plain_value(),
        params.div16_beat9.modulated_plain_value(),
        params.div16_beat10.modulated_plain_value(),
        params.div16_beat11.modulated_plain_value(),
        params.div16_beat12.modulated_plain_value(),
        params.div16_beat13.modulated_plain_value(),
        params.div16_beat14.modulated_plain_value(),
        params.div16_beat15.modulated_plain_value(),
        params.div16_beat16.modulated_plain_value(),
    ];

    data.straight_1_32 = [
        params.div32_beat1.modulated_plain_value(),
        params.div32_beat2.modulated_plain_value(),
        params.div32_beat3.modulated_plain_value(),
        params.div32_beat4.modulated_plain_value(),
        params.div32_beat5.modulated_plain_value(),
        params.div32_beat6.modulated_plain_value(),
        params.div32_beat7.modulated_plain_value(),
        params.div32_beat8.modulated_plain_value(),
        params.div32_beat9.modulated_plain_value(),
        params.div32_beat10.modulated_plain_value(),
        params.div32_beat11.modulated_plain_value(),
        params.div32_beat12.modulated_plain_value(),
        params.div32_beat13.modulated_plain_value(),
        params.div32_beat14.modulated_plain_value(),
        params.div32_beat15.modulated_plain_value(),
        params.div32_beat16.modulated_plain_value(),
        params.div32_beat17.modulated_plain_value(),
        params.div32_beat18.modulated_plain_value(),
        params.div32_beat19.modulated_plain_value(),
        params.div32_beat20.modulated_plain_value(),
        params.div32_beat21.modulated_plain_value(),
        params.div32_beat22.modulated_plain_value(),
        params.div32_beat23.modulated_plain_value(),
        params.div32_beat24.modulated_plain_value(),
        params.div32_beat25.modulated_plain_value(),
        params.div32_beat26.modulated_plain_value(),
        params.div32_beat27.modulated_plain_value(),
        params.div32_beat28.modulated_plain_value(),
        params.div32_beat29.modulated_plain_value(),
        params.div32_beat30.modulated_plain_value(),
        params.div32_beat31.modulated_plain_value(),
        params.div32_beat32.modulated_plain_value(),
    ];

    data.synth_pll_track_speed = params.synth_pll_track_speed.modulated_plain_value();
    data.synth_pll_damping = params.synth_pll_damping.modulated_plain_value();
    data.synth_pll_influence = params.synth_pll_influence.modulated_plain_value();
    data.synth_pll_mult = params.synth_pll_mult.value();
    data.synth_pll_colored = params.synth_pll_colored.value();
    data.synth_pll_mode = params.synth_pll_mode.value();
    data.synth_pll_ref_octave = params.synth_pll_ref_octave.value();
    data.synth_pll_ref_tune = params.synth_pll_ref_tune.value();
    data.synth_pll_ref_fine_tune = params.synth_pll_ref_fine_tune.modulated_plain_value();
    data.synth_pll_ref_pulse_width = params.synth_pll_ref_pulse_width.modulated_plain_value();
    data.synth_pll_feedback = params.synth_pll_feedback.modulated_plain_value();
    data.synth_pll_volume = params.synth_pll_volume.modulated_plain_value();
    data.synth_pll_distortion_amount = params.synth_pll_distortion_amount.modulated_plain_value();
    data.synth_pll_distortion_threshold = params.synth_pll_distortion_threshold.modulated_plain_value();
    data.synth_pll_stereo_damp_offset = params.synth_pll_stereo_damp_offset.modulated_plain_value();
    data.synth_pll_glide = params.synth_pll_glide.modulated_plain_value();

    data.synth_osc_octave = params.synth_osc_octave.value();
    data.synth_osc_d = params.synth_osc_d.modulated_plain_value();
    data.synth_osc_v = params.synth_osc_v.modulated_plain_value();
    data.synth_osc_stereo_v_offset = params.synth_osc_stereo_v_offset.modulated_plain_value();
    data.synth_osc_volume = params.synth_osc_volume.modulated_plain_value();
    data.synth_distortion_amount = params.synth_distortion_amount.modulated_plain_value();
    data.synth_distortion_threshold = params.synth_distortion_threshold.modulated_plain_value();

    data.synth_sub_octave = params.synth_sub_octave.value();
    data.synth_sub_shape = params.synth_sub_shape.modulated_plain_value();
    data.synth_sub_volume = params.synth_sub_volume.modulated_plain_value();

    data.synth_polyblep_octave = params.synth_polyblep_octave.value();
    data.synth_polyblep_pulse_width = params.synth_polyblep_pulse_width.modulated_plain_value();
    data.synth_polyblep_stereo_width = params.synth_polyblep_stereo_width.modulated_plain_value();
    data.synth_polyblep_volume = params.synth_polyblep_volume.modulated_plain_value();

    data.synth_filter_enable = params.synth_filter_enable.value();
    data.synth_filter_cutoff = params.synth_filter_cutoff.modulated_plain_value();
    data.synth_filter_resonance = params.synth_filter_resonance.modulated_plain_value();
    data.synth_filter_env_amount = params.synth_filter_env_amount.modulated_plain_value();
    data.synth_filter_drive = params.synth_filter_drive.modulated_plain_value();
    data.synth_filter_mode = params.synth_filter_mode.value();

    data.synth_vol_attack = params.synth_vol_attack.modulated_plain_value();
    data.synth_vol_decay = params.synth_vol_decay.modulated_plain_value();
    data.synth_vol_sustain = params.synth_vol_sustain.modulated_plain_value();
    data.synth_vol_release = params.synth_vol_release.modulated_plain_value();

    data.synth_filt_attack = params.synth_filt_attack.modulated_plain_value();
    data.synth_filt_decay = params.synth_filt_decay.modulated_plain_value();
    data.synth_filt_sustain = params.synth_filt_sustain.modulated_plain_value();
    data.synth_filt_release = params.synth_filt_release.modulated_plain_value();

    data.synth_reverb_mix = params.synth_reverb_mix.modulated_plain_value();
    data.synth_reverb_time_scale = params.synth_reverb_time_scale.modulated_plain_value();
    data.synth_reverb_decay = params.synth_reverb_decay.modulated_plain_value();
    data.synth_reverb_diffusion = params.synth_reverb_diffusion.modulated_plain_value();
    data.synth_reverb_pre_delay = params.synth_reverb_pre_delay.modulated_plain_value();
    data.synth_reverb_mod_depth = params.synth_reverb_mod_depth.modulated_plain_value();
    data.synth_reverb_hpf = params.synth_reverb_hpf.modulated_plain_value();
    data.synth_reverb_lpf = params.synth_reverb_lpf.modulated_plain_value();

    data.synth_volume = params.synth_volume.modulated_plain_value();

    if let Ok(strength_values) = ui_state.strength_values.lock() {
        for (i, &v) in strength_values.iter().enumerate() {
            if i < 96 {
                data.strength_values[i] = (v * 64.0).clamp(0.0, 64.0) as u8;
            }
        }
    }

    if let Ok(note_pool) = ui_state.note_pool.lock() {
        data.root_note = note_pool.root_note.unwrap_or(48);
        data.notes = note_pool.notes.iter()
            .filter(|n| n.midi_note != data.root_note)
            .map(|n| crate::preset::NotePresetData {
                midi_note: n.midi_note,
                chance: (n.chance * 127.0).clamp(0.0, 127.0) as u8,
                beat: ((n.strength_bias * 63.0) + 64.0).clamp(0.0, 127.0) as u8,
                beat_length: 64,
            })
            .collect();
    }

    data
}
