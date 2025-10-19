use nih_plug::prelude::*;
use nih_plug_egui::EguiState;
use std::sync::Arc;

#[derive(Params)]
pub struct DeviceParams {
    #[persist = "editor-state"]
    pub editor_state: Arc<EguiState>,

    #[id = "beat1_prob"]
    pub beat1_probability: FloatParam,

    #[id = "beat2_prob"]
    pub beat2_probability: FloatParam,

    #[id = "beat3_prob"]
    pub beat3_probability: FloatParam,

    #[id = "beat4_prob"]
    pub beat4_probability: FloatParam,

    #[id = "beat5_prob"]
    pub beat5_probability: FloatParam,

    #[id = "beat6_prob"]
    pub beat6_probability: FloatParam,

    #[id = "beat7_prob"]
    pub beat7_probability: FloatParam,

    #[id = "beat8_prob"]
    pub beat8_probability: FloatParam,

    #[id = "beat9_prob"]
    pub beat9_probability: FloatParam,

    #[id = "beat10_prob"]
    pub beat10_probability: FloatParam,

    #[id = "beat11_prob"]
    pub beat11_probability: FloatParam,

    #[id = "beat12_prob"]
    pub beat12_probability: FloatParam,

    #[id = "beat13_prob"]
    pub beat13_probability: FloatParam,

    #[id = "beat14_prob"]
    pub beat14_probability: FloatParam,

    #[id = "beat15_prob"]
    pub beat15_probability: FloatParam,

    #[id = "beat16_prob"]
    pub beat16_probability: FloatParam,

    #[id = "beat17_prob"]
    pub beat17_probability: FloatParam,

    #[id = "beat18_prob"]
    pub beat18_probability: FloatParam,

    #[id = "beat19_prob"]
    pub beat19_probability: FloatParam,

    #[id = "beat20_prob"]
    pub beat20_probability: FloatParam,

    #[id = "beat21_prob"]
    pub beat21_probability: FloatParam,

    #[id = "beat22_prob"]
    pub beat22_probability: FloatParam,

    #[id = "beat23_prob"]
    pub beat23_probability: FloatParam,

    #[id = "beat24_prob"]
    pub beat24_probability: FloatParam,

    #[id = "beat25_prob"]
    pub beat25_probability: FloatParam,

    #[id = "beat26_prob"]
    pub beat26_probability: FloatParam,

    #[id = "beat27_prob"]
    pub beat27_probability: FloatParam,

    #[id = "beat28_prob"]
    pub beat28_probability: FloatParam,

    #[id = "beat29_prob"]
    pub beat29_probability: FloatParam,

    #[id = "beat30_prob"]
    pub beat30_probability: FloatParam,

    #[id = "beat31_prob"]
    pub beat31_probability: FloatParam,

    #[id = "beat32_prob"]
    pub beat32_probability: FloatParam,
}

impl DeviceParams {
    pub fn get_slider_param(&self, index: usize) -> &FloatParam {
        match index {
            0 => &self.beat1_probability,
            1 => &self.beat2_probability,
            2 => &self.beat3_probability,
            3 => &self.beat4_probability,
            4 => &self.beat5_probability,
            5 => &self.beat6_probability,
            6 => &self.beat7_probability,
            7 => &self.beat8_probability,
            8 => &self.beat9_probability,
            9 => &self.beat10_probability,
            10 => &self.beat11_probability,
            11 => &self.beat12_probability,
            12 => &self.beat13_probability,
            13 => &self.beat14_probability,
            14 => &self.beat15_probability,
            15 => &self.beat16_probability,
            16 => &self.beat17_probability,
            17 => &self.beat18_probability,
            18 => &self.beat19_probability,
            19 => &self.beat20_probability,
            20 => &self.beat21_probability,
            21 => &self.beat22_probability,
            22 => &self.beat23_probability,
            23 => &self.beat24_probability,
            24 => &self.beat25_probability,
            25 => &self.beat26_probability,
            26 => &self.beat27_probability,
            27 => &self.beat28_probability,
            28 => &self.beat29_probability,
            29 => &self.beat30_probability,
            30 => &self.beat31_probability,
            31 => &self.beat32_probability,
            _ => panic!("Invalid slider index: {}", index),
        }
    }
}

