#![allow(clippy::too_many_arguments)]

use crate::params::DeviceParams;
use crate::ui::SharedUiState;
use egui_taffy::taffy::{prelude::*, style::AlignItems};
use egui_taffy::TuiBuilderLogic;
use nih_plug::prelude::{Param, ParamSetter};
use nih_plug_egui::egui;
use nih_plug_egui::egui::Color32;
use std::sync::Arc;

pub fn render(tui: &mut egui_taffy::Tui, params: &Arc<DeviceParams>, setter: &ParamSetter, ui_state: &Arc<SharedUiState>) {
    // Get the current tab state from memory (0 = Sound, 1 = Envelopes & FX)
    let mut current_tab = tui.ui(|ui| {
        ui.memory_mut(|mem| {
            *mem.data.get_temp_mut_or(egui::Id::new("synth_tab"), 0u8)
        })
    });

    tui.ui(|ui| {
        ui.add_space(12.0);
        ui.horizontal(|ui| {
            ui.heading(egui::RichText::new("    Synth").size(22.0));
            ui.add_space(40.0);

            // Tab buttons
            let button_sound = egui::Button::new(egui::RichText::new("Sound").size(14.0))
                .min_size(egui::vec2(80.0, 28.0))
                .corner_radius(egui::CornerRadius { nw: 6, ne: 6, sw: 0, se: 0 })
                .selected(current_tab == 0);
            if ui.add(button_sound).clicked() {
                current_tab = 0;
            }

            let button_env = egui::Button::new(egui::RichText::new("Envelopes & FX").size(14.0))
                .min_size(egui::vec2(140.0, 28.0))
                .corner_radius(egui::CornerRadius { nw: 6, ne: 6, sw: 0, se: 0 })
                .selected(current_tab == 1);
            if ui.add(button_env).clicked() {
                current_tab = 1;
            }
        });
        ui.add_space(8.0);

        // Store the updated tab state
        ui.memory_mut(|mem| {
            mem.data.insert_temp(egui::Id::new("synth_tab"), current_tab);
        });
    });

    tui.style(Style {
        flex_grow: 1.0,
        align_items: Some(AlignItems::Stretch),
        ..Default::default()
    })
    .ui(|ui| {
        // Tab 0: Sound - PLL, Sub, VPS, Saw/Pulse, and Filter
        if current_tab == 0 {
            ui.horizontal(|ui| {
                egui::Frame::default()
                    .fill(ui.visuals().extreme_bg_color)
                    .inner_margin(16.0)
                    .stroke(egui::Stroke::new(1.0, ui.visuals().window_stroke.color))
                    .corner_radius(15.0)
                    .show(ui, |ui| {
                    ui.vertical(|ui| {
                        ui.label(
                            egui::RichText::new("   Phase Locked Loop OSC")
                                .size(16.0)
                                .strong(),
                        );
                        ui.add_space(12.0);
                        ui.horizontal(|ui| {
                            render_int_vertical_slider(
                                ui,
                                params,
                                setter,
                                &params.synth_pll_ref_octave,
                                "Oct",
                                &["-5", "-4", "-3", "-2", "-1", "0", "+1", "+2", "+3", "+4", "+5"],
                                Some(Color32::from_rgb(80, 80, 40)),
                            );
                            render_vertical_slider(
                                ui,
                                params,
                                setter,
                                &params.synth_pll_ref_pulse_width,
                                "PW",
                                0.01,
                                0.99,
                                SliderScale::Linear,
                                |v| format!("{:.2}", v),
                                Some(Color32::from_rgb(80, 80, 40)),
                            );
                            render_vertical_slider(
                                ui,
                                params,
                                setter,
                                &params.synth_pll_track_speed,
                                "Trk",
                                0.0,
                                1.0,
                                SliderScale::Linear,
                                |v| {
                                    if v < 0.3 {
                                        format!("Glide {:.2}", v)
                                    } else if v > 0.7 {
                                        format!("OT {:.2}", v)
                                    } else {
                                        format!("{:.2}", v)
                                    }
                                },
                                Some(Color32::from_rgb(40, 40, 80)),
                            );
                            render_vertical_slider(
                                ui,
                                params,
                                setter,
                                &params.synth_pll_damping,
                                "Dmp",
                                0.0,
                                1.0,
                                SliderScale::Linear,
                                |v| format!("{:.2}", v),
                                Some(Color32::from_rgb(40, 40, 80)),
                            );
                            render_vertical_slider(
                                ui,
                                params,
                                setter,
                                &params.synth_pll_influence,
                                "Inf",
                                0.0,
                                1.0,
                                SliderScale::Linear,
                                |v| format!("{:.2}", v),
                                Some(Color32::from_rgb(40, 40, 80)),
                            );
                            render_vertical_slider(
                                ui,
                                params,
                                setter,
                                &params.synth_pll_stereo_damp_offset,
                                "StΔ",
                                0.0,
                                0.5,
                                SliderScale::Linear,
                                |v| format!("{:.3}", v),
                                Some(Color32::from_rgb(80, 40, 80)),
                            );
                            render_int_vertical_slider(
                                ui,
                                params,
                                setter,
                                &params.synth_pll_mult,
                                "Mlt",
                                &["×1", "×2", "×4", "×8", "×16", "×32", "×64"],
                                Some(Color32::from_rgb(40, 40, 80)),
                            );
                            render_vertical_slider(
                                ui,
                                params,
                                setter,
                                &params.synth_pll_feedback,
                                "FB",
                                0.0,
                                1.0,
                                SliderScale::Linear,
                                |v| format!("{:.2}", v),
                                Some(Color32::from_rgb(40, 40, 80)),
                            );
                            render_vertical_slider(
                                ui,
                                params,
                                setter,
                                &params.synth_pll_glide,
                                "Glide",
                                0.0,
                                2000.0,
                                SliderScale::Logarithmic,
                                |v| {
                                    if v < 1.0 {
                                        "Off".to_string()
                                    } else if v < 1000.0 {
                                        format!("{:.0}ms", v)
                                    } else {
                                        format!("{:.1}s", v / 1000.0)
                                    }
                                },
                                Some(Color32::from_rgb(80, 60, 100)),
                            );
                            render_vertical_slider(
                                ui,
                                params,
                                setter,
                                &params.synth_pll_retrigger,
                                "Rst",
                                0.0,
                                1.0,
                                SliderScale::Linear,
                                |v| {
                                    if v < 0.1 {
                                        "Hard".to_string()
                                    } else if v > 0.9 {
                                        "Leg".to_string()
                                    } else {
                                        format!("{:.2}", v)
                                    }
                                },
                                Some(Color32::from_rgb(100, 60, 60)),
                            );
                            render_vertical_slider(
                                ui,
                                params,
                                setter,
                                &params.synth_pll_burst_amount,
                                "OT",
                                0.0,
                                10.0,
                                SliderScale::Linear,
                                |v| format!("{:.1}", v),
                                Some(Color32::from_rgb(100, 80, 60)),
                            );
                            render_vertical_slider(
                                ui,
                                params,
                                setter,
                                &params.synth_pll_color_amount,
                                "Sat",
                                0.0,
                                1.0,
                                SliderScale::Linear,
                                |v| format!("{:.2}", v),
                                Some(Color32::from_rgb(60, 80, 100)),
                            );
                            render_vertical_slider(
                                ui,
                                params,
                                setter,
                                &params.synth_pll_range,
                                "Rng",
                                0.0,
                                1.0,
                                SliderScale::Linear,
                                |v| {
                                    if v < 0.01 {
                                        "Slow".to_string()
                                    } else if v > 0.99 {
                                        "Fast".to_string()
                                    } else {
                                        format!("{:.2}", v)
                                    }
                                },
                                Some(Color32::from_rgb(60, 100, 80)),
                            );
                            render_vertical_slider(
                                ui,
                                params,
                                setter,
                                &params.synth_pll_stereo_track_offset,
                                "StW",
                                0.0,
                                0.5,
                                SliderScale::Linear,
                                |v| format!("{:.2}", v),
                                Some(Color32::from_rgb(80, 60, 100)),
                            );
                            render_vertical_slider(
                                ui,
                                params,
                                setter,
                                &params.synth_pll_stereo_phase,
                                "StPh",
                                0.0,
                                1.0,
                                SliderScale::Linear,
                                |v| format!("{:.0}°", v * 360.0),
                                Some(Color32::from_rgb(80, 60, 100)),
                            );
                            render_vertical_slider(
                                ui,
                                params,
                                setter,
                                &params.synth_pll_cross_feedback,
                                "XFB",
                                0.0,
                                1.0,
                                SliderScale::Linear,
                                |v| format!("{:.2}", v),
                                Some(Color32::from_rgb(100, 60, 80)),
                            );
                            render_vertical_slider(
                                ui,
                                params,
                                setter,
                                &params.synth_pll_fm_amount,
                                "FM",
                                0.0,
                                1.0,
                                SliderScale::Linear,
                                |v| format!("{:.2}", v),
                                Some(Color32::from_rgb(100, 60, 100)),
                            );
                            render_int_vertical_slider(
                                ui,
                                params,
                                setter,
                                &params.synth_pll_fm_ratio,
                                "Rat",
                                &["×1", "×2", "×3", "×4", "×5", "×6", "×7", "×8"],
                                Some(Color32::from_rgb(100, 60, 100)),
                            );
                            render_vertical_slider(
                                ui,
                                params,
                                setter,
                                &params.synth_pll_fm_env_amount,
                                "FME",
                                0.0,
                                1.0,
                                SliderScale::Linear,
                                |v| format!("{:.2}", v),
                                Some(Color32::from_rgb(100, 60, 100)),
                            );
                            render_vertical_slider(
                                ui,
                                params,
                                setter,
                                &params.synth_pll_volume,
                                "Vol",
                                0.0,
                                1.0,
                                SliderScale::Linear,
                                |v| format!("{:.2}", v),
                                Some(Color32::from_rgb(40, 80, 40)),
                            );

                            ui.vertical(|ui| {
                                ui.horizontal(|ui| {
                                    ui.label(egui::RichText::new("Color").size(12.0));
                                    let mut colored = params.synth_pll_colored.value();
                                    if ui.checkbox(&mut colored, "").changed() {
                                        setter.set_parameter(&params.synth_pll_colored, colored);
                                    }
                                });
                                ui.add_space(4.0);
                                ui.horizontal(|ui| {
                                    ui.label(egui::RichText::new("Edge").size(12.0));
                                    let mut edge_mode = params.synth_pll_mode.value();
                                    if ui.checkbox(&mut edge_mode, "").changed() {
                                        setter.set_parameter(&params.synth_pll_mode, edge_mode);
                                    }
                                });
                            });
                        });
                    });
                });
        });

        ui.add_space(10.0);

        ui.horizontal(|ui| {
            egui::Frame::default()
                .fill(ui.visuals().extreme_bg_color)
                .inner_margin(egui::Margin { left: 16, right: 16, top: 16, bottom: 16 })
                .stroke(egui::Stroke::new(1.0, ui.visuals().window_stroke.color))
                .corner_radius(15.0)
                .show(ui, |ui| {
                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new("   Sub").size(16.0).strong());
                        ui.add_space(12.0);
                        ui.horizontal(|ui| {
                            render_vertical_slider(
                                ui,
                                params,
                                setter,
                                &params.synth_sub_volume,
                                "Vol",
                                0.0,
                                1.0,
                                SliderScale::Linear,
                                |v| format!("{:.2}", v),
                                Some(Color32::from_rgb(40, 80, 40)),
                            );
                        });
                    });
                });

            egui::Frame::default()
                .fill(ui.visuals().extreme_bg_color)
                .inner_margin(egui::Margin { left: 16, right: 8, top: 16, bottom: 16 })
                .stroke(egui::Stroke::new(1.0, ui.visuals().window_stroke.color))
                .corner_radius(15.0)
                .show(ui, |ui| {
                    ui.vertical(|ui| {
                        ui.label(
                            egui::RichText::new("   Vector Phase Shaping OSC")
                                .size(16.0)
                                .strong(),
                        );
                        ui.add_space(12.0);
                        ui.horizontal(|ui| {
                            render_int_vertical_slider(
                                ui,
                                params,
                                setter,
                                &params.synth_osc_octave,
                                "Oct",
                                &["-5", "-4", "-3", "-2", "-1", "0", "+1", "+2", "+3", "+4", "+5"],
                                Some(Color32::from_rgb(80, 80, 40)),
                            );
                            render_vertical_slider(
                                ui,
                                params,
                                setter,
                                &params.synth_osc_d,
                                "D",
                                0.0,
                                1.0,
                                SliderScale::Linear,
                                |v| format!("{:.2}", v),
                                Some(Color32::from_rgb(40, 40, 80)),
                            );
                            render_vertical_slider(
                                ui,
                                params,
                                setter,
                                &params.synth_osc_v,
                                "V",
                                0.0,
                                1.0,
                                SliderScale::Linear,
                                |v| format!("{:.2}", v),
                                Some(Color32::from_rgb(40, 40, 80)),
                            );
                            render_vertical_slider(
                                ui,
                                params,
                                setter,
                                &params.synth_osc_stereo_v_offset,
                                "StΔ",
                                0.0,
                                0.3,
                                SliderScale::Linear,
                                |v| format!("{:.3}", v),
                                Some(Color32::from_rgb(80, 40, 80)),
                            );
                            render_vertical_slider(
                                ui,
                                params,
                                setter,
                                &params.synth_osc_volume,
                                "Vol",
                                0.0,
                                1.0,
                                SliderScale::Linear,
                                |v| format!("{:.2}", v),
                                Some(Color32::from_rgb(40, 80, 40)),
                            );
                        });
                    });
                });

            ui.add_space(8.0);

            ui.add_space(8.0);

            egui::Frame::default()
                .fill(ui.visuals().extreme_bg_color)
                .inner_margin(egui::Margin { left: 16, right: 8, top: 16, bottom: 16 })
                .stroke(egui::Stroke::new(1.0, ui.visuals().window_stroke.color))
                .corner_radius(15.0)
                .show(ui, |ui| {
                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new("   Color").size(16.0).strong());
                        ui.add_space(12.0);
                        ui.horizontal(|ui| {
                            render_vertical_slider(
                                ui,
                                params,
                                setter,
                                &params.synth_ring_mod,
                                "Ring",
                                0.0,
                                1.0,
                                SliderScale::Linear,
                                |v| format!("{:.2}", v),
                                Some(Color32::from_rgb(120, 80, 60)),
                            );
                            render_vertical_slider(
                                ui,
                                params,
                                setter,
                                &params.synth_wavefold,
                                "Fold",
                                0.0,
                                1.0,
                                SliderScale::Linear,
                                |v| format!("{:.2}", v),
                                Some(Color32::from_rgb(120, 80, 60)),
                            );
                            render_vertical_slider(
                                ui,
                                params,
                                setter,
                                &params.synth_drift_amount,
                                "Drft",
                                0.0,
                                1.0,
                                SliderScale::Linear,
                                |v| format!("{:.2}", v),
                                Some(Color32::from_rgb(80, 100, 80)),
                            );
                            render_vertical_slider(
                                ui,
                                params,
                                setter,
                                &params.synth_drift_rate,
                                "Rate",
                                0.1,
                                10.0,
                                SliderScale::Logarithmic,
                                |v| format!("{:.1}", v),
                                Some(Color32::from_rgb(80, 100, 80)),
                            );
                            render_vertical_slider(
                                ui,
                                params,
                                setter,
                                &params.synth_noise_amount,
                                "Nois",
                                0.0,
                                1.0,
                                SliderScale::Linear,
                                |v| format!("{:.2}", v),
                                Some(Color32::from_rgb(100, 100, 60)),
                            );
                            render_vertical_slider(
                                ui,
                                params,
                                setter,
                                &params.synth_tube_drive,
                                "Tube",
                                0.0,
                                1.0,
                                SliderScale::Linear,
                                |v| format!("{:.2}", v),
                                Some(Color32::from_rgb(140, 80, 80)),
                            );
                            render_vertical_slider(
                                ui,
                                params,
                                setter,
                                &params.synth_color_distortion_amount,
                                "Dist",
                                0.0,
                                1.0,
                                SliderScale::Linear,
                                |v| format!("{:.2}", v),
                                Some(Color32::from_rgb(180, 60, 60)),
                            );
                            render_vertical_slider(
                                ui,
                                params,
                                setter,
                                &params.synth_color_distortion_threshold,
                                "Thr",
                                0.1,
                                1.0,
                                SliderScale::Linear,
                                |v| format!("{:.2}", v),
                                Some(Color32::from_rgb(180, 60, 60)),
                            );
                        });
                    });
                });

            egui::Frame::default()
                .fill(ui.visuals().extreme_bg_color)
                .inner_margin(egui::Margin { left: 16, right: 16, top: 16, bottom: 16 })
                .stroke(egui::Stroke::new(1.0, ui.visuals().window_stroke.color))
                .corner_radius(15.0)
                .show(ui, |ui| {
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("   Moog Filter").size(16.0).strong());
                            ui.add_space(16.0);
                            let mut enabled = params.synth_filter_enable.value();
                            if ui.checkbox(&mut enabled, "").changed() {
                                setter.set_parameter(&params.synth_filter_enable, enabled);
                            }
                        });
                        ui.add_space(12.0);
                        ui.horizontal(|ui| {
                            render_vertical_slider(
                                ui,
                                params,
                                setter,
                                &params.synth_filter_cutoff,
                                "Cut",
                                20.0,
                                20000.0,
                                SliderScale::Logarithmic,
                                |v| format!("{:.0}", v),
                                Some(Color32::from_rgb(180, 120, 60)),
                            );
                            render_vertical_slider(
                                ui,
                                params,
                                setter,
                                &params.synth_filter_resonance,
                                "Res",
                                0.0,
                                0.98,
                                SliderScale::Linear,
                                |v| format!("{:.2}", v),
                                Some(Color32::from_rgb(180, 120, 60)),
                            );
                            render_vertical_slider(
                                ui,
                                params,
                                setter,
                                &params.synth_filter_env_amount,
                                "Env",
                                -5000.0,
                                5000.0,
                                SliderScale::Linear,
                                |v| format!("{:.0}", v),
                                Some(Color32::from_rgb(140, 100, 80)),
                            );
                            render_vertical_slider(
                                ui,
                                params,
                                setter,
                                &params.synth_filter_drive,
                                "Drv",
                                1.0,
                                15.0,
                                SliderScale::Linear,
                                |v| format!("{:.1}", v),
                                Some(Color32::from_rgb(140, 100, 80)),
                            );
                        });
                    });
                });
        });
        }
        // Tab 1: Envelopes & FX
        else if current_tab == 1 {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label(egui::RichText::new("VOL ENV").size(16.0).strong());
                    ui.add_space(8.0);
                    render_envelope_controls_compact(ui, params, setter, "vol");
                });

                ui.add_space(40.0);

                ui.vertical(|ui| {
                    ui.label(egui::RichText::new("FILT ENV").size(16.0).strong());
                    ui.add_space(8.0);
                    render_envelope_controls_compact(ui, params, setter, "filt");
                });

                ui.add_space(40.0);

                egui::Frame::default()
                    .fill(ui.visuals().extreme_bg_color)
                    .inner_margin(egui::Margin { left: 16, right: 16, top: 16, bottom: 16 })
                    .stroke(egui::Stroke::new(1.0, ui.visuals().window_stroke.color))
                    .corner_radius(15.0)
                    .show(ui, |ui| {
                        ui.vertical(|ui| {
                            ui.label(egui::RichText::new("REVERB").size(16.0).strong());
                            ui.add_space(12.0);
                            ui.horizontal(|ui| {
                                render_vertical_slider(
                                    ui,
                                    params,
                                    setter,
                                    &params.synth_reverb_mix,
                                    "Dry/Wet",
                                    0.0,
                                    1.0,
                                    SliderScale::Linear,
                                    |v| format!("{:.0}%", v * 100.0),
                                    Some(Color32::from_rgb(100, 80, 140)),
                                );
                                render_vertical_slider(
                                    ui,
                                    params,
                                    setter,
                                    &params.synth_reverb_time_scale,
                                    "Size",
                                    0.0,
                                    1.0,
                                    SliderScale::Linear,
                                    |v| format!("{:.2}", v),
                                    None,
                                );
                                render_vertical_slider(
                                    ui,
                                    params,
                                    setter,
                                    &params.synth_reverb_decay,
                                    "Decay",
                                    0.0,
                                    1.0,
                                    SliderScale::Linear,
                                    |v| format!("{:.2}", v),
                                    None,
                                );
                                render_vertical_slider(
                                    ui,
                                    params,
                                    setter,
                                    &params.synth_reverb_diffusion,
                                    "Diff",
                                    0.0,
                                    1.0,
                                    SliderScale::Linear,
                                    |v| format!("{:.2}", v),
                                    None,
                                );
                                render_vertical_slider(
                                    ui,
                                    params,
                                    setter,
                                    &params.synth_reverb_pre_delay,
                                    "PreDly",
                                    0.0,
                                    500.0,
                                    SliderScale::Linear,
                                    |v| format!("{:.0}ms", v),
                                    None,
                                );
                                render_vertical_slider(
                                    ui,
                                    params,
                                    setter,
                                    &params.synth_reverb_mod_depth,
                                    "ModDep",
                                    0.0,
                                    1.0,
                                    SliderScale::Linear,
                                    |v| format!("{:.2}", v),
                                    None,
                                );
                                render_vertical_slider(
                                    ui,
                                    params,
                                    setter,
                                    &params.synth_reverb_hpf,
                                    "HPF",
                                    20.0,
                                    1000.0,
                                    SliderScale::Logarithmic,
                                    |v| format!("{:.0}Hz", v),
                                    None,
                                );
                                render_vertical_slider(
                                    ui,
                                    params,
                                    setter,
                                    &params.synth_reverb_lpf,
                                    "LPF",
                                    1000.0,
                                    22000.0,
                                    SliderScale::Logarithmic,
                                    |v| format!("{:.0}Hz", v),
                                    None,
                                );
                                render_vertical_slider(
                                    ui,
                                    params,
                                    setter,
                                    &params.synth_reverb_ducking,
                                    "Duck",
                                    0.0,
                                    1.0,
                                    SliderScale::Linear,
                                    |v| format!("{:.2}", v),
                                    None,
                                );
                            });
                        });
                    });
            });

            ui.add_space(24.0);

            egui::Frame::default()
                .fill(ui.visuals().extreme_bg_color)
                .inner_margin(egui::Margin { left: 16, right: 16, top: 12, bottom: 12 })
                .stroke(egui::Stroke::new(1.0, ui.visuals().window_stroke.color))
                .corner_radius(10.0)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("PERF").size(16.0).strong());
                        ui.add_space(24.0);

                        let cpu_load = ui_state.get_cpu_load();
                        let cpu_color = if cpu_load > 80.0 {
                            Color32::from_rgb(200, 80, 80)
                        } else if cpu_load > 50.0 {
                            Color32::from_rgb(200, 180, 80)
                        } else {
                            Color32::from_rgb(80, 200, 80)
                        };
                        ui.label(egui::RichText::new(format!("CPU: {:02}%", cpu_load as u32))
                            .size(14.0)
                            .color(cpu_color));

                        ui.add_space(32.0);
                        ui.separator();
                        ui.add_space(16.0);

                        let mut pll = params.synth_pll_enable.value();
                        if ui.checkbox(&mut pll, egui::RichText::new("PLL").size(14.0)).changed() {
                            setter.set_parameter(&params.synth_pll_enable, pll);
                        }

                        let mut vps = params.synth_vps_enable.value();
                        if ui.checkbox(&mut vps, egui::RichText::new("VPS").size(14.0)).changed() {
                            setter.set_parameter(&params.synth_vps_enable, vps);
                        }

                        let mut color = params.synth_coloration_enable.value();
                        if ui.checkbox(&mut color, egui::RichText::new("Color").size(14.0)).changed() {
                            setter.set_parameter(&params.synth_coloration_enable, color);
                        }

                        let mut reverb = params.synth_reverb_enable.value();
                        if ui.checkbox(&mut reverb, egui::RichText::new("Reverb").size(14.0)).changed() {
                            setter.set_parameter(&params.synth_reverb_enable, reverb);
                        }

                        ui.add_space(12.0);
                        ui.label(egui::RichText::new("OS:").size(14.0));
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
                            .selected_text(egui::RichText::new(os_label).size(14.0))
                            .show_ui(ui, |ui| {
                                let btn_1x = egui::Button::new(egui::RichText::new("1x").size(14.0))
                                    .min_size(egui::vec2(60.0, 36.0))
                                    .selected(os_factor == 0);
                                if ui.add(btn_1x).clicked() {
                                    setter.set_parameter(&params.synth_oversampling_factor, 0);
                                    ui.close_menu();
                                }
                                let btn_2x = egui::Button::new(egui::RichText::new("2x").size(14.0))
                                    .min_size(egui::vec2(60.0, 36.0))
                                    .selected(os_factor == 1);
                                if ui.add(btn_2x).clicked() {
                                    setter.set_parameter(&params.synth_oversampling_factor, 1);
                                    ui.close_menu();
                                }
                                let btn_4x = egui::Button::new(egui::RichText::new("4x").size(14.0))
                                    .min_size(egui::vec2(60.0, 36.0))
                                    .selected(os_factor == 2);
                                if ui.add(btn_4x).clicked() {
                                    setter.set_parameter(&params.synth_oversampling_factor, 2);
                                    ui.close_menu();
                                }
                                let btn_8x = egui::Button::new(egui::RichText::new("8x").size(14.0))
                                    .min_size(egui::vec2(60.0, 36.0))
                                    .selected(os_factor == 3);
                                if ui.add(btn_8x).clicked() {
                                    setter.set_parameter(&params.synth_oversampling_factor, 3);
                                    ui.close_menu();
                                }
                                let btn_16x = egui::Button::new(egui::RichText::new("16x").size(14.0))
                                    .min_size(egui::vec2(60.0, 36.0))
                                    .selected(os_factor == 4);
                                if ui.add(btn_16x).clicked() {
                                    setter.set_parameter(&params.synth_oversampling_factor, 4);
                                    ui.close_menu();
                                }
                            });
                    });
                });
        }
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
    formatter: impl Fn(f32) -> String,
    color: Option<Color32>,
) where
    P::Plain: Into<f32>,
    f32: Into<P::Plain>,
{
    ui.vertical(|ui| {
        ui.set_width(48.0);
        let plain_value = param.modulated_plain_value();
        let mut value: f32 = plain_value.into();

        // Apply color styling if provided
        if let Some(fill_color) = color {
            ui.style_mut().visuals.widgets.inactive.bg_fill = fill_color;
            ui.style_mut().visuals.widgets.hovered.bg_fill = fill_color;
            ui.style_mut().visuals.widgets.active.bg_fill = fill_color;
        }

        ui.style_mut().spacing.slider_width = 160.0;
        ui.style_mut().spacing.slider_rail_height = 14.0;

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

        ui.label(egui::RichText::new(label).size(15.0));
        ui.label(egui::RichText::new(formatter(value)).size(12.0).weak());
    });
}

