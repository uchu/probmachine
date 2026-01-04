#![allow(dead_code)]

use super::data::{Preset, PresetBank};
use super::defaults::create_default_presets;
use std::path::PathBuf;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Bank {
    A = 0,
    B = 1,
    C = 2,
    D = 3,
}

impl Bank {
    pub fn from_index(index: usize) -> Option<Self> {
        match index {
            0 => Some(Bank::A),
            1 => Some(Bank::B),
            2 => Some(Bank::C),
            3 => Some(Bank::D),
            _ => None,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Bank::A => "A",
            Bank::B => "B",
            Bank::C => "C",
            Bank::D => "D",
        }
    }

    pub fn all() -> [Bank; 4] {
        [Bank::A, Bank::B, Bank::C, Bank::D]
    }
}

pub struct PresetManager {
    banks: [PresetBank; 4],
    current_bank: Bank,
    current_preset: usize,
    modified: bool,
}

impl PresetManager {
    pub fn new() -> Self {
        let banks = create_default_presets();
        Self {
            banks,
            current_bank: Bank::A,
            current_preset: 0,
            modified: false,
        }
    }

    pub fn get_bank(&self, bank: Bank) -> &PresetBank {
        &self.banks[bank as usize]
    }

    pub fn get_bank_mut(&mut self, bank: Bank) -> &mut PresetBank {
        &mut self.banks[bank as usize]
    }

    pub fn current_bank(&self) -> Bank {
        self.current_bank
    }

    pub fn set_current_bank(&mut self, bank: Bank) {
        self.current_bank = bank;
    }

    pub fn current_preset_index(&self) -> usize {
        self.current_preset
    }

    pub fn set_current_preset(&mut self, index: usize) {
        if index < 16 {
            self.current_preset = index;
        }
    }

    pub fn get_current_preset(&self) -> &Preset {
        &self.banks[self.current_bank as usize].presets[self.current_preset]
    }

    pub fn get_current_preset_mut(&mut self) -> &mut Preset {
        self.modified = true;
        &mut self.banks[self.current_bank as usize].presets[self.current_preset]
    }

    pub fn get_preset(&self, bank: Bank, index: usize) -> Option<&Preset> {
        if index < 16 {
            Some(&self.banks[bank as usize].presets[index])
        } else {
            None
        }
    }

    pub fn get_preset_mut(&mut self, bank: Bank, index: usize) -> Option<&mut Preset> {
        if index < 16 {
            self.modified = true;
            Some(&mut self.banks[bank as usize].presets[index])
        } else {
            None
        }
    }

    pub fn rename_preset(&mut self, bank: Bank, index: usize, name: &str) {
        if index < 16 {
            self.banks[bank as usize].presets[index].name = name.to_string();
            self.modified = true;
        }
    }

    pub fn save_to_slot(&mut self, bank: Bank, index: usize, preset: Preset) {
        if index < 16 {
            self.banks[bank as usize].presets[index] = preset;
            self.modified = true;
        }
    }

    pub fn copy_preset(&mut self, from_bank: Bank, from_index: usize, to_bank: Bank, to_index: usize) {
        if from_index < 16 && to_index < 16 {
            let preset = self.banks[from_bank as usize].presets[from_index].clone();
            self.banks[to_bank as usize].presets[to_index] = preset;
            self.modified = true;
        }
    }

    pub fn is_modified(&self) -> bool {
        self.modified
    }

    pub fn mark_saved(&mut self) {
        self.modified = false;
    }

    pub fn get_preset_file_path() -> Option<PathBuf> {
        dirs::data_local_dir().map(|mut path| {
            path.push("Device");
            path.push("presets.json");
            path
        })
    }

    pub fn save_to_file(&mut self) -> Result<(), String> {
        let path = Self::get_preset_file_path()
            .ok_or_else(|| "Could not determine preset file path".to_string())?;

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create directory: {}", e))?;
        }

        let json = serde_json::to_string_pretty(&self.banks)
            .map_err(|e| format!("Failed to serialize presets: {}", e))?;

        std::fs::write(&path, json)
            .map_err(|e| format!("Failed to write presets file: {}", e))?;

        self.modified = false;
        Ok(())
    }

    pub fn load_from_file(&mut self) -> Result<(), String> {
        let path = Self::get_preset_file_path()
            .ok_or_else(|| "Could not determine preset file path".to_string())?;

        if !path.exists() {
            return Ok(());
        }

        let json = std::fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read presets file: {}", e))?;

        let banks: [PresetBank; 4] = serde_json::from_str(&json)
            .map_err(|e| format!("Failed to parse presets file: {}", e))?;

        self.banks = banks;
        self.modified = false;
        Ok(())
    }

    pub fn reset_to_defaults(&mut self) {
        self.banks = create_default_presets();
        self.modified = true;
    }
}

impl Default for PresetManager {
    fn default() -> Self {
        Self::new()
    }
}
