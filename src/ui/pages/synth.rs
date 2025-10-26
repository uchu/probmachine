use std::sync::Arc;
use nih_plug_egui::egui;
use egui_taffy::TuiBuilderLogic;
use egui_taffy::taffy::{prelude::*, style::AlignItems};
use crate::params::DeviceParams;
use nih_plug::prelude::{ParamSetter, Param};

pub fn render(
    tui: &mut egui_taffy::Tui,
    params: &Arc<DeviceParams>,
    setter: &ParamSetter,
) {
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
        egui::ScrollArea::vertical().auto_shrink([false; 2]).show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 8.0;

            ui.label(egui::RichText::new("Oscillator").strong());
            ui.separator();

            ui.label("D:");
            let mut value = params.synth_osc_d.modulated_plain_value();
            if ui.add(egui::Slider::new(&mut value, 0.0..=1.0)
                .custom_formatter(|v, _| format!("{:.2}", v)))
                .changed() {
                setter.set_parameter(&params.synth_osc_d, value);
            }

            ui.label("V:");
            let mut value = params.synth_osc_v.modulated_plain_value();
            if ui.add(egui::Slider::new(&mut value, 0.0..=1.0)
                .custom_formatter(|v, _| format!("{:.2}", v)))
                .changed() {
                setter.set_parameter(&params.synth_osc_v, value);
            }

            ui.add_space(15.0);
            ui.label(egui::RichText::new("Distortion").strong());
            ui.separator();

            ui.label("Amount:");
            let mut value = params.synth_distortion_amount.modulated_plain_value();
            if ui.add(egui::Slider::new(&mut value, 0.0..=1.0)
                .custom_formatter(|v, _| format!("{:.2}", v)))
                .changed() {
                setter.set_parameter(&params.synth_distortion_amount, value);
            }

            ui.label("Threshold:");
            let mut value = params.synth_distortion_threshold.modulated_plain_value();
            if ui.add(egui::Slider::new(&mut value, 0.0..=1.0)
                .custom_formatter(|v, _| format!("{:.2}", v)))
                .changed() {
                setter.set_parameter(&params.synth_distortion_threshold, value);
            }

            ui.add_space(15.0);
            ui.label(egui::RichText::new("Filter").strong());
            ui.separator();

            ui.label("Cutoff:");
            let mut value = params.synth_filter_cutoff.modulated_plain_value();
            if ui.add(egui::Slider::new(&mut value, 20.0..=20000.0)
                .logarithmic(true)
                .custom_formatter(|v, _| format!("{:.0} Hz", v)))
                .changed() {
                setter.set_parameter(&params.synth_filter_cutoff, value);
            }

            ui.label("Resonance:");
            let mut value = params.synth_filter_resonance.modulated_plain_value();
            if ui.add(egui::Slider::new(&mut value, 0.0..=0.99)
                .custom_formatter(|v, _| format!("{:.2}", v)))
                .changed() {
                setter.set_parameter(&params.synth_filter_resonance, value);
            }

            ui.label("Env Amount:");
            let mut value = params.synth_filter_env_amount.modulated_plain_value();
            if ui.add(egui::Slider::new(&mut value, -10000.0..=10000.0)
                .custom_formatter(|v, _| format!("{:.0}", v)))
                .changed() {
                setter.set_parameter(&params.synth_filter_env_amount, value);
            }

            ui.add_space(15.0);
            ui.label(egui::RichText::new("Volume Envelope").strong());
            ui.separator();
            render_envelope_controls(ui, params, setter, "vol");

            ui.add_space(15.0);
            ui.label(egui::RichText::new("Filter Envelope").strong());
            ui.separator();
            render_envelope_controls(ui, params, setter, "filt");

            ui.add_space(15.0);
            ui.label(egui::RichText::new("Volume").strong());
            ui.separator();

            ui.label("Level:");
            let mut value = params.synth_volume.modulated_plain_value();
            if ui.add(egui::Slider::new(&mut value, 0.0..=1.0)
                .custom_formatter(|v, _| format!("{:.0}%", v * 100.0)))
                .changed() {
                setter.set_parameter(&params.synth_volume, value);
            }

            ui.add_space(20.0);
        });
    });
}

fn render_envelope_controls(
    ui: &mut egui::Ui,
    params: &Arc<DeviceParams>,
    setter: &ParamSetter,
    prefix: &str,
) {
    let (attack_param, attack_shape_param, decay_param, decay_shape_param,
         sustain_param, release_param, release_shape_param) = match prefix {
        "vol" => (
            &params.synth_vol_attack,
            &params.synth_vol_attack_shape,
            &params.synth_vol_decay,
            &params.synth_vol_decay_shape,
            &params.synth_vol_sustain,
            &params.synth_vol_release,
            &params.synth_vol_release_shape,
        ),
        "filt" => (
            &params.synth_filt_attack,
            &params.synth_filt_attack_shape,
            &params.synth_filt_decay,
            &params.synth_filt_decay_shape,
            &params.synth_filt_sustain,
            &params.synth_filt_release,
            &params.synth_filt_release_shape,
        ),
        _ => panic!("Invalid prefix"),
    };

    ui.label("Attack:");
    let mut value = attack_param.modulated_plain_value();
    if ui.add(egui::Slider::new(&mut value, 0.0..=1000.0)
        .custom_formatter(|v, _| format!("{:.0} ms", v)))
        .changed() {
        setter.set_parameter(attack_param, value);
    }

    ui.label("Attack Shape:");
    let mut value = attack_shape_param.modulated_plain_value();
    if ui.add(egui::Slider::new(&mut value, 0.0..=1.0)
        .custom_formatter(|v, _| format!("{:.2}", v)))
        .changed() {
        setter.set_parameter(attack_shape_param, value);
    }

    ui.label("Decay:");
    let mut value = decay_param.modulated_plain_value();
    if ui.add(egui::Slider::new(&mut value, 0.0..=1000.0)
        .custom_formatter(|v, _| format!("{:.0} ms", v)))
        .changed() {
        setter.set_parameter(decay_param, value);
    }

    ui.label("Decay Shape:");
    let mut value = decay_shape_param.modulated_plain_value();
    if ui.add(egui::Slider::new(&mut value, 0.0..=1.0)
        .custom_formatter(|v, _| format!("{:.2}", v)))
        .changed() {
        setter.set_parameter(decay_shape_param, value);
    }

    ui.label("Sustain:");
    let mut value = sustain_param.modulated_plain_value();
    if ui.add(egui::Slider::new(&mut value, 0.0..=1.0)
        .custom_formatter(|v, _| format!("{:.2}", v)))
        .changed() {
        setter.set_parameter(sustain_param, value);
    }

    ui.label("Release:");
    let mut value = release_param.modulated_plain_value();
    if ui.add(egui::Slider::new(&mut value, 0.0..=1000.0)
        .custom_formatter(|v, _| format!("{:.0} ms", v)))
        .changed() {
        setter.set_parameter(release_param, value);
    }

    ui.label("Release Shape:");
    let mut value = release_shape_param.modulated_plain_value();
    if ui.add(egui::Slider::new(&mut value, 0.0..=1.0)
        .custom_formatter(|v, _| format!("{:.2}", v)))
        .changed() {
        setter.set_parameter(release_shape_param, value);
    }
}
