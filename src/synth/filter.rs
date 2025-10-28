use synfx_dsp::fh_va::{LadderFilter, FilterParams, LadderMode};
use std::sync::Arc;
use std::simd::f32x4;

pub struct MoogFilter {
    filter: LadderFilter,
    params: Arc<FilterParams>,
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
        }
    }

    pub fn process_buffer(&mut self, buffer: &mut [f32; 4], cutoff: f32, resonance: f32) {
        let params = Arc::get_mut(&mut self.params).unwrap();
        params.set_frequency(cutoff);
        params.set_resonance(resonance.clamp(0.0, 0.99));

        let input = f32x4::from_array(*buffer);
        let output = self.filter.tick_pivotal(input);
        *buffer = output.to_array();
    }
}
