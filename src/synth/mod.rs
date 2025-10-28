mod oscillator;
mod filter;
mod envelope;
mod voice;

pub use voice::Voice;
use crate::sequencer::Sequencer;
use crate::params::DeviceParams;

pub struct SynthEngine {
    voice: Voice,
    sample_rate: f32,
    sequencer: Sequencer,
}

impl SynthEngine {
    pub fn new(sample_rate: f32) -> Self {
        let mut voice = Voice::new(sample_rate);
        voice.set_frequency(220.0);

        Self {
            voice,
            sample_rate,
            sequencer: Sequencer::new(sample_rate, 120.0),
        }
    }

    pub fn set_osc_params(&mut self, d: f32, v: f32) {
        self.voice.set_osc_params(d, v);
    }

    pub fn set_osc_volume(&mut self, volume: f32) {
        self.voice.set_osc_volume(volume);
    }

    pub fn set_polyblep_params(&mut self, volume: f32, pulse_width: f32) {
        self.voice.set_polyblep_params(volume, pulse_width);
    }

    pub fn set_sub_volume(&mut self, volume: f32) {
        self.voice.set_sub_volume(volume);
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

    pub fn process_block(&mut self, output_l: &mut [f32], output_r: &mut [f32], params: &DeviceParams) {
        for (l, r) in output_l.iter_mut().zip(output_r.iter_mut()) {
            let (should_trigger, should_release) = self.sequencer.update(params);

            if should_trigger {
                self.voice.trigger();
            }

            if should_release {
                self.voice.release();
            }

            let sample = self.voice.process();
            *l = sample;
            *r = sample;
        }
    }
}
