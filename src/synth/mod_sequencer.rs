use super::dsp::SlewValue;
use super::lfo::{LfoSyncDivision, ModDestination, ModulationValues};

pub struct ModSequencer {
    steps: [f64; 32],
    ties: u32,
    phase: f64,
    sample_rate: f64,
    division: LfoSyncDivision,
    slew_time_ms: f64,
    output_slew: SlewValue,
    current_step: usize,
    prev_step: usize,
    length: usize,
    bipolar: bool,
    retrigger: bool,
    playing: bool,

    destinations: [ModDestination; 4],
    amounts: [f64; 4],
    amount_slews: [SlewValue; 4],
}

impl ModSequencer {
    pub fn new(sample_rate: f64) -> Self {
        let make_slew = || {
            let mut s = SlewValue::new();
            s.set_sample_rate(sample_rate);
            s
        };

        Self {
            steps: [0.0; 32],
            ties: 0,
            phase: 0.0,
            sample_rate,
            division: LfoSyncDivision::Eighth,
            slew_time_ms: 5.0,
            output_slew: make_slew(),
            current_step: 0,
            prev_step: 0,
            length: 16,
            bipolar: true,
            retrigger: false,
            playing: false,
            destinations: [ModDestination::None; 4],
            amounts: [0.0; 4],
            amount_slews: [make_slew(), make_slew(), make_slew(), make_slew()],
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
        if index < 32 {
            self.steps[index] = value.clamp(-1.0, 1.0);
        }
    }

    pub fn set_ties(&mut self, ties: u32) {
        self.ties = ties;
    }

    pub fn set_division(&mut self, division_idx: i32) {
        self.division = LfoSyncDivision::from_index(division_idx);
    }

    pub fn set_slew(&mut self, slew_ms: f64) {
        self.slew_time_ms = slew_ms.clamp(0.0, 200.0);
    }

    pub fn set_length(&mut self, length: usize) {
        self.length = length.clamp(1, 32);
    }

    pub fn set_bipolar(&mut self, bipolar: bool) {
        self.bipolar = bipolar;
    }

    pub fn set_retrigger(&mut self, retrigger: bool) {
        self.retrigger = retrigger;
    }

    pub fn current_step(&self) -> usize {
        self.current_step
    }

    pub fn reset_phase(&mut self) {
        self.phase = 0.0;
    }

    pub fn should_retrigger(&self) -> bool {
        self.retrigger
    }

    pub fn set_playing(&mut self, playing: bool) {
        if !playing && self.playing {
            self.phase = 0.0;
            self.current_step = 0;
            self.prev_step = 0;
        }
        self.playing = playing;
    }

    pub fn set_modulation(&mut self, slot: usize, destination: i32, amount: f64) {
        if slot < 4 {
            self.destinations[slot] = ModDestination::from_index(destination);
            self.amounts[slot] = amount;
        }
    }

    pub fn process(&mut self, bpm: f64) -> ModulationValues {
        let mut mod_values = ModulationValues::default();

        if !self.playing {
            return mod_values;
        }

        let beats_per_second = bpm / 60.0;
        let step_freq = beats_per_second / self.division.beats();

        self.phase += step_freq / self.sample_rate;
        let len = self.length as f64;
        if self.phase >= len {
            self.phase -= len;
        }

        self.current_step = (self.phase as usize) % self.length;
        let frac = self.phase - self.phase.floor();

        let current_val = self.steps[self.current_step];
        let next_step = (self.current_step + 1) % self.length;
        let next_val = self.steps[next_step];

        let tied = (self.ties >> self.current_step) & 1 == 1;

        let raw_output = if tied {
            let t = frac * frac * (3.0 - 2.0 * frac);
            current_val + (next_val - current_val) * t
        } else {
            current_val
        };

        let slew_ms = if tied { 0.5 } else { self.slew_time_ms.max(0.5) };
        let output = self.output_slew.next(raw_output, slew_ms);

        let final_output = if self.bipolar {
            output
        } else {
            (output + 1.0) * 0.5
        };

        for slot in 0..4 {
            let dest = self.destinations[slot];
            let target_amount = self.amounts[slot];
            let slewed_amount = self.amount_slews[slot].next(target_amount, 30.0);
            mod_values.add_modulation(dest, slewed_amount, final_output);
        }

        self.prev_step = self.current_step;

        mod_values
    }
}
