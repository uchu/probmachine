use synfx_dsp::fh_va::{LadderFilter, FilterParams, LadderMode};
use std::sync::Arc;
use std::simd::f32x4;

pub struct MoogFilter {
    filter: LadderFilter,
    params: Arc<FilterParams>,
    sample_rate: f32,
    last_cutoff: f32,
    last_resonance: f32,
}

impl MoogFilter {
    pub fn new(sample_rate: f32) -> Self {
        let mut params = FilterParams::new();
        params.set_sample_rate(sample_rate);
        params.set_frequency(1000.0);
        params.set_resonance(0.0);
        params.ladder_mode = LadderMode::LP24;

        let params = Arc::new(params);
        let mut filter = LadderFilter::new(params.clone());
        filter.reset();

        Self {
            filter,
            params,
            sample_rate,
            last_cutoff: 1000.0,
            last_resonance: 0.0,
        }
    }

    pub fn process_buffer(&mut self, buffer: &mut [f32; 4], cutoff: f32, resonance: f32) {
        let resonance = resonance.clamp(0.0, 0.99);

        if (cutoff - self.last_cutoff).abs() > 0.1 || (resonance - self.last_resonance).abs() > 0.001 {
            let mut params = FilterParams::new();
            params.set_sample_rate(self.sample_rate);
            params.set_frequency(cutoff);
            params.set_resonance(resonance);
            params.ladder_mode = LadderMode::LP24;

            self.params = Arc::new(params);
            self.filter = LadderFilter::new(self.params.clone());

            self.last_cutoff = cutoff;
            self.last_resonance = resonance;
        }

        let input = f32x4::from_array(*buffer);
        let output = self.filter.tick_pivotal(input);
        *buffer = output.to_array();
    }
}
