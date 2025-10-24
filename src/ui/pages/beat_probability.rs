use egui_taffy::taffy::{prelude::*, style::AlignItems};
use egui_taffy::TuiBuilderLogic;
use nih_plug::prelude::Param;
use nih_plug_egui::egui::{self, Color32};
use nih_plug_egui::egui::style::HandleShape;
use std::sync::Arc;
use crate::params::{BeatMode, DeviceParams};

const NUM_SLIDERS: usize = 4;

pub fn render(
    tui: &mut egui_taffy::Tui,
    params: &Arc<DeviceParams>,
    setter: &nih_plug::prelude::ParamSetter,
) {
    let (beat_mode, num_sliders) = tui.ui(|ui| get_beat_state(ui));

    tui.ui(|ui| {
        ui.add_space(12.0);
        ui.heading(egui::RichText::new("    Beat Probability").size(14.0));
        ui.add_space(8.0);
    });

    tui.style(Style {
        flex_grow: 1.0,
        align_items: Some(AlignItems::Stretch),
        ..Default::default()
    })
    .ui(|ui| {
        render_grid_container(ui, params, setter, beat_mode, num_sliders);
    });

    tui.ui(|ui| {
        ui.add_space(12.0);
        render_controls(ui, beat_mode, num_sliders);
    });
}

fn get_beat_state(ui: &mut egui::Ui) -> (BeatMode, usize) {
    ui.memory_mut(|mem| {
        let mode = *mem.data.get_temp_mut_or(
            egui::Id::new("beat_mode"),
            BeatMode::Straight,
        );
        let mut sliders = *mem
            .data
            .get_temp_mut_or(egui::Id::new("num_sliders"), NUM_SLIDERS);

        if !DeviceParams::is_valid_beat_count(mode, sliders) {
            sliders = DeviceParams::get_default_beat_count(mode);
            mem.data.insert_temp(egui::Id::new("num_sliders"), sliders);
        }

        (mode, sliders)
    })
}

fn render_grid_container(
    ui: &mut egui::Ui,
    params: &Arc<DeviceParams>,
    setter: &nih_plug::prelude::ParamSetter,
    beat_mode: BeatMode,
    num_sliders: usize,
) {
    let container_height = 276.0;
    ui.set_min_size(egui::vec2(742.0, container_height));
    ui.set_max_width(742.0);

    egui::Frame::default()
        .fill(ui.visuals().extreme_bg_color)
        .inner_margin(0.0)
        .stroke(egui::Stroke::new(
            1.0,
            ui.visuals().window_stroke.color,
        ))
        .corner_radius(15.0)
        .show(ui, |ui| {
            render_grid_lines(ui, beat_mode, num_sliders, container_height);
            render_occupied_space(ui, params, beat_mode, num_sliders, container_height);
            render_sliders(ui, params, setter, beat_mode, num_sliders, container_height);
        });
}

fn render_grid_lines(
    ui: &mut egui::Ui,
    beat_mode: BeatMode,
    num_sliders: usize,
    container_height: f32,
) {
    let container_rect = ui.available_rect_before_wrap();
    let painter = ui.painter();
    let container_width = 738.0;
    let grid_padding = 10.0;
    let grid_width = container_width - (grid_padding * 2.0);

    let (num_v_grid_positions, grid_spaces) = match beat_mode {
        BeatMode::Straight | BeatMode::Dotted => (33, 32.0),
        BeatMode::Triplet => (25, 24.0),
    };
    let slider_width = grid_width / grid_spaces;

    for i in 0..num_v_grid_positions {
        let x = container_rect.min.x + grid_padding + i as f32 * slider_width;
        let line_num = i + 1;

        let color = match beat_mode {
            BeatMode::Straight | BeatMode::Dotted => {
                if (line_num - 1) % 8 == 0 {
                    Color32::from_rgb(40, 40, 40)
                } else if line_num % 4 == 1 {
                    Color32::from_rgb(25, 25, 25)
                } else if line_num % 2 == 1 {
                    Color32::from_rgb(20, 20, 20)
                } else {
                    Color32::from_rgb(15, 15, 15)
                }
            }
            BeatMode::Triplet => {
                let beat_interval = 24 / num_sliders;

                if i % beat_interval == 0 {
                    Color32::from_rgb(40, 40, 40)
                } else if i % 3 == 0 {
                    Color32::from_rgb(22, 22, 22)
                } else {
                    Color32::from_rgb(15, 15, 15)
                }
            }
        };

        painter.line_segment(
            [
                egui::pos2(x, container_rect.min.y + 1.0),
                egui::pos2(x, container_rect.max.y - 1.0),
            ],
            egui::Stroke::new(1.0, color),
        );
    }

    for i in 0..5 {
        let y = container_rect.min.y + 10.0 + i as f32 * (container_height - 20.0) / 4.0;
        painter.line_segment(
            [
                egui::pos2(container_rect.min.x + 10.0, y),
                egui::pos2(container_rect.max.x - 12.5, y),
            ],
            egui::Stroke::new(1.0, Color32::from_rgb(20, 20, 20)),
        );
    }
}

