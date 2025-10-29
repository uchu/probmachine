use synfx_dsp::fh_va::{LadderFilter, FilterParams, LadderMode};
use std::sync::Arc;
use std::simd::f32x4;

pub struct MoogFilter {
    filter: LadderFilter,
    params: FilterParams,
    sample_rate: f32,
}

impl MoogFilter {
    pub fn new(sample_rate: f32) -> Self {
        let mut params = FilterParams::new();
        params.set_sample_rate(sample_rate);
        params.set_frequency(1000.0);
        params.set_resonance(0.0);
        params.ladder_mode = LadderMode::LP24;

        let params_arc = Arc::new(params.clone());
        let mut filter = LadderFilter::new(params_arc);
        filter.reset();

        Self {
            filter,
            params,
            sample_rate,
        }
    }

    pub fn process_buffer(&mut self, buffer: &mut [f32; 4], cutoff: f32, resonance: f32) {
        let cutoff = cutoff.clamp(20.0, 20000.0);
        let resonance = resonance.clamp(0.0, 0.99);

        self.params.set_frequency(cutoff);
        self.params.set_resonance(resonance);
        self.filter.params = Arc::new(self.params.clone());

        let input = f32x4::from_array(*buffer);
        let output = self.filter.tick_pivotal(input);
        *buffer = output.to_array();
    }
}
