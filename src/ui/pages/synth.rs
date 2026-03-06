#![allow(clippy::too_many_arguments)]

use crate::params::DeviceParams;
use crate::ui::SharedUiState;
use crate::midi_learn::{MidiLearnState, SOUND_PARAMS};
use egui_taffy::taffy::{prelude::*, style::{AlignItems, FlexDirection, Overflow}, geometry::Point};
use egui_taffy::TuiBuilderLogic;
use nih_plug::prelude::{Param, ParamSetter};
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
                1 => render_filter_tab(ui, params, setter, ui_state),
                2 => render_envelopes_tab(ui, params, setter, ui_state),
                3 => super::modulation::render_ui(ui, params, setter),
                _ => super::modulation::render_step_mod_ui(ui, params, setter),
            }
        });
    });
}

const TAB_HEIGHT: f32 = 126.0;
const TAB_GAP: f32 = 4.0;

fn render_tab_bar(ui: &mut egui::Ui, current_tab: u8) {
    let rect = ui.max_rect();
    let tab_names = ["OSCs", "FILTER", "ENV & EQ", "LFOs", "STEP MOD"];

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
                            let mut pll_on = params.synth_pll_enable.value();
                            render_toggle(ui, &mut pll_on, "ON", ml!("synth_pll_enable"));
                            if pll_on != params.synth_pll_enable.value() {
                                setter.set_parameter(&params.synth_pll_enable, pll_on);
                            }
                            ui.add_space(60.0);
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
                        let mut saw_on = params.synth_saw_enable.value();
                        render_toggle(ui, &mut saw_on, "ON", ml!("synth_saw_enable"));
                        if saw_on != params.synth_saw_enable.value() {
                            setter.set_parameter(&params.synth_saw_enable, saw_on);
                        }
                        ui.add_space(8.0);
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
                                    let mut vps_on = params.synth_vps_enable.value();
                                    render_toggle(ui, &mut vps_on, "ON", ml!("synth_vps_enable"));
                                    if vps_on != params.synth_vps_enable.value() {
                                        setter.set_parameter(&params.synth_vps_enable, vps_on);
                                    }
                                    ui.add_space(60.0);
                                    let mut vps_fold_pi = params.synth_vps_fold_range.value() == 1;
                                    render_labeled_toggle(ui, &mut vps_fold_pi, "1X", "PI");
                                    let new_range = if vps_fold_pi { 1 } else { 0 };
                                    if new_range != params.synth_vps_fold_range.value() {
                                        setter.set_parameter(&params.synth_vps_fold_range, new_range);
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
    _params: &Arc<DeviceParams>,
    _setter: &ParamSetter,
    _ui_state: &Arc<SharedUiState>,
) {
    egui::Frame::NONE
        .inner_margin(FRAME_MARGIN)
        .show(ui, |ui| {
            ui.label(egui::RichText::new("FILTER").size(HEADER_FONT).strong());
        });
}

fn render_envelopes_tab(
    ui: &mut egui::Ui,
    params: &Arc<DeviceParams>,
    setter: &ParamSetter,
    _ui_state: &Arc<SharedUiState>,
) {
    ui.horizontal(|ui| {
        egui::Frame::NONE
            .inner_margin(egui::Margin { left: FRAME_MARGIN.left + 5, ..FRAME_MARGIN })
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    ui.label(egui::RichText::new("VOLUME ENVELOPE").size(HEADER_FONT).strong());
                    ui.add_space(10.0);
                    render_envelope_controls_compact(ui, params, setter);
                });
            });

        let line_rect = ui.available_rect_before_wrap();
        let sep_x = line_rect.left() - 7.0;
        ui.painter().line_segment(
            [egui::pos2(sep_x, line_rect.top() - 10.0), egui::pos2(sep_x, line_rect.bottom() + 400.0)],
            egui::Stroke::new(1.0, Color32::BLACK),
        );

        egui::Frame::NONE
            .inner_margin(egui::Margin { left: 40, right: 0, ..FRAME_MARGIN })
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
                    ui.add_space(28.0);
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("SUB").size(LABEL_FONT).color(Color32::from_gray(140)));
                        ui.add_space(6.0);
                        let mut sub_in = params.master_hpf_sub.value() == 1;
                        let label_off = "OUT";
                        let label_on = "IN";
                        let desired_size = egui::vec2(48.0, 24.0);
                        let (alloc_rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::click());
                        if response.clicked() {
                            sub_in = !sub_in;
                            setter.set_parameter(&params.master_hpf_sub, if sub_in { 1 } else { 0 });
                        }
                        let rect = alloc_rect.translate(egui::vec2(0.0, -2.0));
                        let anim_t = ui.ctx().animate_bool_with_time(response.id, sub_in, 0.15);
                        let bg_color = Color32::from_gray(50).lerp_to_gamma(Color32::from_rgb(80, 130, 190), anim_t);
                        let circle_x = egui::lerp(rect.left() + 12.0..=rect.right() - 12.0, anim_t);
                        let circle_color = Color32::from_gray(220).lerp_to_gamma(Color32::WHITE, anim_t);
                        ui.painter().rect_filled(rect, rect.height() / 2.0, bg_color);
                        ui.painter().circle_filled(egui::pos2(circle_x, rect.center().y), 9.0, circle_color);
                        let toggle_label = if sub_in { label_on } else { label_off };
                        ui.label(egui::RichText::new(toggle_label).size(LABEL_FONT).color(Color32::from_gray(140)));
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
                            &[(0.0, "OFF"), (0.25, "25"), (0.5, "50"), (0.75, "75"), (1.0, "100")],
                            None,
                        );
                        let stereo_color = Some(Color32::from_rgb(50, 130, 110));
                        ui.add_space(15.0);
                        render_vertical_slider_with_ticks(
                            ui, params, setter,
                            &params.stereo_mono_bass, "MON",
                            0.0, 300.0, SliderScale::Linear,
                            stereo_color,
                            &[(0.0, "OFF"), (80.0, "80"), (120.0, "120"), (200.0, "200"), (300.0, "300")],
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
) {
    let range_ms = params.synth_env_range.modulated_plain_value().max(20.0);
    let time_max_a = range_ms.min(5000.0).max(1.0);
    let time_max_dr = range_ms.min(10000.0).max(1.0);
    let shape_ticks: &[(f32, &str)] = &[(-1.0, "EXP"), (0.0, "LIN"), (1.0, "LOG")];
    let atk_color = Some(Color32::from_rgb(140, 100, 60));
    let dec_color = Some(Color32::from_rgb(100, 80, 120));
    let sus_color = Some(Color32::from_rgb(60, 100, 80));
    let rel_color = Some(Color32::from_rgb(80, 80, 140));
    let dip_color = Some(Color32::from_rgb(140, 80, 80));

    let time_ticks_a = build_time_ticks(0.5, time_max_a);
    let time_ticks_a_ref: Vec<(f32, &str)> = time_ticks_a.iter().map(|(v, s)| (*v, s.as_str())).collect();
    let time_ticks_dr = build_time_ticks(0.5, time_max_dr);
    let time_ticks_dr_ref: Vec<(f32, &str)> = time_ticks_dr.iter().map(|(v, s)| (*v, s.as_str())).collect();

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 6.0;
        render_vertical_slider_with_ticks(
            ui, params, setter,
            &params.synth_vol_attack, "A",
            0.5, time_max_a, SliderScale::Exponential(2.0),
            atk_color, &time_ticks_a_ref, None,
        );
        ui.add_space(2.0);
        render_vertical_slider_with_ticks(
            ui, params, setter,
            &params.synth_vol_attack_shape, "A\u{2009}SH",
            -1.0, 1.0, SliderScale::Linear,
            atk_color, shape_ticks, None,
        );
        ui.add_space(4.0);
        render_vertical_slider_with_ticks(
            ui, params, setter,
            &params.synth_vol_decay, "D",
            0.5, time_max_dr, SliderScale::Exponential(2.0),
            dec_color, &time_ticks_dr_ref, None,
        );
        ui.add_space(2.0);
        render_vertical_slider_with_ticks(
            ui, params, setter,
            &params.synth_vol_decay_shape, "D\u{2009}SH",
            -1.0, 1.0, SliderScale::Linear,
            dec_color, shape_ticks, None,
        );
        ui.add_space(4.0);
        render_vertical_slider(
            ui, params, setter,
            &params.synth_vol_sustain, "S",
            0.0, 1.0, SliderScale::Linear,
            sus_color, None,
        );
        ui.add_space(4.0);
        render_vertical_slider_with_ticks(
            ui, params, setter,
            &params.synth_vol_release, "R",
            0.5, time_max_dr, SliderScale::Exponential(2.0),
            rel_color, &time_ticks_dr_ref, None,
        );
        ui.add_space(2.0);
        render_vertical_slider_with_ticks(
            ui, params, setter,
            &params.synth_vol_release_shape, "R\u{2009}SH",
            -1.0, 1.0, SliderScale::Linear,
            rel_color, shape_ticks, None,
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
    render_adsr_visualization(ui, params, viz_x, viz_y);

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

    let range_ms = params.synth_env_range.modulated_plain_value();
    let max_a = range_ms.min(5000.0).max(1.0);
    let max_dr = range_ms.min(10000.0).max(1.0);

    let attack_ms = params.synth_vol_attack.modulated_plain_value().clamp(0.5, max_a);
    let decay_ms = params.synth_vol_decay.modulated_plain_value().clamp(0.5, max_dr);
    let sustain = params.synth_vol_sustain.modulated_plain_value().clamp(0.0, 1.0);
    let release_ms = params.synth_vol_release.modulated_plain_value().clamp(0.5, max_dr);
    let attack_shape = params.synth_vol_attack_shape.modulated_plain_value().clamp(-1.0, 1.0);
    let decay_shape = params.synth_vol_decay_shape.modulated_plain_value().clamp(-1.0, 1.0);
    let release_shape = params.synth_vol_release_shape.modulated_plain_value().clamp(-1.0, 1.0);

    let dip = params.synth_retrigger_dip.modulated_plain_value().clamp(0.0, 1.0);
    let dip_ms: f32 = 2.0;
    let has_dip = dip > 0.001;

    let tail_time_ms = params.synth_pll_tail_time.modulated_plain_value().clamp(50.0, range_ms.min(5000.0).max(50.0));
    let tail_amount = params.synth_pll_tail_amount.modulated_plain_value().clamp(0.0, 1.0);
    let pll_tail_on = tail_amount > 0.001;

    let adsr_total = attack_ms + decay_ms + release_ms;
    let sustain_ms = adsr_total * 0.2;
    let tail_ms = if pll_tail_on { tail_time_ms } else { 0.0 };
    let effective_dip_ms = if has_dip { dip_ms } else { 0.0 };
    let total_ms = effective_dip_ms + adsr_total + sustain_ms + tail_ms;
    let time_scale = if total_ms > 0.001 { 1.0 / total_ms } else { 1.0 };

    let dip_w = effective_dip_ms * time_scale * w;
    let a_w = attack_ms * time_scale * w;
    let d_w = decay_ms * time_scale * w;
    let s_w = sustain_ms * time_scale * w;
    let r_w = release_ms * time_scale * w;
    let t_w = tail_ms * time_scale * w;

    let x0 = inner.left();
    let y_bot = inner.bottom();

    let shaped_curve = |t: f32, shape: f32| -> f32 {
        if shape > 0.0 {
            1.0 - (1.0 - t).powf(1.0 + shape * 3.0)
        } else if shape < 0.0 {
            t.powf(1.0 + (-shape) * 3.0)
        } else {
            t
        }
    };

    let segments = 16;
    let mut points = Vec::with_capacity(segments * 4 + 10);
    let dip_color = Color32::from_rgb(140, 80, 80);

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
            let v = dip_target_v + shaped_curve(t, attack_shape) * (1.0 - dip_target_v);
            points.push(egui::pos2(dip_x_end + t * a_w, y_bot - v * h));
        }
    } else {
        points.push(egui::pos2(x0, y_bot));

        for i in 0..=segments {
            let t = i as f32 / segments as f32;
            let v = shaped_curve(t, attack_shape);
            points.push(egui::pos2(x0 + t * a_w, y_bot - v * h));
        }
    }

    let x_d_start = x0 + dip_w + a_w;
    for i in 1..=segments {
        let t = i as f32 / segments as f32;
        let v = 1.0 - shaped_curve(t, decay_shape) * (1.0 - sustain);
        points.push(egui::pos2(x_d_start + t * d_w, y_bot - v * h));
    }

    let x_s_end = x_d_start + d_w + s_w;
    points.push(egui::pos2(x_s_end, y_bot - sustain * h));

    let x_r_start = x_s_end;
    for i in 1..=segments {
        let t = i as f32 / segments as f32;
        let v = sustain * (1.0 - shaped_curve(t, release_shape));
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

    let total_adsr_ms = attack_ms + decay_ms + release_ms;
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


