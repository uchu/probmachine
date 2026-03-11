#![allow(clippy::too_many_arguments)]

use crate::params::DeviceParams;
use crate::ui::SharedUiState;
use crate::midi_learn::{MidiLearnState, SOUND_PARAMS};
use egui_taffy::taffy::{prelude::*, style::{AlignItems, FlexDirection, Overflow}, geometry::Point};
use egui_taffy::TuiBuilderLogic;
use nih_plug::prelude::{BoolParam, Param, ParamSetter};
use nih_plug_egui::egui;
use nih_plug_egui::egui::Color32;
use std::f32::consts::FRAC_PI_2;
use std::sync::Arc;
use std::sync::atomic::Ordering;

const SLIDER_COL_WIDTH: f32 = 58.0;
const SLIDER_RAIL_LENGTH: f32 = 225.0;
const SLIDER_RAIL_THICKNESS: f32 = 18.0;
const LABEL_FONT: f32 = 19.0;
const HEADER_FONT: f32 = 18.0;
const TAB_FONT: f32 = 16.0;
const TAB_BAR_WIDTH: f32 = 52.0;
const FRAME_MARGIN: egui::Margin = egui::Margin { left: 32, right: 4, top: 14, bottom: 14 };
const SELECTED_LABEL_COLOR: Color32 = Color32::from_rgb(255, 180, 60);
const MIDI_ASSIGNED_COLOR: Color32 = Color32::from_rgb(80, 140, 220);
const LFO_MOD_INDICATOR_COLOR: Color32 = Color32::from_rgb(60, 200, 180);

pub fn render(
    tui: &mut egui_taffy::Tui,
    params: &Arc<DeviceParams>,
    setter: &ParamSetter,
    ui_state: &Arc<SharedUiState>,
) {
    let current_tab = tui.ui(|ui| {
        ui.memory_mut(|mem| {
            *mem.data.get_temp_mut_or(egui::Id::new("synth_tab"), 0u8)
        })
    });

    tui.style(Style {
        flex_direction: FlexDirection::Row,
        flex_grow: 1.0,
        align_items: Some(AlignItems::Stretch),
        size: Size {
            width: auto(),
            height: percent(1.0),
        },
        overflow: Point {
            x: Overflow::Visible,
            y: Overflow::Visible,
        },
        margin: egui_taffy::taffy::Rect {
            left: length(-29.0),
            right: length(0.0),
            top: length(0.0),
            bottom: length(0.0),
        },
        ..Default::default()
    })
    .add(|tui| {
        tui.style(Style {
            size: Size {
                width: length(TAB_BAR_WIDTH),
                height: percent(1.0),
            },
            flex_shrink: 0.0,
            ..Default::default()
        })
        .ui(|ui| {
            render_tab_bar(ui, current_tab);
        });

        tui.style(Style {
            flex_grow: 1.0,
            flex_shrink: 0.0,
            size: Size {
                width: auto(),
                height: percent(1.0),
            },
            overflow: Point {
                x: Overflow::Hidden,
                y: Overflow::Visible,
            },
            padding: egui_taffy::taffy::Rect {
                left: length(8.0),
                right: length(0.0),
                top: length(8.0),
                bottom: length(0.0),
            },
            ..Default::default()
        })
        .ui(|ui| {
            match current_tab {
                0 => render_sound_tab(ui, params, setter, ui_state),
                1 => render_vol_env_tab(ui, params, setter, ui_state),
                2 => render_filt_env_tab(ui, params, setter, ui_state),
                3 => render_filter_tab(ui, params, setter, ui_state),
                4 => render_fx_tab(ui, params, setter),
                5 => render_lush_tab(ui, params, setter),
                6 => render_comp_tab(ui, params, setter, ui_state),
                7 => super::modulation::render_ui(ui, params, setter),
                _ => super::modulation::render_step_mod_ui(ui, params, setter, ui_state),
            }
        });
    });
}

const TAB_HEIGHT: f32 = 67.0;
const TAB_GAP: f32 = 3.0;

fn render_tab_bar(ui: &mut egui::Ui, current_tab: u8) {
    let rect = ui.max_rect();
    let tab_names = ["OSCs", "VOLENV", "FLTENV", "FILTER", "FX", "LUSH", "COMP", "LFOs", "STEP"];

    for (i, name) in tab_names.iter().enumerate() {
        let y = rect.min.y + i as f32 * (TAB_HEIGHT + TAB_GAP);
        let button_rect = egui::Rect::from_min_size(
            egui::pos2(rect.min.x, y),
            egui::vec2(TAB_BAR_WIDTH, TAB_HEIGHT),
        );

        let response = ui.interact(
            button_rect,
            egui::Id::new("synth_tab_btn").with(i),
            egui::Sense::click(),
        );
        let is_selected = current_tab == i as u8;
        let is_hovered = response.hovered();

        let bg_color = if is_selected {
            Color32::from_rgb(55, 55, 65)
        } else if is_hovered {
            Color32::from_rgb(45, 45, 55)
        } else {
            Color32::from_rgb(30, 30, 38)
        };
        ui.painter().rect_filled(button_rect, 0.0, bg_color);

        if is_selected {
            let accent_rect = egui::Rect::from_min_size(
                button_rect.left_top(),
                egui::vec2(3.0, TAB_HEIGHT),
            );
            ui.painter()
                .rect_filled(accent_rect, 1.5, Color32::from_rgb(100, 140, 200));
        }

        let font_id = egui::FontId::proportional(TAB_FONT);
        let text_color = if is_selected {
            Color32::WHITE
        } else {
            Color32::from_gray(150)
        };
        let galley =
            ui.painter()
                .layout_no_wrap(name.to_string(), font_id, text_color);
        let tw = galley.size().x;
        let th = galley.size().y;
        let text_pos = egui::pos2(
            button_rect.center().x - th / 2.0,
            button_rect.center().y + tw / 2.0,
        );

        let mut text_shape = egui::epaint::TextShape::new(text_pos, galley, text_color);
        text_shape.angle = -FRAC_PI_2;
        ui.painter().add(text_shape);

        if response.clicked() {
            ui.memory_mut(|mem| {
                mem.data.insert_temp(egui::Id::new("synth_tab"), i as u8);
            });
        }
    }
}

fn render_sound_tab(
    ui: &mut egui::Ui,
    params: &Arc<DeviceParams>,
    setter: &ParamSetter,
    ui_state: &Arc<SharedUiState>,
) {
    let ml = &*ui_state.midi_learn;
    macro_rules! ml {
        ($id:expr) => { Some((ml, $id)) };
    }

    let pll_top = ui.cursor().top();
    let pll_resp = egui::Frame::NONE
        .outer_margin(egui::Margin { top: -7, bottom: -7, left: 0, right: 0 })
        .inner_margin(FRAME_MARGIN)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                egui::Frame::NONE.inner_margin(egui::Margin { left: 0, right: 0, top: 0, bottom: 0 })
                    .show(ui, |ui| {
                        ui.label(
                            egui::RichText::new("PLL OSC")
                                .size(HEADER_FONT)
                                .strong(),
                        );
                    });
                ui.add_space(175.0);
                egui::Frame::NONE.inner_margin(egui::Margin { left: 0, right: 0, top: 2, bottom: 0 })
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            let mut colored = params.synth_pll_colored.value();
                            render_toggle(ui, &mut colored, "COLOR", ml!("synth_pll_colored"));
                            if colored != params.synth_pll_colored.value() {
                                setter.set_parameter(&params.synth_pll_colored, colored);
                            }
                            ui.add_space(60.0);
                            let mut edge_mode = params.synth_pll_mode.value();
                            render_toggle(ui, &mut edge_mode, "EDGE", ml!("synth_pll_mode"));
                            if edge_mode != params.synth_pll_mode.value() {
                                setter.set_parameter(&params.synth_pll_mode, edge_mode);
                            }
                            ui.add_space(60.0);
                            let mut precision = params.synth_pll_precision.value();
                            render_toggle(ui, &mut precision, "TIGHT", ml!("synth_pll_precision"));
                            if precision != params.synth_pll_precision.value() {
                                setter.set_parameter(&params.synth_pll_precision, precision);
                            }
                            ui.add_space(60.0);
                            let mut injection_x4 = params.synth_pll_injection_x4.value();
                            let inj_label = if injection_x4 { "INJ4" } else { "INJ2" };
                            render_toggle(ui, &mut injection_x4, inj_label, ml!("synth_pll_injection_x4"));
                            if injection_x4 != params.synth_pll_injection_x4.value() {
                                setter.set_parameter(&params.synth_pll_injection_x4, injection_x4);
                            }


                        });
                    });
            });
            ui.add_space(10.0);
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 5.0;
                render_int_vertical_slider(
                    ui, params, setter,
                    &params.synth_pll_ref_octave, "OCT",
                    Some(Color32::from_rgb(80, 80, 40)),
                    None, None, ml!("synth_pll_ref_octave"),
                );
                render_int_vertical_slider(
                    ui, params, setter,
                    &params.synth_pll_ref_tune, "TUNE",
                    Some(Color32::from_rgb(80, 80, 40)),
                    Some(&[-12, 0, 12]), None, ml!("synth_pll_ref_tune"),
                );
                render_int_vertical_slider(
                    ui, params, setter,
                    &params.synth_pll_mult, "MULT",
                    Some(Color32::from_rgb(40, 40, 80)),
                    None,
                    Some(&["1", "2", "4", "8", "16", "32", "64"]),
                    ml!("synth_pll_mult"),
                );
                render_vertical_slider(
                    ui, params, setter,
                    &params.synth_pll_mult_slew_time, "SLEW",
                    0.0, 1.0, SliderScale::Linear,
                    Some(Color32::from_rgb(40, 40, 80)),
                    ml!("synth_pll_mult_slew_time"),
                );
                render_vertical_slider(
                    ui, params, setter,
                    &params.synth_pll_track_speed, "TRK",
                    0.0, 1.0, SliderScale::Linear,
                    Some(Color32::from_rgb(40, 40, 80)),
                    ml!("synth_pll_track_speed"),
                );
                render_vertical_slider(
                    ui, params, setter,
                    &params.synth_pll_damping, "DAMP",
                    0.0, 1.0, SliderScale::Linear,
                    Some(Color32::from_rgb(40, 40, 80)),
                    ml!("synth_pll_damping"),
                );
                render_vertical_slider(
                    ui, params, setter,
                    &params.synth_pll_influence, "INF",
                    0.0, 1.0, SliderScale::Linear,
                    Some(Color32::from_rgb(40, 40, 80)),
                    ml!("synth_pll_influence"),
                );
                render_vertical_slider(
                    ui, params, setter,
                    &params.synth_pll_stereo_damp_offset, "STΔ",
                    0.0, 0.5, SliderScale::Linear,
                    Some(Color32::from_rgb(80, 40, 80)),
                    ml!("synth_pll_stereo_damp_offset"),
                );
                render_vertical_slider(
                    ui, params, setter,
                    &params.synth_pll_burst_amount, "OT",
                    0.0, 10.0, SliderScale::Linear,
                    Some(Color32::from_rgb(100, 80, 60)),
                    ml!("synth_pll_burst_amount"),
                );
                render_vertical_slider(
                    ui, params, setter,
                    &params.synth_pll_color_amount, "SAT",
                    0.0, 1.0, SliderScale::Linear,
                    Some(Color32::from_rgb(60, 80, 100)),
                    ml!("synth_pll_color_amount"),
                );
                render_vertical_slider(
                    ui, params, setter,
                    &params.synth_pll_range, "RNG",
                    0.0, 1.0, SliderScale::Linear,
                    Some(Color32::from_rgb(60, 100, 80)),
                    ml!("synth_pll_range"),
                );
                render_vertical_slider(
                    ui, params, setter,
                    &params.synth_pll_fm_amount, "FM",
                    0.0, 1.0, SliderScale::Logarithmic,
                    Some(Color32::from_rgb(100, 60, 100)),
                    ml!("synth_pll_fm_amount"),
                );
                render_vertical_slider(
                    ui, params, setter,
                    &params.synth_pll_fm_ratio_float, "RAT",
                    0.5, 16.0, SliderScale::Linear,
                    Some(Color32::from_rgb(100, 60, 100)),
                    ml!("synth_pll_fm_ratio_float"),
                );
                render_vertical_slider(
                    ui, params, setter,
                    &params.synth_pll_injection_amount, "INJ",
                    0.0, 1.0, SliderScale::Linear,
                    Some(Color32::from_rgb(80, 100, 140)),
                    ml!("synth_pll_injection_amount"),
                );
                render_vertical_slider(
                    ui, params, setter,
                    &params.synth_tube_drive, "TUBE",
                    0.0, 1.0, SliderScale::Linear,
                    Some(Color32::from_rgb(140, 80, 80)),
                    ml!("synth_tube_drive"),
                );
                render_vertical_slider(
                    ui, params, setter,
                    &params.synth_drift_amount, "DRFT",
                    0.0, 1.0, SliderScale::Linear,
                    Some(Color32::from_rgb(80, 100, 80)),
                    ml!("synth_drift_amount"),
                );
                render_vertical_slider(
                    ui, params, setter,
                    &params.synth_drift_rate, "RATE",
                    0.1, 10.0, SliderScale::Logarithmic,
                    Some(Color32::from_rgb(80, 100, 80)),
                    ml!("synth_drift_rate"),
                );
                render_vertical_slider(
                    ui, params, setter,
                    &params.synth_pll_volume, "VOL",
                    0.0, 1.0, SliderScale::Linear,
                    Some(Color32::from_rgb(40, 80, 40)),
                    ml!("synth_pll_volume"),
                );
            });
        });
    {
        let pll_rect = pll_resp.response.rect;
        let bg_painter = ui.painter().clone().with_layer_id(
            egui::LayerId::new(egui::Order::Background, egui::Id::new("pll_section_bg")),
        );
        bg_painter.rect_filled(
            egui::Rect::from_min_max(
                egui::pos2(pll_rect.left() - 50.0, pll_top - 7.0),
                egui::pos2(pll_rect.right(), pll_rect.bottom() + 7.0),
            ),
            0.0,
            Color32::from_rgba_premultiplied(4, 3, 0, 3),
        );
    }

    ui.add_space(5.0);
    let sep_rect = ui.available_rect_before_wrap();
    ui.painter().line_segment(
        [egui::pos2(sep_rect.left(), sep_rect.top()), egui::pos2(sep_rect.right() - 20.0, sep_rect.top())],
        egui::Stroke::new(1.0, Color32::BLACK),
    );
    ui.add_space(5.0);

    let _row_top = ui.cursor().top();
    ui.horizontal(|ui| {
        egui::Frame::NONE
            .inner_margin(egui::Margin { left: FRAME_MARGIN.left + 13, right: 5, ..FRAME_MARGIN })
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.add_space(-12.0);
                        ui.label(egui::RichText::new("SUB").size(HEADER_FONT).strong());
                    });
                    ui.add_space(13.0);
                    ui.horizontal(|ui| {
                        render_vertical_slider(
                            ui, params, setter,
                            &params.synth_sub_volume, "VOL",
                            0.0, 1.0, SliderScale::Linear,
                            Some(Color32::from_rgb(40, 80, 40)),
                            ml!("synth_sub_volume"),
                        );
                    });
                });
            });

        let line_rect = ui.available_rect_before_wrap();

        egui::Frame::NONE
            .inner_margin(egui::Margin { left: 30, right: 0, ..FRAME_MARGIN })
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("SAW").size(HEADER_FONT).strong());
                        ui.add_space(16.0);
                        let mut saw_fold_pi = params.synth_saw_fold_range.value() == 1;
                        render_labeled_toggle(ui, &mut saw_fold_pi, "1X", "PI");
                        let new_range = if saw_fold_pi { 1 } else { 0 };
                        if new_range != params.synth_saw_fold_range.value() {
                            setter.set_parameter(&params.synth_saw_fold_range, new_range);
                        }
                    });
                    ui.add_space(10.0);
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 5.0;
                        render_int_vertical_slider(
                            ui, params, setter,
                            &params.synth_saw_octave, "OCT",
                            Some(Color32::from_rgb(80, 80, 40)),
                            None, None, ml!("synth_saw_octave"),
                        );
                        render_int_vertical_slider(
                            ui, params, setter,
                            &params.synth_saw_tune, "TUNE",
                            Some(Color32::from_rgb(80, 80, 40)),
                            Some(&[-12, 0, 12]), None, ml!("synth_saw_tune"),
                        );
                        render_vertical_slider(
                            ui, params, setter,
                            &params.synth_saw_fine, "FINE",
                            -1.0, 1.0, SliderScale::Linear,
                            Some(Color32::from_rgb(80, 80, 40)),
                            ml!("synth_saw_fine"),
                        );
                        render_vertical_slider(
                            ui, params, setter,
                            &params.synth_saw_fold, "FOLD",
                            0.0, 1.0, SliderScale::Linear,
                            Some(Color32::from_rgb(120, 80, 60)),
                            ml!("synth_saw_fold"),
                        );
                        render_vertical_slider_with_ticks(
                            ui, params, setter,
                            &params.synth_saw_tight, "TGHT",
                            0.0, 1.0, SliderScale::Linear,
                            Some(Color32::from_rgb(100, 70, 50)),
                            &[(0.22, "10"), (0.56, "30"), (0.78, "60"), (1.0, "120")],
                            ml!("synth_saw_tight"),
                        );
                        render_int_vertical_slider(
                            ui, params, setter,
                            &params.synth_saw_shape_type, "SHPE",
                            Some(Color32::from_rgb(100, 80, 100)),
                            None, Some(&["SOFT", "BRAM", "FOLD"]), ml!("synth_saw_shape_type"),
                        );
                        render_vertical_slider(
                            ui, params, setter,
                            &params.synth_saw_shape_amount, "SHP",
                            0.0, 1.0, SliderScale::Linear,
                            Some(Color32::from_rgb(100, 80, 100)),
                            ml!("synth_saw_shape_amount"),
                        );
                        render_vertical_slider(
                            ui, params, setter,
                            &params.synth_saw_volume, "VOL",
                            0.0, 1.0, SliderScale::Linear,
                            Some(Color32::from_rgb(40, 80, 40)),
                            ml!("synth_saw_volume"),
                        );
                    });
                });
            });

        let line_rect2 = ui.available_rect_before_wrap();

        egui::Frame::NONE
            .inner_margin(egui::Margin { left: 40, right: -10, ..FRAME_MARGIN })
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        egui::Frame::NONE.inner_margin(egui::Margin { left: 0, right: 0, top: 0, bottom: 0 })
                            .show(ui, |ui| {
                                ui.label(
                                    egui::RichText::new("VPS")
                                        .size(HEADER_FONT)
                                        .strong(),
                                );
                            });
                        ui.add_space(106.0);
                        egui::Frame::NONE.inner_margin(egui::Margin { left: 0, right: 0, top: 2, bottom: 0 })
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    let mut vps_fold_pi = params.synth_vps_fold_range.value() == 1;
                                    render_labeled_toggle(ui, &mut vps_fold_pi, "1X", "PI");
                                    let new_range = if vps_fold_pi { 1 } else { 0 };
                                    if new_range != params.synth_vps_fold_range.value() {
                                        setter.set_parameter(&params.synth_vps_fold_range, new_range);
                                    }
                                    ui.add_space(60.0);
                                    let mut formant = params.synth_vps_formant.value();
                                    render_toggle(ui, &mut formant, "FMT", ml!("synth_vps_formant"));
                                    if formant != params.synth_vps_formant.value() {
                                        setter.set_parameter(&params.synth_vps_formant, formant);
                                    }
                                });
                            });
                    });
                    ui.add_space(10.0);
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 5.0;
                        render_int_vertical_slider(
                            ui, params, setter,
                            &params.synth_osc_octave, "OCT",
                            Some(Color32::from_rgb(80, 80, 40)),
                            None, None, ml!("synth_osc_octave"),
                        );
                        render_int_vertical_slider(
                            ui, params, setter,
                            &params.synth_osc_tune, "TUNE",
                            Some(Color32::from_rgb(80, 80, 40)),
                            Some(&[-12, 0, 12]), None, ml!("synth_osc_tune"),
                        );
                        render_vertical_slider(
                            ui, params, setter,
                            &params.synth_osc_d, "D",
                            0.0, 1.0, SliderScale::Linear,
                            Some(Color32::from_rgb(40, 40, 80)),
                            ml!("synth_osc_d"),
                        );
                        render_vertical_slider(
                            ui, params, setter,
                            &params.synth_osc_v, "V",
                            0.0, 1.0, SliderScale::Linear,
                            Some(Color32::from_rgb(40, 40, 80)),
                            ml!("synth_osc_v"),
                        );
                        render_vertical_slider(
                            ui, params, setter,
                            &params.synth_osc_stereo_v_offset, "VΔ",
                            0.0, 0.3, SliderScale::Linear,
                            Some(Color32::from_rgb(80, 40, 80)),
                            ml!("synth_osc_stereo_v_offset"),
                        );
                        render_vertical_slider(
                            ui, params, setter,
                            &params.synth_osc_stereo_d_offset, "DΔ",
                            0.0, 0.3, SliderScale::Linear,
                            Some(Color32::from_rgb(80, 40, 80)),
                            ml!("synth_osc_stereo_d_offset"),
                        );
                        render_vertical_slider(
                            ui, params, setter,
                            &params.synth_osc_fold, "FOLD",
                            0.0, 1.0, SliderScale::Linear,
                            Some(Color32::from_rgb(120, 80, 60)),
                            ml!("synth_osc_fold"),
                        );
                        render_int_vertical_slider(
                            ui, params, setter,
                            &params.synth_vps_shape_type, "SHPE",
                            Some(Color32::from_rgb(100, 80, 100)),
                            None, Some(&["SOFT", "BRAM", "FOLD"]), ml!("synth_vps_shape_type"),
                        );
                        render_vertical_slider(
                            ui, params, setter,
                            &params.synth_vps_shape_amount, "SHP",
                            0.0, 0.5, SliderScale::Linear,
                            Some(Color32::from_rgb(120, 80, 60)),
                            ml!("synth_vps_shape_amount"),
                        );
                        render_vertical_slider(
                            ui, params, setter,
                            &params.synth_osc_volume, "VOL",
                            0.0, 1.0, SliderScale::Linear,
                            Some(Color32::from_rgb(40, 80, 40)),
                            ml!("synth_osc_volume"),
                        );
                    });
                });
            });

        let row_rect = ui.max_rect();
        let bottom = row_rect.bottom() + 25.0;
        let sep1_x = line_rect.left() - 15.0;
        let sep2_x = line_rect2.left() - 5.0;

        let bg_painter = ui.painter().clone().with_layer_id(
            egui::LayerId::new(egui::Order::Background, egui::Id::new("sound_section_bg")),
        );
        bg_painter.rect_filled(
            egui::Rect::from_min_max(egui::pos2(row_rect.left() - 50.0, sep_rect.top()), egui::pos2(sep1_x, bottom)),
            0.0,
            Color32::from_rgba_premultiplied(6, 0, 0, 5),
        );
        bg_painter.rect_filled(
            egui::Rect::from_min_max(egui::pos2(sep1_x, sep_rect.top()), egui::pos2(sep2_x, bottom)),
            0.0,
            Color32::from_rgba_premultiplied(0, 3, 0, 2),
        );
        bg_painter.rect_filled(
            egui::Rect::from_min_max(egui::pos2(sep2_x, sep_rect.top()), egui::pos2(row_rect.right(), bottom)),
            0.0,
            Color32::from_rgba_premultiplied(1, 1, 8, 7),
        );

        ui.painter().line_segment(
            [egui::pos2(sep1_x, sep_rect.top()), egui::pos2(sep1_x, bottom - 5.0)],
            egui::Stroke::new(1.0, Color32::BLACK),
        );
        ui.painter().line_segment(
            [egui::pos2(sep2_x, sep_rect.top()), egui::pos2(sep2_x, bottom - 5.0)],
            egui::Stroke::new(1.0, Color32::BLACK),
        );
    });
}

