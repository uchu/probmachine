use std::sync::Arc;
use nih_plug::prelude::Param;
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

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum StrengthStyle {
    FourFourStandard,
    Backbeat,
    Offbeat,
    TripletFeel,
    Shuffle,
    Sparse,
    Dense,
    Polyrhythm34,
    African,
    Reggae,
    Latin,
    Funk,
    Jazz,
    Ambient,
    Driving,
}

impl StrengthStyle {
    pub fn all() -> &'static [StrengthStyle] {
        &[
            StrengthStyle::FourFourStandard,
            StrengthStyle::Backbeat,
            StrengthStyle::Offbeat,
            StrengthStyle::TripletFeel,
            StrengthStyle::Shuffle,
            StrengthStyle::Sparse,
            StrengthStyle::Dense,
            StrengthStyle::Polyrhythm34,
            StrengthStyle::African,
            StrengthStyle::Reggae,
            StrengthStyle::Latin,
            StrengthStyle::Funk,
            StrengthStyle::Jazz,
            StrengthStyle::Ambient,
            StrengthStyle::Driving,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            StrengthStyle::FourFourStandard => "4/4 Standard",
            StrengthStyle::Backbeat => "Backbeat",
            StrengthStyle::Offbeat => "Offbeat",
            StrengthStyle::TripletFeel => "Triplet Feel",
            StrengthStyle::Shuffle => "Shuffle",
            StrengthStyle::Sparse => "Sparse",
            StrengthStyle::Dense => "Dense",
            StrengthStyle::Polyrhythm34 => "Polyrhythm 3:4",
            StrengthStyle::African => "African",
            StrengthStyle::Reggae => "Reggae",
            StrengthStyle::Latin => "Latin",
            StrengthStyle::Funk => "Funk",
            StrengthStyle::Jazz => "Jazz",
            StrengthStyle::Ambient => "Ambient",
            StrengthStyle::Driving => "Driving",
        }
    }

    #[allow(clippy::needless_range_loop)]
    pub fn generate_pattern(&self) -> [u8; 96] {
        let mut s = [40u8; 96];

        match self {
            StrengthStyle::FourFourStandard => {
                for i in 0..96 {
                    let pos_in_beat = i % 24;
                    if i == 0 { s[i] = 100; }
                    else if i == 48 { s[i] = 90; }
                    else if i == 24 || i == 72 { s[i] = 80; }
                    else if pos_in_beat == 12 { s[i] = 65; }
                    else if pos_in_beat == 6 || pos_in_beat == 18 { s[i] = 55; }
                    else if pos_in_beat % 3 == 0 { s[i] = 50; }
                    else { s[i] = 40; }
                }
            }
            StrengthStyle::Backbeat => {
                for i in 0..96 {
                    let pos_in_beat = i % 24;
                    if i == 24 || i == 72 { s[i] = 100; }
                    else if i == 0 || i == 48 { s[i] = 65; }
                    else if pos_in_beat == 12 { s[i] = 55; }
                    else { s[i] = 35; }
                }
            }
            StrengthStyle::Offbeat => {
                for i in 0..96 {
                    if i == 12 || i == 36 || i == 60 || i == 84 { s[i] = 100; }
                    else if i == 0 || i == 24 || i == 48 || i == 72 { s[i] = 50; }
                    else if i % 24 == 6 || i % 24 == 18 { s[i] = 70; }
                    else { s[i] = 35; }
                }
            }
            StrengthStyle::TripletFeel => {
                for i in 0..96 {
                    if i == 0 { s[i] = 100; }
                    else if i == 48 { s[i] = 90; }
                    else if i == 24 || i == 72 { s[i] = 80; }
                    else if i % 32 == 0 { s[i] = 85; }
                    else if i % 32 == 16 { s[i] = 70; }
                    else if i % 8 == 0 { s[i] = 60; }
                    else { s[i] = 40; }
                }
            }
            StrengthStyle::Shuffle => {
                for i in 0..96 {
                    let pos_in_beat = i % 24;
                    if i == 0 { s[i] = 100; }
                    else if i == 48 { s[i] = 90; }
                    else if i == 24 || i == 72 { s[i] = 80; }
                    else if pos_in_beat == 16 { s[i] = 75; }
                    else if pos_in_beat == 8 { s[i] = 55; }
                    else { s[i] = 40; }
                }
            }
            StrengthStyle::Sparse => {
                for i in 0..96 {
                    if i == 0 { s[i] = 100; }
                    else if i == 48 { s[i] = 80; }
                    else if i == 24 || i == 72 { s[i] = 60; }
                    else { s[i] = 25; }
                }
            }
            StrengthStyle::Dense => {
                for i in 0..96 {
                    if i == 0 { s[i] = 100; }
                    else if i == 48 { s[i] = 95; }
                    else if i == 24 || i == 72 { s[i] = 90; }
                    else if i % 24 == 12 { s[i] = 85; }
                    else if i % 24 == 6 || i % 24 == 18 { s[i] = 80; }
                    else if i % 3 == 0 { s[i] = 75; }
                    else { s[i] = 65; }
                }
            }
            StrengthStyle::Polyrhythm34 => {
                for i in 0..96 {
                    let beat4 = i == 0 || i == 24 || i == 48 || i == 72;
                    let beat3 = i == 0 || i == 32 || i == 64;
                    if i == 0 { s[i] = 100; }
                    else if beat3 && beat4 { s[i] = 95; }
                    else if beat4 { s[i] = 80; }
                    else if beat3 { s[i] = 85; }
                    else { s[i] = 40; }
                }
            }
            StrengthStyle::African => {
                for i in 0..96 {
                    if i == 0 { s[i] = 100; }
                    else if i == 18 { s[i] = 90; }
                    else if i == 36 { s[i] = 85; }
                    else if i == 54 { s[i] = 80; }
                    else if i == 72 { s[i] = 75; }
                    else if i == 9 || i == 27 || i == 45 || i == 63 || i == 81 { s[i] = 65; }
                    else { s[i] = 35; }
                }
            }
            StrengthStyle::Reggae => {
                for i in 0..96 {
                    let pos_in_beat = i % 24;
                    if i == 24 || i == 72 { s[i] = 100; }
                    else if pos_in_beat == 12 { s[i] = 85; }
                    else if i == 0 || i == 48 { s[i] = 45; }
                    else { s[i] = 30; }
                }
            }
            StrengthStyle::Latin => {
                let son_clave = [0, 18, 48, 60, 78];
                for i in 0..96 {
                    if i == 0 { s[i] = 100; }
                    else if son_clave.contains(&i) { s[i] = 90; }
                    else if i == 24 || i == 72 { s[i] = 70; }
                    else if i % 24 == 12 { s[i] = 55; }
                    else { s[i] = 35; }
                }
            }
            StrengthStyle::Funk => {
                for i in 0..96 {
                    let pos_in_beat = i % 24;
                    if i == 0 { s[i] = 100; }
                    else if i == 24 || i == 72 { s[i] = 85; }
                    else if i == 48 { s[i] = 75; }
                    else if i == 66 { s[i] = 90; }
                    else if pos_in_beat == 6 || pos_in_beat == 18 { s[i] = 70; }
                    else if pos_in_beat == 12 { s[i] = 60; }
                    else { s[i] = 40; }
                }
            }
            StrengthStyle::Jazz => {
                for i in 0..96 {
                    if i == 0 { s[i] = 90; }
                    else if i == 48 { s[i] = 85; }
                    else if i == 24 || i == 72 { s[i] = 80; }
                    else if i % 32 == 0 { s[i] = 75; }
                    else if i % 32 == 16 { s[i] = 70; }
                    else if i % 8 == 0 { s[i] = 60; }
                    else { s[i] = 45; }
                }
            }
            StrengthStyle::Ambient => {
                for i in 0..96 {
                    let wave = (i as f32 * std::f32::consts::PI * 2.0 / 96.0).sin();
                    s[i] = (55.0 + 25.0 * wave) as u8;
                    if i == 0 { s[i] = 80; }
                    else if i == 48 { s[i] = 70; }
                }
            }
            StrengthStyle::Driving => {
                for i in 0..96 {
                    if i == 0 { s[i] = 100; }
                    else if i == 24 || i == 48 || i == 72 { s[i] = 95; }
                    else if i % 24 == 12 { s[i] = 90; }
                    else if i % 24 == 6 || i % 24 == 18 { s[i] = 85; }
                    else if i % 3 == 0 { s[i] = 75; }
                    else { s[i] = 60; }
                }
            }
        }
        s
    }
}

