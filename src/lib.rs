mod params;

use nih_plug::prelude::*;
use nih_plug_egui::egui::style::HandleShape::Rect;
use nih_plug_egui::{create_egui_editor, egui};
use params::DeviceParams;
use std::sync::Arc;

const NUM_SLIDERS: usize = 8;

pub struct Device {
    params: Arc<DeviceParams>,
}

impl Default for Device {
    fn default() -> Self {
        Self {
            params: Arc::new(DeviceParams::default()),
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
                    ui.vertical(|ui| {
                        ui.heading("Device - 4/4 Rhythm Loop");
                        ui.add_space(30.0);

                        let available_width = ui.available_width();
                        let side_margin = 40.0;
                        // let group_width = available_width - (side_margin * 2.0);
                        let group_width = available_width - 80.0;
                        let slider_height = 280.0;

                        let min_gap: f32 = 0.0;
                        let max_slider_width: f32 = 20.0;

                        let min_total_width = NUM_SLIDERS as f32 * 20.0 + (NUM_SLIDERS + 1) as f32 * min_gap;
                        let slider_width = if min_total_width > group_width {
                            ((group_width - (NUM_SLIDERS + 1) as f32 * min_gap) / NUM_SLIDERS as f32).max(10.0)
                        } else {
                            max_slider_width.min((group_width - (NUM_SLIDERS + 1) as f32 * min_gap) / NUM_SLIDERS as f32)
                        };

                        let total_slider_width = slider_width * NUM_SLIDERS as f32;
                        let total_gap_space = group_width - total_slider_width;
                        let num_gaps = NUM_SLIDERS + 1;
                        let gap_spacing = (total_gap_space / num_gaps as f32).max(min_gap);

                        ui.horizontal(|ui| {
                            ui.add_space(side_margin);

                            egui::Frame::group(ui.style())
                                // .inner_margin(8.0)
                                .show(ui, |ui| {
                                    ui.allocate_ui_with_layout(
                                        egui::vec2(group_width - 16.0, slider_height + 10.0),
                                        egui::Layout::left_to_right(egui::Align::Center),
                                        |ui| {
                                            ui.spacing_mut().item_spacing.x = 0.0;

                                    for i in 0..NUM_SLIDERS {
                                        ui.add_space(gap_spacing);

                                        let param = params.get_slider_param(i);
                                        let mut value = param.modulated_plain_value();

                                        if ui
                                            .add_sized(
                                                [slider_width, slider_height],
                                                egui::Slider::new(&mut value, 0.0..=1.0)
                                                    .vertical()
                                                    .trailing_fill(true)
                                                    .smart_aim(true)
                                                    .handle_shape(Rect { aspect_ratio: 0.0 })
                                                    .show_value(false),
                                            )
                                            .changed()
                                        {
                                            setter.begin_set_parameter(param);
                                            setter.set_parameter(param, value);
                                            setter.end_set_parameter(param);
                                        }
                                    }

                                    ui.add_space(gap_spacing);
                                        },
                                    );
                                });
                        });

                        // ui.add_space(20.0);
                    });
                });
            },
        )
    }

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        _buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        true
    }

    fn reset(&mut self) {}

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        for channel_samples in buffer.iter_samples() {
            for sample_out in channel_samples {
                *sample_out = 0.0;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_constants() {
        assert_eq!(Device::NAME, "Device");
        assert_eq!(Device::VENDOR, "Device Audio");
    }

    #[test]
    fn test_default_device() {
        let _device = Device::default();
    }
}