impl Default for DeviceParams {
    fn default() -> Self {
        Self {
            editor_state: EguiState::from_size(800, 480),

            beat1_probability: FloatParam::new(
                "Beat 1 probability",
                0.0,
                FloatRange::Linear { min: 0.0, max: 127.0 },
            )
            .with_smoother(SmoothingStyle::Linear(50.0)),

            beat2_probability: FloatParam::new(
                "Beat 2 probability",
                0.0,
                FloatRange::Linear { min: 0.0, max: 127.0 },
            )
            .with_smoother(SmoothingStyle::Linear(50.0)),

            beat3_probability: FloatParam::new(
                "Beat 3 probability",
                0.0,
                FloatRange::Linear { min: 0.0, max: 127.0 },
            )
            .with_smoother(SmoothingStyle::Linear(50.0)),

            beat4_probability: FloatParam::new(
                "Beat 4 probability",
                0.0,
                FloatRange::Linear { min: 0.0, max: 127.0 },
            )
            .with_smoother(SmoothingStyle::Linear(50.0)),

            beat5_probability: FloatParam::new(
                "Beat 5 probability",
                0.0,
                FloatRange::Linear { min: 0.0, max: 127.0 },
            )
            .with_smoother(SmoothingStyle::Linear(50.0)),

            beat6_probability: FloatParam::new(
                "Beat 6 probability",
                0.0,
                FloatRange::Linear { min: 0.0, max: 127.0 },
            )
            .with_smoother(SmoothingStyle::Linear(50.0)),

            beat7_probability: FloatParam::new(
                "Beat 7 probability",
                0.0,
                FloatRange::Linear { min: 0.0, max: 127.0 },
            )
            .with_smoother(SmoothingStyle::Linear(50.0)),

            beat8_probability: FloatParam::new(
                "Beat 8 probability",
                0.0,
                FloatRange::Linear { min: 0.0, max: 127.0 },
            )
            .with_smoother(SmoothingStyle::Linear(50.0)),

            beat9_probability: FloatParam::new(
                "Beat 9 probability",
                0.0,
                FloatRange::Linear { min: 0.0, max: 127.0 },
            )
            .with_smoother(SmoothingStyle::Linear(50.0)),

            beat10_probability: FloatParam::new(
                "Beat 10 probability",
                0.0,
                FloatRange::Linear { min: 0.0, max: 127.0 },
            )
            .with_smoother(SmoothingStyle::Linear(50.0)),

            beat11_probability: FloatParam::new(
                "Beat 11 probability",
                0.0,
                FloatRange::Linear { min: 0.0, max: 127.0 },
            )
            .with_smoother(SmoothingStyle::Linear(50.0)),

            beat12_probability: FloatParam::new(
                "Beat 12 probability",
                0.0,
                FloatRange::Linear { min: 0.0, max: 127.0 },
            )
            .with_smoother(SmoothingStyle::Linear(50.0)),

            beat13_probability: FloatParam::new(
                "Beat 13 probability",
                0.0,
                FloatRange::Linear { min: 0.0, max: 127.0 },
            )
            .with_smoother(SmoothingStyle::Linear(50.0)),

            beat14_probability: FloatParam::new(
                "Beat 14 probability",
                0.0,
                FloatRange::Linear { min: 0.0, max: 127.0 },
            )
            .with_smoother(SmoothingStyle::Linear(50.0)),

            beat15_probability: FloatParam::new(
                "Beat 15 probability",
                0.0,
                FloatRange::Linear { min: 0.0, max: 127.0 },
            )
            .with_smoother(SmoothingStyle::Linear(50.0)),

            beat16_probability: FloatParam::new(
                "Beat 16 Probability",
                0.0,
                FloatRange::Linear { min: 0.0, max: 127.0 },
            )
            .with_smoother(SmoothingStyle::Linear(50.0)),

            beat17_probability: FloatParam::new(
                "Beat 17 Probability",
                0.0,
                FloatRange::Linear { min: 0.0, max: 127.0 },
            )
            .with_smoother(SmoothingStyle::Linear(50.0)),

            beat18_probability: FloatParam::new(
                "Beat 18 Probability",
                0.0,
                FloatRange::Linear { min: 0.0, max: 127.0 },
            )
            .with_smoother(SmoothingStyle::Linear(50.0)),

            beat19_probability: FloatParam::new(
                "Beat 19 Probability",
                0.0,
                FloatRange::Linear { min: 0.0, max: 127.0 },
            )
            .with_smoother(SmoothingStyle::Linear(50.0)),

            beat20_probability: FloatParam::new(
                "Beat 20 Probability",
                0.0,
                FloatRange::Linear { min: 0.0, max: 127.0 },
            )
            .with_smoother(SmoothingStyle::Linear(50.0)),

            beat21_probability: FloatParam::new(
                "Beat 21 Probability",
                0.0,
                FloatRange::Linear { min: 0.0, max: 127.0 },
            )
            .with_smoother(SmoothingStyle::Linear(50.0)),

            beat22_probability: FloatParam::new(
                "Beat 22 Probability",
                0.0,
                FloatRange::Linear { min: 0.0, max: 127.0 },
            )
            .with_smoother(SmoothingStyle::Linear(50.0)),

            beat23_probability: FloatParam::new(
                "Beat 23 Probability",
                0.0,
                FloatRange::Linear { min: 0.0, max: 127.0 },
            )
            .with_smoother(SmoothingStyle::Linear(50.0)),

            beat24_probability: FloatParam::new(
                "Beat 24 Probability",
                0.0,
                FloatRange::Linear { min: 0.0, max: 127.0 },
            )
            .with_smoother(SmoothingStyle::Linear(50.0)),

            beat25_probability: FloatParam::new(
                "Beat 25 Probability",
                0.0,
                FloatRange::Linear { min: 0.0, max: 127.0 },
            )
            .with_smoother(SmoothingStyle::Linear(50.0)),

            beat26_probability: FloatParam::new(
                "Beat 26 Probability",
                0.0,
                FloatRange::Linear { min: 0.0, max: 127.0 },
            )
            .with_smoother(SmoothingStyle::Linear(50.0)),

            beat27_probability: FloatParam::new(
                "Beat 27 Probability",
                0.0,
                FloatRange::Linear { min: 0.0, max: 127.0 },
            )
            .with_smoother(SmoothingStyle::Linear(50.0)),

            beat28_probability: FloatParam::new(
                "Beat 28 Probability",
                0.0,
                FloatRange::Linear { min: 0.0, max: 127.0 },
            )
            .with_smoother(SmoothingStyle::Linear(50.0)),

            beat29_probability: FloatParam::new(
                "Beat 29 Probability",
                0.0,
                FloatRange::Linear { min: 0.0, max: 127.0 },
            )
            .with_smoother(SmoothingStyle::Linear(50.0)),

            beat30_probability: FloatParam::new(
                "Beat 30 Probability",
                0.0,
                FloatRange::Linear { min: 0.0, max: 127.0 },
            )
            .with_smoother(SmoothingStyle::Linear(50.0)),

            beat31_probability: FloatParam::new(
                "Beat 31 Probability",
                0.0,
                FloatRange::Linear { min: 0.0, max: 127.0 },
            )
            .with_smoother(SmoothingStyle::Linear(50.0)),

            beat32_probability: FloatParam::new(
                "Beat 32 Probability",
                0.0,
                FloatRange::Linear { min: 0.0, max: 127.0 },
            )
            .with_smoother(SmoothingStyle::Linear(50.0)),
        }
    }
}
