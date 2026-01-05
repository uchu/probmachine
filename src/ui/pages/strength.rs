use std::sync::Arc;
use nih_plug_egui::egui::{self, Color32};
use nih_plug_egui::egui::style::HandleShape;
use egui_taffy::TuiBuilderLogic;
use egui_taffy::taffy::{prelude::*, style::AlignItems};
use crate::params::DeviceParams;
use crate::ui::SharedUiState;

#[derive(Clone, Copy, PartialEq)]
enum BeatStrengthMode {
    Straight,
    Triplet,
}

#[derive(Clone, PartialEq)]
struct StrengthState {
    beat_strength_mode: BeatStrengthMode,
    beat_strength_values: [u8; 96], // LCM of 32 and 24 = 96
    last_preset_version: u64,
}

impl Default for StrengthState {
    fn default() -> Self {
        let mut values = [0u8; 96];
        // Downbeat
        values[0] = 64;
        // Quarter notes (every 24/96)
        values[24] = 48;
        values[48] = 48;
        values[72] = 48;
        // Eighth notes (every 12/96)
        values[12] = 32;
        values[36] = 32;
        values[60] = 32;
        values[84] = 32;

        Self {
            beat_strength_mode: BeatStrengthMode::Straight,
            beat_strength_values: values,
            last_preset_version: 0,
        }
    }
}

fn sync_state_from_shared(state: &mut StrengthState, ui_state: &Arc<SharedUiState>) {
    if let Ok(strength_values) = ui_state.strength_values.lock() {
        for i in 0..96 {
            state.beat_strength_values[i] = (strength_values[i] * 127.0) as u8;
        }
    }
}

// Helper function to map S position to 96-grid
fn straight_to_grid(pos: usize) -> usize {
    pos * 3 // 32 * 3 = 96
}

// Helper function to map T position to 96-grid
fn triplet_to_grid(pos: usize) -> usize {
    pos * 4 // 24 * 4 = 96
}

pub fn render(
    tui: &mut egui_taffy::Tui,
    _params: &Arc<DeviceParams>,
    _setter: &nih_plug::prelude::ParamSetter,
    ui_state: &Arc<SharedUiState>,
) {
    let state_id = egui::Id::new("strength_state");
    let current_version = ui_state.get_preset_version();

    tui.ui(|ui| {
        ui.add_space(12.0);
        ui.heading(egui::RichText::new("    Strength").size(14.0));
        ui.add_space(8.0);
    });

    tui.style(Style {
        flex_grow: 1.0,
        align_items: Some(AlignItems::Stretch),
        ..Default::default()
    })
    .ui(|ui| {
        let mut state = ui.ctx().data_mut(|d| d.get_temp::<StrengthState>(state_id).unwrap_or_default());

        if state.last_preset_version != current_version {
            sync_state_from_shared(&mut state, ui_state);
            state.last_preset_version = current_version;
            ui.ctx().data_mut(|d| d.insert_temp(state_id, state.clone()));
        }

        let state_before = state.clone();

        render_beat_strength(ui, &mut state);

        if state != state_before {
            // Update shared state with normalized strength values (0.0 to 1.0)
            if let Ok(mut strength_values) = ui_state.strength_values.lock() {
                for i in 0..96 {
                    strength_values[i] = state.beat_strength_values[i] as f32 / 127.0;
                }
            }
            ui.ctx().data_mut(|d| d.insert_temp(state_id, state));
            ui.ctx().request_repaint_after(std::time::Duration::from_millis(16));
        }
    });

    tui.ui(|ui| {
        ui.add_space(12.0);
        let mut state = ui.ctx().data_mut(|d| d.get_temp::<StrengthState>(state_id).unwrap_or_default());
        let state_before = state.clone();

        render_mode_buttons(ui, &mut state);

        if state != state_before {
            ui.ctx().data_mut(|d| d.insert_temp(state_id, state));
        }
    });
}

fn render_beat_strength(ui: &mut egui::Ui, state: &mut StrengthState) {
    render_beat_strength_grid(ui, state);
}

fn render_mode_buttons(ui: &mut egui::Ui, state: &mut StrengthState) {
    egui::Frame::default()
        .fill(Color32::from_rgb(30, 30, 30))
        .inner_margin(8.0)
        .stroke(egui::Stroke::new(1.0, Color32::from_rgb(40, 40, 40)))
        .corner_radius(15.0)
        .show(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.horizontal(|ui| {
                    ui.add_space(102.0);

                    let button_s = egui::Button::new(egui::RichText::new("S").size(14.0))
                        .min_size(egui::vec2(60.0, 32.0))
                        .selected(matches!(state.beat_strength_mode, BeatStrengthMode::Straight));

                    if ui.add(button_s).clicked() {
                        state.beat_strength_mode = BeatStrengthMode::Straight;
                    }

                    let button_t = egui::Button::new(egui::RichText::new("T").size(14.0))
                        .min_size(egui::vec2(60.0, 32.0))
                        .selected(matches!(state.beat_strength_mode, BeatStrengthMode::Triplet));

                    if ui.add(button_t).clicked() {
                        state.beat_strength_mode = BeatStrengthMode::Triplet;
                    }
                });
            });
        });
}

fn render_beat_strength_grid(ui: &mut egui::Ui, state: &mut StrengthState) {
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
            let num_sliders = match state.beat_strength_mode {
                BeatStrengthMode::Straight => 32,
                BeatStrengthMode::Triplet => 24,
            };

            render_beat_strength_grid_lines(ui, state.beat_strength_mode, num_sliders, container_height);
            render_opposite_mode_lines(ui, state, container_height);
            render_beat_strength_sliders(ui, state, num_sliders, container_height);
        });
}

