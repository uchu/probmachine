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
    tui.style(Style {
        flex_grow: 1.0,
        align_items: Some(AlignItems::Stretch),
        ..Default::default()
    })
    .ui(|ui| {
        ui.spacing_mut().item_spacing.y = 8.0;

        ui.horizontal(|ui| {
            ui.add_space(16.0);
            render_length_section(ui, setter, params);
            ui.add_space(24.0);
            render_velocity_section(ui, setter, params);
        });

        ui.add_space(8.0);

        ui.horizontal(|ui| {
            ui.add_space(16.0);
            render_position_section(ui, setter, params);
        });
    });
}

fn render_length_section(
    ui: &mut egui::Ui,
    setter: &ParamSetter,
    params: &Arc<DeviceParams>,
) {
    egui::Frame::NONE

        .inner_margin(16.0)


        .show(ui, |ui| {
            ui.set_min_width(440.0);
            ui.vertical(|ui| {
                ui.label(egui::RichText::new("Length Modifiers").size(18.0).color(Color32::from_gray(180)));
                ui.add_space(12.0);

                render_strength_target_row(ui, setter, "1:",
                    &params.len_mod_1_target, &params.len_mod_1_amount, &params.len_mod_1_prob);

                ui.add_space(8.0);

                render_strength_target_row(ui, setter, "2:",
                    &params.len_mod_2_target, &params.len_mod_2_amount, &params.len_mod_2_prob);
            });
        });
}

fn render_strength_target_row(
    ui: &mut egui::Ui,
    setter: &ParamSetter,
    label: &str,
    target_param: &nih_plug::prelude::FloatParam,
    amount_param: &nih_plug::prelude::FloatParam,
    prob_param: &nih_plug::prelude::FloatParam,
) {
    let label_width = 24.0;
    let slider_width = 140.0;

    ui.horizontal(|ui| {
        ui.add_sized(egui::vec2(label_width, 20.0), egui::Label::new(egui::RichText::new(label).size(16.0)));

        let mut target = target_param.modulated_plain_value();
        ui.style_mut().spacing.slider_width = slider_width;
        ui.style_mut().spacing.slider_rail_height = 10.0;
        if ui.add(egui::Slider::new(&mut target, -100.0..=100.0)
            .custom_formatter(|v, _| format_strength_target(v as f32))
            .step_by(1.0))
            .changed() {
            setter.set_parameter(target_param, target);
        }

        ui.add_space(12.0);

        let mut amount = amount_param.modulated_plain_value();
        ui.style_mut().spacing.slider_width = 100.0;
        if ui.add(egui::Slider::new(&mut amount, 0.0..=200.0)
            .custom_formatter(|v, _| format!("{:.0}%", v))
            .step_by(1.0))
            .changed() {
            setter.set_parameter(amount_param, amount);
        }

        ui.add_space(12.0);

        let mut prob = prob_param.modulated_plain_value();
        ui.style_mut().spacing.slider_width = 60.0;
        if ui.add(egui::Slider::new(&mut prob, 0.0..=127.0)
            .custom_formatter(|v, _| format!("{:.0}", v))
            .step_by(1.0))
            .changed() {
            setter.set_parameter(prob_param, prob);
        }
    });

    ui.horizontal(|ui| {
        ui.add_space(label_width);
        ui.label(egui::RichText::new("Weak").size(11.0).color(Color32::from_gray(100)));
        ui.add_space(slider_width / 2.0 - 24.0);
        ui.label(egui::RichText::new("Any").size(11.0).color(Color32::from_gray(100)));
        ui.add_space(slider_width / 2.0 - 24.0);
        ui.label(egui::RichText::new("Strong").size(11.0).color(Color32::from_gray(100)));
        ui.add_space(32.0);
        ui.label(egui::RichText::new("Amt").size(11.0).color(Color32::from_gray(100)));
        ui.add_space(88.0);
        ui.label(egui::RichText::new("P").size(11.0).color(Color32::from_gray(100)));
    });
}

