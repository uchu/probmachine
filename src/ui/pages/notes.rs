use std::sync::Arc;
use std::collections::HashMap;
use egui_taffy::TuiBuilderLogic;
use nih_plug_egui::egui::{self, Color32};
use crate::params::DeviceParams;
use crate::ui::SharedUiState;
use crate::sequencer::scales::{Scale, StabilityPattern, OctaveRandomization, OctaveDirection};

#[derive(Clone, PartialEq)]
struct NoteState {
    selected_note: Option<u8>,
    root_note: u8,
    note_chances: HashMap<u8, u8>,
    note_beats: HashMap<u8, u8>,
    note_beat_lengths: HashMap<u8, u8>,
    scroll_offset: f32,
    last_preset_version: u64,
    scale: Scale,
    stability_pattern: StabilityPattern,
    octave_randomization: OctaveRandomization,
}

impl Default for NoteState {
    fn default() -> Self {
        Self {
            selected_note: Some(48),
            root_note: 48,
            note_chances: HashMap::new(),
            note_beats: HashMap::new(),
            note_beat_lengths: HashMap::new(),
            scroll_offset: 14.0,
            last_preset_version: 0,
            scale: Scale::default(),
            stability_pattern: StabilityPattern::default(),
            octave_randomization: OctaveRandomization::default(),
        }
    }
}

fn sync_state_from_shared(state: &mut NoteState, ui_state: &Arc<SharedUiState>) {
    if let Ok(note_pool) = ui_state.note_pool.lock() {
        state.note_chances.clear();
        state.note_beats.clear();
        state.note_beat_lengths.clear();

        if let Some(root) = note_pool.root_note {
            state.root_note = root;
            state.selected_note = Some(root);
        }

        for note in &note_pool.notes {
            let is_root = Some(note.midi_note) == note_pool.root_note && note.octave_offset == 0;

            let chance = (note.chance * 127.0) as u8;
            let beat = ((note.strength_bias * 63.0) + 64.0) as u8;
            let beat_length = ((note.length_bias * 63.0) + 64.0) as u8;

            // For non-root notes, store chance if > 0
            // Root note chance is always 127 (fixed), so we don't store it
            if !is_root && chance > 0 {
                state.note_chances.insert(note.midi_note, chance);
            }

            // Store strength and length biases for all notes including root
            if beat != 64 {
                state.note_beats.insert(note.midi_note, beat);
            }
            if beat_length != 64 {
                state.note_beat_lengths.insert(note.midi_note, beat_length);
            }
        }
    }

    if let Ok(scale) = ui_state.scale.lock() {
        state.scale = *scale;
    }

    if let Ok(pattern) = ui_state.stability_pattern.lock() {
        state.stability_pattern = *pattern;
    }

    if let Ok(oct_rand) = ui_state.octave_randomization.lock() {
        state.octave_randomization = oct_rand.clone();
    }
}

