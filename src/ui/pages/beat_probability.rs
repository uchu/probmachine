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
    let (beat_mode, num_sliders) = tui.ui(get_beat_state);

    tui.ui(|ui| {
        ui.add_space(16.0);
        ui.horizontal(|ui| {
            ui.heading(egui::RichText::new("    Beat Probability").size(22.0));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.add_space(166.0);
                let clear_button = egui::Button::new(egui::RichText::new("Clear All").size(14.0))
                    .min_size(egui::vec2(80.0, 28.0));
                if ui.add(clear_button).clicked() {
                    for mode in [BeatMode::Straight, BeatMode::Triplet, BeatMode::Dotted] {
                        for (count, _) in DeviceParams::get_divisions_for_mode(mode).iter() {
                            for index in 0..*count {
                                let param = params.get_division_param(mode, *count, index);
                                setter.begin_set_parameter(param);
                                setter.set_parameter(param, 0.0);
                                setter.end_set_parameter(param);
                            }
                        }
                    }
                }
            });
        });
        ui.add_space(12.0);
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
        ui.add_space(16.0);
        render_controls(ui, params, setter, beat_mode, num_sliders);
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
    let container_height = 420.0;
    ui.set_min_size(egui::vec2(1220.0, container_height));
    ui.set_max_width(1220.0);

    let swing = params.swing_amount.modulated_plain_value();

    egui::Frame::default()
        .fill(ui.visuals().extreme_bg_color)
        .inner_margin(0.0)
        .stroke(egui::Stroke::new(
            1.0,
            ui.visuals().window_stroke.color,
        ))
        .corner_radius(15.0)
        .show(ui, |ui| {
            render_grid_lines(ui, beat_mode, num_sliders, container_height, swing);
            render_occupied_space(ui, params, beat_mode, num_sliders, container_height);
            render_sliders(ui, params, setter, beat_mode, num_sliders, container_height, swing);
        });
}

