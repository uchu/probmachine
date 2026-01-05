use super::data::PresetBank;

const FACTORY_BANK_A: &str = include_str!("../../assets/presets/factory_bank_a.json");
const FACTORY_BANK_B: &str = include_str!("../../assets/presets/factory_bank_b.json");
const FACTORY_BANK_C: &str = include_str!("../../assets/presets/factory_bank_c.json");
const FACTORY_BANK_D: &str = include_str!("../../assets/presets/factory_bank_d.json");

pub fn create_default_presets() -> [PresetBank; 4] {
    [
        serde_json::from_str(FACTORY_BANK_A).expect("Failed to parse factory bank A"),
        serde_json::from_str(FACTORY_BANK_B).expect("Failed to parse factory bank B"),
        serde_json::from_str(FACTORY_BANK_C).expect("Failed to parse factory bank C"),
        serde_json::from_str(FACTORY_BANK_D).expect("Failed to parse factory bank D"),
    ]
}