pub fn render(
    tui: &mut egui_taffy::Tui,
    _params: &Arc<DeviceParams>,
    _setter: &nih_plug::prelude::ParamSetter,
    ui_state: &Arc<SharedUiState>,
) {
    let state_id = egui::Id::new("note_state");
    let current_version = ui_state.get_preset_version();

    tui.ui(|ui| {
        let mut state = ui.ctx().data_mut(|d| d.get_temp::<NoteState>(state_id).unwrap_or_default());

        if state.last_preset_version != current_version {
            sync_state_from_shared(&mut state, ui_state);
            state.last_preset_version = current_version;
            ui.ctx().data_mut(|d| d.insert_temp(state_id, state.clone()));
        }

        let state_before = state.clone();

        ui.add_space(8.0);
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Scale:").size(18.0));
            ui.add_space(8.0);

            egui::ComboBox::from_id_salt("scale_select")
                .selected_text(egui::RichText::new(state.scale.name()).size(18.0))
                .width(220.0)
                .height(400.0)
                .show_ui(ui, |ui| {
                    ui.style_mut().spacing.item_spacing.y = 8.0;
                    for scale in Scale::all() {
                        let btn = egui::Button::new(egui::RichText::new(scale.name()).size(18.0))
                            .min_size(egui::vec2(200.0, 32.0))
                            .selected(state.scale == *scale);
                        if ui.add(btn).clicked() {
                            state.scale = *scale;
                            ui.close_menu();
                        }
                    }
                });

            ui.add_space(32.0);

            ui.label(egui::RichText::new("Pattern:").size(18.0));
            ui.add_space(8.0);

            egui::ComboBox::from_id_salt("pattern_select")
                .selected_text(egui::RichText::new(state.stability_pattern.name()).size(18.0))
                .width(200.0)
                .height(300.0)
                .show_ui(ui, |ui| {
                    ui.style_mut().spacing.item_spacing.y = 8.0;
                    for pattern in StabilityPattern::all() {
                        let btn = egui::Button::new(egui::RichText::new(pattern.name()).size(18.0))
                            .min_size(egui::vec2(180.0, 32.0))
                            .selected(state.stability_pattern == *pattern);
                        if ui.add(btn).clicked() {
                            state.stability_pattern = *pattern;
                            ui.close_menu();
                        }
                    }
                });
        });
        ui.add_space(12.0);

        if state != state_before {
            if state.scale != state_before.scale || state.stability_pattern != state_before.stability_pattern {
                apply_scale_and_pattern(&mut state);
                update_shared_state(&state, ui_state);
            }
            ui.ctx().data_mut(|d| d.insert_temp(state_id, state.clone()));
        }
    });

    tui.ui(|ui| {
        let mut state = ui.ctx().data_mut(|d| d.get_temp::<NoteState>(state_id).unwrap_or_default());
        let state_before = state.clone();

        render_piano_container(ui, &mut state);

        if state != state_before {
            ui.ctx().data_mut(|d| d.insert_temp(state_id, state.clone()));
            if (state.scroll_offset - state_before.scroll_offset).abs() > 0.01 {
                ui.ctx().request_repaint_after(std::time::Duration::from_millis(16));
            }
        }
    });

    tui.ui(|ui| {
        let mut state = ui.ctx().data_mut(|d| d.get_temp::<NoteState>(state_id).unwrap_or_default());
        let state_before = state.clone();

        ui.add_space(16.0);
        ui.horizontal(|ui| {
            render_selected_note_info(ui, &mut state);
            ui.add_space(16.0);
            render_octave_randomization(ui, &mut state);
        });

        if state != state_before {
            let chances_changed = state.note_chances != state_before.note_chances;
            let beats_changed = state.note_beats != state_before.note_beats;
            let beat_lengths_changed = state.note_beat_lengths != state_before.note_beat_lengths;
            let root_changed = state.root_note != state_before.root_note;
            let octave_changed = state.octave_randomization != state_before.octave_randomization;

            if chances_changed || beats_changed || beat_lengths_changed || root_changed {
                update_shared_state(&state, ui_state);
            }

            if octave_changed {
                update_octave_randomization_shared(&state.octave_randomization, ui_state);
            }

            ui.ctx().data_mut(|d| d.insert_temp(state_id, state));
            ui.ctx().request_repaint_after(std::time::Duration::from_millis(16));
        }
    });
}

fn apply_scale_and_pattern(state: &mut NoteState) {
    state.note_chances.clear();
    state.note_beats.clear();
    state.note_beat_lengths.clear();

    let root_pitch_class = state.root_note % 12;
    let root_octave = state.root_note / 12;

    for &interval in state.scale.intervals() {
        let midi_note = root_octave * 12 + root_pitch_class + interval;
        if midi_note > 127 {
            continue;
        }

        if let Some(degree) = state.scale.degree_for_interval(interval) {
            let base_chance = state.scale.base_chance_for_degree(degree);
            let settings = state.stability_pattern.get_stability_settings(degree);

            if midi_note != state.root_note {
                state.note_chances.insert(midi_note, base_chance);
                state.note_beats.insert(midi_note, settings.strength_pref);
                state.note_beat_lengths.insert(midi_note, settings.length_pref);
            }
        }
    }
}

