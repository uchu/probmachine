use serde::{Deserialize, Serialize};

pub const MAX_BARS: usize = 8;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize, Default)]
pub enum BarOrderMode {
    #[default]
    Sequential,
    PingPong,
    Random,
    WeightedRandom,
}

impl BarOrderMode {
    pub const ALL: &'static [BarOrderMode] = &[
        BarOrderMode::Sequential,
        BarOrderMode::PingPong,
        BarOrderMode::Random,
        BarOrderMode::WeightedRandom,
    ];

    pub fn name(&self) -> &'static str {
        match self {
            BarOrderMode::Sequential => "Sequential",
            BarOrderMode::PingPong => "Ping-Pong",
            BarOrderMode::Random => "Random",
            BarOrderMode::WeightedRandom => "Weighted",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NoteSlotData {
    pub midi_note: u8,
    pub octave_offset: i8,
    pub chance: f32,
    pub strength_bias: f32,
    pub length_bias: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BarSlot {
    pub notes: Vec<NoteSlotData>,
    pub root_note: u8,
    pub strength_values: Vec<f32>,
    pub weight: u8,
    #[serde(default)]
    pub beat_values: Option<Vec<f32>>,
    #[serde(default)]
    pub swing: Option<f32>,
    #[serde(default)]
    pub melodic_fragment_index: Option<usize>,
}

impl Default for BarSlot {
    fn default() -> Self {
        let mut strength = vec![0.0f32; 96];
        strength[0] = 100.0 / 127.0;
        strength[24] = 75.0 / 127.0;
        strength[48] = 75.0 / 127.0;
        strength[72] = 75.0 / 127.0;
        Self {
            notes: Vec::new(),
            root_note: 48,
            strength_values: strength,
            weight: 64,
            beat_values: None,
            swing: None,
            melodic_fragment_index: None,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MultiBarConfig {
    pub enabled: bool,
    pub bar_count: u8,
    pub order_mode: BarOrderMode,
    pub bars: Vec<BarSlot>,
}

impl Default for MultiBarConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            bar_count: 4,
            order_mode: BarOrderMode::Sequential,
            bars: (0..MAX_BARS).map(|_| BarSlot::default()).collect(),
        }
    }
}

impl MultiBarConfig {
    pub fn next_bar_slot(&self, bar_counter: u64, rng: &mut impl rand::Rng) -> usize {
        let count = (self.bar_count as usize).clamp(1, MAX_BARS);
        match self.order_mode {
            BarOrderMode::Sequential => bar_counter as usize % count,
            BarOrderMode::PingPong => {
                if count <= 1 {
                    return 0;
                }
                let cycle = 2 * (count - 1);
                let pos = bar_counter as usize % cycle;
                if pos < count { pos } else { cycle - pos }
            }
            BarOrderMode::Random => rng.gen_range(0..count),
            BarOrderMode::WeightedRandom => {
                let total: u32 = self.bars.iter().take(count)
                    .map(|b| b.weight.max(1) as u32)
                    .sum();
                let roll = rng.gen_range(0..total);
                let mut cum = 0u32;
                for (i, bar) in self.bars.iter().take(count).enumerate() {
                    cum += bar.weight.max(1) as u32;
                    if roll < cum {
                        return i;
                    }
                }
                0
            }
        }
    }
}