fn render_velocity_section(
    ui: &mut egui::Ui,
    setter: &ParamSetter,
    params: &Arc<DeviceParams>,
) {
    egui::Frame::NONE

        .inner_margin(16.0)


        .show(ui, |ui| {
            ui.set_min_width(440.0);
            ui.vertical(|ui| {
                ui.label(egui::RichText::new("Velocity Modifiers").size(18.0).color(Color32::from_gray(180)));
                ui.add_space(12.0);

                render_velocity_strength_row(ui, setter,
                    &params.vel_strength_target, &params.vel_strength_amount, &params.vel_strength_prob);

                ui.add_space(8.0);

                render_velocity_length_row(ui, setter,
                    &params.vel_length_target, &params.vel_length_amount, &params.vel_length_prob);
            });
        });
}

fn render_velocity_strength_row(
    ui: &mut egui::Ui,
    setter: &ParamSetter,
    target_param: &nih_plug::prelude::FloatParam,
    amount_param: &nih_plug::prelude::FloatParam,
    prob_param: &nih_plug::prelude::FloatParam,
) {
    let slider_width = 140.0;

    ui.horizontal(|ui| {
        let mut target = target_param.modulated_plain_value();
        ui.style_mut().spacing.slider_width = slider_width;
        ui.style_mut().spacing.slider_rail_height = 10.0;
        if ui.add(egui::Slider::new(&mut target, -100.0..=100.0)
            .custom_formatter(|v, _| format_strength_target(v as f32))
            .step_by(1.0))
            .changed() {
            setter.set_parameter(target_param, target);
        }

        ui.add_space(12.0);

        let mut amount = amount_param.modulated_plain_value();
        ui.style_mut().spacing.slider_width = 100.0;
        if ui.add(egui::Slider::new(&mut amount, -99.0..=27.0)
            .custom_formatter(|v, _| format_velocity_amount(v))
            .step_by(1.0))
            .changed() {
            setter.set_parameter(amount_param, amount);
        }

        ui.add_space(12.0);

        let mut prob = prob_param.modulated_plain_value();
        ui.style_mut().spacing.slider_width = 60.0;
        if ui.add(egui::Slider::new(&mut prob, 0.0..=127.0)
            .custom_formatter(|v, _| format!("{:.0}", v))
            .step_by(1.0))
            .changed() {
            setter.set_parameter(prob_param, prob);
        }
    });

    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("Weak").size(11.0).color(Color32::from_gray(100)));
        ui.add_space(slider_width / 2.0 - 24.0);
        ui.label(egui::RichText::new("Any").size(11.0).color(Color32::from_gray(100)));
        ui.add_space(slider_width / 2.0 - 24.0);
        ui.label(egui::RichText::new("Strong").size(11.0).color(Color32::from_gray(100)));
        ui.add_space(32.0);
        ui.label(egui::RichText::new("Vel").size(11.0).color(Color32::from_gray(100)));
        ui.add_space(88.0);
        ui.label(egui::RichText::new("P").size(11.0).color(Color32::from_gray(100)));
    });
}

fn render_velocity_length_row(
    ui: &mut egui::Ui,
    setter: &ParamSetter,
    target_param: &nih_plug::prelude::FloatParam,
    amount_param: &nih_plug::prelude::FloatParam,
    prob_param: &nih_plug::prelude::FloatParam,
) {
    let slider_width = 140.0;

    ui.horizontal(|ui| {
        let mut target = target_param.modulated_plain_value();
        ui.style_mut().spacing.slider_width = slider_width;
        ui.style_mut().spacing.slider_rail_height = 10.0;
        if ui.add(egui::Slider::new(&mut target, -100.0..=100.0)
            .custom_formatter(|v, _| format_length_target(v as f32))
            .step_by(1.0))
            .changed() {
            setter.set_parameter(target_param, target);
        }

        ui.add_space(12.0);

        let mut amount = amount_param.modulated_plain_value();
        ui.style_mut().spacing.slider_width = 100.0;
        if ui.add(egui::Slider::new(&mut amount, -99.0..=27.0)
            .custom_formatter(|v, _| format_velocity_amount(v))
            .step_by(1.0))
            .changed() {
            setter.set_parameter(amount_param, amount);
        }

        ui.add_space(12.0);

        let mut prob = prob_param.modulated_plain_value();
        ui.style_mut().spacing.slider_width = 60.0;
        if ui.add(egui::Slider::new(&mut prob, 0.0..=127.0)
            .custom_formatter(|v, _| format!("{:.0}", v))
            .step_by(1.0))
            .changed() {
            setter.set_parameter(prob_param, prob);
        }
    });

    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("Short").size(11.0).color(Color32::from_gray(100)));
        ui.add_space(slider_width / 2.0 - 24.0);
        ui.label(egui::RichText::new("Any").size(11.0).color(Color32::from_gray(100)));
        ui.add_space(slider_width / 2.0 - 22.0);
        ui.label(egui::RichText::new("Long").size(11.0).color(Color32::from_gray(100)));
        ui.add_space(32.0);
        ui.label(egui::RichText::new("Vel").size(11.0).color(Color32::from_gray(100)));
        ui.add_space(88.0);
        ui.label(egui::RichText::new("P").size(11.0).color(Color32::from_gray(100)));
    });
}

