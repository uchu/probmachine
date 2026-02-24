#![feature(portable_simd)]

mod params;
mod preset;
mod ui;
mod synth;
mod sequencer;
mod midi;

use egui_taffy::taffy::{
    prelude::*,
    style::{AlignItems, FlexDirection, JustifyContent},
};
use egui_taffy::{tui as taffy_layout, TuiBuilderLogic};
use nih_plug::prelude::*;
use nih_plug_egui::{create_egui_editor, egui};
use params::DeviceParams;
use std::sync::Arc;
use ui::{Page, SharedUiState};
use synth::{SynthEngine, MasterLimiter};
use midi::MidiProcessor;

pub struct Device {
    params: Arc<DeviceParams>,
    synth_engine: Option<SynthEngine>,
    ui_state: Arc<SharedUiState>,
    midi_processor: MidiProcessor,
    sample_rate: f32,
    cpu_load_smoothed: f32,
    volume_slew: f32,
    output_level_smoothed: f32,
    limiter: MasterLimiter,
    midi_events_buffer: Vec<(bool, bool, u8, u8, usize)>,
    host_transport_detected: bool,
    was_playing: bool,
    output_buffer_l: Vec<f32>,
    output_buffer_r: Vec<f32>,
    last_preset_version: u64,
    cpu_measure_counter: u32,
}

impl Default for Device {
    fn default() -> Self {
        Self {
            params: Arc::new(DeviceParams::default()),
            synth_engine: None,
            ui_state: Arc::new(SharedUiState::new()),
            midi_processor: MidiProcessor::new(),
            sample_rate: 44100.0,
            cpu_load_smoothed: 0.0,
            volume_slew: 0.5,
            output_level_smoothed: 0.0,
            limiter: MasterLimiter::new(44100.0),
            midi_events_buffer: Vec::with_capacity(64),
            host_transport_detected: false,
            was_playing: false,
            output_buffer_l: Vec::new(),
            output_buffer_r: Vec::new(),
            last_preset_version: 0,
            cpu_measure_counter: 0,
        }
    }
}

