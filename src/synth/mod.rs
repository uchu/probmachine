mod oscillator;
mod filter;
mod envelope;
mod reverb;
mod voice;

pub use voice::Voice;
use crate::sequencer::Sequencer;
use crate::params::DeviceParams;

pub struct SynthEngine {
    voice: Voice,
    #[allow(dead_code)]
    sample_rate: f32,
    sequencer: Sequencer,
    pll_feedback: f32,
}

impl SynthEngine {
    pub fn new(sample_rate: f32) -> Self {
        let mut voice = Voice::new(sample_rate);
        voice.set_frequency(220.0, 0.0, 0.0);

        Self {
            voice,
            sample_rate,
            sequencer: Sequencer::new(sample_rate, 120.0),
            pll_feedback: 0.0,
        }
    }

    pub fn set_osc_params(&mut self, d: f32, v: f32) {
        self.voice.set_osc_params(d, v);
    }

    pub fn set_osc_volume(&mut self, volume: f32) {
        self.voice.set_osc_volume(volume);
    }

    pub fn set_osc_octave(&mut self, octave: i32) {
        self.voice.set_osc_octave(octave);
    }

    pub fn set_vps_stereo_v_offset(&mut self, offset: f32) {
        self.voice.set_vps_stereo_v_offset(offset);
    }

    pub fn set_polyblep_params(&mut self, volume: f32, pulse_width: f32, octave: i32) {
        self.voice.set_polyblep_params(volume, pulse_width, octave);
    }

    pub fn set_polyblep_stereo_width(&mut self, width: f32) {
        self.voice.set_polyblep_stereo_width(width);
    }

    pub fn set_pll_ref_params(&mut self, octave: i32, tune: i32, fine_tune: f32, pulse_width: f32) {
        self.voice.set_pll_ref_params(octave, tune, fine_tune, pulse_width);
    }

    pub fn set_pll_params(&mut self, track: f32, damp: f32, mult: f32, range: f32, colored: bool, edge_mode: bool) {
        self.voice.set_pll_params(track, damp, mult, range, colored, edge_mode);
    }

    pub fn set_pll_volume(&mut self, volume: f32) {
        self.voice.set_pll_volume(volume);
    }

    pub fn set_pll_ki_multiplier(&mut self, ki_mult: f32) {
        self.voice.set_pll_ki_multiplier(ki_mult);
    }

    pub fn set_pll_stereo_damp_offset(&mut self, offset: f32) {
        self.voice.set_pll_stereo_damp_offset(offset);
    }

    pub fn set_pll_distortion_params(&mut self, amount: f32, threshold: f32) {
        self.voice.set_pll_distortion_params(amount, threshold);
    }

    pub fn set_sub_params(&mut self, volume: f32, octave: i32, shape: f32) {
        self.voice.set_sub_params(volume, octave, shape);
    }

    pub fn set_distortion_params(&mut self, amount: f32, threshold: f32) {
        self.voice.set_distortion_params(amount, threshold);
    }

    pub fn set_filter_params(&mut self, cutoff: f32, resonance: f32, env_amount: f32, drive: f32, mode: i32) {
        self.voice.set_filter_params(cutoff, resonance, env_amount, drive, mode);
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

    pub fn set_vps_dry_wet(&mut self, dry_wet: f32) {
        self.voice.set_vps_dry_wet(dry_wet);
    }

    pub fn set_reverb_params(
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
        self.voice.set_reverb_params(
            mix,
            pre_delay_ms,
            time_scale,
            input_hpf_hz,
            input_lpf_hz,
            reverb_hpf_hz,
            reverb_lpf_hz,
            mod_speed,
            mod_depth,
            mod_shape,
            input_diffusion_mix,
            diffusion,
            decay,
        );
    }

    pub fn process_block(&mut self, output_l: &mut [f32], output_r: &mut [f32], params: &DeviceParams, feedback_amount: f32, base_freq: f32) {
        for (l, r) in output_l.iter_mut().zip(output_r.iter_mut()) {
            let (should_trigger, should_release) = self.sequencer.update(params);

            if should_trigger {
                self.voice.set_frequency(base_freq, self.pll_feedback, feedback_amount);
                self.voice.trigger();
            }

            if should_release {
                self.voice.release();
            }

            let (left_sample, right_sample) = self.voice.process(self.pll_feedback);

            *l = left_sample;
            *r = right_sample;
        }
    }
}
