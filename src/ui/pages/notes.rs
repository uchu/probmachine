use std::sync::Arc;
use std::collections::HashMap;
use egui_taffy::TuiBuilderLogic;
use nih_plug_egui::egui::{self, Color32};
use crate::params::DeviceParams;
use crate::ui::SharedUiState;
use nih_plug::prelude::*;

#[derive(Clone, PartialEq)]
struct NoteState {
    selected_note: Option<u8>,
    root_note: u8,
    note_chances: HashMap<u8, u8>,
    note_beats: HashMap<u8, u8>,
    note_beat_lengths: HashMap<u8, u8>,
    scroll_offset: f32,
    last_preset_version: u64,
}

impl Default for NoteState {
    fn default() -> Self {
        Self {
            selected_note: Some(48), // C3
            root_note: 48,           // C3 (default root note)
            note_chances: HashMap::new(),
            note_beats: HashMap::new(),
            note_beat_lengths: HashMap::new(),
            scroll_offset: 18.0, // Position to show C3 area
            last_preset_version: 0,
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
            if Some(note.midi_note) == note_pool.root_note {
                continue;
            }

            let chance = (note.chance * 127.0) as u8;
            let beat = ((note.strength_bias * 63.0) + 64.0) as u8;

            if chance > 0 {
                state.note_chances.insert(note.midi_note, chance);
            }
            if beat != 64 {
                state.note_beats.insert(note.midi_note, beat);
            }
        }
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
        ui.add_space(12.0);
        ui.heading(egui::RichText::new("    Note Probability").size(14.0));
        ui.add_space(8.0);
    });

    tui.ui(|ui| {
        let mut state = ui.ctx().data_mut(|d| d.get_temp::<NoteState>(state_id).unwrap_or_default());

        if state.last_preset_version != current_version {
            sync_state_from_shared(&mut state, ui_state);
            state.last_preset_version = current_version;
            ui.ctx().data_mut(|d| d.insert_temp(state_id, state.clone()));
        }

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

        render_selected_note_info(ui, &mut state);

        if state != state_before {
            let chances_changed = state.note_chances != state_before.note_chances;
            let beats_changed = state.note_beats != state_before.note_beats;
            let beat_lengths_changed = state.note_beat_lengths != state_before.note_beat_lengths;
            let root_changed = state.root_note != state_before.root_note;

            if chances_changed || beats_changed || beat_lengths_changed || root_changed {
                log_note_values(&state);
                update_shared_state(&state, ui_state);
            }

            ui.ctx().data_mut(|d| d.insert_temp(state_id, state));
            ui.ctx().request_repaint_after(std::time::Duration::from_millis(16));
        }
    });
}

fn render_piano_container(ui: &mut egui::Ui, state: &mut NoteState) {
    ui.set_max_width(742.0);

    egui::Frame::default()
        .fill(ui.visuals().extreme_bg_color)
        .inner_margin(10.0)
        .stroke(egui::Stroke::new(
            1.0,
            ui.visuals().window_stroke.color,
        ))
        .corner_radius(15.0)
        .show(ui, |ui| {
            render_piano_keys(ui, state);

            ui.add_space(10.0);

            let total_white_keys = 43.0;
            let visible_white_keys = 15.0;
            let max_scroll = total_white_keys - visible_white_keys;

            let scrollbar_width = 720.0;
            let scrollbar_height = 16.0;

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
            painter.rect_filled(rect, 2.0, bg_color);

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

            painter.rect_filled(handle_rect, 2.0, handle_color);
            painter.rect_stroke(
                handle_rect,
                2.0,
                egui::Stroke::new(1.0, Color32::from_rgb(80, 80, 80)),
                egui::epaint::StrokeKind::Outside,
            );
        });
}

fn render_piano_keys(ui: &mut egui::Ui, state: &mut NoteState) {
    let keyboard_width = 720.0;
    let white_key_height = 70.0;

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
            let midi_note = (octave * 12) as u8 + note_offset;

            let label = if note_in_octave == 0 {
                // Use proper MIDI octave numbering (MIDI note 12 = C0, 24 = C1, etc.)
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
                let midi_note = (octave * 12) as u8 + black_note;

                if midi_note <= 72 {
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
                    key.rect.bottom() - 15.0,
                );
                painter.text(
                    text_pos,
                    egui::Align2::CENTER_CENTER,
                    label,
                    egui::FontId::proportional(10.0),
                    Color32::from_rgb(80, 80, 80),
                );
            }
        }
    });
}

fn midi_note_to_name(midi_note: u8) -> String {
    let note_names = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];
    let octave = (midi_note / 12) as i32 - 1; // Fixed: Proper MIDI octave calculation
    let note_index = (midi_note % 12) as usize;
    format!("{}{}", note_names[note_index], octave)
}

fn log_note_values(state: &NoteState) {
    let mut values = Vec::new();

    for midi_note in 0..=127 {
        let chance = state.note_chances.get(&midi_note).copied().unwrap_or(0);
        let beat = state.note_beats.get(&midi_note).copied().unwrap_or(64);
        let beat_length = state.note_beat_lengths.get(&midi_note).copied().unwrap_or(64);
        let is_root = midi_note == state.root_note;

        let has_non_default = if is_root {
            false
        } else {
            chance != 0 || beat != 64 || beat_length != 64
        };

        if is_root || has_non_default {
            let note_name = midi_note_to_name(midi_note);
            if is_root {
                values.push(format!("{}(root)", note_name));
            } else {
                let mut parts = Vec::new();
                if chance != 0 {
                    parts.push(format!("chance={}", chance));
                }
                if beat != 64 {
                    parts.push(format!("beat={}", beat));
                }
                if beat_length != 64 {
                    parts.push(format!("beat_length={}", beat_length));
                }
                values.push(format!("{}[{}]", note_name, parts.join(", ")));
            }
        }
    }

    if !values.is_empty() {
        nih_log!("Notes: {}", values.join(", "));
    }
}