fn get_division_color(mode: BeatMode, beat_count: usize, alpha: u8) -> Color32 {
    let color_by_note_value = match (mode, beat_count) {
        (BeatMode::Straight, 1) => 0,
        (BeatMode::Straight, 2) | (BeatMode::Triplet, 3) | (BeatMode::Dotted, 2) => 1,
        (BeatMode::Straight, 4) | (BeatMode::Triplet, 6) | (BeatMode::Dotted, 3) => 2,
        (BeatMode::Straight, 8) | (BeatMode::Triplet, 12) | (BeatMode::Dotted, 6) => 3,
        (BeatMode::Straight, 16) | (BeatMode::Triplet, 24) | (BeatMode::Dotted, 11) => 4,
        (BeatMode::Straight, 32) | (BeatMode::Dotted, 22) => 5,
        _ => 6,
    };

    match color_by_note_value {
        0 => Color32::from_rgba_unmultiplied(255, 100, 100, alpha),
        1 => Color32::from_rgba_unmultiplied(255, 150, 100, alpha),
        2 => Color32::from_rgba_unmultiplied(255, 255, 100, alpha),
        3 => Color32::from_rgba_unmultiplied(100, 255, 100, alpha),
        4 => Color32::from_rgba_unmultiplied(100, 100, 255, alpha),
        5 => Color32::from_rgba_unmultiplied(150, 100, 255, alpha),
        _ => Color32::from_rgba_unmultiplied(150, 150, 150, alpha),
    }
}

fn render_occupied_space(
    ui: &mut egui::Ui,
    params: &Arc<DeviceParams>,
    beat_mode: BeatMode,
    num_sliders: usize,
    _container_height: f32,
) {
    let container_rect = ui.available_rect_before_wrap();
    let painter = ui.painter();
    let container_width = 738.0;
    let grid_padding = 10.0;
    let grid_width = container_width - (grid_padding * 2.0);

    let max_height = 256.0;
    let top_y = container_rect.min.y + 10.0;

    #[derive(Clone, Debug)]
    struct Beat {
        mode: BeatMode,
        count: usize,
        index: usize,
        value: f32,
        start_time: f32,
        end_time: f32,
    }

    let mut all_beats = Vec::new();

    for mode in [BeatMode::Straight, BeatMode::Triplet, BeatMode::Dotted] {
        for (count, _) in DeviceParams::get_divisions_for_mode(mode).iter() {
            if mode == beat_mode && *count == num_sliders {
                continue;
            }

            for index in 0..*count {
                let param = params.get_division_param(mode, *count, index);
                let value = param.modulated_plain_value();

                if value > 0.0 {
                    let (start_time, end_time) = DeviceParams::get_beat_time_span(mode, *count, index);
                    all_beats.push(Beat {
                        mode,
                        count: *count,
                        index,
                        value,
                        start_time,
                        end_time,
                    });
                }
            }
        }
    }

    if all_beats.is_empty() {
        return;
    }

    let mut time_points = vec![0.0, 1.0];

    for beat in &all_beats {
        time_points.push(beat.start_time);
        time_points.push(beat.end_time);
    }

    time_points.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    time_points.dedup();

    for segment_idx in 0..time_points.len().saturating_sub(1) {
        let segment_start = time_points[segment_idx];
        let segment_end = time_points[segment_idx + 1];
        let segment_mid = (segment_start + segment_end) / 2.0;

        let mut active_beats: Vec<&Beat> = all_beats
            .iter()
            .filter(|beat| segment_mid >= beat.start_time && segment_mid < beat.end_time)
            .collect();

        if active_beats.is_empty() {
            continue;
        }

        active_beats.sort_by(|a, b| b.value.partial_cmp(&a.value).unwrap_or(std::cmp::Ordering::Equal));

        let segment_start_x = container_rect.min.x + grid_padding + (segment_start * grid_width);
        let segment_end_x = container_rect.min.x + grid_padding + (segment_end * grid_width);
        let segment_width = segment_end_x - segment_start_x;

        let mut current_y = top_y;

        for beat in active_beats {
            let height = max_height * (beat.value / 127.0);
            let color = get_division_color(beat.mode, beat.count, 10);

            painter.rect_filled(
                egui::Rect::from_min_size(
                    egui::pos2(segment_start_x, current_y),
                    egui::vec2(segment_width, height),
                ),
                0.0,
                color,
            );

            current_y += height;
        }
    }
}

