use crate::params::DeviceParams;
use crate::ui::SharedUiState;
use egui_taffy::taffy::prelude::*;
use egui_taffy::TuiBuilderLogic;
use nih_plug::prelude::ParamSetter;
use nih_plug_egui::egui;
use nih_plug_egui::egui::Color32;
use std::sync::Arc;

const TAB_FONT: f32 = 18.0;
const TAB_BTN_WIDTH: f32 = 110.0;
const TAB_BTN_HEIGHT: f32 = 36.0;
const TAB_GAP: f32 = 6.0;

pub fn render(
    tui: &mut egui_taffy::Tui,
    params: &Arc<DeviceParams>,
    setter: &ParamSetter,
    ui_state: &Arc<SharedUiState>,
) {
    let current_tab = tui.ui(|ui| {
        ui.memory_mut(|mem| {
            *mem.data.get_temp_mut_or(egui::Id::new("sequence_tab"), 0u8)
        })
    });

    tui.ui(|ui| {
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            ui.add_space(16.0);
            render_tab_bar(ui, current_tab);
        });
        ui.add_space(2.0);
    });

    tui.style(Style {
        flex_grow: 1.0,
        ..Default::default()
    })
    .add(|tui| {
        match current_tab {
            0 => super::beat_probability::render(tui, params, setter, ui_state),
            1 => super::strength::render(tui, params, setter, ui_state),
            _ => super::notes::render(tui, params, setter, ui_state),
        }
    });
}

fn render_tab_bar(ui: &mut egui::Ui, current_tab: u8) {
    let tab_names = ["BEATS", "STRENGTH", "PITCH"];

    for (i, name) in tab_names.iter().enumerate() {
        if i > 0 {
            ui.add_space(TAB_GAP);
        }

        let size = egui::vec2(TAB_BTN_WIDTH, TAB_BTN_HEIGHT);
        let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click());
        let is_selected = current_tab == i as u8;
        let is_hovered = response.hovered();

        let bg_color = if is_selected {
            Color32::from_rgb(55, 55, 65)
        } else if is_hovered {
            Color32::from_rgb(45, 45, 55)
        } else {
            Color32::from_rgb(30, 30, 38)
        };
        ui.painter().rect_filled(rect, 4.0, bg_color);

        if is_selected {
            let accent_rect = egui::Rect::from_min_size(
                egui::pos2(rect.left(), rect.bottom() - 3.0),
                egui::vec2(TAB_BTN_WIDTH, 3.0),
            );
            ui.painter()
                .rect_filled(accent_rect, 1.5, Color32::from_rgb(100, 140, 200));
        }

        let text_color = if is_selected {
            Color32::WHITE
        } else {
            Color32::from_gray(150)
        };
        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            name,
            egui::FontId::proportional(TAB_FONT),
            text_color,
        );

        if response.clicked() {
            ui.memory_mut(|mem| {
                mem.data.insert_temp(egui::Id::new("sequence_tab"), i as u8);
            });
        }
    }
}
