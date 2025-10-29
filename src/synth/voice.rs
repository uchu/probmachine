use synfx_dsp::{Oversampling, f_distort};
use super::oscillator::{Oscillator, PolyBlepWrapper, PLLOscillator, PllMode};
use super::filter::MoogFilter;
use super::envelope::Envelope;

pub struct Voice {
    oscillator: Oscillator,
    polyblep: PolyBlepWrapper,
    pll: PLLOscillator,
    pll_ref_osc: PolyBlepWrapper,
    sub_osc: PolyBlepWrapper,
    filter: MoogFilter,
    volume_env: Envelope,
    filter_env: Envelope,
    oversampling: Oversampling<4>,

    base_freq: f32,
    pll_feedback_amount: f32,
    osc_octave: i32,
    osc_d: f32,
    osc_v: f32,
    osc_volume: f32,
    polyblep_volume: f32,
    polyblep_pulse_width: f32,
    polyblep_octave: i32,
    pll_ref_octave: i32,
    pll_ref_tune: i32,
    pll_ref_fine_tune: f32,
    pll_ref_pulse_width: f32,
    pll_track_speed: f32,
    pll_damping: f32,
    pll_range: f32,
    pll_mult: f32,
    pll_volume: f32,
    sub_volume: f32,
    sub_octave: i32,
    sub_frequency: f32,
    distortion_amount: f32,
    distortion_threshold: f32,
    filter_cutoff: f32,
    filter_resonance: f32,
    filter_env_amount: f32,
    volume: f32,
    sample_rate: f32,
    last_pll_fb: f32,

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
            pll: PLLOscillator::new(oversampled_rate),
            pll_ref_osc: PolyBlepWrapper::new(oversampled_rate),
            sub_osc: PolyBlepWrapper::new(sample_rate),
            filter: MoogFilter::new(oversampled_rate),
            volume_env: Envelope::new(sample_rate),
            filter_env: Envelope::new(sample_rate),
            oversampling,

            base_freq: 220.0,
            pll_feedback_amount: 0.0,
            osc_octave: 0,
            osc_d: 0.5,
            osc_v: 0.5,
            osc_volume: 1.0,
            polyblep_volume: 0.0,
            polyblep_pulse_width: 0.5,
            polyblep_octave: 0,
            pll_ref_octave: 0,
            pll_ref_tune: 0,
            pll_ref_fine_tune: 0.0,
            pll_ref_pulse_width: 0.5,
            pll_track_speed: 0.2,
            pll_damping: 0.05,
            pll_range: 1.0,
            pll_mult: 1.0,
            pll_volume: 0.0,
            sub_volume: 0.0,
            sub_octave: -1,
            sub_frequency: 110.0,
            distortion_amount: 0.0,
            distortion_threshold: 0.9,
            filter_cutoff: 1000.0,
            filter_resonance: 0.0,
            filter_env_amount: 0.0,
            volume: 0.8,
            sample_rate,
            last_pll_fb: 0.0,

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