fn render_octave_randomization(ui: &mut egui::Ui, state: &mut NoteState) {
    let label_width = 80.0;
    let slider_width = 280.0;

    egui::Frame::NONE

        .inner_margin(16.0)


        .show(ui, |ui| {
            ui.set_min_width(450.0);
            ui.vertical(|ui| {
                ui.label(egui::RichText::new("Octave Randomization").size(18.0).color(Color32::from_gray(180)));
                ui.add_space(12.0);

                ui.horizontal(|ui| {
                    ui.add_sized(egui::vec2(label_width, 20.0), egui::Label::new(egui::RichText::new("Chance:").size(16.0)));
                    let mut chance = state.octave_randomization.chance;
                    ui.style_mut().spacing.slider_width = slider_width;
                    ui.style_mut().spacing.slider_rail_height = 10.0;
                    if ui.add(egui::Slider::new(&mut chance, 0..=127).show_value(true)).changed() {
                        state.octave_randomization.chance = chance;
                    }
                });

                ui.add_space(12.0);

                ui.horizontal(|ui| {
                    ui.add_sized(egui::vec2(label_width, 20.0), egui::Label::new(egui::RichText::new("Strength:").size(16.0)));
                    let mut strength = state.octave_randomization.strength_pref;
                    ui.style_mut().spacing.slider_width = slider_width;
                    ui.style_mut().spacing.slider_rail_height = 10.0;
                    if ui.add(egui::Slider::new(&mut strength, 0..=127).show_value(true)).changed() {
                        state.octave_randomization.strength_pref = strength;
                    }
                });

                ui.horizontal(|ui| {
                    ui.add_space(label_width);
                    ui.label(egui::RichText::new("Weak").size(12.0).color(Color32::from_gray(120)));
                    ui.add_space(slider_width / 2.0 - 40.0);
                    ui.label(egui::RichText::new("Any").size(12.0).color(Color32::from_gray(120)));
                    ui.add_space(slider_width / 2.0 - 40.0);
                    ui.label(egui::RichText::new("Strong").size(12.0).color(Color32::from_gray(120)));
                });

                ui.add_space(12.0);

                ui.horizontal(|ui| {
                    ui.add_sized(egui::vec2(label_width, 20.0), egui::Label::new(egui::RichText::new("Length:").size(16.0)));
                    let mut length = state.octave_randomization.length_pref;
                    ui.style_mut().spacing.slider_width = slider_width;
                    ui.style_mut().spacing.slider_rail_height = 10.0;
                    if ui.add(egui::Slider::new(&mut length, 0..=127).show_value(true)).changed() {
                        state.octave_randomization.length_pref = length;
                    }
                });

                ui.horizontal(|ui| {
                    ui.add_space(label_width);
                    ui.label(egui::RichText::new("Short").size(12.0).color(Color32::from_gray(120)));
                    ui.add_space(slider_width / 2.0 - 40.0);
                    ui.label(egui::RichText::new("Any").size(12.0).color(Color32::from_gray(120)));
                    ui.add_space(slider_width / 2.0 - 40.0);
                    ui.label(egui::RichText::new("Long").size(12.0).color(Color32::from_gray(120)));
                });

                ui.add_space(12.0);

                ui.horizontal(|ui| {
                    ui.add_sized(egui::vec2(label_width, 20.0), egui::Label::new(egui::RichText::new("Direction:").size(16.0)));
                    for dir in OctaveDirection::all() {
                        let is_selected = state.octave_randomization.direction == *dir;
                        let btn = egui::Button::new(egui::RichText::new(dir.name()).size(14.0))
                            .min_size(egui::vec2(60.0, 28.0))
                            .selected(is_selected);
                        if ui.add(btn).clicked() {
                            state.octave_randomization.direction = *dir;
                        }
                        ui.add_space(8.0);
                    }
                });
            });
        });
}