fn render_filter_tab(
    ui: &mut egui::Ui,
    params: &Arc<DeviceParams>,
    setter: &ParamSetter,
    _ui_state: &Arc<SharedUiState>,
) {
    let content_rect = ui.max_rect();
    let half_w = content_rect.width() / 2.0;
    let sep_x = content_rect.left() + half_w;
    let margin = FRAME_MARGIN;

    let left_rect = egui::Rect::from_min_max(
        egui::pos2(content_rect.left() + margin.left as f32 + 5.0, content_rect.top() + margin.top as f32),
        egui::pos2(sep_x - 10.0, content_rect.bottom()),
    );
    let mut left_ui = ui.new_child(egui::UiBuilder::new().max_rect(left_rect));
    left_ui.vertical(|ui| {
        ui.label(egui::RichText::new("FILTER").size(HEADER_FONT).strong());
        ui.add_space(9.0);
        {
            let filter_on = params.synth_filter_enable.value();
            let btn_w = 80.0;
            let btn_h = 48.0;
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 5.0;
                for (label, active) in &[("OFF", false), ("ON", true)] {
                    let is_selected = filter_on == *active;
                    let (bg, text_col) = if is_selected {
                        if *active {
                            (Color32::from_rgb(80, 160, 80), Color32::WHITE)
                        } else {
                            (Color32::from_rgb(180, 60, 60), Color32::WHITE)
                        }
                    } else {
                        (Color32::from_rgb(40, 40, 48), Color32::from_gray(160))
                    };
                    let (rect, response) = ui.allocate_exact_size(
                        egui::vec2(btn_w, btn_h),
                        egui::Sense::click(),
                    );
                    let hover_bg = if response.hovered() && !is_selected {
                        Color32::from_rgb(55, 55, 65)
                    } else {
                        bg
                    };
                    ui.painter().rect_filled(rect, 4.0, hover_bg);
                    if is_selected {
                        let stroke_col = if *active {
                            Color32::from_rgb(100, 190, 100)
                        } else {
                            Color32::from_rgb(210, 80, 80)
                        };
                        ui.painter().rect_stroke(rect, 4.0, egui::Stroke::new(2.0, stroke_col), egui::epaint::StrokeKind::Inside);
                    }
                    let font = egui::FontId::proportional(LABEL_FONT);
                    let galley = ui.painter().layout_no_wrap(label.to_string(), font, text_col);
                    let text_pos = rect.center() - galley.size() / 2.0;
                    ui.painter().galley(text_pos, galley, text_col);
                    if response.clicked() {
                        setter.set_parameter(&params.synth_filter_enable, *active);
                    }
                }
            });
        }

        ui.add_space(13.0);
        ui.label(egui::RichText::new("TYPE").size(LABEL_FONT).color(Color32::from_gray(140)));
        ui.add_space(6.0);
        render_filter_sat_type_buttons(ui, params, setter);

        ui.add_space(13.0);
        ui.label(egui::RichText::new("POLES").size(LABEL_FONT).color(Color32::from_gray(140)));
        ui.add_space(6.0);
        render_filter_poles_buttons(ui, params, setter);

        ui.add_space(13.0);
        ui.label(egui::RichText::new("MODE").size(LABEL_FONT).color(Color32::from_gray(140)));
        ui.add_space(6.0);
        render_filter_mode_buttons(ui, params, setter);

        ui.add_space(13.0);
        ui.label(egui::RichText::new("DRIVE BOOST").size(LABEL_FONT).color(Color32::from_gray(140)));
        ui.add_space(6.0);
        render_drive_boost_buttons(ui, params, setter);

        ui.add_space(13.0);
        ui.label(egui::RichText::new("SUB").size(LABEL_FONT).color(Color32::from_gray(140)));
        ui.add_space(6.0);
        render_sub_filter_route_toggle(ui, params, setter);

    });

    ui.painter().line_segment(
        [egui::pos2(sep_x, content_rect.top()), egui::pos2(sep_x, content_rect.bottom())],
        egui::Stroke::new(1.0, Color32::BLACK),
    );

    let right_rect = egui::Rect::from_min_max(
        egui::pos2(sep_x - 90.0 + margin.left as f32, content_rect.top() + margin.top as f32),
        egui::pos2(content_rect.right() - margin.right as f32, content_rect.bottom()),
    );
    let mut right_ui = ui.new_child(egui::UiBuilder::new().max_rect(right_rect));
    right_ui.vertical(|ui| {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label(egui::RichText::new("TONE").size(LABEL_FONT).color(Color32::from_gray(140)));
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 5.0;
                    render_vertical_slider_with_ticks(
                        ui, params, setter,
                        &params.synth_filter_cutoff, "CUT",
                        20.0, 20000.0, SliderScale::Exponential(3.0),
                        Some(Color32::from_rgb(130, 80, 160)),
                        &[(20.0, "20"), (200.0, "200"), (1000.0, "1k"), (5000.0, "5k"), (20000.0, "20k")],
                        None,
                    );
                    render_vertical_slider_with_ticks(
                        ui, params, setter,
                        &params.synth_filter_resonance, "RES",
                        0.0, 1.05, SliderScale::Linear,
                        Some(Color32::from_rgb(80, 90, 160)),
                        &[(0.0, "0%"), (0.25, "25%"), (0.5, "50%"), (0.75, "75%"), (1.0, "100%")],
                        None,
                    );
                    let boost_db = match params.synth_filter_drive_boost.value() {
                        1 => 12,
                        2 => 24,
                        3 => 48,
                        _ => 0,
                    };
                    let t0 = format!("{}dB", boost_db);
                    let t1 = format!("{}dB", boost_db + 3);
                    let t2 = format!("{}dB", boost_db + 6);
                    let t3 = format!("{}dB", boost_db + 9);
                    let t4 = format!("{}dB", boost_db + 12);
                    let drive_ticks: [(f32, &str); 5] = [
                        (0.0, &t0), (0.25, &t1), (0.5, &t2), (0.75, &t3), (1.0, &t4),
                    ];
                    render_vertical_slider_with_ticks(
                        ui, params, setter,
                        &params.synth_filter_drive, "DRV",
                        0.0, 1.0, SliderScale::Linear,
                        Some(Color32::from_rgb(150, 70, 80)),
                        &drive_ticks,
                        None,
                    );
                });
            });

            ui.add_space(25.0);

            ui.vertical(|ui| {
                ui.label(egui::RichText::new("MODULATION").size(LABEL_FONT).color(Color32::from_gray(140)));
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 5.0;
                    render_vertical_slider_with_ticks(
                        ui, params, setter,
                        &params.synth_filter_key_track, "KEY",
                        0.0, 1.0, SliderScale::Linear,
                        Some(Color32::from_rgb(60, 120, 110)),
                        &[(0.0, "OFF"), (0.25, "25%"), (0.5, "50%"), (0.75, "75%"), (1.0, "100%")],
                        None,
                    );
                    render_vertical_slider_with_ticks(
                        ui, params, setter,
                        &params.synth_filter_env_amount, "ENV",
                        -1.0, 1.0, SliderScale::Linear,
                        Some(Color32::from_rgb(140, 110, 50)),
                        &[(-1.0, "-100%"), (-0.5, "-50%"), (0.0, "0%"), (0.5, "+50%"), (1.0, "+100%")],
                        None,
                    );
                    render_vertical_slider_with_ticks(
                        ui, params, setter,
                        &params.synth_filter_stereo_sep, "SEP",
                        0.0, 0.50, SliderScale::Linear,
                        Some(Color32::from_rgb(60, 90, 150)),
                        &[(0.0, "0%"), (0.125, "25%"), (0.25, "50%"), (0.375, "75%"), (0.50, "100%")],
                        None,
                    );
                });
            });

            ui.add_space(25.0);

            ui.vertical(|ui| {
                ui.label(egui::RichText::new("CHARACTER").size(LABEL_FONT).color(Color32::from_gray(140)));
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 5.0;
                    let is_8pole = params.synth_filter_poles.value() == 1;
                    let morph_ticks_4: [(f32, &str); 5] = [(0.0, "LP24"), (0.25, "LP12"), (0.5, "BP12"), (0.75, "NTCH"), (1.0, "HP24")];
                    let morph_ticks_8: [(f32, &str); 5] = [(0.0, "LP48"), (0.25, "LP24"), (0.5, "BP24"), (0.75, "NTCH"), (1.0, "HP48")];
                    let morph_ticks: &[(f32, &str)] = if is_8pole { &morph_ticks_8 } else { &morph_ticks_4 };
                    render_vertical_slider_with_ticks(
                        ui, params, setter,
                        &params.synth_filter_morph, "MRPH",
                        0.0, 1.0, SliderScale::Linear,
                        Some(Color32::from_rgb(120, 100, 160)),
                        morph_ticks,
                        None,
                    );
                    render_vertical_slider_with_ticks(
                        ui, params, setter,
                        &params.synth_filter_fm, "FM",
                        0.0, 1.0, SliderScale::Linear,
                        Some(Color32::from_rgb(160, 120, 60)),
                        &[(0.0, "0%"), (0.25, "25%"), (0.5, "50%"), (0.75, "75%"), (1.0, "100%")],
                        None,
                    );
                    render_vertical_slider_with_ticks(
                        ui, params, setter,
                        &params.synth_filter_feedback, "FB",
                        -1.0, 1.0, SliderScale::Linear,
                        Some(Color32::from_rgb(160, 80, 80)),
                        &[(-1.0, "-100%"), (-0.5, "-50%"), (0.0, "0%"), (0.5, "+50%"), (1.0, "+100%")],
                        None,
                    );
                });
            });
        });

        ui.add_space(15.0);
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label(egui::RichText::new("DYNAMICS").size(LABEL_FONT).color(Color32::from_gray(140)));
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 5.0;
                    render_vertical_slider_with_ticks(
                        ui, params, setter,
                        &params.synth_filter_bass_lock, "BASS",
                        0.0, 1.0, SliderScale::Linear,
                        Some(Color32::from_rgb(80, 130, 80)),
                        &[(0.0, "0%"), (0.25, "25%"), (0.5, "50%"), (0.75, "75%"), (1.0, "100%")],
                        None,
                    );
                    render_vertical_slider_with_ticks(
                        ui, params, setter,
                        &params.synth_filter_res_character, "CHAR",
                        0.0, 1.0, SliderScale::Linear,
                        Some(Color32::from_rgb(140, 90, 110)),
                        &[(0.0, "0%"), (0.25, "25%"), (0.5, "50%"), (0.75, "75%"), (1.0, "100%")],
                        None,
                    );
                    render_vertical_slider_with_ticks(
                        ui, params, setter,
                        &params.synth_filter_cutoff_slew, "SLEW",
                        0.0, 1.0, SliderScale::Linear,
                        Some(Color32::from_rgb(100, 100, 140)),
                        &[(0.0, "0%"), (0.25, "25%"), (0.5, "50%"), (0.75, "75%"), (1.0, "100%")],
                        None,
                    );
                });
            });

            ui.add_space(25.0);

            ui.vertical(|ui| {
                ui.label(egui::RichText::new("SPREAD").size(LABEL_FONT).color(Color32::from_gray(140)));
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 5.0;
                    render_vertical_slider_with_ticks(
                        ui, params, setter,
                        &params.synth_filter_pole_spread, "SPRD",
                        0.0, 1.0, SliderScale::Linear,
                        Some(Color32::from_rgb(100, 120, 80)),
                        &[(0.0, "0%"), (0.25, "25%"), (0.5, "50%"), (0.75, "75%"), (1.0, "100%")],
                        None,
                    );
                    render_vertical_slider_with_ticks(
                        ui, params, setter,
                        &params.synth_filter_res_tilt, "TILT",
                        -1.0, 1.0, SliderScale::Linear,
                        Some(Color32::from_rgb(60, 100, 150)),
                        &[(-1.0, "-100%"), (-0.5, "-50%"), (0.0, "0%"), (0.5, "+50%"), (1.0, "+100%")],
                        None,
                    );
                });
            });

        });
    });
}

fn render_filter_envelope_controls(
    ui: &mut egui::Ui,
    params: &Arc<DeviceParams>,
    setter: &ParamSetter,
    ui_state: &SharedUiState,
) {
    let range_val = params.synth_filter_env_range.modulated_plain_value().max(1.0);
    let range_ms = range_val * 1000.0;
    let time_max_a = range_ms.min(5000.0).max(1.0);
    let time_max_dr = range_ms.min(10000.0).max(1.0);
    let shape_ticks: &[(f32, &str)] = &[(-1.0, "EXP"), (0.0, "LIN"), (1.0, "LOG")];
    let s_ticks: &[(f32, &str)] = &[(0.0, "LIN"), (0.5, ""), (1.0, "MAX")];
    let atk_color = Some(Color32::from_rgb(140, 100, 60));
    let hold_color = Some(Color32::from_rgb(120, 120, 60));
    let dec_color = Some(Color32::from_rgb(100, 80, 120));
    let sus_color = Some(Color32::from_rgb(60, 100, 80));
    let rel_color = Some(Color32::from_rgb(80, 80, 140));
    let dip_color = Some(Color32::from_rgb(140, 80, 80));

    let time_ticks_a = build_time_ticks(0.5, time_max_a);
    let time_ticks_a_ref: Vec<(f32, &str)> = time_ticks_a.iter().map(|(v, s)| (*v, s.as_str())).collect();
    let time_ticks_dr = build_time_ticks(0.5, time_max_dr);
    let time_ticks_dr_ref: Vec<(f32, &str)> = time_ticks_dr.iter().map(|(v, s)| (*v, s.as_str())).collect();
    let hold_ticks = build_time_ticks(0.0, time_max_a);
    let hold_ticks_ref: Vec<(f32, &str)> = hold_ticks.iter().map(|(v, s)| (*v, s.as_str())).collect();

    let tempo = ui_state.current_tempo.load(std::sync::atomic::Ordering::Relaxed) as f32 / 100.0;

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 6.0;
        if params.synth_filter_env_attack_sync.value() {
            render_vertical_sync_div_slider(ui, setter, &params.synth_filter_env_attack_div, "A", atk_color, tempo, 1.0);
        } else {
            render_vertical_slider_with_ticks(
                ui, params, setter,
                &params.synth_filter_env_attack, "A",
                0.5, time_max_a, SliderScale::Exponential(2.0),
                atk_color, &time_ticks_a_ref, None,
            );
        }
        ui.add_space(2.0);
        let (sh_min, sh_ticks) = if params.synth_filter_env_attack_s.value() { (0.0, s_ticks) } else { (-1.0, shape_ticks) };
        render_vertical_slider_with_ticks(
            ui, params, setter,
            &params.synth_filter_env_attack_shape, "A\u{2009}SH",
            sh_min, 1.0, SliderScale::Linear,
            atk_color, sh_ticks, None,
        );
        ui.add_space(4.0);
        if params.synth_filter_env_hold_sync.value() {
            render_vertical_sync_div_slider(ui, setter, &params.synth_filter_env_hold_div, "H", hold_color, tempo, 1.0);
        } else {
            render_vertical_slider_with_ticks(
                ui, params, setter,
                &params.synth_filter_env_hold, "H",
                0.0, time_max_a, SliderScale::Exponential(2.0),
                hold_color, &hold_ticks_ref, None,
            );
        }
        ui.add_space(4.0);
        if params.synth_filter_env_decay_sync.value() {
            render_vertical_sync_div_slider(ui, setter, &params.synth_filter_env_decay_div, "D", dec_color, tempo, 1.0);
        } else {
            render_vertical_slider_with_ticks(
                ui, params, setter,
                &params.synth_filter_env_decay, "D",
                0.5, time_max_dr, SliderScale::Exponential(2.0),
                dec_color, &time_ticks_dr_ref, None,
            );
        }
        ui.add_space(2.0);
        let (sh_min, sh_ticks) = if params.synth_filter_env_decay_s.value() { (0.0, s_ticks) } else { (-1.0, shape_ticks) };
        render_vertical_slider_with_ticks(
            ui, params, setter,
            &params.synth_filter_env_decay_shape, "D\u{2009}SH",
            sh_min, 1.0, SliderScale::Linear,
            dec_color, sh_ticks, None,
        );
        ui.add_space(4.0);
        render_vertical_slider(
            ui, params, setter,
            &params.synth_filter_env_sustain, "S",
            0.0, 1.0, SliderScale::Linear,
            sus_color, None,
        );
        ui.add_space(4.0);
        if params.synth_filter_env_release_sync.value() {
            render_vertical_sync_div_slider(ui, setter, &params.synth_filter_env_release_div, "R", rel_color, tempo, 4.0);
        } else {
            render_vertical_slider_with_ticks(
                ui, params, setter,
                &params.synth_filter_env_release, "R",
                0.5, time_max_dr, SliderScale::Exponential(2.0),
                rel_color, &time_ticks_dr_ref, None,
            );
        }
        ui.add_space(2.0);
        let (sh_min, sh_ticks) = if params.synth_filter_env_release_s.value() { (0.0, s_ticks) } else { (-1.0, shape_ticks) };
        render_vertical_slider_with_ticks(
            ui, params, setter,
            &params.synth_filter_env_release_shape, "R\u{2009}SH",
            sh_min, 1.0, SliderScale::Linear,
            rel_color, sh_ticks, None,
        );
        ui.add_space(6.0);
        render_vertical_slider(
            ui, params, setter,
            &params.synth_filter_env_dip, "DIP",
            0.0, 1.0, SliderScale::Linear,
            dip_color, None,
        );
    });

    ui.add_space(36.0);
    let row_left = ui.cursor().left();
    let row_top = ui.cursor().top();

    let range_ticks: &[(f32, &str)] = &[(1.0, "1"), (2.0, "2"), (4.0, "4"), (8.0, "8")];

    let rng_x = row_left + 2.0;
    let mut rng_ui = ui.new_child(egui::UiBuilder::new().max_rect(
        egui::Rect::from_min_size(egui::pos2(rng_x, row_top + 25.0), egui::vec2(80.0, 300.0)),
    ));
    rng_ui.horizontal(|ui| {
        let range_color = Some(Color32::from_rgb(100, 100, 80));
        render_vertical_slider_with_ticks(
            ui, params, setter,
            &params.synth_filter_env_range, "RNG",
            1.0, 8.0, SliderScale::Linear,
            range_color, range_ticks, None,
        );
    });

    let viz_x = row_left + 65.0;
    let viz_y = row_top + 35.0;
    render_filter_adsr_visualization(ui, params, viz_x, viz_y, ui_state);
}

