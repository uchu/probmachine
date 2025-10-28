use crate::params::DeviceParams;
use egui_taffy::taffy::{prelude::*, style::AlignItems};
use egui_taffy::TuiBuilderLogic;
use nih_plug::prelude::{Param, ParamSetter};
use nih_plug_egui::egui;
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
            ui.add_space(4.0);

            egui::Frame::default()
                .fill(ui.visuals().extreme_bg_color)
                .inner_margin(10.0)
                .stroke(egui::Stroke::new(1.0, ui.visuals().window_stroke.color))
                .corner_radius(15.0)
                .show(ui, |ui| {
                    ui.vertical(|ui| {
                        ui.label(
                            egui::RichText::new("             Vector Phase Shaping OSC")
                                .size(10.0)
                                .strong(),
                        );
                        ui.add_space(8.0);
                        ui.horizontal(|ui| {
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
                            );
                        });
                    });
                });

            egui::Frame::default()
                .fill(ui.visuals().extreme_bg_color)
                .inner_margin(10.0)
                .stroke(egui::Stroke::new(1.0, ui.visuals().window_stroke.color))
                .corner_radius(15.0)
                .show(ui, |ui| {
                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new("    Saw / Pulse").size(10.0).strong());
                        ui.add_space(8.0);
                        ui.horizontal(|ui| {
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
                            );
                        });
                    });
                });

            egui::Frame::default()
                .fill(ui.visuals().extreme_bg_color)
                .inner_margin(10.0)
                .stroke(egui::Stroke::new(1.0, ui.visuals().window_stroke.color))
                .corner_radius(15.0)
                .show(ui, |ui| {
                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new(" Sub").size(10.0).strong());
                        ui.add_space(8.0);
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
                            );

                        });
                    });
                    ui.add_space(-30.0);
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
                    );
                });
            });
        });

        ui.horizontal(|ui| {
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
) where
    P::Plain: Into<f32>,
    f32: Into<P::Plain>,
{
    ui.vertical(|ui| {
        ui.set_width(28.0);
        let plain_value = param.modulated_plain_value();
        let mut value: f32 = plain_value.into();

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
        );
    });
}
