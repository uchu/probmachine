#![allow(dead_code)]

use synfx_dsp::SlewValue;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum LfoWaveform {
    Sine,
    Triangle,
    Saw,
    Square,
    SampleAndHold,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum LfoSyncDivision {
    // Straight divisions
    Whole,      // 1/1
    Half,       // 1/2
    Quarter,    // 1/4
    Eighth,     // 1/8
    Sixteenth,  // 1/16
    ThirtySecond, // 1/32
    // Dotted divisions (1.5x duration)
    HalfDotted,
    QuarterDotted,
    EighthDotted,
    SixteenthDotted,
    // Triplet divisions (2/3 duration)
    HalfTriplet,
    QuarterTriplet,
    EighthTriplet,
    SixteenthTriplet,
}

impl LfoSyncDivision {
    pub fn beats(&self) -> f64 {
        match self {
            LfoSyncDivision::Whole => 4.0,
            LfoSyncDivision::Half => 2.0,
            LfoSyncDivision::Quarter => 1.0,
            LfoSyncDivision::Eighth => 0.5,
            LfoSyncDivision::Sixteenth => 0.25,
            LfoSyncDivision::ThirtySecond => 0.125,
            // Dotted = 1.5x normal
            LfoSyncDivision::HalfDotted => 3.0,
            LfoSyncDivision::QuarterDotted => 1.5,
            LfoSyncDivision::EighthDotted => 0.75,
            LfoSyncDivision::SixteenthDotted => 0.375,
            // Triplet = 2/3 normal
            LfoSyncDivision::HalfTriplet => 4.0 / 3.0,
            LfoSyncDivision::QuarterTriplet => 2.0 / 3.0,
            LfoSyncDivision::EighthTriplet => 1.0 / 3.0,
            LfoSyncDivision::SixteenthTriplet => 1.0 / 6.0,
        }
    }

    pub fn from_index(idx: i32) -> Self {
        match idx {
            0 => LfoSyncDivision::Whole,
            1 => LfoSyncDivision::Half,
            2 => LfoSyncDivision::Quarter,
            3 => LfoSyncDivision::Eighth,
            4 => LfoSyncDivision::Sixteenth,
            5 => LfoSyncDivision::ThirtySecond,
            6 => LfoSyncDivision::HalfDotted,
            7 => LfoSyncDivision::QuarterDotted,
            8 => LfoSyncDivision::EighthDotted,
            9 => LfoSyncDivision::SixteenthDotted,
            10 => LfoSyncDivision::HalfTriplet,
            11 => LfoSyncDivision::QuarterTriplet,
            12 => LfoSyncDivision::EighthTriplet,
            13 => LfoSyncDivision::SixteenthTriplet,
            _ => LfoSyncDivision::Quarter,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            LfoSyncDivision::Whole => "1/1",
            LfoSyncDivision::Half => "1/2",
            LfoSyncDivision::Quarter => "1/4",
            LfoSyncDivision::Eighth => "1/8",
            LfoSyncDivision::Sixteenth => "1/16",
            LfoSyncDivision::ThirtySecond => "1/32",
            LfoSyncDivision::HalfDotted => "1/2.",
            LfoSyncDivision::QuarterDotted => "1/4.",
            LfoSyncDivision::EighthDotted => "1/8.",
            LfoSyncDivision::SixteenthDotted => "1/16.",
            LfoSyncDivision::HalfTriplet => "1/2T",
            LfoSyncDivision::QuarterTriplet => "1/4T",
            LfoSyncDivision::EighthTriplet => "1/8T",
            LfoSyncDivision::SixteenthTriplet => "1/16T",
        }
    }
}

impl LfoWaveform {
    pub fn from_index(idx: i32) -> Self {
        match idx {
            0 => LfoWaveform::Sine,
            1 => LfoWaveform::Triangle,
            2 => LfoWaveform::Saw,
            3 => LfoWaveform::Square,
            4 => LfoWaveform::SampleAndHold,
            _ => LfoWaveform::Sine,
        }
    }
}

pub struct Lfo {
    phase: f64,
    sample_rate: f64,

    // Parameters
    rate: f64,           // Hz when not synced
    waveform: LfoWaveform,
    tempo_sync: bool,
    sync_division: LfoSyncDivision,

    // Cross-modulation from another LFO
    sync_source: Option<usize>,  // Which LFO to sync from (0, 1, 2 or None)
    last_sync_value: f64,

    // Sample and hold state
    sh_value: f64,
    sh_noise_state: u32,

    // Slew for output smoothing
    output_slew: SlewValue<f64>,
    slew_time_ms: f64,

    // Phase modulation amount from sync source
    phase_mod_amount: f64,
}

impl Lfo {
    pub fn new(sample_rate: f64) -> Self {
        let mut output_slew = SlewValue::new();
        output_slew.set_sample_rate(sample_rate);

        Self {
            phase: 0.0,
            sample_rate,
            rate: 1.0,
            waveform: LfoWaveform::Sine,
            tempo_sync: false,
            sync_division: LfoSyncDivision::Quarter,
            sync_source: None,
            last_sync_value: 0.0,
            sh_value: 0.0,
            sh_noise_state: 12345,
            output_slew,
            slew_time_ms: 5.0,
            phase_mod_amount: 0.0,
        }
    }

    pub fn set_rate(&mut self, rate: f64) {
        self.rate = rate.clamp(0.01, 50.0);
    }

    pub fn set_waveform(&mut self, waveform: LfoWaveform) {
        self.waveform = waveform;
    }

    pub fn set_tempo_sync(&mut self, sync: bool) {
        self.tempo_sync = sync;
    }

    pub fn set_sync_division(&mut self, division: LfoSyncDivision) {
        self.sync_division = division;
    }

    pub fn set_sync_source(&mut self, source: Option<usize>) {
        self.sync_source = source;
    }

    pub fn set_phase_mod_amount(&mut self, amount: f64) {
        self.phase_mod_amount = amount.clamp(0.0, 1.0);
    }

    pub fn set_slew_time(&mut self, ms: f64) {
        self.slew_time_ms = ms.clamp(0.5, 100.0);
    }

    pub fn set_sample_rate(&mut self, sample_rate: f64) {
        self.sample_rate = sample_rate;
        self.output_slew.set_sample_rate(sample_rate);
    }

    pub fn get_phase(&self) -> f64 {
        self.phase
    }

    pub fn reset_phase(&mut self) {
        self.phase = 0.0;
    }

    // Trigger sync from external source (another LFO rising edge)
    pub fn trigger_sync(&mut self) {
        self.phase = 0.0;
    }

    fn next_noise(&mut self) -> f64 {
        // Simple xorshift PRNG
        self.sh_noise_state ^= self.sh_noise_state << 13;
        self.sh_noise_state ^= self.sh_noise_state >> 17;
        self.sh_noise_state ^= self.sh_noise_state << 5;
        (self.sh_noise_state as f64 / u32::MAX as f64) * 2.0 - 1.0
    }

    fn generate_waveform(&mut self, phase: f64) -> f64 {
        match self.waveform {
            LfoWaveform::Sine => {
                (phase * std::f64::consts::TAU).sin()
            }
            LfoWaveform::Triangle => {
                if phase < 0.25 {
                    phase * 4.0
                } else if phase < 0.75 {
                    1.0 - (phase - 0.25) * 4.0
                } else {
                    (phase - 0.75) * 4.0 - 1.0
                }
            }
            LfoWaveform::Saw => {
                2.0 * phase - 1.0
            }
            LfoWaveform::Square => {
                if phase < 0.5 { 1.0 } else { -1.0 }
            }
            LfoWaveform::SampleAndHold => {
                self.sh_value
            }
        }
    }

    pub fn process(&mut self, bpm: f64, sync_input: Option<f64>) -> f64 {
        // Calculate frequency
        let freq = if self.tempo_sync {
            let beats_per_second = bpm / 60.0;
            beats_per_second / self.sync_division.beats()
        } else {
            self.rate
        };

        // Handle cross-modulation sync
        if let Some(sync_val) = sync_input {
            if self.phase_mod_amount > 0.001 {
                // Detect rising edge for hard sync
                if sync_val > 0.0 && self.last_sync_value <= 0.0 {
                    // Soft sync - blend towards reset based on amount
                    self.phase *= 1.0 - self.phase_mod_amount;
                }
                self.last_sync_value = sync_val;
            }
        }

        // Check for phase wrap (for S&H trigger)
        let old_phase = self.phase;

        // Advance phase
        self.phase += freq / self.sample_rate;

        // Wrap phase and trigger S&H
        if self.phase >= 1.0 {
            self.phase -= 1.0;
            if self.waveform == LfoWaveform::SampleAndHold {
                self.sh_value = self.next_noise();
            }
        }

        // Trigger S&H on first sample if needed
        if old_phase == 0.0 && self.waveform == LfoWaveform::SampleAndHold && self.sh_value == 0.0 {
            self.sh_value = self.next_noise();
        }

        // Generate output
        let raw_output = self.generate_waveform(self.phase);

        // Apply slew for smooth output (critical for avoiding crackles)
        self.output_slew.next(raw_output, self.slew_time_ms)
    }
}

// Modulation destination enum
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ModDestination {
    None,
    // PLL parameters
    PllDamping,
    PllInfluence,
    PllTrackSpeed,
    PllFeedback,
    PllFmAmount,
    PllPulseWidth,
    PllStereoPhase,
    PllCrossFeedback,
    PllFmEnvAmount,
    PllBurstAmount,
    PllRange,
    // VPS parameters
    VpsD,
    VpsV,
    // Filter parameters
    FilterCutoff,
    FilterResonance,
    FilterDrive,
    // Coloration
    RingMod,
    Wavefold,
    DriftAmount,
    NoiseAmount,
    TubeDrive,
    // Reverb
    ReverbMix,
    ReverbDecay,
    // Volume/Mix
    PllVolume,
    VpsVolume,
    SubVolume,
}

impl ModDestination {
    pub fn from_index(idx: i32) -> Self {
        match idx {
            0 => ModDestination::None,
            1 => ModDestination::PllDamping,
            2 => ModDestination::PllInfluence,
            3 => ModDestination::PllTrackSpeed,
            4 => ModDestination::PllFeedback,
            5 => ModDestination::PllFmAmount,
            6 => ModDestination::PllPulseWidth,
            7 => ModDestination::PllStereoPhase,
            8 => ModDestination::PllCrossFeedback,
            9 => ModDestination::PllFmEnvAmount,
            10 => ModDestination::PllBurstAmount,
            11 => ModDestination::PllRange,
            12 => ModDestination::VpsD,
            13 => ModDestination::VpsV,
            14 => ModDestination::FilterCutoff,
            15 => ModDestination::FilterResonance,
            16 => ModDestination::FilterDrive,
            17 => ModDestination::RingMod,
            18 => ModDestination::Wavefold,
            19 => ModDestination::DriftAmount,
            20 => ModDestination::NoiseAmount,
            21 => ModDestination::TubeDrive,
            22 => ModDestination::ReverbMix,
            23 => ModDestination::ReverbDecay,
            24 => ModDestination::PllVolume,
            25 => ModDestination::VpsVolume,
            26 => ModDestination::SubVolume,
            _ => ModDestination::None,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            ModDestination::None => "None",
            ModDestination::PllDamping => "PLL Damp",
            ModDestination::PllInfluence => "PLL Infl",
            ModDestination::PllTrackSpeed => "PLL Track",
            ModDestination::PllFeedback => "PLL FB",
            ModDestination::PllFmAmount => "PLL FM",
            ModDestination::PllPulseWidth => "PLL PW",
            ModDestination::PllStereoPhase => "PLL StPh",
            ModDestination::PllCrossFeedback => "PLL XFB",
            ModDestination::PllFmEnvAmount => "PLL FMEnv",
            ModDestination::PllBurstAmount => "PLL OT",
            ModDestination::PllRange => "PLL Rng",
            ModDestination::VpsD => "VPS D",
            ModDestination::VpsV => "VPS V",
            ModDestination::FilterCutoff => "Filt Cut",
            ModDestination::FilterResonance => "Filt Res",
            ModDestination::FilterDrive => "Filt Drv",
            ModDestination::RingMod => "Ring Mod",
            ModDestination::Wavefold => "Wavefold",
            ModDestination::DriftAmount => "Drift",
            ModDestination::NoiseAmount => "Noise",
            ModDestination::TubeDrive => "Tube",
            ModDestination::ReverbMix => "Rev Mix",
            ModDestination::ReverbDecay => "Rev Decay",
            ModDestination::PllVolume => "PLL Vol",
            ModDestination::VpsVolume => "VPS Vol",
            ModDestination::SubVolume => "Sub Vol",
        }
    }
}

// Modulation matrix - holds all modulation values for a frame
#[derive(Clone, Default)]
pub struct ModulationValues {
    pub pll_damping: f64,
    pub pll_influence: f64,
    pub pll_track_speed: f64,
    pub pll_feedback: f64,
    pub pll_fm_amount: f64,
    pub pll_pulse_width: f64,
    pub pll_stereo_phase: f64,
    pub pll_cross_feedback: f64,
    pub pll_fm_env_amount: f64,
    pub pll_burst_amount: f64,
    pub pll_range: f64,
    pub vps_d: f64,
    pub vps_v: f64,
    pub filter_cutoff: f64,
    pub filter_resonance: f64,
    pub filter_drive: f64,
    pub ring_mod: f64,
    pub wavefold: f64,
    pub drift_amount: f64,
    pub noise_amount: f64,
    pub tube_drive: f64,
    pub reverb_mix: f64,
    pub reverb_decay: f64,
    pub pll_volume: f64,
    pub vps_volume: f64,
    pub sub_volume: f64,
}

impl ModulationValues {
    pub fn add_modulation(&mut self, dest: ModDestination, amount: f64, lfo_value: f64) {
        let mod_value = lfo_value * amount;
        match dest {
            ModDestination::None => {}
            ModDestination::PllDamping => self.pll_damping += mod_value,
            ModDestination::PllInfluence => self.pll_influence += mod_value,
            ModDestination::PllTrackSpeed => self.pll_track_speed += mod_value,
            ModDestination::PllFeedback => self.pll_feedback += mod_value,
            ModDestination::PllFmAmount => self.pll_fm_amount += mod_value,
            ModDestination::PllPulseWidth => self.pll_pulse_width += mod_value,
            ModDestination::PllStereoPhase => self.pll_stereo_phase += mod_value,
            ModDestination::PllCrossFeedback => self.pll_cross_feedback += mod_value,
            ModDestination::PllFmEnvAmount => self.pll_fm_env_amount += mod_value,
            ModDestination::PllBurstAmount => self.pll_burst_amount += mod_value,
            ModDestination::PllRange => self.pll_range += mod_value,
            ModDestination::VpsD => self.vps_d += mod_value,
            ModDestination::VpsV => self.vps_v += mod_value,
            ModDestination::FilterCutoff => self.filter_cutoff += mod_value,
            ModDestination::FilterResonance => self.filter_resonance += mod_value,
            ModDestination::FilterDrive => self.filter_drive += mod_value,
            ModDestination::RingMod => self.ring_mod += mod_value,
            ModDestination::Wavefold => self.wavefold += mod_value,
            ModDestination::DriftAmount => self.drift_amount += mod_value,
            ModDestination::NoiseAmount => self.noise_amount += mod_value,
            ModDestination::TubeDrive => self.tube_drive += mod_value,
            ModDestination::ReverbMix => self.reverb_mix += mod_value,
            ModDestination::ReverbDecay => self.reverb_decay += mod_value,
            ModDestination::PllVolume => self.pll_volume += mod_value,
            ModDestination::VpsVolume => self.vps_volume += mod_value,
            ModDestination::SubVolume => self.sub_volume += mod_value,
        }
    }
}

// LFO bank with 3 LFOs and modulation routing
pub struct LfoBank {
    pub lfos: [Lfo; 3],

    // Each LFO has 2 modulation slots (destination + amount)
    pub destinations: [[ModDestination; 2]; 3],
    pub amounts: [[f64; 2]; 3],

    // Slews for amounts to avoid clicks when changing modulation depth
    amount_slews: [[SlewValue<f64>; 2]; 3],

    // Current LFO outputs (for cross-modulation)
    lfo_outputs: [f64; 3],

    sample_rate: f64,
}

impl LfoBank {
    pub fn new(sample_rate: f64) -> Self {
        let make_slew = || {
            let mut s = SlewValue::new();
            s.set_sample_rate(sample_rate);
            s
        };

        Self {
            lfos: [
                Lfo::new(sample_rate),
                Lfo::new(sample_rate),
                Lfo::new(sample_rate),
            ],
            destinations: [[ModDestination::None; 2]; 3],
            amounts: [[0.0; 2]; 3],
            amount_slews: [
                [make_slew(), make_slew()],
                [make_slew(), make_slew()],
                [make_slew(), make_slew()],
            ],
            lfo_outputs: [0.0; 3],
            sample_rate,
        }
    }

    pub fn set_lfo_params(
        &mut self,
        lfo_idx: usize,
        rate: f64,
        waveform: i32,
        tempo_sync: bool,
        sync_division: i32,
        sync_source: i32,
        phase_mod_amount: f64,
    ) {
        if lfo_idx >= 3 { return; }

        let lfo = &mut self.lfos[lfo_idx];
        lfo.set_rate(rate);
        lfo.set_waveform(LfoWaveform::from_index(waveform));
        lfo.set_tempo_sync(tempo_sync);
        lfo.set_sync_division(LfoSyncDivision::from_index(sync_division));

        // Sync source: -1 = none, 0/1/2 = LFO 1/2/3
        let source = if sync_source < 0 || sync_source as usize == lfo_idx {
            None
        } else {
            Some(sync_source as usize)
        };
        lfo.set_sync_source(source);
        lfo.set_phase_mod_amount(phase_mod_amount);
    }

    pub fn set_modulation(
        &mut self,
        lfo_idx: usize,
        slot: usize,
        destination: i32,
        amount: f64,
    ) {
        if lfo_idx >= 3 || slot >= 2 { return; }

        self.destinations[lfo_idx][slot] = ModDestination::from_index(destination);
        self.amounts[lfo_idx][slot] = amount;
    }

    pub fn process(&mut self, bpm: f64) -> ModulationValues {
        let mut mod_values = ModulationValues::default();

        // Process LFOs in order (so earlier LFOs can sync later ones)
        for i in 0..3 {
            let sync_input = self.lfos[i].sync_source.map(|src| self.lfo_outputs[src]);
            self.lfo_outputs[i] = self.lfos[i].process(bpm, sync_input);
        }

        // Apply modulations
        for lfo_idx in 0..3 {
            let lfo_value = self.lfo_outputs[lfo_idx];

            for slot in 0..2 {
                let dest = self.destinations[lfo_idx][slot];
                let target_amount = self.amounts[lfo_idx][slot];

                // Slew the amount to avoid clicks
                let slewed_amount = self.amount_slews[lfo_idx][slot].next(target_amount, 30.0);

                mod_values.add_modulation(dest, slewed_amount, lfo_value);
            }
        }

        mod_values
    }

    pub fn get_lfo_output(&self, idx: usize) -> f64 {
        if idx < 3 { self.lfo_outputs[idx] } else { 0.0 }
    }

    pub fn set_sample_rate(&mut self, sample_rate: f64) {
        self.sample_rate = sample_rate;
        for lfo in &mut self.lfos {
            lfo.set_sample_rate(sample_rate);
        }
        for slots in &mut self.amount_slews {
            for slew in slots {
                slew.set_sample_rate(sample_rate);
            }
        }
    }
}
