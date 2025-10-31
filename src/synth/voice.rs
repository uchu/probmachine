use synfx_dsp::{Oversampling, f_distort};
use super::oscillator::{Oscillator, PolyBlepWrapper, PLLOscillator, PllMode};
use super::filter::MoogFilter;
use super::envelope::Envelope;
use super::reverb::StereoReverb;

pub struct Voice {
    // ===== Oscillators =====
    vps_oscillator_left: Oscillator,
    vps_oscillator_right: Oscillator,
    polyblep_oscillator_left: PolyBlepWrapper,
    polyblep_oscillator_right: PolyBlepWrapper,
    sub_oscillator_sine: PolyBlepWrapper,
    sub_oscillator_square: PolyBlepWrapper,
    pll_oscillator_left: PLLOscillator,
    pll_oscillator_right: PLLOscillator,
    pll_reference_oscillator: PolyBlepWrapper,

    // ===== Processing =====
    filter_left: MoogFilter,
    filter_right: MoogFilter,
    volume_envelope: Envelope,
    filter_envelope: Envelope,
    oversampling_left: Oversampling<4>,
    oversampling_right: Oversampling<4>,

    // ===== Global Parameters =====
    base_frequency: f64,
    master_volume: f64,
    sample_rate: f64,

    // ===== Stereo =====
    stereo_width: f64,
    pll_damping_stereo_offset: f64,

    // ===== VPS Oscillator =====
    vps_octave: i32,
    vps_d_param: f64,
    vps_v_param: f64,
    vps_volume: f64,
    vps_stereo_v_offset: f64,

    // ===== PolyBLEP Pulse =====
    polyblep_volume: f64,
    polyblep_pulse_width: f64,
    polyblep_octave: i32,
    polyblep_stereo_width: f64,

    // ===== Sub Oscillator =====
    sub_volume: f64,
    sub_octave: i32,
    sub_shape: f64,

    // ===== PLL =====
    pll_volume: f64,
    pll_track_speed: f64,
    pll_damping: f64,
    pll_range: f64,
    pll_multiplier: f64,
    pll_feedback_amount: f64,
    pll_feedback_state: f64,
    pll_distortion_amount: f64,
    pll_distortion_threshold: f64,
    pll_mode_is_edge: bool,
    pll_colored: bool,

    // ===== PLL Reference =====
    pll_ref_octave: i32,
    pll_ref_tune_semitones: i32,
    pll_ref_fine_tune_cents: f64,
    pll_ref_pulse_width: f64,

    // ===== Distortion =====
    distortion_amount: f64,
    distortion_threshold: f64,

    // ===== Filter =====
    filter_enabled: bool,
    filter_cutoff: f64,
    filter_resonance: f64,
    filter_envelope_amount: f64,
    filter_drive: f64,
    filter_mode: i32,

    // ===== Volume Envelope =====
    vol_env_attack: f64,
    vol_env_attack_shape: f64,
    vol_env_decay: f64,
    vol_env_decay_shape: f64,
    vol_env_sustain: f64,
    vol_env_release: f64,
    vol_env_release_shape: f64,

    // ===== Filter Envelope =====
    filt_env_attack: f64,
    filt_env_attack_shape: f64,
    filt_env_decay: f64,
    filt_env_decay_shape: f64,
    filt_env_sustain: f64,
    filt_env_release: f64,
    filt_env_release_shape: f64,

    // ===== Reverb =====
    reverb: StereoReverb,
}