fn render_filter_adsr_visualization(
    ui: &mut egui::Ui,
    params: &Arc<DeviceParams>,
    viz_x: f32,
    viz_y: f32,
    ui_state: &SharedUiState,
) {
    let viz_w: f32 = 295.0;
    let viz_h: f32 = 200.0;
    let pad: f32 = 10.0;

    let rect = egui::Rect::from_min_size(egui::pos2(viz_x, viz_y), egui::vec2(viz_w, viz_h));

    let bg_color = Color32::from_rgb(25, 25, 30);
    let border_color = Color32::from_rgb(50, 50, 60);
    let curve_color = Color32::from_rgb(140, 100, 200);
    let sustain_color = Color32::from_rgb(60, 100, 80);
    let grid_color = Color32::from_rgb(35, 35, 42);

    ui.painter().rect_filled(rect, 4.0, bg_color);
    ui.painter().rect_stroke(rect, 4.0, egui::Stroke::new(1.0, border_color), egui::epaint::StrokeKind::Inside);

    let inner = rect.shrink(pad);
    let w = inner.width();
    let h = inner.height();

    for i in 1..4 {
        let y = inner.top() + h * i as f32 / 4.0;
        ui.painter().line_segment(
            [egui::pos2(inner.left(), y), egui::pos2(inner.right(), y)],
            egui::Stroke::new(0.5, grid_color),
        );
    }

    let tempo = ui_state.current_tempo.load(std::sync::atomic::Ordering::Relaxed) as f32 / 100.0;
    let fe_div_to_ms = |div_idx: i32| -> f32 {
        let div = crate::synth::lfo::LfoSyncDivision::from_index(div_idx);
        (div.beats() as f64 / tempo.max(1.0) as f64 * 60000.0).max(0.5) as f32
    };

    let range_val = params.synth_filter_env_range.modulated_plain_value().max(1.0);
    let range_ms = range_val * 1000.0;
    let max_a = range_ms.min(5000.0).max(1.0);
    let max_dr = range_ms.min(10000.0).max(1.0);

    let attack_ms = if params.synth_filter_env_attack_sync.value() {
        fe_div_to_ms(params.synth_filter_env_attack_div.value())
    } else {
        params.synth_filter_env_attack.modulated_plain_value().clamp(0.5, max_a)
    };
    let hold_ms = if params.synth_filter_env_hold_sync.value() {
        fe_div_to_ms(params.synth_filter_env_hold_div.value())
    } else {
        params.synth_filter_env_hold.modulated_plain_value().clamp(0.0, 5000.0)
    };
    let decay_ms = if params.synth_filter_env_decay_sync.value() {
        fe_div_to_ms(params.synth_filter_env_decay_div.value())
    } else {
        params.synth_filter_env_decay.modulated_plain_value().clamp(0.5, max_dr)
    };
    let release_ms = if params.synth_filter_env_release_sync.value() {
        fe_div_to_ms(params.synth_filter_env_release_div.value())
    } else {
        params.synth_filter_env_release.modulated_plain_value().clamp(0.5, max_dr)
    };
    let sustain = params.synth_filter_env_sustain.modulated_plain_value().clamp(0.0, 1.0);
    let attack_shape = params.synth_filter_env_attack_shape.modulated_plain_value().clamp(-1.0, 1.0);
    let decay_shape = params.synth_filter_env_decay_shape.modulated_plain_value().clamp(-1.0, 1.0);
    let release_shape = params.synth_filter_env_release_shape.modulated_plain_value().clamp(-1.0, 1.0);
    let attack_s = params.synth_filter_env_attack_s.value();
    let decay_s = params.synth_filter_env_decay_s.value();
    let release_s = params.synth_filter_env_release_s.value();

    let dip = params.synth_filter_env_dip.modulated_plain_value().clamp(0.0, 1.0);
    let dip_ms: f32 = 2.0;
    let has_dip = dip > 0.001;

    let adsr_total = attack_ms + hold_ms + decay_ms + release_ms;
    let sustain_ms = adsr_total * 0.2;
    let effective_dip_ms = if has_dip { dip_ms } else { 0.0 };
    let total_ms = effective_dip_ms + adsr_total + sustain_ms;
    let time_scale = if total_ms > 0.001 { 1.0 / total_ms } else { 1.0 };

    let dip_w = effective_dip_ms * time_scale * w;
    let a_w = attack_ms * time_scale * w;
    let h_w = hold_ms * time_scale * w;
    let d_w = decay_ms * time_scale * w;
    let s_w = sustain_ms * time_scale * w;
    let r_w = release_ms * time_scale * w;

    let x0 = inner.left();
    let y_bot = inner.bottom();

    {
        let div_line_color = Color32::from_rgb(45, 45, 55);
        let div_label_color = Color32::from_gray(50);
        let div_font = egui::FontId::proportional(9.0);
        let divs: &[(f64, &str)] = &[
            (0.03125, "1/128"), (0.0625, "1/64"), (0.125, "1/32"),
            (0.25, "1/16"), (0.5, "1/8"), (1.0, "1/4"),
            (2.0, "1/2"), (4.0, "1/1"), (8.0, "2/1"), (16.0, "4/1"),
        ];
        let mut last_x = -100.0_f32;
        for &(beats, label) in divs {
            let ms = (beats / tempo.max(1.0) as f64 * 60000.0) as f32;
            if ms < 0.5 || ms > total_ms { continue; }
            let x = inner.left() + ms / total_ms * w;
            if (x - last_x) < 20.0 { continue; }
            last_x = x;
            ui.painter().line_segment(
                [egui::pos2(x, inner.top()), egui::pos2(x, inner.bottom())],
                egui::Stroke::new(0.5, div_line_color),
            );
            ui.painter().text(
                egui::pos2(x, inner.top() + 1.0),
                egui::Align2::CENTER_TOP,
                label,
                div_font.clone(),
                div_label_color,
            );
        }
    }

    let shaped_curve = |t: f32, shape: f32, s_curve: bool| -> f32 {
        let sd = if s_curve { (shape as f64).max(0.0) * 2.0 } else { shape as f64 };
        let td = t as f64;
        if sd.abs() < 0.01 { return t; }
        let k = 1.0 + sd.abs() * 9.0;
        let ln_k = if sd > 0.0 { k.ln() } else { -k.ln() };
        if s_curve {
            let ka = ln_k.abs();
            let denom = ka.exp_m1();
            (if ln_k > 0.0 {
                if td <= 0.5 { 0.5 * (td * 2.0 * ka).exp_m1() / denom }
                else { 1.0 - 0.5 * ((1.0 - td) * 2.0 * ka).exp_m1() / denom }
            } else {
                if td <= 0.5 { 0.5 * (1.0 - ((1.0 - td * 2.0) * ka).exp_m1() / denom) }
                else { 1.0 - 0.5 * (1.0 - ((1.0 - (1.0 - td) * 2.0) * ka).exp_m1() / denom) }
            }) as f32
        } else {
            (if ln_k > 0.0 {
                (td * ln_k).exp_m1() / ln_k.exp_m1()
            } else {
                let pk = -ln_k;
                1.0 - ((1.0 - td) * pk).exp_m1() / pk.exp_m1()
            }) as f32
        }
    };

    let segments = 16;
    let mut points = Vec::with_capacity(segments * 4 + 10);
    let dip_color = Color32::from_rgb(140, 80, 80);
    let hold_color = Color32::from_rgb(120, 120, 60);

    if has_dip {
        let dip_start_v = sustain;
        let dip_target_v = sustain * (1.0 - dip);
        points.push(egui::pos2(x0, y_bot - dip_start_v * h));
        points.push(egui::pos2(x0 + dip_w, y_bot - dip_target_v * h));

        for i in 0..points.len().saturating_sub(1) {
            ui.painter().line_segment(
                [points[i], points[i + 1]],
                egui::Stroke::new(1.5, dip_color),
            );
        }

        let dip_x_end = x0 + dip_w;
        points.clear();
        points.push(egui::pos2(dip_x_end, y_bot - dip_target_v * h));

        for i in 0..=segments {
            let t = i as f32 / segments as f32;
            let v = dip_target_v + shaped_curve(t, attack_shape, attack_s) * (1.0 - dip_target_v);
            points.push(egui::pos2(dip_x_end + t * a_w, y_bot - v * h));
        }
    } else {
        points.push(egui::pos2(x0, y_bot));

        for i in 0..=segments {
            let t = i as f32 / segments as f32;
            let v = shaped_curve(t, attack_shape, attack_s);
            points.push(egui::pos2(x0 + t * a_w, y_bot - v * h));
        }
    }

    let x_h_start = x0 + dip_w + a_w;
    if h_w > 0.5 {
        let hold_end = egui::pos2(x_h_start + h_w, y_bot - h);
        points.push(hold_end);

        let hold_start_idx = points.len() - 2;
        ui.painter().line_segment(
            [points[hold_start_idx], hold_end],
            egui::Stroke::new(1.5, hold_color),
        );
    }

    let x_d_start = x_h_start + h_w;
    for i in 1..=segments {
        let t = i as f32 / segments as f32;
        let v = 1.0 - shaped_curve(t, decay_shape, decay_s) * (1.0 - sustain);
        points.push(egui::pos2(x_d_start + t * d_w, y_bot - v * h));
    }

    let x_s_end = x_d_start + d_w + s_w;
    points.push(egui::pos2(x_s_end, y_bot - sustain * h));

    let x_r_start = x_s_end;
    for i in 1..=segments {
        let t = i as f32 / segments as f32;
        let v = sustain * (1.0 - shaped_curve(t, release_shape, release_s));
        points.push(egui::pos2(x_r_start + t * r_w, y_bot - v * h));
    }

    for i in 0..points.len().saturating_sub(1) {
        ui.painter().line_segment(
            [points[i], points[i + 1]],
            egui::Stroke::new(1.5, curve_color),
        );
    }

    let sus_y = y_bot - sustain * h;
    ui.painter().line_segment(
        [egui::pos2(inner.left(), sus_y), egui::pos2(x_s_end, sus_y)],
        egui::Stroke::new(0.5, sustain_color),
    );

    let total_adsr_ms = attack_ms + hold_ms + decay_ms + release_ms;
    let duration_str = if total_adsr_ms >= 1000.0 {
        format!("{:.1}s", total_adsr_ms / 1000.0)
    } else {
        format!("{:.0}ms", total_adsr_ms)
    };
    ui.painter().text(
        egui::pos2(rect.center().x, rect.bottom() + 4.0),
        egui::Align2::CENTER_TOP,
        duration_str,
        egui::FontId::proportional(14.0),
        Color32::from_gray(70),
    );
}

fn render_drive_boost_buttons(
    ui: &mut egui::Ui,
    params: &Arc<DeviceParams>,
    setter: &ParamSetter,
) {
    let current = params.synth_filter_drive_boost.value();
    let options = [("OFF", 0), ("+12dB", 1), ("+24dB", 2), ("+48dB", 3)];
    let btn_w = 80.0;
    let btn_h = 48.0;

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 5.0;
        for (label, value) in &options {
            let is_selected = current == *value;
            let (bg, text_col) = if is_selected {
                (Color32::from_rgb(140, 80, 160), Color32::WHITE)
            } else {
                (Color32::from_rgb(40, 40, 48), Color32::from_gray(160))
            };

            let (rect, response) = ui.allocate_exact_size(
                egui::vec2(btn_w, btn_h),
                egui::Sense::click(),
            );

            let hover_bg = if response.hovered() && !is_selected {
                Color32::from_rgb(55, 55, 65)
            } else {
                bg
            };

            ui.painter().rect_filled(rect, 4.0, hover_bg);
            if is_selected {
                ui.painter().rect_stroke(rect, 4.0, egui::Stroke::new(2.0, Color32::from_rgb(160, 100, 190)), egui::epaint::StrokeKind::Inside);
            }

            let font = egui::FontId::proportional(LABEL_FONT);
            let galley = ui.painter().layout_no_wrap(label.to_string(), font, text_col);
            let text_pos = rect.center() - galley.size() / 2.0;
            ui.painter().galley(text_pos, galley, text_col);

            if response.clicked() {
                setter.set_parameter(&params.synth_filter_drive_boost, *value);
            }
        }
    });
}

fn render_filter_poles_buttons(
    ui: &mut egui::Ui,
    params: &Arc<DeviceParams>,
    setter: &ParamSetter,
) {
    let current = params.synth_filter_poles.value();
    let options = [("4-POLE", 0), ("8-POLE", 1)];
    let btn_w = 80.0;
    let btn_h = 48.0;

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 5.0;
        for (label, value) in &options {
            let is_selected = current == *value;
            let (bg, text_col) = if is_selected {
                (Color32::from_rgb(140, 80, 160), Color32::WHITE)
            } else {
                (Color32::from_rgb(40, 40, 48), Color32::from_gray(160))
            };

            let (rect, response) = ui.allocate_exact_size(
                egui::vec2(btn_w, btn_h),
                egui::Sense::click(),
            );

            let hover_bg = if response.hovered() && !is_selected {
                Color32::from_rgb(55, 55, 65)
            } else {
                bg
            };

            ui.painter().rect_filled(rect, 4.0, hover_bg);
            if is_selected {
                ui.painter().rect_stroke(rect, 4.0, egui::Stroke::new(2.0, Color32::from_rgb(160, 100, 190)), egui::epaint::StrokeKind::Inside);
            }

            let font = egui::FontId::proportional(LABEL_FONT);
            let galley = ui.painter().layout_no_wrap(label.to_string(), font, text_col);
            let text_pos = rect.center() - galley.size() / 2.0;
            ui.painter().galley(text_pos, galley, text_col);

            if response.clicked() {
                setter.set_parameter(&params.synth_filter_poles, *value);
            }
        }
    });
}

fn render_filter_mode_buttons(
    ui: &mut egui::Ui,
    params: &Arc<DeviceParams>,
    setter: &ParamSetter,
) {
    let morph_val = params.synth_filter_morph.unmodulated_plain_value();
    let is_8pole = params.synth_filter_poles.value() == 1;
    let options: [(&str, f32); 5] = if is_8pole {
        [("LP48", 0.0), ("LP24", 0.25), ("BP24", 0.5), ("NTCH", 0.75), ("HP48", 1.0)]
    } else {
        [("LP24", 0.0), ("LP12", 0.25), ("BP12", 0.5), ("NTCH", 0.75), ("HP24", 1.0)]
    };
    let btn_w = 80.0;
    let btn_h = 48.0;
    let bg_off = [40u8, 40, 48];
    let bg_on = [140u8, 80, 160];
    let stroke_on = [160u8, 100, 190];
    let text_off = 160u8;

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 5.0;
        for (label, morph_pos) in &options {
            let dist = (morph_val - morph_pos).abs();
            let t = (1.0 - dist * 4.0).clamp(0.0, 1.0) as f64;

            let bg = Color32::from_rgb(
                (bg_off[0] as f64 + t * (bg_on[0] as f64 - bg_off[0] as f64)) as u8,
                (bg_off[1] as f64 + t * (bg_on[1] as f64 - bg_off[1] as f64)) as u8,
                (bg_off[2] as f64 + t * (bg_on[2] as f64 - bg_off[2] as f64)) as u8,
            );
            let text_col = Color32::from_gray(
                (text_off as f64 + t * (255.0 - text_off as f64)) as u8,
            );

            let (rect, response) = ui.allocate_exact_size(
                egui::vec2(btn_w, btn_h),
                egui::Sense::click(),
            );

            let fill = if response.hovered() && t < 0.01 {
                Color32::from_rgb(55, 55, 65)
            } else {
                bg
            };

            ui.painter().rect_filled(rect, 4.0, fill);
            if t > 0.01 {
                let stroke_alpha = (t * 255.0) as u8;
                ui.painter().rect_stroke(
                    rect, 4.0,
                    egui::Stroke::new(2.0, Color32::from_rgba_unmultiplied(
                        stroke_on[0], stroke_on[1], stroke_on[2], stroke_alpha,
                    )),
                    egui::epaint::StrokeKind::Inside,
                );
            }

            let font = egui::FontId::proportional(LABEL_FONT);
            let galley = ui.painter().layout_no_wrap(label.to_string(), font, text_col);
            let text_pos = rect.center() - galley.size() / 2.0;
            ui.painter().galley(text_pos, galley, text_col);

            if response.clicked() {
                setter.set_parameter(&params.synth_filter_morph, *morph_pos);
            }
        }
    });
}

fn render_sub_filter_route_toggle(
    ui: &mut egui::Ui,
    params: &Arc<DeviceParams>,
    setter: &ParamSetter,
) {
    let through_filter = params.synth_sub_filter_route.value();
    let btn_w = 56.0;
    let btn_h = 36.0;

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 4.0;
        for (label, active) in &[("OUT", false), ("IN", true)] {
            let is_selected = through_filter == *active;
            let (bg, text_col) = if is_selected {
                if *active {
                    (Color32::from_rgb(60, 100, 60), Color32::WHITE)
                } else {
                    (Color32::from_rgb(60, 60, 68), Color32::from_gray(180))
                }
            } else {
                (Color32::from_rgb(40, 40, 48), Color32::from_gray(140))
            };

            let (rect, response) = ui.allocate_exact_size(
                egui::vec2(btn_w, btn_h),
                egui::Sense::click(),
            );

            let hover_bg = if response.hovered() && !is_selected {
                Color32::from_rgb(55, 55, 65)
            } else {
                bg
            };

            ui.painter().rect_filled(rect, 4.0, hover_bg);
            if is_selected {
                let stroke_col = if *active {
                    Color32::from_rgb(80, 140, 80)
                } else {
                    Color32::from_rgb(80, 80, 90)
                };
                ui.painter().rect_stroke(rect, 4.0, egui::Stroke::new(2.0, stroke_col), egui::epaint::StrokeKind::Inside);
            }

            let font = egui::FontId::proportional(LABEL_FONT);
            let galley = ui.painter().layout_no_wrap(label.to_string(), font, text_col);
            let text_pos = rect.center() - galley.size() / 2.0;
            ui.painter().galley(text_pos, galley, text_col);

            if response.clicked() {
                setter.set_parameter(&params.synth_sub_filter_route, *active);
            }
        }
    });
}

fn render_route_toggle(
    ui: &mut egui::Ui,
    setter: &ParamSetter,
    param: &BoolParam,
    label: &str,
) {
    render_route_toggle_dimmed(ui, setter, param, label, false);
}

fn render_route_toggle_dimmed(
    ui: &mut egui::Ui,
    setter: &ParamSetter,
    param: &BoolParam,
    label: &str,
    dimmed: bool,
) {
    let is_on = param.value();
    let btn_w = 56.0;
    let btn_h = 48.0;
    let alpha = if dimmed { 0.3 } else { 1.0 };

    ui.vertical(|ui| {
        let label_gray = (160.0 * alpha) as u8;
        ui.label(egui::RichText::new(label).size(LABEL_FONT).color(Color32::from_gray(label_gray)));
        ui.add_space(2.0);
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 4.0;
            for (text, active) in &[("OUT", false), ("IN", true)] {
                let is_selected = is_on == *active;
                let (bg, text_col) = if is_selected {
                    if *active {
                        let g = (100.0 * alpha) as u8;
                        (Color32::from_rgb((60.0 * alpha) as u8, g, (60.0 * alpha) as u8),
                         Color32::from_gray((255.0 * alpha) as u8))
                    } else {
                        (Color32::from_rgb((60.0 * alpha) as u8, (60.0 * alpha) as u8, (68.0 * alpha) as u8),
                         Color32::from_gray((180.0 * alpha) as u8))
                    }
                } else {
                    (Color32::from_rgb((40.0 * alpha) as u8, (40.0 * alpha) as u8, (48.0 * alpha) as u8),
                     Color32::from_gray((140.0 * alpha) as u8))
                };

                let sense = if dimmed { egui::Sense::hover() } else { egui::Sense::click() };
                let (rect, response) = ui.allocate_exact_size(
                    egui::vec2(btn_w, btn_h),
                    sense,
                );

                let hover_bg = if !dimmed && response.hovered() && !is_selected {
                    Color32::from_rgb(55, 55, 65)
                } else {
                    bg
                };

                ui.painter().rect_filled(rect, 4.0, hover_bg);
                if is_selected && !dimmed {
                    let stroke_col = if *active {
                        Color32::from_rgb(80, 140, 80)
                    } else {
                        Color32::from_rgb(80, 80, 90)
                    };
                    ui.painter().rect_stroke(rect, 4.0, egui::Stroke::new(2.0, stroke_col), egui::epaint::StrokeKind::Inside);
                }

                let font = egui::FontId::proportional(LABEL_FONT);
                let galley = ui.painter().layout_no_wrap(text.to_string(), font, text_col);
                let text_pos = rect.center() - galley.size() / 2.0;
                ui.painter().galley(text_pos, galley, text_col);

                if !dimmed && response.clicked() {
                    setter.set_parameter(param, *active);
                }
            }
        });
    });
}

fn render_filter_sat_type_buttons(
    ui: &mut egui::Ui,
    params: &Arc<DeviceParams>,
    setter: &ParamSetter,
) {
    let current = params.synth_filter_sat_type.value();
    let options = [("TRAN", 0), ("DIODE", 1), ("TUBE", 2)];
    let btn_w = 80.0;
    let btn_h = 48.0;

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 5.0;
        for (label, value) in &options {
            let is_selected = current == *value;
            let (bg, text_col) = if is_selected {
                (Color32::from_rgb(140, 80, 160), Color32::WHITE)
            } else {
                (Color32::from_rgb(40, 40, 48), Color32::from_gray(160))
            };

            let (rect, response) = ui.allocate_exact_size(
                egui::vec2(btn_w, btn_h),
                egui::Sense::click(),
            );

            let hover_bg = if response.hovered() && !is_selected {
                Color32::from_rgb(55, 55, 65)
            } else {
                bg
            };

            ui.painter().rect_filled(rect, 4.0, hover_bg);
            if is_selected {
                ui.painter().rect_stroke(rect, 4.0, egui::Stroke::new(2.0, Color32::from_rgb(160, 100, 190)), egui::epaint::StrokeKind::Inside);
            }

            let font = egui::FontId::proportional(LABEL_FONT);
            let galley = ui.painter().layout_no_wrap(label.to_string(), font, text_col);
            let text_pos = rect.center() - galley.size() / 2.0;
            ui.painter().galley(text_pos, galley, text_col);

            if response.clicked() {
                setter.set_parameter(&params.synth_filter_sat_type, *value);
            }
        }
    });
}

fn render_vol_env_tab(
    ui: &mut egui::Ui,
    params: &Arc<DeviceParams>,
    setter: &ParamSetter,
    ui_state: &SharedUiState,
) {
    let content_rect = ui.max_rect();
    let sep_x = content_rect.left() + content_rect.width() * 0.58;
    let margin = FRAME_MARGIN;

    let left_rect = egui::Rect::from_min_max(
        egui::pos2(content_rect.left() + margin.left as f32 + 5.0, content_rect.top() + margin.top as f32),
        egui::pos2(sep_x - 10.0, content_rect.bottom()),
    );
    let mut left_ui = ui.new_child(egui::UiBuilder::new().max_rect(left_rect));
    left_ui.vertical(|ui| {
        ui.label(egui::RichText::new("VOLUME ENVELOPE").size(HEADER_FONT).strong());
        ui.add_space(10.0);
        render_envelope_controls_compact(ui, params, setter, ui_state);
    });

    ui.painter().line_segment(
        [egui::pos2(sep_x, content_rect.top()), egui::pos2(sep_x, content_rect.bottom())],
        egui::Stroke::new(1.0, Color32::BLACK),
    );

    let right_rect = egui::Rect::from_min_max(
        egui::pos2(sep_x + 10.0, content_rect.top() + margin.top as f32),
        egui::pos2(content_rect.right() - margin.right as f32, content_rect.bottom()),
    );
    let mut right_ui = ui.new_child(egui::UiBuilder::new().max_rect(right_rect));
    right_ui.vertical(|ui| {
        ui.label(egui::RichText::new("MODIFIERS").size(HEADER_FONT).strong());
        ui.add_space(10.0);

        let mod_color = Some(Color32::from_rgb(100, 120, 80));
        let vel_color = Some(Color32::from_rgb(120, 100, 60));
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 5.0;
            render_vertical_slider(
                ui, params, setter,
                &params.synth_vol_depth, "DPTH",
                0.0, 1.0, SliderScale::Linear,
                mod_color, None,
            );
            render_vertical_slider(
                ui, params, setter,
                &params.synth_env_key_track, "KEY",
                0.0, 1.0, SliderScale::Linear,
                mod_color, None,
            );
            ui.add_space(4.0);
            render_vertical_slider_with_ticks(
                ui, params, setter,
                &params.synth_env_vel_to_attack, "V>A",
                -1.0, 1.0, SliderScale::Linear,
                vel_color, &[(-1.0, "-1"), (0.0, "0"), (1.0, "+1")], None,
            );
            render_vertical_slider_with_ticks(
                ui, params, setter,
                &params.synth_env_vel_to_decay, "V>D",
                -1.0, 1.0, SliderScale::Linear,
                vel_color, &[(-1.0, "-1"), (0.0, "0"), (1.0, "+1")], None,
            );
            render_vertical_slider_with_ticks(
                ui, params, setter,
                &params.synth_env_vel_to_sustain, "V>S",
                -1.0, 1.0, SliderScale::Linear,
                vel_color, &[(-1.0, "-1"), (0.0, "0"), (1.0, "+1")], None,
            );
            ui.add_space(8.0);
            let s_accent = Color32::from_rgb(180, 140, 60);
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing.y = 5.0;
                render_s_mode_button(ui, setter, &params.synth_vol_attack_s, "A S-MODE", s_accent, 100.0, 48.0);
                render_s_mode_button(ui, setter, &params.synth_vol_decay_s, "D S-MODE", s_accent, 100.0, 48.0);
                render_s_mode_button(ui, setter, &params.synth_vol_release_s, "R S-MODE", s_accent, 100.0, 48.0);
            });
        });

        ui.add_space(20.0);
        let loop_accent = Color32::from_rgb(80, 120, 160);
        let current_loop = params.synth_vol_loop_mode.value();
        ui.label(egui::RichText::new("LOOP MODE").size(14.0).color(Color32::from_gray(140)));
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 5.0;
            render_looper_select_button(ui, setter, &params.synth_vol_loop_mode, "ONE", 0, current_loop, loop_accent, 80.0, 48.0);
            render_looper_select_button(ui, setter, &params.synth_vol_loop_mode, "LOOP", 1, current_loop, loop_accent, 80.0, 48.0);
        });

        ui.add_space(13.0);
        ui.label(egui::RichText::new("TEMPO SYNC").size(14.0).color(Color32::from_gray(140)));
        ui.add_space(6.0);
        let tempo = ui_state.current_tempo.load(std::sync::atomic::Ordering::Relaxed) as f32 / 100.0;
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 5.0;
            render_env_sync_row(ui, setter, "A", &params.synth_vol_attack_sync, &params.synth_vol_attack_div, "vs_a", tempo);
            render_env_sync_row(ui, setter, "D", &params.synth_vol_decay_sync, &params.synth_vol_decay_div, "vs_d", tempo);
        });
        ui.add_space(5.0);
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 5.0;
            render_env_sync_row(ui, setter, "H", &params.synth_vol_hold_sync, &params.synth_vol_hold_div, "vs_h", tempo);
            render_env_sync_row(ui, setter, "R", &params.synth_vol_release_sync, &params.synth_vol_release_div, "vs_r", tempo);
        });
    });
}

