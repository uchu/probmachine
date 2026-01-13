#![allow(dead_code)]

use super::data::{Preset, PresetBank};
use super::defaults::create_default_presets;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FactoryBank {
    A = 0,
    B = 1,
    C = 2,
    D = 3,
    E = 4,
    F = 5,
    G = 6,
    H = 7,
}

impl FactoryBank {
    pub fn from_index(index: usize) -> Option<Self> {
        match index {
            0 => Some(FactoryBank::A),
            1 => Some(FactoryBank::B),
            2 => Some(FactoryBank::C),
            3 => Some(FactoryBank::D),
            4 => Some(FactoryBank::E),
            5 => Some(FactoryBank::F),
            6 => Some(FactoryBank::G),
            7 => Some(FactoryBank::H),
            _ => None,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            FactoryBank::A => "A",
            FactoryBank::B => "B",
            FactoryBank::C => "C",
            FactoryBank::D => "D",
            FactoryBank::E => "E",
            FactoryBank::F => "F",
            FactoryBank::G => "G",
            FactoryBank::H => "H",
        }
    }

    pub fn all() -> [FactoryBank; 8] {
        [FactoryBank::A, FactoryBank::B, FactoryBank::C, FactoryBank::D,
         FactoryBank::E, FactoryBank::F, FactoryBank::G, FactoryBank::H]
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum UserBank {
    U1 = 0,
    U2 = 1,
    U3 = 2,
    U4 = 3,
    U5 = 4,
    U6 = 5,
    U7 = 6,
    U8 = 7,
}

impl UserBank {
    pub fn from_index(index: usize) -> Option<Self> {
        match index {
            0 => Some(UserBank::U1),
            1 => Some(UserBank::U2),
            2 => Some(UserBank::U3),
            3 => Some(UserBank::U4),
            4 => Some(UserBank::U5),
            5 => Some(UserBank::U6),
            6 => Some(UserBank::U7),
            7 => Some(UserBank::U8),
            _ => None,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            UserBank::U1 => "A",
            UserBank::U2 => "B",
            UserBank::U3 => "C",
            UserBank::U4 => "D",
            UserBank::U5 => "E",
            UserBank::U6 => "F",
            UserBank::U7 => "G",
            UserBank::U8 => "H",
        }
    }

    pub fn all() -> [UserBank; 8] {
        [UserBank::U1, UserBank::U2, UserBank::U3, UserBank::U4,
         UserBank::U5, UserBank::U6, UserBank::U7, UserBank::U8]
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PresetLocation {
    Factory(FactoryBank, usize),
    User(UserBank, usize),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Bank {
    A = 0,
    B = 1,
    C = 2,
    D = 3,
    E = 4,
    F = 5,
    G = 6,
    H = 7,
}

impl Bank {
    pub fn from_index(index: usize) -> Option<Self> {
        match index {
            0 => Some(Bank::A),
            1 => Some(Bank::B),
            2 => Some(Bank::C),
            3 => Some(Bank::D),
            4 => Some(Bank::E),
            5 => Some(Bank::F),
            6 => Some(Bank::G),
            7 => Some(Bank::H),
            _ => None,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Bank::A => "A",
            Bank::B => "B",
            Bank::C => "C",
            Bank::D => "D",
            Bank::E => "E",
            Bank::F => "F",
            Bank::G => "G",
            Bank::H => "H",
        }
    }

    pub fn all() -> [Bank; 8] {
        [Bank::A, Bank::B, Bank::C, Bank::D, Bank::E, Bank::F, Bank::G, Bank::H]
    }
}

fn create_empty_user_banks() -> [PresetBank; 8] {
    [
        PresetBank::new("User A"),
        PresetBank::new("User B"),
        PresetBank::new("User C"),
        PresetBank::new("User D"),
        PresetBank::new("User E"),
        PresetBank::new("User F"),
        PresetBank::new("User G"),
        PresetBank::new("User H"),
    ]
}

#[derive(Serialize, Deserialize)]
struct UserPresetsFile {
    banks: [PresetBank; 8],
}

pub struct PresetManager {
    factory_banks: [PresetBank; 8],
    user_banks: [PresetBank; 8],
    current_location: PresetLocation,
    modified: bool,
}

impl PresetManager {
    pub fn new() -> Self {
        let factory_banks = create_default_presets();
        let user_banks = create_empty_user_banks();
        Self {
            factory_banks,
            user_banks,
            current_location: PresetLocation::Factory(FactoryBank::A, 0),
            modified: false,
        }
    }

    pub fn get_factory_bank(&self, bank: FactoryBank) -> &PresetBank {
        &self.factory_banks[bank as usize]
    }

    pub fn get_user_bank(&self, bank: UserBank) -> &PresetBank {
        &self.user_banks[bank as usize]
    }

    pub fn get_user_bank_mut(&mut self, bank: UserBank) -> &mut PresetBank {
        &mut self.user_banks[bank as usize]
    }

    pub fn current_location(&self) -> PresetLocation {
        self.current_location
    }

    pub fn set_current_location(&mut self, location: PresetLocation) {
        self.current_location = location;
    }

    pub fn get_current_preset(&self) -> &Preset {
        match self.current_location {
            PresetLocation::Factory(bank, index) => &self.factory_banks[bank as usize].presets[index],
            PresetLocation::User(bank, index) => &self.user_banks[bank as usize].presets[index],
        }
    }

    pub fn get_factory_preset(&self, bank: FactoryBank, index: usize) -> Option<&Preset> {
        if index < 32 {
            Some(&self.factory_banks[bank as usize].presets[index])
        } else {
            None
        }
    }

    pub fn get_user_preset(&self, bank: UserBank, index: usize) -> Option<&Preset> {
        if index < 32 {
            Some(&self.user_banks[bank as usize].presets[index])
        } else {
            None
        }
    }

    pub fn get_user_preset_mut(&mut self, bank: UserBank, index: usize) -> Option<&mut Preset> {
        if index < 32 {
            self.modified = true;
            Some(&mut self.user_banks[bank as usize].presets[index])
        } else {
            None
        }
    }

    pub fn save_to_user_slot(&mut self, bank: UserBank, index: usize, preset: Preset) {
        if index < 32 {
            self.user_banks[bank as usize].presets[index] = preset;
            self.modified = true;
        }
    }

    pub fn init_user_preset(&mut self, bank: UserBank, index: usize) {
        if index < 32 {
            self.user_banks[bank as usize].presets[index] = Preset::new(&format!("User {}", index + 1));
            self.modified = true;
        }
    }

    pub fn rename_user_preset(&mut self, bank: UserBank, index: usize, name: &str) {
        if index < 32 {
            self.user_banks[bank as usize].presets[index].name = name.to_string();
            self.modified = true;
        }
    }

    pub fn is_user_preset_empty(&self, bank: UserBank, index: usize) -> bool {
        if index < 32 {
            let preset = &self.user_banks[bank as usize].presets[index];
            preset.name.starts_with("User ") || preset.name == "Init"
        } else {
            true
        }
    }

    pub fn copy_factory_to_user(&mut self, from_bank: FactoryBank, from_index: usize, to_bank: UserBank, to_index: usize) {
        if from_index < 16 && to_index < 16 {
            let preset = self.factory_banks[from_bank as usize].presets[from_index].clone();
            self.user_banks[to_bank as usize].presets[to_index] = preset;
            self.modified = true;
        }
    }

    pub fn copy_user_to_user(&mut self, from_bank: UserBank, from_index: usize, to_bank: UserBank, to_index: usize) {
        if from_index < 16 && to_index < 16 {
            let preset = self.user_banks[from_bank as usize].presets[from_index].clone();
            self.user_banks[to_bank as usize].presets[to_index] = preset;
            self.modified = true;
        }
    }

    pub fn is_modified(&self) -> bool {
        self.modified
    }

    pub fn mark_saved(&mut self) {
        self.modified = false;
    }

    pub fn get_user_presets_file_path() -> Option<PathBuf> {
        dirs::data_local_dir().map(|mut path| {
            path.push("Device");
            path.push("user_presets.json");
            path
        })
    }

    pub fn save_user_presets(&mut self) -> Result<(), String> {
        let path = Self::get_user_presets_file_path()
            .ok_or_else(|| "Could not determine preset file path".to_string())?;

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create directory: {}", e))?;
        }

        let file_data = UserPresetsFile {
            banks: self.user_banks.clone(),
        };

        let json = serde_json::to_string_pretty(&file_data)
            .map_err(|e| format!("Failed to serialize presets: {}", e))?;

        std::fs::write(&path, json)
            .map_err(|e| format!("Failed to write presets file: {}", e))?;

        self.modified = false;
        Ok(())
    }

    pub fn load_user_presets(&mut self) -> Result<(), String> {
        let path = Self::get_user_presets_file_path()
            .ok_or_else(|| "Could not determine preset file path".to_string())?;

        if !path.exists() {
            return Ok(());
        }

        let json = std::fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read presets file: {}", e))?;

        let file_data: UserPresetsFile = serde_json::from_str(&json)
            .map_err(|e| format!("Failed to parse presets file: {}", e))?;

        self.user_banks = file_data.banks;
        self.modified = false;
        Ok(())
    }

    pub fn reset_user_presets(&mut self) {
        self.user_banks = create_empty_user_banks();
        self.modified = true;
    }

    pub fn export_preset(&self, location: PresetLocation, path: &std::path::Path) -> Result<(), String> {
        let preset = match location {
            PresetLocation::Factory(bank, index) => {
                if index >= 32 { return Err("Invalid preset index".to_string()); }
                &self.factory_banks[bank as usize].presets[index]
            }
            PresetLocation::User(bank, index) => {
                if index >= 32 { return Err("Invalid preset index".to_string()); }
                &self.user_banks[bank as usize].presets[index]
            }
        };

        let json = serde_json::to_string_pretty(preset)
            .map_err(|e| format!("Failed to serialize preset: {}", e))?;

        std::fs::write(path, json)
            .map_err(|e| format!("Failed to write preset file: {}", e))?;

        Ok(())
    }

    pub fn import_preset_to_user(&mut self, bank: UserBank, index: usize, path: &std::path::Path) -> Result<(), String> {
        if index >= 32 {
            return Err("Invalid preset index".to_string());
        }

        let json = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read preset file: {}", e))?;

        let preset: Preset = serde_json::from_str(&json)
            .map_err(|e| format!("Failed to parse preset file: {}", e))?;

        self.user_banks[bank as usize].presets[index] = preset;
        self.modified = true;
        Ok(())
    }

    pub fn current_bank(&self) -> Bank {
        match self.current_location {
            PresetLocation::Factory(b, _) => Bank::from_index(b as usize).unwrap_or(Bank::A),
            PresetLocation::User(b, _) => Bank::from_index(b as usize).unwrap_or(Bank::A),
        }
    }

    pub fn current_preset_index(&self) -> usize {
        match self.current_location {
            PresetLocation::Factory(_, idx) => idx,
            PresetLocation::User(_, idx) => idx,
        }
    }

    pub fn is_viewing_factory(&self) -> bool {
        matches!(self.current_location, PresetLocation::Factory(_, _))
    }

    pub fn get_bank(&self, bank: Bank) -> &PresetBank {
        &self.factory_banks[bank as usize]
    }

    pub fn get_preset(&self, bank: Bank, index: usize) -> Option<&Preset> {
        if index < 32 {
            Some(&self.factory_banks[bank as usize].presets[index])
        } else {
            None
        }
    }

    pub fn set_current_bank(&mut self, bank: Bank) {
        let idx = self.current_preset_index();
        self.current_location = PresetLocation::Factory(
            FactoryBank::from_index(bank as usize).unwrap_or(FactoryBank::A),
            idx
        );
    }

    pub fn set_current_preset(&mut self, index: usize) {
        if index < 32 {
            match self.current_location {
                PresetLocation::Factory(bank, _) => {
                    self.current_location = PresetLocation::Factory(bank, index);
                }
                PresetLocation::User(bank, _) => {
                    self.current_location = PresetLocation::User(bank, index);
                }
            }
        }
    }

    pub fn rename_preset(&mut self, _bank: Bank, _index: usize, _name: &str) {
    }

    pub fn save_to_slot(&mut self, _bank: Bank, _index: usize, _preset: Preset) {
    }

    pub fn reset_to_defaults(&mut self) {
        self.factory_banks = create_default_presets();
        self.user_banks = create_empty_user_banks();
        self.modified = true;
    }

    pub fn save_to_file(&mut self) -> Result<(), String> {
        self.save_user_presets()
    }

    pub fn load_from_file(&mut self) -> Result<(), String> {
        self.load_user_presets()
    }
}

impl Default for PresetManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn export_factory_presets_to_json() {
        let banks = create_default_presets();
        let project_root = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        let presets_dir = std::path::PathBuf::from(&project_root).join("assets/presets");

        std::fs::create_dir_all(&presets_dir).unwrap();

        let bank_names = ["factory_bank_a", "factory_bank_b", "factory_bank_c", "factory_bank_d",
                         "factory_bank_e", "factory_bank_f", "factory_bank_g", "factory_bank_h"];

        for (i, bank) in banks.iter().enumerate() {
            let path = presets_dir.join(format!("{}.json", bank_names[i]));
            let json = serde_json::to_string_pretty(bank).unwrap();
            std::fs::write(&path, json).unwrap();
            println!("Exported {} to {:?}", bank_names[i], path);
        }
    }
}