impl Voice {
    pub fn new(sample_rate: f32) -> Self {
        let mut oversampling_left = Oversampling::<4>::new();
        let mut oversampling_right = Oversampling::<4>::new();
        oversampling_left.set_sample_rate(sample_rate);
        oversampling_right.set_sample_rate(sample_rate);

        let sample_rate_f64 = sample_rate as f64;
        let oversampled_rate = sample_rate_f64 * 4.0;

        Self {
            vps_oscillator_left: Oscillator::new(oversampled_rate),
            vps_oscillator_right: Oscillator::new(oversampled_rate),
            polyblep_oscillator_left: PolyBlepWrapper::new(oversampled_rate),
            polyblep_oscillator_right: PolyBlepWrapper::new(oversampled_rate),
            sub_oscillator_sine: PolyBlepWrapper::new(oversampled_rate),
            sub_oscillator_square: PolyBlepWrapper::new(oversampled_rate),
            pll_oscillator_left: PLLOscillator::new(oversampled_rate),
            pll_oscillator_right: PLLOscillator::new(oversampled_rate),
            pll_reference_oscillator: PolyBlepWrapper::new(oversampled_rate),

            filter_left: MoogFilter::new(oversampled_rate),
            filter_right: MoogFilter::new(oversampled_rate),
            volume_envelope: Envelope::new(sample_rate_f64),
            filter_envelope: Envelope::new(sample_rate_f64),
            oversampling_left,
            oversampling_right,

            base_frequency: 220.0,
            master_volume: 0.8,
            sample_rate: sample_rate_f64,
            stereo_width: 0.0,
            pll_damping_stereo_offset: 0.0,

            vps_octave: 0,
            vps_d_param: 0.5,
            vps_v_param: 0.5,
            vps_volume: 1.0,
            vps_stereo_v_offset: 0.0,

            polyblep_volume: 0.0,
            polyblep_pulse_width: 0.5,
            polyblep_octave: 0,
            polyblep_stereo_width: 0.0,

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
            pll_mode_is_edge: false,
            pll_colored: false,

            pll_ref_octave: 0,
            pll_ref_tune_semitones: 0,
            pll_ref_fine_tune_cents: 0.0,
            pll_ref_pulse_width: 0.5,

            distortion_amount: 0.0,
            distortion_threshold: 0.9,

            filter_enabled: true,
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

            reverb: StereoReverb::new(oversampled_rate as f32),
        }
    }

    pub fn set_pll_stereo_damp_offset(&mut self, offset: f64) {
        self.pll_damping_stereo_offset = offset.clamp(0.0, 1.0);
    }

    pub fn set_frequency(&mut self, freq: f64, _pll_feedback: f64, feedback_amount: f64) {
        self.base_frequency = freq;
        self.pll_feedback_amount = feedback_amount;
    }

    pub fn set_osc_params(&mut self, d: f64, v: f64) {
        self.vps_d_param = d;
        self.vps_v_param = v;
        self.vps_oscillator_left.set_params(d, v);
        self.vps_oscillator_right.set_params(d, v);
    }

    pub fn set_vps_stereo_v_offset(&mut self, offset: f64) {
        self.vps_stereo_v_offset = offset.clamp(0.0, 1.0);
    }

    pub fn set_osc_volume(&mut self, volume: f64) {
        self.vps_volume = volume;
    }

    pub fn set_osc_octave(&mut self, octave: i32) {
        self.vps_octave = octave;
    }

    pub fn set_polyblep_params(&mut self, volume: f64, pulse_width: f64, octave: i32) {
        self.polyblep_volume = volume;
        self.polyblep_pulse_width = pulse_width;
        self.polyblep_octave = octave;
    }

    pub fn set_polyblep_stereo_width(&mut self, width: f64) {
        self.polyblep_stereo_width = width.clamp(0.0, 1.0);
    }

    pub fn set_sub_params(&mut self, volume: f64, octave: i32, shape: f64) {
        self.sub_volume = volume;
        self.sub_octave = octave;
        self.sub_shape = shape;
    }

    pub fn set_pll_ref_params(&mut self, octave: i32, tune: i32, fine_tune: f64, pulse_width: f64) {
        self.pll_ref_octave = octave;
        self.pll_ref_tune_semitones = tune;
        self.pll_ref_fine_tune_cents = fine_tune;
        self.pll_ref_pulse_width = pulse_width;
    }

    pub fn set_pll_params(&mut self, track: f64, damp: f64, mult: f64, range: f64, colored: bool, edge_mode: bool) {
        self.pll_track_speed = track;
        self.pll_damping = damp;
        self.pll_multiplier = mult;
        self.pll_range = range;
        self.pll_mode_is_edge = edge_mode;
        self.pll_colored = colored;

        let mode = if edge_mode { PllMode::EdgePFD } else { PllMode::AnalogLikePD };
        self.pll_oscillator_left.set_params(track, damp, mult, range, colored, mode);
        self.pll_oscillator_right.set_params(track, damp, mult, range, colored, mode);
    }

    pub fn set_pll_volume(&mut self, volume: f64) {
        self.pll_volume = volume;
    }

    pub fn set_pll_ki_multiplier(&mut self, ki_mult: f64) {
        self.pll_oscillator_left.set_ki_multiplier(ki_mult);
        self.pll_oscillator_right.set_ki_multiplier(ki_mult);
    }

    pub fn set_pll_distortion_params(&mut self, amount: f64, threshold: f64) {
        self.pll_distortion_amount = amount;
        self.pll_distortion_threshold = threshold;
    }

    pub fn set_distortion_params(&mut self, amount: f64, threshold: f64) {
        self.distortion_amount = amount;
        self.distortion_threshold = threshold;
    }

