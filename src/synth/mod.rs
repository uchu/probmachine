mod oscillator;
mod filter;
mod envelope;
mod voice;

pub use voice::Voice;

pub struct SynthEngine {
    voice: Voice,
    sample_rate: f32,
    retrigger_timer: f32,
}

impl SynthEngine {
    pub fn new(sample_rate: f32) -> Self {
        let mut voice = Voice::new(sample_rate);
        voice.set_frequency(220.0);

        Self {
            voice,
            sample_rate,
            retrigger_timer: 0.0,
        }
    }

    pub fn set_osc_params(&mut self, d: f32, v: f32) {
        self.voice.set_osc_params(d, v);
    }

    pub fn set_distortion_params(&mut self, amount: f32, threshold: f32) {
        self.voice.set_distortion_params(amount, threshold);
    }

    pub fn set_filter_params(&mut self, cutoff: f32, resonance: f32, env_amount: f32) {
        self.voice.set_filter_params(cutoff, resonance, env_amount);
    }

    pub fn set_volume(&mut self, volume: f32) {
        self.voice.set_volume(volume);
    }

    pub fn set_volume_envelope(&mut self, attack: f32, attack_shape: f32, decay: f32, decay_shape: f32, sustain: f32, release: f32, release_shape: f32) {
        self.voice.set_volume_envelope(attack, attack_shape, decay, decay_shape, sustain, release, release_shape);
    }

    pub fn set_filter_envelope(&mut self, attack: f32, attack_shape: f32, decay: f32, decay_shape: f32, sustain: f32, release: f32, release_shape: f32) {
        self.voice.set_filter_envelope(attack, attack_shape, decay, decay_shape, sustain, release, release_shape);
    }

    pub fn process_block(&mut self, output_l: &mut [f32], output_r: &mut [f32]) {
        let samples_per_second = self.sample_rate;
        let retrigger_interval = samples_per_second;

        for (l, r) in output_l.iter_mut().zip(output_r.iter_mut()) {
            if self.retrigger_timer >= retrigger_interval {
                self.voice.trigger();
                self.retrigger_timer = 0.0;
            }

            let sample = self.voice.process();
            *l = sample;
            *r = sample;

            self.retrigger_timer += 1.0;
        }
    }
}