#[derive(Clone, PartialEq)]
struct StrengthState {
    beat_strength_mode: BeatStrengthMode,
    beat_strength_values: [u8; 96], // LCM of 32 and 24 = 96, values 0-100
    last_preset_version: u64,
    selected_style: Option<StrengthStyle>,
}

impl Default for StrengthState {
    fn default() -> Self {
        let mut values = [0u8; 96];
        // Downbeat - strongest
        values[0] = 100;
        // Quarter notes (every 24/96)
        values[24] = 75;
        values[48] = 75;
        values[72] = 75;
        // Eighth notes (every 12/96)
        values[12] = 50;
        values[36] = 50;
        values[60] = 50;
        values[84] = 50;

        Self {
            beat_strength_mode: BeatStrengthMode::Straight,
            beat_strength_values: values,
            last_preset_version: 0,
            selected_style: None,
        }
    }
}

fn sync_state_from_shared(state: &mut StrengthState, ui_state: &Arc<SharedUiState>) {
    if let Ok(strength_values) = ui_state.strength_values.lock() {
        for i in 0..96 {
            state.beat_strength_values[i] = (strength_values[i] * 100.0).round() as u8;
        }
    }
    state.selected_style = None;
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
    params: &Arc<DeviceParams>,
    _setter: &nih_plug::prelude::ParamSetter,
    ui_state: &Arc<SharedUiState>,
) {
    let state_id = egui::Id::new("strength_state");
    let current_version = ui_state.get_preset_version();
    let swing = params.swing_amount.modulated_plain_value();

    tui.ui(|ui| {
        ui.add_space(12.0);
        ui.horizontal(|ui| {
            ui.heading(egui::RichText::new("    Strength").size(22.0));
            ui.add_space(40.0);

            let mut state = ui.ctx().data_mut(|d| d.get_temp::<StrengthState>(state_id).unwrap_or_default());
            let state_before = state.clone();

            ui.label(egui::RichText::new("Style:").size(18.0));
            ui.add_space(8.0);

            let current_name = state.selected_style.map(|s| s.name()).unwrap_or("Custom");
            egui::ComboBox::from_id_salt("strength_style_header")
                .selected_text(egui::RichText::new(current_name).size(18.0))
                .width(180.0)
                .height(500.0)
                .show_ui(ui, |ui| {
                    for style in StrengthStyle::all() {
                        let btn = egui::Button::new(egui::RichText::new(style.name()).size(18.0))
                            .min_size(egui::vec2(160.0, 40.0))
                            .selected(state.selected_style == Some(*style));
                        if ui.add(btn).clicked() {
                            state.selected_style = Some(*style);
                            state.beat_strength_values = style.generate_pattern();
                            ui.close_menu();
                        }
                    }
                });

            if state != state_before {
                if let Ok(mut strength_values) = ui_state.strength_values.lock() {
                    for i in 0..96 {
                        strength_values[i] = state.beat_strength_values[i] as f32 / 100.0;
                    }
                }
                ui.ctx().data_mut(|d| d.insert_temp(state_id, state));
            }
        });
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

        render_beat_strength(ui, &mut state, swing);

        if state != state_before {
            if let Ok(mut strength_values) = ui_state.strength_values.lock() {
                for i in 0..96 {
                    strength_values[i] = state.beat_strength_values[i] as f32 / 100.0;
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

        ui.horizontal(|ui| {
            ui.add_space(48.0);

            let button_s = egui::Button::new(egui::RichText::new("S").size(20.0))
                .min_size(egui::vec2(96.0, 48.0))
                .selected(matches!(state.beat_strength_mode, BeatStrengthMode::Straight));

            if ui.add(button_s).clicked() {
                state.beat_strength_mode = BeatStrengthMode::Straight;
            }

            let button_t = egui::Button::new(egui::RichText::new("T").size(20.0))
                .min_size(egui::vec2(96.0, 48.0))
                .selected(matches!(state.beat_strength_mode, BeatStrengthMode::Triplet));

            if ui.add(button_t).clicked() {
                state.beat_strength_mode = BeatStrengthMode::Triplet;
            }
        });

        if state != state_before {
            ui.ctx().data_mut(|d| d.insert_temp(state_id, state));
        }
    });
}

fn render_beat_strength(ui: &mut egui::Ui, state: &mut StrengthState, swing: f32) {
    render_beat_strength_grid(ui, state, swing);
}


fn render_beat_strength_grid(ui: &mut egui::Ui, state: &mut StrengthState, swing: f32) {
    let container_height = 420.0;
    ui.set_min_size(egui::vec2(1220.0, container_height));
    ui.set_max_width(1220.0);

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

            render_beat_strength_grid_lines(ui, state.beat_strength_mode, num_sliders, container_height, swing);
            render_opposite_mode_lines(ui, state, container_height, swing);
            render_beat_strength_sliders(ui, state, num_sliders, container_height, swing);
        });
}

fn render_opposite_mode_lines(ui: &mut egui::Ui, state: &StrengthState, _container_height: f32, swing: f32) {
    let container_rect = ui.available_rect_before_wrap();
    let painter = ui.painter();
    let container_width = 1216.0;
    let grid_padding = 16.0;
    let grid_width = container_width - (grid_padding * 2.0);
    let max_height = 388.0;
    let top_y = container_rect.min.y + 16.0;

    match state.beat_strength_mode {
        BeatStrengthMode::Straight => {
            for triplet_pos in 0..24 {
                let grid_pos = triplet_to_grid(triplet_pos);
                let coincides = grid_pos.is_multiple_of(3) && (grid_pos / 3) < 32;

                if !coincides {
                    let value = state.beat_strength_values[grid_pos];
                    if value > 0 {
                        let height = max_height * (value as f32 / 100.0);
                        let time_pos = triplet_pos as f32 / 24.0;
                        let swung_time = DeviceParams::apply_swing(time_pos, swing);
                        let x = container_rect.min.x + grid_padding + swung_time * grid_width;

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
            for straight_pos in 0..32 {
                let grid_pos = straight_to_grid(straight_pos);
                let coincides = grid_pos.is_multiple_of(4) && (grid_pos / 4) < 24;

                if !coincides {
                    let value = state.beat_strength_values[grid_pos];
                    if value > 0 {
                        let height = max_height * (value as f32 / 100.0);
                        let time_pos = straight_pos as f32 / 32.0;
                        let swung_time = DeviceParams::apply_swing(time_pos, swing);
                        let x = container_rect.min.x + grid_padding + swung_time * grid_width;

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

fn render_beat_strength_grid_lines(ui: &mut egui::Ui, mode: BeatStrengthMode, num_sliders: usize, container_height: f32, swing: f32) {
    let container_rect = ui.available_rect_before_wrap();
    let painter = ui.painter();
    let container_width = 1216.0;
    let grid_padding = 16.0;
    let grid_width = container_width - (grid_padding * 2.0);

    let (num_v_grid_positions, grid_spaces) = match mode {
        BeatStrengthMode::Straight => (33, 32.0),
        BeatStrengthMode::Triplet => (25, 24.0),
    };

    for i in 0..num_v_grid_positions {
        let normalized_pos = i as f32 / grid_spaces;
        let swung_pos = DeviceParams::apply_swing(normalized_pos, swing);
        let x = container_rect.min.x + grid_padding + swung_pos * grid_width;
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
        let y = container_rect.min.y + 16.0 + i as f32 * (container_height - 32.0) / 4.0;
        painter.line_segment(
            [
                egui::pos2(container_rect.min.x + 16.0, y),
                egui::pos2(container_rect.max.x - 18.0, y),
            ],
            egui::Stroke::new(1.0, Color32::from_rgb(20, 20, 20)),
        );
    }
}

fn render_beat_strength_sliders(ui: &mut egui::Ui, state: &mut StrengthState, num_sliders: usize, container_height: f32, swing: f32) {
    let container_width = 1216.0;
    let grid_padding = 16.0;
    let grid_width = container_width - (grid_padding * 2.0);

    ui.vertical(|ui| {
        ui.set_min_size(egui::vec2(1216.0, container_height));
        ui.set_max_width(1216.0);
        ui.add_space(16.0);

        ui.horizontal_top(|ui| {
            let grid_base = match state.beat_strength_mode {
                BeatStrengthMode::Straight => 32.0,
                BeatStrengthMode::Triplet => 24.0,
            };

            for i in 0..num_sliders {
                let normalized_pos = i as f32 / grid_base;
                let swung_pos = DeviceParams::apply_swing(normalized_pos, swing);
                let target_x = grid_padding + swung_pos * grid_width;
                let current_x = ui.cursor().min.x - 31.0;
                let space_needed = target_x - current_x;

                ui.add_space(space_needed);

                ui.vertical(|ui| {
                    let grid_pos = match state.beat_strength_mode {
                        BeatStrengthMode::Straight => straight_to_grid(i),
                        BeatStrengthMode::Triplet => triplet_to_grid(i),
                    };
                    let value = &mut state.beat_strength_values[grid_pos];

                    let slider_height = 388.0;

                    ui.style_mut().spacing.slider_width = slider_height;
                    ui.style_mut().spacing.slider_rail_height = 16.0;
                    ui.style_mut().visuals.selection.bg_fill = Color32::from_rgb(100, 150, 200);

                    ui.add(
                        egui::Slider::new(value, 0..=100)
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
