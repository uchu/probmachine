#![allow(clippy::too_many_arguments)]

mod data;
pub mod manager;
mod defaults;

pub use data::{Preset, PresetData, NotePresetData, OctaveRandomizationPresetData, StyleConfigPresetData, MultiBarPresetData, BarSlotPresetData, NoteSlotPresetData, MelodicConfigPresetData};
pub use manager::PresetManager;
