mod params;

use egui_taffy::taffy::{
    prelude::*,
    style::{AlignItems, FlexDirection, JustifyContent},
};
use egui_taffy::{tui as taffy_layout, TuiBuilderLogic};
use nih_plug::prelude::*;
use nih_plug_egui::egui::style::HandleShape;
use nih_plug_egui::egui::{Color32, Margin};
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
                    let frame_margin = 0.0;
                    let container_height = 276.0;

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
                            tui.ui(|ui| {
                                egui::Frame::default()
                                    .outer_margin(Margin{left:-30, right:0, top: -10, bottom: 0})
                                    .inner_margin(Margin{left:30, right:30, top:8, bottom: 8})
                                    .fill(Color32::BLACK)
                                    .show(ui, |ui| {
                                        ui.set_min_width(800.0);
                                        ui.set_max_width(800.0);
                                        ui.horizontal(|ui| {
                                            for i in 0..7 {
                                                ui.add(
                                                    egui::Button::new(
                                                        egui::RichText::new(format!("Some {}", i))
                                                            .size(14.0),
                                                    )
                                                    .min_size(egui::vec2(60.0, 32.0)),
                                                );
                                            }
                                        });
                                    });
                            });

                            let (beat_mode, num_sliders) = tui.ui(|ui| {
                                let (mode, sliders) = ui.memory_mut(|mem| {
                                    let mode = *mem.data.get_temp_mut_or(
                                        egui::Id::new("beat_mode"),
                                        BeatMode::Straight,
                                    );
                                    let mut sliders = *mem
                                        .data
                                        .get_temp_mut_or(egui::Id::new("num_sliders"), NUM_SLIDERS);

                                    if !DeviceParams::is_valid_beat_count(mode, sliders) {
                                        sliders = DeviceParams::get_default_beat_count(mode);
                                        mem.data.insert_temp(egui::Id::new("num_sliders"), sliders);
                                    }

                                    (mode, sliders)
                                });

                                ui.add_space(12.0);
                                ui.heading(egui::RichText::new("    Beat Probability").size(14.0));
                                ui.add_space(8.0);

                                (mode, sliders)
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
                                    .show(ui, |ui| {
                                        let container_rect = ui.available_rect_before_wrap();
                                        let painter = ui.painter();
                                        let container_width = 738.0;
                                        let grid_padding = 10.0;
                                        let grid_width = container_width - (grid_padding * 2.0);

                                        let (num_v_grid_positions, grid_spaces) = match beat_mode {
                                            BeatMode::Straight | BeatMode::Dotted => (33, 32.0),
                                            BeatMode::Triplet => (25, 24.0),
                                        };
                                        let slider_width = grid_width / grid_spaces;

                                        for i in 0..num_v_grid_positions {
                                            let x = container_rect.min.x
                                                + grid_padding
                                                + i as f32 * slider_width;
                                            let line_num = i + 1;

                                            let color = match beat_mode {
                                                BeatMode::Straight | BeatMode::Dotted => {
                                                    if (line_num - 1) % 8 == 0 {
                                                        Color32::from_rgb(40, 40, 40)
                                                    } else if line_num % 4 == 1 {
                                                        Color32::from_rgb(25, 25, 25)
                                                    } else if line_num % 2 == 1 {
                                                        Color32::from_rgb(20, 20, 20)
                                                    } else {
                                                        Color32::from_rgb(15, 15, 15)
                                                    }
                                                }
                                                BeatMode::Triplet => {
                                                    let beat_interval = 24 / num_sliders;

                                                    if i % beat_interval == 0 {
                                                        Color32::from_rgb(40, 40, 40)
                                                    } else if i % 3 == 0 {
                                                        Color32::from_rgb(22, 22, 22)
                                                    } else {
                                                        Color32::from_rgb(15, 15, 15)
                                                    }
                                                }
                                            };

                                            painter.line_segment(
                                                [
                                                    egui::pos2(x, container_rect.min.y + 1.0),
                                                    egui::pos2(x, container_rect.max.y - 1.0),
                                                ],
                                                egui::Stroke::new(1.0, color),
                                            );
                                        }

                                        for i in 0..5 {
                                            let y = container_rect.min.y
                                                + 10.0
                                                + i as f32 * (container_height - 20.0) / 4.0;
                                            painter.line_segment(
                                                [
                                                    egui::pos2(container_rect.min.x + 10.0, y),
                                                    egui::pos2(container_rect.max.x - 12.5, y),
                                                ],
                                                egui::Stroke::new(
                                                    1.0,
                                                    Color32::from_rgb(20, 20, 20),
                                                ),
                                            );
                                        }

                                        let grid_padding = 10.0;
                                        let grid_width = container_width - (grid_padding * 2.0);

                                        ui.vertical(|ui| {
                                            ui.set_min_size(egui::vec2(738.0, container_height));
                                            ui.set_max_width(738.0);
                                            ui.add_space(10.0);

                                            ui.horizontal_top(|ui| {
                                                let grid_base = match beat_mode {
                                                    BeatMode::Straight => 32.0,
                                                    BeatMode::Triplet => 24.0,
                                                    BeatMode::Dotted => 32.0,
                                                };

                                                for i in 0..num_sliders {
                                                    let grid_pos = match beat_mode {
                                                        BeatMode::Straight => {
                                                            i as f32
                                                                * (grid_base / num_sliders as f32)
                                                        }
                                                        BeatMode::Triplet => {
                                                            i as f32
                                                                * (grid_base / num_sliders as f32)
                                                        }
                                                        BeatMode::Dotted => {
                                                            let dotted_duration = match num_sliders
                                                            {
                                                                2 => 24.0,
                                                                3 => 12.0,
                                                                6 => 6.0,
                                                                11 => 3.0,
                                                                22 => 1.5,
                                                                _ => panic!(
                                                                    "Invalid dotted division: {}",
                                                                    num_sliders
                                                                ),
                                                            };
                                                            i as f32 * dotted_duration
                                                        }
                                                    };

                                                    let grid_spaces = match beat_mode {
                                                        BeatMode::Straight | BeatMode::Dotted => {
                                                            32.0
                                                        }
                                                        BeatMode::Triplet => 24.0,
                                                    };
                                                    let slider_width_for_pos =
                                                        grid_width / grid_spaces;
                                                    let target_x = grid_padding
                                                        + grid_pos * slider_width_for_pos;
                                                    let current_x = ui.cursor().min.x - 24.0;
                                                    let space_needed = target_x - current_x;

                                                    ui.add_space(space_needed);

                                                    ui.vertical(|ui| {
                                                        let param = params.get_division_param(
                                                            beat_mode,
                                                            num_sliders,
                                                            i,
                                                        );
                                                        let mut value =
                                                            param.modulated_plain_value();
                                                        ui.style_mut().spacing.slider_width = 256.0;
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
                                                }
                                            });
                                        });
                                    });
                            });

                            tui.ui(|ui| {
                                ui.add_space(12.0);
                                egui::Frame::default()
                                    .fill(Color32::from_rgb(30, 30, 30))
                                    .inner_margin(8.0)
                                    .stroke(egui::Stroke::new(1.0, Color32::from_rgb(40, 40, 40)))
                                    .corner_radius(15.0)
                                    .show(ui, |ui| {
                                        ui.vertical_centered(|ui| {
                                            ui.horizontal(|ui| {
                                                ui.add_space(5.0);
                                                let divisions =
                                                    DeviceParams::get_divisions_for_mode(beat_mode);
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
                                                    let button_label =
                                                        if beat_mode == BeatMode::Straight {
                                                            label.to_string()
                                                        } else {
                                                            format!("{}{}", label, mode_suffix)
                                                        };

                                                    let button = egui::Button::new(
                                                        egui::RichText::new(button_label)
                                                            .size(14.0),
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
                                                        egui::Button::new("")
                                                            .min_size(egui::vec2(60.0, 32.0)),
                                                    );
                                                }
                                                ui.add_space(5.0);
                                            });

                                            ui.add_space(5.0);

                                            ui.horizontal(|ui| {
                                                ui.add_space(102.0);
                                                let button_s = egui::Button::new(
                                                    egui::RichText::new("S").size(14.0),
                                                )
                                                .min_size(egui::vec2(60.0, 32.0))
                                                .selected(beat_mode == BeatMode::Straight);

                                                if ui.add(button_s).clicked()
                                                    && beat_mode != BeatMode::Straight
                                                {
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

                                                let button_t = egui::Button::new(
                                                    egui::RichText::new("T").size(14.0),
                                                )
                                                .min_size(egui::vec2(60.0, 32.0))
                                                .selected(beat_mode == BeatMode::Triplet);

                                                if ui.add(button_t).clicked()
                                                    && beat_mode != BeatMode::Triplet
                                                {
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

                                                let button_d = egui::Button::new(
                                                    egui::RichText::new("D").size(14.0),
                                                )
                                                .min_size(egui::vec2(60.0, 32.0))
                                                .selected(beat_mode == BeatMode::Dotted);

                                                if ui.add(button_d).clicked()
                                                    && beat_mode != BeatMode::Dotted
                                                {
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
