use std::sync::Arc;
use egui_taffy::Tui;
use crate::params::DeviceParams;
use super::SharedUiState;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Page {
    BeatProbability,
    Length,
    Notes,
    Strength,
    Synth,
    Presets,
}

impl Page {
    pub fn all() -> [Page; 6] {
        [
            Page::BeatProbability,
            Page::Length,
            Page::Notes,
            Page::Strength,
            Page::Synth,
            Page::Presets,
        ]
    }

    pub fn label(&self) -> &'static str {
        match self {
            Page::BeatProbability => "Beats",
            Page::Length => "Length",
            Page::Notes => "Notes",
            Page::Strength => "Strength",
            Page::Synth => "Synth",
            Page::Presets => "Presets",
        }
    }

    pub fn render(
        &self,
        tui: &mut Tui,
        params: &Arc<DeviceParams>,
        setter: &nih_plug::prelude::ParamSetter,
        ui_state: &Arc<SharedUiState>,
    ) {
        match self {
            Page::BeatProbability => {
                super::pages::beat_probability::render(tui, params, setter)
            },
            Page::Length => {
                super::pages::length::render(tui, params, setter)
            },
            Page::Notes => {
                super::pages::notes::render(tui, params, setter, ui_state)
            },
            Page::Strength => {
                super::pages::strength::render(tui, params, setter, ui_state)
            }
            Page::Synth => {
                super::pages::synth::render(tui, params, setter)
            }
            Page::Presets => {
                super::pages::presets::render(tui, params, setter, ui_state)
            }
        }
    }
}