fn render_piano_container(ui: &mut egui::Ui, state: &mut NoteState) {
    ui.set_max_width(1220.0);

    egui::Frame::NONE

        .inner_margin(16.0)


        .show(ui, |ui| {
            render_piano_keys(ui, state);

            ui.add_space(12.0);

            let total_white_keys = 50.0;
            let visible_white_keys = 15.0;
            let max_scroll = total_white_keys - visible_white_keys;

            let scrollbar_width = 1180.0;
            let scrollbar_height = 24.0;

            let (rect, response) = ui.allocate_exact_size(
                egui::vec2(scrollbar_width, scrollbar_height),
                egui::Sense::click_and_drag()
            );

            if response.dragged() || response.clicked() {
                if let Some(pos) = response.interact_pointer_pos() {
                    let relative_x = (pos.x - rect.left()).max(0.0).min(rect.width());
                    let ratio = relative_x / rect.width();
                    let new_scroll = (ratio * max_scroll).clamp(0.0, max_scroll);
                    let new_scroll_int = new_scroll.round();

                    if (new_scroll_int - state.scroll_offset).abs() >= 0.9 {
                        state.scroll_offset = new_scroll_int;
                    }
                }
            }

            let painter = ui.painter();

            let bg_color = ui.visuals().extreme_bg_color;
            painter.rect_filled(rect, 3.0, bg_color);

            let handle_ratio = visible_white_keys / total_white_keys;
            let handle_width = rect.width() * handle_ratio;
            let scroll_ratio = state.scroll_offset / max_scroll;
            let handle_x_offset = scroll_ratio * (rect.width() - handle_width);

            let handle_rect = egui::Rect::from_min_size(
                egui::pos2(rect.left() + handle_x_offset, rect.top()),
                egui::vec2(handle_width, scrollbar_height)
            );

            let handle_color = if response.dragged() {
                Color32::from_rgb(100, 100, 100)
            } else if response.hovered() {
                Color32::from_rgb(120, 120, 120)
            } else {
                Color32::from_rgb(140, 140, 140)
            };

            painter.rect_filled(handle_rect, 3.0, handle_color);
            painter.rect_stroke(
                handle_rect,
                3.0,
                egui::Stroke::new(1.0, Color32::from_rgb(80, 80, 80)),
                egui::epaint::StrokeKind::Outside,
            );
        });
}

