use synfx_dsp::{
    process_simper_svf,
    process_1pole_tpt_lowpass,
    process_1pole_tpt_highpass,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FilterMode {
    LP6 = 0,
    LP12 = 1,
    LP18 = 2,
    LP24 = 3,
    HP6 = 4,
    HP12 = 5,
    HP18 = 6,
    HP24 = 7,
    BP12 = 8,
    BP24 = 9,
    Notch = 10,
}

impl FilterMode {
    pub fn from_i32(v: i32) -> Self {
        match v {
            0 => FilterMode::LP6,
            1 => FilterMode::LP12,
            2 => FilterMode::LP18,
            3 => FilterMode::LP24,
            4 => FilterMode::HP6,
            5 => FilterMode::HP12,
            6 => FilterMode::HP18,
            7 => FilterMode::HP24,
            8 => FilterMode::BP12,
            9 => FilterMode::BP24,
            10 => FilterMode::Notch,
            _ => FilterMode::LP24,
        }
    }
}

pub struct MultiModeFilter {
    israte: f32,

    // SVF stage 1 state
    svf1_ic1eq: f32,
    svf1_ic2eq: f32,

    // SVF stage 2 state (for 24dB modes)
    svf2_ic1eq: f32,
    svf2_ic2eq: f32,

    // One-pole states (for 6dB and 18dB modes)
    lp1_z: f32,
    hp1_z: f32,
}

impl MultiModeFilter {
    pub fn new(sample_rate: f64) -> Self {
        Self {
            israte: 1.0 / sample_rate as f32,
            svf1_ic1eq: 0.0,
            svf1_ic2eq: 0.0,
            svf2_ic1eq: 0.0,
            svf2_ic2eq: 0.0,
            lp1_z: 0.0,
            hp1_z: 0.0,
        }
    }

    #[allow(dead_code)]
    pub fn reset(&mut self) {
        self.svf1_ic1eq = 0.0;
        self.svf1_ic2eq = 0.0;
        self.svf2_ic1eq = 0.0;
        self.svf2_ic2eq = 0.0;
        self.lp1_z = 0.0;
        self.hp1_z = 0.0;
    }

    #[inline]
    fn process_sample_internal(&mut self, input: f32, freq: f32, res: f32, mode: FilterMode) -> f32 {
        match mode {
            FilterMode::LP6 => {
                process_1pole_tpt_lowpass(input, freq, self.israte, &mut self.lp1_z)
            }
            FilterMode::LP12 => {
                let (lp, _, _) = process_simper_svf(
                    input, freq, res, self.israte,
                    &mut self.svf1_ic1eq, &mut self.svf1_ic2eq
                );
                lp
            }
            FilterMode::LP18 => {
                // 12dB SVF + 6dB one-pole
                let (lp12, _, _) = process_simper_svf(
                    input, freq, res, self.israte,
                    &mut self.svf1_ic1eq, &mut self.svf1_ic2eq
                );
                process_1pole_tpt_lowpass(lp12, freq, self.israte, &mut self.lp1_z)
            }
            FilterMode::LP24 => {
                // Two cascaded 12dB SVFs
                let (lp1, _, _) = process_simper_svf(
                    input, freq, res, self.israte,
                    &mut self.svf1_ic1eq, &mut self.svf1_ic2eq
                );
                let (lp2, _, _) = process_simper_svf(
                    lp1, freq, res * 0.5, self.israte, // reduce res on second stage
                    &mut self.svf2_ic1eq, &mut self.svf2_ic2eq
                );
                lp2
            }
            FilterMode::HP6 => {
                process_1pole_tpt_highpass(input, freq, self.israte, &mut self.hp1_z)
            }
            FilterMode::HP12 => {
                let (_, _, hp) = process_simper_svf(
                    input, freq, res, self.israte,
                    &mut self.svf1_ic1eq, &mut self.svf1_ic2eq
                );
                hp
            }
            FilterMode::HP18 => {
                // 12dB SVF + 6dB one-pole
                let (_, _, hp12) = process_simper_svf(
                    input, freq, res, self.israte,
                    &mut self.svf1_ic1eq, &mut self.svf1_ic2eq
                );
                process_1pole_tpt_highpass(hp12, freq, self.israte, &mut self.hp1_z)
            }
            FilterMode::HP24 => {
                // Two cascaded 12dB SVFs
                let (_, _, hp1) = process_simper_svf(
                    input, freq, res, self.israte,
                    &mut self.svf1_ic1eq, &mut self.svf1_ic2eq
                );
                let (_, _, hp2) = process_simper_svf(
                    hp1, freq, res * 0.5, self.israte,
                    &mut self.svf2_ic1eq, &mut self.svf2_ic2eq
                );
                hp2
            }
            FilterMode::BP12 => {
                let (_, bp, _) = process_simper_svf(
                    input, freq, res, self.israte,
                    &mut self.svf1_ic1eq, &mut self.svf1_ic2eq
                );
                bp
            }
            FilterMode::BP24 => {
                // Two cascaded bandpass
                let (_, bp1, _) = process_simper_svf(
                    input, freq, res, self.israte,
                    &mut self.svf1_ic1eq, &mut self.svf1_ic2eq
                );
                let (_, bp2, _) = process_simper_svf(
                    bp1, freq, res * 0.5, self.israte,
                    &mut self.svf2_ic1eq, &mut self.svf2_ic2eq
                );
                bp2
            }
            FilterMode::Notch => {
                // Notch = input - bandpass
                let (_, bp, _) = process_simper_svf(
                    input, freq, res, self.israte,
                    &mut self.svf1_ic1eq, &mut self.svf1_ic2eq
                );
                input - bp * (1.0 + res) // Scale BP removal by resonance for better notch depth
            }
        }
    }

    pub fn process_buffer(&mut self, buffer: &mut [f32; 4], cutoff: f32, resonance: f32, _drive: f32, mode: i32) {
        let freq = cutoff.clamp(20.0, 20000.0);
        let res = resonance.clamp(0.0, 0.99);
        let filter_mode = FilterMode::from_i32(mode);

        for sample in buffer.iter_mut() {
            *sample = self.process_sample_internal(*sample, freq, res, filter_mode);
        }
    }
}

// Keep the old name for compatibility
pub type MoogFilter = MultiModeFilter;
