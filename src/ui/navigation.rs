use nih_plug::prelude::{Param, ParamSetter};
use nih_plug_egui::egui::{self, Color32, Margin};
use super::page::Page;
use super::SharedUiState;
use crate::params::DeviceParams;
use std::sync::Arc;

pub fn render(ui: &mut egui::Ui, current_page: &mut Page, params: &Arc<DeviceParams>, setter: &ParamSetter, ui_state: &Arc<SharedUiState>) {
    egui::Frame::default()
        .outer_margin(Margin {
            left: -30,
            right: 0,
            top: -10,
            bottom: 0,
        })
        .inner_margin(Margin {
            left: 30,
            right: 30,
            top: 10,
            bottom: 6,
        })
        .fill(Color32::BLACK)
        .show(ui, |ui| {
            ui.set_min_width(800.0);
            ui.set_max_width(800.0);
            ui.horizontal(|ui| {
                for page in Page::all() {
                    let button = egui::Button::new(egui::RichText::new(page.label()).size(14.0))
                        .min_size(egui::vec2(60.0, 32.0))
                        .selected(*current_page == page);

                    if ui.add(button).clicked() {
                        *current_page = page;
                    }
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let limiter_on = params.limiter_enable.value();
                    let btn_color = if limiter_on {
                        Color32::from_rgb(220, 80, 80)
                    } else {
                        Color32::from_rgb(80, 80, 80)
                    };
                    let btn = egui::Button::new(egui::RichText::new("L").size(12.0).color(btn_color))
                        .min_size(egui::vec2(20.0, 20.0));
                    if ui.add(btn).clicked() {
                        setter.set_parameter(&params.limiter_enable, !limiter_on);
                    }

                    ui.add_space(6.0);

                    let output_level = ui_state.get_output_level();
                    let box_size = egui::vec2(6.0, 12.0);
                    let spacing = 2.0;
                    let total_width = 5.0 * box_size.x + 4.0 * spacing;
                    let (rect, _) = ui.allocate_exact_size(egui::vec2(total_width, box_size.y), egui::Sense::hover());

                    let level = ((output_level * 5.0).ceil() as usize).min(5);
                    let colors_on = [
                        Color32::from_rgb(80, 200, 80),
                        Color32::from_rgb(80, 200, 80),
                        Color32::from_rgb(80, 200, 80),
                        Color32::from_rgb(220, 200, 60),
                        Color32::from_rgb(220, 80, 80),
                    ];
                    let color_off = Color32::from_rgb(40, 40, 40);

                    for i in 0..5 {
                        let x = rect.left() + (4 - i) as f32 * (box_size.x + spacing);
                        let box_rect = egui::Rect::from_min_size(egui::pos2(x, rect.top()), box_size);
                        let color = if (4 - i) < level { colors_on[4 - i] } else { color_off };
                        ui.painter().rect_filled(box_rect, 1.0, color);
                    }

                    ui.add_space(8.0);
                    let mut volume = params.global_volume.modulated_plain_value();
                    let slider = egui::Slider::new(&mut volume, 0.0..=1.0)
                        .show_value(false)
                        .trailing_fill(true);
                    ui.style_mut().spacing.slider_width = 80.0;
                    if ui.add(slider).changed() {
                        setter.set_parameter(&params.global_volume, volume);
                    }
                    ui.add_space(5.0);
                    ui.label(egui::RichText::new("Vol").size(10.0).color(Color32::GRAY));
                });
            });
        });
}
