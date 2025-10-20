mod params;

use egui_taffy::taffy::{
    prelude::*,
    style::{AlignItems, FlexDirection, JustifyContent},
};
use egui_taffy::{tui as taffy_layout, TuiBuilderLogic};
use nih_plug::prelude::*;
use nih_plug_egui::egui::style::HandleShape;
use nih_plug_egui::egui::{Color32, Shadow};
use nih_plug_egui::{create_egui_editor, egui};
use params::{BeatMode, DeviceParams};
use std::sync::Arc;

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
                    let outer_padding = 20.0;
                    let frame_margin = 0.0;
                    let container_height = 300.0;

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
                                        if ui
                                            .selectable_label(
                                                selected_tab == i,
                                                format!("Something {}", i + 1),
                                            )
                                            .clicked()
                                        {
                                            ui.memory_mut(|mem| {
                                                mem.data
                                                    .insert_temp(ui.id().with("selected_tab"), i);
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
                                ui.set_min_size(egui::vec2(742.0, container_height));
                                ui.set_max_width(742.0);
                                egui::Frame::default()
                                    .fill(ui.visuals().extreme_bg_color)
                                    .inner_margin(frame_margin)
                                    .stroke(egui::Stroke::new(
                                        1.0,
                                        ui.visuals().window_stroke.color,
                                    ))
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
                                        let container_width = 738.0;
                                        let num_grid_positions = 33;
                                        let slider_width =
                                            container_width / num_grid_positions as f32;

                                        for i in 0..num_grid_positions {
                                            let x = container_rect.min.x
                                                + i as f32 * slider_width
                                                + (slider_width / 2.0);
                                            let line_num = i + 1;

                                            let color = if (line_num - 1) % 8 == 0 {
                                                Color32::from_white_alpha(7)
                                            } else if line_num % 4 == 1 {
                                                Color32::from_white_alpha(3)
                                            } else if line_num % 2 == 1 {
                                                Color32::from_white_alpha(2)
                                            } else {
                                                Color32::from_white_alpha(1)
                                            };

                                            painter.line_segment(
                                                [
                                                    egui::pos2(x, container_rect.min.y),
                                                    egui::pos2(x, container_rect.max.y),
                                                ],
                                                egui::Stroke::new(1.0, color),
                                            );
                                        }

                                        ui.vertical(|ui| {
                                            ui.set_min_size(egui::vec2(738.0, container_height));
                                            ui.set_max_width(738.0);
                                            ui.add_space(24.0);

                                            let (beat_mode, num_sliders) = ui.memory_mut(|mem| {
                                                let mode = *mem.data.get_temp_mut_or(
                                                    egui::Id::new("beat_mode"),
                                                    BeatMode::Straight,
                                                );
                                                let mut sliders = *mem.data.get_temp_mut_or(
                                                    egui::Id::new("num_sliders"),
                                                    NUM_SLIDERS,
                                                );

                                                if !DeviceParams::is_valid_beat_count(mode, sliders)
                                                {
                                                    sliders =
                                                        DeviceParams::get_default_beat_count(mode);
                                                    mem.data.insert_temp(
                                                        egui::Id::new("num_sliders"),
                                                        sliders,
                                                    );
                                                }

                                                (mode, sliders)
                                            });

                                            ui.horizontal_top(|ui| {
                                                ui.add_space(2.0);
                                                for i in 0..num_sliders {
                                                    ui.vertical(|ui| {
                                                        let param = params.get_division_param(
                                                            beat_mode,
                                                            num_sliders,
                                                            i,
                                                        );
                                                        let mut value =
                                                            param.modulated_plain_value();
                                                        ui.style_mut().spacing.slider_width = 250.0;
                                                        ui.style_mut().spacing.slider_rail_height =
                                                            9.0;
                                                        if ui
                                                            .add(
                                                                egui::Slider::new(
                                                                    &mut value,
                                                                    0.0..=127.0,
                                                                )
                                                                .vertical()
                                                                .trailing_fill(true)
                                                                .step_by(1.0)
                                                                .handle_shape(HandleShape::Rect {
                                                                    aspect_ratio: 0.0,
                                                                })
                                                                .show_value(false),
                                                            )
                                                            .changed()
                                                        {
                                                            setter.begin_set_parameter(param);
                                                            setter.set_parameter(param, value);
                                                            setter.end_set_parameter(param);
                                                        }
                                                    });

                                                    let space = match (beat_mode, num_sliders) {
                                                        (_, 1) => 0.0,
                                                        (BeatMode::Straight, 2) => 332.0,
                                                        (BeatMode::Dotted, 2) => 440.0,
                                                        (_, 3) => 215.0,
                                                        (_, 4) => 153.0,
                                                        (_, 5) => 121.0,
                                                        (_, 6) => 92.0,
                                                        (_, 8) => 63.5,
                                                        (_, 10) => 46.0,
                                                        (_, 12) => 33.5,
                                                        (_, 16) => 18.75,
                                                        (_, 21) => 7.5,
                                                        (_, 24) => 3.4,
                                                        (_, 32) => -3.63,
                                                        _ => {
                                                            panic!(
                                                                "Invalid division: {}",
                                                                num_sliders
                                                            )
                                                        }
                                                    };
                                                    ui.add_space(space);
                                                }
                                            });
                                        });
                                    });
                            });

                            tui.ui(|ui| {
                                ui.add_space(16.0);
                                ui.horizontal(|ui| {
                                    let (beat_mode, num_sliders) = ui.memory_mut(|mem| {
                                        let mode = *mem.data.get_temp_mut_or(
                                            egui::Id::new("beat_mode"),
                                            BeatMode::Straight,
                                        );
                                        let mut sliders = *mem.data.get_temp_mut_or(
                                            egui::Id::new("num_sliders"),
                                            NUM_SLIDERS,
                                        );

                                        if !DeviceParams::is_valid_beat_count(mode, sliders) {
                                            sliders = DeviceParams::get_default_beat_count(mode);
                                            mem.data
                                                .insert_temp(egui::Id::new("num_sliders"), sliders);
                                        }

                                        (mode, sliders)
                                    });

                                    let divisions = DeviceParams::get_divisions_for_mode(beat_mode);
                                    let mode_suffix = beat_mode.as_str();

                                    match beat_mode {
                                        BeatMode::Straight => {}
                                        BeatMode::Triplet | BeatMode::Dotted => {
                                            ui.add_enabled(
                                                false,
                                                egui::Button::new("")
                                                    .min_size(egui::vec2(60.0, 32.0)),
                                            );
                                        }
                                    }

                                    for (count, label) in divisions.iter() {
                                        let button_label = if beat_mode == BeatMode::Straight {
                                            label.to_string()
                                        } else {
                                            format!("{}{}", label, mode_suffix)
                                        };

                                        let button = egui::Button::new(
                                            egui::RichText::new(button_label).size(16.0),
                                        )
                                        .min_size(egui::vec2(60.0, 32.0))
                                        .selected(num_sliders == *count);

                                        if ui.add(button).clicked() {
                                            ui.memory_mut(|mem| {
                                                mem.data.insert_temp(
                                                    egui::Id::new("num_sliders"),
                                                    *count,
                                                );
                                            });
                                        }
                                    }

                                    if beat_mode == BeatMode::Triplet {
                                        ui.add_enabled(
                                            false,
                                            egui::Button::new("").min_size(egui::vec2(60.0, 32.0)),
                                        );
                                    }

                                    ui.add_space(40.0);

                                    let button_s =
                                        egui::Button::new(egui::RichText::new("S").size(16.0))
                                            .min_size(egui::vec2(60.0, 32.0))
                                            .selected(beat_mode == BeatMode::Straight);

                                    if ui.add(button_s).clicked() {
                                        ui.memory_mut(|mem| {
                                            mem.data.insert_temp(
                                                egui::Id::new("beat_mode"),
                                                BeatMode::Straight,
                                            );
                                            let default_division = 4;
                                            mem.data.insert_temp(
                                                egui::Id::new("num_sliders"),
                                                default_division,
                                            );
                                        });
                                    }

                                    let button_t =
                                        egui::Button::new(egui::RichText::new("T").size(16.0))
                                            .min_size(egui::vec2(60.0, 32.0))
                                            .selected(beat_mode == BeatMode::Triplet);

                                    if ui.add(button_t).clicked() {
                                        ui.memory_mut(|mem| {
                                            mem.data.insert_temp(
                                                egui::Id::new("beat_mode"),
                                                BeatMode::Triplet,
                                            );
                                            let default_division = 6;
                                            mem.data.insert_temp(
                                                egui::Id::new("num_sliders"),
                                                default_division,
                                            );
                                        });
                                    }

                                    let button_d =
                                        egui::Button::new(egui::RichText::new("D").size(16.0))
                                            .min_size(egui::vec2(60.0, 32.0))
                                            .selected(beat_mode == BeatMode::Dotted);

                                    if ui.add(button_d).clicked() {
                                        ui.memory_mut(|mem| {
                                            mem.data.insert_temp(
                                                egui::Id::new("beat_mode"),
                                                BeatMode::Dotted,
                                            );
                                            let default_division = 2;
                                            mem.data.insert_temp(
                                                egui::Id::new("num_sliders"),
                                                default_division,
                                            );
                                        });
                                    }
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
