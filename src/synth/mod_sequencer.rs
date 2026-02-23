use synfx_dsp::SlewValue;
use super::lfo::{LfoSyncDivision, ModDestination, ModulationValues};

pub struct ModSequencer {
    steps: [f64; 16],
    ties: u16,
    phase: f64,
    sample_rate: f64,
    division: LfoSyncDivision,
    slew_time_ms: f64,
    output_slew: SlewValue<f64>,
    current_step: usize,

    destinations: [ModDestination; 2],
    amounts: [f64; 2],
    amount_slews: [SlewValue<f64>; 2],
}

impl ModSequencer {
    pub fn new(sample_rate: f64) -> Self {
        let make_slew = || {
            let mut s = SlewValue::new();
            s.set_sample_rate(sample_rate);
            s
        };

        Self {
            steps: [0.0; 16],
            ties: 0,
            phase: 0.0,
            sample_rate,
            division: LfoSyncDivision::Eighth,
            slew_time_ms: 5.0,
            output_slew: make_slew(),
            current_step: 0,
            destinations: [ModDestination::None; 2],
            amounts: [0.0; 2],
            amount_slews: [make_slew(), make_slew()],
        }
    }

    pub fn set_sample_rate(&mut self, sample_rate: f64) {
        self.sample_rate = sample_rate;
        self.output_slew.set_sample_rate(sample_rate);
        for slew in &mut self.amount_slews {
            slew.set_sample_rate(sample_rate);
        }
    }

    pub fn set_step(&mut self, index: usize, value: f64) {
        if index < 16 {
            self.steps[index] = value.clamp(-1.0, 1.0);
        }
    }

    pub fn set_ties(&mut self, ties: u16) {
        self.ties = ties;
    }

    pub fn set_division(&mut self, division_idx: i32) {
        self.division = LfoSyncDivision::from_index(division_idx);
    }

    pub fn set_slew(&mut self, slew_ms: f64) {
        self.slew_time_ms = slew_ms.clamp(0.0, 200.0);
    }

    pub fn set_modulation(&mut self, slot: usize, destination: i32, amount: f64) {
        if slot < 2 {
            self.destinations[slot] = ModDestination::from_index(destination);
            self.amounts[slot] = amount;
        }
    }

    pub fn process(&mut self, bpm: f64) -> ModulationValues {
        let mut mod_values = ModulationValues::default();

        let beats_per_second = bpm / 60.0;
        let step_freq = beats_per_second / self.division.beats();

        self.phase += step_freq / self.sample_rate;
        if self.phase >= 16.0 {
            self.phase -= 16.0;
        }

        self.current_step = (self.phase as usize) % 16;
        let frac = self.phase - self.phase.floor();

        let current_val = self.steps[self.current_step];
        let next_step = (self.current_step + 1) % 16;
        let next_val = self.steps[next_step];

        let tied = (self.ties >> self.current_step) & 1 == 1;

        let raw_output = if tied {
            current_val + (next_val - current_val) * frac
        } else {
            current_val
        };

        let slew_ms = if tied { 0.5 } else { self.slew_time_ms.max(0.5) };
        let output = self.output_slew.next(raw_output, slew_ms);

        for slot in 0..2 {
            let dest = self.destinations[slot];
            let target_amount = self.amounts[slot];
            let slewed_amount = self.amount_slews[slot].next(target_amount, 30.0);
            mod_values.add_modulation(dest, slewed_amount, output);
        }

        mod_values
    }
}
