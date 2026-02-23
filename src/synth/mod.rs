#![allow(clippy::too_many_arguments)]

pub mod simd;
mod oscillator;
mod filter;
mod envelope;
mod reverb;
mod voice;
pub mod lfo;
mod limiter;
pub mod mod_sequencer;

pub use voice::Voice;
pub use lfo::LfoBank;
pub use limiter::MasterLimiter;
use crate::sequencer::Sequencer;
use crate::params::DeviceParams;
use mod_sequencer::ModSequencer;

pub struct SynthEngine {
    voice: Voice,
    sequencer: Sequencer,
    pll_feedback: f64,
    pub lfo_bank: LfoBank,
    pub mod_sequencer: ModSequencer,
}

impl SynthEngine {
    pub fn new(sample_rate: f32) -> Self {
        let mut voice = Voice::new(sample_rate);
        let sample_rate_f64 = sample_rate as f64;
        voice.set_frequency(220.0, 0.0, 0.0);

        Self {
            voice,
            sequencer: Sequencer::new(sample_rate_f64, 120.0),
            pll_feedback: 0.0,
            lfo_bank: LfoBank::new(sample_rate_f64),
            mod_sequencer: ModSequencer::new(sample_rate_f64),
        }
    }

    #[allow(dead_code)]
    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.voice.set_sample_rate(sample_rate);
        self.sequencer.set_sample_rate(sample_rate as f64);
        self.lfo_bank.set_sample_rate(sample_rate as f64);
        self.mod_sequencer.set_sample_rate(sample_rate as f64);
    }

    pub fn set_bpm(&mut self, bpm: f64) {
        self.sequencer.set_bpm(bpm);
        self.voice.set_bpm(bpm);
    }

    pub fn set_osc_params(&mut self, d: f32, v: f32) {
        self.voice.set_osc_params(d as f64, v as f64);
    }

    pub fn set_osc_volume(&mut self, volume: f32) {
        self.voice.set_osc_volume(volume as f64);
    }

    pub fn set_osc_octave(&mut self, octave: i32) {
        self.voice.set_osc_octave(octave);
    }

    pub fn set_osc_tune(&mut self, tune: i32, fine: f32) {
        self.voice.set_osc_tune(tune, fine as f64);
    }

    pub fn set_osc_fold(&mut self, fold: f32) {
        self.voice.set_osc_fold(fold as f64);
    }

    pub fn set_vps_stereo_v_offset(&mut self, offset: f32) {
        self.voice.set_vps_stereo_v_offset(offset as f64);
    }

    pub fn set_pll_ref_params(&mut self, octave: i32, pulse_width: f32) {
        self.voice.set_pll_ref_params(octave, pulse_width as f64);
    }

    pub fn set_pll_ref_tune(&mut self, tune: i32, fine: f32) {
        self.voice.set_pll_ref_tune(tune, fine as f64);
    }

    pub fn set_pll_params(&mut self, track: f32, damp: f32, mult: f32, influence: f32, colored: bool, edge_mode: bool) {
        self.voice.set_pll_params(track as f64, damp as f64, mult as f64, influence as f64, colored, edge_mode);
    }

    pub fn set_pll_mult_slew(&mut self, enabled: bool) {
        self.voice.set_pll_mult_slew(enabled);
    }

    pub fn set_pll_volume(&mut self, volume: f32) {
        self.voice.set_pll_volume(volume as f64);
    }

    pub fn set_pll_stereo_damp_offset(&mut self, offset: f32) {
        self.voice.set_pll_stereo_damp_offset(offset as f64);
    }

    pub fn set_pll_glide(&mut self, glide_ms: f32) {
        self.voice.set_glide_time(glide_ms as f64);
    }

    pub fn set_legato_mode(&mut self, enabled: bool) {
        self.voice.set_legato_mode(enabled);
    }

    pub fn set_legato_time(&mut self, time_ms: f32) {
        self.voice.set_glide_time(time_ms as f64);
    }

    pub fn set_pll_fm_params(&mut self, amount: f32, ratio: i32) {
        self.voice.set_pll_fm_params(amount as f64, ratio);
    }

    pub fn set_pll_experimental_params(
        &mut self,
        retrigger: f32,
        burst_threshold: f32,
        burst_amount: f32,
        loop_saturation: f32,
        color_amount: f32,
        edge_sensitivity: f32,
        range: f32,
        stereo_track_offset: f32,
    ) {
        self.voice.set_pll_experimental_params(
            retrigger as f64,
            burst_threshold as f64,
            burst_amount as f64,
            loop_saturation as f64,
            color_amount as f64,
            edge_sensitivity as f64,
            range as f64,
            stereo_track_offset as f64,
        );
    }

    pub fn set_pll_stereo_phase(&mut self, phase: f32) {
        self.voice.set_pll_stereo_phase(phase as f64);
    }

    pub fn set_pll_cross_feedback(&mut self, amount: f32) {
        self.voice.set_pll_cross_feedback(amount as f64);
    }

    pub fn set_pll_fm_env_amount(&mut self, amount: f32) {
        self.voice.set_pll_fm_env_amount(amount as f64);
    }

    pub fn set_coloration_params(
        &mut self,
        ring_mod: f32,
        wavefold: f32,
        drift_amount: f32,
        drift_rate: f32,
        tube: f32,
    ) {
        self.voice.set_coloration_params(
            ring_mod as f64,
            wavefold as f64,
            drift_amount as f64,
            drift_rate as f64,
            tube as f64,
        );
    }

    pub fn set_bypass_switches(
        &mut self,
        pll: bool,
        vps: bool,
        coloration: bool,
        reverb: bool,
        oversampling_factor: i32,
    ) {
        self.voice.set_bypass_switches(pll, vps, coloration, reverb, oversampling_factor);
    }

    pub fn set_base_rate(&mut self, rate_option: i32) {
        self.voice.set_base_rate(rate_option);
    }

    pub fn set_vps_stereo_d_offset(&mut self, offset: f32) {
        self.voice.set_vps_stereo_d_offset(offset as f64);
    }

    pub fn set_vps_shape(&mut self, shape_type: i32, amount: f32) {
        self.voice.set_vps_shape(shape_type, amount as f64);
    }

    pub fn set_vps_phase_mode(&mut self, mode: i32) {
        self.voice.set_vps_phase_mode(mode);
    }

    pub fn set_sub_volume(&mut self, volume: f32) {
        self.voice.set_sub_volume(volume as f64);
    }

    pub fn set_sub_source(&mut self, source: i32) {
        self.voice.set_sub_source(source);
    }

    pub fn set_filter_params(&mut self, enabled: bool, cutoff: f32, resonance: f32, env_amount: f32, drive: f32) {
        self.voice.set_filter_params(enabled, cutoff as f64, resonance as f64, env_amount as f64, drive as f64);
    }

    pub fn set_volume(&mut self, volume: f32) {
        self.voice.set_volume(volume as f64);
    }

    pub fn set_volume_envelope(&mut self, attack: f32, attack_shape: f32, decay: f32, decay_shape: f32, sustain: f32, release: f32, release_shape: f32) {
        self.voice.set_volume_envelope(attack as f64, attack_shape as f64, decay as f64, decay_shape as f64, sustain as f64, release as f64, release_shape as f64);
    }

    pub fn set_filter_envelope(&mut self, attack: f32, attack_shape: f32, decay: f32, decay_shape: f32, sustain: f32, release: f32, release_shape: f32) {
        self.voice.set_filter_envelope(attack as f64, attack_shape as f64, decay as f64, decay_shape as f64, sustain as f64, release as f64, release_shape as f64);
    }

    pub fn update_note_pool(&mut self, note_pool: crate::sequencer::NotePool) {
        self.sequencer.note_pool = note_pool;
    }

    pub fn update_strength_values(&mut self, strength_values: Vec<f32>) {
        if strength_values.len() == 96 {
            self.sequencer.strength_values = strength_values;
        }
    }

    pub fn update_octave_randomization(&mut self, octave_randomization: crate::sequencer::OctaveRandomization) {
        self.sequencer.octave_randomization = octave_randomization;
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
        ducking: f32,
    ) {
        self.voice.set_reverb_params(
            mix as f64,
            pre_delay_ms as f64,
            time_scale as f64,
            input_hpf_hz as f64,
            input_lpf_hz as f64,
            reverb_hpf_hz as f64,
            reverb_lpf_hz as f64,
            mod_speed as f64,
            mod_depth as f64,
            mod_shape as f64,
            input_diffusion_mix as f64,
            diffusion as f64,
            decay as f64,
            ducking as f64,
        );
    }

    pub fn set_lfo_params(
        &mut self,
        lfo_idx: usize,
        rate: f32,
        waveform: i32,
        tempo_sync: bool,
        sync_division: i32,
        sync_source: i32,
        phase_mod_amount: f32,
    ) {
        self.lfo_bank.set_lfo_params(
            lfo_idx,
            rate as f64,
            waveform,
            tempo_sync,
            sync_division,
            sync_source,
            phase_mod_amount as f64,
        );
    }

    pub fn set_lfo_modulation(
        &mut self,
        lfo_idx: usize,
        slot: usize,
        destination: i32,
        amount: f32,
    ) {
        self.lfo_bank.set_modulation(lfo_idx, slot, destination, amount as f64);
    }

    pub fn set_mod_seq_step(&mut self, index: usize, value: f32) {
        self.mod_sequencer.set_step(index, value as f64);
    }

    pub fn set_mod_seq_params(&mut self, ties: i32, division: i32, slew: f32) {
        self.mod_sequencer.set_ties(ties as u16);
        self.mod_sequencer.set_division(division);
        self.mod_sequencer.set_slew(slew as f64);
    }

    pub fn set_mod_seq_modulation(&mut self, slot: usize, destination: i32, amount: f32) {
        self.mod_sequencer.set_modulation(slot, destination, amount as f64);
    }

    #[allow(dead_code)]
    pub fn get_lfo_output(&self, idx: usize) -> f32 {
        self.lfo_bank.get_lfo_output(idx) as f32
    }

    pub fn process_block(
        &mut self,
        output_l: &mut [f32],
        output_r: &mut [f32],
        params: &DeviceParams,
        feedback_amount: f32,
        _base_freq: f32,
        midi_events: &mut Vec<(bool, bool, u8, u8, usize)>,
        seq_playing: bool,
    ) {
        let bpm = self.sequencer.get_bpm();
        self.voice.set_bpm(bpm);
        midi_events.clear();

        for (sample_idx, (l, r)) in output_l.iter_mut().zip(output_r.iter_mut()).enumerate() {
            if seq_playing {
                let (should_trigger, should_release, frequency, velocity, midi_note) = self.sequencer.update(params);

                if should_trigger {
                    self.voice.set_frequency(frequency, self.pll_feedback, feedback_amount as f64);
                    self.voice.set_velocity(velocity);
                    self.voice.trigger();
                    midi_events.push((true, false, midi_note, velocity, sample_idx));
                }

                if should_release {
                    self.voice.release();
                    midi_events.push((false, true, midi_note, velocity, sample_idx));
                }
            }

            let mut mod_values = self.lfo_bank.process(bpm);
            let seq_mod = self.mod_sequencer.process(bpm);
            mod_values.accumulate(&seq_mod);
            self.voice.apply_modulation(&mod_values);

            let (left_sample, right_sample) = self.voice.process(self.pll_feedback);

            *l = left_sample as f32;
            *r = right_sample as f32;
        }
    }
}
