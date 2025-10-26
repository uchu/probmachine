use std::sync::Arc;
use egui_taffy::TuiBuilderLogic;
use egui_taffy::taffy::{prelude::*, style::AlignItems};
use nih_plug_egui::egui::{self};
use crate::params::DeviceParams;
use nih_plug::prelude::{ParamSetter, Param};

pub fn render(
    tui: &mut egui_taffy::Tui,
    params: &Arc<DeviceParams>,
    setter: &ParamSetter,
) {
    tui.ui(|ui| {
        ui.add_space(12.0);
        ui.heading(egui::RichText::new("    Length").size(14.0));
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

            ui.label(egui::RichText::new("Note Length").strong());
            ui.separator();

            ui.label("Length (%):");
            let mut value = params.note_length_percent.modulated_plain_value();
            if ui.add(egui::Slider::new(&mut value, 0.0..=200.0)
                .custom_formatter(|v, _| format!("{:.0}%", v)))
                .changed() {
                setter.set_parameter(&params.note_length_percent, value);
            }
        });
    });
}