fn render_grid_lines(
    ui: &mut egui::Ui,
    beat_mode: BeatMode,
    num_sliders: usize,
    container_height: f32,
    swing: f32,
) {
    let container_rect = ui.available_rect_before_wrap();
    let painter = ui.painter();
    let container_width = 1216.0;
    let grid_padding = 16.0;
    let grid_width = container_width - (grid_padding * 2.0);

    let (num_v_grid_positions, grid_spaces) = match beat_mode {
        BeatMode::Straight | BeatMode::Dotted => (33, 32.0),
        BeatMode::Triplet => (25, 24.0),
    };

    for i in 0..num_v_grid_positions {
        let normalized_pos = i as f32 / grid_spaces;
        let swung_pos = DeviceParams::apply_swing(normalized_pos, swing);
        let x = container_rect.min.x + grid_padding + swung_pos * grid_width;
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
        let y = container_rect.min.y + 16.0 + i as f32 * (container_height - 32.0) / 4.0;
        painter.line_segment(
            [
                egui::pos2(container_rect.min.x + 16.0, y),
                egui::pos2(container_rect.max.x - 20.0, y),
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
    let container_width = 1216.0;
    let grid_padding = 16.0;
    let grid_width = container_width - (grid_padding * 2.0);

    let max_height = 388.0;
    let top_y = container_rect.min.y + 16.0;

    #[derive(Clone, Debug)]
    struct Beat {
        mode: BeatMode,
        count: usize,
        #[allow(dead_code)]
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

        active_beats.sort_by(|a, b| a.count.cmp(&b.count));

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
    swing: f32,
) {
    let container_width = 1216.0;
    let grid_padding = 16.0;
    let grid_width = container_width - (grid_padding * 2.0);

    ui.vertical(|ui| {
        ui.set_min_size(egui::vec2(1216.0, container_height));
        ui.set_max_width(1216.0);
        ui.add_space(16.0);

        ui.horizontal_top(|ui| {
            let grid_base = match beat_mode {
                BeatMode::Straight => 32.0,
                BeatMode::Triplet => 24.0,
                BeatMode::Dotted => 32.0,
            };

            for i in 0..num_sliders {
                let normalized_pos = match beat_mode {
                    BeatMode::Straight => i as f32 / num_sliders as f32,
                    BeatMode::Triplet => i as f32 / num_sliders as f32,
                    BeatMode::Dotted => {
                        let dotted_duration = match num_sliders {
                            2 => 24.0,
                            3 => 12.0,
                            6 => 6.0,
                            11 => 3.0,
                            22 => 1.5,
                            _ => panic!("Invalid dotted division: {}", num_sliders),
                        };
                        (i as f32 * dotted_duration) / 32.0
                    }
                };

                let swung_pos = DeviceParams::apply_swing(normalized_pos, swing);
                let grid_pos = swung_pos * grid_base;

                let grid_spaces = match beat_mode {
                    BeatMode::Straight | BeatMode::Dotted => 32.0,
                    BeatMode::Triplet => 24.0,
                };
                let slider_width_for_pos = grid_width / grid_spaces;
                let target_x = grid_padding + grid_pos * slider_width_for_pos;
                let current_x = ui.cursor().min.x - 31.0;
                let space_needed = target_x - current_x;

                ui.add_space(space_needed);

                let slider_response = ui.vertical(|ui| {
                    let param = params.get_division_param(beat_mode, num_sliders, i);
                    let available_range = params.calculate_available_range(beat_mode, num_sliders, i);
                    let mut value = param.modulated_plain_value();

                    let max_value = available_range;
                    value = value.min(max_value);

                    let slider_height = 388.0 * (max_value / 127.0);
                    let padding_top = 388.0 - slider_height;

                    ui.add_space(padding_top);

                    ui.style_mut().spacing.slider_width = slider_height;
                    ui.style_mut().spacing.slider_rail_height = 16.0;

                    let division_color = get_division_color(beat_mode, num_sliders, 127);
                    ui.style_mut().visuals.selection.bg_fill = division_color;

                    let mut slider_response_opt = None;

                    if max_value > 0.0 {
                        let response = ui.add(
                            egui::Slider::new(&mut value, 0.0..=max_value)
                                .vertical()
                                .trailing_fill(true)
                                .step_by(1.0)
                                .handle_shape(HandleShape::Rect {
                                    aspect_ratio: 0.0,
                                })
                                .show_value(false),
                        );

                        slider_response_opt = Some(response.clone());

                        if response.changed() {
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

                    slider_response_opt
                }).inner;

                if let Some(slider_resp) = slider_response {
                    if slider_resp.dragged() {
                        let (beat_start, beat_end) = DeviceParams::get_beat_time_span(beat_mode, num_sliders, i);
                        let slider_center_x = slider_resp.rect.center().x;

                        let param = params.get_division_param(beat_mode, num_sliders, i);
                        let current_value = param.modulated_plain_value();
                        let available_range = params.calculate_available_range(beat_mode, num_sliders, i);
                        let clamped_value = current_value.min(available_range);

                        let slider_height = slider_resp.rect.height();
                        let value_ratio = if available_range > 0.0 { clamped_value / available_range } else { 0.0 };
                        let handle_y = slider_resp.rect.bottom() - (value_ratio * slider_height);

                        let grid_spaces = match beat_mode {
                            BeatMode::Straight | BeatMode::Dotted => 32.0,
                            BeatMode::Triplet => 24.0,
                        };
                        let slider_width_for_pos = grid_width / grid_spaces;

                        let line_start_x = slider_center_x;
                        let line_end_time = beat_end.min(1.0);
                        let line_end_grid_pos = line_end_time * grid_spaces;
                        let container_left = slider_center_x - (beat_start * grid_spaces * slider_width_for_pos);
                        let line_end_x = container_left + (line_end_grid_pos * slider_width_for_pos);

                        let painter = ui.painter();

                        painter.line_segment(
                            [
                                egui::pos2(line_start_x, handle_y),
                                egui::pos2(line_end_x, handle_y),
                            ],
                            egui::Stroke::new(1.5, Color32::from_rgb(40, 40, 40)),
                        );
                    }
                }
            }
        });
    });
}

fn render_controls(ui: &mut egui::Ui, params: &Arc<DeviceParams>, setter: &nih_plug::prelude::ParamSetter, beat_mode: BeatMode, num_sliders: usize) {
    ui.horizontal(|ui| {
        egui::Frame::default()
            .fill(Color32::from_rgb(30, 30, 30))
            .inner_margin(12.0)
            .stroke(egui::Stroke::new(1.0, Color32::from_rgb(40, 40, 40)))
            .corner_radius(15.0)
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    render_division_buttons(ui, params, beat_mode, num_sliders);
                    ui.add_space(8.0);
                    render_mode_buttons(ui, params, beat_mode);
                });
            });

        ui.add_space(16.0);

        egui::Frame::default()
            .fill(Color32::from_rgb(30, 30, 30))
            .inner_margin(12.0)
            .stroke(egui::Stroke::new(1.0, Color32::from_rgb(40, 40, 40)))
            .corner_radius(15.0)
            .show(ui, |ui| {
                render_timing_controls(ui, params, setter);
            });
    });
}

fn render_timing_controls(ui: &mut egui::Ui, params: &Arc<DeviceParams>, setter: &nih_plug::prelude::ParamSetter) {
    ui.vertical(|ui| {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Length:").size(18.0));
            ui.add_space(8.0);

            let mut length = params.note_length_percent.modulated_plain_value();
            ui.style_mut().spacing.slider_width = 140.0;
            ui.style_mut().spacing.slider_rail_height = 10.0;
            let response = ui.add(
                egui::Slider::new(&mut length, 1.0..=200.0)
                    .fixed_decimals(0)
                    .clamping(egui::SliderClamping::Always)
                    .show_value(false)
            );
            if response.changed() {
                setter.set_parameter(&params.note_length_percent, length);
            }

            ui.add_space(12.0);
            let mut length_edit = length;
            ui.style_mut().spacing.interact_size.y = 32.0;
            let edit_response = ui.add_sized(
                egui::vec2(70.0, 32.0),
                egui::DragValue::new(&mut length_edit)
                    .range(1.0..=200.0)
                    .speed(1.0)
                    .suffix("%")
                    .min_decimals(0)
                    .max_decimals(0)
            );
            if edit_response.changed() {
                setter.set_parameter(&params.note_length_percent, length_edit);
            }
        });

        ui.add_space(16.0);

        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Swing:").size(18.0));
            ui.add_space(8.0);

            let mut swing = params.swing_amount.modulated_plain_value();
            ui.style_mut().spacing.slider_width = 140.0;
            ui.style_mut().spacing.slider_rail_height = 10.0;
            let response = ui.add(
                egui::Slider::new(&mut swing, 50.0..=75.0)
                    .fixed_decimals(0)
                    .clamping(egui::SliderClamping::Always)
                    .show_value(false)
            );
            if response.changed() {
                setter.set_parameter(&params.swing_amount, swing);
            }

            ui.add_space(12.0);
            let mut swing_edit = swing;
            ui.style_mut().spacing.interact_size.y = 32.0;
            let edit_response = ui.add_sized(
                egui::vec2(70.0, 32.0),
                egui::DragValue::new(&mut swing_edit)
                    .range(50.0..=75.0)
                    .speed(0.5)
                    .suffix("%")
                    .min_decimals(0)
                    .max_decimals(0)
            );
            if edit_response.changed() {
                setter.set_parameter(&params.swing_amount, swing_edit);
            }
        });

        ui.add_space(16.0);

        ui.horizontal(|ui| {
            let legato_on = params.legato_mode.value();
            let button = egui::Button::new(egui::RichText::new("Legato").size(18.0))
                .min_size(egui::vec2(80.0, 32.0))
                .selected(legato_on);
            if ui.add(button).clicked() {
                setter.set_parameter(&params.legato_mode, !legato_on);
            }

            ui.add_space(16.0);

            if legato_on {
                ui.label(egui::RichText::new("Time:").size(18.0));
                ui.add_space(8.0);

                let mut time = params.legato_time.modulated_plain_value();
                ui.style_mut().spacing.slider_width = 100.0;
                ui.style_mut().spacing.slider_rail_height = 10.0;
                let response = ui.add(
                    egui::Slider::new(&mut time, 1.0..=500.0)
                        .fixed_decimals(0)
                        .clamping(egui::SliderClamping::Always)
                        .show_value(false)
                );
                if response.changed() {
                    setter.set_parameter(&params.legato_time, time);
                }

                ui.add_space(8.0);
                let mut time_edit = time;
                ui.style_mut().spacing.interact_size.y = 32.0;
                let edit_response = ui.add_sized(
                    egui::vec2(70.0, 32.0),
                    egui::DragValue::new(&mut time_edit)
                        .range(1.0..=500.0)
                        .speed(1.0)
                        .suffix(" ms")
                        .min_decimals(0)
                        .max_decimals(0)
                );
                if edit_response.changed() {
                    setter.set_parameter(&params.legato_time, time_edit);
                }
            }
        });
    });
}