fn render_filt_env_tab(
    ui: &mut egui::Ui,
    params: &Arc<DeviceParams>,
    setter: &ParamSetter,
    ui_state: &SharedUiState,
) {
    let content_rect = ui.max_rect();
    let sep_x = content_rect.left() + content_rect.width() * 0.58;
    let margin = FRAME_MARGIN;

    let left_rect = egui::Rect::from_min_max(
        egui::pos2(content_rect.left() + margin.left as f32 + 5.0, content_rect.top() + margin.top as f32),
        egui::pos2(sep_x - 10.0, content_rect.bottom()),
    );
    let mut left_ui = ui.new_child(egui::UiBuilder::new().max_rect(left_rect));
    left_ui.vertical(|ui| {
        ui.label(egui::RichText::new("FILTER ENVELOPE").size(HEADER_FONT).strong());
        ui.add_space(10.0);
        render_filter_envelope_controls(ui, params, setter, ui_state);
    });

    ui.painter().line_segment(
        [egui::pos2(sep_x, content_rect.top()), egui::pos2(sep_x, content_rect.bottom())],
        egui::Stroke::new(1.0, Color32::BLACK),
    );

    let right_rect = egui::Rect::from_min_max(
        egui::pos2(sep_x + 10.0, content_rect.top() + margin.top as f32),
        egui::pos2(content_rect.right() - margin.right as f32, content_rect.bottom()),
    );
    let mut right_ui = ui.new_child(egui::UiBuilder::new().max_rect(right_rect));
    right_ui.vertical(|ui| {
        ui.label(egui::RichText::new("OPTIONS").size(HEADER_FONT).strong());
        ui.add_space(10.0);

        let btn_size = egui::vec2(120.0, 48.0);
        let (btn_rect, response) = ui.allocate_exact_size(btn_size, egui::Sense::click());
        let bg = if response.hovered() {
            Color32::from_rgb(55, 55, 65)
        } else {
            Color32::from_rgb(40, 40, 48)
        };
        ui.painter().rect_filled(btn_rect, 4.0, bg);
        let font = egui::FontId::proportional(LABEL_FONT);
        let galley = ui.painter().layout_no_wrap("COPY VOL".to_string(), font, Color32::from_gray(160));
        let text_pos = btn_rect.center() - galley.size() / 2.0;
        ui.painter().galley(text_pos, galley, Color32::from_gray(160));
        if response.clicked() {
            setter.set_parameter(&params.synth_filter_env_attack, params.synth_vol_attack.modulated_plain_value());
            setter.set_parameter(&params.synth_filter_env_attack_shape, params.synth_vol_attack_shape.modulated_plain_value());
            setter.set_parameter(&params.synth_filter_env_decay, params.synth_vol_decay.modulated_plain_value());
            setter.set_parameter(&params.synth_filter_env_decay_shape, params.synth_vol_decay_shape.modulated_plain_value());
            setter.set_parameter(&params.synth_filter_env_sustain, params.synth_vol_sustain.modulated_plain_value());
            setter.set_parameter(&params.synth_filter_env_release, params.synth_vol_release.modulated_plain_value());
            setter.set_parameter(&params.synth_filter_env_release_shape, params.synth_vol_release_shape.modulated_plain_value());
            setter.set_parameter(&params.synth_filter_env_dip, params.synth_retrigger_dip.modulated_plain_value());
            setter.set_parameter(&params.synth_filter_env_hold, params.synth_vol_hold.modulated_plain_value());
            setter.set_parameter(&params.synth_filter_env_loop_mode, params.synth_vol_loop_mode.value());
            setter.set_parameter(&params.synth_filter_env_attack_sync, params.synth_vol_attack_sync.value());
            setter.set_parameter(&params.synth_filter_env_hold_sync, params.synth_vol_hold_sync.value());
            setter.set_parameter(&params.synth_filter_env_decay_sync, params.synth_vol_decay_sync.value());
            setter.set_parameter(&params.synth_filter_env_release_sync, params.synth_vol_release_sync.value());
            setter.set_parameter(&params.synth_filter_env_attack_div, params.synth_vol_attack_div.value());
            setter.set_parameter(&params.synth_filter_env_hold_div, params.synth_vol_hold_div.value());
            setter.set_parameter(&params.synth_filter_env_decay_div, params.synth_vol_decay_div.value());
            setter.set_parameter(&params.synth_filter_env_release_div, params.synth_vol_release_div.value());
            setter.set_parameter(&params.synth_filter_env_attack_s, params.synth_vol_attack_s.value());
            setter.set_parameter(&params.synth_filter_env_decay_s, params.synth_vol_decay_s.value());
            setter.set_parameter(&params.synth_filter_env_release_s, params.synth_vol_release_s.value());
        }

        ui.add_space(15.0);
        ui.label(egui::RichText::new("S-CURVE").size(14.0).color(Color32::from_gray(140)));
        ui.add_space(6.0);
        let s_accent = Color32::from_rgb(180, 140, 60);
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 5.0;
            render_s_mode_button(ui, setter, &params.synth_filter_env_attack_s, "A S-MODE", s_accent, 100.0, 48.0);
            render_s_mode_button(ui, setter, &params.synth_filter_env_decay_s, "D S-MODE", s_accent, 100.0, 48.0);
            render_s_mode_button(ui, setter, &params.synth_filter_env_release_s, "R S-MODE", s_accent, 100.0, 48.0);
        });

        ui.add_space(15.0);
        let loop_accent = Color32::from_rgb(100, 80, 160);
        let current_loop = params.synth_filter_env_loop_mode.value();
        ui.label(egui::RichText::new("LOOP MODE").size(14.0).color(Color32::from_gray(140)));
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 5.0;
            render_looper_select_button(ui, setter, &params.synth_filter_env_loop_mode, "ONE", 0, current_loop, loop_accent, 80.0, 48.0);
            render_looper_select_button(ui, setter, &params.synth_filter_env_loop_mode, "LOOP", 1, current_loop, loop_accent, 80.0, 48.0);
        });

        ui.add_space(13.0);
        ui.label(egui::RichText::new("TEMPO SYNC").size(14.0).color(Color32::from_gray(140)));
        ui.add_space(6.0);
        let tempo = ui_state.current_tempo.load(std::sync::atomic::Ordering::Relaxed) as f32 / 100.0;
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 5.0;
            render_env_sync_row(ui, setter, "A", &params.synth_filter_env_attack_sync, &params.synth_filter_env_attack_div, "fs_a", tempo);
            render_env_sync_row(ui, setter, "D", &params.synth_filter_env_decay_sync, &params.synth_filter_env_decay_div, "fs_d", tempo);
        });
        ui.add_space(5.0);
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 5.0;
            render_env_sync_row(ui, setter, "H", &params.synth_filter_env_hold_sync, &params.synth_filter_env_hold_div, "fs_h", tempo);
            render_env_sync_row(ui, setter, "R", &params.synth_filter_env_release_sync, &params.synth_filter_env_release_div, "fs_r", tempo);
        });
    });
}

fn render_env_sync_row(
    ui: &mut egui::Ui,
    setter: &ParamSetter,
    label: &str,
    sync_param: &nih_plug::prelude::BoolParam,
    div_param: &nih_plug::prelude::IntParam,
    _id: &str,
    tempo: f32,
) {
    let is_on = sync_param.value();
    let accent = Color32::from_rgb(80, 140, 120);
    ui.horizontal(|ui| {
        ui.set_min_width(170.0);
        ui.set_max_width(170.0);
        ui.spacing_mut().item_spacing.x = 8.0;

        let btn_text = format!("{} SYNC", label);
        let btn_size = egui::vec2(80.0, 48.0);
        let (rect, response) = ui.allocate_exact_size(btn_size, egui::Sense::click());

        let (bg, text_col) = if is_on {
            (accent, Color32::WHITE)
        } else {
            let hover = if response.hovered() { Color32::from_rgb(55, 55, 65) } else { Color32::from_rgb(40, 40, 48) };
            (hover, Color32::from_gray(160))
        };
        ui.painter().rect_filled(rect, 4.0, bg);
        if is_on {
            ui.painter().rect_stroke(rect, 4.0,
                egui::Stroke::new(2.0, Color32::from_rgb(
                    accent.r().saturating_add(30),
                    accent.g().saturating_add(20),
                    accent.b().saturating_add(20),
                )),
                egui::epaint::StrokeKind::Inside,
            );
        }
        let font = egui::FontId::proportional(LABEL_FONT);
        let galley = ui.painter().layout_no_wrap(btn_text, font, text_col);
        let text_pos = rect.center() - galley.size() / 2.0;
        ui.painter().galley(text_pos, galley, text_col);
        if response.clicked() {
            setter.set_parameter(sync_param, !is_on);
        }

        if is_on {
            let div = crate::synth::lfo::LfoSyncDivision::from_index(div_param.value());
            let ms = div.beats() / tempo.max(1.0) as f64 * 60000.0;
            let val_str = if ms >= 1000.0 {
                format!("{} {:.1}s", div.label(), ms / 1000.0)
            } else {
                format!("{} {:.0}ms", div.label(), ms)
            };
            ui.label(egui::RichText::new(val_str).size(14.0).color(Color32::from_gray(120)));
        }
    });
}

fn render_fx_tab(
    ui: &mut egui::Ui,
    params: &Arc<DeviceParams>,
    setter: &ParamSetter,
) {
    ui.horizontal(|ui| {
        egui::Frame::NONE
            .inner_margin(egui::Margin { left: FRAME_MARGIN.left + 5, ..FRAME_MARGIN })
            .show(ui, |ui| {
                render_looper_section(ui, params, setter);
            });

        ui.add_space(10.0);

        egui::Frame::NONE
            .inner_margin(egui::Margin { left: 5, ..FRAME_MARGIN })
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    ui.label(egui::RichText::new("MASTER HPF").size(HEADER_FONT).strong());
                    ui.add_space(9.0);
                    render_hpf_buttons(ui, params, setter);
                    ui.add_space(13.0);
                    ui.label(egui::RichText::new("BOOST").size(LABEL_FONT).color(Color32::from_gray(140)));
                    ui.add_space(6.0);
                    render_hpf_boost_buttons(ui, params, setter);
                    ui.add_space(13.0);
                    ui.label(egui::RichText::new("BOX CUT").size(LABEL_FONT).color(Color32::from_gray(140)));
                    ui.add_space(6.0);
                    render_box_cut_buttons(ui, params, setter);
                    ui.add_space(18.0);
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("SUB").size(LABEL_FONT).color(Color32::from_gray(140)));
                        ui.add_space(6.0);
                        render_toggle_switch(ui, params, setter, &params.master_hpf_sub, "OUT", "IN");
                    });
                    ui.add_space(20.0);
                    let chart_x = ui.cursor().left();
                    let chart_y = ui.cursor().top() + 12.0;
                    render_filter_visualization(ui, params, chart_x, chart_y);
                });
            });

        ui.add_space(20.0);

        egui::Frame::NONE
            .inner_margin(egui::Margin { left: 5, right: 0, ..FRAME_MARGIN })
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("BRILL").size(HEADER_FONT).strong());
                        ui.add_space(25.0);
                        ui.label(egui::RichText::new("STEREO").size(HEADER_FONT).strong());
                    });
                    ui.add_space(10.0);
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 5.0;
                        let brill_color = Some(Color32::from_rgb(160, 130, 60));
                        render_vertical_slider_with_ticks(
                            ui, params, setter,
                            &params.brilliance_amount, "BRILL",
                            0.0, 1.0, SliderScale::Linear,
                            brill_color,
                            &[(0.0, "OFF"), (0.25, "25%"), (0.5, "50%"), (0.75, "75%"), (1.0, "100%")],
                            None,
                        );
                        let stereo_color = Some(Color32::from_rgb(50, 130, 110));
                        ui.add_space(15.0);
                        render_vertical_slider_with_ticks(
                            ui, params, setter,
                            &params.stereo_mono_bass, "MON",
                            0.0, 300.0, SliderScale::Linear,
                            stereo_color,
                            &[(0.0, "OFF"), (80.0, "80Hz"), (120.0, "120"), (200.0, "200"), (300.0, "300Hz")],
                            None,
                        );
                        render_vertical_slider_with_ticks(
                            ui, params, setter,
                            &params.stereo_width, "WIDTH",
                            0.0, 2.0, SliderScale::Linear,
                            stereo_color,
                            &[(0.0, "0%"), (0.5, "50%"), (1.0, "100%"), (1.5, "150%"), (2.0, "200%")],
                            None,
                        );
                    });
                });
            });
    });
}

fn render_lush_tab(
    ui: &mut egui::Ui,
    params: &Arc<DeviceParams>,
    setter: &ParamSetter,
) {
    let reverb_color = Some(Color32::from_rgb(90, 110, 190));
    let tone_color = Some(Color32::from_rgb(80, 140, 140));
    let input_color = Some(Color32::from_rgb(140, 100, 160));
    let mod_color = Some(Color32::from_rgb(130, 130, 80));
    let duck_color = Some(Color32::from_rgb(160, 120, 80));

    let content_rect = ui.max_rect();
    let half_w = content_rect.width() / 2.0;
    let sep_x = content_rect.left() + half_w;
    let margin = FRAME_MARGIN;

    // ===== LEFT PANEL: BUTTONS & TOGGLES =====
    let left_rect = egui::Rect::from_min_max(
        egui::pos2(content_rect.left() + margin.left as f32 + 5.0, content_rect.top() + margin.top as f32),
        egui::pos2(sep_x - 10.0, content_rect.bottom()),
    );
    let mut left_ui = ui.new_child(egui::UiBuilder::new().max_rect(left_rect));
    left_ui.vertical(|ui| {
        ui.label(egui::RichText::new("REVERB").size(HEADER_FONT).strong());
        ui.add_space(9.0);
        {
            let on = params.synth_reverb_enable.value();
            let btn_w = 80.0;
            let btn_h = 48.0;
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 5.0;
                for (label, active) in &[("OFF", false), ("ON", true)] {
                    let is_selected = on == *active;
                    let (bg, text_col) = if is_selected {
                        if *active {
                            (Color32::from_rgb(80, 160, 80), Color32::WHITE)
                        } else {
                            (Color32::from_rgb(180, 60, 60), Color32::WHITE)
                        }
                    } else {
                        (Color32::from_rgb(40, 40, 48), Color32::from_gray(160))
                    };
                    let (rect, response) = ui.allocate_exact_size(
                        egui::vec2(btn_w, btn_h),
                        egui::Sense::click(),
                    );
                    let hover_bg = if response.hovered() && !is_selected {
                        Color32::from_rgb(55, 55, 65)
                    } else {
                        bg
                    };
                    ui.painter().rect_filled(rect, 4.0, hover_bg);
                    if is_selected {
                        let stroke_col = if *active {
                            Color32::from_rgb(100, 190, 100)
                        } else {
                            Color32::from_rgb(210, 80, 80)
                        };
                        ui.painter().rect_stroke(rect, 4.0, egui::Stroke::new(2.0, stroke_col), egui::epaint::StrokeKind::Inside);
                    }
                    let font = egui::FontId::proportional(LABEL_FONT);
                    let galley = ui.painter().layout_no_wrap(label.to_string(), font, text_col);
                    let text_pos = rect.center() - galley.size() / 2.0;
                    ui.painter().galley(text_pos, galley, text_col);
                    if response.clicked() {
                        setter.set_parameter(&params.synth_reverb_enable, *active);
                    }
                }
            });
        }

        ui.add_space(13.0);
        let rev_filter_on = params.synth_reverb_send_filter.value();
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 16.0;
            render_route_toggle_dimmed(ui, setter, &params.synth_reverb_send_vps, "VPS", rev_filter_on);
            render_route_toggle_dimmed(ui, setter, &params.synth_reverb_send_pll, "PLL", rev_filter_on);
            render_route_toggle_dimmed(ui, setter, &params.synth_reverb_send_saw, "SAW", rev_filter_on);
        });
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 16.0;
            render_route_toggle_dimmed(ui, setter, &params.synth_reverb_send_sub, "SUB", rev_filter_on);
            render_route_toggle(ui, setter, &params.synth_reverb_send_filter, "FLTR");
            render_route_toggle(ui, setter, &params.synth_reverb_send_looper, "LOOP");
        });

        ui.add_space(13.0);
        render_route_toggle(ui, setter, &params.synth_reverb_pre_delay_sync, "PRE-DELAY SYNC");
        if params.synth_reverb_pre_delay_sync.value() {
            ui.add_space(6.0);
            render_looper_division_combo(ui, setter, &params.synth_reverb_pre_delay_division,
                "predly_div", 100.0, Color32::from_rgb(90, 110, 190));
        }

        ui.add_space(13.0);
        ui.label(egui::RichText::new("DUCKING").size(LABEL_FONT).color(Color32::from_gray(140)));
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("REL").size(12.0).color(Color32::from_gray(120)));
            ui.add_space(2.0);
            render_looper_division_combo(ui, setter, &params.synth_reverb_duck_division,
                "duck_div", 80.0, Color32::from_rgb(160, 120, 80));
        });

        ui.add_space(13.0);
        ui.label(egui::RichText::new("SIDECHAIN").size(LABEL_FONT).color(Color32::from_gray(140)));
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("RATE").size(12.0).color(Color32::from_gray(120)));
            ui.add_space(2.0);
            render_looper_division_combo(ui, setter, &params.synth_reverb_rhythm_duck_division,
                "rhythm_duck_div", 80.0, Color32::from_rgb(160, 120, 80));
        });
    });

    // ===== SEPARATOR =====
    ui.painter().line_segment(
        [egui::pos2(sep_x, content_rect.top()), egui::pos2(sep_x, content_rect.bottom())],
        egui::Stroke::new(1.0, Color32::BLACK),
    );

    // ===== RIGHT PANEL: ALL SLIDERS =====
    let right_rect = egui::Rect::from_min_max(
        egui::pos2(sep_x - 90.0 + margin.left as f32, content_rect.top() + margin.top as f32),
        egui::pos2(content_rect.right() - margin.right as f32, content_rect.bottom()),
    );
    let mut right_ui = ui.new_child(egui::UiBuilder::new().max_rect(right_rect));
    right_ui.vertical(|ui| {
        // ===== ROW 1: REVERB + TONE =====
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label(egui::RichText::new("REVERB").size(LABEL_FONT).color(Color32::from_gray(140)));
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 5.0;
                    render_vertical_slider_with_ticks(
                        ui, params, setter, &params.synth_reverb_mix, "MIX",
                        0.0, 1.0, SliderScale::Linear, reverb_color,
                        &[(0.0, "DRY"), (0.25, "-12dB"), (0.5, "-6dB"), (0.75, "-3dB"), (1.0, "WET")], None,
                    );
                    render_vertical_slider_with_ticks(
                        ui, params, setter, &params.synth_reverb_decay, "DCY",
                        0.0, 1.0, SliderScale::Linear, reverb_color,
                        &[(0.0, "SHORT"), (0.25, "1s"), (0.5, "3s"), (0.75, "8s"), (1.0, "INF")], None,
                    );
                    render_vertical_slider_with_ticks(
                        ui, params, setter, &params.synth_reverb_time_scale, "SIZE",
                        0.0, 1.0, SliderScale::Linear, reverb_color,
                        &[(0.0, "TINY"), (0.25, "SM"), (0.5, "MED"), (0.75, "LG"), (1.0, "HALL")], None,
                    );
                    render_vertical_slider_with_ticks(
                        ui, params, setter, &params.synth_reverb_stereo_width, "WDTH",
                        0.0, 1.0, SliderScale::Linear, reverb_color,
                        &[(0.0, "MONO"), (0.25, "25%"), (0.5, "50%"), (0.75, "75%"), (1.0, "FULL")], None,
                    );
                    if !params.synth_reverb_pre_delay_sync.value() {
                        render_vertical_slider_with_ticks(
                            ui, params, setter, &params.synth_reverb_pre_delay, "DLY",
                            0.0, 500.0, SliderScale::Linear, reverb_color,
                            &[(0.0, "0ms"), (50.0, "50"), (125.0, "125"), (250.0, "250"), (500.0, "500")], None,
                        );
                    }
                });
            });

            ui.add_space(25.0);

            ui.vertical(|ui| {
                ui.label(egui::RichText::new("TONE").size(LABEL_FONT).color(Color32::from_gray(140)));
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 5.0;
                    render_vertical_slider_with_ticks(
                        ui, params, setter, &params.synth_reverb_lpf, "LPF",
                        20.0, 22000.0, SliderScale::Logarithmic, tone_color,
                        &[(20.0, "20"), (200.0, "200"), (2000.0, "2k"), (10000.0, "10k"), (22000.0, "22k")], None,
                    );
                    render_vertical_slider_with_ticks(
                        ui, params, setter, &params.synth_reverb_hpf, "HPF",
                        20.0, 22000.0, SliderScale::Logarithmic, tone_color,
                        &[(20.0, "20"), (100.0, "100"), (500.0, "500"), (2000.0, "2k"), (10000.0, "10k")], None,
                    );
                    render_vertical_slider_with_ticks(
                        ui, params, setter, &params.synth_reverb_diffusion, "DIFF",
                        0.0, 1.0, SliderScale::Linear, tone_color,
                        &[(0.0, "CLEAN"), (0.25, "25%"), (0.5, "50%"), (0.75, "75%"), (1.0, "LUSH")], None,
                    );
                    render_vertical_slider_with_ticks(
                        ui, params, setter, &params.synth_reverb_diffusion_mix, "DIFM",
                        0.0, 1.0, SliderScale::Linear, tone_color,
                        &[(0.0, "0%"), (0.25, "25%"), (0.5, "50%"), (0.75, "75%"), (1.0, "100%")], None,
                    );
                    render_vertical_slider_with_ticks(
                        ui, params, setter, &params.synth_reverb_saturation, "SAT",
                        0.0, 1.0, SliderScale::Linear, tone_color,
                        &[(0.0, "CLEAN"), (0.25, "25%"), (0.5, "50%"), (0.75, "75%"), (1.0, "WARM")], None,
                    );
                });
            });
        });

        ui.add_space(15.0);

        // ===== ROW 2: INPUT + MOD + DUCKING + SIDECHAIN =====
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label(egui::RichText::new("INPUT").size(LABEL_FONT).color(Color32::from_gray(140)));
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 5.0;
                    render_vertical_slider_with_ticks(
                        ui, params, setter, &params.synth_reverb_input_hpf, "IHPF",
                        20.0, 22000.0, SliderScale::Logarithmic, input_color,
                        &[(20.0, "20"), (100.0, "100"), (500.0, "500"), (2000.0, "2k"), (10000.0, "10k")], None,
                    );
                    render_vertical_slider_with_ticks(
                        ui, params, setter, &params.synth_reverb_input_lpf, "ILPF",
                        20.0, 22000.0, SliderScale::Logarithmic, input_color,
                        &[(20.0, "20"), (200.0, "200"), (2000.0, "2k"), (10000.0, "10k"), (22000.0, "22k")], None,
                    );
                });
            });

            ui.add_space(25.0);

            ui.vertical(|ui| {
                ui.label(egui::RichText::new("MOD").size(LABEL_FONT).color(Color32::from_gray(140)));
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 5.0;
                    render_vertical_slider_with_ticks(
                        ui, params, setter, &params.synth_reverb_mod_depth, "DPTH",
                        0.0, 1.0, SliderScale::Linear, mod_color,
                        &[(0.0, "OFF"), (0.25, "25%"), (0.5, "50%"), (0.75, "75%"), (1.0, "100%")], None,
                    );
                    render_vertical_slider_with_ticks(
                        ui, params, setter, &params.synth_reverb_mod_speed, "SPD",
                        0.0, 1.0, SliderScale::Linear, mod_color,
                        &[(0.0, "SLOW"), (0.25, "25%"), (0.5, "50%"), (0.75, "75%"), (1.0, "FAST")], None,
                    );
                    render_vertical_slider_with_ticks(
                        ui, params, setter, &params.synth_reverb_mod_shape, "SHPE",
                        0.0, 1.0, SliderScale::Linear, mod_color,
                        &[(0.0, "SIN"), (0.5, "MIX"), (1.0, "RND")], None,
                    );
                });
            });

            ui.add_space(25.0);

            ui.vertical(|ui| {
                ui.label(egui::RichText::new("DUCKING").size(LABEL_FONT).color(Color32::from_gray(140)));
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 5.0;
                    render_vertical_slider_with_ticks(
                        ui, params, setter, &params.synth_reverb_ducking, "DUCK",
                        0.0, 1.0, SliderScale::Linear, duck_color,
                        &[(0.0, "OFF"), (0.25, "-3dB"), (0.5, "-6dB"), (0.75, "-12dB"), (1.0, "-∞")], None,
                    );
                });
            });

            ui.add_space(25.0);

            ui.vertical(|ui| {
                ui.label(egui::RichText::new("SIDECHAIN").size(LABEL_FONT).color(Color32::from_gray(140)));
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 5.0;
                    render_vertical_slider_with_ticks(
                        ui, params, setter, &params.synth_reverb_rhythm_duck_depth, "DPTH",
                        0.0, 1.0, SliderScale::Linear, duck_color,
                        &[(0.0, "OFF"), (0.25, "-3dB"), (0.5, "-6dB"), (0.75, "-12dB"), (1.0, "-∞")], None,
                    );
                    render_vertical_slider_with_ticks(
                        ui, params, setter, &params.synth_reverb_rhythm_duck_smooth, "SMTH",
                        10.0, 300.0, SliderScale::Logarithmic, duck_color,
                        &[(10.0, "10ms"), (50.0, "50"), (100.0, "100"), (200.0, "200"), (300.0, "300")], None,
                    );
                });
            });
        });
    });
}