    pub fn set_frequency(&mut self, freq: f32, pll_feedback: f32, feedback_amount: f32) {
        self.base_freq = freq;
        self.pll_feedback_amount = feedback_amount;

        let osc_freq_shifted = freq * 2.0_f32.powi(self.osc_octave);
        let modulated_freq = osc_freq_shifted * (1.0 + pll_feedback * feedback_amount);
        self.oscillator.set_frequency(modulated_freq);
        self.polyblep.set_frequency(modulated_freq);

        let tune_in_octaves = (self.pll_ref_tune as f32 + self.pll_ref_fine_tune) / 12.0;
        let pll_ref_freq = freq * 2.0_f32.powi(self.pll_ref_octave) * 2.0_f32.powf(tune_in_octaves);
        self.pll_ref_osc.set_frequency(pll_ref_freq);
        self.pll.set_frequency(pll_ref_freq);

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

    pub fn set_osc_octave(&mut self, octave: i32) {
        self.osc_octave = octave;
    }

    pub fn set_polyblep_params(&mut self, volume: f32, pulse_width: f32, octave: i32) {
        self.polyblep_volume = volume;
        self.polyblep_pulse_width = pulse_width;
        self.polyblep_octave = octave;
    }

    pub fn set_pll_ref_params(&mut self, octave: i32, tune: i32, fine_tune: f32, pulse_width: f32) {
        self.pll_ref_octave = octave;
        self.pll_ref_tune = tune;
        self.pll_ref_fine_tune = fine_tune;
        self.pll_ref_pulse_width = pulse_width;
    }

    pub fn set_pll_params(&mut self, track: f32, damp: f32, mult: f32, range: f32, colored: bool, edge_mode: bool) {
        self.pll_track_speed = track;
        self.pll_damping = damp;
        self.pll_mult = mult;
        self.pll_range = range;
        let mode = if edge_mode { PllMode::EdgePFD } else { PllMode::AnalogLikePD };
        self.pll.set_params(track, damp, mult, range, colored, mode);
    }

    pub fn set_pll_volume(&mut self, volume: f32) {
        self.pll_volume = volume;
    }

    pub fn set_pll_ki_multiplier(&mut self, ki_mult: f32) {
        self.pll.set_ki_multiplier(ki_mult);
    }

    pub fn set_sub_params(&mut self, volume: f32, octave: i32) {
        self.sub_volume = volume;
        self.sub_octave = octave;
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

        self.pll.trigger();
    }

    pub fn release(&mut self) {
        self.volume_env.release();
        self.filter_env.release();
    }

    pub fn process(&mut self, _pll_feedback: f32) -> (f32, f32) {
        let vol_env = self.volume_env.next();
        let filt_env = self.filter_env.next();

        let modulated_cutoff = self.filter_cutoff + (filt_env * self.filter_env_amount);
        let modulated_cutoff = modulated_cutoff.clamp(20.0, 20000.0);

        self.pll.prepare_block();

        let overbuf = self.oversampling.resample_buffer();
        let mut fb_state = self.last_pll_fb;

        for sample in overbuf.iter_mut() {
            let osc_freq_shifted = self.base_freq * 2.0_f32.powi(self.osc_octave);
            self.oscillator.set_frequency(osc_freq_shifted);

            let polyblep_freq_shifted = self.base_freq * 2.0_f32.powi(self.polyblep_octave);
            self.polyblep.set_frequency(polyblep_freq_shifted);

            let osc_sample = self.oscillator.next(self.osc_d, self.osc_v);

            let gain = 0.1 + self.distortion_amount * 4.9;
            let distorted = f_distort(gain, self.distortion_threshold, osc_sample);
            let enveloped = distorted * self.osc_volume * vol_env;

            let ref_pulse = if self.polyblep_volume > 0.0 {
                self.polyblep.next(self.polyblep_pulse_width)
            } else {
                0.0
            };
            let polyblep_enveloped = ref_pulse * self.polyblep_volume * vol_env;

            let fb_mod = fb_state * self.pll_feedback_amount * 5.0;
            let tune_in_octaves = (self.pll_ref_tune as f32 + self.pll_ref_fine_tune) / 12.0;
            let pll_ref_freq = self.base_freq * 2.0_f32.powi(self.pll_ref_octave) * 2.0_f32.powf(tune_in_octaves);
            let pll_ref_freq_modulated = (pll_ref_freq * (1.0 + fb_mod)).clamp(20.0, self.sample_rate * 2.0);
            self.pll_ref_osc.set_frequency(pll_ref_freq_modulated);

            let pll_ref_pulse = self.pll_ref_osc.next(self.pll_ref_pulse_width);
            let ref_phase = self.pll_ref_osc.get_phase();

            let pll_sample = if self.pll_volume > 0.0 {
                self.pll.next(ref_phase, pll_ref_freq_modulated, pll_ref_pulse)
            } else {
                0.0
            };
            let pll_enveloped = pll_sample * self.pll_volume * vol_env / 4.0;

            fb_state = fb_state * 0.9 + pll_sample * 0.1;

            *sample = enveloped + polyblep_enveloped + pll_enveloped;
            //*sample = enveloped + pll_enveloped;
        }

        self.filter.process_buffer(overbuf, modulated_cutoff, self.filter_resonance);

        self.last_pll_fb = fb_state;

        let main_output = self.oversampling.downsample();

        let sub_sample = if self.sub_volume > 0.0 {
            let sub_freq = self.base_freq * 2.0_f32.powi(self.sub_octave);
            self.sub_osc.set_frequency(sub_freq);
            self.sub_osc.next_sin() * self.sub_volume * vol_env * 2.0
        } else {
            0.0
        };

        let output = (main_output + sub_sample) * self.volume;
        (output, self.last_pll_fb)
    }
}
