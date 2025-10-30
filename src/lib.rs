#![feature(portable_simd)]

mod params;
mod ui;
mod synth;
mod sequencer;

use egui_taffy::taffy::{
    prelude::*,
    style::{AlignItems, FlexDirection, JustifyContent},
};
use egui_taffy::{tui as taffy_layout, TuiBuilderLogic};
use nih_plug::prelude::*;
use nih_plug_egui::{create_egui_editor, egui};
use params::DeviceParams;
use std::sync::Arc;
use ui::Page;
use synth::SynthEngine;

pub struct Device {
    params: Arc<DeviceParams>,
    synth_engine: Option<SynthEngine>,
}

impl Default for Device {
    fn default() -> Self {
        Self {
            params: Arc::new(DeviceParams::default()),
            synth_engine: None,
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

    const MIDI_INPUT: MidiConfig = MidiConfig::None;
    const MIDI_OUTPUT: MidiConfig = MidiConfig::None;
    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        let params = self.params.clone();
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
                                ui::navigation::render(ui, &mut current_page);
                            });

                            tui.ui(|ui| {
                                ui.memory_mut(|mem| {
                                    mem.data.insert_temp(egui::Id::new("current_page"), current_page);
                                });
                            });

                            current_page.render(tui, &params, setter);
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
        self.synth_engine = Some(SynthEngine::new(buffer_config.sample_rate));
        true
    }

    fn reset(&mut self) {}

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        if let Some(synth) = &mut self.synth_engine {
            synth.set_osc_params(
                self.params.synth_osc_d.modulated_plain_value(),
                self.params.synth_osc_v.modulated_plain_value(),
            );

            synth.set_osc_volume(self.params.synth_osc_volume.modulated_plain_value());

            synth.set_osc_octave(self.params.synth_osc_octave.value());

            synth.set_vps_stereo_v_offset(self.params.synth_osc_stereo_v_offset.modulated_plain_value());

            synth.set_sub_params(
                self.params.synth_sub_volume.modulated_plain_value(),
                self.params.synth_sub_octave.value(),
                self.params.synth_sub_shape.modulated_plain_value(),
            );

            synth.set_polyblep_params(
                self.params.synth_polyblep_volume.modulated_plain_value(),
                self.params.synth_polyblep_pulse_width.modulated_plain_value(),
                self.params.synth_polyblep_octave.value(),
            );

            synth.set_polyblep_stereo_width(self.params.synth_polyblep_stereo_width.modulated_plain_value());

            synth.set_pll_ref_params(
                self.params.synth_pll_ref_octave.value(),
                self.params.synth_pll_ref_tune.value(),
                self.params.synth_pll_ref_fine_tune.modulated_plain_value(),
                self.params.synth_pll_ref_pulse_width.modulated_plain_value(),
            );

            let pll_range = self.params.synth_pll_range.modulated_plain_value();
            let pll_mult = match self.params.synth_pll_mult.value() {
                0 => 1.0,
                1 => 2.0,
                2 => 4.0,
                3 => 8.0,
                4 => 16.0,
                _ => 1.0,
            };

            synth.set_pll_params(
                self.params.synth_pll_track_speed.modulated_plain_value(),
                self.params.synth_pll_damping.modulated_plain_value(),
                pll_mult,
                pll_range,
                self.params.synth_pll_colored.value(),
                self.params.synth_pll_mode.value(),
            );

            synth.set_pll_volume(self.params.synth_pll_volume.modulated_plain_value());

            synth.set_pll_ki_multiplier(self.params.synth_pll_ki_multiplier.modulated_plain_value());

            synth.set_pll_stereo_damp_offset(self.params.synth_pll_stereo_damp_offset.modulated_plain_value());

            synth.set_pll_distortion_params(
                self.params.synth_pll_distortion_amount.modulated_plain_value(),
                self.params.synth_pll_distortion_threshold.modulated_plain_value(),
            );

            synth.set_distortion_params(
                self.params.synth_distortion_amount.modulated_plain_value(),
                self.params.synth_distortion_threshold.modulated_plain_value(),
            );

            synth.set_filter_params(
                self.params.synth_filter_cutoff.modulated_plain_value(),
                self.params.synth_filter_resonance.modulated_plain_value(),
                self.params.synth_filter_env_amount.modulated_plain_value(),
                self.params.synth_filter_drive.modulated_plain_value(),
                self.params.synth_filter_mode.value(),
            );

            synth.set_volume(self.params.synth_volume.modulated_plain_value());

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

            let mut output_l = vec![0.0; buffer.samples()];
            let mut output_r = vec![0.0; buffer.samples()];

            let pll_feedback_amt = self.params.synth_pll_feedback.modulated_plain_value();
            let base_freq = 220.0;

            synth.process_block(&mut output_l, &mut output_r, &self.params, pll_feedback_amt, base_freq);

            for (i, channel_samples) in buffer.iter_samples().enumerate() {
                let mut iter = channel_samples.into_iter();
                if let Some(left) = iter.next() {
                    *left = output_l[i];
                }
                if let Some(right) = iter.next() {
                    *right = output_r[i];
                }
            }
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