fn render_division_buttons(ui: &mut egui::Ui, params: &Arc<DeviceParams>, beat_mode: BeatMode, num_sliders: usize) {
    ui.horizontal(|ui| {
        ui.add_space(8.0);
        let divisions = DeviceParams::get_divisions_for_mode(beat_mode);
        let mode_suffix = beat_mode.as_str();

        match beat_mode {
            BeatMode::Straight => {}
            BeatMode::Triplet | BeatMode::Dotted => {
                ui.add_enabled(false, egui::Button::new("").min_size(egui::vec2(96.0, 48.0)));
            }
        }

        for (count, label) in divisions.iter() {
            let button_label = if beat_mode == BeatMode::Straight {
                label.to_string()
            } else {
                format!("{}{}", label, mode_suffix)
            };

            let button = egui::Button::new(egui::RichText::new(button_label).size(20.0))
                .min_size(egui::vec2(96.0, 48.0))
                .selected(num_sliders == *count);

            let response = ui.add(button);

            let has_values = (0..*count).any(|i| {
                let param = params.get_division_param(beat_mode, *count, i);
                param.modulated_plain_value() > 0.0
            });

            if has_values {
                let color = get_division_color(beat_mode, *count, 127);
                let opaque_color = Color32::from_rgb(color.r(), color.g(), color.b());
                let button_rect = response.rect;
                let circle_center = egui::pos2(
                    button_rect.right() - 10.0,
                    button_rect.bottom() - 10.0
                );
                ui.painter().circle_filled(circle_center, 5.0, opaque_color);
            }

            if response.clicked() {
                ui.memory_mut(|mem| {
                    mem.data.insert_temp(egui::Id::new("num_sliders"), *count);
                });
            }
        }

        if beat_mode == BeatMode::Triplet {
            ui.add_enabled(false, egui::Button::new("").min_size(egui::vec2(96.0, 48.0)));
        }
        ui.add_space(8.0);
    });
}

