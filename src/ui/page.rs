use std::sync::Arc;
use nih_plug_egui::egui;
use egui_taffy::{Tui, TuiBuilderLogic};
use crate::params::DeviceParams;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Page {
    BeatProbability,
    Length,
    Notes,
    Strength,
    Synth,
    Page5,
    Page6,
}

impl Page {
    pub fn all() -> [Page; 7] {
        [
            Page::BeatProbability,
            Page::Length,
            Page::Notes,
            Page::Strength,
            Page::Synth,
            Page::Page5,
            Page::Page6,
        ]
    }

    pub fn label(&self) -> &'static str {
        match self {
            Page::BeatProbability => "Beats",
            Page::Length => "Length",
            Page::Notes => "Notes",
            Page::Strength => "Strength",
            Page::Synth => "Synth",
            Page::Page5 => "Page 5",
            Page::Page6 => "Page 6",
        }
    }

    pub fn render(
        &self,
        tui: &mut Tui,
        params: &Arc<DeviceParams>,
        setter: &nih_plug::prelude::ParamSetter,
    ) {
        match self {
            Page::BeatProbability => {
                super::pages::beat_probability::render(tui, params, setter)
            },
            Page::Length => {
                super::pages::length::render(tui, params, setter)
            },
            Page::Notes => {
                super::pages::notes::render(tui, params, setter)
            },
            Page::Strength => {
                super::pages::strength::render(tui, params, setter)
            }
            Page::Synth => {
                super::pages::synth::render(tui, params, setter)
            }
            _ => {
                tui.ui(|ui| {
                    ui.centered_and_justified(|ui| {
                        ui.label(egui::RichText::new("Coming soon...").size(24.0));
                    });
                });
            }
        }
    }
}
