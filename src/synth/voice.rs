use synfx_dsp::{Oversampling, apply_distortion};
use super::oscillator::Oscillator;
use super::filter::MoogFilter;
use super::envelope::Envelope;

pub struct Voice {
    oscillator: Oscillator,
    filter: MoogFilter,
    volume_env: Envelope,
    filter_env: Envelope,
    oversampling: Oversampling<4>,
    sample_rate: f32,

    osc_d: f32,
    osc_v: f32,
    distortion_amount: f32,
    distortion_threshold: f32,
    filter_cutoff: f32,
    filter_resonance: f32,
    filter_env_amount: f32,
    volume: f32,

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

        Self {
            oscillator: Oscillator::new(sample_rate),
            filter: MoogFilter::new(sample_rate),
            volume_env: Envelope::new(sample_rate),
            filter_env: Envelope::new(sample_rate),
            oversampling,
            sample_rate,

            osc_d: 0.5,
            osc_v: 0.5,
            distortion_amount: 0.0,
            distortion_threshold: 0.9,
            filter_cutoff: 1000.0,
            filter_resonance: 0.0,
            filter_env_amount: 0.0,
            volume: 0.8,

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

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        self.oscillator.set_sample_rate(sample_rate);
        self.filter.set_sample_rate(sample_rate);
        self.volume_env.set_sample_rate(sample_rate);
        self.filter_env.set_sample_rate(sample_rate);
        self.oversampling.set_sample_rate(sample_rate);
    }

    pub fn set_frequency(&mut self, freq: f32) {
        self.oscillator.set_frequency(freq);
    }

    pub fn set_osc_params(&mut self, d: f32, v: f32) {
        self.osc_d = d;
        self.osc_v = v;
        self.oscillator.set_params(d, v);
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

    pub fn is_active(&self) -> bool {
        self.volume_env.is_running() || self.filter_env.is_running()
    }

    pub fn process(&mut self) -> f32 {
        // Get envelope values at 1x rate (they're slow-moving control signals)
        let vol_env = self.volume_env.next();
        let filt_env = self.filter_env.next();

        // Calculate filter cutoff modulation at 1x rate
        let modulated_cutoff = self.filter_cutoff + (filt_env * self.filter_env_amount);
        let modulated_cutoff = modulated_cutoff.clamp(20.0, 20000.0);

        // Follow the exact pattern from synfx-dsp VPSOscillator documentation
        let overbuf = self.oversampling.resample_buffer();
        for sample in overbuf.iter_mut() {
            // Generate oscillator sample (called 4 times with base israte per the example)
            let osc_sample = self.oscillator.next(self.osc_d, self.osc_v);
            // Apply distortion as shown in the example (amount 0.0-1.0, threshold typically 0.9)
            // Note: amount parameter is u8, so we convert the 0.0-1.0 range to 0-1
            let distorted = apply_distortion(osc_sample, self.distortion_threshold, (self.distortion_amount.round() as u8).min(1));
            // Apply volume envelope
            let enveloped = distorted * vol_env;
            // Process through Moog filter
            *sample = self.filter.process(enveloped, modulated_cutoff, self.filter_resonance);
        }

        // Downsample using Butterworth filters (4-stage cascade)
        let output = self.oversampling.downsample();

        output * self.volume
    }
}