fn render_position_section(
    ui: &mut egui::Ui,
    setter: &ParamSetter,
    params: &Arc<DeviceParams>,
) {
    egui::Frame::NONE

        .inner_margin(16.0)


        .show(ui, |ui| {
            ui.set_min_width(440.0);
            ui.vertical(|ui| {
                ui.label(egui::RichText::new("Position Modifiers").size(18.0).color(Color32::from_gray(180)));
                ui.add_space(12.0);

                render_position_modifier_row(ui, setter, "1:",
                    &params.pos_mod_1_target, &params.pos_mod_1_shift, &params.pos_mod_1_prob);

                ui.add_space(8.0);

                render_position_modifier_row(ui, setter, "2:",
                    &params.pos_mod_2_target, &params.pos_mod_2_shift, &params.pos_mod_2_prob);
            });
        });
}

fn render_position_modifier_row(
    ui: &mut egui::Ui,
    setter: &ParamSetter,
    label: &str,
    target_param: &nih_plug::prelude::FloatParam,
    shift_param: &nih_plug::prelude::FloatParam,
    prob_param: &nih_plug::prelude::FloatParam,
) {
    let label_width = 24.0;
    let slider_width = 140.0;

    ui.horizontal(|ui| {
        ui.add_sized(egui::vec2(label_width, 20.0), egui::Label::new(egui::RichText::new(label).size(16.0)));

        let mut target = target_param.modulated_plain_value();
        ui.style_mut().spacing.slider_width = slider_width;
        ui.style_mut().spacing.slider_rail_height = 10.0;
        if ui.add(egui::Slider::new(&mut target, -100.0..=100.0)
            .custom_formatter(|v, _| format_strength_target(v as f32))
            .step_by(1.0))
            .changed() {
            setter.set_parameter(target_param, target);
        }

        ui.add_space(12.0);

        let mut shift = shift_param.modulated_plain_value();
        ui.style_mut().spacing.slider_width = 100.0;
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

        ui.add_space(12.0);

        let mut prob = prob_param.modulated_plain_value();
        ui.style_mut().spacing.slider_width = 60.0;
        if ui.add(egui::Slider::new(&mut prob, 0.0..=127.0)
            .custom_formatter(|v, _| format!("{:.0}", v))
            .step_by(1.0))
            .changed() {
            setter.set_parameter(prob_param, prob);
        }
    });

    ui.horizontal(|ui| {
        ui.add_space(label_width);
        ui.label(egui::RichText::new("Weak").size(11.0).color(Color32::from_gray(100)));
        ui.add_space(slider_width / 2.0 - 24.0);
        ui.label(egui::RichText::new("Any").size(11.0).color(Color32::from_gray(100)));
        ui.add_space(slider_width / 2.0 - 24.0);
        ui.label(egui::RichText::new("Strong").size(11.0).color(Color32::from_gray(100)));
        ui.add_space(32.0);
        ui.label(egui::RichText::new("Shift").size(11.0).color(Color32::from_gray(100)));
        ui.add_space(80.0);
        ui.label(egui::RichText::new("P").size(11.0).color(Color32::from_gray(100)));
    });
}

fn format_strength_target(v: f32) -> String {
    if v.abs() < 5.0 {
        "Any".to_string()
    } else if v < 0.0 {
        format!("W{:.0}", -v)
    } else {
        format!("S{:.0}", v)
    }
}

fn format_length_target(v: f32) -> String {
    if v.abs() < 5.0 {
        "Any".to_string()
    } else if v < 0.0 {
        format!("Sh{:.0}", -v)
    } else {
        format!("L{:.0}", v)
    }
}

fn format_velocity_amount(v: f64) -> String {
    if v > 0.5 { format!("+{:.0}", v) }
    else if v < -0.5 { format!("{:.0}", v) }
    else { "0".to_string() }
}
