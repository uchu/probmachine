use std::sync::Arc;
use egui_taffy::TuiBuilderLogic;
use egui_taffy::taffy::{prelude::*, style::AlignItems};
use nih_plug_egui::egui::{self, Color32};
use crate::params::DeviceParams;
use nih_plug::prelude::{ParamSetter, Param};

pub fn render(
    tui: &mut egui_taffy::Tui,
    params: &Arc<DeviceParams>,
    setter: &ParamSetter,
) {
    tui.ui(|ui| {
        ui.add_space(12.0);
        ui.heading(egui::RichText::new("    Length & Position").size(22.0));
        ui.add_space(8.0);
    });

    tui.style(Style {
        flex_grow: 1.0,
        align_items: Some(AlignItems::Stretch),
        ..Default::default()
    })
    .ui(|ui| {
        ui.spacing_mut().item_spacing.y = 10.0;

        ui.label(egui::RichText::new("Length Modifiers").size(18.0).strong());
        ui.label(egui::RichText::new("Target: Weak ← Center (off) → Strong").weak().size(14.0));

        render_length_modifier(ui, setter, 1,
            &params.len_mod_1_target, &params.len_mod_1_amount, &params.len_mod_1_prob);
        render_length_modifier(ui, setter, 2,
            &params.len_mod_2_target, &params.len_mod_2_amount, &params.len_mod_2_prob);

        ui.add_space(12.0);
        ui.separator();
        ui.add_space(4.0);
        ui.label(egui::RichText::new("Decay Modifiers").size(18.0).strong());
        ui.label(egui::RichText::new("Modifies vol env decay time").weak().size(14.0));

        render_decay_modifier(ui, setter, 1,
            &params.decay_mod_1_target, &params.decay_mod_1_amount, &params.decay_mod_1_prob);
        render_decay_modifier(ui, setter, 2,
            &params.decay_mod_2_target, &params.decay_mod_2_amount, &params.decay_mod_2_prob);

        ui.add_space(12.0);
        ui.separator();
        ui.add_space(4.0);
        ui.label(egui::RichText::new("Position Modifiers").size(18.0).strong());
        ui.label(egui::RichText::new("Shift: - = early, + = late").weak().size(14.0));

        render_position_modifier(ui, setter, 1,
            &params.pos_mod_1_target, &params.pos_mod_1_shift, &params.pos_mod_1_prob);
        render_position_modifier(ui, setter, 2,
            &params.pos_mod_2_target, &params.pos_mod_2_shift, &params.pos_mod_2_prob);
    });
}

fn format_target(v: f32) -> String {
    if v.abs() < 5.0 {
        "Off".to_string()
    } else if v < 0.0 {
        format!("Weak {:.0}", -v)
    } else {
        format!("Strong {:.0}", v)
    }
}

fn render_length_modifier(
    ui: &mut egui::Ui,
    setter: &ParamSetter,
    slot: usize,
    target_param: &nih_plug::prelude::FloatParam,
    amount_param: &nih_plug::prelude::FloatParam,
    prob_param: &nih_plug::prelude::FloatParam,
) {
    let target_value = target_param.modulated_plain_value();
    let is_active = target_value.abs() >= 5.0;

    let frame_color = if is_active {
        if target_value < 0.0 {
            Color32::from_rgb(42, 38, 48)
        } else {
            Color32::from_rgb(48, 42, 38)
        }
    } else {
        Color32::from_rgb(35, 35, 35)
    };

    egui::Frame::default()
        .fill(frame_color)
        .inner_margin(12.0)
        .corner_radius(10.0)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(format!("{}:", slot)).size(16.0));
                ui.add_space(8.0);
                let mut target = target_value;
                ui.style_mut().spacing.slider_width = 200.0;
                if ui.add(egui::Slider::new(&mut target, -100.0..=100.0)
                    .custom_formatter(|v, _| format_target(v as f32))
                    .step_by(1.0))
                    .changed() {
                    setter.set_parameter(target_param, target);
                }

                ui.add_space(16.0);
                ui.separator();
                ui.add_space(16.0);

                let mut amount = amount_param.modulated_plain_value();
                ui.label(egui::RichText::new("Amt:").size(14.0));
                ui.style_mut().spacing.slider_width = 140.0;
                if ui.add(egui::Slider::new(&mut amount, 0.0..=200.0)
                    .custom_formatter(|v, _| format!("{:.0}%", v))
                    .step_by(1.0))
                    .changed() {
                    setter.set_parameter(amount_param, amount);
                }

                ui.add_space(16.0);

                let mut prob = prob_param.modulated_plain_value();
                ui.label(egui::RichText::new("P:").size(14.0));
                ui.style_mut().spacing.slider_width = 100.0;
                if ui.add(egui::Slider::new(&mut prob, 0.0..=127.0)
                    .custom_formatter(|v, _| format!("{:.0}", v))
                    .step_by(1.0))
                    .changed() {
                    setter.set_parameter(prob_param, prob);
                }
            });
        });
}