    pub fn set_filter_params(&mut self, enabled: bool, cutoff: f64, resonance: f64, env_amount: f64, drive: f64, mode: i32) {
        self.filter_enabled = enabled;
        self.filter_cutoff = cutoff;
        self.filter_resonance = resonance;
        self.filter_envelope_amount = env_amount;
        self.filter_drive = drive;
        self.filter_mode = mode;
    }

    pub fn set_reverb_params(
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
    ) {
        self.reverb.set_params(
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

    pub fn set_volume(&mut self, volume: f64) {
        self.master_volume = volume;
    }

    pub fn set_volume_envelope(
        &mut self,
        attack: f64,
        attack_shape: f64,
        decay: f64,
        decay_shape: f64,
        sustain: f64,
        release: f64,
        release_shape: f64,
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
        attack: f64,
        attack_shape: f64,
        decay: f64,
        decay_shape: f64,
        sustain: f64,
        release: f64,
        release_shape: f64,
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

        self.pll_oscillator_left.trigger();
        self.pll_oscillator_right.trigger();
    }

    pub fn release(&mut self) {
        self.volume_envelope.release();
        self.filter_envelope.release();
    }


    pub fn process(&mut self, _pll_feedback: f64) -> (f64, f64) {
        let volume_env = self.volume_envelope.next();
        let filter_env = self.filter_envelope.next();

        let cutoff = (self.filter_cutoff + filter_env * self.filter_envelope_amount)
            .clamp(20.0, 20000.0);

        self.pll_oscillator_left.prepare_block();
        self.pll_oscillator_right.prepare_block();

        let buf_l = self.oversampling_left.resample_buffer();
        let buf_r = self.oversampling_right.resample_buffer();
        let mut feedback = self.pll_feedback_state;

        for i in 0..4 {
            let mut mixed_oscillators_l = 0.0_f64;
            let mut mixed_oscillators_r = 0.0_f64;

            // VPS Oscillators
            if self.vps_volume > 0.0 {
                let base_freq = self.base_frequency * 2.0_f64.powi(self.vps_octave);
                let detune = self.stereo_width * 0.002;

                let v_left = (self.vps_v_param - self.vps_stereo_v_offset).clamp(0.0, 1.0);
                let v_right = (self.vps_v_param + self.vps_stereo_v_offset).clamp(0.0, 1.0);

                self.vps_oscillator_left.set_frequency(base_freq * (1.0 - detune));
                let raw_l = self.vps_oscillator_left.next(self.vps_d_param, v_left);

                self.vps_oscillator_right.set_frequency(base_freq * (1.0 + detune));
                let raw_r = self.vps_oscillator_right.next(self.vps_d_param, v_right);

                let distortion_gain = 0.1 + self.distortion_amount * 4.9;
                let left = f_distort(distortion_gain as f32, self.distortion_threshold as f32, raw_l as f32) as f64;
                let right = f_distort(distortion_gain as f32, self.distortion_threshold as f32, raw_r as f32) as f64;

                mixed_oscillators_l += left * self.vps_volume * volume_env;
                mixed_oscillators_r += right * self.vps_volume * volume_env;
            }

            // PolyBLEP Oscillators
            if self.polyblep_volume > 0.0 {
                let base_freq = self.base_frequency * 2.0_f64.powi(self.polyblep_octave);

                let detune_amount = self.polyblep_stereo_width * 0.01;
                let pw_offset = self.polyblep_stereo_width * 0.15;

                let pw_left = (self.polyblep_pulse_width - pw_offset).clamp(0.01, 0.99);
                let pw_right = (self.polyblep_pulse_width + pw_offset).clamp(0.01, 0.99);

                self.polyblep_oscillator_left.set_frequency(base_freq * (1.0 - detune_amount));
                let raw_l = self.polyblep_oscillator_left.next(pw_left);

                self.polyblep_oscillator_right.set_frequency(base_freq * (1.0 + detune_amount));
                let raw_r = self.polyblep_oscillator_right.next(pw_right);

                mixed_oscillators_l += raw_l * self.polyblep_volume * volume_env;
                mixed_oscillators_r += raw_r * self.polyblep_volume * volume_env;
            }

            // PLL Oscillators
            if self.pll_volume > 0.0 {
                let tune_oct = (self.pll_ref_tune_semitones as f64 + self.pll_ref_fine_tune_cents) / 12.0;
                let ref_freq = self.base_frequency * 2.0_f64.powi(self.pll_ref_octave) * 2.0_f64.powf(tune_oct);

                let fb_mod = feedback * self.pll_feedback_amount * 5.0;
                let ref_mod = (ref_freq * (1.0 + fb_mod)).clamp(20.0, self.sample_rate * 2.0);

                self.pll_reference_oscillator.set_frequency(ref_mod);
                let ref_pulse = self.pll_reference_oscillator.next(self.pll_ref_pulse_width);
                let ref_phase = self.pll_reference_oscillator.get_phase();

                let damp_left = (self.pll_damping - self.pll_damping_stereo_offset).clamp(0.001, 1.0);
                let damp_right = (self.pll_damping + self.pll_damping_stereo_offset).clamp(0.001, 1.0);

                let mode = if self.pll_mode_is_edge {
                    super::oscillator::PllMode::EdgePFD
                } else {
                    super::oscillator::PllMode::AnalogLikePD
                };

                self.pll_oscillator_left.set_params(self.pll_track_speed, damp_left, self.pll_multiplier, self.pll_range, self.pll_colored, mode);
                let pll_raw_l = self.pll_oscillator_left.next(ref_phase, ref_mod, ref_pulse);

                self.pll_oscillator_right.set_params(self.pll_track_speed, damp_right, self.pll_multiplier, self.pll_range, self.pll_colored, mode);
                let pll_raw_r = self.pll_oscillator_right.next(ref_phase, ref_mod, ref_pulse);

                let pll_dist_gain = 0.1 + self.pll_distortion_amount * 4.9;
                let pll_out_l = f_distort(pll_dist_gain as f32, self.pll_distortion_threshold as f32, pll_raw_l as f32) as f64;
                let pll_out_r = f_distort(pll_dist_gain as f32, self.pll_distortion_threshold as f32, pll_raw_r as f32) as f64;

                mixed_oscillators_l += pll_out_l * self.pll_volume * volume_env;
                mixed_oscillators_r += pll_out_r * self.pll_volume * volume_env;

                feedback = feedback * 0.9 + (pll_raw_l + pll_raw_r) * 0.05;
            }

            buf_l[i] = mixed_oscillators_l as f32;
            buf_r[i] = mixed_oscillators_r as f32;
        }

        // Apply filter at oversampled rate (only if enabled)
        if self.filter_enabled {
            self.filter_left.process_buffer(
                unsafe { &mut *(buf_l.as_mut_ptr() as *mut [f32; 4]) },
                cutoff as f32,
                self.filter_resonance as f32,
                self.filter_drive as f32,
                self.filter_mode,
            );
            self.filter_right.process_buffer(
                unsafe { &mut *(buf_r.as_mut_ptr() as *mut [f32; 4]) },
                cutoff as f32,
                self.filter_resonance as f32,
                self.filter_drive as f32,
                self.filter_mode,
            );
        }

        // Apply reverb at oversampled rate (processes the buffer in-place with dry/wet mix)
        if self.reverb.mix > 0.0 {
            for i in 0..4 {
                let dry_l = buf_l[i] as f64;
                let dry_r = buf_r[i] as f64;

                let (wet_l, wet_r) = self.reverb.process(dry_l, dry_r);

                buf_l[i] = (dry_l * (1.0 - self.reverb.mix) + wet_l * self.reverb.mix) as f32;
                buf_r[i] = (dry_r * (1.0 - self.reverb.mix) + wet_r * self.reverb.mix) as f32;
            }
        }

        // Add sub oscillator at oversampled rate (after reverb, so it's never processed through reverb)
        if self.sub_volume > 0.0 {
            let sub_freq = self.base_frequency * 2.0_f64.powi(self.sub_octave);
            for i in 0..4 {
                self.sub_oscillator_sine.set_frequency(sub_freq);
                self.sub_oscillator_square.set_frequency(sub_freq);
                let sine_sample = self.sub_oscillator_sine.next_sin();
                let square_sample = self.sub_oscillator_square.next(0.5);
                let sub_sample = (sine_sample * (1.0 - self.sub_shape) + square_sample * self.sub_shape)
                    * self.sub_volume * volume_env;

                buf_l[i] += sub_sample as f32;
                buf_r[i] += sub_sample as f32;
            }
        }

        self.pll_feedback_state = feedback;

        // Downsample the complete signal
        let downsampled_l = self.oversampling_left.downsample() as f64;
        let downsampled_r = self.oversampling_right.downsample() as f64;

        // Apply master volume
        let final_l = downsampled_l * self.master_volume;
        let final_r = downsampled_r * self.master_volume;

        (final_l, final_r)
    }
}