fn render_mode_buttons(ui: &mut egui::Ui, params: &Arc<DeviceParams>, beat_mode: BeatMode) {
    let mode_has_values = |mode: BeatMode| -> bool {
        DeviceParams::get_divisions_for_mode(mode)
            .iter()
            .any(|(count, _)| {
                (0..*count).any(|i| {
                    let param = params.get_division_param(mode, *count, i);
                    param.modulated_plain_value() > 0.0
                })
            })
    };

    ui.horizontal(|ui| {
        ui.add_space(8.0);

        let button_s = egui::Button::new(egui::RichText::new("S").size(20.0))
            .min_size(egui::vec2(96.0, 48.0))
            .selected(beat_mode == BeatMode::Straight);

        let response_s = ui.add(button_s);

        if mode_has_values(BeatMode::Straight) {
            let circle_color = Color32::from_rgba_unmultiplied(255, 255, 255, 127);
            let button_rect = response_s.rect;
            let circle_center = egui::pos2(
                button_rect.right() - 10.0,
                button_rect.bottom() - 10.0
            );
            ui.painter().circle_filled(circle_center, 5.0, circle_color);
        }

        if response_s.clicked() && beat_mode != BeatMode::Straight {
            ui.memory_mut(|mem| {
                mem.data.insert_temp(egui::Id::new("beat_mode"), BeatMode::Straight);
                mem.data.insert_temp(egui::Id::new("num_sliders"), 4);
            });
        }

        let button_t = egui::Button::new(egui::RichText::new("T").size(20.0))
            .min_size(egui::vec2(96.0, 48.0))
            .selected(beat_mode == BeatMode::Triplet);

        let response_t = ui.add(button_t);

        if mode_has_values(BeatMode::Triplet) {
            let circle_color = Color32::from_rgba_unmultiplied(255, 255, 255, 127);
            let button_rect = response_t.rect;
            let circle_center = egui::pos2(
                button_rect.right() - 10.0,
                button_rect.bottom() - 10.0
            );
            ui.painter().circle_filled(circle_center, 5.0, circle_color);
        }

        if response_t.clicked() && beat_mode != BeatMode::Triplet {
            ui.memory_mut(|mem| {
                mem.data.insert_temp(egui::Id::new("beat_mode"), BeatMode::Triplet);
                mem.data.insert_temp(egui::Id::new("num_sliders"), 6);
            });
        }

        let button_d = egui::Button::new(egui::RichText::new("D").size(20.0))
            .min_size(egui::vec2(96.0, 48.0))
            .selected(beat_mode == BeatMode::Dotted);

        let response_d = ui.add(button_d);

        if mode_has_values(BeatMode::Dotted) {
            let circle_color = Color32::from_rgba_unmultiplied(255, 255, 255, 127);
            let button_rect = response_d.rect;
            let circle_center = egui::pos2(
                button_rect.right() - 10.0,
                button_rect.bottom() - 10.0
            );
            ui.painter().circle_filled(circle_center, 5.0, circle_color);
        }

        if response_d.clicked() && beat_mode != BeatMode::Dotted {
            ui.memory_mut(|mem| {
                mem.data.insert_temp(egui::Id::new("beat_mode"), BeatMode::Dotted);
                mem.data.insert_temp(egui::Id::new("num_sliders"), 2);
            });
        }
    });
}