fn render_sliders(
    ui: &mut egui::Ui,
    params: &Arc<DeviceParams>,
    setter: &nih_plug::prelude::ParamSetter,
    beat_mode: BeatMode,
    num_sliders: usize,
    container_height: f32,
) {
    let container_width = 738.0;
    let grid_padding = 10.0;
    let grid_width = container_width - (grid_padding * 2.0);

    ui.vertical(|ui| {
        ui.set_min_size(egui::vec2(738.0, container_height));
        ui.set_max_width(738.0);
        ui.add_space(10.0);

        ui.horizontal_top(|ui| {
            let grid_base = match beat_mode {
                BeatMode::Straight => 32.0,
                BeatMode::Triplet => 24.0,
                BeatMode::Dotted => 32.0,
            };

            for i in 0..num_sliders {
                let grid_pos = match beat_mode {
                    BeatMode::Straight => i as f32 * (grid_base / num_sliders as f32),
                    BeatMode::Triplet => i as f32 * (grid_base / num_sliders as f32),
                    BeatMode::Dotted => {
                        let dotted_duration = match num_sliders {
                            2 => 24.0,
                            3 => 12.0,
                            6 => 6.0,
                            11 => 3.0,
                            22 => 1.5,
                            _ => panic!("Invalid dotted division: {}", num_sliders),
                        };
                        i as f32 * dotted_duration
                    }
                };

                let grid_spaces = match beat_mode {
                    BeatMode::Straight | BeatMode::Dotted => 32.0,
                    BeatMode::Triplet => 24.0,
                };
                let slider_width_for_pos = grid_width / grid_spaces;
                let target_x = grid_padding + grid_pos * slider_width_for_pos;
                let current_x = ui.cursor().min.x - 24.0;
                let space_needed = target_x - current_x;

                ui.add_space(space_needed);

                ui.vertical(|ui| {
                    let param = params.get_division_param(beat_mode, num_sliders, i);
                    let available_range = params.calculate_available_range(beat_mode, num_sliders, i);
                    let mut value = param.modulated_plain_value();

                    let max_value = available_range;
                    value = value.min(max_value);

                    let slider_height = 256.0 * (max_value / 127.0);
                    let padding_top = 256.0 - slider_height;

                    ui.add_space(padding_top);

                    ui.style_mut().spacing.slider_width = slider_height;
                    ui.style_mut().spacing.slider_rail_height = 9.0;

                    if max_value > 0.0 {
                        if ui
                            .add(
                                egui::Slider::new(&mut value, 0.0..=max_value)
                                    .vertical()
                                    .trailing_fill(true)
                                    .step_by(1.0)
                                    .handle_shape(HandleShape::Rect {
                                        aspect_ratio: 0.0,
                                    })
                                    .show_value(false),
                            )
                            .changed()
                        {
                            setter.begin_set_parameter(param);
                            setter.set_parameter(param, value);
                            setter.end_set_parameter(param);
                            params.log_all_values();
                        }
                    } else {
                        ui.style_mut().spacing.slider_width = 0.0;
                        ui.add(
                            egui::Slider::new(&mut value, 0.0..=0.0)
                                .vertical()
                                .show_value(false),
                        );
                    }
                });
            }
        });
    });
}

