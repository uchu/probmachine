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
    oversampling: Oversampling<4>,

    // ===== Global Parameters =====
    base_frequency: f32,
    master_volume: f32,
    sample_rate: f32,

    // ===== VPS Oscillator Parameters =====
    vps_octave: i32,
    vps_d_param: f32,
    vps_v_param: f32,
    vps_volume: f32,

    // ===== PolyBLEP Pulse Oscillator Parameters =====
    polyblep_volume: f32,
    polyblep_pulse_width: f32,
    polyblep_octave: i32,

    // ===== Sub Oscillator Parameters =====
    sub_volume: f32,
    sub_octave: i32,
    sub_shape: f32,

    // ===== PLL Parameters =====
    pll_volume: f32,
    pll_track_speed: f32,
    pll_damping: f32,
    pll_range: f32,
    pll_multiplier: f32,
    pll_feedback_amount: f32,
    pll_feedback_state: f32,
    pll_distortion_amount: f32,
    pll_distortion_threshold: f32,

    // ===== PLL Reference Oscillator Parameters =====
    pll_ref_octave: i32,
    pll_ref_tune_semitones: i32,
    pll_ref_fine_tune_cents: f32,
    pll_ref_pulse_width: f32,

    // ===== Distortion Parameters =====
    distortion_amount: f32,
    distortion_threshold: f32,

    // ===== Filter Parameters =====
    filter_cutoff: f32,
    filter_resonance: f32,
    filter_envelope_amount: f32,
    filter_drive: f32,
    filter_mode: i32,

    // ===== Volume Envelope Parameters =====
    vol_env_attack: f32,
    vol_env_attack_shape: f32,
    vol_env_decay: f32,
    vol_env_decay_shape: f32,
    vol_env_sustain: f32,
    vol_env_release: f32,
    vol_env_release_shape: f32,

    // ===== Filter Envelope Parameters =====
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
        let mut oversampling = Oversampling::<4>::new();
        oversampling.set_sample_rate(sample_rate);

        let oversampled_rate = sample_rate * 4.0;

        Self {
            // Oscillators
            vps_oscillator: Oscillator::new(oversampled_rate),
            polyblep_oscillator: PolyBlepWrapper::new(oversampled_rate),
            sub_oscillator_sine: PolyBlepWrapper::new(sample_rate),
            sub_oscillator_square: PolyBlepWrapper::new(sample_rate),
            pll_oscillator: PLLOscillator::new(oversampled_rate),
            pll_reference_oscillator: PolyBlepWrapper::new(oversampled_rate),

            // Processing
            filter: MoogFilter::new(oversampled_rate),
            volume_envelope: Envelope::new(sample_rate),
            filter_envelope: Envelope::new(sample_rate),
            oversampling,

            // Global
            base_frequency: 220.0,
            master_volume: 0.8,
            sample_rate,

            // VPS Oscillator
            vps_octave: 0,
            vps_d_param: 0.5,
            vps_v_param: 0.5,
            vps_volume: 1.0,

            // PolyBLEP Pulse
            polyblep_volume: 0.0,
            polyblep_pulse_width: 0.5,
            polyblep_octave: 0,

            // Sub Oscillator
            sub_volume: 0.0,
            sub_octave: -1,
            sub_shape: 0.0,

            // PLL
            pll_volume: 0.0,
            pll_track_speed: 0.2,
            pll_damping: 0.05,
            pll_range: 1.0,
            pll_multiplier: 1.0,
            pll_feedback_amount: 0.0,
            pll_feedback_state: 0.0,
            pll_distortion_amount: 0.0,
            pll_distortion_threshold: 0.9,

            // PLL Reference
            pll_ref_octave: 0,
            pll_ref_tune_semitones: 0,
            pll_ref_fine_tune_cents: 0.0,
            pll_ref_pulse_width: 0.5,

            // Distortion
            distortion_amount: 0.0,
            distortion_threshold: 0.9,

            // Filter
            filter_cutoff: 1000.0,
            filter_resonance: 0.0,
            filter_envelope_amount: 0.0,
            filter_drive: 1.0,
            filter_mode: 3,

            // Volume Envelope
            vol_env_attack: 1.0,
            vol_env_attack_shape: 0.5,
            vol_env_decay: 20.0,
            vol_env_decay_shape: 0.5,
            vol_env_sustain: 1.0,
            vol_env_release: 5.0,
            vol_env_release_shape: 0.5,

            // Filter Envelope
            filt_env_attack: 1.0,
            filt_env_attack_shape: 0.5,
            filt_env_decay: 20.0,
            filt_env_decay_shape: 0.5,
            filt_env_sustain: 1.0,
            filt_env_release: 5.0,
            filt_env_release_shape: 0.5,
        }
    }

    // ===== Frequency Control =====

    pub fn set_frequency(&mut self, freq: f32, _pll_feedback: f32, feedback_amount: f32) {
        self.base_frequency = freq;
        self.pll_feedback_amount = feedback_amount;
    }

    // ===== VPS Oscillator Setters =====

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

    // ===== PolyBLEP Pulse Oscillator Setters =====

    pub fn set_polyblep_params(&mut self, volume: f32, pulse_width: f32, octave: i32) {
        self.polyblep_volume = volume;
        self.polyblep_pulse_width = pulse_width;
        self.polyblep_octave = octave;
    }

    // ===== Sub Oscillator Setters =====

    pub fn set_sub_params(&mut self, volume: f32, octave: i32, shape: f32) {
        self.sub_volume = volume;
        self.sub_octave = octave;
        self.sub_shape = shape;
    }

    // ===== PLL Reference Oscillator Setters =====

    pub fn set_pll_ref_params(&mut self, octave: i32, tune: i32, fine_tune: f32, pulse_width: f32) {
        self.pll_ref_octave = octave;
        self.pll_ref_tune_semitones = tune;
        self.pll_ref_fine_tune_cents = fine_tune;
        self.pll_ref_pulse_width = pulse_width;
    }

    // ===== PLL Oscillator Setters =====

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

    // ===== Distortion Setters =====

    pub fn set_distortion_params(&mut self, amount: f32, threshold: f32) {
        self.distortion_amount = amount;
        self.distortion_threshold = threshold;
    }

    // ===== Filter Setters =====

    pub fn set_filter_params(&mut self, cutoff: f32, resonance: f32, env_amount: f32, drive: f32, mode: i32) {
        self.filter_cutoff = cutoff;
        self.filter_resonance = resonance;
        self.filter_envelope_amount = env_amount;
        self.filter_drive = drive;
        self.filter_mode = mode;
    }

    // ===== Master Volume =====

    pub fn set_volume(&mut self, volume: f32) {
        self.master_volume = volume;
    }

    // ===== Envelope Setters =====

    pub fn set_volume_envelope(&mut self, attack: f32, attack_shape: f32, decay: f32, decay_shape: f32, sustain: f32, release: f32, release_shape: f32) {
        self.vol_env_attack = attack;
        self.vol_env_attack_shape = attack_shape;
        self.vol_env_decay = decay;
        self.vol_env_decay_shape = decay_shape;
        self.vol_env_sustain = sustain;
        self.vol_env_release = release;
        self.vol_env_release_shape = release_shape;
    }

    pub fn set_filter_envelope(&mut self, attack: f32, attack_shape: f32, decay: f32, decay_shape: f32, sustain: f32, release: f32, release_shape: f32) {
        self.filt_env_attack = attack;
        self.filt_env_attack_shape = attack_shape;
        self.filt_env_decay = decay;
        self.filt_env_decay_shape = decay_shape;
        self.filt_env_sustain = sustain;
        self.filt_env_release = release;
        self.filt_env_release_shape = release_shape;
    }

    // ===== Trigger & Release =====

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

    // ===== Audio Processing =====

    pub fn process(&mut self, _pll_feedback: f32) -> (f32, f32) {
        // Get envelope values
        let volume_env_value = self.volume_envelope.next();
        let filter_env_value = self.filter_envelope.next();

        // Calculate filter cutoff with envelope modulation
        let modulated_filter_cutoff = (self.filter_cutoff + filter_env_value * self.filter_envelope_amount)
            .clamp(20.0, 20000.0);

        // Prepare PLL for this block
        self.pll_oscillator.prepare_block();

        // Get oversampling buffer
        let oversample_buffer = self.oversampling.resample_buffer();
        let mut feedback_state = self.pll_feedback_state;

        // Process each oversampled sample
        for sample_output in oversample_buffer.iter_mut() {
            let mut mixed_signal = 0.0;

            // ========== VPS Oscillator ==========
            if self.vps_volume > 0.0 {
                let vps_frequency = self.base_frequency * 2.0_f32.powi(self.vps_octave);
                self.vps_oscillator.set_frequency(vps_frequency);

                let vps_raw = self.vps_oscillator.next(self.vps_d_param, self.vps_v_param);

                // Apply distortion
                let distortion_gain = 0.1 + self.distortion_amount * 4.9;
                let vps_distorted = f_distort(distortion_gain, self.distortion_threshold, vps_raw);

                let vps_output = vps_distorted * self.vps_volume * volume_env_value;
                mixed_signal += vps_output;
            }

            // ========== PolyBLEP Pulse Oscillator ==========
            if self.polyblep_volume > 0.0 {
                let polyblep_frequency = self.base_frequency * 2.0_f32.powi(self.polyblep_octave);
                self.polyblep_oscillator.set_frequency(polyblep_frequency);

                let polyblep_raw = self.polyblep_oscillator.next(self.polyblep_pulse_width);
                let polyblep_output = polyblep_raw * self.polyblep_volume * volume_env_value;
                mixed_signal += polyblep_output;
            }

            // ========== PLL System ==========
            if self.pll_volume > 0.0 {
                // Calculate PLL reference frequency with tuning
                let tune_in_octaves = (self.pll_ref_tune_semitones as f32 + self.pll_ref_fine_tune_cents) / 12.0;
                let pll_ref_base_freq = self.base_frequency
                    * 2.0_f32.powi(self.pll_ref_octave)
                    * 2.0_f32.powf(tune_in_octaves);

                // Apply feedback modulation to reference frequency
                let feedback_modulation = feedback_state * self.pll_feedback_amount * 5.0;
                let pll_ref_modulated_freq = (pll_ref_base_freq * (1.0 + feedback_modulation))
                    .clamp(20.0, self.sample_rate * 2.0);

                self.pll_reference_oscillator.set_frequency(pll_ref_modulated_freq);

                // Generate reference pulse and phase
                let pll_ref_pulse = self.pll_reference_oscillator.next(self.pll_ref_pulse_width);
                let pll_ref_phase = self.pll_reference_oscillator.get_phase();

                // Process PLL tracking
                let pll_raw = self.pll_oscillator.next(pll_ref_phase, pll_ref_modulated_freq, pll_ref_pulse);

                // Apply distortion to PLL output
                let pll_distortion_gain = 0.1 + self.pll_distortion_amount * 4.9;
                let pll_distorted = f_distort(pll_distortion_gain, self.pll_distortion_threshold, pll_raw);

                let pll_output = pll_distorted * self.pll_volume * volume_env_value / 4.0;
                mixed_signal += pll_output;

                // Update feedback state with low-pass filter
                feedback_state = feedback_state * 0.9 + pll_raw * 0.1;
            }

            *sample_output = mixed_signal;
        }

        // Apply filter to oversampled buffer
        self.filter.process_buffer(oversample_buffer, modulated_filter_cutoff, self.filter_resonance, self.filter_drive, self.filter_mode);

        // Store feedback state for next block
        self.pll_feedback_state = feedback_state;

        // Downsample to output rate
        let main_output = self.oversampling.downsample();

        // ========== Sub Oscillator (not oversampled) ==========
        let sub_output = if self.sub_volume > 0.0 {
            let sub_frequency = self.base_frequency * 2.0_f32.powi(self.sub_octave);

            // Set frequency on both oscillators to keep them in sync
            self.sub_oscillator_sine.set_frequency(sub_frequency);
            self.sub_oscillator_square.set_frequency(sub_frequency);

            // Generate both waveforms (each advances its own phase)
            let sub_sine = self.sub_oscillator_sine.next_sin();
            let sub_square = self.sub_oscillator_square.next(0.5);

            // Morph between sine (0.0) and square (1.0)
            let sub_morphed = sub_sine * (1.0 - self.sub_shape) + sub_square * self.sub_shape;

            sub_morphed * self.sub_volume * volume_env_value * 2.0
        } else {
            0.0
        };

        // Final mix and master volume
        let final_output = (main_output + sub_output) * self.master_volume;

        (final_output, self.pll_feedback_state)
    }
}
