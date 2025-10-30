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
        filter.set_mix(LadderMode::LP24);
        filter.reset();

        Self {
            filter,
            params,
            sample_rate,
        }
    }

    pub fn process_buffer(&mut self, buffer: &mut [f32; 4], cutoff: f32, resonance: f32, drive: f32, mode: i32) {
        let cutoff = cutoff.clamp(20.0, 20000.0);
        let resonance = resonance.clamp(0.0, 0.99);
        let drive = drive.clamp(1.0, 15.849);

        self.params.set_frequency(cutoff);
        self.params.set_resonance(resonance);
        self.params.drive = drive;

        // Map mode integer to LadderMode enum
        let new_mode = match mode {
            0 => LadderMode::LP6,
            1 => LadderMode::LP12,
            2 => LadderMode::LP18,
            3 => LadderMode::LP24,
            4 => LadderMode::HP6,
            5 => LadderMode::HP12,
            6 => LadderMode::HP18,
            7 => LadderMode::HP24,
            8 => LadderMode::BP12,
            9 => LadderMode::BP24,
            10 => LadderMode::N12,
            _ => LadderMode::LP24,
        };

        if self.params.ladder_mode != new_mode {
            self.params.ladder_mode = new_mode;
            self.filter.set_mix(new_mode);
            self.filter.params = Arc::new(self.params.clone());
        } else {
            self.filter.params = Arc::new(self.params.clone());
        }

        let input = f32x4::from_array(*buffer);
        let output = self.filter.tick_pivotal(input);
        *buffer = output.to_array();
    }

    pub fn process_stereo(
        &mut self,
        left: &mut [f32],
        right: &mut [f32],
        cutoff: f32,
        resonance: f32,
        drive: f32,
        mode: i32,
    ) {
        debug_assert_eq!(left.len(), 4);
        debug_assert_eq!(right.len(), 4);

        unsafe {
            self.process_buffer(
                &mut *(left.as_mut_ptr() as *mut [f32; 4]),
                cutoff,
                resonance,
                drive,
                mode,
            );
            self.process_buffer(
                &mut *(right.as_mut_ptr() as *mut [f32; 4]),
                cutoff,
                resonance,
                drive,
                mode,
            );
        }
    }

}
