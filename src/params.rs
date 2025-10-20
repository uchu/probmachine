use nih_plug::prelude::*;
use nih_plug_egui::EguiState;
use std::sync::Arc;

#[derive(Params)]
pub struct DeviceParams {
    #[persist = "editor-state"]
    pub editor_state: Arc<EguiState>,

    #[id = "div1_beat1"]
    pub div1_beat1: FloatParam,

    #[id = "div2_beat1"]
    pub div2_beat1: FloatParam,
    #[id = "div2_beat2"]
    pub div2_beat2: FloatParam,

    #[id = "div4_beat1"]
    pub div4_beat1: FloatParam,
    #[id = "div4_beat2"]
    pub div4_beat2: FloatParam,
    #[id = "div4_beat3"]
    pub div4_beat3: FloatParam,
    #[id = "div4_beat4"]
    pub div4_beat4: FloatParam,

    #[id = "div8_beat1"]
    pub div8_beat1: FloatParam,
    #[id = "div8_beat2"]
    pub div8_beat2: FloatParam,
    #[id = "div8_beat3"]
    pub div8_beat3: FloatParam,
    #[id = "div8_beat4"]
    pub div8_beat4: FloatParam,
    #[id = "div8_beat5"]
    pub div8_beat5: FloatParam,
    #[id = "div8_beat6"]
    pub div8_beat6: FloatParam,
    #[id = "div8_beat7"]
    pub div8_beat7: FloatParam,
    #[id = "div8_beat8"]
    pub div8_beat8: FloatParam,

    #[id = "div16_beat1"]
    pub div16_beat1: FloatParam,
    #[id = "div16_beat2"]
    pub div16_beat2: FloatParam,
    #[id = "div16_beat3"]
    pub div16_beat3: FloatParam,
    #[id = "div16_beat4"]
    pub div16_beat4: FloatParam,
    #[id = "div16_beat5"]
    pub div16_beat5: FloatParam,
    #[id = "div16_beat6"]
    pub div16_beat6: FloatParam,
    #[id = "div16_beat7"]
    pub div16_beat7: FloatParam,
    #[id = "div16_beat8"]
    pub div16_beat8: FloatParam,
    #[id = "div16_beat9"]
    pub div16_beat9: FloatParam,
    #[id = "div16_beat10"]
    pub div16_beat10: FloatParam,
    #[id = "div16_beat11"]
    pub div16_beat11: FloatParam,
    #[id = "div16_beat12"]
    pub div16_beat12: FloatParam,
    #[id = "div16_beat13"]
    pub div16_beat13: FloatParam,
    #[id = "div16_beat14"]
    pub div16_beat14: FloatParam,
    #[id = "div16_beat15"]
    pub div16_beat15: FloatParam,
    #[id = "div16_beat16"]
    pub div16_beat16: FloatParam,

    #[id = "div32_beat1"]
    pub div32_beat1: FloatParam,
    #[id = "div32_beat2"]
    pub div32_beat2: FloatParam,
    #[id = "div32_beat3"]
    pub div32_beat3: FloatParam,
    #[id = "div32_beat4"]
    pub div32_beat4: FloatParam,
    #[id = "div32_beat5"]
    pub div32_beat5: FloatParam,
    #[id = "div32_beat6"]
    pub div32_beat6: FloatParam,
    #[id = "div32_beat7"]
    pub div32_beat7: FloatParam,
    #[id = "div32_beat8"]
    pub div32_beat8: FloatParam,
    #[id = "div32_beat9"]
    pub div32_beat9: FloatParam,
    #[id = "div32_beat10"]
    pub div32_beat10: FloatParam,
    #[id = "div32_beat11"]
    pub div32_beat11: FloatParam,
    #[id = "div32_beat12"]
    pub div32_beat12: FloatParam,
    #[id = "div32_beat13"]
    pub div32_beat13: FloatParam,
    #[id = "div32_beat14"]
    pub div32_beat14: FloatParam,
    #[id = "div32_beat15"]
    pub div32_beat15: FloatParam,
    #[id = "div32_beat16"]
    pub div32_beat16: FloatParam,
    #[id = "div32_beat17"]
    pub div32_beat17: FloatParam,
    #[id = "div32_beat18"]
    pub div32_beat18: FloatParam,
    #[id = "div32_beat19"]
    pub div32_beat19: FloatParam,
    #[id = "div32_beat20"]
    pub div32_beat20: FloatParam,
    #[id = "div32_beat21"]
    pub div32_beat21: FloatParam,
    #[id = "div32_beat22"]
    pub div32_beat22: FloatParam,
    #[id = "div32_beat23"]
    pub div32_beat23: FloatParam,
    #[id = "div32_beat24"]
    pub div32_beat24: FloatParam,
    #[id = "div32_beat25"]
    pub div32_beat25: FloatParam,
    #[id = "div32_beat26"]
    pub div32_beat26: FloatParam,
    #[id = "div32_beat27"]
    pub div32_beat27: FloatParam,
    #[id = "div32_beat28"]
    pub div32_beat28: FloatParam,
    #[id = "div32_beat29"]
    pub div32_beat29: FloatParam,
    #[id = "div32_beat30"]
    pub div32_beat30: FloatParam,
    #[id = "div32_beat31"]
    pub div32_beat31: FloatParam,
    #[id = "div32_beat32"]
    pub div32_beat32: FloatParam,
}

