#![allow(clippy::too_many_arguments)]

use crate::params::DeviceParams;
use crate::ui::SharedUiState;
use egui_taffy::taffy::{prelude::*, style::{AlignItems, FlexDirection, Overflow}, geometry::Point};
use egui_taffy::TuiBuilderLogic;
use nih_plug::prelude::{Param, ParamSetter};
use nih_plug_egui::egui;
use nih_plug_egui::egui::Color32;
use std::f32::consts::FRAC_PI_2;
use std::sync::Arc;

const SLIDER_COL_WIDTH: f32 = 62.0;
const SLIDER_RAIL_LENGTH: f32 = 215.0;
const SLIDER_RAIL_THICKNESS: f32 = 18.0;
const LABEL_FONT: f32 = 19.0;
const HEADER_FONT: f32 = 18.0;
const TAB_FONT: f32 = 16.0;
const UI_FONT: f32 = 16.0;
const TAB_BAR_WIDTH: f32 = 52.0;
const FRAME_MARGIN: egui::Margin = egui::Margin { left: 32, right: 4, top: 14, bottom: 14 };

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
                0 => render_sound_tab(ui, params, setter),
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
) {

    egui::Frame::NONE
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
                ui.add_space(625.0);
                egui::Frame::NONE.inner_margin(egui::Margin { left: 0, right: 0, top: 2, bottom: 0 })
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            let mut colored = params.synth_pll_colored.value();
                            render_toggle(ui, &mut colored, "COLOR");
                            if colored != params.synth_pll_colored.value() {
                                setter.set_parameter(&params.synth_pll_colored, colored);
                            }
                            ui.add_space(100.0);
                            let mut edge_mode = params.synth_pll_mode.value();
                            render_toggle(ui, &mut edge_mode, "EDGE");
                            if edge_mode != params.synth_pll_mode.value() {
                                setter.set_parameter(&params.synth_pll_mode, edge_mode);
                            }
                            ui.add_space(80.0);
                            let mut mult_slew_fast = params.synth_pll_mult_slew.value();
                            render_labeled_toggle(ui, &mut mult_slew_fast, "SLOW", "FAST");
                            if mult_slew_fast != params.synth_pll_mult_slew.value() {
                                setter.set_parameter(&params.synth_pll_mult_slew, mult_slew_fast);
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
                    None, None,
                );
                render_int_vertical_slider(
                    ui, params, setter,
                    &params.synth_pll_ref_tune, "TUNE",
                    Some(Color32::from_rgb(80, 80, 40)),
                    Some(&[-12, 0, 12]), None,
                );
                render_int_vertical_slider(
                    ui, params, setter,
                    &params.synth_pll_mult, "MULT",
                    Some(Color32::from_rgb(40, 40, 80)),
                    None,
                    Some(&["1", "2", "4", "8", "16", "32", "64"]),
                );
                render_vertical_slider(
                    ui, params, setter,
                    &params.synth_pll_track_speed, "TRK",
                    0.0, 1.0, SliderScale::Linear,
                    Some(Color32::from_rgb(40, 40, 80)),
                );
                render_vertical_slider(
                    ui, params, setter,
                    &params.synth_pll_damping, "DAMP",
                    0.0, 1.0, SliderScale::Linear,
                    Some(Color32::from_rgb(40, 40, 80)),
                );
                render_vertical_slider(
                    ui, params, setter,
                    &params.synth_pll_influence, "INF",
                    0.0, 1.0, SliderScale::Linear,
                    Some(Color32::from_rgb(40, 40, 80)),
                );
                render_vertical_slider(
                    ui, params, setter,
                    &params.synth_pll_stereo_damp_offset, "STΔ",
                    0.0, 0.5, SliderScale::Linear,
                    Some(Color32::from_rgb(80, 40, 80)),
                );
                render_vertical_slider(
                    ui, params, setter,
                    &params.synth_pll_burst_amount, "OT",
                    0.0, 10.0, SliderScale::Linear,
                    Some(Color32::from_rgb(100, 80, 60)),
                );
                render_vertical_slider(
                    ui, params, setter,
                    &params.synth_pll_color_amount, "SAT",
                    0.0, 1.0, SliderScale::Linear,
                    Some(Color32::from_rgb(60, 80, 100)),
                );
                render_vertical_slider(
                    ui, params, setter,
                    &params.synth_pll_range, "RNG",
                    0.0, 1.0, SliderScale::Linear,
                    Some(Color32::from_rgb(60, 100, 80)),
                );
                render_vertical_slider(
                    ui, params, setter,
                    &params.synth_pll_cross_feedback, "XFB",
                    0.0, 1.0, SliderScale::Linear,
                    Some(Color32::from_rgb(100, 60, 80)),
                );
                render_vertical_slider(
                    ui, params, setter,
                    &params.synth_pll_fm_amount, "FM",
                    0.0, 1.0, SliderScale::Linear,
                    Some(Color32::from_rgb(100, 60, 100)),
                );
                render_int_vertical_slider(
                    ui, params, setter,
                    &params.synth_pll_fm_ratio, "RAT",
                    Some(Color32::from_rgb(100, 60, 100)),
                    None, None,
                );
                render_vertical_slider(
                    ui, params, setter,
                    &params.synth_tube_drive, "TUBE",
                    0.0, 1.0, SliderScale::Linear,
                    Some(Color32::from_rgb(140, 80, 80)),
                );
                render_vertical_slider(
                    ui, params, setter,
                    &params.synth_drift_amount, "DRFT",
                    0.0, 1.0, SliderScale::Linear,
                    Some(Color32::from_rgb(80, 100, 80)),
                );
                render_vertical_slider(
                    ui, params, setter,
                    &params.synth_drift_rate, "RATE",
                    0.1, 10.0, SliderScale::Logarithmic,
                    Some(Color32::from_rgb(80, 100, 80)),
                );
                render_vertical_slider(
                    ui, params, setter,
                    &params.synth_pll_volume, "VOL",
                    0.0, 1.0, SliderScale::Linear,
                    Some(Color32::from_rgb(40, 80, 40)),
                );
            });
        });

    ui.add_space(5.0);
    let sep_rect = ui.available_rect_before_wrap();
    ui.painter().line_segment(
        [egui::pos2(sep_rect.left() + 20.0, sep_rect.top()), egui::pos2(sep_rect.right() - 40.0, sep_rect.top())],
        egui::Stroke::new(1.0, Color32::BLACK),
    );
    ui.add_space(5.0);

    ui.horizontal(|ui| {
        egui::Frame::NONE
            .inner_margin(egui::Margin { left: FRAME_MARGIN.left + 25, right: 0, ..FRAME_MARGIN })
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    ui.label(egui::RichText::new("SUB").size(HEADER_FONT).strong());
                    ui.add_space(10.0);
                    ui.horizontal(|ui| {
                        render_vertical_slider(
                            ui, params, setter,
                            &params.synth_sub_volume, "VOL",
                            0.0, 1.0, SliderScale::Linear,
                            Some(Color32::from_rgb(40, 80, 40)),
                        );
                    });
                });
            });

        let line_rect = ui.available_rect_before_wrap();
        ui.painter().line_segment(
            [egui::pos2(line_rect.left(), sep_rect.top()), egui::pos2(line_rect.left(), line_rect.bottom() - 5.0)],
            egui::Stroke::new(1.0, Color32::BLACK),
        );

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
                        ui.add_space(206.0);
                        egui::Frame::NONE.inner_margin(egui::Margin { left: 0, right: 0, top: 2, bottom: 0 })
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    let mut phase_sync = params.synth_vps_phase_mode.value() == 1;
                                    render_labeled_toggle(ui, &mut phase_sync, "FREE", "SYNC");
                                    if (phase_sync as i32) != params.synth_vps_phase_mode.value() {
                                        setter.set_parameter(&params.synth_vps_phase_mode, if phase_sync { 1 } else { 0 });
                                    }
                                    ui.add_space(82.0);
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
                            None, None,
                        );
                        render_int_vertical_slider(
                            ui, params, setter,
                            &params.synth_osc_tune, "TUNE",
                            Some(Color32::from_rgb(80, 80, 40)),
                            Some(&[-12, 0, 12]), None,
                        );
                        render_vertical_slider(
                            ui, params, setter,
                            &params.synth_osc_d, "D",
                            0.0, 1.0, SliderScale::Linear,
                            Some(Color32::from_rgb(40, 40, 80)),
                        );
                        render_vertical_slider(
                            ui, params, setter,
                            &params.synth_osc_v, "V",
                            0.0, 1.0, SliderScale::Linear,
                            Some(Color32::from_rgb(40, 40, 80)),
                        );
                        render_vertical_slider(
                            ui, params, setter,
                            &params.synth_osc_stereo_v_offset, "VΔ",
                            0.0, 0.3, SliderScale::Linear,
                            Some(Color32::from_rgb(80, 40, 80)),
                        );
                        render_vertical_slider(
                            ui, params, setter,
                            &params.synth_osc_stereo_d_offset, "DΔ",
                            0.0, 0.3, SliderScale::Linear,
                            Some(Color32::from_rgb(80, 40, 80)),
                        );
                        render_vertical_slider(
                            ui, params, setter,
                            &params.synth_osc_fold, "FOLD",
                            0.0, 1.0, SliderScale::Linear,
                            Some(Color32::from_rgb(120, 80, 60)),
                        );
                        render_vertical_slider(
                            ui, params, setter,
                            &params.synth_vps_shape_amount, "SHP",
                            0.0, 1.0, SliderScale::Linear,
                            Some(Color32::from_rgb(120, 80, 60)),
                        );
                        render_vertical_slider(
                            ui, params, setter,
                            &params.synth_osc_volume, "VOL",
                            0.0, 1.0, SliderScale::Linear,
                            Some(Color32::from_rgb(40, 80, 40)),
                        );
                    });
                });
            });

        let line_rect2 = ui.available_rect_before_wrap();
        ui.painter().line_segment(
            [egui::pos2(line_rect2.left(), sep_rect.top()), egui::pos2(line_rect2.left(), line_rect2.bottom() - 5.0)],
            egui::Stroke::new(1.0, Color32::BLACK),
        );

        egui::Frame::NONE
            .inner_margin(egui::Margin { left: 40, right: -10, ..FRAME_MARGIN })
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        egui::Frame::NONE.inner_margin(egui::Margin { left: 0, right: 0, top: 0, bottom: 0 })
                            .show(ui, |ui| {
                                ui.label(
                                    egui::RichText::new("FILTER").size(HEADER_FONT).strong(),
                                );
                            });
                        ui.add_space(16.0);
                        egui::Frame::NONE.inner_margin(egui::Margin { left: 0, right: 0, top: 2, bottom: 0 })
                            .show(ui, |ui| {
                                let mut enabled = params.synth_filter_enable.value();
                                render_toggle(ui, &mut enabled, "ON");
                                if enabled != params.synth_filter_enable.value() {
                                    setter.set_parameter(&params.synth_filter_enable, enabled);
                                }
                            });
                    });
                    ui.add_space(10.0);
                    ui.horizontal(|ui| {
                        render_vertical_slider(
                            ui, params, setter,
                            &params.synth_filter_cutoff, "CUT",
                            20.0, 20000.0, SliderScale::Logarithmic,
                            Some(Color32::from_rgb(180, 120, 60)),
                        );
                        render_vertical_slider(
                            ui, params, setter,
                            &params.synth_filter_resonance, "RES",
                            0.0, 0.98, SliderScale::Linear,
                            Some(Color32::from_rgb(180, 120, 60)),
                        );
                        render_vertical_slider(
                            ui, params, setter,
                            &params.synth_filter_env_amount, "ENV",
                            -5000.0, 5000.0, SliderScale::Linear,
                            Some(Color32::from_rgb(140, 100, 80)),
                        );
                        render_vertical_slider(
                            ui, params, setter,
                            &params.synth_filter_drive, "DRV",
                            1.0, 15.0, SliderScale::Linear,
                            Some(Color32::from_rgb(140, 100, 80)),
                        );
                    });
                });
            });

    });
}

