#![allow(clippy::too_many_arguments)]

mod oscillator;
mod filter;
mod envelope;
mod reverb;
mod formant;
mod voice;
pub mod lfo;

pub use voice::Voice;
pub use lfo::LfoBank;
use crate::sequencer::Sequencer;
use crate::params::DeviceParams;

pub struct SynthEngine {
    voice: Voice,
    sequencer: Sequencer,
    pll_feedback: f64,
    pub lfo_bank: LfoBank,
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
        }
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

    pub fn set_vps_stereo_v_offset(&mut self, offset: f32) {
        self.voice.set_vps_stereo_v_offset(offset as f64);
    }

    pub fn set_pll_ref_params(&mut self, octave: i32, tune: i32, fine_tune: f32, pulse_width: f32) {
        self.voice.set_pll_ref_params(octave, tune, fine_tune as f64, pulse_width as f64);
    }

    pub fn set_pll_params(&mut self, track: f32, damp: f32, mult: f32, influence: f32, colored: bool, edge_mode: bool) {
        self.voice.set_pll_params(track as f64, damp as f64, mult as f64, influence as f64, colored, edge_mode);
    }

    pub fn set_pll_volume(&mut self, volume: f32) {
        self.voice.set_pll_volume(volume as f64);
    }

    pub fn set_pll_stereo_damp_offset(&mut self, offset: f32) {
        self.voice.set_pll_stereo_damp_offset(offset as f64);
    }

    pub fn set_pll_distortion(&mut self, amount: f32) {
        self.voice.set_pll_distortion(amount as f64);
    }

    pub fn set_pll_glide(&mut self, glide_ms: f32) {
        self.voice.set_glide_time(glide_ms as f64);
    }

    pub fn set_pll_fm_params(&mut self, amount: f32, ratio: i32) {
        self.voice.set_pll_fm_params(amount as f64, ratio);
    }

    pub fn set_sub_volume(&mut self, volume: f32) {
        self.voice.set_sub_volume(volume as f64);
    }

    pub fn set_distortion(&mut self, amount: f32) {
        self.voice.set_distortion(amount as f64);
    }

    pub fn set_formant_params(&mut self, mix: f32, vowel: f32, shift: f32) {
        self.voice.set_formant_params(mix as f64, vowel as f64, shift as f64);
    }

    pub fn set_filter_params(&mut self, enabled: bool, cutoff: f32, resonance: f32, env_amount: f32, drive: f32, mode: i32) {
        self.voice.set_filter_params(enabled, cutoff as f64, resonance as f64, env_amount as f64, drive as f64, mode);
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

    #[allow(dead_code)]
    pub fn get_lfo_output(&self, idx: usize) -> f32 {
        self.lfo_bank.get_lfo_output(idx) as f32
    }

    pub fn process_block(&mut self, output_l: &mut [f32], output_r: &mut [f32], params: &DeviceParams, feedback_amount: f32, _base_freq: f32) {
        let bpm = self.sequencer.get_bpm();

        for (l, r) in output_l.iter_mut().zip(output_r.iter_mut()) {
            let (should_trigger, should_release, frequency) = self.sequencer.update(params);

            if should_trigger {
                self.voice.set_frequency(frequency, self.pll_feedback, feedback_amount as f64);
                self.voice.trigger();
            }

            if should_release {
                self.voice.release();
            }

            // Process LFOs and get modulation values
            let mod_values = self.lfo_bank.process(bpm);

            // Apply modulation to voice
            self.voice.apply_modulation(&mod_values);

            let (left_sample, right_sample) = self.voice.process(self.pll_feedback);

            *l = left_sample as f32;
            *r = right_sample as f32;
        }
    }
}
