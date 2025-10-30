use synfx_dsp::{Oversampling, f_distort};
use super::oscillator::{Oscillator, PolyBlepWrapper, PLLOscillator, PllMode};
use super::filter::MoogFilter;
use super::envelope::Envelope;

pub struct Voice {
    // ===== Oscillators =====
    vps_oscillator: Oscillator,
    polyblep_oscillator: PolyBlepWrapper,
    sub_oscillator_sine: PolyBlepWrapper,
    sub_oscillator_square: PolyBlepWrapper,
    pll_oscillator: PLLOscillator,
    pll_reference_oscillator: PolyBlepWrapper,

    // ===== Processing =====
    filter: MoogFilter,
    volume_envelope: Envelope,
    filter_envelope: Envelope,
    oversampling_left: Oversampling<4>,
    oversampling_right: Oversampling<4>,

    // ===== Global Parameters =====
    base_frequency: f32,
    master_volume: f32,
    sample_rate: f32,

    // ===== Stereo =====
    stereo_width: f32, // 0.0 = mono, 1.0 = full width

    // ===== VPS Oscillator =====
    vps_octave: i32,
    vps_d_param: f32,
    vps_v_param: f32,
    vps_volume: f32,

    // ===== PolyBLEP Pulse =====
    polyblep_volume: f32,
    polyblep_pulse_width: f32,
    polyblep_octave: i32,

    // ===== Sub Oscillator =====
    sub_volume: f32,
    sub_octave: i32,
    sub_shape: f32,

    // ===== PLL =====
    pll_volume: f32,
    pll_track_speed: f32,
    pll_damping: f32,
    pll_range: f32,
    pll_multiplier: f32,
    pll_feedback_amount: f32,
    pll_feedback_state: f32,
    pll_distortion_amount: f32,
    pll_distortion_threshold: f32,

    // ===== PLL Reference =====
    pll_ref_octave: i32,
    pll_ref_tune_semitones: i32,
    pll_ref_fine_tune_cents: f32,
    pll_ref_pulse_width: f32,

    // ===== Distortion =====
    distortion_amount: f32,
    distortion_threshold: f32,

    // ===== Filter =====
    filter_cutoff: f32,
    filter_resonance: f32,
    filter_envelope_amount: f32,
    filter_drive: f32,
    filter_mode: i32,

    // ===== Volume Envelope =====
    vol_env_attack: f32,
    vol_env_attack_shape: f32,
    vol_env_decay: f32,
    vol_env_decay_shape: f32,
    vol_env_sustain: f32,
    vol_env_release: f32,
    vol_env_release_shape: f32,

    // ===== Filter Envelope =====
    filt_env_attack: f32,
    filt_env_attack_shape: f32,
    filt_env_decay: f32,
    filt_env_decay_shape: f32,
    filt_env_sustain: f32,
    filt_env_release: f32,
    filt_env_release_shape: f32,
}

impl Voice {
    pub fn new(sample_rate: f32) -> Self {
        let mut oversampling_left = Oversampling::<4>::new();
        let mut oversampling_right = Oversampling::<4>::new();
        oversampling_left.set_sample_rate(sample_rate);
        oversampling_right.set_sample_rate(sample_rate);

        let oversampled_rate = sample_rate * 4.0;

        Self {
            vps_oscillator: Oscillator::new(oversampled_rate),
            polyblep_oscillator: PolyBlepWrapper::new(oversampled_rate),
            sub_oscillator_sine: PolyBlepWrapper::new(sample_rate),
            sub_oscillator_square: PolyBlepWrapper::new(sample_rate),
            pll_oscillator: PLLOscillator::new(oversampled_rate),
            pll_reference_oscillator: PolyBlepWrapper::new(oversampled_rate),

            filter: MoogFilter::new(oversampled_rate),
            volume_envelope: Envelope::new(sample_rate),
            filter_envelope: Envelope::new(sample_rate),
            oversampling_left,
            oversampling_right,

            base_frequency: 220.0,
            master_volume: 0.8,
            sample_rate,
            stereo_width: 0.0, // start mono-centered

            vps_octave: 0,
            vps_d_param: 0.5,
            vps_v_param: 0.5,
            vps_volume: 1.0,

            polyblep_volume: 0.0,
            polyblep_pulse_width: 0.5,
            polyblep_octave: 0,

            sub_volume: 0.0,
            sub_octave: -1,
            sub_shape: 0.0,

            pll_volume: 0.0,
            pll_track_speed: 0.2,
            pll_damping: 0.05,
            pll_range: 1.0,
            pll_multiplier: 1.0,
            pll_feedback_amount: 0.0,
            pll_feedback_state: 0.0,
            pll_distortion_amount: 0.0,
            pll_distortion_threshold: 0.9,

            pll_ref_octave: 0,
            pll_ref_tune_semitones: 0,
            pll_ref_fine_tune_cents: 0.0,
            pll_ref_pulse_width: 0.5,

            distortion_amount: 0.0,
            distortion_threshold: 0.9,

            filter_cutoff: 1000.0,
            filter_resonance: 0.0,
            filter_envelope_amount: 0.0,
            filter_drive: 1.0,
            filter_mode: 3,

            vol_env_attack: 1.0,
            vol_env_attack_shape: 0.5,
            vol_env_decay: 20.0,
            vol_env_decay_shape: 0.5,
            vol_env_sustain: 1.0,
            vol_env_release: 5.0,
            vol_env_release_shape: 0.5,

            filt_env_attack: 1.0,
            filt_env_attack_shape: 0.5,
            filt_env_decay: 20.0,
            filt_env_decay_shape: 0.5,
            filt_env_sustain: 1.0,
            filt_env_release: 5.0,
            filt_env_release_shape: 0.5,
        }
    }

