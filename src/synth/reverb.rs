use synfx_dsp::{DattorroReverb, DattorroReverbParams};

pub struct ReverbParams {
    pub pre_delay_ms: f32,
    pub time_scale: f32,
    pub input_hpf_hz: f32,
    pub input_lpf_hz: f32,
    pub reverb_hpf_hz: f32,
    pub reverb_lpf_hz: f32,
    pub mod_speed: f32,
    pub mod_depth: f32,
    pub mod_shape: f32,
    pub input_diffusion_mix: f32,
    pub diffusion: f32,
    pub decay: f32,
}

impl DattorroReverbParams for ReverbParams {
    fn pre_delay_time_ms(&self) -> f64 {
        self.pre_delay_ms as f64
    }

    fn time_scale(&self) -> f64 {
        self.time_scale as f64
    }

    fn input_high_cutoff_hz(&self) -> f64 {
        self.input_hpf_hz as f64
    }

    fn input_low_cutoff_hz(&self) -> f64 {
        self.input_lpf_hz as f64
    }

    fn reverb_high_cutoff_hz(&self) -> f64 {
        self.reverb_hpf_hz as f64
    }

    fn reverb_low_cutoff_hz(&self) -> f64 {
        self.reverb_lpf_hz as f64
    }

    fn mod_speed(&self) -> f64 {
        self.mod_speed as f64
    }

    fn mod_depth(&self) -> f64 {
        self.mod_depth as f64
    }

    fn mod_shape(&self) -> f64 {
        self.mod_shape as f64
    }

    fn input_diffusion_mix(&self) -> f64 {
        self.input_diffusion_mix as f64
    }

    fn diffusion(&self) -> f64 {
        self.diffusion as f64
    }

    fn decay(&self) -> f64 {
        self.decay as f64
    }
}

pub struct StereoReverb {
    reverb: DattorroReverb,
    params: ReverbParams,
    pub mix: f32,
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
        }
    }

    pub fn reset(&mut self) {
        self.reverb.reset();
    }

    pub fn set_params(
        &mut self,
        mix: f32,
        pre_delay_ms: f32,
        time_scale: f32,
        input_hpf_hz: f32,
        input_lpf_hz: f32,
        reverb_hpf_hz: f32,
        reverb_lpf_hz: f32,
        mod_speed: f32,
        mod_depth: f32,
        mod_shape: f32,
        input_diffusion_mix: f32,
        diffusion: f32,
        decay: f32,
    ) {
        self.mix = mix;
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

        self.reverb.set_time_scale(time_scale as f64);
    }

    pub fn process(&mut self, left: f32, right: f32) -> (f32, f32) {
        let (wet_l, wet_r) = self.reverb.process(&mut self.params, left as f64, right as f64);
        (wet_l as f32, wet_r as f32)
    }
}