fn render_int_vertical_slider(
    ui: &mut egui::Ui,
    _params: &Arc<DeviceParams>,
    setter: &ParamSetter,
    param: &nih_plug::prelude::IntParam,
    label: &str,
    labels: &[&str],
    color: Option<Color32>,
) {
    ui.vertical(|ui| {
        ui.set_width(48.0);
        let mut value = param.value();

        // Apply color styling if provided
        if let Some(fill_color) = color {
            ui.style_mut().visuals.widgets.inactive.bg_fill = fill_color;
            ui.style_mut().visuals.widgets.hovered.bg_fill = fill_color;
            ui.style_mut().visuals.widgets.active.bg_fill = fill_color;
        }

        ui.style_mut().spacing.slider_width = 160.0;
        ui.style_mut().spacing.slider_rail_height = 14.0;

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

        if ui.add(slider).changed() {
            setter.set_parameter(param, value);
        }

        ui.label(egui::RichText::new(label).size(15.0));
        let index = (value - min) as usize;
        let display_text = labels.get(index).unwrap_or(&"?");
        ui.label(egui::RichText::new(*display_text).size(12.0).weak());
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
            ui,
            params,
            setter,
            attack_param,
            "A",
            1.0,
            1000.0,
            SliderScale::Exponential(2.0),
            |v| format!("{:.0}", v),
            None,
        );
        ui.add_space(4.0);
        render_vertical_slider(
            ui,
            params,
            setter,
            decay_param,
            "D",
            1.0,
            1000.0,
            SliderScale::Exponential(2.0),
            |v| format!("{:.0}", v),
            None,
        );
        ui.add_space(4.0);
        render_vertical_slider(
            ui,
            params,
            setter,
            sustain_param,
            "S",
            0.0,
            1.0,
            SliderScale::Exponential(2.0),
            |v| format!("{:.2}", v),
            None,
        );
        ui.add_space(4.0);
        render_vertical_slider(
            ui,
            params,
            setter,
            release_param,
            "R",
            1.0,
            1000.0,
            SliderScale::Exponential(2.0),
            |v| format!("{:.0}", v),
            None,
        );
    });
}
