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
                1 => render_envelopes_tab(ui, params, setter, ui_state),
                2 => super::modulation::render_ui(ui, params, setter),
                _ => super::modulation::render_step_mod_ui(ui, params, setter),
            }
        });
    });
}

const TAB_HEIGHT: f32 = 158.0;
const TAB_GAP: f32 = 4.0;

fn render_tab_bar(ui: &mut egui::Ui, current_tab: u8) {
    let rect = ui.max_rect();
    let tab_names = ["SOUND", "ENV & FX", "LFO", "STEP MOD"];

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
                            ui.add_space(60.0);
                            let mut fm_expand = params.synth_pll_fm_expand.value();
                            render_toggle(ui, &mut fm_expand, "EXP", ml!("synth_pll_fm_expand"));
                            if fm_expand != params.synth_pll_fm_expand.value() {
                                setter.set_parameter(&params.synth_pll_fm_expand, fm_expand);
                            }
                        });
                    });
            });
            ui.add_space(10.0);
            ui.horizontal(|ui| {
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
                    0.0, 1.0, SliderScale::Linear,
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
                            0.0, 1.0,
                            Some(Color32::from_rgb(100, 70, 50)),
                            &[(0.22, "10"), (0.56, "30"), (0.78, "60"), (1.0, "120")],
                            ml!("synth_saw_tight"),
                        );
                        render_int_vertical_slider(
                            ui, params, setter,
                            &params.synth_saw_shape_type, "SHPE",
                            Some(Color32::from_rgb(100, 80, 100)),
                            None, None, ml!("synth_saw_shape_type"),
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
                                    let mut shape_fold = params.synth_vps_shape_type.value() == 1;
                                    render_labeled_toggle(ui, &mut shape_fold, "SOFT", "FOLD");
                                    if (shape_fold as i32) != params.synth_vps_shape_type.value() {
                                        setter.set_parameter(&params.synth_vps_shape_type, if shape_fold { 1 } else { 0 });
                                    }
                                });
                            });
                    });
                    ui.add_space(10.0);
                    ui.horizontal(|ui| {
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

fn render_envelopes_tab(
    ui: &mut egui::Ui,
    params: &Arc<DeviceParams>,
    setter: &ParamSetter,
    _ui_state: &Arc<SharedUiState>,
) {
    ui.horizontal(|ui| {
        egui::Frame::NONE
            .inner_margin(FRAME_MARGIN)
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    ui.label(egui::RichText::new("VOL ENV").size(HEADER_FONT).strong());
                    ui.add_space(10.0);
                    render_envelope_controls_compact(ui, params, setter);
                });
            });

        let line_rect = ui.available_rect_before_wrap();
        ui.painter().line_segment(
            [egui::pos2(line_rect.left(), line_rect.top()), egui::pos2(line_rect.left(), line_rect.bottom() - 5.0)],
            egui::Stroke::new(1.0, Color32::BLACK),
        );

        egui::Frame::NONE
            .inner_margin(egui::Margin { left: 40, right: 0, ..FRAME_MARGIN })
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    ui.label(egui::RichText::new("MASTER HPF").size(HEADER_FONT).strong());
                    ui.add_space(14.0);
                    render_hpf_buttons(ui, params, setter);
                    ui.add_space(16.0);
                    ui.label(egui::RichText::new("BOOST").size(LABEL_FONT).color(Color32::from_gray(140)));
                    ui.add_space(6.0);
                    render_hpf_boost_buttons(ui, params, setter);
                    ui.add_space(16.0);
                    ui.label(egui::RichText::new("SUB").size(LABEL_FONT).color(Color32::from_gray(140)));
                    ui.add_space(6.0);
                    render_hpf_sub_buttons(ui, params, setter);
                });
            });

        let line_rect = ui.available_rect_before_wrap();
        ui.painter().line_segment(
            [egui::pos2(line_rect.left(), line_rect.top()), egui::pos2(line_rect.left(), line_rect.bottom() - 5.0)],
            egui::Stroke::new(1.0, Color32::BLACK),
        );

        egui::Frame::NONE
            .inner_margin(egui::Margin { left: 30, right: 0, ..FRAME_MARGIN })
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    ui.label(egui::RichText::new("BRILLIANCE").size(HEADER_FONT).strong());
                    ui.add_space(10.0);
                    let color = Some(Color32::from_rgb(160, 130, 60));
                    render_brilliance_slider(ui, params, setter, color);
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
        ui.spacing_mut().item_spacing.x = 8.0;
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
    let options = [("FLAT", 0), ("MED", 1), ("HIGH", 2)];
    let btn_w = 80.0;
    let btn_h = 48.0;

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 8.0;
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
                setter.set_parameter(&params.master_hpf_boost, *value);
            }
        }
    });
}

fn render_hpf_sub_buttons(
    ui: &mut egui::Ui,
    params: &Arc<DeviceParams>,
    setter: &ParamSetter,
) {
    let current = params.master_hpf_sub.value();
    let options = [("OUT", 0), ("IN", 1)];
    let btn_w = 80.0;
    let btn_h = 48.0;

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 8.0;
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
                setter.set_parameter(&params.master_hpf_sub, *value);
            }
        }
    });
}