impl Plugin for Device {
    const NAME: &'static str = "Device";
    const VENDOR: &'static str = "Device Audio";
    const URL: &'static str = "https://example.com";
    const EMAIL: &'static str = "info@example.com";
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
        main_input_channels: None,
        main_output_channels: NonZeroU32::new(2),
        aux_input_ports: &[],
        aux_output_ports: &[],
        names: PortNames::const_default(),
    }];

    const MIDI_INPUT: MidiConfig = MidiConfig::MidiCCs;
    const MIDI_OUTPUT: MidiConfig = MidiConfig::MidiCCs;
    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        let params = self.params.clone();
        let ui_state = self.ui_state.clone();
        create_egui_editor(
            self.params.editor_state.clone(),
            (),
            |_, _| {},
            move |egui_ctx, setter, _state| {
                egui_ctx.style_mut(|style| {
                    let bg = egui::Color32::from_gray(18);
                    style.visuals.panel_fill = bg;
                    style.visuals.window_fill = bg;
                    style.visuals.faint_bg_color = bg;
                });
                egui::CentralPanel::default().show(egui_ctx, |ui| {
                    taffy_layout(ui, ui.id().with("page_layout"))
                        .reserve_available_space()
                        .style(Style {
                            display: Display::Flex,
                            flex_direction: FlexDirection::Column,
                            justify_content: Some(JustifyContent::SpaceBetween),
                            align_items: Some(AlignItems::Stretch),
                            padding: egui_taffy::taffy::Rect {
                                left: length(23.5),
                                right: length(0.0),
                                top: length(0.0),
                                bottom: length(0.0),
                            },
                            gap: Size {
                                width: length(0.0),
                                height: length(0.0),
                            },
                            ..Default::default()
                        })
                        .show(|tui| {
                            let mut current_page = tui.ui(|ui| {
                                ui.memory_mut(|mem| {
                                    *mem.data.get_temp_mut_or(egui::Id::new("current_page"), Page::Synth)
                                })
                            });

                            tui.ui(|ui| {
                                ui::navigation::render(ui, &mut current_page, &params, setter, &ui_state);
                            });

                            tui.ui(|ui| {
                                ui.memory_mut(|mem| {
                                    mem.data.insert_temp(egui::Id::new("current_page"), current_page);
                                });
                            });

                            current_page.render(tui, &params, setter, &ui_state);
                        });
                });
            },
        )
    }

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        let new_sample_rate = std::env::var("SAMPLE_RATE")
            .ok()
            .and_then(|s| s.parse::<f32>().ok())
            .unwrap_or(buffer_config.sample_rate);

        let sample_rate_changed = (new_sample_rate - self.sample_rate).abs() > 0.1;

        self.sample_rate = new_sample_rate;

        if self.synth_engine.is_none() || sample_rate_changed {
            self.synth_engine = Some(SynthEngine::new(new_sample_rate));
            self.limiter.set_sample_rate(new_sample_rate);
        }

        true
    }

    fn reset(&mut self) {
        if let Some(synth) = &mut self.synth_engine {
            synth.stop();
        }

        self.midi_processor.clear_all();
        self.was_playing = false;
        self.volume_slew = 0.0;
        self.output_level_smoothed = 0.0;
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        self.midi_processor.begin_buffer();

        while let Some(event) = context.next_event() {
            self.midi_processor.process_incoming_event(event);
        }

        let transport = context.transport();
        let tempo = transport.tempo.unwrap_or(120.0);
        let is_playing = transport.playing;

        if !is_playing {
            self.host_transport_detected = true;
        }

        // Detect transport stop transition: release voice and send MIDI note-off
        let just_stopped = self.was_playing && !is_playing;
        self.was_playing = is_playing;

        if just_stopped {
            if let Some(synth) = &mut self.synth_engine {
                synth.stop();
            }
            self.midi_processor.stop_all_notes(0);
        }

        // Check for DSP reset request from preset change
        if self.ui_state.take_dsp_reset_request() {
            if let Some(synth) = &mut self.synth_engine {
                synth.reset();
            }
        }

        if let Some(synth) = &mut self.synth_engine {
            synth.set_bpm(tempo);

            if let Ok(note_pool) = self.ui_state.note_pool.try_lock() {
                synth.update_note_pool(note_pool.clone());
            }
            if let Ok(strength_values) = self.ui_state.strength_values.try_lock() {
                synth.update_strength_values(strength_values.clone());
            }
            if let Ok(octave_rand) = self.ui_state.octave_randomization.try_lock() {
                synth.update_octave_randomization(octave_rand.clone());
            }

            synth.set_osc_params(
                self.params.synth_osc_d.modulated_plain_value(),
                self.params.synth_osc_v.modulated_plain_value(),
            );

            synth.set_osc_volume(self.params.synth_osc_volume.modulated_plain_value());

            synth.set_osc_octave(self.params.synth_osc_octave.value());
            synth.set_osc_tune(
                self.params.synth_osc_tune.value(),
                self.params.synth_osc_fine.modulated_plain_value(),
            );
            synth.set_osc_fold(self.params.synth_osc_fold.modulated_plain_value());

            synth.set_vps_stereo_v_offset(self.params.synth_osc_stereo_v_offset.modulated_plain_value());
            synth.set_vps_stereo_d_offset(self.params.synth_osc_stereo_d_offset.modulated_plain_value());
            synth.set_vps_shape(
                self.params.synth_vps_shape_type.value(),
                self.params.synth_vps_shape_amount.modulated_plain_value(),
            );
            synth.set_vps_phase_mode(self.params.synth_vps_phase_mode.value());

            synth.set_sub_volume(self.params.synth_sub_volume.modulated_plain_value());
            synth.set_sub_source(self.params.synth_sub_source.value());

            synth.set_pll_fm_params(
                self.params.synth_pll_fm_amount.modulated_plain_value(),
                self.params.synth_pll_fm_ratio_float.modulated_plain_value(),
                self.params.synth_pll_fm_expand.value(),
            );

            synth.set_pll_experimental_params(
                self.params.synth_pll_retrigger.modulated_plain_value(),
                self.params.synth_pll_burst_threshold.modulated_plain_value(),
                self.params.synth_pll_burst_amount.modulated_plain_value(),
                self.params.synth_pll_loop_saturation.modulated_plain_value(),
                self.params.synth_pll_color_amount.modulated_plain_value(),
                self.params.synth_pll_edge_sensitivity.modulated_plain_value(),
                self.params.synth_pll_range.modulated_plain_value(),
                self.params.synth_pll_stereo_track_offset.modulated_plain_value(),
            );

            synth.set_pll_stereo_phase(self.params.synth_pll_stereo_phase.modulated_plain_value());
            synth.set_pll_cross_feedback(self.params.synth_pll_cross_feedback.modulated_plain_value());
            synth.set_pll_fm_env_amount(self.params.synth_pll_fm_env_amount.modulated_plain_value());

            synth.set_coloration_params(
                self.params.synth_ring_mod.modulated_plain_value(),
                self.params.synth_wavefold.modulated_plain_value(),
                self.params.synth_drift_amount.modulated_plain_value(),
                self.params.synth_drift_rate.modulated_plain_value(),
                self.params.synth_tube_drive.modulated_plain_value(),
            );

            synth.set_bypass_switches(
                self.params.synth_pll_enable.value(),
                self.params.synth_vps_enable.value(),
                self.params.synth_coloration_enable.value(),
                self.params.synth_reverb_enable.value(),
                self.params.synth_oversampling_factor.value(),
            );

            synth.set_base_rate(self.params.synth_base_rate.value());

            synth.set_pll_ref_params(
                self.params.synth_pll_ref_octave.value(),
                self.params.synth_pll_ref_pulse_width.modulated_plain_value(),
            );
            synth.set_pll_ref_tune(
                self.params.synth_pll_ref_tune.value(),
                self.params.synth_pll_ref_fine.modulated_plain_value(),
            );

            let pll_mult = match self.params.synth_pll_mult.value() {
                0 => 1.0,
                1 => 2.0,
                2 => 4.0,
                3 => 8.0,
                4 => 16.0,
                5 => 32.0,
                6 => 64.0,
                _ => 1.0,
            };

            synth.set_pll_params(
                self.params.synth_pll_track_speed.modulated_plain_value(),
                self.params.synth_pll_damping.modulated_plain_value(),
                pll_mult,
                self.params.synth_pll_influence.modulated_plain_value(),
                self.params.synth_pll_colored.value(),
                self.params.synth_pll_mode.value(),
            );
            synth.set_pll_mult_slew_time(self.params.synth_pll_mult_slew_time.modulated_plain_value());
            synth.set_pll_precision(self.params.synth_pll_precision.value());
            synth.set_pll_advanced_params(
                self.params.synth_pll_anti_alias.value(),
                self.params.synth_pll_injection_amount.modulated_plain_value(),
                self.params.synth_pll_injection_x4.value(),
            );

            synth.set_pll_volume(self.params.synth_pll_volume.modulated_plain_value());

            synth.set_pll_stereo_damp_offset(self.params.synth_pll_stereo_damp_offset.modulated_plain_value());

            synth.set_pll_glide(self.params.synth_pll_glide.modulated_plain_value());

            synth.set_legato_mode(self.params.legato_mode.value());
            if self.params.legato_mode.value() {
                synth.set_legato_time(self.params.legato_time.modulated_plain_value());
            }

            synth.set_filter_params(
                self.params.synth_filter_enable.value(),
                self.params.synth_filter_cutoff.modulated_plain_value(),
                self.params.synth_filter_resonance.modulated_plain_value(),
                self.params.synth_filter_env_amount.modulated_plain_value(),
                self.params.synth_filter_drive.modulated_plain_value(),
            );

            synth.set_volume(1.0);

            synth.set_volume_envelope(
                self.params.synth_vol_attack.modulated_plain_value(),
                self.params.synth_vol_attack_shape.modulated_plain_value(),
                self.params.synth_vol_decay.modulated_plain_value(),
                self.params.synth_vol_decay_shape.modulated_plain_value(),
                self.params.synth_vol_sustain.modulated_plain_value(),
                self.params.synth_vol_release.modulated_plain_value(),
                self.params.synth_vol_release_shape.modulated_plain_value(),
            );

            synth.set_filter_envelope(
                self.params.synth_filt_attack.modulated_plain_value(),
                self.params.synth_filt_attack_shape.modulated_plain_value(),
                self.params.synth_filt_decay.modulated_plain_value(),
                self.params.synth_filt_decay_shape.modulated_plain_value(),
                self.params.synth_filt_sustain.modulated_plain_value(),
                self.params.synth_filt_release.modulated_plain_value(),
                self.params.synth_filt_release_shape.modulated_plain_value(),
            );

            synth.set_reverb_params(
                self.params.synth_reverb_mix.modulated_plain_value(),
                self.params.synth_reverb_pre_delay.modulated_plain_value(),
                self.params.synth_reverb_time_scale.modulated_plain_value(),
                self.params.synth_reverb_input_hpf.modulated_plain_value(),
                self.params.synth_reverb_input_lpf.modulated_plain_value(),
                self.params.synth_reverb_hpf.modulated_plain_value(),
                self.params.synth_reverb_lpf.modulated_plain_value(),
                self.params.synth_reverb_mod_speed.modulated_plain_value(),
                self.params.synth_reverb_mod_depth.modulated_plain_value(),
                self.params.synth_reverb_mod_shape.modulated_plain_value(),
                self.params.synth_reverb_diffusion_mix.modulated_plain_value(),
                self.params.synth_reverb_diffusion.modulated_plain_value(),
                self.params.synth_reverb_decay.modulated_plain_value(),
                self.params.synth_reverb_ducking.modulated_plain_value(),
            );

            synth.set_lfo_params(
                0,
                self.params.lfo1_rate.modulated_plain_value(),
                self.params.lfo1_waveform.value(),
                self.params.lfo1_tempo_sync.value(),
                self.params.lfo1_sync_division.value(),
                self.params.lfo1_sync_source.value(),
                self.params.lfo1_phase_mod.modulated_plain_value(),
            );
            synth.set_lfo_modulation(0, 0, self.params.lfo1_dest1.value(), self.params.lfo1_amount1.modulated_plain_value());
            synth.set_lfo_modulation(0, 1, self.params.lfo1_dest2.value(), self.params.lfo1_amount2.modulated_plain_value());

            synth.set_lfo_params(
                1,
                self.params.lfo2_rate.modulated_plain_value(),
                self.params.lfo2_waveform.value(),
                self.params.lfo2_tempo_sync.value(),
                self.params.lfo2_sync_division.value(),
                self.params.lfo2_sync_source.value(),
                self.params.lfo2_phase_mod.modulated_plain_value(),
            );
            synth.set_lfo_modulation(1, 0, self.params.lfo2_dest1.value(), self.params.lfo2_amount1.modulated_plain_value());
            synth.set_lfo_modulation(1, 1, self.params.lfo2_dest2.value(), self.params.lfo2_amount2.modulated_plain_value());

            synth.set_lfo_params(
                2,
                self.params.lfo3_rate.modulated_plain_value(),
                self.params.lfo3_waveform.value(),
                self.params.lfo3_tempo_sync.value(),
                self.params.lfo3_sync_division.value(),
                self.params.lfo3_sync_source.value(),
                self.params.lfo3_phase_mod.modulated_plain_value(),
            );
            synth.set_lfo_modulation(2, 0, self.params.lfo3_dest1.value(), self.params.lfo3_amount1.modulated_plain_value());
            synth.set_lfo_modulation(2, 1, self.params.lfo3_dest2.value(), self.params.lfo3_amount2.modulated_plain_value());

            synth.set_mod_seq_step(0, self.params.mseq_step_1.value());
            synth.set_mod_seq_step(1, self.params.mseq_step_2.value());
            synth.set_mod_seq_step(2, self.params.mseq_step_3.value());
            synth.set_mod_seq_step(3, self.params.mseq_step_4.value());
            synth.set_mod_seq_step(4, self.params.mseq_step_5.value());
            synth.set_mod_seq_step(5, self.params.mseq_step_6.value());
            synth.set_mod_seq_step(6, self.params.mseq_step_7.value());
            synth.set_mod_seq_step(7, self.params.mseq_step_8.value());
            synth.set_mod_seq_step(8, self.params.mseq_step_9.value());
            synth.set_mod_seq_step(9, self.params.mseq_step_10.value());
            synth.set_mod_seq_step(10, self.params.mseq_step_11.value());
            synth.set_mod_seq_step(11, self.params.mseq_step_12.value());
            synth.set_mod_seq_step(12, self.params.mseq_step_13.value());
            synth.set_mod_seq_step(13, self.params.mseq_step_14.value());
            synth.set_mod_seq_step(14, self.params.mseq_step_15.value());
            synth.set_mod_seq_step(15, self.params.mseq_step_16.value());
            synth.set_mod_seq_params(
                self.params.mseq_ties.value(),
                self.params.mseq_division.value(),
                self.params.mseq_slew.modulated_plain_value(),
            );
            synth.set_mod_seq_modulation(0, self.params.mseq_dest1.value(), self.params.mseq_amount1.modulated_plain_value());
            synth.set_mod_seq_modulation(1, self.params.mseq_dest2.value(), self.params.mseq_amount2.modulated_plain_value());

            let num_samples = buffer.samples();
            self.output_buffer_l.resize(num_samples, 0.0);
            self.output_buffer_r.resize(num_samples, 0.0);
            self.output_buffer_l.fill(0.0);
            self.output_buffer_r.fill(0.0);

            let pll_feedback_amt = self.params.synth_pll_feedback.modulated_plain_value();
            let base_freq = 220.0;

            let seq_playing = if self.host_transport_detected {
                self.params.sequencer_enable.value() || is_playing
            } else {
                self.params.sequencer_enable.value()
            };

            let measure_cpu = self.cpu_measure_counter == 0;
            self.cpu_measure_counter = (self.cpu_measure_counter + 1) % 32;

            let start_time = if measure_cpu { Some(std::time::Instant::now()) } else { None };
            synth.process_block(
                &mut self.output_buffer_l,
                &mut self.output_buffer_r,
                &self.params,
                pll_feedback_amt,
                base_freq,
                &mut self.midi_events_buffer,
                seq_playing,
            );

            for (is_note_on, is_note_off, midi_note, velocity, sample_idx) in &self.midi_events_buffer {
                if *is_note_on {
                    self.midi_processor.note_on_from_sequencer(*midi_note, *velocity, *sample_idx as u32);
                } else if *is_note_off {
                    self.midi_processor.note_off_from_sequencer(*sample_idx as u32);
                }
            }

            self.midi_processor.send_output::<Device>(
                context,
                is_playing,
                buffer.samples(),
                self.sample_rate,
                tempo,
            );

            if let Some(start) = start_time {
                let elapsed = start.elapsed();
                let buf_time = buffer.samples() as f32 / self.sample_rate;
                let cpu_load = (elapsed.as_secs_f32() / buf_time) * 100.0;

                let smoothing_time = 1.5;
                let alpha = 1.0 - (-buf_time / smoothing_time).exp();
                self.cpu_load_smoothed = alpha * cpu_load + (1.0 - alpha) * self.cpu_load_smoothed;
                self.ui_state.set_cpu_load(self.cpu_load_smoothed);
            }

            let target_volume = self.params.global_volume.modulated_plain_value();
            let slew_coeff = 1.0 - (-1.0 / (self.sample_rate * 0.01)).exp();

            for i in 0..num_samples {
                self.volume_slew += (target_volume - self.volume_slew) * slew_coeff;
                self.output_buffer_l[i] *= self.volume_slew;
                self.output_buffer_r[i] *= self.volume_slew;
            }

            self.limiter.process_block(&mut self.output_buffer_l, &mut self.output_buffer_r);

            let mut peak: f32 = 0.0;
            for (i, channel_samples) in buffer.iter_samples().enumerate() {
                peak = peak.max(self.output_buffer_l[i].abs()).max(self.output_buffer_r[i].abs());

                let mut iter = channel_samples.into_iter();
                if let Some(left) = iter.next() {
                    *left = self.output_buffer_l[i];
                }
                if let Some(right) = iter.next() {
                    *right = self.output_buffer_r[i];
                }
            }

            let buffer_time_secs = buffer.samples() as f32 / self.sample_rate;
            let decay_time = 0.3;
            let decay_coeff = (-buffer_time_secs / decay_time).exp();
            if peak > self.output_level_smoothed {
                self.output_level_smoothed = peak;
            } else {
                self.output_level_smoothed *= decay_coeff;
            }
            self.ui_state.set_output_level(self.output_level_smoothed.min(1.0));
        } else {
            for channel_samples in buffer.iter_samples() {
                for sample_out in channel_samples {
                    *sample_out = 0.0;
                }
            }
        }

        ProcessStatus::Normal
    }
}

impl ClapPlugin for Device {
    const CLAP_ID: &'static str = "com.device-audio.device";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("A 4/4 rhythm loop device");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[
        ClapFeature::Instrument,
        ClapFeature::Synthesizer,
        ClapFeature::Stereo,
    ];
}

impl Vst3Plugin for Device {
    const VST3_CLASS_ID: [u8; 16] = *b"DeviceAudioDev01";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Instrument, Vst3SubCategory::Synth];
}

nih_plug::nih_export_clap!(Device);
nih_plug::nih_export_vst3!(Device);
