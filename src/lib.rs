#![feature(portable_simd)]

mod params;
mod preset;
mod ui;
mod synth;
mod sequencer;
mod midi;
mod midi_modes;
mod midi_devices;
mod midi_learn;
mod midi_clock;

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
use synth::{SynthEngine, MasterLimiter, MasterHpf, BrillianceFilter};
use synth::master_hpf::{HpfMode, HpfBoost};
use midi::MidiProcessor;
use midi_modes::{MidiInputMode, MidiModeProcessor, MidiModeResult};

pub struct PhaseBurn {
    params: Arc<DeviceParams>,
    synth_engine: Option<SynthEngine>,
    ui_state: Arc<SharedUiState>,
    midi_processor: MidiProcessor,
    sample_rate: f32,
    cpu_load_smoothed: f32,
    volume_slew: f32,
    output_level_smoothed: f32,
    limiter: MasterLimiter,
    master_hpf: MasterHpf,
    brilliance: BrillianceFilter,
    midi_events_buffer: Vec<(bool, bool, u8, u8, usize)>,
    midi_mode_processor: MidiModeProcessor,
    midi_clock_pll: midi_clock::MidiClockPll,
    midi_clock_out_phase: f64,
    process_time_seconds: f64,
    transport_has_played: bool,
    was_playing: bool,
    was_seq_playing: bool,
    output_buffer_l: Vec<f32>,
    output_buffer_r: Vec<f32>,
    sub_buffer: Vec<f32>,
    cpu_measure_counter: u32,
}

impl Default for PhaseBurn {
    fn default() -> Self {
        Self {
            params: Arc::new(DeviceParams::default()),
            synth_engine: None,
            ui_state: Arc::new(SharedUiState::new()),
            midi_processor: MidiProcessor::new(),
            sample_rate: 44100.0,
            cpu_load_smoothed: 0.0,
            volume_slew: 0.125,
            output_level_smoothed: 0.0,
            limiter: MasterLimiter::new(44100.0),
            master_hpf: MasterHpf::new(44100.0),
            brilliance: BrillianceFilter::new(44100.0),
            midi_events_buffer: Vec::with_capacity(64),
            midi_mode_processor: MidiModeProcessor::new(),
            midi_clock_pll: midi_clock::MidiClockPll::new(),
            midi_clock_out_phase: 0.0,
            process_time_seconds: 0.0,
            transport_has_played: false,
            was_playing: false,
            was_seq_playing: false,
            output_buffer_l: Vec::new(),
            output_buffer_r: Vec::new(),
            sub_buffer: Vec::new(),
            cpu_measure_counter: 0,
        }
    }
}

