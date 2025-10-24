use std::sync::Arc;
use std::collections::HashMap;
use egui_taffy::TuiBuilderLogic;
use nih_plug_egui::egui::{self, Color32};
use crate::params::DeviceParams;

#[derive(Clone, PartialEq)]
struct NoteState {
    selected_note: Option<u8>,
    root_note: u8,
    note_chances: HashMap<u8, u8>,
    note_beats: HashMap<u8, u8>,
}

impl Default for NoteState {
    fn default() -> Self {
        Self {
            selected_note: Some(24),
            root_note: 24,
            note_chances: HashMap::new(),
            note_beats: HashMap::new(),
        }
    }
}

pub fn render(
    tui: &mut egui_taffy::Tui,
    _params: &Arc<DeviceParams>,
    _setter: &nih_plug::prelude::ParamSetter,
) {
    let state_id = egui::Id::new("note_state");

    tui.ui(|ui| {
        ui.add_space(12.0);
        ui.heading(egui::RichText::new("    Note Probability").size(14.0));
        ui.add_space(8.0);

        let mut state = ui.ctx().data_mut(|d| d.get_temp::<NoteState>(state_id).unwrap_or_default());

        render_piano_container(ui, &mut state);
        render_selected_note_info(ui, &mut state);

        ui.ctx().data_mut(|d| d.insert_temp(state_id, state));
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

        let mut white_key_index = 0;
        let mut octave_number = 2;

        for octave in 0..2 {
            for (note_idx, &note) in white_key_pattern.iter().enumerate() {
                let x = start_pos.x + (white_key_index as f32 * white_key_width);
                let y = start_pos.y;

                let key_rect = egui::Rect::from_min_size(
                    egui::pos2(x, y),
                    egui::vec2(white_key_width - 1.0, white_key_height),
                );

                let response = ui.allocate_rect(key_rect, egui::Sense::click());

                let label = if note_idx == 0 {
                    Some(format!("C{}", octave_number))
                } else {
                    None
                };

                let midi_note = 24 + (octave * 12) + note;

                keys.push(KeyInfo {
                    rect: key_rect,
                    is_black: false,
                    hovered: response.hovered(),
                    clicked: response.clicked(),
                    label,
                    midi_note,
                });

                white_key_index += 1;
            }
            octave_number += 1;
        }

        let x = start_pos.x + (white_key_index as f32 * white_key_width);
        let y = start_pos.y;

        let key_rect = egui::Rect::from_min_size(
            egui::pos2(x, y),
            egui::vec2(white_key_width - 1.0, white_key_height),
        );

        let response = ui.allocate_rect(key_rect, egui::Sense::click());

        let midi_note = 24 + (2 * 12);

        keys.push(KeyInfo {
            rect: key_rect,
            is_black: false,
            hovered: response.hovered(),
            clicked: response.clicked(),
            label: Some(format!("C{}", octave_number)),
            midi_note,
        });

        white_key_index = 0;

        for octave in 0..2 {
            for i in 0..white_key_pattern.len() {
                let white_note = white_key_pattern[i];

                let has_black_key_after = black_key_pattern.iter().any(|&black| black == white_note + 1);

                if has_black_key_after {
                    let x = start_pos.x + (white_key_index as f32 * white_key_width) + white_key_width - (black_key_width / 2.0);
                    let y = start_pos.y;

                    let key_rect = egui::Rect::from_min_size(
                        egui::pos2(x, y),
                        egui::vec2(black_key_width, black_key_height),
                    );

                    let response = ui.allocate_rect(key_rect, egui::Sense::click());

                    let black_note = white_note + 1;
                    let midi_note = 24 + (octave * 12) + black_note;

                    keys.push(KeyInfo {
                        rect: key_rect,
                        is_black: true,
                        hovered: response.hovered(),
                        clicked: response.clicked(),
                        label: None,
                        midi_note,
                    });
                }

                white_key_index += 1;
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
                    egui::vec2(key.rect.width(), gray_height),
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
                    egui::pos2(key.rect.left(), key.rect.bottom() - blue_height),
                    egui::vec2(key.rect.width(), blue_height),
                );

                let blue_color = if beat_value < 64 {
                    Color32::from_rgba_unmultiplied(100, 150, 255, 120)
                } else {
                    Color32::from_rgba_unmultiplied(50, 100, 200, 120)
                };

                painter.rect_filled(blue_rect, 2.0, blue_color);
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
                    Color32::from_rgb(100, 100, 100),
                );
            }
        }
    });
}

fn midi_note_to_name(midi_note: u8) -> String {
    let note_names = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];
    let octave = (midi_note / 12) as i32;
    let note_index = (midi_note % 12) as usize;
    format!("{}{}", note_names[note_index], octave)
}

fn render_selected_note_info(ui: &mut egui::Ui, state: &mut NoteState) {
    ui.add_space(8.0);
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
            }
        });
}