fn render_comp_tab(
    ui: &mut egui::Ui,
    params: &Arc<DeviceParams>,
    setter: &ParamSetter,
    ui_state: &Arc<SharedUiState>,
) {
    let comp_color = Some(Color32::from_rgb(160, 90, 130));

    let content_rect = ui.max_rect();
    let half_w = content_rect.width() / 2.0;
    let sep_x = content_rect.left() + half_w;
    let margin = FRAME_MARGIN;

    let left_rect = egui::Rect::from_min_max(
        egui::pos2(content_rect.left() + margin.left as f32 + 5.0, content_rect.top() + margin.top as f32),
        egui::pos2(sep_x - 10.0, content_rect.bottom()),
    );
    let mut left_ui = ui.new_child(egui::UiBuilder::new().max_rect(left_rect));
    left_ui.vertical(|ui| {
        ui.label(egui::RichText::new("COMPRESSOR").size(HEADER_FONT).strong());
        ui.add_space(9.0);
        {
            let on = params.comp_enable.value();
            let btn_w = 80.0;
            let btn_h = 48.0;
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 5.0;
                for (label, active) in &[("OFF", false), ("ON", true)] {
                    let is_selected = on == *active;
                    let (bg, text_col) = if is_selected {
                        if *active {
                            (Color32::from_rgb(80, 160, 80), Color32::WHITE)
                        } else {
                            (Color32::from_rgb(180, 60, 60), Color32::WHITE)
                        }
                    } else {
                        (Color32::from_rgb(40, 40, 48), Color32::from_gray(160))
                    };
                    let (rect, response) = ui.allocate_exact_size(
                        egui::vec2(btn_w, btn_h),
                        egui::Sense::click(),
                    );
                    let hover_bg = if response.hovered() && !is_selected {
                        Color32::from_rgb(55, 55, 65)
                    } else {
                        bg
                    };
                    ui.painter().rect_filled(rect, 4.0, hover_bg);
                    if is_selected {
                        let stroke_col = if *active {
                            Color32::from_rgb(100, 190, 100)
                        } else {
                            Color32::from_rgb(210, 80, 80)
                        };
                        ui.painter().rect_stroke(rect, 4.0, egui::Stroke::new(2.0, stroke_col), egui::epaint::StrokeKind::Inside);
                    }
                    let font = egui::FontId::proportional(LABEL_FONT);
                    let galley = ui.painter().layout_no_wrap(label.to_string(), font, text_col);
                    let text_pos = rect.center() - galley.size() / 2.0;
                    ui.painter().galley(text_pos, galley, text_col);
                    if response.clicked() {
                        setter.set_parameter(&params.comp_enable, *active);
                    }
                }

                ui.add_space(8.0);
                render_comp_auto_makeup_toggle(ui, params, setter);
            });
        }

        ui.add_space(13.0);
        ui.label(egui::RichText::new("SIDECHAIN").size(LABEL_FONT).color(Color32::from_gray(140)));
        ui.add_space(6.0);
        render_comp_sc_hpf_buttons(ui, params, setter);

        ui.add_space(13.0);
        ui.label(egui::RichText::new("LOOKAHEAD").size(LABEL_FONT).color(Color32::from_gray(140)));
        ui.add_space(6.0);
        render_comp_lookahead_buttons(ui, params, setter);

        ui.add_space(13.0);
        ui.label(egui::RichText::new("ROUTE").size(LABEL_FONT).color(Color32::from_gray(140)));
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 16.0;
            render_route_toggle(ui, setter, &params.comp_route_master, "MSTR");
            render_route_toggle(ui, setter, &params.comp_route_looper, "LOOP");
            render_route_toggle(ui, setter, &params.comp_route_reverb, "VERB");
        });

        ui.add_space(10.0);
        render_comp_status(ui, ui_state);
    });

    ui.painter().line_segment(
        [egui::pos2(sep_x, content_rect.top()), egui::pos2(sep_x, content_rect.bottom())],
        egui::Stroke::new(1.0, Color32::BLACK),
    );

    let right_rect = egui::Rect::from_min_max(
        egui::pos2(sep_x - 90.0 + margin.left as f32, content_rect.top() + margin.top as f32),
        egui::pos2(content_rect.right() - margin.right as f32, content_rect.bottom()),
    );
    let mut right_ui = ui.new_child(egui::UiBuilder::new().max_rect(right_rect));
    right_ui.vertical(|ui| {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label(egui::RichText::new("DYNAMICS").size(LABEL_FONT).color(Color32::from_gray(140)));
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 5.0;
                    render_vertical_slider_with_ticks(
                        ui, params, setter, &params.comp_threshold, "THRS",
                        -40.0, 0.0, SliderScale::Linear, comp_color,
                        &[(-40.0, "-40dB"), (-30.0, "-30"), (-20.0, "-20"), (-10.0, "-10"), (0.0, "0dB")], None,
                    );
                    render_vertical_slider_with_ticks(
                        ui, params, setter, &params.comp_ratio, "RTO",
                        1.0, 20.0, SliderScale::Exponential(0.5), comp_color,
                        &[(1.0, "1:1"), (2.0, "2:1"), (4.0, "4:1"), (8.0, "8:1"), (20.0, "20:1")], None,
                    );
                    render_vertical_slider_with_ticks(
                        ui, params, setter, &params.comp_attack, "ATK",
                        0.1, 100.0, SliderScale::Logarithmic, comp_color,
                        &[(0.1, "0.1ms"), (1.0, "1"), (10.0, "10"), (50.0, "50"), (100.0, "100")], None,
                    );
                    render_vertical_slider_with_ticks(
                        ui, params, setter, &params.comp_release, "REL",
                        5.0, 2000.0, SliderScale::Logarithmic, comp_color,
                        &[(5.0, "5ms"), (50.0, "50"), (200.0, "200"), (1000.0, "1k"), (2000.0, "2k")], None,
                    );
                    render_vertical_slider_with_ticks(
                        ui, params, setter, &params.comp_knee, "KNEE",
                        0.0, 12.0, SliderScale::Linear, comp_color,
                        &[(0.0, "HARD"), (3.0, "3dB"), (6.0, "6dB"), (9.0, "9dB"), (12.0, "12dB")], None,
                    );
                });
            });

            ui.add_space(25.0);

            ui.vertical(|ui| {
                ui.label(egui::RichText::new("OUTPUT").size(LABEL_FONT).color(Color32::from_gray(140)));
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 5.0;
                    render_vertical_slider_with_ticks(
                        ui, params, setter, &params.comp_makeup, "MKUP",
                        0.0, 24.0, SliderScale::Linear, comp_color,
                        &[(0.0, "0dB"), (6.0, "6"), (12.0, "12"), (18.0, "18"), (24.0, "24dB")], None,
                    );
                    render_vertical_slider_with_ticks(
                        ui, params, setter, &params.comp_mix, "MIX",
                        0.0, 1.0, SliderScale::Linear, comp_color,
                        &[(0.0, "DRY"), (0.25, "-12dB"), (0.5, "-6dB"), (0.75, "-3dB"), (1.0, "WET")], None,
                    );
                    render_vertical_slider_with_ticks(
                        ui, params, setter, &params.comp_stereo_link, "LINK",
                        0.0, 1.0, SliderScale::Linear, comp_color,
                        &[(0.0, "DUAL"), (0.25, "25%"), (0.5, "50%"), (0.75, "75%"), (1.0, "STEREO")], None,
                    );
                });
            });
        });
    });
}

fn render_comp_auto_makeup_toggle(
    ui: &mut egui::Ui,
    params: &Arc<DeviceParams>,
    setter: &ParamSetter,
) {
    let is_on = params.comp_auto_makeup.value();
    let btn_w = 56.0;
    let btn_h = 48.0;
    let (bg, text_col) = if is_on {
        (Color32::from_rgb(80, 60, 70), Color32::WHITE)
    } else {
        (Color32::from_rgb(40, 40, 48), Color32::from_gray(140))
    };
    let (rect, response) = ui.allocate_exact_size(
        egui::vec2(btn_w, btn_h),
        egui::Sense::click(),
    );
    let hover_bg = if response.hovered() && !is_on {
        Color32::from_rgb(55, 55, 65)
    } else {
        bg
    };
    ui.painter().rect_filled(rect, 4.0, hover_bg);
    if is_on {
        ui.painter().rect_stroke(rect, 4.0, egui::Stroke::new(2.0, Color32::from_rgb(160, 90, 130)), egui::epaint::StrokeKind::Inside);
    }
    let font = egui::FontId::proportional(12.0);
    let galley = ui.painter().layout_no_wrap("AUTO".to_string(), font, text_col);
    ui.painter().galley(
        egui::pos2(rect.center().x - galley.size().x / 2.0, rect.center().y - galley.size().y / 2.0),
        galley,
        text_col,
    );
    if response.clicked() {
        setter.set_parameter(&params.comp_auto_makeup, !is_on);
    }
}

fn render_comp_status(
    ui: &mut egui::Ui,
    ui_state: &Arc<SharedUiState>,
) {
    let comp_latency = ui_state.comp_latency_samples.load(Ordering::Relaxed);
    let sample_rate = ui_state.sample_rate.load(Ordering::Relaxed) as f32;
    let gr_raw = ui_state.comp_gr_db.load(Ordering::Relaxed);
    let gr_db = gr_raw as f32 / 100.0;

    if gr_db > 0.01 {
        let gr_bar_width = 180.0_f32;
        let gr_bar_height = 12.0_f32;
        let (bar_rect, _) = ui.allocate_exact_size(
            egui::vec2(gr_bar_width, gr_bar_height),
            egui::Sense::hover(),
        );
        ui.painter().rect_filled(bar_rect, 2.0, Color32::from_rgb(25, 25, 30));
        let fill_frac = (gr_db / 30.0).min(1.0);
        let fill_rect = egui::Rect::from_min_max(
            bar_rect.min,
            egui::pos2(bar_rect.left() + gr_bar_width * fill_frac, bar_rect.bottom()),
        );
        let gr_color = if gr_db > 12.0 {
            Color32::from_rgb(200, 80, 60)
        } else if gr_db > 6.0 {
            Color32::from_rgb(200, 160, 60)
        } else {
            Color32::from_rgb(160, 90, 130)
        };
        ui.painter().rect_filled(fill_rect, 2.0, gr_color);
        let font = egui::FontId::proportional(10.0);
        let text = format!("GR: -{:.1}dB", gr_db);
        let galley = ui.painter().layout_no_wrap(text, font, Color32::from_gray(200));
        ui.painter().galley(
            egui::pos2(bar_rect.left() + 4.0, bar_rect.center().y - galley.size().y / 2.0),
            galley,
            Color32::from_gray(200),
        );
        ui.add_space(4.0);
    }

    if comp_latency > 0 && sample_rate > 0.0 {
        let latency_ms = comp_latency as f32 / sample_rate * 1000.0;
        ui.label(
            egui::RichText::new(format!(
                "Lookahead: {} smp ({:.1}ms)",
                comp_latency, latency_ms
            ))
            .size(11.0)
            .color(Color32::from_gray(100)),
        );
    }
}

fn render_comp_sc_hpf_buttons(
    ui: &mut egui::Ui,
    params: &Arc<DeviceParams>,
    setter: &ParamSetter,
) {
    let current = params.comp_sc_hpf.value();
    let labels = ["OFF", "80", "150", "250"];

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 4.0;
        for (i, label) in labels.iter().enumerate() {
            let is_selected = current == i as i32;
            let (bg, text_col) = if is_selected {
                (Color32::from_rgb(80, 60, 70), Color32::WHITE)
            } else {
                (Color32::from_rgb(40, 40, 48), Color32::from_gray(140))
            };

            let (rect, response) = ui.allocate_exact_size(
                egui::vec2(48.0, 32.0),
                egui::Sense::click(),
            );

            let hover_bg = if response.hovered() && !is_selected {
                Color32::from_rgb(55, 55, 65)
            } else {
                bg
            };

            ui.painter().rect_filled(rect, 4.0, hover_bg);
            if is_selected {
                ui.painter().rect_stroke(rect, 4.0, egui::Stroke::new(2.0, Color32::from_rgb(160, 90, 130)), egui::epaint::StrokeKind::Inside);
            }

            let font = egui::FontId::proportional(14.0);
            let galley = ui.painter().layout_no_wrap(label.to_string(), font, text_col);
            ui.painter().galley(
                egui::pos2(rect.center().x - galley.size().x / 2.0, rect.center().y - galley.size().y / 2.0),
                galley,
                text_col,
            );

            if response.clicked() {
                setter.set_parameter(&params.comp_sc_hpf, i as i32);
            }
        }
    });
}

fn render_comp_lookahead_buttons(
    ui: &mut egui::Ui,
    params: &Arc<DeviceParams>,
    setter: &ParamSetter,
) {
    let current = params.comp_lookahead.value();
    let labels = ["OFF", "1ms", "2.5", "5ms"];

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 4.0;
        for (i, label) in labels.iter().enumerate() {
            let is_selected = current == i as i32;
            let (bg, text_col) = if is_selected {
                (Color32::from_rgb(80, 60, 70), Color32::WHITE)
            } else {
                (Color32::from_rgb(40, 40, 48), Color32::from_gray(140))
            };

            let (rect, response) = ui.allocate_exact_size(
                egui::vec2(48.0, 32.0),
                egui::Sense::click(),
            );

            let hover_bg = if response.hovered() && !is_selected {
                Color32::from_rgb(55, 55, 65)
            } else {
                bg
            };

            ui.painter().rect_filled(rect, 4.0, hover_bg);
            if is_selected {
                ui.painter().rect_stroke(rect, 4.0, egui::Stroke::new(2.0, Color32::from_rgb(160, 90, 130)), egui::epaint::StrokeKind::Inside);
            }

            let font = egui::FontId::proportional(14.0);
            let galley = ui.painter().layout_no_wrap(label.to_string(), font, text_col);
            ui.painter().galley(
                egui::pos2(rect.center().x - galley.size().x / 2.0, rect.center().y - galley.size().y / 2.0),
                galley,
                text_col,
            );

            if response.clicked() {
                setter.set_parameter(&params.comp_lookahead, i as i32);
            }
        }
    });
}

fn render_toggle_switch(
    ui: &mut egui::Ui,
    _params: &Arc<DeviceParams>,
    setter: &ParamSetter,
    param: &nih_plug::prelude::IntParam,
    label_off: &str,
    label_on: &str,
) {
    let mut is_on = param.value() == 1;
    let desired_size = egui::vec2(48.0, 24.0);
    let (alloc_rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::click());
    if response.clicked() {
        is_on = !is_on;
        setter.set_parameter(param, if is_on { 1 } else { 0 });
    }
    let rect = alloc_rect.translate(egui::vec2(0.0, -2.0));
    let anim_t = ui.ctx().animate_bool_with_time(response.id, is_on, 0.15);
    let bg = Color32::from_gray(50).lerp_to_gamma(Color32::from_rgb(80, 130, 190), anim_t);
    let circle_x = egui::lerp(rect.left() + 12.0..=rect.right() - 12.0, anim_t);
    let circle_color = Color32::from_gray(220).lerp_to_gamma(Color32::WHITE, anim_t);
    ui.painter().rect_filled(rect, rect.height() / 2.0, bg);
    ui.painter().circle_filled(egui::pos2(circle_x, rect.center().y), 9.0, circle_color);
    let label = if is_on { label_on } else { label_off };
    ui.label(egui::RichText::new(label).size(LABEL_FONT).color(Color32::from_gray(140)));
}

fn render_octave_buttons(
    ui: &mut egui::Ui,
    setter: &ParamSetter,
    param: &nih_plug::prelude::FloatParam,
    color: Color32,
) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 2.0;
        for (label, delta) in &[("-12", -12.0f32), ("+12", 12.0f32)] {
            let (rect, response) = ui.allocate_exact_size(egui::vec2(28.0, 24.0), egui::Sense::click());
            let bg = if response.hovered() {
                Color32::from_rgb(55, 55, 65)
            } else {
                Color32::from_rgb(40, 40, 48)
            };
            ui.painter().rect_filled(rect, 3.0, bg);
            let font = egui::FontId::proportional(11.0);
            let galley = ui.painter().layout_no_wrap(label.to_string(), font, color);
            let text_pos = rect.center() - galley.size() / 2.0;
            ui.painter().galley(text_pos, galley, color);
            if response.clicked() {
                let current = param.modulated_plain_value();
                let rounded = (current / 12.0).round() * 12.0;
                let new_val = (rounded + delta).clamp(-24.0, 24.0);
                setter.set_parameter(param, new_val);
            }
        }
    });
}

fn render_looper_section(
    ui: &mut egui::Ui,
    params: &Arc<DeviceParams>,
    setter: &ParamSetter,
) {
    let looper_color = Color32::from_rgb(200, 100, 50);

    ui.vertical(|ui| {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("LOOPER").size(HEADER_FONT).strong());
            ui.add_space(8.0);
            render_looper_bool_button(ui, setter, &params.looper_enabled, "ON",
                Color32::from_rgb(60, 160, 60), 68.0, 48.0);
        });

        ui.add_space(6.0);
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 5.0;
            let current_dir = params.looper_direction.value();
            for (label, val) in &[("FWD", 0), ("REV", 1), ("PING", 2)] {
                render_looper_select_button(ui, setter, &params.looper_direction,
                    label, *val, current_dir, looper_color, 68.0, 48.0);
            }
            ui.add_space(8.0);
            render_looper_bool_button(ui, setter, &params.looper_freeze, "FRZ",
                Color32::from_rgb(80, 140, 200), 68.0, 48.0);
            render_looper_bool_button(ui, setter, &params.looper_key_track, "KEY",
                Color32::from_rgb(180, 140, 60), 68.0, 48.0);
        });

        ui.add_space(8.0);
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 5.0;
            ui.vertical(|ui| {
                render_vertical_slider_with_ticks(
                    ui, params, setter,
                    &params.looper_pitch, "PTCH",
                    -24.0, 24.0, SliderScale::Linear,
                    Some(Color32::from_rgb(130, 80, 160)),
                    &[(-24.0, "-24"), (-12.0, "-12"), (0.0, "0"), (12.0, "+12"), (24.0, "+24")],
                    None,
                );
                render_octave_buttons(ui, setter, &params.looper_pitch, Color32::from_rgb(130, 80, 160));
            });
            ui.vertical(|ui| {
                render_vertical_slider_with_ticks(
                    ui, params, setter,
                    &params.looper_doppler, "DPLR",
                    -24.0, 24.0, SliderScale::Linear,
                    Some(Color32::from_rgb(160, 120, 60)),
                    &[(-24.0, "-24"), (-12.0, "-12"), (0.0, "0"), (12.0, "+12"), (24.0, "+24")],
                    None,
                );
                render_octave_buttons(ui, setter, &params.looper_doppler, Color32::from_rgb(160, 120, 60));
            });
            render_vertical_slider_with_ticks(
                ui, params, setter,
                &params.looper_decay, "DCY",
                0.0, 1.0, SliderScale::Linear,
                Some(Color32::from_rgb(100, 80, 120)),
                &[(0.0, "0%"), (0.25, "25%"), (0.5, "50%"), (0.75, "75%"), (1.0, "100%")],
                None,
            );
            render_vertical_slider_with_ticks(
                ui, params, setter,
                &params.looper_start, "STRT",
                0.0, 1.0, SliderScale::Linear,
                Some(Color32::from_rgb(60, 120, 110)),
                &[(0.0, "0%"), (0.25, "25%"), (0.5, "50%"), (0.75, "75%"), (1.0, "100%")],
                None,
            );
            render_vertical_slider_with_ticks(
                ui, params, setter,
                &params.looper_mix, "VOL",
                0.0, 3.0, SliderScale::Linear,
                Some(Color32::from_rgb(200, 100, 50)),
                &[(0.0, "-∞"), (0.5, "-6dB"), (1.0, "0dB"), (2.0, "+6dB"), (3.0, "+10dB")],
                None,
            );
        });

        ui.add_space(6.0);
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("LEN").size(12.0).color(Color32::from_gray(120)));
            ui.add_space(2.0);
            render_looper_division_combo(ui, setter, &params.looper_length,
                "looper_len", 100.0, looper_color);
        });

        ui.add_space(4.0);
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 5.0;
            ui.label(egui::RichText::new("STT").size(12.0).color(Color32::from_gray(120)));
            ui.add_space(2.0);
            let current_stt = params.looper_stutter.value();
            for (label, val) in &[("OFF", 0), ("2", 1), ("4", 2), ("8", 3), ("16", 4)] {
                render_looper_select_button(ui, setter, &params.looper_stutter,
                    label, *val, current_stt, looper_color, 52.0, 48.0);
            }
        });

        ui.add_space(4.0);
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 5.0;
            ui.label(egui::RichText::new("REC").size(12.0).color(Color32::from_gray(120)));
            ui.add_space(2.0);
            render_looper_division_combo(ui, setter, &params.looper_auto_rec_len,
                "looper_auto_len", 80.0, looper_color);
            ui.add_space(3.0);
            ui.label(egui::RichText::new("/").size(12.0).color(Color32::from_gray(120)));
            let current_int = params.looper_auto_rec_interval.value();
            for (label, val) in &[("1", 0), ("2", 1), ("4", 2), ("8", 3)] {
                render_looper_select_button(ui, setter, &params.looper_auto_rec_interval,
                    label, *val, current_int, looper_color, 52.0, 48.0);
            }
            ui.label(egui::RichText::new("bar").size(12.0).color(Color32::from_gray(120)));
        });

        ui.add_space(8.0);
        let lp_premaster = params.looper_input_premaster.value();
        let lp_filter = params.looper_input_filter.value();
        let oscs_dimmed = lp_premaster || lp_filter;
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 16.0;
            render_route_toggle_dimmed(ui, setter, &params.looper_input_vps, "VPS", oscs_dimmed);
            render_route_toggle_dimmed(ui, setter, &params.looper_input_pll, "PLL", oscs_dimmed);
        });
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 16.0;
            render_route_toggle_dimmed(ui, setter, &params.looper_input_saw, "SAW", oscs_dimmed);
            render_route_toggle_dimmed(ui, setter, &params.looper_input_filter, "FLTR", lp_premaster);
        });
        ui.add_space(6.0);
        render_route_toggle(ui, setter, &params.looper_input_premaster, "PRE");
    });
}

