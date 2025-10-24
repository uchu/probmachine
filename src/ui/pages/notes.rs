use std::sync::Arc;
use egui_taffy::TuiBuilderLogic;
use nih_plug_egui::egui::{self, Color32};
use crate::params::DeviceParams;

pub fn render(
    tui: &mut egui_taffy::Tui,
    _params: &Arc<DeviceParams>,
    _setter: &nih_plug::prelude::ParamSetter,
) {
    tui.ui(|ui| {
        ui.add_space(12.0);
        ui.heading(egui::RichText::new("    Note Probability").size(14.0));
        ui.add_space(8.0);
    });

    tui.ui(|ui| {
        render_piano_container(ui);
    });
}

fn render_piano_container(ui: &mut egui::Ui) {
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
            render_piano_keys(ui);
        });
}

fn render_piano_keys(ui: &mut egui::Ui) {
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
        label: Option<String>,
    }

    ui.horizontal(|ui| {
        let mut keys = Vec::new();
        let start_pos = ui.cursor().min;

        let mut white_key_index = 0;
        let mut octave_number = 2;

        for _octave in 0..2 {
            for (note_idx, &_note) in white_key_pattern.iter().enumerate() {
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

                keys.push(KeyInfo {
                    rect: key_rect,
                    is_black: false,
                    hovered: response.hovered(),
                    label,
                });

                if response.clicked() {
                    // Placeholder for future logic
                }

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

        keys.push(KeyInfo {
            rect: key_rect,
            is_black: false,
            hovered: response.hovered(),
            label: Some(format!("C{}", octave_number)),
        });

        if response.clicked() {
            // Placeholder for future logic
        }

        white_key_index = 0;

        for _octave in 0..2 {
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

                    keys.push(KeyInfo {
                        rect: key_rect,
                        is_black: true,
                        hovered: response.hovered(),
                        label: None,
                    });

                    if response.clicked() {
                        // Placeholder for future logic
                    }
                }

                white_key_index += 1;
            }
        }

        let painter = ui.painter();

        for key in &keys {
            let (fill_color, stroke_color) = if key.is_black {
                let fill = if key.hovered {
                    Color32::from_rgb(60, 60, 60)
                } else {
                    Color32::from_rgb(30, 30, 30)
                };
                (fill, Color32::from_rgb(10, 10, 10))
            } else {
                let fill = if key.hovered {
                    Color32::from_rgb(220, 220, 220)
                } else {
                    Color32::from_rgb(255, 255, 255)
                };
                (fill, Color32::from_rgb(100, 100, 100))
            };

            painter.rect_filled(key.rect, 2.0, fill_color);
            painter.rect_stroke(
                key.rect,
                2.0,
                egui::Stroke::new(1.0, stroke_color),
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