fn render_controls(ui: &mut egui::Ui, beat_mode: BeatMode, num_sliders: usize) {
    egui::Frame::default()
        .fill(Color32::from_rgb(30, 30, 30))
        .inner_margin(8.0)
        .stroke(egui::Stroke::new(1.0, Color32::from_rgb(40, 40, 40)))
        .corner_radius(15.0)
        .show(ui, |ui| {
            ui.vertical_centered(|ui| {
                render_division_buttons(ui, beat_mode, num_sliders);
                ui.add_space(5.0);
                render_mode_buttons(ui, beat_mode);
            });
        });
}

fn render_division_buttons(ui: &mut egui::Ui, beat_mode: BeatMode, num_sliders: usize) {
    ui.horizontal(|ui| {
        ui.add_space(5.0);
        let divisions = DeviceParams::get_divisions_for_mode(beat_mode);
        let mode_suffix = beat_mode.as_str();

        match beat_mode {
            BeatMode::Straight => {}
            BeatMode::Triplet | BeatMode::Dotted => {
                ui.add_enabled(false, egui::Button::new("").min_size(egui::vec2(60.0, 32.0)));
            }
        }

        for (count, label) in divisions.iter() {
            let button_label = if beat_mode == BeatMode::Straight {
                label.to_string()
            } else {
                format!("{}{}", label, mode_suffix)
            };

            let button = egui::Button::new(egui::RichText::new(button_label).size(14.0))
                .min_size(egui::vec2(60.0, 32.0))
                .selected(num_sliders == *count);

            let response = ui.add(button);

            let color = get_division_color(beat_mode, *count, 127);
            let opaque_color = Color32::from_rgb(color.r(), color.g(), color.b());
            let button_rect = response.rect;
            let circle_center = egui::pos2(
                button_rect.right() - 6.0,
                button_rect.bottom() - 6.0
            );
            ui.painter().circle_filled(circle_center, 3.5, opaque_color);

            if response.clicked() {
                ui.memory_mut(|mem| {
                    mem.data.insert_temp(egui::Id::new("num_sliders"), *count);
                });
            }
        }

        if beat_mode == BeatMode::Triplet {
            ui.add_enabled(false, egui::Button::new("").min_size(egui::vec2(60.0, 32.0)));
        }
        ui.add_space(5.0);
    });
}

fn render_mode_buttons(ui: &mut egui::Ui, beat_mode: BeatMode) {
    ui.horizontal(|ui| {
        ui.add_space(102.0);

        let button_s = egui::Button::new(egui::RichText::new("S").size(14.0))
            .min_size(egui::vec2(60.0, 32.0))
            .selected(beat_mode == BeatMode::Straight);

        if ui.add(button_s).clicked() && beat_mode != BeatMode::Straight {
            ui.memory_mut(|mem| {
                mem.data.insert_temp(egui::Id::new("beat_mode"), BeatMode::Straight);
                mem.data.insert_temp(egui::Id::new("num_sliders"), 4);
            });
        }

        let button_t = egui::Button::new(egui::RichText::new("T").size(14.0))
            .min_size(egui::vec2(60.0, 32.0))
            .selected(beat_mode == BeatMode::Triplet);

        if ui.add(button_t).clicked() && beat_mode != BeatMode::Triplet {
            ui.memory_mut(|mem| {
                mem.data.insert_temp(egui::Id::new("beat_mode"), BeatMode::Triplet);
                mem.data.insert_temp(egui::Id::new("num_sliders"), 6);
            });
        }

        let button_d = egui::Button::new(egui::RichText::new("D").size(14.0))
            .min_size(egui::vec2(60.0, 32.0))
            .selected(beat_mode == BeatMode::Dotted);

        if ui.add(button_d).clicked() && beat_mode != BeatMode::Dotted {
            ui.memory_mut(|mem| {
                mem.data.insert_temp(egui::Id::new("beat_mode"), BeatMode::Dotted);
                mem.data.insert_temp(egui::Id::new("num_sliders"), 2);
            });
        }
    });
}