fn render_looper_bool_button(
    ui: &mut egui::Ui,
    setter: &ParamSetter,
    param: &nih_plug::prelude::BoolParam,
    label: &str,
    on_color: Color32,
    w: f32,
    h: f32,
) {
    let is_on = param.value();
    let (rect, response) = ui.allocate_exact_size(egui::vec2(w, h), egui::Sense::click());

    if response.clicked() {
        setter.set_parameter(param, !is_on);
    }

    let (bg, text_col) = if is_on {
        (on_color, Color32::WHITE)
    } else {
        let hover = if response.hovered() { Color32::from_rgb(55, 55, 65) } else { Color32::from_rgb(40, 40, 48) };
        (hover, Color32::from_gray(160))
    };

    ui.painter().rect_filled(rect, 4.0, bg);
    if is_on {
        ui.painter().rect_stroke(rect, 4.0,
            egui::Stroke::new(2.0, Color32::from_rgb(
                on_color.r().saturating_add(40),
                on_color.g().saturating_add(40),
                on_color.b().saturating_add(40),
            )),
            egui::epaint::StrokeKind::Inside,
        );
    }

    let font = egui::FontId::proportional(LABEL_FONT);
    let galley = ui.painter().layout_no_wrap(label.to_string(), font, text_col);
    let text_pos = rect.center() - galley.size() / 2.0;
    ui.painter().galley(text_pos, galley, text_col);
}

fn render_looper_select_button(
    ui: &mut egui::Ui,
    setter: &ParamSetter,
    param: &nih_plug::prelude::IntParam,
    label: &str,
    value: i32,
    current: i32,
    accent: Color32,
    w: f32,
    h: f32,
) {
    let is_selected = current == value;
    let (rect, response) = ui.allocate_exact_size(egui::vec2(w, h), egui::Sense::click());

    let (bg, text_col) = if is_selected {
        (accent, Color32::WHITE)
    } else {
        let hover = if response.hovered() { Color32::from_rgb(55, 55, 65) } else { Color32::from_rgb(40, 40, 48) };
        (hover, Color32::from_gray(160))
    };

    ui.painter().rect_filled(rect, 4.0, bg);
    if is_selected {
        ui.painter().rect_stroke(rect, 4.0,
            egui::Stroke::new(2.0, Color32::from_rgb(
                accent.r().saturating_add(30),
                accent.g().saturating_add(20),
                accent.b().saturating_add(20),
            )),
            egui::epaint::StrokeKind::Inside,
        );
    }

    let font = egui::FontId::proportional(LABEL_FONT);
    let galley = ui.painter().layout_no_wrap(label.to_string(), font, text_col);
    let text_pos = rect.center() - galley.size() / 2.0;
    ui.painter().galley(text_pos, galley, text_col);

    if response.clicked() {
        setter.set_parameter(param, value);
    }
}

fn render_s_mode_button(
    ui: &mut egui::Ui,
    setter: &ParamSetter,
    param: &nih_plug::prelude::BoolParam,
    label: &str,
    accent: Color32,
    w: f32,
    h: f32,
) {
    let is_on = param.value();
    let (rect, response) = ui.allocate_exact_size(egui::vec2(w, h), egui::Sense::click());

    let (bg, text_col) = if is_on {
        (accent, Color32::WHITE)
    } else {
        let hover = if response.hovered() { Color32::from_rgb(55, 55, 65) } else { Color32::from_rgb(40, 40, 48) };
        (hover, Color32::from_gray(160))
    };

    ui.painter().rect_filled(rect, 4.0, bg);
    if is_on {
        ui.painter().rect_stroke(rect, 4.0,
            egui::Stroke::new(2.0, Color32::from_rgb(
                accent.r().saturating_add(30),
                accent.g().saturating_add(20),
                accent.b().saturating_add(20),
            )),
            egui::epaint::StrokeKind::Inside,
        );
    }

    let font = egui::FontId::proportional(LABEL_FONT);
    let galley = ui.painter().layout_no_wrap(label.to_string(), font, text_col);
    let text_pos = rect.center() - galley.size() / 2.0;
    ui.painter().galley(text_pos, galley, text_col);

    if response.clicked() {
        setter.set_parameter(param, !is_on);
    }
}

const LOOPER_DIV_NAMES: [&str; 18] = [
    "1/1", "1/2", "1/4", "1/8", "1/16", "1/32",
    "1/2.", "1/4.", "1/8.", "1/16.",
    "1/2T", "1/4T", "1/8T", "1/16T",
    "2/1", "4/1",
    "1/64", "1/128",
];
const LOOPER_DIV_ORDER: [usize; 18] = [
    15, 14, 0, 1, 2, 3, 4, 5, 16, 17,
    6, 7, 8, 9,
    10, 11, 12, 13,
];

fn render_looper_division_combo(
    ui: &mut egui::Ui,
    setter: &ParamSetter,
    param: &nih_plug::prelude::IntParam,
    id: &str,
    width: f32,
    accent: Color32,
) {
    let current = param.value() as usize;
    let current_name = LOOPER_DIV_NAMES.get(current).copied().unwrap_or("?");
    let mut selected = None;

    ui.style_mut().visuals.widgets.inactive.bg_fill = Color32::from_rgb(
        accent.r() / 3, accent.g() / 3, accent.b() / 3,
    );
    ui.style_mut().spacing.button_padding = egui::vec2(6.0, 4.0);

    egui::ComboBox::from_id_salt(id)
        .width(width)
        .height(380.0)
        .selected_text(egui::RichText::new(current_name).size(13.0))
        .show_ui(ui, |ui| {
            ui.style_mut().spacing.item_spacing.y = 2.0;
            for &idx in &LOOPER_DIV_ORDER {
                let name = LOOPER_DIV_NAMES[idx];
                let btn = egui::Button::new(egui::RichText::new(name).size(13.0))
                    .min_size(egui::vec2(width - 10.0, 28.0))
                    .selected(current == idx);
                if ui.add(btn).clicked() {
                    selected = Some(idx);
                    ui.close_menu();
                }
            }
        });
    if let Some(idx) = selected {
        setter.set_parameter(param, idx as i32);
    }
}

fn render_hpf_buttons(
    ui: &mut egui::Ui,
    params: &Arc<DeviceParams>,
    setter: &ParamSetter,
) {
    let current = params.master_hpf.value();
    let options = [("OFF", 0), ("35", 1), ("80", 2), ("120", 3), ("220", 4)];
    let btn_w = 64.0;
    let btn_h = 48.0;

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 5.0;
        for (label, value) in &options {
            let is_selected = current == *value;
            let (bg, text_col) = if is_selected {
                (Color32::from_rgb(80, 120, 180), Color32::WHITE)
            } else {
                (Color32::from_rgb(40, 40, 48), Color32::from_gray(160))
            };

            let (rect, response) = ui.allocate_exact_size(
                egui::vec2(btn_w, btn_h),
                egui::Sense::click(),
            );

            let hover_bg = if response.hovered() && !is_selected {
                Color32::from_rgb(55, 55, 65)
            } else {
                bg
            };

            ui.painter().rect_filled(rect, 4.0, hover_bg);
            if is_selected {
                ui.painter().rect_stroke(rect, 4.0, egui::Stroke::new(2.0, Color32::from_rgb(100, 150, 220)), egui::epaint::StrokeKind::Inside);
            }

            let font = egui::FontId::proportional(LABEL_FONT);
            let galley = ui.painter().layout_no_wrap(label.to_string(), font, text_col);
            let text_pos = rect.center() - galley.size() / 2.0;
            ui.painter().galley(text_pos, galley, text_col);

            if response.clicked() {
                setter.set_parameter(&params.master_hpf, *value);
            }
        }
    });
}

fn render_hpf_boost_buttons(
    ui: &mut egui::Ui,
    params: &Arc<DeviceParams>,
    setter: &ParamSetter,
) {
    let current = params.master_hpf_boost.value();
    let hpf_off = params.master_hpf.value() == 0;
    let options = [("FLAT", 0), ("MED", 1), ("HIGH", 2)];
    let btn_w = 80.0;
    let btn_h = 48.0;

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 5.0;
        for (label, value) in &options {
            let is_selected = current == *value;
            let (bg, text_col) = if hpf_off {
                (Color32::from_rgb(30, 30, 34), Color32::from_gray(70))
            } else if is_selected {
                (Color32::from_rgb(80, 120, 180), Color32::WHITE)
            } else {
                (Color32::from_rgb(40, 40, 48), Color32::from_gray(160))
            };

            let (rect, response) = ui.allocate_exact_size(
                egui::vec2(btn_w, btn_h),
                egui::Sense::click(),
            );

            let hover_bg = if hpf_off {
                bg
            } else if response.hovered() && !is_selected {
                Color32::from_rgb(55, 55, 65)
            } else {
                bg
            };

            ui.painter().rect_filled(rect, 4.0, hover_bg);
            if is_selected && !hpf_off {
                ui.painter().rect_stroke(rect, 4.0, egui::Stroke::new(2.0, Color32::from_rgb(100, 150, 220)), egui::epaint::StrokeKind::Inside);
            }

            let font = egui::FontId::proportional(LABEL_FONT);
            let galley = ui.painter().layout_no_wrap(label.to_string(), font, text_col);
            let text_pos = rect.center() - galley.size() / 2.0;
            ui.painter().galley(text_pos, galley, text_col);

            if response.clicked() && !hpf_off {
                setter.set_parameter(&params.master_hpf_boost, *value);
            }
        }
    });
}


fn render_box_cut_buttons(
    ui: &mut egui::Ui,
    params: &Arc<DeviceParams>,
    setter: &ParamSetter,
) {
    let current = params.box_cut_mode.value();
    let options = [("OFF", 0), ("LOW", 1), ("MED", 2), ("HIGH", 3)];
    let btn_w = 64.0;
    let btn_h = 48.0;

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 5.0;
        for (label, value) in &options {
            let is_selected = current == *value;
            let (bg, text_col) = if is_selected {
                (Color32::from_rgb(140, 100, 60), Color32::WHITE)
            } else {
                (Color32::from_rgb(40, 40, 48), Color32::from_gray(160))
            };

            let (rect, response) = ui.allocate_exact_size(
                egui::vec2(btn_w, btn_h),
                egui::Sense::click(),
            );

            let hover_bg = if response.hovered() && !is_selected {
                Color32::from_rgb(55, 55, 65)
            } else {
                bg
            };

            ui.painter().rect_filled(rect, 4.0, hover_bg);
            if is_selected {
                ui.painter().rect_stroke(rect, 4.0, egui::Stroke::new(2.0, Color32::from_rgb(170, 130, 80)), egui::epaint::StrokeKind::Inside);
            }

            let font = egui::FontId::proportional(LABEL_FONT);
            let galley = ui.painter().layout_no_wrap(label.to_string(), font, text_col);
            let text_pos = rect.center() - galley.size() / 2.0;
            ui.painter().galley(text_pos, galley, text_col);

            if response.clicked() {
                setter.set_parameter(&params.box_cut_mode, *value);
            }
        }
    });
}

fn draw_cc_pickup_indicator(
    ui: &egui::Ui,
    rail_rect: egui::Rect,
    midi_learn: &Option<(&MidiLearnState, &str)>,
) {
    let Some((ml, param_id)) = midi_learn else { return };
    if !is_selected_param(ml, param_id) { return; }
    if ml.value_cc_picked_up.load(Ordering::Relaxed) { return; }
    let value_cc = ml.value_cc.load(Ordering::Relaxed);
    if value_cc >= 128 { return; }

    let cc_val = ml.read_cc(value_cc);

    let handle_radius = SLIDER_RAIL_THICKNESS * 0.55;
    let rail_top = rail_rect.top() + handle_radius;
    let rail_bottom = rail_rect.bottom() - handle_radius;
    let y = rail_bottom - cc_val * (rail_bottom - rail_top);

    let indicator_color = Color32::from_rgba_premultiplied(255, 160, 40, 180);
    let cx = rail_rect.center().x;
    let hw = SLIDER_RAIL_THICKNESS * 0.6;

    ui.painter().line_segment(
        [egui::pos2(cx - hw, y), egui::pos2(cx + hw, y)],
        egui::Stroke::new(2.0, indicator_color),
    );

    let tri_size = 4.0;
    ui.painter().add(egui::Shape::convex_polygon(
        vec![
            egui::pos2(cx - hw - 1.0, y - tri_size),
            egui::pos2(cx - hw - 1.0, y + tri_size),
            egui::pos2(cx - hw - tri_size - 1.0, y),
        ],
        indicator_color,
        egui::Stroke::NONE,
    ));
    ui.painter().add(egui::Shape::convex_polygon(
        vec![
            egui::pos2(cx + hw + 1.0, y - tri_size),
            egui::pos2(cx + hw + 1.0, y + tri_size),
            egui::pos2(cx + hw + tri_size + 1.0, y),
        ],
        indicator_color,
        egui::Stroke::NONE,
    ));
}

fn is_selected_param(ml: &MidiLearnState, param_id: &str) -> bool {
    let sel_cc = ml.selector_cc.load(Ordering::Relaxed);
    if sel_cc >= 128 { return false; }
    let idx = ml.selected_param_idx.load(Ordering::Relaxed) as usize;
    idx < SOUND_PARAMS.len() && SOUND_PARAMS[idx] == param_id
}

enum SliderScale {
    Linear,
    Logarithmic,
    Exponential(f32),
}

fn render_vertical_slider<P: Param>(
    ui: &mut egui::Ui,
    params: &Arc<DeviceParams>,
    setter: &ParamSetter,
    param: &P,
    label: &str,
    min: f32,
    max: f32,
    scale: SliderScale,
    color: Option<Color32>,
    midi_learn: Option<(&MidiLearnState, &str)>,
) where
    P::Plain: Into<f32>,
    f32: Into<P::Plain>,
{
    ui.vertical(|ui| {
        ui.set_width(SLIDER_COL_WIDTH);
        let plain_value = param.modulated_plain_value();
        let mut value: f32 = plain_value.into();

        let learn_active = midi_learn.as_ref()
            .map(|(ml, _)| ml.learn_active.load(Ordering::Relaxed))
            .unwrap_or(false);

        if let Some(fill_color) = color {
            if learn_active {
                let brighter = Color32::from_rgb(
                    fill_color.r().saturating_add(30),
                    fill_color.g().saturating_add(20),
                    fill_color.b().saturating_add(10),
                );
                ui.style_mut().visuals.widgets.inactive.bg_fill = brighter;
                ui.style_mut().visuals.widgets.hovered.bg_fill = brighter;
                ui.style_mut().visuals.widgets.active.bg_fill = brighter;
            } else {
                ui.style_mut().visuals.widgets.inactive.bg_fill = fill_color;
                ui.style_mut().visuals.widgets.hovered.bg_fill = fill_color;
                ui.style_mut().visuals.widgets.active.bg_fill = fill_color;
            }
        }

        ui.style_mut().spacing.slider_width = SLIDER_RAIL_LENGTH;
        ui.style_mut().spacing.slider_rail_height = SLIDER_RAIL_THICKNESS;

        let response = match scale {
            SliderScale::Linear => {
                let slider = egui::Slider::new(&mut value, min..=max)
                    .vertical()
                    .show_value(false);
                let r = ui.add(slider);
                if r.changed() {
                    setter.set_parameter(param, value.into());
                }
                r
            }
            SliderScale::Logarithmic => {
                let slider = egui::Slider::new(&mut value, min..=max)
                    .vertical()
                    .logarithmic(true)
                    .show_value(false);
                let r = ui.add(slider);
                if r.changed() {
                    setter.set_parameter(param, value.into());
                }
                r
            }
            SliderScale::Exponential(exponent) => {
                let normalized = (value - min) / (max - min);
                let mut slider_value = normalized.powf(1.0 / exponent);

                let slider = egui::Slider::new(&mut slider_value, 0.0..=1.0)
                    .vertical()
                    .show_value(false);

                let r = ui.add(slider);
                if r.changed() {
                    let new_normalized = slider_value.powf(exponent);
                    value = min + new_normalized * (max - min);
                    setter.set_parameter(param, value.into());
                }
                r
            }
        };

        if let Some((ml, param_id)) = &midi_learn {
            if learn_active && (response.drag_started() || response.clicked()) {
                if let Ok(mut awaiting) = ml.awaiting_param.try_lock() {
                    *awaiting = Some(param_id.to_string());
                }
            }
        }

        draw_cc_pickup_indicator(ui, response.rect, &midi_learn);

        if let Some((_, param_id)) = &midi_learn {
            if let Some(mod_idx) = param_id_to_mod_dest_index(param_id) {
                let depth = get_lfo_max_depth(params, mod_idx);
                if depth > 0.001 {
                    draw_lfo_depth_indicator(ui, response.rect, value, min, max, depth);
                }
            }
        }

        ui.add_space(2.0);
        let has_cc = midi_learn.as_ref().and_then(|(ml, param_id)| {
            ml.mappings.try_lock().ok().and_then(|m| m.find_by_param(param_id))
        }).is_some();
        let selected = midi_learn.as_ref()
            .map(|(ml, param_id)| is_selected_param(ml, param_id))
            .unwrap_or(false);
        ui.horizontal(|ui| {
            let offset = match label.chars().count() {
                1 => 5.0,
                2 => -1.0,
                3 => -7.0,
                _ => -12.0,
            };
            ui.add_space(offset);
            let label_text = if selected {
                egui::RichText::new(label).size(LABEL_FONT).color(SELECTED_LABEL_COLOR).strong()
            } else if has_cc {
                egui::RichText::new(label).size(LABEL_FONT).color(MIDI_ASSIGNED_COLOR)
            } else {
                egui::RichText::new(label).size(LABEL_FONT)
            };
            ui.label(label_text);
        });
    });
}

fn render_vertical_slider_with_ticks<P: Param>(
    ui: &mut egui::Ui,
    params: &Arc<DeviceParams>,
    setter: &ParamSetter,
    param: &P,
    label: &str,
    min: f32,
    max: f32,
    scale: SliderScale,
    color: Option<Color32>,
    ticks: &[(f32, &str)],
    midi_learn: Option<(&MidiLearnState, &str)>,
) where
    P::Plain: Into<f32>,
    f32: Into<P::Plain>,
{
    ui.vertical(|ui| {
        ui.set_width(SLIDER_COL_WIDTH);
        let plain_value = param.modulated_plain_value();
        let mut value: f32 = plain_value.into();

        let learn_active = midi_learn.as_ref()
            .map(|(ml, _)| ml.learn_active.load(Ordering::Relaxed))
            .unwrap_or(false);

        if let Some(fill_color) = color {
            if learn_active {
                let brighter = Color32::from_rgb(
                    fill_color.r().saturating_add(30),
                    fill_color.g().saturating_add(20),
                    fill_color.b().saturating_add(10),
                );
                ui.style_mut().visuals.widgets.inactive.bg_fill = brighter;
                ui.style_mut().visuals.widgets.hovered.bg_fill = brighter;
                ui.style_mut().visuals.widgets.active.bg_fill = brighter;
            } else {
                ui.style_mut().visuals.widgets.inactive.bg_fill = fill_color;
                ui.style_mut().visuals.widgets.hovered.bg_fill = fill_color;
                ui.style_mut().visuals.widgets.active.bg_fill = fill_color;
            }
        }

        ui.style_mut().spacing.slider_width = SLIDER_RAIL_LENGTH;
        ui.style_mut().spacing.slider_rail_height = SLIDER_RAIL_THICKNESS;

        let scale_exp = match &scale { SliderScale::Exponential(e) => Some(*e), _ => None };
        let scale_is_log = matches!(&scale, SliderScale::Logarithmic);

        let response = match scale {
            SliderScale::Linear => {
                let slider = egui::Slider::new(&mut value, min..=max)
                    .vertical()
                    .show_value(false);
                let r = ui.add(slider);
                if r.changed() { setter.set_parameter(param, value.into()); }
                r
            }
            SliderScale::Logarithmic => {
                let slider = egui::Slider::new(&mut value, min..=max)
                    .vertical()
                    .logarithmic(true)
                    .show_value(false);
                let r = ui.add(slider);
                if r.changed() { setter.set_parameter(param, value.into()); }
                r
            }
            SliderScale::Exponential(exponent) => {
                let normalized = (value - min) / (max - min);
                let mut slider_value = normalized.powf(1.0 / exponent);
                let slider = egui::Slider::new(&mut slider_value, 0.0..=1.0)
                    .vertical()
                    .show_value(false);
                let r = ui.add(slider);
                if r.changed() {
                    let new_normalized = slider_value.powf(exponent);
                    value = min + new_normalized * (max - min);
                    setter.set_parameter(param, value.into());
                }
                r
            }
        };

        if let Some((ml, param_id)) = &midi_learn {
            if learn_active && (response.drag_started() || response.clicked()) {
                if let Ok(mut awaiting) = ml.awaiting_param.try_lock() {
                    *awaiting = Some(param_id.to_string());
                }
            }
        }

        draw_cc_pickup_indicator(ui, response.rect, &midi_learn);

        if let Some((_, param_id)) = &midi_learn {
            if let Some(mod_idx) = param_id_to_mod_dest_index(param_id) {
                let depth = get_lfo_max_depth(params, mod_idx);
                if depth > 0.001 {
                    draw_lfo_depth_indicator(ui, response.rect, value, min, max, depth);
                }
            }
        }

        let rail_rect = response.rect;
        let handle_radius = SLIDER_RAIL_THICKNESS * 0.55;
        let rail_top = rail_rect.top() + handle_radius;
        let rail_bottom = rail_rect.bottom() - handle_radius;
        let rail_height = rail_bottom - rail_top;
        let label_color = Color32::from_gray(55);
        let label_font = egui::FontId::proportional(10.0);
        let label_x = rail_rect.right() + 3.0;

        for &(val, text) in ticks {
            let t = if let Some(exp) = scale_exp {
                ((val - min) / (max - min)).powf(1.0 / exp)
            } else if scale_is_log && min > 0.0 && val > 0.0 {
                (val.ln() - min.ln()) / (max.ln() - min.ln())
            } else {
                (val - min) / (max - min)
            };
            let y = rail_bottom - t * rail_height;
            let galley = ui.painter().layout_no_wrap(text.to_string(), label_font.clone(), label_color);
            let text_height = galley.size().y;
            ui.painter().galley(egui::pos2(label_x, y - text_height / 2.0), galley, label_color);
        }

        ui.add_space(2.0);
        let has_cc = midi_learn.as_ref().and_then(|(ml, param_id)| {
            ml.mappings.try_lock().ok().and_then(|m| m.find_by_param(param_id))
        }).is_some();
        let selected = midi_learn.as_ref()
            .map(|(ml, param_id)| is_selected_param(ml, param_id))
            .unwrap_or(false);
        ui.horizontal(|ui| {
            let offset = match label.chars().count() {
                1 => 5.0,
                2 => -1.0,
                3 => -7.0,
                _ => -12.0,
            };
            ui.add_space(offset);
            let label_text = if selected {
                egui::RichText::new(label).size(LABEL_FONT).color(SELECTED_LABEL_COLOR).strong()
            } else if has_cc {
                egui::RichText::new(label).size(LABEL_FONT).color(MIDI_ASSIGNED_COLOR)
            } else {
                egui::RichText::new(label).size(LABEL_FONT)
            };
            ui.label(label_text);
        });
    });
}

