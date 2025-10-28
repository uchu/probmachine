use synfx_dsp::{Oversampling, f_distort};
use super::oscillator::{Oscillator, PolyBlepWrapper};
use super::filter::MoogFilter;
use super::envelope::Envelope;

pub struct Voice {
    oscillator: Oscillator,
    polyblep: PolyBlepWrapper,
    sub_phase: f32,
    filter: MoogFilter,
    volume_env: Envelope,
    sub_env: Envelope,
    filter_env: Envelope,
    oversampling: Oversampling<4>,

    osc_d: f32,
    osc_v: f32,
    osc_volume: f32,
    polyblep_volume: f32,
    polyblep_pulse_width: f32,
    sub_volume: f32,
    sub_frequency: f32,
    distortion_amount: f32,
    distortion_threshold: f32,
    filter_cutoff: f32,
    filter_resonance: f32,
    filter_env_amount: f32,
    volume: f32,
    sample_rate: f32,

    volume_attack: f32,
    volume_attack_shape: f32,
    volume_decay: f32,
    volume_decay_shape: f32,
    volume_sustain: f32,
    volume_release: f32,
    volume_release_shape: f32,

    filter_attack: f32,
    filter_attack_shape: f32,
    filter_decay: f32,
    filter_decay_shape: f32,
    filter_sustain: f32,
    filter_release: f32,
    filter_release_shape: f32,
}

impl Voice {
    pub fn new(sample_rate: f32) -> Self {
        let mut oversampling = Oversampling::<4>::new();
        oversampling.set_sample_rate(sample_rate);

        let oversampled_rate = sample_rate * 4.0;
        Self {
            oscillator: Oscillator::new(oversampled_rate),
            polyblep: PolyBlepWrapper::new(oversampled_rate),
            sub_phase: 0.0,
            filter: MoogFilter::new(oversampled_rate),
            volume_env: Envelope::new(sample_rate),
            sub_env: Envelope::new(sample_rate),
            filter_env: Envelope::new(sample_rate),
            oversampling,

            osc_d: 0.5,
            osc_v: 0.5,
            osc_volume: 1.0,
            polyblep_volume: 0.0,
            polyblep_pulse_width: 0.5,
            sub_volume: 0.0,
            sub_frequency: 110.0,
            distortion_amount: 0.0,
            distortion_threshold: 0.9,
            filter_cutoff: 1000.0,
            filter_resonance: 0.0,
            filter_env_amount: 0.0,
            volume: 0.8,
            sample_rate,

            volume_attack: 10.0,
            volume_attack_shape: 0.5,
            volume_decay: 100.0,
            volume_decay_shape: 0.5,
            volume_sustain: 0.7,
            volume_release: 200.0,
            volume_release_shape: 0.5,

            filter_attack: 10.0,
            filter_attack_shape: 0.5,
            filter_decay: 100.0,
            filter_decay_shape: 0.5,
            filter_sustain: 0.5,
            filter_release: 200.0,
            filter_release_shape: 0.5,
        }
    }

    pub fn set_frequency(&mut self, freq: f32) {
        self.oscillator.set_frequency(freq);
        self.polyblep.set_frequency(freq);
        self.sub_frequency = freq * 0.5;
    }

    pub fn set_osc_params(&mut self, d: f32, v: f32) {
        self.osc_d = d;
        self.osc_v = v;
        self.oscillator.set_params(d, v);
    }

    pub fn set_osc_volume(&mut self, volume: f32) {
        self.osc_volume = volume;
    }

    pub fn set_polyblep_params(&mut self, volume: f32, pulse_width: f32) {
        self.polyblep_volume = volume;
        self.polyblep_pulse_width = pulse_width;
    }

    pub fn set_sub_volume(&mut self, volume: f32) {
        self.sub_volume = volume;
    }

    pub fn set_distortion_params(&mut self, amount: f32, threshold: f32) {
        self.distortion_amount = amount;
        self.distortion_threshold = threshold;
    }

