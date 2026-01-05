#![allow(clippy::too_many_arguments)]

use synfx_dsp::{DattorroReverb, DattorroReverbParams};

pub struct ReverbParams {
    pub pre_delay_ms: f64,
    pub time_scale: f64,
    pub input_hpf_hz: f64,
    pub input_lpf_hz: f64,
    pub reverb_hpf_hz: f64,
    pub reverb_lpf_hz: f64,
    pub mod_speed: f64,
    pub mod_depth: f64,
    pub mod_shape: f64,
    pub input_diffusion_mix: f64,
    pub diffusion: f64,
    pub decay: f64,
}

impl DattorroReverbParams for ReverbParams {
    fn pre_delay_time_ms(&self) -> f64 {
        self.pre_delay_ms
    }

    fn time_scale(&self) -> f64 {
        self.time_scale
    }

    fn input_high_cutoff_hz(&self) -> f64 {
        self.input_hpf_hz
    }

    fn input_low_cutoff_hz(&self) -> f64 {
        self.input_lpf_hz
    }

    fn reverb_high_cutoff_hz(&self) -> f64 {
        self.reverb_hpf_hz
    }

    fn reverb_low_cutoff_hz(&self) -> f64 {
        self.reverb_lpf_hz
    }

    fn mod_speed(&self) -> f64 {
        self.mod_speed
    }

    fn mod_depth(&self) -> f64 {
        self.mod_depth
    }

    fn mod_shape(&self) -> f64 {
        self.mod_shape
    }

    fn input_diffusion_mix(&self) -> f64 {
        self.input_diffusion_mix
    }

    fn diffusion(&self) -> f64 {
        self.diffusion
    }

    fn decay(&self) -> f64 {
        self.decay
    }
}

pub struct StereoReverb {
    reverb: DattorroReverb,
    params: ReverbParams,
    pub mix: f64,
    pub ducking: f64,
    ducking_envelope: f64,
}

impl StereoReverb {
    pub fn new(sample_rate: f32) -> Self {
        let mut reverb = DattorroReverb::new();
        reverb.set_sample_rate(sample_rate as f64);
        reverb.set_time_scale(0.85);

        Self {
            reverb,
            params: ReverbParams {
                pre_delay_ms: 50.0,
                time_scale: 0.85,
                input_hpf_hz: 20.0,
                input_lpf_hz: 18000.0,
                reverb_hpf_hz: 100.0,
                reverb_lpf_hz: 14000.0,
                mod_speed: 0.3,
                mod_depth: 0.4,
                mod_shape: 0.5,
                input_diffusion_mix: 0.85,
                diffusion: 0.75,
                decay: 0.8,
            },
            mix: 0.0,
            ducking: 0.0,
            ducking_envelope: 0.0,
        }
    }

    pub fn set_params(
        &mut self,
        mix: f64,
        pre_delay_ms: f64,
        time_scale: f64,
        input_hpf_hz: f64,
        input_lpf_hz: f64,
        reverb_hpf_hz: f64,
        reverb_lpf_hz: f64,
        mod_speed: f64,
        mod_depth: f64,
        mod_shape: f64,
        input_diffusion_mix: f64,
        diffusion: f64,
        decay: f64,
        ducking: f64,
    ) {
        self.mix = mix;
        self.ducking = ducking;
        self.params.pre_delay_ms = pre_delay_ms;
        self.params.time_scale = time_scale;
        self.params.input_hpf_hz = input_hpf_hz;
        self.params.input_lpf_hz = input_lpf_hz;
        self.params.reverb_hpf_hz = reverb_hpf_hz;
        self.params.reverb_lpf_hz = reverb_lpf_hz;
        self.params.mod_speed = mod_speed;
        self.params.mod_depth = mod_depth;
        self.params.mod_shape = mod_shape;
        self.params.input_diffusion_mix = input_diffusion_mix;
        self.params.diffusion = diffusion;
        self.params.decay = decay;

        self.reverb.set_time_scale(time_scale);
    }

    pub fn process(&mut self, left: f64, right: f64) -> (f64, f64) {
        let (wet_l, wet_r) = self.reverb.process(&mut self.params, left, right);

        // Apply ducking - reduce reverb when dry signal is loud
        if self.ducking > 0.001 {
            let dry_level = (left.abs() + right.abs()) * 0.5;
            // Envelope follower for smooth ducking
            let attack = 0.01;
            let release = 0.995;
            if dry_level > self.ducking_envelope {
                self.ducking_envelope = self.ducking_envelope * (1.0 - attack) + dry_level * attack;
            } else {
                self.ducking_envelope *= release;
            }
            // Apply ducking amount
            let duck_factor = 1.0 - (self.ducking_envelope * self.ducking).min(0.95);
            return (wet_l * duck_factor, wet_r * duck_factor);
        }

        (wet_l, wet_r)
    }
}