// Division indices sorted by beat duration (ascending: fastest → slowest)
const SYNC_DIV_BY_BEATS: [usize; 18] = [
    17, 16, 5, 13, 4, 12, 9, 3, 11, 8, 2, 10, 7, 1, 6, 0, 14, 15,
];

fn render_vertical_sync_div_slider(
    ui: &mut egui::Ui,
    setter: &ParamSetter,
    div_param: &nih_plug::prelude::IntParam,
    label: &str,
    color: Option<Color32>,
    _tempo: f32,
    max_beats: f64,
) {
    use crate::synth::lfo::LfoSyncDivision;

    let allowed: Vec<usize> = SYNC_DIV_BY_BEATS.iter().copied()
        .filter(|&idx| LfoSyncDivision::from_index(idx as i32).beats() <= max_beats + 0.001)
        .collect();
    let max_pos = (allowed.len() as f32 - 1.0).max(0.0);

    ui.vertical(|ui| {
        ui.set_width(SLIDER_COL_WIDTH);

        if let Some(fill_color) = color {
            ui.style_mut().visuals.widgets.inactive.bg_fill = fill_color;
            ui.style_mut().visuals.widgets.hovered.bg_fill = fill_color;
            ui.style_mut().visuals.widgets.active.bg_fill = fill_color;
        }

        ui.style_mut().spacing.slider_width = SLIDER_RAIL_LENGTH;
        ui.style_mut().spacing.slider_rail_height = SLIDER_RAIL_THICKNESS;

        let current_div = div_param.value() as usize;
        let mut current_pos = 0.0f32;
        for (i, &div_idx) in allowed.iter().enumerate() {
            if div_idx == current_div {
                current_pos = i as f32;
                break;
            }
        }

        let mut slider_value = current_pos;
        let slider = egui::Slider::new(&mut slider_value, 0.0..=max_pos)
            .vertical()
            .show_value(false)
            .step_by(1.0);
        let r = ui.add(slider);
        if r.changed() {
            let pos = slider_value.round().clamp(0.0, max_pos) as usize;
            let new_div_idx = allowed[pos];
            setter.set_parameter(div_param, new_div_idx as i32);
        }

        let rail_rect = r.rect;
        let handle_radius = SLIDER_RAIL_THICKNESS * 0.55;
        let rail_top = rail_rect.top() + handle_radius;
        let rail_bottom = rail_rect.bottom() - handle_radius;
        let rail_height = rail_bottom - rail_top;
        let label_color = Color32::from_gray(55);
        let label_font = egui::FontId::proportional(10.0);
        let label_x = rail_rect.right() + 3.0;

        let n = allowed.len();
        let step = if n <= 6 { 1 } else { (n - 1) / 5 };
        for i in (0..n).step_by(step.max(1)) {
            let div = LfoSyncDivision::from_index(allowed[i] as i32);
            let t = i as f32 / max_pos;
            let y = rail_bottom - t * rail_height;
            let galley = ui.painter().layout_no_wrap(div.label().to_string(), label_font.clone(), label_color);
            let text_height = galley.size().y;
            ui.painter().galley(egui::pos2(label_x, y - text_height / 2.0), galley, label_color);
        }
        if n > 1 {
            let last = n - 1;
            if last % step != 0 {
                let div = LfoSyncDivision::from_index(allowed[last] as i32);
                let y = rail_top;
                let galley = ui.painter().layout_no_wrap(div.label().to_string(), label_font.clone(), label_color);
                let text_height = galley.size().y;
                ui.painter().galley(egui::pos2(label_x, y - text_height / 2.0), galley, label_color);
            }
        }

        ui.add_space(2.0);
        ui.horizontal(|ui| {
            let offset = match label.chars().count() {
                1 => 5.0,
                2 => -1.0,
                3 => -7.0,
                _ => -12.0,
            };
            ui.add_space(offset);
            ui.label(egui::RichText::new(label).size(LABEL_FONT));
        });
    });
}

fn render_int_vertical_slider(
    ui: &mut egui::Ui,
    params: &Arc<DeviceParams>,
    setter: &ParamSetter,
    param: &nih_plug::prelude::IntParam,
    label: &str,
    color: Option<Color32>,
    tick_labels: Option<&[i32]>,
    value_display: Option<&[&str]>,
    midi_learn: Option<(&MidiLearnState, &str)>,
) {
    ui.vertical(|ui| {
        ui.set_width(SLIDER_COL_WIDTH);
        let mut value = param.value();

        let learn_active = midi_learn.as_ref()
            .map(|(ml, _)| ml.learn_active.load(Ordering::Relaxed))
            .unwrap_or(false);

        if let Some(fill_color) = color {
            if learn_active {
                let brighter = Color32::from_rgb(
                    fill_color.r().saturating_add(30),
                    fill_color.g().saturating_add(20),
                    fill_color.b().saturating_add(10),
                );
                ui.style_mut().visuals.widgets.inactive.bg_fill = brighter;
                ui.style_mut().visuals.widgets.hovered.bg_fill = brighter;
                ui.style_mut().visuals.widgets.active.bg_fill = brighter;
            } else {
                ui.style_mut().visuals.widgets.inactive.bg_fill = fill_color;
                ui.style_mut().visuals.widgets.hovered.bg_fill = fill_color;
                ui.style_mut().visuals.widgets.active.bg_fill = fill_color;
            }
        }

        ui.style_mut().spacing.slider_width = SLIDER_RAIL_LENGTH;
        ui.style_mut().spacing.slider_rail_height = SLIDER_RAIL_THICKNESS;

        let (min, max) = match param.range() {
            nih_plug::prelude::IntRange::Linear { min, max } => (min, max),
            nih_plug::prelude::IntRange::Reversed(inner) => match inner {
                nih_plug::prelude::IntRange::Linear { min, max } => (*min, *max),
                _ => (0, 0),
            },
        };

        let slider = egui::Slider::new(&mut value, min..=max)
            .vertical()
            .show_value(false);

        let response = ui.add(slider);
        if response.changed() {
            setter.set_parameter(param, value);
        }

        if let Some((ml, param_id)) = &midi_learn {
            if learn_active && (response.drag_started() || response.clicked()) {
                if let Ok(mut awaiting) = ml.awaiting_param.try_lock() {
                    *awaiting = Some(param_id.to_string());
                }
            }
        }

        draw_cc_pickup_indicator(ui, response.rect, &midi_learn);

        if let Some((_, param_id)) = &midi_learn {
            if let Some(mod_idx) = param_id_to_mod_dest_index(param_id) {
                let depth = get_lfo_max_depth(params, mod_idx);
                if depth > 0.001 {
                    let fmin = min as f32;
                    let fmax = max as f32;
                    let fval = value as f32;
                    draw_lfo_depth_indicator(ui, response.rect, fval, fmin, fmax, depth);
                }
            }
        }

        let steps = max - min;
        if steps > 0 {
            let rail_rect = response.rect;
            let handle_radius = SLIDER_RAIL_THICKNESS * 0.55;
            let rail_top = rail_rect.top() + handle_radius;
            let rail_bottom = rail_rect.bottom() - handle_radius;
            let rail_height = rail_bottom - rail_top;
            let label_color = Color32::from_gray(55);
            let label_font = egui::FontId::proportional(10.0);
            let label_x = rail_rect.right() + 3.0;

            let values_to_show: Vec<i32> = match tick_labels {
                Some(labels) => labels.to_vec(),
                None => (min..=max).collect(),
            };

            for val in values_to_show {
                if val < min || val > max { continue; }
                let t = (val - min) as f32 / steps as f32;
                let y = rail_bottom - t * rail_height;
                let text = if let Some(displays) = value_display {
                    let idx = (val - min) as usize;
                    if idx < displays.len() {
                        displays[idx].to_string()
                    } else {
                        format!("{}", val)
                    }
                } else {
                    format!("{}", val)
                };
                let galley = ui.painter().layout_no_wrap(text, label_font.clone(), label_color);
                let text_height = galley.size().y;
                ui.painter().galley(egui::pos2(label_x, y - text_height / 2.0), galley, label_color);
            }
        }

        ui.add_space(2.0);
        let has_cc = midi_learn.as_ref().and_then(|(ml, param_id)| {
            ml.mappings.try_lock().ok().and_then(|m| m.find_by_param(param_id))
        }).is_some();
        let selected = midi_learn.as_ref()
            .map(|(ml, param_id)| is_selected_param(ml, param_id))
            .unwrap_or(false);
        ui.horizontal(|ui| {
            let offset = match label.chars().count() {
                1 => 5.0,
                2 => -1.0,
                3 => -7.0,
                _ => -12.0,
            };
            ui.add_space(offset);
            let label_text = if selected {
                egui::RichText::new(label).size(LABEL_FONT).color(SELECTED_LABEL_COLOR).strong()
            } else if has_cc {
                egui::RichText::new(label).size(LABEL_FONT).color(MIDI_ASSIGNED_COLOR)
            } else {
                egui::RichText::new(label).size(LABEL_FONT)
            };
            ui.label(label_text);
        });
    });
}

fn param_id_to_mod_dest_index(param_id: &str) -> Option<i32> {
    match param_id {
        "synth_pll_damping" => Some(1),
        "synth_pll_influence" => Some(2),
        "synth_pll_track_speed" => Some(3),
        "synth_pll_fm_amount" => Some(4),
        "synth_pll_burst_amount" => Some(6),
        "synth_pll_range" => Some(7),
        "synth_osc_d" => Some(8),
        "synth_osc_v" => Some(9),
        "synth_drift_amount" => Some(13),
        "synth_tube_drive" => Some(14),
        "synth_pll_volume" => Some(17),
        "synth_osc_volume" => Some(18),
        "synth_sub_volume" => Some(19),
        "synth_pll_mult" => Some(20),
        "synth_vps_shape_amount" => Some(22),
        "synth_osc_stereo_d_offset" => Some(23),
        "synth_osc_fold" => Some(24),
        "synth_osc_stereo_v_offset" => Some(25),
        "synth_pll_injection_amount" => Some(26),
        "synth_pll_mult_slew_time" => Some(27),
        "synth_saw_fold" => Some(28),
        "synth_saw_shape_amount" => Some(29),
        "synth_saw_volume" => Some(30),
        "synth_env_range" => Some(39),
        "synth_pll_tail_amount" => Some(40),
        "synth_pll_tail_time" => Some(41),
        _ => None,
    }
}

fn get_lfo_max_depth(params: &DeviceParams, mod_dest_index: i32) -> f32 {
    let mut total_depth: f32 = 0.0;
    let destinations = [
        (params.lfo1_dest1.value(), params.lfo1_amount1.modulated_plain_value()),
        (params.lfo1_dest2.value(), params.lfo1_amount2.modulated_plain_value()),
        (params.lfo2_dest1.value(), params.lfo2_amount1.modulated_plain_value()),
        (params.lfo2_dest2.value(), params.lfo2_amount2.modulated_plain_value()),
        (params.lfo3_dest1.value(), params.lfo3_amount1.modulated_plain_value()),
        (params.lfo3_dest2.value(), params.lfo3_amount2.modulated_plain_value()),
    ];
    for (dest, amount) in &destinations {
        if *dest == mod_dest_index {
            total_depth += amount.abs();
        }
    }
    total_depth
}

fn draw_lfo_depth_indicator(
    ui: &egui::Ui,
    rail_rect: egui::Rect,
    value: f32,
    min: f32,
    max: f32,
    depth: f32,
) {
    let handle_radius = SLIDER_RAIL_THICKNESS * 0.55;
    let rail_top = rail_rect.top() + handle_radius;
    let rail_bottom = rail_rect.bottom() - handle_radius;
    let rail_height = rail_bottom - rail_top;
    let normalized = (value - min) / (max - min);
    let current_y = rail_bottom - normalized * rail_height;
    let depth_pixels = depth * rail_height;
    let mod_top = (current_y - depth_pixels).max(rail_top);
    let mod_bottom = (current_y + depth_pixels).min(rail_bottom);
    let x = rail_rect.left() - 3.0;
    ui.painter().line_segment(
        [egui::pos2(x, mod_top), egui::pos2(x, mod_bottom)],
        egui::Stroke::new(2.0, LFO_MOD_INDICATOR_COLOR),
    );
}

fn build_time_ticks(min: f32, max: f32) -> Vec<(f32, String)> {
    let candidates: &[(f32, &str)] = &[
        (0.5, "0.5"), (1.0, "1"), (2.0, "2"), (5.0, "5"),
        (10.0, "10"), (20.0, "20"), (50.0, "50"),
        (100.0, "100"), (200.0, "200"), (500.0, "500"),
        (1000.0, "1s"), (2000.0, "2s"), (5000.0, "5s"), (10000.0, "10s"),
    ];
    let mut ticks = Vec::new();
    for &(val, label) in candidates {
        if val >= min && val <= max {
            ticks.push((val, label.to_string()));
        }
    }
    if ticks.len() > 4 {
        let len = ticks.len();
        let indices = [0, len / 3, 2 * len / 3, len - 1];
        ticks = indices.iter().filter_map(|&i| ticks.get(i).cloned()).collect();
    }
    ticks
}

fn render_envelope_controls_compact(
    ui: &mut egui::Ui,
    params: &Arc<DeviceParams>,
    setter: &ParamSetter,
    ui_state: &SharedUiState,
) {
    let range_ms = params.synth_env_range.modulated_plain_value().max(20.0);
    let time_max_a = range_ms.min(5000.0).max(1.0);
    let time_max_dr = range_ms.min(10000.0).max(1.0);
    let shape_ticks: &[(f32, &str)] = &[(-1.0, "EXP"), (0.0, "LIN"), (1.0, "LOG")];
    let s_ticks: &[(f32, &str)] = &[(0.0, "LIN"), (0.5, ""), (1.0, "MAX")];
    let atk_color = Some(Color32::from_rgb(140, 100, 60));
    let dec_color = Some(Color32::from_rgb(100, 80, 120));
    let sus_color = Some(Color32::from_rgb(60, 100, 80));
    let rel_color = Some(Color32::from_rgb(80, 80, 140));
    let dip_color = Some(Color32::from_rgb(140, 80, 80));

    let time_ticks_a = build_time_ticks(0.5, time_max_a);
    let time_ticks_a_ref: Vec<(f32, &str)> = time_ticks_a.iter().map(|(v, s)| (*v, s.as_str())).collect();
    let time_ticks_dr = build_time_ticks(0.5, time_max_dr);
    let time_ticks_dr_ref: Vec<(f32, &str)> = time_ticks_dr.iter().map(|(v, s)| (*v, s.as_str())).collect();

    let hold_color = Some(Color32::from_rgb(120, 120, 60));
    let hold_ticks = build_time_ticks(0.0, time_max_a);
    let hold_ticks_ref: Vec<(f32, &str)> = hold_ticks.iter().map(|(v, s)| (*v, s.as_str())).collect();

    let tempo = ui_state.current_tempo.load(std::sync::atomic::Ordering::Relaxed) as f32 / 100.0;

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 6.0;
        if params.synth_vol_attack_sync.value() {
            render_vertical_sync_div_slider(ui, setter, &params.synth_vol_attack_div, "A", atk_color, tempo, 1.0);
        } else {
            render_vertical_slider_with_ticks(
                ui, params, setter,
                &params.synth_vol_attack, "A",
                0.5, time_max_a, SliderScale::Exponential(2.0),
                atk_color, &time_ticks_a_ref, None,
            );
        }
        ui.add_space(2.0);
        let (sh_min, sh_ticks) = if params.synth_vol_attack_s.value() { (0.0, s_ticks) } else { (-1.0, shape_ticks) };
        render_vertical_slider_with_ticks(
            ui, params, setter,
            &params.synth_vol_attack_shape, "A\u{2009}SH",
            sh_min, 1.0, SliderScale::Linear,
            atk_color, sh_ticks, None,
        );
        ui.add_space(4.0);
        if params.synth_vol_hold_sync.value() {
            render_vertical_sync_div_slider(ui, setter, &params.synth_vol_hold_div, "H", hold_color, tempo, 1.0);
        } else {
            render_vertical_slider_with_ticks(
                ui, params, setter,
                &params.synth_vol_hold, "H",
                0.0, time_max_a, SliderScale::Exponential(2.0),
                hold_color, &hold_ticks_ref, None,
            );
        }
        ui.add_space(4.0);
        if params.synth_vol_decay_sync.value() {
            render_vertical_sync_div_slider(ui, setter, &params.synth_vol_decay_div, "D", dec_color, tempo, 1.0);
        } else {
            render_vertical_slider_with_ticks(
                ui, params, setter,
                &params.synth_vol_decay, "D",
                0.5, time_max_dr, SliderScale::Exponential(2.0),
                dec_color, &time_ticks_dr_ref, None,
            );
        }
        ui.add_space(2.0);
        let (sh_min, sh_ticks) = if params.synth_vol_decay_s.value() { (0.0, s_ticks) } else { (-1.0, shape_ticks) };
        render_vertical_slider_with_ticks(
            ui, params, setter,
            &params.synth_vol_decay_shape, "D\u{2009}SH",
            sh_min, 1.0, SliderScale::Linear,
            dec_color, sh_ticks, None,
        );
        ui.add_space(4.0);
        render_vertical_slider(
            ui, params, setter,
            &params.synth_vol_sustain, "S",
            0.0, 1.0, SliderScale::Linear,
            sus_color, None,
        );
        ui.add_space(4.0);
        if params.synth_vol_release_sync.value() {
            render_vertical_sync_div_slider(ui, setter, &params.synth_vol_release_div, "R", rel_color, tempo, 4.0);
        } else {
            render_vertical_slider_with_ticks(
                ui, params, setter,
                &params.synth_vol_release, "R",
                0.5, time_max_dr, SliderScale::Exponential(2.0),
                rel_color, &time_ticks_dr_ref, None,
            );
        }
        ui.add_space(2.0);
        let (sh_min, sh_ticks) = if params.synth_vol_release_s.value() { (0.0, s_ticks) } else { (-1.0, shape_ticks) };
        render_vertical_slider_with_ticks(
            ui, params, setter,
            &params.synth_vol_release_shape, "R\u{2009}SH",
            sh_min, 1.0, SliderScale::Linear,
            rel_color, sh_ticks, None,
        );
        ui.add_space(6.0);
        render_vertical_slider(
            ui, params, setter,
            &params.synth_retrigger_dip, "DIP",
            0.0, 1.0, SliderScale::Linear,
            dip_color, None,
        );
    });

    ui.add_space(36.0);
    let row_left = ui.cursor().left();
    let row_top = ui.cursor().top();

    let range_ticks = build_time_ticks(20.0, 10000.0);
    let range_ticks_ref: Vec<(f32, &str)> = range_ticks.iter().map(|(v, s)| (*v, s.as_str())).collect();
    let pll_time_max = range_ms.min(5000.0).max(50.0);
    let pll_time_ticks = build_time_ticks(50.0, pll_time_max);
    let pll_time_ticks_ref: Vec<(f32, &str)> = pll_time_ticks.iter().map(|(v, s)| (*v, s.as_str())).collect();

    // RNG slider on the left
    let rng_x = row_left + 2.0;
    let mut rng_ui = ui.new_child(egui::UiBuilder::new().max_rect(
        egui::Rect::from_min_size(egui::pos2(rng_x, row_top + 25.0), egui::vec2(80.0, 300.0)),
    ));
    rng_ui.horizontal(|ui| {
        let range_color = Some(Color32::from_rgb(100, 100, 80));
        render_vertical_slider_with_ticks(
            ui, params, setter,
            &params.synth_env_range, "RNG",
            20.0, 10000.0, SliderScale::Exponential(2.0),
            range_color, &range_ticks_ref, None,
        );
    });

    // ADSR chart in the middle
    let viz_x = row_left + 65.0;
    let viz_y = row_top + 35.0;
    render_adsr_visualization(ui, params, viz_x, viz_y, ui_state);

    // PLL TAIL label + AMT + TIME sliders on the right
    let slider_x = row_left + 400.0;
    let mut tail_ui = ui.new_child(egui::UiBuilder::new().max_rect(
        egui::Rect::from_min_size(egui::pos2(slider_x, row_top - 10.0), egui::vec2(200.0, 300.0)),
    ));
    tail_ui.vertical(|ui| {
        ui.label(egui::RichText::new("PLL TAIL").size(HEADER_FONT).strong());
        ui.add_space(6.0);
    });
    let mut tail_ui2 = ui.new_child(egui::UiBuilder::new().max_rect(
        egui::Rect::from_min_size(egui::pos2(slider_x, row_top + 24.0), egui::vec2(200.0, 300.0)),
    ));
    tail_ui2.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 5.0;
        render_vertical_slider(
            ui, params, setter,
            &params.synth_pll_tail_amount, "AMT",
            0.0, 1.0, SliderScale::Linear,
            Some(Color32::from_rgb(100, 80, 60)),
            None,
        );
        ui.add_space(2.0);
        render_vertical_slider_with_ticks(
            ui, params, setter,
            &params.synth_pll_tail_time, "TIME",
            50.0, pll_time_max, SliderScale::Exponential(2.0),
            Some(Color32::from_rgb(80, 100, 120)),
            &pll_time_ticks_ref, None,
        );
    });
}