fn update_shared_state(state: &NoteState, ui_state: &Arc<SharedUiState>) {
    if let Ok(mut note_pool) = ui_state.note_pool.lock() {
        // Clear existing pool
        note_pool.notes.clear();

        // Set root note
        note_pool.set_root_note(state.root_note);

        // Add all notes with their parameters
        for midi_note in 0..=127 {
            let chance = state.note_chances.get(&midi_note).copied().unwrap_or(0);
            let beat = state.note_beats.get(&midi_note).copied().unwrap_or(64);

            // Skip notes with 0 chance (except root note which is already handled)
            if midi_note != state.root_note && chance == 0 {
                continue;
            }

            // Convert chance from 0-127 to 0.0-1.0
            let chance_normalized = chance as f32 / 127.0;

            // Convert beat from 0-127 to -1.0 to 1.0 (strength bias)
            // 0 = fully weak (-1.0), 64 = neutral (0.0), 127 = fully strong (1.0)
            let strength_bias = (beat as f32 - 64.0) / 63.0;

            // Update the note in the pool
            if midi_note != state.root_note {
                note_pool.set_note(midi_note, chance_normalized, strength_bias);
            }
        }
    }
}

fn render_selected_note_info(ui: &mut egui::Ui, state: &mut NoteState) {
    egui::Frame::default()
        .fill(ui.visuals().extreme_bg_color)
        .inner_margin(10.0)
        .stroke(egui::Stroke::new(
            1.0,
            ui.visuals().window_stroke.color,
        ))
        .corner_radius(15.0)
        .show(ui, |ui| {
            ui.vertical(|ui| {
            ui.horizontal(|ui| {
                let selected_note_name = if let Some(note) = state.selected_note {
                    midi_note_to_name(note)
                } else {
                    "None".to_string()
                };

                let root_note_name = midi_note_to_name(state.root_note);

                ui.label(egui::RichText::new(format!("Selected: {}", selected_note_name)).size(12.0));
                ui.add_space(16.0);
                ui.label(egui::RichText::new(format!("Root: {}", root_note_name)).size(12.0));

                ui.add_space(16.0);

                if let Some(selected) = state.selected_note {
                    if ui.button("Set as Root").clicked() {
                        state.root_note = selected;
                    }
                }
            });

            ui.add_space(8.0);

            if let Some(selected) = state.selected_note {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Chance:").size(12.0));
                    ui.add_space(8.0);

                    let is_root = selected == state.root_note;
                    let mut chance_value = if is_root {
                        127
                    } else {
                        *state.note_chances.get(&selected).unwrap_or(&0)
                    };

                    let slider = egui::Slider::new(&mut chance_value, 0..=127)
                        .show_value(true);

                    let slider_response = if is_root {
                        ui.add_enabled(false, slider)
                    } else {
                        ui.add(slider)
                    };

                    if slider_response.changed() && !is_root {
                        state.note_chances.insert(selected, chance_value);
                    }
                });

                ui.add_space(8.0);

                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("Beat:").size(12.0));
                        ui.add_space(8.0);

                        let is_root = selected == state.root_note;
                        let mut beat_value = if is_root {
                            64
                        } else {
                            *state.note_beats.get(&selected).unwrap_or(&64)
                        };

                        let slider = egui::Slider::new(&mut beat_value, 0..=127)
                            .show_value(true);

                        let slider_response = if is_root {
                            ui.add_enabled(false, slider)
                        } else {
                            ui.add(slider)
                        };

                        if slider_response.changed() && !is_root {
                            state.note_beats.insert(selected, beat_value);
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.add_space(40.0);
                        ui.label(egui::RichText::new("Weak (0)").size(10.0).color(Color32::from_gray(150)));
                        ui.add_space(60.0);
                        ui.label(egui::RichText::new("Any (64)").size(10.0).color(Color32::from_gray(150)));
                        ui.add_space(55.0);
                        ui.label(egui::RichText::new("Strong (127)").size(10.0).color(Color32::from_gray(150)));
                    });
                });

                ui.add_space(8.0);

                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("Beat Length:").size(12.0));
                        ui.add_space(8.0);

                        let is_root = selected == state.root_note;
                        let mut beat_length_value = if is_root {
                            64
                        } else {
                            *state.note_beat_lengths.get(&selected).unwrap_or(&64)
                        };

                        let slider = egui::Slider::new(&mut beat_length_value, 0..=127)
                            .show_value(true);

                        let slider_response = if is_root {
                            ui.add_enabled(false, slider)
                        } else {
                            ui.add(slider)
                        };

                        if slider_response.changed() && !is_root {
                            state.note_beat_lengths.insert(selected, beat_length_value);
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.add_space(78.0);
                        ui.label(egui::RichText::new("Long (0)").size(10.0).color(Color32::from_gray(150)));
                        ui.add_space(60.0);
                        ui.label(egui::RichText::new("Any (64)").size(10.0).color(Color32::from_gray(150)));
                        ui.add_space(60.0);
                        ui.label(egui::RichText::new("Short (127)").size(10.0).color(Color32::from_gray(150)));
                    });
                });
            }
            });
        });
}
