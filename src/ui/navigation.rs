use nih_plug::prelude::{Param, ParamSetter};
use nih_plug_egui::egui::{self, Color32, Margin};
use super::page::Page;
use super::SharedUiState;
use crate::params::DeviceParams;
use std::sync::Arc;

pub fn render(ui: &mut egui::Ui, current_page: &mut Page, params: &Arc<DeviceParams>, setter: &ParamSetter, ui_state: &Arc<SharedUiState>) {
    egui::Frame::default()
        .outer_margin(Margin {
            left: -48,
            right: -48,
            top: -16,
            bottom: 0,
        })
        .inner_margin(Margin {
            left: 48,
            right: 24,
            top: 18,
            bottom: 10,
        })
        .fill(Color32::BLACK)
        .show(ui, |ui| {
            ui.set_min_width(1280.0);
            ui.set_max_width(1280.0);
            ui.horizontal(|ui| {
                for page in Page::all() {
                    let button = egui::Button::new(egui::RichText::new(page.label().to_uppercase()).size(20.0).color(Color32::WHITE))
                        .min_size(egui::vec2(96.0, 56.0))
                        .selected(*current_page == page);

                    if ui.add(button).clicked() {
                        *current_page = page;
                    }
                }

                ui.add_space(24.0);

                let playing = params.sequencer_enable.value();
                let play_label = if playing { "\u{23F9}" } else { "\u{25B6}" };
                let play_color = if playing {
                    Color32::from_rgb(80, 200, 80)
                } else {
                    Color32::from_rgb(160, 160, 160)
                };
                let play_btn = egui::Button::new(
                    egui::RichText::new(play_label).size(24.0).color(play_color)
                )
                .min_size(egui::vec2(56.0, 56.0));
                if ui.add(play_btn).clicked() {
                    setter.set_parameter(&params.sequencer_enable, !playing);
                }

                ui.add_space(16.0);

                let mut volume = params.global_volume.modulated_plain_value();
                ui.style_mut().spacing.slider_width = 220.0;
                ui.style_mut().spacing.slider_rail_height = 14.0;
                let slider = egui::Slider::new(&mut volume, 0.0..=1.0)
                    .show_value(false)
                    .trailing_fill(true);
                if ui.add(slider).changed() {
                    setter.set_parameter(&params.global_volume, volume);
                }

                ui.add_space(12.0);

                let output_level = ui_state.get_output_level();
                let box_size = egui::vec2(10.0, 26.0);
                let spacing = 3.0;
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
                    let x = rect.left() + i as f32 * (box_size.x + spacing);
                    let box_rect = egui::Rect::from_min_size(egui::pos2(x, rect.top()), box_size);
                    let color = if i < level { colors_on[i] } else { color_off };
                    ui.painter().rect_filled(box_rect, 2.0, color);
                }
            });
        });
}