fn render_brilliance_slider(
    ui: &mut egui::Ui,
    params: &Arc<DeviceParams>,
    setter: &ParamSetter,
    color: Option<Color32>,
) {
    ui.vertical(|ui| {
        ui.set_width(SLIDER_COL_WIDTH);
        let mut value: f32 = params.brilliance_amount.modulated_plain_value();

        if let Some(fill_color) = color {
            ui.style_mut().visuals.widgets.inactive.bg_fill = fill_color;
            ui.style_mut().visuals.widgets.hovered.bg_fill = fill_color;
            ui.style_mut().visuals.widgets.active.bg_fill = fill_color;
        }

        ui.style_mut().spacing.slider_width = SLIDER_RAIL_LENGTH;
        ui.style_mut().spacing.slider_rail_height = SLIDER_RAIL_THICKNESS;

        let slider = egui::Slider::new(&mut value, 0.0..=1.0)
            .vertical()
            .show_value(false);
        if ui.add(slider).changed() {
            setter.set_parameter(&params.brilliance_amount, value);
            setter.set_parameter(&params.brilliance_drive, value);
        }

        ui.add_space(2.0);
        ui.horizontal(|ui| {
            ui.add_space(-7.0);
            ui.label(egui::RichText::new("AMT").size(LABEL_FONT));
        });
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
    _params: &Arc<DeviceParams>,
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

        ui.add_space(2.0);
        let cc_label = midi_learn.as_ref().and_then(|(ml, param_id)| {
            ml.mappings.try_lock().ok().and_then(|m| m.find_by_param(param_id).map(|cc| format!("CC{}", cc)))
        });
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
            } else {
                egui::RichText::new(label).size(LABEL_FONT)
            };
            if let Some(cc_text) = cc_label {
                ui.vertical(|ui| {
                    ui.label(label_text);
                    ui.label(egui::RichText::new(cc_text).size(10.0).color(Color32::from_rgb(200, 160, 60)));
                });
            } else {
                ui.label(label_text);
            }
        });
    });
}

fn render_vertical_slider_with_ticks<P: Param>(
    ui: &mut egui::Ui,
    _params: &Arc<DeviceParams>,
    setter: &ParamSetter,
    param: &P,
    label: &str,
    min: f32,
    max: f32,
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

        let slider = egui::Slider::new(&mut value, min..=max)
            .vertical()
            .show_value(false);

        let response = ui.add(slider);
        if response.changed() {
            setter.set_parameter(param, value.into());
        }

        if let Some((ml, param_id)) = &midi_learn {
            if learn_active && (response.drag_started() || response.clicked()) {
                if let Ok(mut awaiting) = ml.awaiting_param.try_lock() {
                    *awaiting = Some(param_id.to_string());
                }
            }
        }

        draw_cc_pickup_indicator(ui, response.rect, &midi_learn);

        let rail_rect = response.rect;
        let handle_radius = SLIDER_RAIL_THICKNESS * 0.55;
        let rail_top = rail_rect.top() + handle_radius;
        let rail_bottom = rail_rect.bottom() - handle_radius;
        let rail_height = rail_bottom - rail_top;
        let label_color = Color32::from_gray(55);
        let label_font = egui::FontId::proportional(10.0);
        let label_x = rail_rect.right() + 3.0;

        for &(val, text) in ticks {
            let t = (val - min) / (max - min);
            let y = rail_bottom - t * rail_height;
            let galley = ui.painter().layout_no_wrap(text.to_string(), label_font.clone(), label_color);
            let text_height = galley.size().y;
            ui.painter().galley(egui::pos2(label_x, y - text_height / 2.0), galley, label_color);
        }

        ui.add_space(2.0);
        let cc_label = midi_learn.as_ref().and_then(|(ml, param_id)| {
            ml.mappings.try_lock().ok().and_then(|m| m.find_by_param(param_id).map(|cc| format!("CC{}", cc)))
        });
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
            } else {
                egui::RichText::new(label).size(LABEL_FONT)
            };
            if let Some(cc_text) = cc_label {
                ui.vertical(|ui| {
                    ui.label(label_text);
                    ui.label(egui::RichText::new(cc_text).size(10.0).color(Color32::from_rgb(200, 160, 60)));
                });
            } else {
                ui.label(label_text);
            }
        });
    });
}

fn render_int_vertical_slider(
    ui: &mut egui::Ui,
    _params: &Arc<DeviceParams>,
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
        let cc_label = midi_learn.as_ref().and_then(|(ml, param_id)| {
            ml.mappings.try_lock().ok().and_then(|m| m.find_by_param(param_id).map(|cc| format!("CC{}", cc)))
        });
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
            } else {
                egui::RichText::new(label).size(LABEL_FONT)
            };
            if let Some(cc_text) = cc_label {
                ui.vertical(|ui| {
                    ui.label(label_text);
                    ui.label(egui::RichText::new(cc_text).size(10.0).color(Color32::from_rgb(200, 160, 60)));
                });
            } else {
                ui.label(label_text);
            }
        });
    });
}

fn render_envelope_controls_compact(
    ui: &mut egui::Ui,
    params: &Arc<DeviceParams>,
    setter: &ParamSetter,
) {
    let (attack_param, decay_param, sustain_param, release_param) = (
        &params.synth_vol_attack,
        &params.synth_vol_decay,
        &params.synth_vol_sustain,
        &params.synth_vol_release,
    );

    ui.horizontal(|ui| {
        render_vertical_slider(
            ui, params, setter,
            attack_param, "A",
            1.0, 1000.0, SliderScale::Exponential(2.0),
            None, None,
        );
        ui.add_space(4.0);
        render_vertical_slider(
            ui, params, setter,
            decay_param, "D",
            1.0, 1000.0, SliderScale::Exponential(2.0),
            None, None,
        );
        ui.add_space(4.0);
        render_vertical_slider(
            ui, params, setter,
            sustain_param, "S",
            0.0, 1.0, SliderScale::Exponential(2.0),
            None, None,
        );
        ui.add_space(4.0);
        render_vertical_slider(
            ui, params, setter,
            release_param, "R",
            1.0, 1000.0, SliderScale::Exponential(2.0),
            None, None,
        );
    });
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