fn render_envelopes_tab(
    ui: &mut egui::Ui,
    params: &Arc<DeviceParams>,
    setter: &ParamSetter,
    ui_state: &Arc<SharedUiState>,
) {

    ui.horizontal(|ui| {
        egui::Frame::NONE
            .inner_margin(FRAME_MARGIN)
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    ui.label(egui::RichText::new("VOL ENV").size(HEADER_FONT).strong());
                    ui.add_space(10.0);
                    render_envelope_controls_compact(ui, params, setter, "vol");
                });
            });

        egui::Frame::NONE
            .inner_margin(FRAME_MARGIN)
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    ui.label(egui::RichText::new("FILT ENV").size(HEADER_FONT).strong());
                    ui.add_space(10.0);
                    render_envelope_controls_compact(ui, params, setter, "filt");
                });
            });
    });

    ui.horizontal(|ui| {
        egui::Frame::NONE
            .inner_margin(FRAME_MARGIN)
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("REVERB").size(HEADER_FONT).strong());
                        ui.add_space(16.0);
                        let mut rev_enabled = params.synth_reverb_enable.value();
                        render_toggle(ui, &mut rev_enabled, "ON");
                        if rev_enabled != params.synth_reverb_enable.value() {
                            setter.set_parameter(&params.synth_reverb_enable, rev_enabled);
                        }
                    });
                    ui.add_space(10.0);
                    ui.horizontal(|ui| {
                        render_vertical_slider(
                            ui, params, setter,
                            &params.synth_reverb_mix, "D/W",
                            0.0, 1.0, SliderScale::Linear,
                            Some(Color32::from_rgb(100, 80, 140)),
                        );
                        render_vertical_slider(
                            ui, params, setter,
                            &params.synth_reverb_time_scale, "SIZE",
                            0.0, 1.0, SliderScale::Linear,
                            None,
                        );
                        render_vertical_slider(
                            ui, params, setter,
                            &params.synth_reverb_decay, "DCY",
                            0.0, 1.0, SliderScale::Linear,
                            None,
                        );
                        render_vertical_slider(
                            ui, params, setter,
                            &params.synth_reverb_diffusion, "DIFF",
                            0.0, 1.0, SliderScale::Linear,
                            None,
                        );
                        render_vertical_slider(
                            ui, params, setter,
                            &params.synth_reverb_pre_delay, "PDL",
                            0.0, 500.0, SliderScale::Linear,
                            None,
                        );
                        render_vertical_slider(
                            ui, params, setter,
                            &params.synth_reverb_mod_depth, "MOD",
                            0.0, 1.0, SliderScale::Linear,
                            None,
                        );
                        render_vertical_slider(
                            ui, params, setter,
                            &params.synth_reverb_hpf, "HPF",
                            20.0, 1000.0, SliderScale::Logarithmic,
                            None,
                        );
                        render_vertical_slider(
                            ui, params, setter,
                            &params.synth_reverb_lpf, "LPF",
                            1000.0, 22000.0, SliderScale::Logarithmic,
                            None,
                        );
                        render_vertical_slider(
                            ui, params, setter,
                            &params.synth_reverb_ducking, "DUCK",
                            0.0, 1.0, SliderScale::Linear,
                            None,
                        );
                    });
                });
            });

        egui::Frame::NONE
            .inner_margin(egui::Margin { left: 16, right: 16, top: 12, bottom: 12 })
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    ui.label(egui::RichText::new("PERF").size(HEADER_FONT).strong());
                    ui.add_space(8.0);

                    let cpu_load = ui_state.get_cpu_load();
                    let cpu_color = if cpu_load > 80.0 {
                        Color32::from_rgb(200, 80, 80)
                    } else if cpu_load > 50.0 {
                        Color32::from_rgb(200, 180, 80)
                    } else {
                        Color32::from_rgb(80, 200, 80)
                    };
                    ui.label(
                        egui::RichText::new(format!("CPU: {:02}%", cpu_load as u32))
                            .size(UI_FONT)
                            .color(cpu_color),
                    );

                    ui.add_space(8.0);
                    ui.separator();
                    ui.add_space(8.0);

                    let mut pll = params.synth_pll_enable.value();
                    if ui
                        .checkbox(&mut pll, egui::RichText::new("PLL").size(UI_FONT))
                        .changed()
                    {
                        setter.set_parameter(&params.synth_pll_enable, pll);
                    }

                    let mut vps = params.synth_vps_enable.value();
                    if ui
                        .checkbox(&mut vps, egui::RichText::new("VPS").size(UI_FONT))
                        .changed()
                    {
                        setter.set_parameter(&params.synth_vps_enable, vps);
                    }

                    let mut color = params.synth_coloration_enable.value();
                    if ui
                        .checkbox(&mut color, egui::RichText::new("Color").size(UI_FONT))
                        .changed()
                    {
                        setter.set_parameter(&params.synth_coloration_enable, color);
                    }

                    ui.add_space(8.0);

                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("OS:").size(UI_FONT));
                        let os_factor = params.synth_oversampling_factor.value();
                        let os_label = match os_factor {
                            0 => "1x",
                            1 => "2x",
                            2 => "4x",
                            3 => "8x",
                            _ => "16x",
                        };
                        egui::ComboBox::from_id_salt("os_selector")
                            .width(70.0)
                            .selected_text(egui::RichText::new(os_label).size(UI_FONT))
                            .show_ui(ui, |ui| {
                                for (val, label) in
                                    [(0, "1x"), (1, "2x"), (2, "4x"), (3, "8x"), (4, "16x")]
                                {
                                    let btn =
                                        egui::Button::new(egui::RichText::new(label).size(UI_FONT))
                                            .min_size(egui::vec2(60.0, 36.0))
                                            .selected(os_factor == val);
                                    if ui.add(btn).clicked() {
                                        setter.set_parameter(
                                            &params.synth_oversampling_factor,
                                            val,
                                        );
                                        ui.close_menu();
                                    }
                                }
                            });
                    });
                });
            });
    });
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
) where
    P::Plain: Into<f32>,
    f32: Into<P::Plain>,
{
    ui.vertical(|ui| {
        ui.set_width(SLIDER_COL_WIDTH);
        let plain_value = param.modulated_plain_value();
        let mut value: f32 = plain_value.into();

        if let Some(fill_color) = color {
            ui.style_mut().visuals.widgets.inactive.bg_fill = fill_color;
            ui.style_mut().visuals.widgets.hovered.bg_fill = fill_color;
            ui.style_mut().visuals.widgets.active.bg_fill = fill_color;
        }

        ui.style_mut().spacing.slider_width = SLIDER_RAIL_LENGTH;
        ui.style_mut().spacing.slider_rail_height = SLIDER_RAIL_THICKNESS;

        match scale {
            SliderScale::Linear => {
                let slider = egui::Slider::new(&mut value, min..=max)
                    .vertical()
                    .show_value(false);
                if ui.add(slider).changed() {
                    setter.set_parameter(param, value.into());
                }
            }
            SliderScale::Logarithmic => {
                let slider = egui::Slider::new(&mut value, min..=max)
                    .vertical()
                    .logarithmic(true)
                    .show_value(false);
                if ui.add(slider).changed() {
                    setter.set_parameter(param, value.into());
                }
            }
            SliderScale::Exponential(exponent) => {
                let normalized = (value - min) / (max - min);
                let mut slider_value = normalized.powf(1.0 / exponent);

                let slider = egui::Slider::new(&mut slider_value, 0.0..=1.0)
                    .vertical()
                    .show_value(false);

                if ui.add(slider).changed() {
                    let new_normalized = slider_value.powf(exponent);
                    value = min + new_normalized * (max - min);
                    setter.set_parameter(param, value.into());
                }
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
    _params: &Arc<DeviceParams>,
    setter: &ParamSetter,
    param: &nih_plug::prelude::IntParam,
    label: &str,
    color: Option<Color32>,
    tick_labels: Option<&[i32]>,
    value_display: Option<&[&str]>,
) {
    ui.vertical(|ui| {
        ui.set_width(SLIDER_COL_WIDTH);
        let mut value = param.value();

        if let Some(fill_color) = color {
            ui.style_mut().visuals.widgets.inactive.bg_fill = fill_color;
            ui.style_mut().visuals.widgets.hovered.bg_fill = fill_color;
            ui.style_mut().visuals.widgets.active.bg_fill = fill_color;
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

fn render_envelope_controls_compact(
    ui: &mut egui::Ui,
    params: &Arc<DeviceParams>,
    setter: &ParamSetter,
    prefix: &str,
) {
    let (attack_param, decay_param, sustain_param, release_param) = match prefix {
        "vol" => (
            &params.synth_vol_attack,
            &params.synth_vol_decay,
            &params.synth_vol_sustain,
            &params.synth_vol_release,
        ),
        "filt" => (
            &params.synth_filt_attack,
            &params.synth_filt_decay,
            &params.synth_filt_sustain,
            &params.synth_filt_release,
        ),
        _ => panic!("Invalid prefix"),
    };

    ui.horizontal(|ui| {
        render_vertical_slider(
            ui, params, setter,
            attack_param, "A",
            1.0, 1000.0, SliderScale::Exponential(2.0),
            None,
        );
        ui.add_space(4.0);
        render_vertical_slider(
            ui, params, setter,
            decay_param, "D",
            1.0, 1000.0, SliderScale::Exponential(2.0),
            None,
        );
        ui.add_space(4.0);
        render_vertical_slider(
            ui, params, setter,
            sustain_param, "S",
            0.0, 1.0, SliderScale::Exponential(2.0),
            None,
        );
        ui.add_space(4.0);
        render_vertical_slider(
            ui, params, setter,
            release_param, "R",
            1.0, 1000.0, SliderScale::Exponential(2.0),
            None,
        );
    });
}

fn render_toggle(ui: &mut egui::Ui, value: &mut bool, label: &str) {
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

    let text_color = if *value { egui::Color32::WHITE } else { egui::Color32::from_gray(140) };
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

