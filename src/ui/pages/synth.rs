use crate::params::DeviceParams;
use egui_taffy::taffy::{prelude::*, style::AlignItems};
use egui_taffy::TuiBuilderLogic;
use nih_plug::prelude::{Param, ParamSetter};
use nih_plug_egui::egui;
use nih_plug_egui::egui::Color32;
use std::sync::Arc;

pub fn render(tui: &mut egui_taffy::Tui, params: &Arc<DeviceParams>, setter: &ParamSetter) {
    tui.ui(|ui| {
        ui.add_space(12.0);
        ui.heading(egui::RichText::new("    Synth").size(14.0));
        ui.add_space(8.0);
    });

    tui.style(Style {
        flex_grow: 1.0,
        align_items: Some(AlignItems::Stretch),
        ..Default::default()
    })
    .ui(|ui| {
        ui.horizontal(|ui| {
            egui::Frame::default()
                .fill(ui.visuals().extreme_bg_color)
                .inner_margin(10.0)
                .stroke(egui::Stroke::new(1.0, ui.visuals().window_stroke.color))
                .corner_radius(15.0)
                .show(ui, |ui| {
                    ui.vertical(|ui| {
                        ui.label(
                            egui::RichText::new("   Phase Locked Loop OSC")
                                .size(10.0)
                                .strong(),
                        );
                        ui.add_space(8.0);
                        ui.horizontal(|ui| {
                            render_int_vertical_slider(
                                ui,
                                params,
                                setter,
                                &params.synth_pll_ref_octave,
                                "Oct",
                                &["-2", "-1", "0", "+1", "+2"],
                                Some(Color32::from_rgb(80, 80, 40)),
                            );
                            render_int_vertical_slider(
                                ui,
                                params,
                                setter,
                                &params.synth_pll_ref_tune,
                                "Tune",
                                &[
                                    "-12", "-11", "-10", "-9", "-8", "-7", "-6", "-5", "-4", "-3",
                                    "-2", "-1", "0", "+1", "+2", "+3", "+4", "+5", "+6", "+7",
                                    "+8", "+9", "+10", "+11", "+12",
                                ],
                                Some(Color32::from_rgb(80, 80, 40)),
                            );
                            render_vertical_slider(
                                ui,
                                params,
                                setter,
                                &params.synth_pll_ref_fine_tune,
                                "Fine",
                                -1.0,
                                1.0,
                                SliderScale::Linear,
                                |v| format!("{:.2}", v),
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
                                0.01,
                                1.0,
                                SliderScale::Linear,
                                |v| format!("{:.2}", v),
                                Some(Color32::from_rgb(40, 40, 80)),
                            );
                            render_vertical_slider(
                                ui,
                                params,
                                setter,
                                &params.synth_pll_damping,
                                "Dmp",
                                0.001,
                                0.3,
                                SliderScale::Linear,
                                |v| format!("{:.3}", v),
                                Some(Color32::from_rgb(40, 40, 80)),
                            );
                            render_vertical_slider(
                                ui,
                                params,
                                setter,
                                &params.synth_pll_stereo_damp_offset,
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
                                &params.synth_pll_range,
                                "Rng",
                                0.1,
                                100.0,
                                SliderScale::Logarithmic,
                                |v| {
                                    if v >= 10.0 {
                                        format!("{:.0}", v)
                                    } else if v >= 1.0 {
                                        format!("{:.1}", v)
                                    } else {
                                        format!("{:.2}", v)
                                    }
                                },
                                Some(Color32::from_rgb(40, 40, 80)),
                            );
                            render_int_vertical_slider(
                                ui,
                                params,
                                setter,
                                &params.synth_pll_mult,
                                "Mlt",
                                &["×1", "×2", "×4", "×8", "×16"],
                                Some(Color32::from_rgb(40, 40, 80)),
                            );
                            render_vertical_slider(
                                ui,
                                params,
                                setter,
                                &params.synth_pll_feedback,
                                "FB",
                                0.0,
                                0.1,
                                SliderScale::Linear,
                                |v| format!("{:.3}", v),
                                Some(Color32::from_rgb(40, 40, 80)),
                            );
                            render_vertical_slider(
                                ui,
                                params,
                                setter,
                                &params.synth_pll_ki_multiplier,
                                "Ki",
                                1.0,
                                100000.0,
                                SliderScale::Logarithmic,
                                |v| {
                                    if v >= 1000.0 {
                                        format!("{:.0}k", v / 1000.0)
                                    } else {
                                        format!("{:.0}", v)
                                    }
                                },
                                Some(Color32::from_rgb(40, 40, 80)),
                            );
                            render_vertical_slider(
                                ui,
                                params,
                                setter,
                                &params.synth_pll_distortion_amount,
                                "Dist",
                                0.0,
                                1.0,
                                SliderScale::Linear,
                                |v| format!("{:.2}", v),
                                Some(Color32::from_rgb(80, 40, 40)),
                            );
                            render_vertical_slider(
                                ui,
                                params,
                                setter,
                                &params.synth_pll_distortion_threshold,
                                "Thrs",
                                0.0,
                                1.0,
                                SliderScale::Linear,
                                |v| format!("{:.2}", v),
                                Some(Color32::from_rgb(80, 40, 40)),
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
                                    ui.label(egui::RichText::new("Color").size(8.0));
                                    let mut colored = params.synth_pll_colored.value();
                                    if ui.checkbox(&mut colored, "").changed() {
                                        setter.set_parameter(&params.synth_pll_colored, colored);
                                    }
                                });
                                ui.horizontal(|ui| {
                                    ui.label(egui::RichText::new("Edge").size(8.0));
                                    let mut edge_mode = params.synth_pll_mode.value();
                                    if ui.checkbox(&mut edge_mode, "").changed() {
                                        setter.set_parameter(&params.synth_pll_mode, edge_mode);
                                    }
                                });
                            });
                        });
                    });
                });

            egui::Frame::default()
                .fill(ui.visuals().extreme_bg_color)
                .inner_margin(egui::Margin { left: 10, right: 0, top: 10, bottom: 10 })
                .stroke(egui::Stroke::new(1.0, ui.visuals().window_stroke.color))
                .corner_radius(15.0)
                .show(ui, |ui| {
                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new("   Sub OSC").size(10.0).strong());
                        ui.add_space(8.0);
                        ui.horizontal(|ui| {
                            render_int_vertical_slider(
                                ui,
                                params,
                                setter,
                                &params.synth_sub_octave,
                                "Oct",
                                &[" -2", " -1", " 0", " +1", " +2"],
                                Some(Color32::from_rgb(80, 80, 40)),
                            );
                            render_vertical_slider(
                                ui,
                                params,
                                setter,
                                &params.synth_sub_shape,
                                "Shp",
                                0.0,
                                1.0,
                                SliderScale::Linear,
                                |v| {
                                    if v < 0.1 {
                                        "Sine".to_string()
                                    } else if v > 0.9 {
                                        "Sqr".to_string()
                                    } else {
                                        format!("{:.2}", v)
                                    }
                                },
                                Some(Color32::from_rgb(80, 80, 40)),
                            );
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
        });

        ui.add_space(10.0);

        ui.horizontal(|ui| {
            egui::Frame::default()
                .fill(ui.visuals().extreme_bg_color)
                .inner_margin(egui::Margin { left: 10, right: 0, top: 10, bottom: 10 })
                .stroke(egui::Stroke::new(1.0, ui.visuals().window_stroke.color))
                .corner_radius(15.0)
                .show(ui, |ui| {
                    ui.vertical(|ui| {
                        ui.label(
                            egui::RichText::new("   Vector Phase Shaping OSC")
                                .size(10.0)
                                .strong(),
                        );
                        ui.add_space(8.0);
                        ui.horizontal(|ui| {
                            render_int_vertical_slider(
                                ui,
                                params,
                                setter,
                                &params.synth_osc_octave,
                                "Oct",
                                &["-2", "-1", "0", "+1", "+2"],
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
                                &params.synth_distortion_amount,
                                "Amt",
                                0.0,
                                1.0,
                                SliderScale::Linear,
                                |v| format!("{:.2}", v),
                                Some(Color32::from_rgb(80, 40, 40)),
                            );
                            render_vertical_slider(
                                ui,
                                params,
                                setter,
                                &params.synth_distortion_threshold,
                                "Thr",
                                0.0,
                                1.0,
                                SliderScale::Linear,
                                |v| format!("{:.2}", v),
                                Some(Color32::from_rgb(80, 40, 40)),
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

            egui::Frame::default()
                .fill(ui.visuals().extreme_bg_color)
                .inner_margin(egui::Margin { left: 10, right: 0, top: 10, bottom: 10 })
                .stroke(egui::Stroke::new(1.0, ui.visuals().window_stroke.color))
                .corner_radius(15.0)
                .show(ui, |ui| {
                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new("   Saw / Pulse OSC").size(10.0).strong());
                        ui.add_space(8.0);
                        ui.horizontal(|ui| {
                            render_int_vertical_slider(
                                ui,
                                params,
                                setter,
                                &params.synth_polyblep_octave,
                                "Oct",
                                &["-2", "-1", "0", "+1", "+2"],
                                Some(Color32::from_rgb(80, 80, 40)),
                            );
                            render_vertical_slider(
                                ui,
                                params,
                                setter,
                                &params.synth_polyblep_pulse_width,
                                "S/P",
                                0.0,
                                1.0,
                                SliderScale::Linear,
                                |v| format!("{:.2}", v),
                                Some(Color32::from_rgb(80, 80, 40)),
                            );
                            render_vertical_slider(
                                ui,
                                params,
                                setter,
                                &params.synth_polyblep_stereo_width,
                                "StΔ",
                                0.0,
                                1.0,
                                SliderScale::Linear,
                                |v| format!("{:.2}", v),
                                Some(Color32::from_rgb(80, 40, 80)),
                            );
                            render_vertical_slider(
                                ui,
                                params,
                                setter,
                                &params.synth_polyblep_volume,
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

            ui.vertical(|ui| {
                ui.label(egui::RichText::new("FILT").size(10.0).strong());
                ui.add_space(2.0);
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
                        None,
                    );
                    ui.add_space(3.0);
                    render_vertical_slider(
                        ui,
                        params,
                        setter,
                        &params.synth_filter_resonance,
                        "Res",
                        0.0,
                        0.99,
                        SliderScale::Linear,
                        |v| format!("{:.2}", v),
                        None,
                    );
                    ui.add_space(3.0);
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
                        None,
                    );
                    ui.add_space(3.0);
                    render_vertical_slider(
                        ui,
                        params,
                        setter,
                        &params.synth_filter_drive,
                        "Drv",
                        1.0,
                        15.849,
                        SliderScale::Linear,
                        |v| format!("{:.1}", v),
                        None,
                    );
                    ui.add_space(3.0);
                    render_int_vertical_slider(
                        ui,
                        params,
                        setter,
                        &params.synth_filter_mode,
                        "Mod",
                        &["LP6", "LP12", "LP18", "LP24", "HP6", "HP12", "HP18", "HP24", "BP12", "BP24", "N12"],
                        None,
                    );
                });
            });

            ui.vertical(|ui| {
                ui.label(egui::RichText::new("VOL ENV").size(10.0).strong());
                ui.add_space(2.0);
                render_envelope_controls_compact(ui, params, setter, "vol");
            });

            ui.add_space(8.0);

            ui.vertical(|ui| {
                ui.label(egui::RichText::new("FILT ENV").size(10.0).strong());
                ui.add_space(2.0);
                render_envelope_controls_compact(ui, params, setter, "filt");
            });

            ui.add_space(8.0);

            ui.vertical(|ui| {
                ui.label(egui::RichText::new("VOL").size(10.0).strong());
                ui.add_space(2.0);
                render_vertical_slider(
                    ui,
                    params,
                    setter,
                    &params.synth_volume,
                    "Lvl",
                    0.0,
                    1.0,
                    SliderScale::Linear,
                    |v| format!("{:.0}%", v * 100.0),
                    Some(Color32::from_rgb(50, 180, 80)),
                );
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
    formatter: impl Fn(f32) -> String,
    color: Option<Color32>,
) where
    P::Plain: Into<f32>,
    f32: Into<P::Plain>,
{
    ui.vertical(|ui| {
        ui.set_width(28.0);
        let plain_value = param.modulated_plain_value();
        let mut value: f32 = plain_value.into();

        // Apply color styling if provided
        if let Some(fill_color) = color {
            ui.style_mut().visuals.widgets.inactive.bg_fill = fill_color;
            ui.style_mut().visuals.widgets.hovered.bg_fill = fill_color;
            ui.style_mut().visuals.widgets.active.bg_fill = fill_color;
        }

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

        ui.label(egui::RichText::new(label).size(9.0));
        ui.label(egui::RichText::new(formatter(value)).size(8.0).weak());
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
        ui.set_width(28.0);
        let mut value = param.value();

        // Apply color styling if provided
        if let Some(fill_color) = color {
            ui.style_mut().visuals.widgets.inactive.bg_fill = fill_color;
            ui.style_mut().visuals.widgets.hovered.bg_fill = fill_color;
            ui.style_mut().visuals.widgets.active.bg_fill = fill_color;
        }

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

        ui.label(egui::RichText::new(label).size(9.0));
        let index = (value - min) as usize;
        let display_text = labels.get(index).unwrap_or(&"?");
        ui.label(egui::RichText::new(*display_text).size(8.0).weak());
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
            0.0,
            1000.0,
            SliderScale::Exponential(2.0),
            |v| format!("{:.0}", v),
            None,
        );
        ui.add_space(2.0);
        render_vertical_slider(
            ui,
            params,
            setter,
            decay_param,
            "D",
            0.0,
            1000.0,
            SliderScale::Exponential(2.0),
            |v| format!("{:.0}", v),
            None,
        );
        ui.add_space(2.0);
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
        ui.add_space(2.0);
        render_vertical_slider(
            ui,
            params,
            setter,
            release_param,
            "R",
            0.0,
            1000.0,
            SliderScale::Exponential(2.0),
            |v| format!("{:.0}", v),
            None,
        );
    });
}
