use nih_plug_egui::egui::{self, Color32, Margin};
use super::page::Page;

pub fn render(ui: &mut egui::Ui, current_page: &mut Page) {
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
            });
        });
}
