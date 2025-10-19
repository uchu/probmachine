mod params;

use nih_plug::prelude::*;
use nih_plug_egui::egui::style::HandleShape;
use nih_plug_egui::{create_egui_editor, egui};
use params::DeviceParams;
use std::sync::Arc;
use egui_taffy::taffy::{
    prelude::*,
    style::{AlignItems, FlexDirection, JustifyContent},
};
use egui_taffy::{tui as taffy_layout, TuiBuilderLogic};
use nih_plug_egui::egui::{Color32, Shadow};

const NUM_SLIDERS: usize = 4;

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
                    let slider_height = 300.0;
                    let max_slider_width = 10.0;
                    let outer_padding = 20.0;
                    let frame_margin = 0.0;
                    let inner_padding = 0.0;

                    taffy_layout(ui, ui.id().with("page_layout"))
                        .reserve_available_space()
                        .style(Style {
                            display: Display::Flex,
                            flex_direction: FlexDirection::Column,
                            justify_content: Some(JustifyContent::SpaceBetween),
                            align_items: Some(AlignItems::Stretch),
                            padding: egui_taffy::taffy::Rect {
                                left: length(outer_padding),
                                right: length(outer_padding),
                                top: length(0.0),
                                bottom: length(outer_padding),
                            },
                            gap: Size {
                                width: length(0.0),
                                height: length(0.0),
                            },
                            ..Default::default()
                        })
                        .show(|tui| {

                            tui.ui(|ui| {
                                ui.horizontal(|ui| {
                                    let selected_tab = ui.memory_mut(|mem| {
                                        *mem.data.get_temp_mut_or(ui.id().with("selected_tab"), 0)
                                    });

                                    for i in 0..7 {
                                        if ui.selectable_label(selected_tab == i, format!("Something {}", i + 1)).clicked() {
                                            ui.memory_mut(|mem| {
                                                mem.data.insert_temp(ui.id().with("selected_tab"), i);
                                            });
                                        }
                                    }
                                });
                            });


                            tui.ui(|ui| {
                                ui.add_space(24.0);
                                ui.heading("   Beat Probability");
                                ui.add_space(8.0);
                            });

                            tui.style(Style {
                                flex_grow: 1.0,
                                align_items: Some(AlignItems::Stretch),
                                ..Default::default()
                            })
                            .ui(|ui| {
                                ui.set_min_size(egui::vec2(742.0, slider_height));
                                ui.set_max_width(742.0);
                                egui::Frame::default()
                                    .fill(ui.visuals().extreme_bg_color)
                                    .inner_margin(frame_margin)
                                    .stroke(egui::Stroke::new(1.0, ui.visuals().window_stroke.color))
                                    .corner_radius(15.0)
                                    .shadow(Shadow {
                                        offset: [0, 4],
                                        blur: 12,
                                        spread: 0,
                                        color: Color32::from_black_alpha(80),
                                    })
                                    .show(ui, |ui| {
                                        let container_rect = ui.available_rect_before_wrap();
                                        let painter = ui.painter();
                                        let container_width = 732.0;
                                        let line_spacing = container_width / 32.0;

                                        for i in 0..32 {
                                            let x = container_rect.min.x + (i as f32 * line_spacing);
                                            let line_num = i + 1;

                                            let (color, width) = if (line_num - 1) % 4 == 0 {
                                                (Color32::from_white_alpha(120), 2.0)
                                            } else if line_num % 2 == 1 {
                                                (Color32::from_white_alpha(40), 1.0)
                                            } else {
                                                (Color32::from_white_alpha(15), 1.0)
                                            };

                                            painter.line_segment(
                                                [
                                                    egui::pos2(x, container_rect.min.y),
                                                    egui::pos2(x, container_rect.max.y),
                                                ],
                                                egui::Stroke::new(width, color),
                                            );
                                        }

                                        taffy_layout(ui, ui.id().with("sliders_container"))
                                            .reserve_available_space()
                                            .style(Style {
                                                display: Display::Flex,
                                                flex_direction: FlexDirection::Row,
                                                justify_content: Some(JustifyContent::SpaceEvenly),
                                                align_items: Some(AlignItems::Center),
                                                padding: egui_taffy::taffy::Rect {
                                                    left: length(inner_padding),
                                                    right: length(inner_padding),
                                                    top: length(24.0),
                                                    bottom: length(0.0),
                                                },
                                                size: Size {
                                                    width: length(732.0),
                                                    height: length(slider_height),
                                                },
                                                ..Default::default()
                                            })
                                            .show(|sliders_tui| {
                                                for i in 0..NUM_SLIDERS {
                                                    sliders_tui
                                                        .style(Style {
                                                            size: Size {
                                                                width: length(max_slider_width),
                                                                height: length(slider_height),
                                                            },
                                                            ..Default::default()
                                                        })
                                                        .ui(|ui| {
                                                            let param = params.get_slider_param(i);
                                                            let mut value = param.modulated_plain_value();

                                                            if ui
                                                                .add_sized(
                                                                    [max_slider_width, slider_height],
                                                                    egui::Slider::new(&mut value, 0.0..=1.0)
                                                                        .vertical()
                                                                        .trailing_fill(true)
                                                                        .smart_aim(true)
                                                                        .handle_shape(HandleShape::Rect { aspect_ratio: 0.0 })
                                                                        .show_value(false),
                                                                )
                                                                .changed()
                                                            {
                                                                setter.begin_set_parameter(param);
                                                                setter.set_parameter(param, value);
                                                                setter.end_set_parameter(param);
                                                            }
                                                        });
                                                }
                                            });
                                    });
                            });

                            tui.ui(|ui| {
                                ui.horizontal(|ui| {
                                    ui.label("v0.1.0");
                                    ui.separator();
                                    ui.label("Device Audio");
                                });
                            });
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