    pub fn set_stereo_width(&mut self, width: f32) {
        self.stereo_width = width.clamp(0.0, 1.0);
    }

    pub fn set_frequency(&mut self, freq: f32, _pll_feedback: f32, feedback_amount: f32) {
        self.base_frequency = freq;
        self.pll_feedback_amount = feedback_amount;
    }

    pub fn set_osc_params(&mut self, d: f32, v: f32) {
        self.vps_d_param = d;
        self.vps_v_param = v;
        self.vps_oscillator.set_params(d, v);
    }

    pub fn set_osc_volume(&mut self, volume: f32) {
        self.vps_volume = volume;
    }

    pub fn set_osc_octave(&mut self, octave: i32) {
        self.vps_octave = octave;
    }

    pub fn set_polyblep_params(&mut self, volume: f32, pulse_width: f32, octave: i32) {
        self.polyblep_volume = volume;
        self.polyblep_pulse_width = pulse_width;
        self.polyblep_octave = octave;
    }

    pub fn set_sub_params(&mut self, volume: f32, octave: i32, shape: f32) {
        self.sub_volume = volume;
        self.sub_octave = octave;
        self.sub_shape = shape;
    }

    pub fn set_pll_ref_params(&mut self, octave: i32, tune: i32, fine_tune: f32, pulse_width: f32) {
        self.pll_ref_octave = octave;
        self.pll_ref_tune_semitones = tune;
        self.pll_ref_fine_tune_cents = fine_tune;
        self.pll_ref_pulse_width = pulse_width;
    }

    pub fn set_pll_params(&mut self, track: f32, damp: f32, mult: f32, range: f32, colored: bool, edge_mode: bool) {
        self.pll_track_speed = track;
        self.pll_damping = damp;
        self.pll_multiplier = mult;
        self.pll_range = range;

        let mode = if edge_mode { PllMode::EdgePFD } else { PllMode::AnalogLikePD };
        self.pll_oscillator.set_params(track, damp, mult, range, colored, mode);
    }

    pub fn set_pll_volume(&mut self, volume: f32) {
        self.pll_volume = volume;
    }

    pub fn set_pll_ki_multiplier(&mut self, ki_mult: f32) {
        self.pll_oscillator.set_ki_multiplier(ki_mult);
    }

    pub fn set_pll_distortion_params(&mut self, amount: f32, threshold: f32) {
        self.pll_distortion_amount = amount;
        self.pll_distortion_threshold = threshold;
    }

    pub fn set_distortion_params(&mut self, amount: f32, threshold: f32) {
        self.distortion_amount = amount;
        self.distortion_threshold = threshold;
    }

    pub fn set_filter_params(&mut self, cutoff: f32, resonance: f32, env_amount: f32, drive: f32, mode: i32) {
        self.filter_cutoff = cutoff;
        self.filter_resonance = resonance;
        self.filter_envelope_amount = env_amount;
        self.filter_drive = drive;
        self.filter_mode = mode;
    }

    pub fn set_volume(&mut self, volume: f32) {
        self.master_volume = volume;
    }

    pub fn set_volume_envelope(
        &mut self,
        attack: f32,
        attack_shape: f32,
        decay: f32,
        decay_shape: f32,
        sustain: f32,
        release: f32,
        release_shape: f32,
    ) {
        self.vol_env_attack = attack;
        self.vol_env_attack_shape = attack_shape;
        self.vol_env_decay = decay;
        self.vol_env_decay_shape = decay_shape;
        self.vol_env_sustain = sustain;
        self.vol_env_release = release;
        self.vol_env_release_shape = release_shape;
    }

    pub fn set_filter_envelope(
        &mut self,
        attack: f32,
        attack_shape: f32,
        decay: f32,
        decay_shape: f32,
        sustain: f32,
        release: f32,
        release_shape: f32,
    ) {
        self.filt_env_attack = attack;
        self.filt_env_attack_shape = attack_shape;
        self.filt_env_decay = decay;
        self.filt_env_decay_shape = decay_shape;
        self.filt_env_sustain = sustain;
        self.filt_env_release = release;
        self.filt_env_release_shape = release_shape;
    }