    pub fn set_filter_params(&mut self, cutoff: f32, resonance: f32, env_amount: f32) {
        self.filter_cutoff = cutoff;
        self.filter_resonance = resonance;
        self.filter_env_amount = env_amount;
    }

    pub fn set_volume(&mut self, volume: f32) {
        self.volume = volume;
    }

    pub fn set_volume_envelope(&mut self, attack: f32, attack_shape: f32, decay: f32, decay_shape: f32, sustain: f32, release: f32, release_shape: f32) {
        self.volume_attack = attack;
        self.volume_attack_shape = attack_shape;
        self.volume_decay = decay;
        self.volume_decay_shape = decay_shape;
        self.volume_sustain = sustain;
        self.volume_release = release;
        self.volume_release_shape = release_shape;
    }

    pub fn set_filter_envelope(&mut self, attack: f32, attack_shape: f32, decay: f32, decay_shape: f32, sustain: f32, release: f32, release_shape: f32) {
        self.filter_attack = attack;
        self.filter_attack_shape = attack_shape;
        self.filter_decay = decay;
        self.filter_decay_shape = decay_shape;
        self.filter_sustain = sustain;
        self.filter_release = release;
        self.filter_release_shape = release_shape;
    }

    pub fn trigger(&mut self) {
        self.volume_env.trigger(
            self.volume_attack,
            self.volume_attack_shape,
            self.volume_decay,
            self.volume_decay_shape,
            self.volume_sustain,
            self.volume_release,
            self.volume_release_shape,
        );

        self.sub_env.trigger(
            self.volume_attack,
            self.volume_attack_shape,
            self.volume_decay,
            self.volume_decay_shape,
            self.volume_sustain,
            self.volume_release,
            self.volume_release_shape,
        );

        self.filter_env.trigger(
            self.filter_attack,
            self.filter_attack_shape,
            self.filter_decay,
            self.filter_decay_shape,
            self.filter_sustain,
            self.filter_release,
            self.filter_release_shape,
        );
    }

    pub fn release(&mut self) {
        self.volume_env.release();
        self.sub_env.release();
        self.filter_env.release();
    }

    pub fn process(&mut self) -> f32 {
        let vol_env = self.volume_env.next();
        let sub_env = self.sub_env.next();
        let filt_env = self.filter_env.next();

        let modulated_cutoff = self.filter_cutoff + (filt_env * self.filter_env_amount);
        let modulated_cutoff = modulated_cutoff.clamp(20.0, 20000.0);

        let overbuf = self.oversampling.resample_buffer();
        for sample in overbuf.iter_mut() {
            let osc_sample = self.oscillator.next(self.osc_d, self.osc_v);
            let gain = 0.1 + self.distortion_amount * 4.9;
            let distorted = f_distort(gain, self.distortion_threshold, osc_sample);
            let enveloped = distorted * self.osc_volume * vol_env;

            let polyblep_sample = if self.polyblep_volume > 0.0 {
                self.polyblep.next(self.polyblep_pulse_width)
            } else {
                0.0
            };
            let polyblep_enveloped = polyblep_sample * self.polyblep_volume * vol_env;

            *sample = enveloped + polyblep_enveloped;
        }

        self.filter.process_buffer(overbuf, modulated_cutoff, self.filter_resonance);

        let main_output = self.oversampling.downsample();

        let sub_sample = if self.sub_volume > 0.0 {
            use std::f32::consts::PI;
            let phase_increment = 2.0 * PI * self.sub_frequency / self.sample_rate;
            self.sub_phase += phase_increment;
            if self.sub_phase >= 2.0 * PI {
                self.sub_phase -= 2.0 * PI;
            }
            self.sub_phase.sin() * self.sub_volume * sub_env
        } else {
            0.0
        };

        (main_output + sub_sample) * self.volume
    }
}