impl DeviceParams {
    pub fn get_division_param(&self, division: usize, beat_index: usize) -> &FloatParam {
        match division {
            1 => match beat_index {
                0 => &self.div1_beat1,
                _ => panic!("Invalid beat index {} for division 1/1", beat_index),
            },
            2 => match beat_index {
                0 => &self.div2_beat1,
                1 => &self.div2_beat2,
                _ => panic!("Invalid beat index {} for division 1/2", beat_index),
            },
            4 => match beat_index {
                0 => &self.div4_beat1,
                1 => &self.div4_beat2,
                2 => &self.div4_beat3,
                3 => &self.div4_beat4,
                _ => panic!("Invalid beat index {} for division 1/4", beat_index),
            },
            8 => match beat_index {
                0 => &self.div8_beat1,
                1 => &self.div8_beat2,
                2 => &self.div8_beat3,
                3 => &self.div8_beat4,
                4 => &self.div8_beat5,
                5 => &self.div8_beat6,
                6 => &self.div8_beat7,
                7 => &self.div8_beat8,
                _ => panic!("Invalid beat index {} for division 1/8", beat_index),
            },
            16 => match beat_index {
                0 => &self.div16_beat1,
                1 => &self.div16_beat2,
                2 => &self.div16_beat3,
                3 => &self.div16_beat4,
                4 => &self.div16_beat5,
                5 => &self.div16_beat6,
                6 => &self.div16_beat7,
                7 => &self.div16_beat8,
                8 => &self.div16_beat9,
                9 => &self.div16_beat10,
                10 => &self.div16_beat11,
                11 => &self.div16_beat12,
                12 => &self.div16_beat13,
                13 => &self.div16_beat14,
                14 => &self.div16_beat15,
                15 => &self.div16_beat16,
                _ => panic!("Invalid beat index {} for division 1/16", beat_index),
            },
            32 => match beat_index {
                0 => &self.div32_beat1,
                1 => &self.div32_beat2,
                2 => &self.div32_beat3,
                3 => &self.div32_beat4,
                4 => &self.div32_beat5,
                5 => &self.div32_beat6,
                6 => &self.div32_beat7,
                7 => &self.div32_beat8,
                8 => &self.div32_beat9,
                9 => &self.div32_beat10,
                10 => &self.div32_beat11,
                11 => &self.div32_beat12,
                12 => &self.div32_beat13,
                13 => &self.div32_beat14,
                14 => &self.div32_beat15,
                15 => &self.div32_beat16,
                16 => &self.div32_beat17,
                17 => &self.div32_beat18,
                18 => &self.div32_beat19,
                19 => &self.div32_beat20,
                20 => &self.div32_beat21,
                21 => &self.div32_beat22,
                22 => &self.div32_beat23,
                23 => &self.div32_beat24,
                24 => &self.div32_beat25,
                25 => &self.div32_beat26,
                26 => &self.div32_beat27,
                27 => &self.div32_beat28,
                28 => &self.div32_beat29,
                29 => &self.div32_beat30,
                30 => &self.div32_beat31,
                31 => &self.div32_beat32,
                _ => panic!("Invalid beat index {} for division 1/32", beat_index),
            },
            _ => panic!("Invalid division: {}", division),
        }
    }

    fn create_param(name: String, default: f32) -> FloatParam {
        FloatParam::new(name, default, FloatRange::Linear { min: 0.0, max: 127.0 })
            .with_smoother(SmoothingStyle::Linear(50.0))
    }
}

