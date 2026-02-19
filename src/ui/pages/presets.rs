#![allow(clippy::field_reassign_with_default)]

use std::sync::Arc;
use egui_taffy::TuiBuilderLogic;
use nih_plug_egui::egui::{self, Color32};
use crate::params::DeviceParams;
use crate::ui::SharedUiState;
use crate::preset::manager::{FactoryBank, UserBank, PresetLocation};
use crate::preset::Preset;
use nih_plug::prelude::*;

#[derive(Clone, PartialEq)]
enum PresetSection {
    Factory,
    User,
}

#[derive(Clone, PartialEq)]
enum PageMode {
    Browse,
    Save,
}

#[derive(Clone, PartialEq)]
struct PresetPageState {
    section: PresetSection,
    factory_bank: FactoryBank,
    user_bank: UserBank,
    selected_preset: usize,
    mode: PageMode,
    name_buffer: String,
    author_buffer: String,
    status_message: Option<(String, std::time::Instant)>,
}

impl Default for PresetPageState {
    fn default() -> Self {
        Self {
            section: PresetSection::Factory,
            factory_bank: FactoryBank::A,
            user_bank: UserBank::U1,
            selected_preset: 0,
            mode: PageMode::Browse,
            name_buffer: String::new(),
            author_buffer: String::new(),
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
    use egui_taffy::taffy::{prelude::*, style::AlignItems};

    let state_id = egui::Id::new("preset_page_state");

    tui.style(Style {
        flex_grow: 1.0,
        align_items: Some(AlignItems::Stretch),
        ..Default::default()
    })
    .ui(|ui| {
        let mut state = ui.ctx().data_mut(|d| d.get_temp::<PresetPageState>(state_id).unwrap_or_default());

        if let Some((_, time)) = &state.status_message {
            if time.elapsed().as_secs() > 3 {
                state.status_message = None;
            }
        }

        let screen_rect = ui.ctx().screen_rect();
        let top_y = ui.cursor().min.y;

        let padding = 20.0;
        let content_rect = egui::Rect::from_min_max(
            egui::pos2(screen_rect.left() + padding, top_y + padding + 4.0),
            egui::pos2(screen_rect.right() - padding, screen_rect.bottom() - padding),
        );

        ui.allocate_new_ui(egui::UiBuilder::new().max_rect(content_rect), |ui| {

                // Top row: Section toggle + Preset name + Actions (or Save controls)
                let (selected_preset_name_for_header, selected_preset_author) = if let Ok(manager) = ui_state.preset_manager.lock() {
                    let p = match state.section {
                        PresetSection::Factory => &manager.get_factory_bank(state.factory_bank).presets[state.selected_preset],
                        PresetSection::User => &manager.get_user_bank(state.user_bank).presets[state.selected_preset],
                    };
                    (p.name.clone(), p.author.clone())
                } else {
                    (String::new(), String::new())
                };

                ui.horizontal(|ui| {
                    let factory_btn = egui::Button::new(
                        egui::RichText::new("FACTORY").size(18.0).strong().color(Color32::WHITE)
                    )
                    .min_size(egui::vec2(120.0, 48.0))
                    .fill(if state.section == PresetSection::Factory {
                        Color32::from_rgb(60, 100, 160)
                    } else {
                        Color32::from_rgb(50, 50, 50)
                    });

                    let user_btn = egui::Button::new(
                        egui::RichText::new("USER").size(18.0).strong().color(Color32::WHITE)
                    )
                    .min_size(egui::vec2(120.0, 48.0))
                    .fill(if state.section == PresetSection::User {
                        Color32::from_rgb(100, 80, 60)
                    } else {
                        Color32::from_rgb(50, 50, 50)
                    });

                    if ui.add(factory_btn).clicked() {
                        state.section = PresetSection::Factory;
                        state.selected_preset = 0;
                    }
                    ui.add_space(8.0);
                    if ui.add(user_btn).clicked() {
                        state.section = PresetSection::User;
                        state.selected_preset = 0;
                    }

                    match state.mode {
                        PageMode::Browse => {
                            ui.add_space(24.0);
                            let title_resp = ui.label(egui::RichText::new(selected_preset_name_for_header.to_uppercase()).size(20.0).strong());
                            if !selected_preset_author.is_empty() {
                                ui.painter().text(
                                    egui::pos2(title_resp.rect.right() + 8.0, title_resp.rect.bottom() - 1.0),
                                    egui::Align2::LEFT_BOTTOM,
                                    format!("by {}", selected_preset_author),
                                    egui::FontId::proportional(14.0),
                                    Color32::from_rgb(120, 120, 120),
                                );
                            }

                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                let is_fav = if let Ok(mgr) = ui_state.preset_manager.lock() {
                                    let location = match state.section {
                                        PresetSection::Factory => PresetLocation::Factory(state.factory_bank, state.selected_preset),
                                        PresetSection::User => PresetLocation::User(state.user_bank, state.selected_preset),
                                    };
                                    mgr.is_favorite(location)
                                } else {
                                    false
                                };

                                let star_label = if is_fav { "★" } else { "☆" };
                                let star_btn = egui::Button::new(
                                    egui::RichText::new(star_label).size(18.0)
                                ).min_size(egui::vec2(48.0, 48.0))
                                .fill(if is_fav { Color32::from_rgb(180, 140, 40) } else { Color32::from_rgb(60, 60, 60) });

                                if ui.add(star_btn).clicked() {
                                    if let Ok(mut mgr) = ui_state.preset_manager.lock() {
                                        let location = match state.section {
                                            PresetSection::Factory => PresetLocation::Factory(state.factory_bank, state.selected_preset),
                                            PresetSection::User => PresetLocation::User(state.user_bank, state.selected_preset),
                                        };
                                        let is_now_fav = mgr.toggle_favorite(location);
                                        let _ = mgr.save_favorites();
                                        state.status_message = Some((
                                            if is_now_fav { "Added to favorites".to_string() } else { "Removed from favorites".to_string() },
                                            std::time::Instant::now()
                                        ));
                                    }
                                }

                                ui.add_space(8.0);

                                let init_btn = egui::Button::new(
                                    egui::RichText::new("INIT").size(18.0).color(Color32::WHITE)
                                ).min_size(egui::vec2(80.0, 48.0))
                                .fill(Color32::from_rgb(100, 70, 70));

                                if ui.add(init_btn).clicked() {
                                    let default_data = crate::preset::PresetData::default();
                                    load_preset_to_params(&default_data, params, setter, ui_state);
                                    if matches!(state.section, PresetSection::User) {
                                        if let Ok(mut mgr) = ui_state.preset_manager.lock() {
                                            mgr.init_user_preset(state.user_bank, state.selected_preset);
                                            let _ = mgr.save_user_presets();
                                        }
                                    }
                                    state.status_message = Some(("Initialized to default".to_string(), std::time::Instant::now()));
                                }

                                ui.add_space(8.0);

                                let save_btn = egui::Button::new(
                                    egui::RichText::new("SAVE").size(18.0).color(Color32::WHITE)
                                ).min_size(egui::vec2(80.0, 48.0))
                                .fill(Color32::from_rgb(80, 100, 60));

                                if ui.add(save_btn).clicked() {
                                    state.mode = PageMode::Save;
                                    state.name_buffer = selected_preset_name_for_header.clone();
                                    state.author_buffer = "User".to_string();
                                }
                            });
                        }
                        PageMode::Save => {
                            ui.add_space(16.0);

                            ui.label(egui::RichText::new("Name:").size(20.0));
                            ui.add_space(4.0);
                            ui.add(
                                egui::TextEdit::singleline(&mut state.name_buffer)
                                    .desired_width(180.0)
                                    .font(egui::FontId::proportional(20.0))
                            );

                            ui.add_space(16.0);

                            ui.label(egui::RichText::new("Author:").size(20.0));
                            ui.add_space(4.0);
                            ui.add(
                                egui::TextEdit::singleline(&mut state.author_buffer)
                                    .desired_width(120.0)
                                    .font(egui::FontId::proportional(20.0))
                            );

                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                let cancel_btn = egui::Button::new(
                                    egui::RichText::new("CANCEL").size(18.0).color(Color32::WHITE)
                                ).min_size(egui::vec2(90.0, 48.0));

                                if ui.add(cancel_btn).clicked() {
                                    state.mode = PageMode::Browse;
                                }

                                ui.add_space(8.0);

                                let confirm_btn = egui::Button::new(
                                    egui::RichText::new("CONFIRM").size(18.0).color(Color32::WHITE)
                                ).min_size(egui::vec2(100.0, 48.0))
                                .fill(Color32::from_rgb(80, 120, 80));

                                if ui.add(confirm_btn).clicked() && !state.name_buffer.is_empty() {
                                    let data = save_params_to_preset_data(params, ui_state);
                                    let preset = Preset::with_author_and_description(
                                        &state.name_buffer,
                                        &state.author_buffer,
                                        "",
                                        data,
                                    );
                                    if let Ok(mut mgr) = ui_state.preset_manager.lock() {
                                        match state.section {
                                            PresetSection::Factory => {
                                                mgr.save_to_factory_slot(state.factory_bank, state.selected_preset, preset);
                                                mgr.set_current_location(PresetLocation::Factory(state.factory_bank, state.selected_preset));
                                                match mgr.save_factory_presets() {
                                                    Ok(_) => {
                                                        state.status_message = Some(("Preset saved!".to_string(), std::time::Instant::now()));
                                                    }
                                                    Err(e) => {
                                                        state.status_message = Some((format!("Save error: {}", e), std::time::Instant::now()));
                                                    }
                                                }
                                            }
                                            PresetSection::User => {
                                                mgr.save_to_user_slot(state.user_bank, state.selected_preset, preset);
                                                mgr.set_current_location(PresetLocation::User(state.user_bank, state.selected_preset));
                                                match mgr.save_user_presets() {
                                                    Ok(_) => {
                                                        state.status_message = Some(("Preset saved!".to_string(), std::time::Instant::now()));
                                                    }
                                                    Err(e) => {
                                                        state.status_message = Some((format!("Save error: {}", e), std::time::Instant::now()));
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    state.mode = PageMode::Browse;
                                }
                            });
                        }
                    }
                });

                ui.add_space(22.0);

                // Bank buttons (2 cols) + Preset grid (8 cols) side by side
                let mut clicked_preset: Option<usize> = None;

                if let Ok(manager) = ui_state.preset_manager.lock() {
                    let current_location = Some(manager.current_location());

                    let banks: Vec<(usize, &str, bool)> = match state.section {
                        PresetSection::Factory => {
                            FactoryBank::all().iter().enumerate().map(|(i, b)| {
                                (i, b.label(), state.factory_bank == *b)
                            }).collect()
                        }
                        PresetSection::User => {
                            UserBank::all().iter().enumerate().map(|(i, b)| {
                                (i, b.label(), state.user_bank == *b)
                            }).collect()
                        }
                    };

                    let current_bank_idx = match state.section {
                        PresetSection::Factory => state.factory_bank as usize,
                        PresetSection::User => state.user_bank as usize,
                    };
                    let bank_colors: [(u8, u8, u8); 8] = [
                        (65, 45, 45),   // warm red
                        (60, 55, 38),   // amber
                        (40, 58, 45),   // forest
                        (40, 52, 62),   // steel blue
                        (52, 42, 62),   // violet
                        (62, 42, 55),   // rose
                        (42, 58, 58),   // teal
                        (55, 50, 42),   // sand
                    ];

                    let rounding = 4.0;
                    let spacing = 6.0_f32;
                    let bank_gap = 8.0_f32;
                    let content_w = ui.available_width();
                    let btn_w = (content_w - 9.0 * spacing - bank_gap) / 10.0;
                    let grid_h = ui.available_height();
                    let row_gap = 6.0_f32;
                    let btn_h = (grid_h - 3.0 * row_gap) / 4.0 - 3.0;

                    for row in 0..4 {
                        let row_start = row * 8;
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = 6.0;

                            for col in 0..2 {
                                let bank_idx = row * 2 + col;
                                if bank_idx < banks.len() {
                                    let (_, label, is_selected) = banks[bank_idx];
                                    let (br, bg, bb) = bank_colors[bank_idx % bank_colors.len()];
                                    let fill = if is_selected {
                                        let boost = 1.6_f32;
                                        let avg = (br as f32 + bg as f32 + bb as f32) / 3.0;
                                        let sr = avg + (br as f32 - avg) * boost;
                                        let sg = avg + (bg as f32 - avg) * boost;
                                        let sb = avg + (bb as f32 - avg) * boost;
                                        Color32::from_rgb(sr.clamp(0.0, 255.0) as u8, sg.clamp(0.0, 255.0) as u8, sb.clamp(0.0, 255.0) as u8)
                                    } else {
                                        Color32::from_rgb(br, bg, bb)
                                    };
                                    let button = egui::Button::new(
                                        egui::RichText::new(label).size(22.0).strong()
                                    )
                                    .min_size(egui::vec2(btn_w, btn_h))
                                    .corner_radius(rounding)
                                    .fill(fill);

                                    if ui.add(button).clicked() {
                                        match state.section {
                                            PresetSection::Factory => {
                                                if let Some(b) = FactoryBank::from_index(bank_idx) {
                                                    state.factory_bank = b;
                                                    state.selected_preset = 0;
                                                }
                                            }
                                            PresetSection::User => {
                                                if let Some(b) = UserBank::from_index(bank_idx) {
                                                    state.user_bank = b;
                                                    state.selected_preset = 0;
                                                }
                                            }
                                        }
                                    }
                                }
                            }

                            ui.add_space(8.0);

                            for col in 0..8 {
                                let i = row_start + col;
                                let (is_empty, is_favorite, preset_name) = match state.section {
                                    PresetSection::Factory => {
                                        let p = &manager.get_factory_bank(state.factory_bank).presets[i];
                                        let empty = p.name.starts_with("Init ");
                                        let fav = manager.is_favorite_by_indices(true, state.factory_bank as usize, i);
                                        (empty, fav, p.name.clone())
                                    }
                                    PresetSection::User => {
                                        let p = &manager.get_user_bank(state.user_bank).presets[i];
                                        let empty = p.name.starts_with("User ") || p.name.starts_with("Init");
                                        let fav = manager.is_favorite_by_indices(false, state.user_bank as usize, i);
                                        (empty, fav, p.name.clone())
                                    }
                                };

                                let is_selected = state.selected_preset == i;
                                let is_current = match (&state.section, current_location) {
                                    (PresetSection::Factory, Some(PresetLocation::Factory(b, idx))) => {
                                        b == state.factory_bank && idx == i
                                    }
                                    (PresetSection::User, Some(PresetLocation::User(b, idx))) => {
                                        b == state.user_bank && idx == i
                                    }
                                    _ => false,
                                };

                                let is_group_start = i % 4 == 0;
                                let (cr, cg, cb) = bank_colors[current_bank_idx % bank_colors.len()];
                                let fill_color = if is_current {
                                    Color32::from_rgb(80, 140, 80)
                                } else if is_selected {
                                    match state.section {
                                        PresetSection::Factory => Color32::from_rgb(70, 100, 140),
                                        PresetSection::User => Color32::from_rgb(100, 80, 60),
                                    }
                                } else if is_empty {
                                    if is_group_start { Color32::from_rgb(28, 28, 28) } else { Color32::from_rgb(35, 35, 35) }
                                } else {
                                    let base = 36_u8;
                                    let tint = if is_group_start { 0.3_f32 } else { 0.35 };
                                    Color32::from_rgb(
                                        base + ((cr as f32 - base as f32) * tint) as u8,
                                        base + ((cg as f32 - base as f32) * tint) as u8,
                                        base + ((cb as f32 - base as f32) * tint) as u8,
                                    )
                                };

                                let (rect, response) = ui.allocate_exact_size(
                                    egui::vec2(btn_w, btn_h),
                                    egui::Sense::click(),
                                );
                                ui.painter().rect_filled(rect, rounding, fill_color);
                                if response.hovered() {
                                    ui.painter().rect_stroke(rect, rounding, egui::Stroke::new(1.0, Color32::from_rgb(140, 140, 140)), egui::epaint::StrokeKind::Inside);
                                }

                                let mut header_job = egui::text::LayoutJob::default();
                                header_job.append(
                                    &format!("{}", i + 1),
                                    0.0,
                                    egui::TextFormat {
                                        font_id: egui::FontId::proportional(18.0),
                                        color: Color32::from_rgb(160, 160, 160),
                                        ..Default::default()
                                    },
                                );
                                let header_galley = ui.ctx().fonts(|f| f.layout_job(header_job));
                                ui.painter().galley(
                                    rect.left_top() + egui::vec2(8.0, 6.0),
                                    header_galley,
                                    Color32::TRANSPARENT,
                                );

                                if is_favorite {
                                    ui.painter().text(
                                        rect.right_top() + egui::vec2(-8.0, 6.0),
                                        egui::Align2::RIGHT_TOP,
                                        "★",
                                        egui::FontId::proportional(14.0),
                                        Color32::from_rgb(220, 180, 60),
                                    );
                                }

                                if !is_empty {
                                    let mut name_job = egui::text::LayoutJob::default();
                                    name_job.wrap.max_width = rect.width() - 14.0;
                                    name_job.append(
                                        &preset_name,
                                        0.0,
                                        egui::TextFormat {
                                            font_id: egui::FontId::proportional(18.0),
                                            color: Color32::from_rgb(220, 220, 220),
                                            ..Default::default()
                                        },
                                    );
                                    let name_galley = ui.ctx().fonts(|f| f.layout_job(name_job));
                                    let clip = rect.shrink2(egui::vec2(4.0, 2.0));
                                    ui.painter().with_clip_rect(clip).galley(
                                        rect.left_top() + egui::vec2(8.0, 32.0),
                                        name_galley,
                                        Color32::TRANSPARENT,
                                    );
                                }

                                if response.clicked() {
                                    state.selected_preset = i;
                                    if state.mode == PageMode::Browse {
                                        clicked_preset = Some(i);
                                    }
                                }
                            }
                        });
                        if row < 3 {
                            ui.add_space(6.0);
                        }
                    }
                }

                // Load preset on click (only in Browse mode)
                if let Some(preset_idx) = clicked_preset {
                    if let Ok(mut mgr) = ui_state.preset_manager.lock() {
                        let location = match state.section {
                            PresetSection::Factory => PresetLocation::Factory(state.factory_bank, preset_idx),
                            PresetSection::User => PresetLocation::User(state.user_bank, preset_idx),
                        };
                        mgr.set_current_location(location);
                        let preset = mgr.get_current_preset().clone();
                        drop(mgr);
                        load_preset_to_params(&preset.data, params, setter, ui_state);
                        state.status_message = Some((format!("Loaded: {}", preset.name), std::time::Instant::now()));
                    }
                }

            });

        ui.ctx().data_mut(|d| d.insert_temp(state_id, state));
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

    setter.set_parameter(&params.div3t_beat1, data.triplet_1_2t[0]);
    setter.set_parameter(&params.div3t_beat2, data.triplet_1_2t[1]);
    setter.set_parameter(&params.div3t_beat3, data.triplet_1_2t[2]);

    setter.set_parameter(&params.div6t_beat1, data.triplet_1_4t[0]);
    setter.set_parameter(&params.div6t_beat2, data.triplet_1_4t[1]);
    setter.set_parameter(&params.div6t_beat3, data.triplet_1_4t[2]);
    setter.set_parameter(&params.div6t_beat4, data.triplet_1_4t[3]);
    setter.set_parameter(&params.div6t_beat5, data.triplet_1_4t[4]);
    setter.set_parameter(&params.div6t_beat6, data.triplet_1_4t[5]);

    setter.set_parameter(&params.div12t_beat1, data.triplet_1_8t[0]);
    setter.set_parameter(&params.div12t_beat2, data.triplet_1_8t[1]);
    setter.set_parameter(&params.div12t_beat3, data.triplet_1_8t[2]);
    setter.set_parameter(&params.div12t_beat4, data.triplet_1_8t[3]);
    setter.set_parameter(&params.div12t_beat5, data.triplet_1_8t[4]);
    setter.set_parameter(&params.div12t_beat6, data.triplet_1_8t[5]);
    setter.set_parameter(&params.div12t_beat7, data.triplet_1_8t[6]);
    setter.set_parameter(&params.div12t_beat8, data.triplet_1_8t[7]);
    setter.set_parameter(&params.div12t_beat9, data.triplet_1_8t[8]);
    setter.set_parameter(&params.div12t_beat10, data.triplet_1_8t[9]);
    setter.set_parameter(&params.div12t_beat11, data.triplet_1_8t[10]);
    setter.set_parameter(&params.div12t_beat12, data.triplet_1_8t[11]);

    setter.set_parameter(&params.div24t_beat1, data.triplet_1_16t[0]);
    setter.set_parameter(&params.div24t_beat2, data.triplet_1_16t[1]);
    setter.set_parameter(&params.div24t_beat3, data.triplet_1_16t[2]);
    setter.set_parameter(&params.div24t_beat4, data.triplet_1_16t[3]);
    setter.set_parameter(&params.div24t_beat5, data.triplet_1_16t[4]);
    setter.set_parameter(&params.div24t_beat6, data.triplet_1_16t[5]);
    setter.set_parameter(&params.div24t_beat7, data.triplet_1_16t[6]);
    setter.set_parameter(&params.div24t_beat8, data.triplet_1_16t[7]);
    setter.set_parameter(&params.div24t_beat9, data.triplet_1_16t[8]);
    setter.set_parameter(&params.div24t_beat10, data.triplet_1_16t[9]);
    setter.set_parameter(&params.div24t_beat11, data.triplet_1_16t[10]);
    setter.set_parameter(&params.div24t_beat12, data.triplet_1_16t[11]);
    setter.set_parameter(&params.div24t_beat13, data.triplet_1_16t[12]);
    setter.set_parameter(&params.div24t_beat14, data.triplet_1_16t[13]);
    setter.set_parameter(&params.div24t_beat15, data.triplet_1_16t[14]);
    setter.set_parameter(&params.div24t_beat16, data.triplet_1_16t[15]);
    setter.set_parameter(&params.div24t_beat17, data.triplet_1_16t[16]);
    setter.set_parameter(&params.div24t_beat18, data.triplet_1_16t[17]);
    setter.set_parameter(&params.div24t_beat19, data.triplet_1_16t[18]);
    setter.set_parameter(&params.div24t_beat20, data.triplet_1_16t[19]);
    setter.set_parameter(&params.div24t_beat21, data.triplet_1_16t[20]);
    setter.set_parameter(&params.div24t_beat22, data.triplet_1_16t[21]);
    setter.set_parameter(&params.div24t_beat23, data.triplet_1_16t[22]);
    setter.set_parameter(&params.div24t_beat24, data.triplet_1_16t[23]);

    setter.set_parameter(&params.div2d_beat1, data.dotted_1_2d[0]);
    setter.set_parameter(&params.div2d_beat2, data.dotted_1_2d[1]);

    setter.set_parameter(&params.div3d_beat1, data.dotted_1_4d[0]);
    setter.set_parameter(&params.div3d_beat2, data.dotted_1_4d[1]);
    setter.set_parameter(&params.div3d_beat3, data.dotted_1_4d[2]);

    setter.set_parameter(&params.div6d_beat1, data.dotted_1_8d[0]);
    setter.set_parameter(&params.div6d_beat2, data.dotted_1_8d[1]);
    setter.set_parameter(&params.div6d_beat3, data.dotted_1_8d[2]);
    setter.set_parameter(&params.div6d_beat4, data.dotted_1_8d[3]);
    setter.set_parameter(&params.div6d_beat5, data.dotted_1_8d[4]);
    setter.set_parameter(&params.div6d_beat6, data.dotted_1_8d[5]);

    setter.set_parameter(&params.div11d_beat1, data.dotted_1_16d[0]);
    setter.set_parameter(&params.div11d_beat2, data.dotted_1_16d[1]);
    setter.set_parameter(&params.div11d_beat3, data.dotted_1_16d[2]);
    setter.set_parameter(&params.div11d_beat4, data.dotted_1_16d[3]);
    setter.set_parameter(&params.div11d_beat5, data.dotted_1_16d[4]);
    setter.set_parameter(&params.div11d_beat6, data.dotted_1_16d[5]);
    setter.set_parameter(&params.div11d_beat7, data.dotted_1_16d[6]);
    setter.set_parameter(&params.div11d_beat8, data.dotted_1_16d[7]);
    setter.set_parameter(&params.div11d_beat9, data.dotted_1_16d[8]);
    setter.set_parameter(&params.div11d_beat10, data.dotted_1_16d[9]);
    setter.set_parameter(&params.div11d_beat11, data.dotted_1_16d[10]);

    setter.set_parameter(&params.div22d_beat1, data.dotted_1_32d[0]);
    setter.set_parameter(&params.div22d_beat2, data.dotted_1_32d[1]);
    setter.set_parameter(&params.div22d_beat3, data.dotted_1_32d[2]);
    setter.set_parameter(&params.div22d_beat4, data.dotted_1_32d[3]);
    setter.set_parameter(&params.div22d_beat5, data.dotted_1_32d[4]);
    setter.set_parameter(&params.div22d_beat6, data.dotted_1_32d[5]);
    setter.set_parameter(&params.div22d_beat7, data.dotted_1_32d[6]);
    setter.set_parameter(&params.div22d_beat8, data.dotted_1_32d[7]);
    setter.set_parameter(&params.div22d_beat9, data.dotted_1_32d[8]);
    setter.set_parameter(&params.div22d_beat10, data.dotted_1_32d[9]);
    setter.set_parameter(&params.div22d_beat11, data.dotted_1_32d[10]);
    setter.set_parameter(&params.div22d_beat12, data.dotted_1_32d[11]);
    setter.set_parameter(&params.div22d_beat13, data.dotted_1_32d[12]);
    setter.set_parameter(&params.div22d_beat14, data.dotted_1_32d[13]);
    setter.set_parameter(&params.div22d_beat15, data.dotted_1_32d[14]);
    setter.set_parameter(&params.div22d_beat16, data.dotted_1_32d[15]);
    setter.set_parameter(&params.div22d_beat17, data.dotted_1_32d[16]);
    setter.set_parameter(&params.div22d_beat18, data.dotted_1_32d[17]);
    setter.set_parameter(&params.div22d_beat19, data.dotted_1_32d[18]);
    setter.set_parameter(&params.div22d_beat20, data.dotted_1_32d[19]);
    setter.set_parameter(&params.div22d_beat21, data.dotted_1_32d[20]);
    setter.set_parameter(&params.div22d_beat22, data.dotted_1_32d[21]);

    setter.set_parameter(&params.synth_pll_track_speed, data.synth_pll_track_speed);
    setter.set_parameter(&params.synth_pll_damping, data.synth_pll_damping);
    setter.set_parameter(&params.synth_pll_influence, data.synth_pll_influence);
    setter.set_parameter(&params.synth_pll_mult, data.synth_pll_mult);
    setter.set_parameter(&params.synth_pll_colored, data.synth_pll_colored);
    setter.set_parameter(&params.synth_pll_mode, data.synth_pll_mode);
    setter.set_parameter(&params.synth_pll_ref_octave, data.synth_pll_ref_octave);
    setter.set_parameter(&params.synth_pll_ref_tune, data.synth_pll_ref_tune);
    setter.set_parameter(&params.synth_pll_ref_fine, data.synth_pll_ref_fine);
    setter.set_parameter(&params.synth_pll_ref_pulse_width, data.synth_pll_ref_pulse_width);
    setter.set_parameter(&params.synth_pll_feedback, data.synth_pll_feedback);
    setter.set_parameter(&params.synth_pll_volume, data.synth_pll_volume);
    setter.set_parameter(&params.synth_pll_stereo_damp_offset, data.synth_pll_stereo_damp_offset);
    setter.set_parameter(&params.synth_pll_glide, data.synth_pll_glide);
    setter.set_parameter(&params.synth_pll_fm_amount, data.synth_pll_fm_amount);
    setter.set_parameter(&params.synth_pll_fm_ratio, data.synth_pll_fm_ratio);
    setter.set_parameter(&params.synth_pll_retrigger, data.synth_pll_retrigger);
    setter.set_parameter(&params.synth_pll_burst_threshold, data.synth_pll_burst_threshold);
    setter.set_parameter(&params.synth_pll_burst_amount, data.synth_pll_burst_amount);
    setter.set_parameter(&params.synth_pll_loop_saturation, data.synth_pll_loop_saturation);
    setter.set_parameter(&params.synth_pll_color_amount, data.synth_pll_color_amount);
    setter.set_parameter(&params.synth_pll_edge_sensitivity, data.synth_pll_edge_sensitivity);
    setter.set_parameter(&params.synth_pll_range, data.synth_pll_range);
    setter.set_parameter(&params.synth_pll_stereo_track_offset, data.synth_pll_stereo_track_offset);
    setter.set_parameter(&params.synth_pll_stereo_phase, data.synth_pll_stereo_phase);
    setter.set_parameter(&params.synth_pll_cross_feedback, data.synth_pll_cross_feedback);
    setter.set_parameter(&params.synth_pll_fm_env_amount, data.synth_pll_fm_env_amount);
    setter.set_parameter(&params.synth_pll_enable, data.synth_pll_enable);

    setter.set_parameter(&params.synth_osc_octave, data.synth_osc_octave);
    setter.set_parameter(&params.synth_osc_tune, data.synth_osc_tune);
    setter.set_parameter(&params.synth_osc_fine, data.synth_osc_fine);
    setter.set_parameter(&params.synth_osc_fold, data.synth_osc_fold);
    setter.set_parameter(&params.synth_osc_d, data.synth_osc_d);
    setter.set_parameter(&params.synth_osc_v, data.synth_osc_v);
    setter.set_parameter(&params.synth_osc_stereo_v_offset, data.synth_osc_stereo_v_offset);
    setter.set_parameter(&params.synth_osc_volume, data.synth_osc_volume);

    setter.set_parameter(&params.synth_sub_volume, data.synth_sub_volume);

    setter.set_parameter(&params.synth_filter_enable, data.synth_filter_enable);
    setter.set_parameter(&params.synth_filter_cutoff, data.synth_filter_cutoff);
    setter.set_parameter(&params.synth_filter_resonance, data.synth_filter_resonance);
    setter.set_parameter(&params.synth_filter_env_amount, data.synth_filter_env_amount);
    setter.set_parameter(&params.synth_filter_drive, data.synth_filter_drive);

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
    setter.set_parameter(&params.synth_reverb_ducking, data.synth_reverb_ducking);

    setter.set_parameter(&params.lfo1_rate, data.lfo1_rate);
    setter.set_parameter(&params.lfo1_waveform, data.lfo1_waveform);
    setter.set_parameter(&params.lfo1_tempo_sync, data.lfo1_tempo_sync);
    setter.set_parameter(&params.lfo1_sync_division, data.lfo1_sync_division);
    setter.set_parameter(&params.lfo1_sync_source, data.lfo1_sync_source);
    setter.set_parameter(&params.lfo1_phase_mod, data.lfo1_phase_mod);
    setter.set_parameter(&params.lfo1_dest1, data.lfo1_dest1);
    setter.set_parameter(&params.lfo1_amount1, data.lfo1_amount1);
    setter.set_parameter(&params.lfo1_dest2, data.lfo1_dest2);
    setter.set_parameter(&params.lfo1_amount2, data.lfo1_amount2);

    setter.set_parameter(&params.lfo2_rate, data.lfo2_rate);
    setter.set_parameter(&params.lfo2_waveform, data.lfo2_waveform);
    setter.set_parameter(&params.lfo2_tempo_sync, data.lfo2_tempo_sync);
    setter.set_parameter(&params.lfo2_sync_division, data.lfo2_sync_division);
    setter.set_parameter(&params.lfo2_sync_source, data.lfo2_sync_source);
    setter.set_parameter(&params.lfo2_phase_mod, data.lfo2_phase_mod);
    setter.set_parameter(&params.lfo2_dest1, data.lfo2_dest1);
    setter.set_parameter(&params.lfo2_amount1, data.lfo2_amount1);
    setter.set_parameter(&params.lfo2_dest2, data.lfo2_dest2);
    setter.set_parameter(&params.lfo2_amount2, data.lfo2_amount2);

    setter.set_parameter(&params.lfo3_rate, data.lfo3_rate);
    setter.set_parameter(&params.lfo3_waveform, data.lfo3_waveform);
    setter.set_parameter(&params.lfo3_tempo_sync, data.lfo3_tempo_sync);
    setter.set_parameter(&params.lfo3_sync_division, data.lfo3_sync_division);
    setter.set_parameter(&params.lfo3_sync_source, data.lfo3_sync_source);
    setter.set_parameter(&params.lfo3_phase_mod, data.lfo3_phase_mod);
    setter.set_parameter(&params.lfo3_dest1, data.lfo3_dest1);
    setter.set_parameter(&params.lfo3_amount1, data.lfo3_amount1);
    setter.set_parameter(&params.lfo3_dest2, data.lfo3_dest2);
    setter.set_parameter(&params.lfo3_amount2, data.lfo3_amount2);

    setter.set_parameter(&params.swing_amount, data.swing_amount);
    setter.set_parameter(&params.note_length_percent, data.note_length_percent);

    setter.set_parameter(&params.legato_mode, data.legato_mode);
    setter.set_parameter(&params.legato_time, data.legato_time);

    setter.set_parameter(&params.len_mod_1_target, data.len_mod_1_target);
    setter.set_parameter(&params.len_mod_1_amount, data.len_mod_1_amount);
    setter.set_parameter(&params.len_mod_1_prob, data.len_mod_1_prob);

    setter.set_parameter(&params.len_mod_2_target, data.len_mod_2_target);
    setter.set_parameter(&params.len_mod_2_amount, data.len_mod_2_amount);
    setter.set_parameter(&params.len_mod_2_prob, data.len_mod_2_prob);

    setter.set_parameter(&params.vel_strength_target, data.vel_strength_target);
    setter.set_parameter(&params.vel_strength_amount, data.vel_strength_amount);
    setter.set_parameter(&params.vel_strength_prob, data.vel_strength_prob);
    setter.set_parameter(&params.vel_length_target, data.vel_length_target);
    setter.set_parameter(&params.vel_length_amount, data.vel_length_amount);
    setter.set_parameter(&params.vel_length_prob, data.vel_length_prob);

    setter.set_parameter(&params.pos_mod_1_target, data.pos_mod_1_target);
    setter.set_parameter(&params.pos_mod_1_shift, data.pos_mod_1_shift);
    setter.set_parameter(&params.pos_mod_1_prob, data.pos_mod_1_prob);

    setter.set_parameter(&params.pos_mod_2_target, data.pos_mod_2_target);
    setter.set_parameter(&params.pos_mod_2_shift, data.pos_mod_2_shift);
    setter.set_parameter(&params.pos_mod_2_prob, data.pos_mod_2_prob);

    setter.set_parameter(&params.synth_ring_mod, data.synth_ring_mod);
    setter.set_parameter(&params.synth_wavefold, data.synth_wavefold);
    setter.set_parameter(&params.synth_drift_amount, data.synth_drift_amount);
    setter.set_parameter(&params.synth_drift_rate, data.synth_drift_rate);
    setter.set_parameter(&params.synth_tube_drive, data.synth_tube_drive);
    setter.set_parameter(&params.synth_vps_enable, data.synth_vps_enable);

    if let Ok(mut strength_values) = ui_state.strength_values.lock() {
        for i in 0..96 {
            if i < data.strength_values.len() {
                strength_values[i] = data.strength_values[i] as f32 / 100.0;
            } else {
                strength_values[i] = 0.0;
            }
        }
    }

    if let Ok(mut note_pool) = ui_state.note_pool.lock() {
        note_pool.notes.clear();
        note_pool.set_root_note(data.root_note);

        for note_data in &data.notes {
            let chance = note_data.chance as f32 / 127.0;
            let strength_bias = (note_data.beat as f32 - 64.0) / 63.0;
            let length_bias = (note_data.beat_length as f32 - 64.0) / 63.0;
            note_pool.set_note_full(note_data.midi_note, note_data.octave_offset, chance, strength_bias, length_bias);
        }
    }

    if let Ok(mut scale) = ui_state.scale.lock() {
        *scale = data.scale;
    }

    if let Ok(mut pattern) = ui_state.stability_pattern.lock() {
        *pattern = data.stability_pattern;
    }

    if let Ok(mut oct_rand) = ui_state.octave_randomization.lock() {
        oct_rand.chance = data.octave_randomization.chance;
        oct_rand.strength_pref = data.octave_randomization.strength_pref;
        oct_rand.length_pref = data.octave_randomization.length_pref;
        oct_rand.direction = data.octave_randomization.direction;
    }

    ui_state.increment_preset_version();
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

    data.triplet_1_2t = [
        params.div3t_beat1.modulated_plain_value(),
        params.div3t_beat2.modulated_plain_value(),
        params.div3t_beat3.modulated_plain_value(),
    ];

    data.triplet_1_4t = [
        params.div6t_beat1.modulated_plain_value(),
        params.div6t_beat2.modulated_plain_value(),
        params.div6t_beat3.modulated_plain_value(),
        params.div6t_beat4.modulated_plain_value(),
        params.div6t_beat5.modulated_plain_value(),
        params.div6t_beat6.modulated_plain_value(),
    ];

    data.triplet_1_8t = [
        params.div12t_beat1.modulated_plain_value(),
        params.div12t_beat2.modulated_plain_value(),
        params.div12t_beat3.modulated_plain_value(),
        params.div12t_beat4.modulated_plain_value(),
        params.div12t_beat5.modulated_plain_value(),
        params.div12t_beat6.modulated_plain_value(),
        params.div12t_beat7.modulated_plain_value(),
        params.div12t_beat8.modulated_plain_value(),
        params.div12t_beat9.modulated_plain_value(),
        params.div12t_beat10.modulated_plain_value(),
        params.div12t_beat11.modulated_plain_value(),
        params.div12t_beat12.modulated_plain_value(),
    ];

    data.triplet_1_16t = [
        params.div24t_beat1.modulated_plain_value(),
        params.div24t_beat2.modulated_plain_value(),
        params.div24t_beat3.modulated_plain_value(),
        params.div24t_beat4.modulated_plain_value(),
        params.div24t_beat5.modulated_plain_value(),
        params.div24t_beat6.modulated_plain_value(),
        params.div24t_beat7.modulated_plain_value(),
        params.div24t_beat8.modulated_plain_value(),
        params.div24t_beat9.modulated_plain_value(),
        params.div24t_beat10.modulated_plain_value(),
        params.div24t_beat11.modulated_plain_value(),
        params.div24t_beat12.modulated_plain_value(),
        params.div24t_beat13.modulated_plain_value(),
        params.div24t_beat14.modulated_plain_value(),
        params.div24t_beat15.modulated_plain_value(),
        params.div24t_beat16.modulated_plain_value(),
        params.div24t_beat17.modulated_plain_value(),
        params.div24t_beat18.modulated_plain_value(),
        params.div24t_beat19.modulated_plain_value(),
        params.div24t_beat20.modulated_plain_value(),
        params.div24t_beat21.modulated_plain_value(),
        params.div24t_beat22.modulated_plain_value(),
        params.div24t_beat23.modulated_plain_value(),
        params.div24t_beat24.modulated_plain_value(),
    ];

    data.dotted_1_2d = [
        params.div2d_beat1.modulated_plain_value(),
        params.div2d_beat2.modulated_plain_value(),
    ];

    data.dotted_1_4d = [
        params.div3d_beat1.modulated_plain_value(),
        params.div3d_beat2.modulated_plain_value(),
        params.div3d_beat3.modulated_plain_value(),
    ];

    data.dotted_1_8d = [
        params.div6d_beat1.modulated_plain_value(),
        params.div6d_beat2.modulated_plain_value(),
        params.div6d_beat3.modulated_plain_value(),
        params.div6d_beat4.modulated_plain_value(),
        params.div6d_beat5.modulated_plain_value(),
        params.div6d_beat6.modulated_plain_value(),
    ];

    data.dotted_1_16d = [
        params.div11d_beat1.modulated_plain_value(),
        params.div11d_beat2.modulated_plain_value(),
        params.div11d_beat3.modulated_plain_value(),
        params.div11d_beat4.modulated_plain_value(),
        params.div11d_beat5.modulated_plain_value(),
        params.div11d_beat6.modulated_plain_value(),
        params.div11d_beat7.modulated_plain_value(),
        params.div11d_beat8.modulated_plain_value(),
        params.div11d_beat9.modulated_plain_value(),
        params.div11d_beat10.modulated_plain_value(),
        params.div11d_beat11.modulated_plain_value(),
    ];

    data.dotted_1_32d = [
        params.div22d_beat1.modulated_plain_value(),
        params.div22d_beat2.modulated_plain_value(),
        params.div22d_beat3.modulated_plain_value(),
        params.div22d_beat4.modulated_plain_value(),
        params.div22d_beat5.modulated_plain_value(),
        params.div22d_beat6.modulated_plain_value(),
        params.div22d_beat7.modulated_plain_value(),
        params.div22d_beat8.modulated_plain_value(),
        params.div22d_beat9.modulated_plain_value(),
        params.div22d_beat10.modulated_plain_value(),
        params.div22d_beat11.modulated_plain_value(),
        params.div22d_beat12.modulated_plain_value(),
        params.div22d_beat13.modulated_plain_value(),
        params.div22d_beat14.modulated_plain_value(),
        params.div22d_beat15.modulated_plain_value(),
        params.div22d_beat16.modulated_plain_value(),
        params.div22d_beat17.modulated_plain_value(),
        params.div22d_beat18.modulated_plain_value(),
        params.div22d_beat19.modulated_plain_value(),
        params.div22d_beat20.modulated_plain_value(),
        params.div22d_beat21.modulated_plain_value(),
        params.div22d_beat22.modulated_plain_value(),
    ];

    data.synth_pll_track_speed = params.synth_pll_track_speed.modulated_plain_value();
    data.synth_pll_damping = params.synth_pll_damping.modulated_plain_value();
    data.synth_pll_influence = params.synth_pll_influence.modulated_plain_value();
    data.synth_pll_mult = params.synth_pll_mult.value();
    data.synth_pll_colored = params.synth_pll_colored.value();
    data.synth_pll_mode = params.synth_pll_mode.value();
    data.synth_pll_ref_octave = params.synth_pll_ref_octave.value();
    data.synth_pll_ref_tune = params.synth_pll_ref_tune.value();
    data.synth_pll_ref_fine = params.synth_pll_ref_fine.modulated_plain_value();
    data.synth_pll_ref_pulse_width = params.synth_pll_ref_pulse_width.modulated_plain_value();
    data.synth_pll_feedback = params.synth_pll_feedback.modulated_plain_value();
    data.synth_pll_volume = params.synth_pll_volume.modulated_plain_value();
    data.synth_pll_stereo_damp_offset = params.synth_pll_stereo_damp_offset.modulated_plain_value();
    data.synth_pll_glide = params.synth_pll_glide.modulated_plain_value();
    data.synth_pll_fm_amount = params.synth_pll_fm_amount.modulated_plain_value();
    data.synth_pll_fm_ratio = params.synth_pll_fm_ratio.value();
    data.synth_pll_retrigger = params.synth_pll_retrigger.modulated_plain_value();
    data.synth_pll_burst_threshold = params.synth_pll_burst_threshold.modulated_plain_value();
    data.synth_pll_burst_amount = params.synth_pll_burst_amount.modulated_plain_value();
    data.synth_pll_loop_saturation = params.synth_pll_loop_saturation.modulated_plain_value();
    data.synth_pll_color_amount = params.synth_pll_color_amount.modulated_plain_value();
    data.synth_pll_edge_sensitivity = params.synth_pll_edge_sensitivity.modulated_plain_value();
    data.synth_pll_range = params.synth_pll_range.modulated_plain_value();
    data.synth_pll_stereo_track_offset = params.synth_pll_stereo_track_offset.modulated_plain_value();
    data.synth_pll_stereo_phase = params.synth_pll_stereo_phase.modulated_plain_value();
    data.synth_pll_cross_feedback = params.synth_pll_cross_feedback.modulated_plain_value();
    data.synth_pll_fm_env_amount = params.synth_pll_fm_env_amount.modulated_plain_value();
    data.synth_pll_enable = params.synth_pll_enable.value();

    data.synth_osc_octave = params.synth_osc_octave.value();
    data.synth_osc_tune = params.synth_osc_tune.value();
    data.synth_osc_fine = params.synth_osc_fine.modulated_plain_value();
    data.synth_osc_fold = params.synth_osc_fold.modulated_plain_value();
    data.synth_osc_d = params.synth_osc_d.modulated_plain_value();
    data.synth_osc_v = params.synth_osc_v.modulated_plain_value();
    data.synth_osc_stereo_v_offset = params.synth_osc_stereo_v_offset.modulated_plain_value();
    data.synth_osc_volume = params.synth_osc_volume.modulated_plain_value();

    data.synth_sub_volume = params.synth_sub_volume.modulated_plain_value();

    data.synth_filter_enable = params.synth_filter_enable.value();
    data.synth_filter_cutoff = params.synth_filter_cutoff.modulated_plain_value();
    data.synth_filter_resonance = params.synth_filter_resonance.modulated_plain_value();
    data.synth_filter_env_amount = params.synth_filter_env_amount.modulated_plain_value();
    data.synth_filter_drive = params.synth_filter_drive.modulated_plain_value();

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
    data.synth_reverb_ducking = params.synth_reverb_ducking.modulated_plain_value();

    data.lfo1_rate = params.lfo1_rate.modulated_plain_value();
    data.lfo1_waveform = params.lfo1_waveform.value();
    data.lfo1_tempo_sync = params.lfo1_tempo_sync.value();
    data.lfo1_sync_division = params.lfo1_sync_division.value();
    data.lfo1_sync_source = params.lfo1_sync_source.value();
    data.lfo1_phase_mod = params.lfo1_phase_mod.modulated_plain_value();
    data.lfo1_dest1 = params.lfo1_dest1.value();
    data.lfo1_amount1 = params.lfo1_amount1.modulated_plain_value();
    data.lfo1_dest2 = params.lfo1_dest2.value();
    data.lfo1_amount2 = params.lfo1_amount2.modulated_plain_value();

    data.lfo2_rate = params.lfo2_rate.modulated_plain_value();
    data.lfo2_waveform = params.lfo2_waveform.value();
    data.lfo2_tempo_sync = params.lfo2_tempo_sync.value();
    data.lfo2_sync_division = params.lfo2_sync_division.value();
    data.lfo2_sync_source = params.lfo2_sync_source.value();
    data.lfo2_phase_mod = params.lfo2_phase_mod.modulated_plain_value();
    data.lfo2_dest1 = params.lfo2_dest1.value();
    data.lfo2_amount1 = params.lfo2_amount1.modulated_plain_value();
    data.lfo2_dest2 = params.lfo2_dest2.value();
    data.lfo2_amount2 = params.lfo2_amount2.modulated_plain_value();

    data.lfo3_rate = params.lfo3_rate.modulated_plain_value();
    data.lfo3_waveform = params.lfo3_waveform.value();
    data.lfo3_tempo_sync = params.lfo3_tempo_sync.value();
    data.lfo3_sync_division = params.lfo3_sync_division.value();
    data.lfo3_sync_source = params.lfo3_sync_source.value();
    data.lfo3_phase_mod = params.lfo3_phase_mod.modulated_plain_value();
    data.lfo3_dest1 = params.lfo3_dest1.value();
    data.lfo3_amount1 = params.lfo3_amount1.modulated_plain_value();
    data.lfo3_dest2 = params.lfo3_dest2.value();
    data.lfo3_amount2 = params.lfo3_amount2.modulated_plain_value();

    data.swing_amount = params.swing_amount.modulated_plain_value();
    data.note_length_percent = params.note_length_percent.modulated_plain_value();

    data.legato_mode = params.legato_mode.value();
    data.legato_time = params.legato_time.modulated_plain_value();

    data.len_mod_1_target = params.len_mod_1_target.modulated_plain_value();
    data.len_mod_1_amount = params.len_mod_1_amount.modulated_plain_value();
    data.len_mod_1_prob = params.len_mod_1_prob.modulated_plain_value();

    data.len_mod_2_target = params.len_mod_2_target.modulated_plain_value();
    data.len_mod_2_amount = params.len_mod_2_amount.modulated_plain_value();
    data.len_mod_2_prob = params.len_mod_2_prob.modulated_plain_value();

    data.vel_strength_target = params.vel_strength_target.modulated_plain_value();
    data.vel_strength_amount = params.vel_strength_amount.modulated_plain_value();
    data.vel_strength_prob = params.vel_strength_prob.modulated_plain_value();
    data.vel_length_target = params.vel_length_target.modulated_plain_value();
    data.vel_length_amount = params.vel_length_amount.modulated_plain_value();
    data.vel_length_prob = params.vel_length_prob.modulated_plain_value();

    data.pos_mod_1_target = params.pos_mod_1_target.modulated_plain_value();
    data.pos_mod_1_shift = params.pos_mod_1_shift.modulated_plain_value();
    data.pos_mod_1_prob = params.pos_mod_1_prob.modulated_plain_value();

    data.pos_mod_2_target = params.pos_mod_2_target.modulated_plain_value();
    data.pos_mod_2_shift = params.pos_mod_2_shift.modulated_plain_value();
    data.pos_mod_2_prob = params.pos_mod_2_prob.modulated_plain_value();

    data.synth_ring_mod = params.synth_ring_mod.modulated_plain_value();
    data.synth_wavefold = params.synth_wavefold.modulated_plain_value();
    data.synth_drift_amount = params.synth_drift_amount.modulated_plain_value();
    data.synth_drift_rate = params.synth_drift_rate.modulated_plain_value();
    data.synth_noise_amount = 0.0;
    data.synth_tube_drive = params.synth_tube_drive.modulated_plain_value();
    data.synth_color_distortion_amount = 0.0;
    data.synth_color_distortion_threshold = 0.7;
    data.synth_vps_enable = params.synth_vps_enable.value();

    if let Ok(strength_values) = ui_state.strength_values.lock() {
        for (i, &v) in strength_values.iter().enumerate() {
            if i < 96 {
                data.strength_values[i] = (v * 100.0).clamp(0.0, 100.0) as u8;
            }
        }
    }

    if let Ok(note_pool) = ui_state.note_pool.lock() {
        data.root_note = note_pool.root_note.unwrap_or(48);
        data.notes = note_pool.notes.iter()
            .map(|n| crate::preset::NotePresetData {
                midi_note: n.midi_note,
                chance: (n.chance * 127.0).clamp(0.0, 127.0) as u8,
                beat: ((n.strength_bias * 63.0) + 64.0).clamp(0.0, 127.0) as u8,
                beat_length: ((n.length_bias * 63.0) + 64.0).clamp(0.0, 127.0) as u8,
                octave_offset: n.octave_offset,
            })
            .collect();
    }

    if let Ok(scale) = ui_state.scale.lock() {
        data.scale = *scale;
    }

    if let Ok(pattern) = ui_state.stability_pattern.lock() {
        data.stability_pattern = *pattern;
    }

    if let Ok(oct_rand) = ui_state.octave_randomization.lock() {
        data.octave_randomization = crate::preset::OctaveRandomizationPresetData {
            chance: oct_rand.chance,
            strength_pref: oct_rand.strength_pref,
            length_pref: oct_rand.length_pref,
            direction: oct_rand.direction,
        };
    }

    data
}