fn render_piano_keys(ui: &mut egui::Ui, state: &mut NoteState) {
    let keyboard_width = 1180.0;
    let white_key_height = 170.0;

    let num_white_keys = 15;
    let white_key_width = keyboard_width / num_white_keys as f32;

    let black_key_width = white_key_width * 0.6;
    let black_key_height = white_key_height * 0.6;

    let white_key_pattern = [0, 2, 4, 5, 7, 9, 11];
    let black_key_pattern = [1, 3, 6, 8, 10];

    struct KeyInfo {
        rect: egui::Rect,
        is_black: bool,
        hovered: bool,
        clicked: bool,
        label: Option<String>,
        midi_note: u8,
    }

    ui.horizontal(|ui| {
        let mut keys = Vec::new();
        let start_pos = ui.cursor().min;

        let scroll_offset_int = state.scroll_offset as usize;

        for local_white_key_idx in 0..num_white_keys {
            let global_white_key_idx = scroll_offset_int + local_white_key_idx;
            let octave = global_white_key_idx / 7;
            let note_in_octave = global_white_key_idx % 7;

            let x = start_pos.x + (local_white_key_idx as f32 * white_key_width);
            let y = start_pos.y;

            let key_rect = egui::Rect::from_min_size(
                egui::pos2(x, y),
                egui::vec2(white_key_width - 1.0, white_key_height),
            );

            let response = ui.allocate_rect(key_rect, egui::Sense::click());

            let note_offset = white_key_pattern[note_in_octave];
            let midi_note = (octave * 12) as u8 + note_offset + 12;

            let label = if note_in_octave == 0 {
                let midi_octave = (midi_note / 12) as i32 - 1;
                Some(format!("C{}", midi_octave))
            } else {
                None
            };

            keys.push(KeyInfo {
                rect: key_rect,
                is_black: false,
                hovered: response.hovered(),
                clicked: response.clicked(),
                label,
                midi_note,
            });
        }

        for local_white_key_idx in 0..num_white_keys {
            let global_white_key_idx = scroll_offset_int + local_white_key_idx;
            let octave = global_white_key_idx / 7;
            let note_in_octave = global_white_key_idx % 7;

            let white_note = white_key_pattern[note_in_octave];
            let has_black_key_after = black_key_pattern.iter().any(|&black| black == white_note + 1);

            let is_last_white_key = local_white_key_idx == num_white_keys - 1;

            if has_black_key_after && !is_last_white_key {
                let black_note = white_note + 1;
                let midi_note = (octave * 12) as u8 + black_note + 12;

                if midi_note <= 96 {
                    let x = start_pos.x + (local_white_key_idx as f32 * white_key_width) + white_key_width - (black_key_width / 2.0);
                    let y = start_pos.y;

                    let key_rect = egui::Rect::from_min_size(
                        egui::pos2(x, y),
                        egui::vec2(black_key_width, black_key_height),
                    );

                    let response = ui.allocate_rect(key_rect, egui::Sense::click());

                    keys.push(KeyInfo {
                        rect: key_rect,
                        is_black: true,
                        hovered: response.hovered(),
                        clicked: response.clicked(),
                        label: None,
                        midi_note,
                    });
                }
            }
        }

        let painter = ui.painter();

        for key in &keys {
            if key.clicked {
                state.selected_note = Some(key.midi_note);
            }

            let is_selected = state.selected_note == Some(key.midi_note);
            let is_root = state.root_note == key.midi_note;

            let (fill_color, stroke_color) = if key.is_black {
                let fill = if is_selected {
                    Color32::from_rgb(80, 120, 180)
                } else if key.hovered {
                    Color32::from_rgb(60, 60, 60)
                } else {
                    Color32::from_rgb(30, 30, 30)
                };
                (fill, Color32::from_rgb(10, 10, 10))
            } else {
                let fill = if is_selected {
                    Color32::from_rgb(150, 180, 220)
                } else if key.hovered {
                    Color32::from_rgb(220, 220, 220)
                } else {
                    Color32::from_rgb(255, 255, 255)
                };
                (fill, Color32::from_rgb(100, 100, 100))
            };

            painter.rect_filled(key.rect, 2.0, fill_color);

            let key_width = key.rect.width();
            let section_width = key_width / 3.0;

            let chance_value = if is_root {
                127
            } else {
                *state.note_chances.get(&key.midi_note).unwrap_or(&0)
            };

            if chance_value > 0 {
                let chance_ratio = chance_value as f32 / 127.0;
                let gray_height = key.rect.height() * chance_ratio;
                let gray_rect = egui::Rect::from_min_size(
                    egui::pos2(key.rect.left(), key.rect.bottom() - gray_height),
                    egui::vec2(section_width, gray_height),
                );

                let gray_color = if key.is_black {
                    Color32::from_rgba_unmultiplied(80, 80, 80, 150)
                } else {
                    Color32::from_rgba_unmultiplied(120, 120, 120, 150)
                };

                painter.rect_filled(gray_rect, 2.0, gray_color);
            }

            let beat_value = if is_root {
                64
            } else {
                *state.note_beats.get(&key.midi_note).unwrap_or(&64)
            };

            if beat_value != 64 {
                let beat_ratio = if beat_value < 64 {
                    (64 - beat_value) as f32 / 64.0
                } else {
                    (beat_value - 64) as f32 / 63.0
                };

                let blue_height = key.rect.height() * beat_ratio;
                let blue_rect = egui::Rect::from_min_size(
                    egui::pos2(key.rect.left() + section_width, key.rect.bottom() - blue_height),
                    egui::vec2(section_width, blue_height),
                );

                let blue_color = if beat_value < 64 {
                    Color32::from_rgba_unmultiplied(100, 150, 255, 120)
                } else {
                    Color32::from_rgba_unmultiplied(50, 100, 200, 120)
                };

                painter.rect_filled(blue_rect, 2.0, blue_color);
            }

            let beat_length_value = if is_root {
                64
            } else {
                *state.note_beat_lengths.get(&key.midi_note).unwrap_or(&64)
            };

            if beat_length_value != 64 {
                let beat_length_ratio = if beat_length_value < 64 {
                    (64 - beat_length_value) as f32 / 64.0
                } else {
                    (beat_length_value - 64) as f32 / 63.0
                };

                let green_height = key.rect.height() * beat_length_ratio;
                let green_rect = egui::Rect::from_min_size(
                    egui::pos2(key.rect.left() + section_width * 2.0, key.rect.bottom() - green_height),
                    egui::vec2(section_width, green_height),
                );

                let green_color = if beat_length_value < 64 {
                    Color32::from_rgba_unmultiplied(100, 200, 100, 120)
                } else {
                    Color32::from_rgba_unmultiplied(50, 150, 50, 120)
                };

                painter.rect_filled(green_rect, 2.0, green_color);
            }

            let stroke_width = if is_root { 2.0 } else { 1.0 };
            let final_stroke_color = if is_root {
                Color32::from_rgb(255, 100, 100)
            } else {
                stroke_color
            };

            painter.rect_stroke(
                key.rect,
                2.0,
                egui::Stroke::new(stroke_width, final_stroke_color),
                egui::epaint::StrokeKind::Outside,
            );

            if let Some(ref label) = key.label {
                let text_pos = egui::pos2(
                    key.rect.center().x,
                    key.rect.bottom() - 20.0,
                );
                painter.text(
                    text_pos,
                    egui::Align2::CENTER_CENTER,
                    label,
                    egui::FontId::proportional(14.0),
                    Color32::from_rgb(80, 80, 80),
                );
            }
        }
    });
}