fn render_adsr_visualization(
    ui: &mut egui::Ui,
    params: &Arc<DeviceParams>,
    viz_x: f32,
    viz_y: f32,
    ui_state: &SharedUiState,
) {
    let viz_w: f32 = 295.0;
    let viz_h: f32 = 200.0;
    let pad: f32 = 10.0;

    let rect = egui::Rect::from_min_size(egui::pos2(viz_x, viz_y), egui::vec2(viz_w, viz_h));

    let bg_color = Color32::from_rgb(25, 25, 30);
    let border_color = Color32::from_rgb(50, 50, 60);
    let curve_color = Color32::from_rgb(120, 160, 220);
    let tail_color = Color32::from_rgb(160, 100, 60);
    let sustain_color = Color32::from_rgb(60, 100, 80);
    let grid_color = Color32::from_rgb(35, 35, 42);

    ui.painter().rect_filled(rect, 4.0, bg_color);
    ui.painter().rect_stroke(rect, 4.0, egui::Stroke::new(1.0, border_color), egui::epaint::StrokeKind::Inside);

    let inner = rect.shrink(pad);
    let w = inner.width();
    let h = inner.height();

    for i in 1..4 {
        let y = inner.top() + h * i as f32 / 4.0;
        ui.painter().line_segment(
            [egui::pos2(inner.left(), y), egui::pos2(inner.right(), y)],
            egui::Stroke::new(0.5, grid_color),
        );
    }

    let tempo = ui_state.current_tempo.load(std::sync::atomic::Ordering::Relaxed) as f32 / 100.0;
    let div_to_ms = |div_idx: i32| -> f32 {
        let div = crate::synth::lfo::LfoSyncDivision::from_index(div_idx);
        (div.beats() as f64 / tempo.max(1.0) as f64 * 60000.0).max(0.5) as f32
    };

    let range_ms = params.synth_env_range.modulated_plain_value();
    let max_a = range_ms.min(5000.0).max(1.0);
    let max_dr = range_ms.min(10000.0).max(1.0);

    let attack_ms = if params.synth_vol_attack_sync.value() {
        div_to_ms(params.synth_vol_attack_div.value())
    } else {
        params.synth_vol_attack.modulated_plain_value().clamp(0.5, max_a)
    };
    let hold_ms = if params.synth_vol_hold_sync.value() {
        div_to_ms(params.synth_vol_hold_div.value())
    } else {
        params.synth_vol_hold.modulated_plain_value().clamp(0.0, 5000.0)
    };
    let decay_ms = if params.synth_vol_decay_sync.value() {
        div_to_ms(params.synth_vol_decay_div.value())
    } else {
        params.synth_vol_decay.modulated_plain_value().clamp(0.5, max_dr)
    };
    let release_ms = if params.synth_vol_release_sync.value() {
        div_to_ms(params.synth_vol_release_div.value())
    } else {
        params.synth_vol_release.modulated_plain_value().clamp(0.5, max_dr)
    };
    let sustain = params.synth_vol_sustain.modulated_plain_value().clamp(0.0, 1.0);
    let attack_shape = params.synth_vol_attack_shape.modulated_plain_value().clamp(-1.0, 1.0);
    let decay_shape = params.synth_vol_decay_shape.modulated_plain_value().clamp(-1.0, 1.0);
    let release_shape = params.synth_vol_release_shape.modulated_plain_value().clamp(-1.0, 1.0);
    let attack_s = params.synth_vol_attack_s.value();
    let decay_s = params.synth_vol_decay_s.value();
    let release_s = params.synth_vol_release_s.value();
    let depth = params.synth_vol_depth.modulated_plain_value().clamp(0.0, 1.0);

    let dip = params.synth_retrigger_dip.modulated_plain_value().clamp(0.0, 1.0);
    let dip_ms: f32 = 2.0;
    let has_dip = dip > 0.001;

    let tail_time_ms = params.synth_pll_tail_time.modulated_plain_value().clamp(50.0, range_ms.min(5000.0).max(50.0));
    let tail_amount = params.synth_pll_tail_amount.modulated_plain_value().clamp(0.0, 1.0);
    let pll_tail_on = tail_amount > 0.001;

    let adsr_total = attack_ms + hold_ms + decay_ms + release_ms;
    let sustain_ms = adsr_total * 0.2;
    let tail_ms = if pll_tail_on { tail_time_ms } else { 0.0 };
    let effective_dip_ms = if has_dip { dip_ms } else { 0.0 };
    let total_ms = effective_dip_ms + adsr_total + sustain_ms + tail_ms;
    let time_scale = if total_ms > 0.001 { 1.0 / total_ms } else { 1.0 };

    let dip_w = effective_dip_ms * time_scale * w;
    let a_w = attack_ms * time_scale * w;
    let h_w = hold_ms * time_scale * w;
    let d_w = decay_ms * time_scale * w;
    let s_w = sustain_ms * time_scale * w;
    let r_w = release_ms * time_scale * w;
    let t_w = tail_ms * time_scale * w;

    let x0 = inner.left();
    let y_bot = inner.bottom();

    {
        let div_line_color = Color32::from_rgb(45, 45, 55);
        let div_label_color = Color32::from_gray(50);
        let div_font = egui::FontId::proportional(9.0);
        let divs: &[(f64, &str)] = &[
            (0.03125, "1/128"), (0.0625, "1/64"), (0.125, "1/32"),
            (0.25, "1/16"), (0.5, "1/8"), (1.0, "1/4"),
            (2.0, "1/2"), (4.0, "1/1"), (8.0, "2/1"), (16.0, "4/1"),
        ];
        let mut last_x = -100.0_f32;
        for &(beats, label) in divs {
            let ms = (beats / tempo.max(1.0) as f64 * 60000.0) as f32;
            if ms < 0.5 || ms > total_ms { continue; }
            let x = inner.left() + ms / total_ms * w;
            if (x - last_x) < 20.0 { continue; }
            last_x = x;
            ui.painter().line_segment(
                [egui::pos2(x, inner.top()), egui::pos2(x, inner.bottom())],
                egui::Stroke::new(0.5, div_line_color),
            );
            ui.painter().text(
                egui::pos2(x, inner.top() + 1.0),
                egui::Align2::CENTER_TOP,
                label,
                div_font.clone(),
                div_label_color,
            );
        }
    }

    let shaped_curve = |t: f32, shape: f32, s_curve: bool| -> f32 {
        let sd = if s_curve { (shape as f64).max(0.0) * 2.0 } else { shape as f64 };
        let td = t as f64;
        if sd.abs() < 0.01 { return t; }
        let k = 1.0 + sd.abs() * 9.0;
        let ln_k = if sd > 0.0 { k.ln() } else { -k.ln() };
        if s_curve {
            let ka = ln_k.abs();
            let denom = ka.exp_m1();
            (if ln_k > 0.0 {
                if td <= 0.5 { 0.5 * (td * 2.0 * ka).exp_m1() / denom }
                else { 1.0 - 0.5 * ((1.0 - td) * 2.0 * ka).exp_m1() / denom }
            } else {
                if td <= 0.5 { 0.5 * (1.0 - ((1.0 - td * 2.0) * ka).exp_m1() / denom) }
                else { 1.0 - 0.5 * (1.0 - ((1.0 - (1.0 - td) * 2.0) * ka).exp_m1() / denom) }
            }) as f32
        } else {
            (if ln_k > 0.0 {
                (td * ln_k).exp_m1() / ln_k.exp_m1()
            } else {
                let pk = -ln_k;
                1.0 - ((1.0 - td) * pk).exp_m1() / pk.exp_m1()
            }) as f32
        }
    };

    let segments = 16;
    let mut points = Vec::with_capacity(segments * 4 + 10);
    let dip_color = Color32::from_rgb(140, 80, 80);
    let hold_color = Color32::from_rgb(120, 120, 60);

    if has_dip {
        let dip_start_v = sustain;
        let dip_target_v = sustain * (1.0 - dip);
        points.push(egui::pos2(x0, y_bot - dip_start_v * h));
        points.push(egui::pos2(x0 + dip_w, y_bot - dip_target_v * h));

        for i in 0..points.len().saturating_sub(1) {
            ui.painter().line_segment(
                [points[i], points[i + 1]],
                egui::Stroke::new(1.5, dip_color),
            );
        }

        let dip_x_end = x0 + dip_w;
        points.clear();
        points.push(egui::pos2(dip_x_end, y_bot - dip_target_v * h));

        for i in 0..=segments {
            let t = i as f32 / segments as f32;
            let v = dip_target_v + shaped_curve(t, attack_shape, attack_s) * (1.0 - dip_target_v);
            points.push(egui::pos2(dip_x_end + t * a_w, y_bot - v * h));
        }
    } else {
        points.push(egui::pos2(x0, y_bot));

        for i in 0..=segments {
            let t = i as f32 / segments as f32;
            let v = shaped_curve(t, attack_shape, attack_s);
            points.push(egui::pos2(x0 + t * a_w, y_bot - v * h));
        }
    }

    let x_h_start = x0 + dip_w + a_w;
    if h_w > 0.5 {
        let hold_end = egui::pos2(x_h_start + h_w, y_bot - h);
        points.push(hold_end);

        let hold_start_idx = points.len() - 2;
        ui.painter().line_segment(
            [points[hold_start_idx], hold_end],
            egui::Stroke::new(1.5, hold_color),
        );
    }

    let x_d_start = x_h_start + h_w;
    for i in 1..=segments {
        let t = i as f32 / segments as f32;
        let v = 1.0 - shaped_curve(t, decay_shape, decay_s) * (1.0 - sustain);
        points.push(egui::pos2(x_d_start + t * d_w, y_bot - v * h));
    }

    let x_s_end = x_d_start + d_w + s_w;
    points.push(egui::pos2(x_s_end, y_bot - sustain * h));

    let x_r_start = x_s_end;
    for i in 1..=segments {
        let t = i as f32 / segments as f32;
        let v = sustain * (1.0 - shaped_curve(t, release_shape, release_s));
        points.push(egui::pos2(x_r_start + t * r_w, y_bot - v * h));
    }

    for i in 0..points.len().saturating_sub(1) {
        ui.painter().line_segment(
            [points[i], points[i + 1]],
            egui::Stroke::new(1.5, curve_color),
        );
    }

    if pll_tail_on && tail_amount > 0.001 {
        let x_tail_start = x_r_start + r_w;
        let tail_segments = 20;
        let mut tail_points = Vec::with_capacity(tail_segments + 2);

        tail_points.push(egui::pos2(x_tail_start, y_bot));
        tail_points.push(egui::pos2(x_tail_start, y_bot - tail_amount * h));

        for i in 1..=tail_segments {
            let t = i as f32 / tail_segments as f32;
            let v = tail_amount * 0.001_f32.powf(t);
            tail_points.push(egui::pos2(x_tail_start + t * t_w, y_bot - v * h));
        }

        for i in 0..tail_points.len().saturating_sub(1) {
            ui.painter().line_segment(
                [tail_points[i], tail_points[i + 1]],
                egui::Stroke::new(1.5, tail_color),
            );
        }
    }

    let sus_y = y_bot - sustain * h;
    ui.painter().line_segment(
        [egui::pos2(inner.left(), sus_y), egui::pos2(x_s_end, sus_y)],
        egui::Stroke::new(0.5, sustain_color),
    );

    if depth < 0.999 {
        let depth_color = Color32::from_rgb(80, 60, 100);
        let floor_y = y_bot - (1.0 - depth) * h;
        ui.painter().line_segment(
            [egui::pos2(inner.left(), floor_y), egui::pos2(inner.right(), floor_y)],
            egui::Stroke::new(0.5, depth_color),
        );
    }

    let total_adsr_ms = attack_ms + hold_ms + decay_ms + release_ms;
    let duration_str = if total_adsr_ms >= 1000.0 {
        format!("{:.1}s", total_adsr_ms / 1000.0)
    } else {
        format!("{:.0}ms", total_adsr_ms)
    };
    let caption = if pll_tail_on {
        format!("{} + TAIL", duration_str)
    } else {
        duration_str
    };
    ui.painter().text(
        egui::pos2(rect.center().x, rect.bottom() + 4.0),
        egui::Align2::CENTER_TOP,
        caption,
        egui::FontId::proportional(14.0),
        Color32::from_gray(70),
    );
}

fn render_filter_visualization(
    ui: &mut egui::Ui,
    params: &Arc<DeviceParams>,
    viz_x: f32,
    viz_y: f32,
) {
    let viz_w: f32 = 553.0;
    let viz_h: f32 = 200.0;
    let pad: f32 = 10.0;

    let rect = egui::Rect::from_min_size(egui::pos2(viz_x, viz_y), egui::vec2(viz_w, viz_h));

    let bg_color = Color32::from_rgb(25, 25, 30);
    let border_color = Color32::from_rgb(50, 50, 60);
    let curve_color = Color32::from_rgb(120, 160, 220);
    let hpf_color = Color32::from_rgb(80, 140, 180);
    let notch_color = Color32::from_rgb(140, 100, 60);
    let brill_color = Color32::from_rgb(160, 130, 60);
    let grid_color = Color32::from_rgb(35, 35, 42);

    ui.painter().rect_filled(rect, 4.0, bg_color);
    ui.painter().rect_stroke(rect, 4.0, egui::Stroke::new(1.0, border_color), egui::epaint::StrokeKind::Inside);

    let inner = rect.shrink(pad);
    let w = inner.width();
    let h = inner.height();

    // Grid lines (horizontal = dB, vertical = freq)
    for i in 1..4 {
        let y = inner.top() + h * i as f32 / 4.0;
        ui.painter().line_segment(
            [egui::pos2(inner.left(), y), egui::pos2(inner.right(), y)],
            egui::Stroke::new(0.5, grid_color),
        );
    }
    // Vertical grid at key frequencies
    let freq_marks = [100.0_f32, 200.0, 1000.0, 5000.0, 10000.0];
    let log_min = 20.0_f32.ln();
    let log_max = 20000.0_f32.ln();
    for freq in &freq_marks {
        let t = (freq.ln() - log_min) / (log_max - log_min);
        let x = inner.left() + t * w;
        ui.painter().line_segment(
            [egui::pos2(x, inner.top()), egui::pos2(x, inner.bottom())],
            egui::Stroke::new(0.5, grid_color),
        );
    }

    // Read parameters
    let hpf_mode = params.master_hpf.value();
    let hpf_boost = params.master_hpf_boost.value();
    let box_cut_mode = params.box_cut_mode.value();
    let brilliance_amount = params.brilliance_amount.modulated_plain_value();
    let mono_bass_hz = params.stereo_mono_bass.modulated_plain_value();

    let hpf_freq = match hpf_mode {
        1 => 35.0_f32,
        2 => 80.0,
        3 => 120.0,
        4 => 220.0,
        _ => 0.0,
    };
    let hpf_q = match hpf_boost {
        1 => 2.0_f32,
        2 => 4.0,
        _ => 0.707,
    };
    let box_cut_amount = match box_cut_mode {
        1 => 0.2921_f32,  // -3 dB
        2 => 0.4988,      // -6 dB
        3 => 0.7488,      // -12 dB
        _ => 0.0,
    };
    let box_cut_freq = 400.0_f32;
    let box_cut_q = 1.5_f32;
    let brill_freq = 4500.0_f32;

    // 0dB reference line (center)
    let zero_db_y = inner.top() + h * 0.5;
    let db_scale = h * 0.5 / 18.0; // ±18dB range

    // Compute HPF response at a frequency
    let hpf_response_db = |freq: f32| -> f32 {
        if hpf_freq < 1.0 { return 0.0; }
        let ratio = freq / hpf_freq;
        let r2 = ratio * ratio;
        let denom = ((1.0 - r2) * (1.0 - r2) + r2 / (hpf_q * hpf_q)).sqrt();
        let mag = r2 / denom;
        20.0 * mag.max(0.0001).log10()
    };

    // Compute box cut (notch) response
    let notch_response_db = |freq: f32| -> f32 {
        if box_cut_amount < 0.001 { return 0.0; }
        let ratio = freq / box_cut_freq;
        let r2 = ratio * ratio;
        let bp_mag2 = (ratio / box_cut_q).powi(2)
            / ((1.0 - r2).powi(2) + (ratio / box_cut_q).powi(2));
        let atten = 1.0 - box_cut_amount * bp_mag2.sqrt();
        20.0 * atten.max(0.0001).log10()
    };

    // Compute brilliance response (high-shelf boost)
    let brill_response_db = |freq: f32| -> f32 {
        if brilliance_amount < 0.001 { return 0.0; }
        let ratio = freq / brill_freq;
        let r2 = ratio * ratio;
        let hp_mag2 = r2 * r2 / ((1.0 - r2).powi(2) + r2 * 4.0); // Q=0.5
        let boost = brilliance_amount * hp_mag2.sqrt();
        20.0 * (1.0 + boost).log10()
    };

    let segments = 128;
    let mut combined_points = Vec::with_capacity(segments + 1);
    let mut hpf_points = Vec::with_capacity(segments + 1);
    let mut notch_points = Vec::with_capacity(segments + 1);
    let mut brill_points = Vec::with_capacity(segments + 1);

    for i in 0..=segments {
        let t = i as f32 / segments as f32;
        let freq = (log_min + t * (log_max - log_min)).exp();
        let x = inner.left() + t * w;

        let hpf_db = hpf_response_db(freq);
        let notch_db = notch_response_db(freq);
        let brill_db = brill_response_db(freq);
        let total_db = hpf_db + notch_db + brill_db;

        let clamp_y = |y: f32| y.clamp(inner.top(), inner.bottom());
        combined_points.push(egui::pos2(x, clamp_y(zero_db_y - total_db * db_scale)));
        if hpf_freq > 0.0 {
            hpf_points.push(egui::pos2(x, clamp_y(zero_db_y - hpf_db * db_scale)));
        }
        if box_cut_amount > 0.001 {
            notch_points.push(egui::pos2(x, clamp_y(zero_db_y - notch_db * db_scale)));
        }
        if brilliance_amount > 0.001 {
            brill_points.push(egui::pos2(x, clamp_y(zero_db_y - brill_db * db_scale)));
        }
    }

    // Draw individual filter curves (subtle)
    let subtle_stroke = 0.8;
    for pts in [(&hpf_points, hpf_color), (&notch_points, notch_color), (&brill_points, brill_color)] {
        for i in 0..pts.0.len().saturating_sub(1) {
            ui.painter().line_segment(
                [pts.0[i], pts.0[i + 1]],
                egui::Stroke::new(subtle_stroke, Color32::from_rgba_premultiplied(
                    pts.1.r(), pts.1.g(), pts.1.b(), 80,
                )),
            );
        }
    }

    // Draw combined response (bright)
    for i in 0..combined_points.len().saturating_sub(1) {
        ui.painter().line_segment(
            [combined_points[i], combined_points[i + 1]],
            egui::Stroke::new(1.5, curve_color),
        );
    }

    // 0dB reference line
    ui.painter().line_segment(
        [egui::pos2(inner.left(), zero_db_y), egui::pos2(inner.right(), zero_db_y)],
        egui::Stroke::new(0.5, Color32::from_rgb(60, 60, 70)),
    );

    // Mono bass crossover frequency marker
    if mono_bass_hz >= 20.0 {
        let mono_t = (mono_bass_hz.ln() - log_min) / (log_max - log_min);
        let mono_x = inner.left() + mono_t * w;
        let mono_color = Color32::from_rgb(50, 130, 110);
        ui.painter().line_segment(
            [egui::pos2(mono_x, inner.top()), egui::pos2(mono_x, inner.bottom())],
            egui::Stroke::new(1.0, Color32::from_rgba_premultiplied(
                mono_color.r(), mono_color.g(), mono_color.b(), 120,
            )),
        );
        ui.painter().text(
            egui::pos2(mono_x - 4.0, inner.top() + 2.0),
            egui::Align2::RIGHT_TOP,
            "MON",
            egui::FontId::proportional(9.0),
            mono_color,
        );
        ui.painter().text(
            egui::pos2(mono_x + 4.0, inner.top() + 2.0),
            egui::Align2::LEFT_TOP,
            format!("{}Hz", mono_bass_hz as i32),
            egui::FontId::proportional(9.0),
            mono_color,
        );
    }

    // Freq labels along bottom
    let freq_labels = [(100.0, "100"), (1000.0, "1k"), (10000.0, "10k")];
    for (freq, label) in &freq_labels {
        let t = ((*freq as f32).ln() - log_min) / (log_max - log_min);
        ui.painter().text(
            egui::pos2(inner.left() + t * w, inner.bottom() - 2.0),
            egui::Align2::CENTER_BOTTOM,
            *label,
            egui::FontId::proportional(9.0),
            Color32::from_gray(45),
        );
    }

    // Caption
    let mut parts = Vec::new();
    if hpf_freq > 0.0 {
        parts.push(format!("HPF {}Hz", hpf_freq as i32));
    }
    if box_cut_amount > 0.001 {
        parts.push("BOX CUT".to_string());
    }
    if brilliance_amount > 0.001 {
        parts.push("BRILL".to_string());
    }
    if mono_bass_hz >= 20.0 {
        parts.push(format!("MON {}Hz", mono_bass_hz as i32));
    }
    let caption = if parts.is_empty() {
        "FILTERS OFF".to_string()
    } else {
        parts.join(" + ")
    };
    ui.painter().text(
        egui::pos2(rect.center().x, rect.bottom() + 4.0),
        egui::Align2::CENTER_TOP,
        caption,
        egui::FontId::proportional(14.0),
        Color32::from_gray(70),
    );
}

fn render_toggle(
    ui: &mut egui::Ui,
    value: &mut bool,
    label: &str,
    midi_learn: Option<(&MidiLearnState, &str)>,
) {
    let desired_size = egui::vec2(48.0, 24.0);
    let (alloc_rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::click());

    if response.clicked() {
        if let Some((ml, param_id)) = midi_learn {
            if ml.learn_active.load(Ordering::Relaxed) {
                if let Ok(mut awaiting) = ml.awaiting_param.try_lock() {
                    *awaiting = Some(param_id.to_string());
                }
            }
        }
        *value = !*value;
    }

    let rect = alloc_rect.translate(egui::vec2(0.0, -5.0));

    let anim_t = ui.ctx().animate_bool_with_time(response.id, *value, 0.15);

    let bg_color = egui::Color32::from_gray(50).lerp_to_gamma(egui::Color32::from_rgb(80, 130, 190), anim_t);
    let circle_x = egui::lerp(rect.left() + 12.0..=rect.right() - 12.0, anim_t);
    let circle_color = egui::Color32::from_gray(220).lerp_to_gamma(egui::Color32::WHITE, anim_t);

    ui.painter().rect_filled(rect, rect.height() / 2.0, bg_color);
    ui.painter().circle_filled(egui::pos2(circle_x, rect.center().y), 9.0, circle_color);

    let is_selected = midi_learn.map_or(false, |(ml, param_id)| {
        let idx = ml.selected_param_idx.load(Ordering::Relaxed) as usize;
        SOUND_PARAMS.get(idx).copied() == Some(param_id)
    });
    let text_color = if is_selected {
        SELECTED_LABEL_COLOR
    } else if *value {
        egui::Color32::WHITE
    } else {
        egui::Color32::from_gray(140)
    };
    ui.painter().text(
        egui::pos2(rect.right() + 8.0, rect.center().y),
        egui::Align2::LEFT_CENTER,
        label,
        egui::FontId::proportional(HEADER_FONT),
        text_color,
    );
}

fn render_labeled_toggle(ui: &mut egui::Ui, value: &mut bool, left_label: &str, right_label: &str) {
    let font = egui::FontId::proportional(HEADER_FONT);
    let left_color = if !*value { egui::Color32::WHITE } else { egui::Color32::from_gray(100) };
    let right_color = if *value { egui::Color32::WHITE } else { egui::Color32::from_gray(100) };

    ui.painter().text(
        egui::pos2(ui.cursor().left(), ui.cursor().top() + 7.0),
        egui::Align2::LEFT_CENTER,
        left_label,
        font.clone(),
        left_color,
    );
    let left_text_width = ui.fonts(|f| f.layout_no_wrap(left_label.to_string(), font.clone(), left_color).size().x);
    ui.add_space(left_text_width + 6.0);

    let desired_size = egui::vec2(48.0, 24.0);
    let (alloc_rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::click());
    if response.clicked() {
        *value = !*value;
    }

    let rect = alloc_rect.translate(egui::vec2(0.0, -5.0));
    let anim_t = ui.ctx().animate_bool_with_time(response.id, *value, 0.15);

    let bg_color = egui::Color32::from_gray(50).lerp_to_gamma(egui::Color32::from_rgb(80, 130, 190), anim_t);
    let circle_x = egui::lerp(rect.left() + 12.0..=rect.right() - 12.0, anim_t);
    let circle_color = egui::Color32::from_gray(220).lerp_to_gamma(egui::Color32::WHITE, anim_t);

    ui.painter().rect_filled(rect, rect.height() / 2.0, bg_color);
    ui.painter().circle_filled(egui::pos2(circle_x, rect.center().y), 9.0, circle_color);

    ui.painter().text(
        egui::pos2(rect.right() + 8.0, rect.center().y),
        egui::Align2::LEFT_CENTER,
        right_label,
        font,
        right_color,
    );
}


