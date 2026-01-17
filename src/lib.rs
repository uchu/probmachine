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
use midi::MidiCCState;

pub struct Device {
    params: Arc<DeviceParams>,
    synth_engine: Option<SynthEngine>,
    ui_state: Arc<SharedUiState>,
    midi_state: MidiCCState,
    sample_rate: f32,
    cpu_load_smoothed: f32,
    volume_slew: f32,
    output_level_smoothed: f32,
    limiter: MasterLimiter,
}

impl Default for Device {
    fn default() -> Self {
        Self {
            params: Arc::new(DeviceParams::default()),
            synth_engine: None,
            ui_state: Arc::new(SharedUiState::new()),
            midi_state: MidiCCState::new(),
            sample_rate: 44100.0,
            cpu_load_smoothed: 0.0,
            volume_slew: 0.5,
            output_level_smoothed: 0.0,
            limiter: MasterLimiter::new(44100.0),
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
    const MIDI_OUTPUT: MidiConfig = MidiConfig::None;
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
                                    *mem.data.get_temp_mut_or(egui::Id::new("current_page"), Page::BeatProbability)
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
        // Workaround for jack crate bug on macOS: use SAMPLE_RATE env var if set
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

    fn reset(&mut self) {}

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        while let Some(event) = context.next_event() {
            midi::process_midi_event(event, &mut self.midi_state, &self.params);
        }

        if let Some(synth) = &mut self.synth_engine {
            if let Ok(note_pool) = self.ui_state.note_pool.lock() {
                synth.update_note_pool(note_pool.clone());
            }
            if let Ok(strength_values) = self.ui_state.strength_values.lock() {
                synth.update_strength_values(strength_values.clone());
            }
            if let Ok(octave_rand) = self.ui_state.octave_randomization.lock() {
                synth.update_octave_randomization(octave_rand.clone());
            }

            synth.set_osc_params(
                self.params.synth_osc_d.modulated_plain_value(),
                self.params.synth_osc_v.modulated_plain_value(),
            );

            synth.set_osc_volume(self.params.synth_osc_volume.modulated_plain_value());

            synth.set_osc_octave(self.params.synth_osc_octave.value());

            synth.set_vps_stereo_v_offset(self.params.synth_osc_stereo_v_offset.modulated_plain_value());

            synth.set_sub_volume(self.params.synth_sub_volume.modulated_plain_value());

            synth.set_pll_fm_params(
                self.params.synth_pll_fm_amount.modulated_plain_value(),
                self.params.synth_pll_fm_ratio.value(),
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
                self.params.synth_noise_amount.modulated_plain_value(),
                self.params.synth_tube_drive.modulated_plain_value(),
                self.params.synth_color_distortion_amount.modulated_plain_value(),
                self.params.synth_color_distortion_threshold.modulated_plain_value(),
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

            synth.set_pll_volume(self.params.synth_pll_volume.modulated_plain_value());

            synth.set_pll_stereo_damp_offset(self.params.synth_pll_stereo_damp_offset.modulated_plain_value());

            synth.set_pll_glide(self.params.synth_pll_glide.modulated_plain_value());

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

            // VPS dry/wet removed - reverb now has its own mix control

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

            // LFO 1 params
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

            // LFO 2 params
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

            // LFO 3 params
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

            let mut output_l = vec![0.0; buffer.samples()];
            let mut output_r = vec![0.0; buffer.samples()];

            let pll_feedback_amt = self.params.synth_pll_feedback.modulated_plain_value();
            let base_freq = 220.0;

            let start_time = std::time::Instant::now();
            synth.process_block(&mut output_l, &mut output_r, &self.params, pll_feedback_amt, base_freq);
            let elapsed = start_time.elapsed();

            let buffer_time_secs = buffer.samples() as f32 / self.sample_rate;
            let cpu_load = (elapsed.as_secs_f32() / buffer_time_secs) * 100.0;

            let smoothing_time = 1.5;
            let alpha = 1.0 - (-buffer_time_secs / smoothing_time).exp();
            self.cpu_load_smoothed = alpha * cpu_load + (1.0 - alpha) * self.cpu_load_smoothed;
            self.ui_state.set_cpu_load(self.cpu_load_smoothed);

            let target_volume = self.params.global_volume.modulated_plain_value();
            let slew_coeff = 1.0 - (-1.0 / (self.sample_rate * 0.01)).exp();

            for i in 0..output_l.len() {
                self.volume_slew += (target_volume - self.volume_slew) * slew_coeff;
                output_l[i] *= self.volume_slew;
                output_r[i] *= self.volume_slew;
            }

            if self.params.limiter_enable.value() {
                self.limiter.process_block(&mut output_l, &mut output_r);
            }

            let mut peak: f32 = 0.0;
            for (i, channel_samples) in buffer.iter_samples().enumerate() {
                peak = peak.max(output_l[i].abs()).max(output_r[i].abs());

                let mut iter = channel_samples.into_iter();
                if let Some(left) = iter.next() {
                    *left = output_l[i];
                }
                if let Some(right) = iter.next() {
                    *right = output_r[i];
                }
            }

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