fn render_opposite_mode_lines(ui: &mut egui::Ui, state: &StrengthState, _container_height: f32) {
    let container_rect = ui.available_rect_before_wrap();
    let painter = ui.painter();
    let container_width = 738.0;
    let grid_padding = 10.0;
    let grid_width = container_width - (grid_padding * 2.0);
    let max_height = 256.0;
    let top_y = container_rect.min.y + 10.0;

    match state.beat_strength_mode {
        BeatStrengthMode::Straight => {
            // Show Triplet positions in Straight mode
            let straight_grid_spaces = 32.0;

            for triplet_pos in 0..24 {
                let grid_pos = triplet_to_grid(triplet_pos);

                // Check if this grid position is also used by a straight position
                let coincides = grid_pos.is_multiple_of(3) && (grid_pos / 3) < 32;

                if !coincides {
                    let value = state.beat_strength_values[grid_pos];
                    if value > 0 {
                        let height = max_height * (value as f32 / 64.0);
                        let time_pos = triplet_pos as f32 / 24.0;
                        let straight_grid_pos = time_pos * straight_grid_spaces;
                        let x = container_rect.min.x + grid_padding + straight_grid_pos * (grid_width / straight_grid_spaces);

                        painter.line_segment(
                            [
                                egui::pos2(x, top_y + max_height - height),
                                egui::pos2(x, top_y + max_height),
                            ],
                            egui::Stroke::new(2.0, Color32::from_rgba_unmultiplied(200, 100, 100, 150)),
                        );
                    }
                }
            }
        }
        BeatStrengthMode::Triplet => {
            // Show Straight positions in Triplet mode
            let triplet_grid_spaces = 24.0;

            for straight_pos in 0..32 {
                let grid_pos = straight_to_grid(straight_pos);

                // Check if this grid position is also used by a triplet position
                let coincides = grid_pos.is_multiple_of(4) && (grid_pos / 4) < 24;

                if !coincides {
                    let value = state.beat_strength_values[grid_pos];
                    if value > 0 {
                        let height = max_height * (value as f32 / 64.0);
                        let time_pos = straight_pos as f32 / 32.0;
                        let triplet_grid_pos = time_pos * triplet_grid_spaces;
                        let x = container_rect.min.x + grid_padding + triplet_grid_pos * (grid_width / triplet_grid_spaces);

                        painter.line_segment(
                            [
                                egui::pos2(x, top_y + max_height - height),
                                egui::pos2(x, top_y + max_height),
                            ],
                            egui::Stroke::new(2.0, Color32::from_rgba_unmultiplied(200, 100, 100, 150)),
                        );
                    }
                }
            }
        }
    }
}

fn render_beat_strength_grid_lines(ui: &mut egui::Ui, mode: BeatStrengthMode, num_sliders: usize, container_height: f32) {
    let container_rect = ui.available_rect_before_wrap();
    let painter = ui.painter();
    let container_width = 738.0;
    let grid_padding = 10.0;
    let grid_width = container_width - (grid_padding * 2.0);

    let (num_v_grid_positions, grid_spaces) = match mode {
        BeatStrengthMode::Straight => (33, 32.0),
        BeatStrengthMode::Triplet => (25, 24.0),
    };
    let slider_width = grid_width / grid_spaces;

    for i in 0..num_v_grid_positions {
        let x = container_rect.min.x + grid_padding + i as f32 * slider_width;
        let line_num = i + 1;

        let color = match mode {
            BeatStrengthMode::Straight => {
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
            BeatStrengthMode::Triplet => {
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

fn render_beat_strength_sliders(ui: &mut egui::Ui, state: &mut StrengthState, num_sliders: usize, container_height: f32) {
    let container_width = 738.0;
    let grid_padding = 10.0;
    let grid_width = container_width - (grid_padding * 2.0);

    ui.vertical(|ui| {
        ui.set_min_size(egui::vec2(738.0, container_height));
        ui.set_max_width(738.0);
        ui.add_space(10.0);

        ui.horizontal_top(|ui| {
            let grid_base = match state.beat_strength_mode {
                BeatStrengthMode::Straight => 32.0,
                BeatStrengthMode::Triplet => 24.0,
            };

            for i in 0..num_sliders {
                let grid_pos = i as f32;
                let slider_width_for_pos = grid_width / grid_base;
                let target_x = grid_padding + grid_pos * slider_width_for_pos;
                let current_x = ui.cursor().min.x - 24.0;
                let space_needed = target_x - current_x;

                ui.add_space(space_needed);

                ui.vertical(|ui| {
                    let grid_pos = match state.beat_strength_mode {
                        BeatStrengthMode::Straight => straight_to_grid(i),
                        BeatStrengthMode::Triplet => triplet_to_grid(i),
                    };
                    let value = &mut state.beat_strength_values[grid_pos];

                    let slider_height = 256.0;

                    ui.style_mut().spacing.slider_width = slider_height;
                    ui.style_mut().spacing.slider_rail_height = 9.0;
                    ui.style_mut().visuals.selection.bg_fill = Color32::from_rgb(100, 150, 200);

                    ui.add(
                        egui::Slider::new(value, 0..=64)
                            .vertical()
                            .trailing_fill(true)
                            .handle_shape(HandleShape::Rect {
                                aspect_ratio: 0.0,
                            })
                            .show_value(false),
                    );
                });
            }
        });
    });
}