    pub fn trigger(&mut self) {
        self.volume_envelope.trigger(
            self.vol_env_attack,
            self.vol_env_attack_shape,
            self.vol_env_decay,
            self.vol_env_decay_shape,
            self.vol_env_sustain,
            self.vol_env_release,
            self.vol_env_release_shape,
        );

        self.filter_envelope.trigger(
            self.filt_env_attack,
            self.filt_env_attack_shape,
            self.filt_env_decay,
            self.filt_env_decay_shape,
            self.filt_env_sustain,
            self.filt_env_release,
            self.filt_env_release_shape,
        );

        self.pll_oscillator.trigger();
    }

    pub fn release(&mut self) {
        self.volume_envelope.release();
        self.filter_envelope.release();
    }


    pub fn process(&mut self, _pll_feedback: f32) -> (f32, f32) {
        let volume_env = self.volume_envelope.next();
        let filter_env = self.filter_envelope.next();

        let cutoff = (self.filter_cutoff + filter_env * self.filter_envelope_amount)
            .clamp(20.0, 20000.0);

        self.pll_oscillator.prepare_block();

        let buf_l = self.oversampling_left.resample_buffer();
        let buf_r = self.oversampling_right.resample_buffer();
        let mut feedback = self.pll_feedback_state;

        for i in 0..4 {
            let mut l = 0.0;
            let mut r = 0.0;

            // VPS
            if self.vps_volume > 0.0 {
                let base_freq = self.base_frequency * 2.0_f32.powi(self.vps_octave);
                // stereo detune based on width
                let detune = self.stereo_width * 0.002; // ~0.2%
                self.vps_oscillator.set_frequency(base_freq * (1.0 - detune));
                let raw_l = self.vps_oscillator.next(self.vps_d_param, self.vps_v_param);
                self.vps_oscillator.set_frequency(base_freq * (1.0 + detune));
                let raw_r = self.vps_oscillator.next(self.vps_d_param, self.vps_v_param);

                let distortion_gain = 0.1 + self.distortion_amount * 4.9;
                let left = f_distort(distortion_gain, self.distortion_threshold, raw_l);
                let right = f_distort(distortion_gain, self.distortion_threshold, raw_r);

                l += left * self.vps_volume * volume_env;
                r += right * self.vps_volume * volume_env;
            }

            // PolyBLEP
            if self.polyblep_volume > 0.0 {
                let base_freq = self.base_frequency * 2.0_f32.powi(self.polyblep_octave);
                let detune = self.stereo_width * 0.003;
                self.polyblep_oscillator.set_frequency(base_freq * (1.0 - detune));
                let raw_l = self.polyblep_oscillator.next(self.polyblep_pulse_width);
                self.polyblep_oscillator.set_frequency(base_freq * (1.0 + detune));
                let raw_r = self.polyblep_oscillator.next(self.polyblep_pulse_width);

                l += raw_l * self.polyblep_volume * volume_env;
                r += raw_r * self.polyblep_volume * volume_env;
            }

            // PLL
            if self.pll_volume > 0.0 {
                let tune_oct = (self.pll_ref_tune_semitones as f32 + self.pll_ref_fine_tune_cents) / 12.0;
                let ref_freq = self.base_frequency * 2.0_f32.powi(self.pll_ref_octave) * 2.0_f32.powf(tune_oct);

                let fb_mod = feedback * self.pll_feedback_amount * 5.0;
                let ref_mod = (ref_freq * (1.0 + fb_mod)).clamp(20.0, self.sample_rate * 2.0);

                self.pll_reference_oscillator.set_frequency(ref_mod);
                let ref_pulse = self.pll_reference_oscillator.next(self.pll_ref_pulse_width);
                let ref_phase = self.pll_reference_oscillator.get_phase();

                let pll_raw = self.pll_oscillator.next(ref_phase, ref_mod, ref_pulse);
                let pll_dist_gain = 0.1 + self.pll_distortion_amount * 4.9;
                let pll_out = f_distort(pll_dist_gain, self.pll_distortion_threshold, pll_raw);

                // simple L/R variance using stereo_width
                l += pll_out * self.pll_volume * volume_env * (1.0 - self.stereo_width * 0.2);
                r += pll_out * self.pll_volume * volume_env * (1.0 + self.stereo_width * 0.2);

                feedback = feedback * 0.9 + pll_raw * 0.1;
            }

            buf_l[i] = l;
            buf_r[i] = r;
        }

        // Stereo filter
        self.filter
            .process_stereo(buf_l, buf_r, cutoff, self.filter_resonance, self.filter_drive, self.filter_mode);

        self.pll_feedback_state = feedback;

        let out_l = self.oversampling_left.downsample();
        let out_r = self.oversampling_right.downsample();

        // Sub oscillator (mono center)
        let sub = if self.sub_volume > 0.0 {
            let f = self.base_frequency * 2.0_f32.powi(self.sub_octave);
            self.sub_oscillator_sine.set_frequency(f);
            self.sub_oscillator_square.set_frequency(f);
            let s = self.sub_oscillator_sine.next_sin();
            let q = self.sub_oscillator_square.next(0.5);
            (s * (1.0 - self.sub_shape) + q * self.sub_shape) * self.sub_volume * volume_env
        } else { 0.0 };

        let final_l = (out_l + sub) * self.master_volume;
        let final_r = (out_r + sub) * self.master_volume;

        (final_l, final_r)
    }
}