impl Plugin for PhaseBurn {
    const NAME: &'static str = "PhaseBurn";
    const VENDOR: &'static str = "PhaseBurn Audio";
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
            |egui_ctx, _| {
                let mut fonts = egui::FontDefinitions::default();
                fonts.font_data.insert(
                    "inter_bold".to_owned(),
                    std::sync::Arc::new(egui::FontData::from_static(include_bytes!("../assets/fonts/Inter-Bold.ttf"))),
                );
                fonts.families.insert(
                    egui::FontFamily::Name("bold".into()),
                    vec!["inter_bold".to_owned()],
                );
                egui_ctx.set_fonts(fonts);
            },
            move |egui_ctx, setter, _state| {
                egui_ctx.style_mut(|style| {
                    let bg = egui::Color32::from_gray(18);
                    style.visuals.panel_fill = bg;
                    style.visuals.window_fill = bg;
                    style.visuals.faint_bg_color = bg;
                });

                apply_midi_learn(&params, setter, &ui_state);

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

                    if let Ok(mut mgr) = ui_state.midi_device_manager.try_lock() {
                        mgr.flush_output();
                    }
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

        self.ui_state.sample_rate.store(
            new_sample_rate as u32,
            std::sync::atomic::Ordering::Relaxed,
        );

        if self.synth_engine.is_none() || sample_rate_changed {
            self.synth_engine = Some(SynthEngine::new(new_sample_rate));
            self.limiter.set_sample_rate(new_sample_rate);
            self.master_hpf.set_sample_rate(new_sample_rate);
            self.brilliance.set_sample_rate(new_sample_rate);
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

        let clock_in_enabled = self.ui_state.midi_clock_in.load(std::sync::atomic::Ordering::Relaxed);
        let transport_in_enabled = self.ui_state.midi_transport_in.load(std::sync::atomic::Ordering::Relaxed);

        if let Ok(mut q) = self.ui_state.midi_device_input_queue.try_lock() {
            while let Some(raw) = q.pop_front() {
                if raw.len >= 1 {
                    match raw.data[0] {
                        0xF8 => {
                            if clock_in_enabled {
                                self.midi_clock_pll.process_tick(self.process_time_seconds);
                            }
                            continue;
                        }
                        0xFA | 0xFB => {
                            if transport_in_enabled {
                                self.ui_state.midi_transport_start.store(true, std::sync::atomic::Ordering::Relaxed);
                            }
                            continue;
                        }
                        0xFC => {
                            if transport_in_enabled {
                                self.ui_state.midi_transport_stop.store(true, std::sync::atomic::Ordering::Relaxed);
                            }
                            continue;
                        }
                        _ => {}
                    }
                }
                if let Some(event) = midi_devices::raw_midi_to_note_event(&raw) {
                    self.midi_processor.process_incoming_event(event);
                }
            }
        }

        for cc in 0u8..128 {
            let value = self.midi_processor.input.cc_state.get_cc(cc);
            let prev = f32::from_bits(
                self.ui_state.midi_learn.cc_values[cc as usize]
                    .load(std::sync::atomic::Ordering::Relaxed),
            );
            if (value - prev).abs() > 0.001 {
                self.ui_state.midi_learn.store_cc(cc, value);
            }
        }

        let transport = context.transport();
        let num_samples = buffer.samples();

        self.midi_clock_pll.advance_samples(num_samples as u32, self.sample_rate);
        self.process_time_seconds += num_samples as f64 / self.sample_rate as f64;

        let tempo = if clock_in_enabled && self.midi_clock_pll.is_locked() {
            self.midi_clock_pll.bpm()
        } else {
            transport.tempo.unwrap_or(120.0)
        };

        let mut is_playing = transport.playing;

        if transport_in_enabled {
            if self.ui_state.midi_transport_start.swap(false, std::sync::atomic::Ordering::Relaxed) {
                is_playing = true;
                self.transport_has_played = true;
            }
            if self.ui_state.midi_transport_stop.swap(false, std::sync::atomic::Ordering::Relaxed) {
                is_playing = false;
            }
        }

        if is_playing {
            self.transport_has_played = true;
        }

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

            if self.ui_state.take_seq_dirty() {
                if let Ok(note_pool) = self.ui_state.note_pool.try_lock() {
                    synth.update_note_pool(note_pool.clone());
                }
                if let Ok(strength_values) = self.ui_state.strength_values.try_lock() {
                    synth.update_strength_values(strength_values.clone());
                }
                if let Ok(octave_rand) = self.ui_state.octave_randomization.try_lock() {
                    synth.update_octave_randomization(octave_rand.clone());
                }
                if let Ok(style_config) = self.ui_state.style_config.try_lock() {
                    synth.update_style_config(style_config.clone());
                }
                if let Ok(multi_bar) = self.ui_state.multi_bar_config.try_lock() {
                    synth.update_multi_bar_config(multi_bar.clone());
                }
                if let Ok(melodic) = self.ui_state.melodic_config.try_lock() {
                    synth.update_melodic_config(melodic.clone());
                }
                if let Ok(links) = self.ui_state.beat_links.try_lock() {
                    synth.update_beat_links(links.clone());
                }
            }

            if self.ui_state.ml_dataset_dirty.swap(false, std::sync::atomic::Ordering::AcqRel) {
                if let Ok(guard) = self.ui_state.ml_dataset.try_lock() {
                    synth.update_ml_dataset(guard.clone());
                }
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
            synth.set_vps_fold_range(self.params.synth_vps_fold_range.value());
            synth.set_vps_phase_mode(self.params.synth_vps_phase_mode.value());

            synth.set_sub_volume(self.params.synth_sub_volume.modulated_plain_value());

            synth.set_saw_volume(self.params.synth_saw_volume.modulated_plain_value());
            synth.set_saw_octave(self.params.synth_saw_octave.value());
            synth.set_saw_tune(self.params.synth_saw_tune.value());
            synth.set_saw_fold(self.params.synth_saw_fold.modulated_plain_value());
            synth.set_saw_fold_range(self.params.synth_saw_fold_range.value());
            synth.set_saw_tight(self.params.synth_saw_tight.modulated_plain_value());
            synth.set_saw_shape(
                self.params.synth_saw_shape_type.value(),
                self.params.synth_saw_shape_amount.modulated_plain_value(),
            );

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
                self.params.synth_saw_enable.value(),
            );
            synth.set_oversampling(
                self.params.synth_pll_oversampling.value(),
                self.params.synth_saw_oversampling.value(),
                self.params.synth_vps_oversampling.value(),
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
            synth.set_legato_velocity_lock(self.params.legato_velocity_lock.value());
            synth.set_vca_mode(self.params.vca_mode.value());
            synth.set_note_priority(self.params.note_priority.value());


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
            synth.set_retrigger_dip(self.params.synth_retrigger_dip.modulated_plain_value());
            synth.set_phase_reset_on_retrigger(self.params.synth_phase_reset.value());
            synth.set_pll_tail(
                self.params.synth_pll_tail.value(),
                self.params.synth_pll_tail_time.modulated_plain_value(),
                self.params.synth_pll_tail_amount.modulated_plain_value(),
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
            self.sub_buffer.resize(num_samples, 0.0);
            self.output_buffer_l.fill(0.0);
            self.output_buffer_r.fill(0.0);
            self.sub_buffer.fill(0.0);

            let pll_feedback_amt = self.params.synth_pll_feedback.modulated_plain_value();
            let base_freq = 220.0;

            let seq_playing = if self.transport_has_played {
                self.params.sequencer_enable.value() && is_playing
            } else {
                self.params.sequencer_enable.value()
            };

            let prev_seq_playing = self.was_seq_playing;
            if prev_seq_playing && !seq_playing {
                synth.stop();
                self.midi_processor.stop_all_notes(0);
            }
            self.was_seq_playing = seq_playing;

            let midi_mode = MidiInputMode::from_index(
                self.ui_state.midi_mode.load(std::sync::atomic::Ordering::Relaxed),
            );
            self.midi_mode_processor.set_mode(midi_mode);

            if self.ui_state.midi_clear_memory.swap(false, std::sync::atomic::Ordering::Relaxed) {
                self.midi_mode_processor.clear_accompaniment();
            }

            let pos_beats = transport.pos_beats().unwrap_or(0.0);
            let bar_index = (pos_beats / 4.0).floor().max(0.0) as u64;
            let bar_position = ((pos_beats % 4.0) / 4.0) as f32;

            let external_notes = &self.midi_processor.input.external_notes;
            let mode_result = self.midi_mode_processor.process_events(
                external_notes,
                bar_index,
                bar_position,
            );

            let passthrough_notes: &[midi::ExternalNoteEvent] = match midi_mode {
                MidiInputMode::Passthrough => external_notes,
                _ => &[],
            };

            if let MidiModeResult::NotePoolUpdate(pool) = mode_result {
                synth.update_note_pool(pool);
            }

            if let Ok(mut display) = self.ui_state.midi_mode_display.try_lock() {
                *display = self.midi_mode_processor.get_display();
            }

            let measure_cpu = self.cpu_measure_counter == 0;
            self.cpu_measure_counter = (self.cpu_measure_counter + 1) % 32;

            let start_time = if measure_cpu { Some(std::time::Instant::now()) } else { None };
            synth.process_block(
                &mut self.output_buffer_l,
                &mut self.output_buffer_r,
                &mut self.sub_buffer,
                &self.params,
                pll_feedback_amt,
                base_freq,
                &mut self.midi_events_buffer,
                seq_playing,
                passthrough_notes,
            );

            for (is_note_on, is_note_off, midi_note, velocity, sample_idx) in &self.midi_events_buffer {
                if *is_note_on {
                    self.midi_processor.note_on_from_sequencer(*midi_note, *velocity, *sample_idx as u32);
                } else if *is_note_off {
                    self.midi_processor.note_off_from_sequencer(*sample_idx as u32);
                }
            }

            self.midi_processor.send_output::<PhaseBurn>(
                context,
                is_playing,
                buffer.samples(),
                self.sample_rate,
                tempo,
            );

            let transport_out_enabled = self.ui_state.midi_transport_out.load(std::sync::atomic::Ordering::Relaxed);
            let clock_out_enabled = self.ui_state.midi_clock_out.load(std::sync::atomic::Ordering::Relaxed);

            if transport_out_enabled {
                let seq_just_started = seq_playing && !prev_seq_playing;
                let seq_just_stopped = !seq_playing && prev_seq_playing;
                if seq_just_started {
                    if let Ok(mut q) = self.ui_state.midi_device_output_queue.try_lock() {
                        q.push_back(midi_devices::RawMidiMessage { data: [0xFA, 0, 0], len: 1 });
                    }
                    self.midi_clock_out_phase = 0.0;
                }
                if seq_just_stopped {
                    if let Ok(mut q) = self.ui_state.midi_device_output_queue.try_lock() {
                        q.push_back(midi_devices::RawMidiMessage { data: [0xFC, 0, 0], len: 1 });
                    }
                }
            }

            if clock_out_enabled && seq_playing {
                let ticks_per_second = tempo / 60.0 * 24.0;
                let block_ticks = ticks_per_second * (num_samples as f64 / self.sample_rate as f64);
                let prev_phase = self.midi_clock_out_phase;
                self.midi_clock_out_phase += block_ticks;
                let ticks_to_send = self.midi_clock_out_phase.floor() as u32 - prev_phase.floor() as u32;
                if ticks_to_send > 0 {
                    if let Ok(mut q) = self.ui_state.midi_device_output_queue.try_lock() {
                        for _ in 0..ticks_to_send.min(24) {
                            q.push_back(midi_devices::RawMidiMessage { data: [0xF8, 0, 0], len: 1 });
                        }
                    }
                }
            } else if !seq_playing {
                self.midi_clock_out_phase = 0.0;
            }

            if let Some(start) = start_time {
                let elapsed = start.elapsed();
                let buf_time = buffer.samples() as f32 / self.sample_rate;
                let cpu_load = (elapsed.as_secs_f32() / buf_time) * 100.0;

                let smoothing_time = 1.5;
                let alpha = 1.0 - (-buf_time / smoothing_time).exp();
                self.cpu_load_smoothed = alpha * cpu_load + (1.0 - alpha) * self.cpu_load_smoothed;
                self.ui_state.set_cpu_load(self.cpu_load_smoothed);
            }

            let sub_in_hpf = self.params.master_hpf_sub.value() == 1;

            if sub_in_hpf {
                for i in 0..num_samples {
                    self.output_buffer_l[i] += self.sub_buffer[i];
                    self.output_buffer_r[i] += self.sub_buffer[i];
                }
            }

            self.master_hpf.set_mode(HpfMode::from_index(self.params.master_hpf.value()));
            self.master_hpf.set_boost(HpfBoost::from_index(self.params.master_hpf_boost.value()));
            self.master_hpf.process_block(&mut self.output_buffer_l, &mut self.output_buffer_r);

            if !sub_in_hpf {
                for i in 0..num_samples {
                    self.output_buffer_l[i] += self.sub_buffer[i];
                    self.output_buffer_r[i] += self.sub_buffer[i];
                }
            }

            self.brilliance.set_amount(self.params.brilliance_amount.modulated_plain_value() as f64);
            self.brilliance.set_drive(self.params.brilliance_drive.modulated_plain_value() as f64);
            self.brilliance.process_block(&mut self.output_buffer_l, &mut self.output_buffer_r);

            let linear_volume = self.params.global_volume.modulated_plain_value();
            let target_volume = linear_volume * linear_volume * linear_volume;
            let slew_coeff = 1.0 - (-1.0 / (self.sample_rate * 0.04)).exp();

            for i in 0..num_samples {
                self.volume_slew += (target_volume - self.volume_slew) * slew_coeff;
                self.output_buffer_l[i] *= self.volume_slew;
                self.output_buffer_r[i] *= self.volume_slew;
            }

            if self.params.limiter_enable.value() {
                self.limiter.process_block(&mut self.output_buffer_l, &mut self.output_buffer_r);
                self.ui_state.limiter_latency_samples.store(
                    self.limiter.lookahead_samples() as u32,
                    std::sync::atomic::Ordering::Relaxed,
                );
            } else {
                self.ui_state.limiter_latency_samples.store(0, std::sync::atomic::Ordering::Relaxed);
            }

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

fn apply_midi_learn(
    params: &Arc<DeviceParams>,
    setter: &nih_plug::prelude::ParamSetter,
    ui_state: &Arc<SharedUiState>,
) {
    let midi_learn = &ui_state.midi_learn;

    let learn_mode = midi_learn.learn_mode.load(std::sync::atomic::Ordering::Relaxed);
    if learn_mode == 1 {
        for cc in 0u8..128 {
            if midi_learn.take_changed(cc) {
                midi_learn.selector_cc.store(cc, std::sync::atomic::Ordering::Relaxed);
                midi_learn.learn_mode.store(0, std::sync::atomic::Ordering::Relaxed);
                save_selector_value_ccs(ui_state);
                return;
            }
        }
    } else if learn_mode == 2 {
        for cc in 0u8..128 {
            if midi_learn.take_changed(cc) {
                midi_learn.value_cc.store(cc, std::sync::atomic::Ordering::Relaxed);
                midi_learn.learn_mode.store(0, std::sync::atomic::Ordering::Relaxed);
                save_selector_value_ccs(ui_state);
                return;
            }
        }
    }

    let learn_active = midi_learn.learn_active.load(std::sync::atomic::Ordering::Relaxed);

    if learn_active {
        if let Ok(awaiting) = midi_learn.awaiting_param.try_lock() {
            if awaiting.is_some() {
                for cc in 0u8..128 {
                    if midi_learn.take_changed(cc) {
                        let param_id = awaiting.as_ref().unwrap().clone();
                        drop(awaiting);
                        if let Ok(mut mappings) = midi_learn.mappings.try_lock() {
                            mappings.add(cc, param_id);
                        }
                        if let Ok(mut awaiting) = midi_learn.awaiting_param.try_lock() {
                            *awaiting = None;
                        }
                        save_midi_learn_mappings(ui_state);
                        return;
                    }
                }
            }
        }
    }

    let selector_cc = midi_learn.selector_cc.load(std::sync::atomic::Ordering::Relaxed);
    let value_cc = midi_learn.value_cc.load(std::sync::atomic::Ordering::Relaxed);

    let soft_takeover_enabled = ui_state.soft_takeover.load(std::sync::atomic::Ordering::Relaxed);

    if selector_cc < 128 && midi_learn.take_changed(selector_cc) {
        let val = midi_learn.read_cc(selector_cc);
        let idx = (val * (midi_learn::SOUND_PARAMS.len() - 1) as f32).round() as u8;
        let prev_idx = midi_learn.selected_param_idx.swap(idx, std::sync::atomic::Ordering::Relaxed);
        if idx != prev_idx {
            midi_learn.value_cc_picked_up.store(false, std::sync::atomic::Ordering::Relaxed);
            if value_cc < 128 {
                if let Some(current_val) = params.read_normalized_value(
                    midi_learn::SOUND_PARAMS.get(idx as usize).copied().unwrap_or(""),
                ) {
                    let cc_value = (current_val * 127.0).round() as u8;
                    let out_channel = ui_state.midi_device_manager.try_lock()
                        .map(|mgr| mgr.output_channel())
                        .unwrap_or(0);
                    let msg = midi_devices::RawMidiMessage {
                        data: [0xB0 | out_channel, value_cc, cc_value],
                        len: 3,
                    };
                    if let Ok(mut q) = ui_state.midi_device_output_queue.try_lock() {
                        q.push_back(msg);
                    }
                }
            }
        }
    }

    if value_cc < 128 && midi_learn.take_changed(value_cc) {
        let cc_val = midi_learn.read_cc(value_cc);
        let idx = midi_learn.selected_param_idx.load(std::sync::atomic::Ordering::Relaxed) as usize;
        if idx < midi_learn::SOUND_PARAMS.len() {
            let param_id = midi_learn::SOUND_PARAMS[idx];
            if !soft_takeover_enabled {
                params.apply_normalized_cc(setter, param_id, cc_val);
            } else if midi_learn.value_cc_picked_up.load(std::sync::atomic::Ordering::Relaxed) {
                params.apply_normalized_cc(setter, param_id, cc_val);
            } else if let Some(current) = params.read_normalized_value(param_id) {
                if (cc_val - current).abs() < 0.05 {
                    midi_learn.value_cc_picked_up.store(true, std::sync::atomic::Ordering::Relaxed);
                    params.apply_normalized_cc(setter, param_id, cc_val);
                }
            }
        }
    }

    if let Ok(mappings) = midi_learn.mappings.try_lock() {
        for mapping in &mappings.mappings {
            let cc = mapping.cc_number;
            if midi_learn.take_changed(cc) {
                let value = midi_learn.read_cc(cc);
                params.apply_normalized_cc(setter, &mapping.param_id, value);
            }
        }
    }
}

fn save_midi_learn_mappings(ui_state: &Arc<SharedUiState>) {
    if let Ok(mappings) = ui_state.midi_learn.mappings.try_lock() {
        if let Ok(mut mgr) = ui_state.midi_device_manager.try_lock() {
            mgr.set_midi_learn_mappings(mappings.mappings.clone());
            mgr.save_config();
        }
    }
}

fn save_selector_value_ccs(ui_state: &Arc<SharedUiState>) {
    let sel = ui_state.midi_learn.selector_cc.load(std::sync::atomic::Ordering::Relaxed);
    let val = ui_state.midi_learn.value_cc.load(std::sync::atomic::Ordering::Relaxed);
    if let Ok(mut mgr) = ui_state.midi_device_manager.try_lock() {
        mgr.set_selector_cc(if sel < 128 { Some(sel) } else { None });
        mgr.set_value_cc(if val < 128 { Some(val) } else { None });
        mgr.save_config();
    }
}

impl ClapPlugin for PhaseBurn {
    const CLAP_ID: &'static str = "com.phaseburn-audio.phaseburn";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("A monophonic synthesizer and probability sequencer");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[
        ClapFeature::Instrument,
        ClapFeature::Synthesizer,
        ClapFeature::Stereo,
    ];
}

impl Vst3Plugin for PhaseBurn {
    const VST3_CLASS_ID: [u8; 16] = *b"PhaseBurnAudi01\0";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Instrument, Vst3SubCategory::Synth];
}

nih_plug::nih_export_clap!(PhaseBurn);
nih_plug::nih_export_vst3!(PhaseBurn);