fn midi_note_to_name(midi_note: u8) -> String {
    let note_names = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];
    let octave = (midi_note / 12) as i32 - 1;
    let note_index = (midi_note % 12) as usize;
    format!("{}{}", note_names[note_index], octave)
}

fn update_shared_state(state: &NoteState, ui_state: &Arc<SharedUiState>) {
    if let Ok(mut note_pool) = ui_state.note_pool.lock() {
        note_pool.notes.clear();
        note_pool.set_root_note(state.root_note);

        // Update root note's biases (chance stays at 1.0)
        let root_beat = state.note_beats.get(&state.root_note).copied().unwrap_or(64);
        let root_beat_length = state.note_beat_lengths.get(&state.root_note).copied().unwrap_or(64);
        let root_strength_bias = (root_beat as f32 - 64.0) / 63.0;
        let root_length_bias = (root_beat_length as f32 - 64.0) / 63.0;
        note_pool.set_note_full(state.root_note, 0, 1.0, root_strength_bias, root_length_bias);

        for midi_note in 0..=127 {
            if midi_note == state.root_note {
                continue; // Already handled above
            }

            let chance = state.note_chances.get(&midi_note).copied().unwrap_or(0);
            let beat = state.note_beats.get(&midi_note).copied().unwrap_or(64);
            let beat_length = state.note_beat_lengths.get(&midi_note).copied().unwrap_or(64);

            if chance == 0 {
                continue;
            }

            let chance_normalized = chance as f32 / 127.0;
            let strength_bias = (beat as f32 - 64.0) / 63.0;
            let length_bias = (beat_length as f32 - 64.0) / 63.0;

            note_pool.set_note_full(midi_note, 0, chance_normalized, strength_bias, length_bias);
        }
    }

    if let Ok(mut scale) = ui_state.scale.lock() {
        *scale = state.scale;
    }

    if let Ok(mut pattern) = ui_state.stability_pattern.lock() {
        *pattern = state.stability_pattern;
    }
}

fn update_octave_randomization_shared(oct_rand: &OctaveRandomization, ui_state: &Arc<SharedUiState>) {
    if let Ok(mut shared_oct_rand) = ui_state.octave_randomization.lock() {
        *shared_oct_rand = oct_rand.clone();
    }
}