fn render_decay_modifier(
    ui: &mut egui::Ui,
    setter: &ParamSetter,
    slot: usize,
    target_param: &nih_plug::prelude::FloatParam,
    amount_param: &nih_plug::prelude::FloatParam,
    prob_param: &nih_plug::prelude::FloatParam,
) {
    let target_value = target_param.modulated_plain_value();
    let is_active = target_value.abs() >= 5.0;

    let frame_color = if is_active {
        if target_value < 0.0 {
            Color32::from_rgb(38, 42, 48)
        } else {
            Color32::from_rgb(48, 48, 38)
        }
    } else {
        Color32::from_rgb(35, 35, 35)
    };

    egui::Frame::default()
        .fill(frame_color)
        .inner_margin(12.0)
        .corner_radius(10.0)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(format!("{}:", slot)).size(16.0));
                ui.add_space(8.0);
                let mut target = target_value;
                ui.style_mut().spacing.slider_width = 200.0;
                if ui.add(egui::Slider::new(&mut target, -100.0..=100.0)
                    .custom_formatter(|v, _| format_target(v as f32))
                    .step_by(1.0))
                    .changed() {
                    setter.set_parameter(target_param, target);
                }

                ui.add_space(16.0);
                ui.separator();
                ui.add_space(16.0);

                let mut amount = amount_param.modulated_plain_value();
                ui.label(egui::RichText::new("Amt:").size(14.0));
                ui.style_mut().spacing.slider_width = 140.0;
                if ui.add(egui::Slider::new(&mut amount, 0.0..=200.0)
                    .custom_formatter(|v, _| format!("{:.0}%", v))
                    .step_by(1.0))
                    .changed() {
                    setter.set_parameter(amount_param, amount);
                }

                ui.add_space(16.0);

                let mut prob = prob_param.modulated_plain_value();
                ui.label(egui::RichText::new("P:").size(14.0));
                ui.style_mut().spacing.slider_width = 100.0;
                if ui.add(egui::Slider::new(&mut prob, 0.0..=127.0)
                    .custom_formatter(|v, _| format!("{:.0}", v))
                    .step_by(1.0))
                    .changed() {
                    setter.set_parameter(prob_param, prob);
                }
            });
        });
}

fn render_position_modifier(
    ui: &mut egui::Ui,
    setter: &ParamSetter,
    slot: usize,
    target_param: &nih_plug::prelude::FloatParam,
    shift_param: &nih_plug::prelude::FloatParam,
    prob_param: &nih_plug::prelude::FloatParam,
) {
    let target_value = target_param.modulated_plain_value();
    let is_active = target_value.abs() >= 5.0;

    let frame_color = if is_active {
        if target_value < 0.0 {
            Color32::from_rgb(42, 38, 48)
        } else {
            Color32::from_rgb(48, 42, 38)
        }
    } else {
        Color32::from_rgb(35, 35, 35)
    };

    egui::Frame::default()
        .fill(frame_color)
        .inner_margin(12.0)
        .corner_radius(10.0)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(format!("{}:", slot)).size(16.0));
                ui.add_space(8.0);
                let mut target = target_value;
                ui.style_mut().spacing.slider_width = 200.0;
                if ui.add(egui::Slider::new(&mut target, -100.0..=100.0)
                    .custom_formatter(|v, _| format_target(v as f32))
                    .step_by(1.0))
                    .changed() {
                    setter.set_parameter(target_param, target);
                }

                ui.add_space(16.0);
                ui.separator();
                ui.add_space(16.0);

                let mut shift = shift_param.modulated_plain_value();
                ui.label(egui::RichText::new("Shift:").size(14.0));
                ui.style_mut().spacing.slider_width = 140.0;
                if ui.add(egui::Slider::new(&mut shift, -50.0..=50.0)
                    .custom_formatter(|v, _| {
                        if v < -0.5 { format!("{:.0}%", v) }
                        else if v > 0.5 { format!("+{:.0}%", v) }
                        else { "0".to_string() }
                    })
                    .step_by(1.0))
                    .changed() {
                    setter.set_parameter(shift_param, shift);
                }

                ui.add_space(16.0);

                let mut prob = prob_param.modulated_plain_value();
                ui.label(egui::RichText::new("P:").size(14.0));
                ui.style_mut().spacing.slider_width = 100.0;
                if ui.add(egui::Slider::new(&mut prob, 0.0..=127.0)
                    .custom_formatter(|v, _| format!("{:.0}", v))
                    .step_by(1.0))
                    .changed() {
                    setter.set_parameter(prob_param, prob);
                }
            });
        });
}