impl Default for DeviceParams {
    fn default() -> Self {
        Self {
            editor_state: EguiState::from_size(800, 480),

            div1_beat1: Self::create_param("1/1 Beat 1".to_string(), 0.0),

            div2_beat1: Self::create_param("1/2 Beat 1".to_string(), 0.0),
            div2_beat2: Self::create_param("1/2 Beat 2".to_string(), 0.0),

            div4_beat1: Self::create_param("1/4 Beat 1".to_string(), 0.0),
            div4_beat2: Self::create_param("1/4 Beat 2".to_string(), 0.0),
            div4_beat3: Self::create_param("1/4 Beat 3".to_string(), 0.0),
            div4_beat4: Self::create_param("1/4 Beat 4".to_string(), 0.0),

            div8_beat1: Self::create_param("1/8 Beat 1".to_string(), 0.0),
            div8_beat2: Self::create_param("1/8 Beat 2".to_string(), 0.0),
            div8_beat3: Self::create_param("1/8 Beat 3".to_string(), 0.0),
            div8_beat4: Self::create_param("1/8 Beat 4".to_string(), 0.0),
            div8_beat5: Self::create_param("1/8 Beat 5".to_string(), 0.0),
            div8_beat6: Self::create_param("1/8 Beat 6".to_string(), 0.0),
            div8_beat7: Self::create_param("1/8 Beat 7".to_string(), 0.0),
            div8_beat8: Self::create_param("1/8 Beat 8".to_string(), 0.0),

            div16_beat1: Self::create_param("1/16 Beat 1".to_string(), 0.0),
            div16_beat2: Self::create_param("1/16 Beat 2".to_string(), 0.0),
            div16_beat3: Self::create_param("1/16 Beat 3".to_string(), 0.0),
            div16_beat4: Self::create_param("1/16 Beat 4".to_string(), 0.0),
            div16_beat5: Self::create_param("1/16 Beat 5".to_string(), 0.0),
            div16_beat6: Self::create_param("1/16 Beat 6".to_string(), 0.0),
            div16_beat7: Self::create_param("1/16 Beat 7".to_string(), 0.0),
            div16_beat8: Self::create_param("1/16 Beat 8".to_string(), 0.0),
            div16_beat9: Self::create_param("1/16 Beat 9".to_string(), 0.0),
            div16_beat10: Self::create_param("1/16 Beat 10".to_string(), 0.0),
            div16_beat11: Self::create_param("1/16 Beat 11".to_string(), 0.0),
            div16_beat12: Self::create_param("1/16 Beat 12".to_string(), 0.0),
            div16_beat13: Self::create_param("1/16 Beat 13".to_string(), 0.0),
            div16_beat14: Self::create_param("1/16 Beat 14".to_string(), 0.0),
            div16_beat15: Self::create_param("1/16 Beat 15".to_string(), 0.0),
            div16_beat16: Self::create_param("1/16 Beat 16".to_string(), 0.0),

            div32_beat1: Self::create_param("1/32 Beat 1".to_string(), 0.0),
            div32_beat2: Self::create_param("1/32 Beat 2".to_string(), 0.0),
            div32_beat3: Self::create_param("1/32 Beat 3".to_string(), 0.0),
            div32_beat4: Self::create_param("1/32 Beat 4".to_string(), 0.0),
            div32_beat5: Self::create_param("1/32 Beat 5".to_string(), 0.0),
            div32_beat6: Self::create_param("1/32 Beat 6".to_string(), 0.0),
            div32_beat7: Self::create_param("1/32 Beat 7".to_string(), 0.0),
            div32_beat8: Self::create_param("1/32 Beat 8".to_string(), 0.0),
            div32_beat9: Self::create_param("1/32 Beat 9".to_string(), 0.0),
            div32_beat10: Self::create_param("1/32 Beat 10".to_string(), 0.0),
            div32_beat11: Self::create_param("1/32 Beat 11".to_string(), 0.0),
            div32_beat12: Self::create_param("1/32 Beat 12".to_string(), 0.0),
            div32_beat13: Self::create_param("1/32 Beat 13".to_string(), 0.0),
            div32_beat14: Self::create_param("1/32 Beat 14".to_string(), 0.0),
            div32_beat15: Self::create_param("1/32 Beat 15".to_string(), 0.0),
            div32_beat16: Self::create_param("1/32 Beat 16".to_string(), 0.0),
            div32_beat17: Self::create_param("1/32 Beat 17".to_string(), 0.0),
            div32_beat18: Self::create_param("1/32 Beat 18".to_string(), 0.0),
            div32_beat19: Self::create_param("1/32 Beat 19".to_string(), 0.0),
            div32_beat20: Self::create_param("1/32 Beat 20".to_string(), 0.0),
            div32_beat21: Self::create_param("1/32 Beat 21".to_string(), 0.0),
            div32_beat22: Self::create_param("1/32 Beat 22".to_string(), 0.0),
            div32_beat23: Self::create_param("1/32 Beat 23".to_string(), 0.0),
            div32_beat24: Self::create_param("1/32 Beat 24".to_string(), 0.0),
            div32_beat25: Self::create_param("1/32 Beat 25".to_string(), 0.0),
            div32_beat26: Self::create_param("1/32 Beat 26".to_string(), 0.0),
            div32_beat27: Self::create_param("1/32 Beat 27".to_string(), 0.0),
            div32_beat28: Self::create_param("1/32 Beat 28".to_string(), 0.0),
            div32_beat29: Self::create_param("1/32 Beat 29".to_string(), 0.0),
            div32_beat30: Self::create_param("1/32 Beat 30".to_string(), 0.0),
            div32_beat31: Self::create_param("1/32 Beat 31".to_string(), 0.0),
            div32_beat32: Self::create_param("1/32 Beat 32".to_string(), 0.0),
        }
    }
}
