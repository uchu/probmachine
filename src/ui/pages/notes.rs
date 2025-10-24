use std::sync::Arc;
use egui_taffy::TuiBuilderLogic;
use nih_plug_egui::egui::{self};
use crate::params::DeviceParams;

pub fn render(
    tui: &mut egui_taffy::Tui,
    _params: &Arc<DeviceParams>,
    _setter: &nih_plug::prelude::ParamSetter,
) {
    tui.ui(|ui| {
        ui.add_space(12.0);
        ui.heading(egui::RichText::new("    Notes").size(14.0));
        ui.add_space(8.0);
    });
}