fn render_selected_note_info(ui: &mut egui::Ui, state: &mut NoteState) {
    let label_width = 80.0;
    let slider_width = 280.0;

    egui::Frame::NONE

        .inner_margin(16.0)


        .show(ui, |ui| {
            ui.set_min_width(450.0);
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    let selected_note_name = if let Some(note) = state.selected_note {
                        midi_note_to_name(note)
                    } else {
                        "None".to_string()
                    };

                    let root_note_name = midi_note_to_name(state.root_note);

                    ui.label(egui::RichText::new(format!("Selected: {}", selected_note_name)).size(18.0));
                    ui.add_space(24.0);
                    ui.label(egui::RichText::new(format!("Root: {}", root_note_name)).size(18.0));

                    ui.add_space(24.0);

                    if let Some(selected) = state.selected_note {
                        let btn = egui::Button::new(egui::RichText::new("Set as Root").size(14.0))
                            .min_size(egui::vec2(100.0, 32.0));
                        if ui.add(btn).clicked() {
                            state.root_note = selected;
                        }
                    }
                });

                ui.add_space(12.0);

                if let Some(selected) = state.selected_note {
                    ui.horizontal(|ui| {
                        ui.add_sized(egui::vec2(label_width, 20.0), egui::Label::new(egui::RichText::new("Chance:").size(16.0)));

                        let is_root = selected == state.root_note;
                        let mut chance_value = if is_root {
                            127
                        } else {
                            *state.note_chances.get(&selected).unwrap_or(&0)
                        };

                        ui.style_mut().spacing.slider_width = slider_width;
                        ui.style_mut().spacing.slider_rail_height = 10.0;
                        let slider = egui::Slider::new(&mut chance_value, 0..=127).show_value(true);

                        let slider_response = if is_root {
                            ui.add_enabled(false, slider)
                        } else {
                            ui.add(slider)
                        };

                        if slider_response.changed() && !is_root {
                            state.note_chances.insert(selected, chance_value);
                        }
                    });

                    ui.add_space(20.0);

                    ui.horizontal(|ui| {
                        ui.add_sized(egui::vec2(label_width, 20.0), egui::Label::new(egui::RichText::new("Strength:").size(16.0)));

                        let mut beat_value = *state.note_beats.get(&selected).unwrap_or(&64);

                        ui.style_mut().spacing.slider_width = slider_width;
                        ui.style_mut().spacing.slider_rail_height = 10.0;
                        let slider = egui::Slider::new(&mut beat_value, 0..=127).show_value(true);
                        let slider_response = ui.add(slider);

                        if slider_response.changed() {
                            state.note_beats.insert(selected, beat_value);
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.add_space(label_width);
                        ui.label(egui::RichText::new("Weak").size(12.0).color(Color32::from_gray(120)));
                        ui.add_space(slider_width / 2.0 - 40.0);
                        ui.label(egui::RichText::new("Any").size(12.0).color(Color32::from_gray(120)));
                        ui.add_space(slider_width / 2.0 - 40.0);
                        ui.label(egui::RichText::new("Strong").size(12.0).color(Color32::from_gray(120)));
                    });

                    ui.add_space(12.0);

                    ui.horizontal(|ui| {
                        ui.add_sized(egui::vec2(label_width, 20.0), egui::Label::new(egui::RichText::new("Length:").size(16.0)));

                        let mut beat_length_value = *state.note_beat_lengths.get(&selected).unwrap_or(&64);

                        ui.style_mut().spacing.slider_width = slider_width;
                        ui.style_mut().spacing.slider_rail_height = 10.0;
                        let slider = egui::Slider::new(&mut beat_length_value, 0..=127).show_value(true);
                        let slider_response = ui.add(slider);

                        if slider_response.changed() {
                            state.note_beat_lengths.insert(selected, beat_length_value);
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.add_space(label_width);
                        ui.label(egui::RichText::new("Short").size(12.0).color(Color32::from_gray(120)));
                        ui.add_space(slider_width / 2.0 - 40.0);
                        ui.label(egui::RichText::new("Any").size(12.0).color(Color32::from_gray(120)));
                        ui.add_space(slider_width / 2.0 - 40.0);
                        ui.label(egui::RichText::new("Long").size(12.0).color(Color32::from_gray(120)));
                    });
                }
            });
        });
}
