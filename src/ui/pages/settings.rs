use crate::params::DeviceParams;
use crate::ui::SharedUiState;
use crate::midi_modes::MidiInputMode;
use crate::midi_devices::MidiChannel;
use egui_taffy::TuiBuilderLogic;
use nih_plug::prelude::ParamSetter;
use nih_plug_egui::egui;
use nih_plug_egui::egui::Color32;
use std::sync::Arc;
use std::sync::atomic::Ordering;

const HEADER_FONT: f32 = 18.0;
const UI_FONT: f32 = 16.0;
const HINT_FONT: f32 = 13.0;
const SECTION_GAP: f32 = 12.0;
const COL_GAP: f32 = 32.0;
const MARGIN: f32 = 20.0;

pub fn render(
    tui: &mut egui_taffy::Tui,
    params: &Arc<DeviceParams>,
    setter: &ParamSetter,
    ui_state: &Arc<SharedUiState>,
) {
    use egui_taffy::taffy::{prelude::*, style::AlignItems};

    tui.style(Style {
        flex_grow: 1.0,
        align_items: Some(AlignItems::Stretch),
        ..Default::default()
    })
    .ui(|ui| {
        let screen_rect = ui.ctx().screen_rect();
        let top_y = ui.cursor().min.y;

        let content_rect = egui::Rect::from_min_max(
            egui::pos2(screen_rect.left() + MARGIN, top_y + MARGIN),
            egui::pos2(screen_rect.right() - MARGIN, screen_rect.bottom() - MARGIN),
        );

        let col_w = (content_rect.width() - COL_GAP) / 2.0;

        let left_rect = egui::Rect::from_min_size(
            content_rect.min,
            egui::vec2(col_w, content_rect.height()),
        );
        let right_rect = egui::Rect::from_min_size(
            egui::pos2(content_rect.min.x + col_w + COL_GAP, content_rect.min.y),
            egui::vec2(col_w, content_rect.height()),
        );

        let sep_x = content_rect.min.x + col_w + COL_GAP / 2.0;
        ui.painter().line_segment(
            [
                egui::pos2(sep_x, content_rect.top() + 4.0),
                egui::pos2(sep_x, content_rect.bottom() - 4.0),
            ],
            egui::Stroke::new(1.0, Color32::from_gray(50)),
        );

        ui.allocate_new_ui(egui::UiBuilder::new().max_rect(left_rect), |ui| {
            render_midi_devices_section(ui, ui_state);
            section_separator(ui);
            render_midi_input_section(ui, ui_state);
            section_separator(ui);
            render_midi_sync_section(ui, ui_state);
        });

        ui.allocate_new_ui(egui::UiBuilder::new().max_rect(right_rect), |ui| {
            render_midi_learn_section(ui, ui_state);
            section_separator(ui);
            render_performance_section(ui, params, setter, ui_state);
        });
    });
}

fn section_separator(ui: &mut egui::Ui) {
    ui.add_space(SECTION_GAP);
    ui.separator();
    ui.add_space(SECTION_GAP);
}

fn render_midi_devices_section(ui: &mut egui::Ui, ui_state: &Arc<SharedUiState>) {
    ui.label(egui::RichText::new("MIDI DEVICES").size(HEADER_FONT).strong());
    ui.add_space(8.0);

    let Ok(mut mgr) = ui_state.midi_device_manager.try_lock() else {
        ui.label(egui::RichText::new("Loading...").size(UI_FONT).weak());
        return;
    };

    if ui.add(
        egui::Button::new(egui::RichText::new("Refresh Devices").size(UI_FONT))
            .min_size(egui::vec2(140.0, 36.0))
    ).clicked() {
        mgr.refresh_devices();
    }

    ui.add_space(8.0);

    let input_devices: Vec<String> = mgr.input_devices().iter().map(|d| d.name.clone()).collect();
    let output_devices: Vec<String> = mgr.output_devices().iter().map(|d| d.name.clone()).collect();
    let current_input = mgr.connected_input_name().map(|s| s.to_string());
    let current_output = mgr.connected_output_name().map(|s| s.to_string());
    let current_in_channel = mgr.input_channel().clone();
    let current_out_channel = mgr.output_channel();
    let feedback_risk = mgr.has_feedback_risk();

    let input_label = current_input.as_deref().unwrap_or("None");
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("Input:").size(UI_FONT));
        egui::ComboBox::from_id_salt("midi_input_device")
            .width(260.0)
            .selected_text(egui::RichText::new(input_label).size(UI_FONT))
            .show_ui(ui, |ui| {
                let none_btn = egui::Button::new(egui::RichText::new("None").size(UI_FONT))
                    .min_size(egui::vec2(240.0, 36.0))
                    .selected(current_input.is_none());
                if ui.add(none_btn).clicked() {
                    mgr.disconnect_input();
                    mgr.save_config();
                    ui.close_menu();
                }
                for name in &input_devices {
                    let selected = current_input.as_deref() == Some(name.as_str());
                    let btn = egui::Button::new(egui::RichText::new(name).size(UI_FONT))
                        .min_size(egui::vec2(240.0, 36.0))
                        .selected(selected);
                    if ui.add(btn).clicked() {
                        mgr.connect_input(name);
                        mgr.save_config();
                        ui.close_menu();
                    }
                }
            });
        ui.add_space(12.0);
        ui.label(egui::RichText::new("Ch:").size(UI_FONT));
        egui::ComboBox::from_id_salt("midi_input_channel")
            .width(80.0)
            .selected_text(egui::RichText::new(current_in_channel.label()).size(UI_FONT))
            .show_ui(ui, |ui| {
                for ch in MidiChannel::all_options() {
                    let selected = ch == current_in_channel;
                    let btn = egui::Button::new(egui::RichText::new(ch.label()).size(UI_FONT))
                        .min_size(egui::vec2(70.0, 36.0))
                        .selected(selected);
                    if ui.add(btn).clicked() {
                        mgr.set_input_channel(ch);
                        mgr.save_config();
                        ui.close_menu();
                    }
                }
            });
    });

    ui.add_space(4.0);

    let output_label = current_output.as_deref().unwrap_or("None");
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("Output:").size(UI_FONT));
        egui::ComboBox::from_id_salt("midi_output_device")
            .width(260.0)
            .selected_text(egui::RichText::new(output_label).size(UI_FONT))
            .show_ui(ui, |ui| {
                let none_btn = egui::Button::new(egui::RichText::new("None").size(UI_FONT))
                    .min_size(egui::vec2(240.0, 36.0))
                    .selected(current_output.is_none());
                if ui.add(none_btn).clicked() {
                    mgr.disconnect_output();
                    mgr.save_config();
                    ui.close_menu();
                }
                for name in &output_devices {
                    let selected = current_output.as_deref() == Some(name.as_str());
                    let btn = egui::Button::new(egui::RichText::new(name).size(UI_FONT))
                        .min_size(egui::vec2(240.0, 36.0))
                        .selected(selected);
                    if ui.add(btn).clicked() {
                        mgr.connect_output(name);
                        mgr.save_config();
                        ui.close_menu();
                    }
                }
            });
        ui.add_space(12.0);
        ui.label(egui::RichText::new("Ch:").size(UI_FONT));
        egui::ComboBox::from_id_salt("midi_output_channel")
            .width(80.0)
            .selected_text(egui::RichText::new(format!("{}", current_out_channel + 1)).size(UI_FONT))
            .show_ui(ui, |ui| {
                for ch in 0u8..16 {
                    let btn = egui::Button::new(egui::RichText::new(format!("{}", ch + 1)).size(UI_FONT))
                        .min_size(egui::vec2(70.0, 36.0))
                        .selected(ch == current_out_channel);
                    if ui.add(btn).clicked() {
                        mgr.set_output_channel(ch);
                        mgr.save_config();
                        ui.close_menu();
                    }
                }
            });
    });

    if feedback_risk {
        ui.add_space(4.0);
        ui.label(
            egui::RichText::new("⚠ Feedback risk: same device and overlapping channels")
                .size(UI_FONT)
                .color(Color32::from_rgb(220, 200, 60)),
        );
    }
}

fn render_midi_input_section(ui: &mut egui::Ui, ui_state: &Arc<SharedUiState>) {
    ui.label(egui::RichText::new("MIDI INPUT").size(HEADER_FONT).strong());
    ui.add_space(8.0);

    let current_mode_idx = ui_state.midi_mode.load(Ordering::Relaxed);
    let current_mode = MidiInputMode::from_index(current_mode_idx);

    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("Mode:").size(UI_FONT));
        egui::ComboBox::from_id_salt("midi_mode_selector")
            .width(160.0)
            .selected_text(egui::RichText::new(current_mode.label()).size(UI_FONT))
            .show_ui(ui, |ui| {
                for mode in MidiInputMode::all() {
                    let btn = egui::Button::new(egui::RichText::new(mode.label()).size(UI_FONT))
                        .min_size(egui::vec2(150.0, 36.0))
                        .selected(mode == current_mode);
                    if ui.add(btn).clicked() {
                        ui_state.midi_mode.store(mode.to_index(), Ordering::Relaxed);
                        if let Ok(mut mgr) = ui_state.midi_device_manager.try_lock() {
                            mgr.set_midi_mode(mode.to_index());
                            mgr.save_config();
                        }
                        ui.close_menu();
                    }
                }
            });
    });

    ui.add_space(8.0);

    match current_mode {
        MidiInputMode::Passthrough => {
            ui.label(
                egui::RichText::new("Playing incoming MIDI notes directly")
                    .size(UI_FONT)
                    .weak(),
            );
        }
        MidiInputMode::ChordFollow => {
            if let Ok(display) = ui_state.midi_mode_display.try_lock() {
                if display.held_notes.is_empty() {
                    ui.label(
                        egui::RichText::new("Waiting for MIDI input...")
                            .size(UI_FONT)
                            .weak(),
                    );
                } else {
                    let note_names = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];
                    let names: Vec<String> = display.held_notes.iter()
                        .map(|&(note, _)| {
                            let name = note_names[(note % 12) as usize];
                            let octave = (note / 12) as i8 - 1;
                            format!("{}{}", name, octave)
                        })
                        .collect();
                    ui.label(
                        egui::RichText::new(format!("Held: {}", names.join(" ")))
                            .size(UI_FONT),
                    );
                }
            }
        }
        MidiInputMode::Accompaniment => {
            if let Ok(display) = ui_state.midi_mode_display.try_lock() {
                let note_names = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];

                if let Some((root, scale)) = &display.detected_key {
                    let confidence_pct = (display.confidence * 100.0) as u32;
                    ui.label(
                        egui::RichText::new(format!(
                            "Key: {} {:?} ({}%)",
                            note_names[*root as usize],
                            scale,
                            confidence_pct,
                        ))
                        .size(UI_FONT),
                    );
                } else {
                    ui.label(
                        egui::RichText::new("Listening...")
                            .size(UI_FONT)
                            .weak(),
                    );
                }

                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(format!("Bars analyzed: {}", display.bars_analyzed))
                            .size(UI_FONT),
                    );

                    if ui
                        .add(
                            egui::Button::new(egui::RichText::new("Clear").size(UI_FONT))
                                .min_size(egui::vec2(60.0, 28.0)),
                        )
                        .clicked()
                    {
                        ui_state.midi_clear_memory.store(true, Ordering::Relaxed);
                    }
                });
            }
        }
    }
}

fn render_midi_sync_section(ui: &mut egui::Ui, ui_state: &Arc<SharedUiState>) {
    ui.label(egui::RichText::new("MIDI SYNC").size(HEADER_FONT).strong());
    ui.add_space(8.0);

    let mut soft_takeover = ui_state.soft_takeover.load(Ordering::Relaxed);
    ui.horizontal(|ui| {
        let resp = ui.checkbox(&mut soft_takeover, egui::RichText::new("Soft Takeover").size(UI_FONT));
        ui.label(egui::RichText::new("Knob picks up before changing").size(HINT_FONT).weak());
        if resp.changed() {
            ui_state.soft_takeover.store(soft_takeover, Ordering::Relaxed);
            if let Ok(mut mgr) = ui_state.midi_device_manager.try_lock() {
                mgr.set_soft_takeover(soft_takeover);
                mgr.save_config();
            }
        }
    });

    ui.add_space(4.0);

    let mut clock_in = ui_state.midi_clock_in.load(Ordering::Relaxed);
    ui.horizontal(|ui| {
        let resp = ui.checkbox(&mut clock_in, egui::RichText::new("Clock In").size(UI_FONT));
        ui.label(egui::RichText::new("Sync from external clock").size(HINT_FONT).weak());
        if resp.changed() {
            ui_state.midi_clock_in.store(clock_in, Ordering::Relaxed);
            if let Ok(mut mgr) = ui_state.midi_device_manager.try_lock() {
                mgr.set_midi_clock_in(clock_in);
                mgr.save_config();
            }
        }
    });

    ui.add_space(4.0);

    let mut clock_out = ui_state.midi_clock_out.load(Ordering::Relaxed);
    ui.horizontal(|ui| {
        let resp = ui.checkbox(&mut clock_out, egui::RichText::new("Clock Out").size(UI_FONT));
        ui.label(egui::RichText::new("Send clock to output").size(HINT_FONT).weak());
        if resp.changed() {
            ui_state.midi_clock_out.store(clock_out, Ordering::Relaxed);
            if let Ok(mut mgr) = ui_state.midi_device_manager.try_lock() {
                mgr.set_midi_clock_out(clock_out);
                mgr.save_config();
            }
        }
    });

    ui.add_space(4.0);

    let mut transport_in = ui_state.midi_transport_in.load(Ordering::Relaxed);
    ui.horizontal(|ui| {
        let resp = ui.checkbox(&mut transport_in, egui::RichText::new("Transport In").size(UI_FONT));
        ui.label(egui::RichText::new("Start/stop from MIDI").size(HINT_FONT).weak());
        if resp.changed() {
            ui_state.midi_transport_in.store(transport_in, Ordering::Relaxed);
            if let Ok(mut mgr) = ui_state.midi_device_manager.try_lock() {
                mgr.set_midi_transport_in(transport_in);
                mgr.save_config();
            }
        }
    });

    ui.add_space(4.0);

    let mut transport_out = ui_state.midi_transport_out.load(Ordering::Relaxed);
    ui.horizontal(|ui| {
        let resp = ui.checkbox(&mut transport_out, egui::RichText::new("Transport Out").size(UI_FONT));
        ui.label(egui::RichText::new("Send start/stop to output").size(HINT_FONT).weak());
        if resp.changed() {
            ui_state.midi_transport_out.store(transport_out, Ordering::Relaxed);
            if let Ok(mut mgr) = ui_state.midi_device_manager.try_lock() {
                mgr.set_midi_transport_out(transport_out);
                mgr.save_config();
            }
        }
    });
}

fn render_midi_learn_section(ui: &mut egui::Ui, ui_state: &Arc<SharedUiState>) {
    ui.label(egui::RichText::new("MIDI LEARN").size(HEADER_FONT).strong());
    ui.add_space(8.0);

    let ml = &ui_state.midi_learn;
    let learn_active = ml.learn_active.load(Ordering::Relaxed);

    let mapping_count = ml.mappings.try_lock()
        .map(|m| m.len())
        .unwrap_or(0);

    let awaiting = ml.awaiting_param.try_lock()
        .ok()
        .and_then(|a| a.clone());

    ui.horizontal(|ui| {
        let learn_btn_color = if learn_active {
            Color32::from_rgb(200, 140, 40)
        } else {
            Color32::from_rgb(60, 60, 70)
        };
        let learn_text_color = if learn_active {
            Color32::BLACK
        } else {
            Color32::from_gray(200)
        };
        let learn_btn = egui::Button::new(
            egui::RichText::new("LEARN").size(UI_FONT).color(learn_text_color),
        )
            .fill(learn_btn_color)
            .min_size(egui::vec2(90.0, 40.0));
        if ui.add(learn_btn).clicked() {
            let new_state = !learn_active;
            ml.learn_active.store(new_state, Ordering::Relaxed);
            if !new_state {
                if let Ok(mut a) = ml.awaiting_param.try_lock() {
                    *a = None;
                }
            }
        }

        ui.add_space(8.0);

        let forget_btn = egui::Button::new(egui::RichText::new("FORGET LAST").size(UI_FONT))
            .min_size(egui::vec2(120.0, 40.0));
        if ui.add(forget_btn).clicked() {
            if let Ok(mut mappings) = ml.mappings.try_lock() {
                mappings.remove_last();
            }
            save_midi_learn(ui_state);
        }

        ui.add_space(8.0);

        let clear_btn = egui::Button::new(egui::RichText::new("CLEAR ALL").size(UI_FONT))
            .min_size(egui::vec2(100.0, 40.0));
        if ui.add(clear_btn).clicked() {
            if let Ok(mut mappings) = ml.mappings.try_lock() {
                mappings.clear();
            }
            save_midi_learn(ui_state);
        }
    });

    ui.add_space(6.0);

    let learn_mode = ml.learn_mode.load(Ordering::Relaxed);
    let selector_cc_val = ml.selector_cc.load(Ordering::Relaxed);
    let value_cc_val = ml.value_cc.load(Ordering::Relaxed);

    ui.horizontal(|ui| {
        let sel_active = learn_mode == 1;
        let sel_btn_color = if sel_active {
            Color32::from_rgb(200, 140, 40)
        } else {
            Color32::from_rgb(60, 60, 70)
        };
        let sel_text_color = if sel_active { Color32::BLACK } else { Color32::from_gray(200) };
        let sel_label = if selector_cc_val < 128 {
            format!("SELECT CC{}", selector_cc_val)
        } else {
            "SELECT".to_string()
        };
        let sel_btn = egui::Button::new(
            egui::RichText::new(sel_label).size(UI_FONT).color(sel_text_color),
        )
            .fill(sel_btn_color)
            .min_size(egui::vec2(130.0, 40.0));
        if ui.add(sel_btn).clicked() {
            if sel_active {
                ml.learn_mode.store(0, Ordering::Relaxed);
            } else {
                ml.learn_mode.store(1, Ordering::Relaxed);
            }
        }

        ui.add_space(8.0);

        let val_active = learn_mode == 2;
        let val_btn_color = if val_active {
            Color32::from_rgb(200, 140, 40)
        } else {
            Color32::from_rgb(60, 60, 70)
        };
        let val_text_color = if val_active { Color32::BLACK } else { Color32::from_gray(200) };
        let val_label = if value_cc_val < 128 {
            format!("VALUE CC{}", value_cc_val)
        } else {
            "VALUE".to_string()
        };
        let val_btn = egui::Button::new(
            egui::RichText::new(val_label).size(UI_FONT).color(val_text_color),
        )
            .fill(val_btn_color)
            .min_size(egui::vec2(130.0, 40.0));
        if ui.add(val_btn).clicked() {
            if val_active {
                ml.learn_mode.store(0, Ordering::Relaxed);
            } else {
                ml.learn_mode.store(2, Ordering::Relaxed);
            }
        }
    });

    ui.add_space(6.0);

    let status = if learn_mode == 1 {
        "Turn knob for SELECT CC...".to_string()
    } else if learn_mode == 2 {
        "Turn knob for VALUE CC...".to_string()
    } else if learn_active {
        if awaiting.is_some() {
            format!("Waiting for CC... ({})", awaiting.unwrap())
        } else {
            "Waiting for slider...".to_string()
        }
    } else {
        format!("{} mapping{} active", mapping_count, if mapping_count == 1 { "" } else { "s" })
    };

    let status_color = if learn_active || learn_mode > 0 {
        Color32::from_rgb(200, 160, 60)
    } else {
        Color32::from_gray(140)
    };

    ui.label(egui::RichText::new(status).size(UI_FONT).color(status_color));
}

fn save_midi_learn(ui_state: &Arc<SharedUiState>) {
    if let Ok(mappings) = ui_state.midi_learn.mappings.try_lock() {
        if let Ok(mut mgr) = ui_state.midi_device_manager.try_lock() {
            mgr.set_midi_learn_mappings(mappings.mappings.clone());
            mgr.save_config();
        }
    }
}

fn render_performance_section(
    ui: &mut egui::Ui,
    params: &Arc<DeviceParams>,
    setter: &ParamSetter,
    ui_state: &Arc<SharedUiState>,
) {
    ui.label(egui::RichText::new("PERFORMANCE").size(HEADER_FONT).strong());
    ui.add_space(8.0);

    let cpu_load = ui_state.get_cpu_load();
    let cpu_color = if cpu_load > 80.0 {
        Color32::from_rgb(200, 80, 80)
    } else if cpu_load > 50.0 {
        Color32::from_rgb(200, 180, 80)
    } else {
        Color32::from_rgb(80, 200, 80)
    };
    ui.label(
        egui::RichText::new(format!("CPU: {:02}%", cpu_load as u32))
            .size(UI_FONT)
            .color(cpu_color),
    );

    ui.add_space(8.0);

    let mut limiter = params.limiter_enable.value();
    if ui
        .checkbox(&mut limiter, egui::RichText::new("Limiter").size(UI_FONT))
        .changed()
    {
        setter.set_parameter(&params.limiter_enable, limiter);
    }

    ui.add_space(8.0);

    let sample_rate = ui_state.sample_rate.load(Ordering::Relaxed) as f32;
    let latency_samples = ui_state.limiter_latency_samples.load(Ordering::Relaxed);
    let latency_ms = if sample_rate > 0.0 {
        latency_samples as f32 / sample_rate * 1000.0
    } else {
        0.0
    };
    ui.label(
        egui::RichText::new(format!(
            "Latency: {} samples ({:.1}ms @ {}Hz)",
            latency_samples, latency_ms, sample_rate as u32
        ))
        .size(UI_FONT)
        .weak(),
    );

    ui.add_space(8.0);

    ui.label(egui::RichText::new("Oversampling").size(UI_FONT).strong());
    ui.add_space(4.0);

    render_os_combo(ui, "Factor:", "os_all", params, setter, ui_state, |p| &p.synth_oversampling);
}

fn os_label(val: i32) -> &'static str {
    match val { 1 => "2x", 2 => "4x", 3 => "8x", 4 => "16x", 5 => "32x", 6 => "64x", _ => "128x" }
}

fn render_os_combo(
    ui: &mut egui::Ui,
    label: &str,
    id: &str,
    params: &Arc<DeviceParams>,
    setter: &ParamSetter,
    ui_state: &Arc<SharedUiState>,
    get_param: fn(&DeviceParams) -> &nih_plug::prelude::IntParam,
) {
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(label).size(UI_FONT));
        let param = get_param(params);
        let current = param.value();
        egui::ComboBox::from_id_salt(id)
            .width(90.0)
            .selected_text(egui::RichText::new(os_label(current)).size(UI_FONT))
            .show_ui(ui, |ui| {
                for (val, lbl) in [(1, "2x"), (2, "4x"), (3, "8x"), (4, "16x"), (5, "32x"), (6, "64x"), (7, "128x")] {
                    let btn = egui::Button::new(egui::RichText::new(lbl).size(UI_FONT))
                        .min_size(egui::vec2(80.0, 36.0))
                        .selected(current == val);
                    if ui.add(btn).clicked() {
                        setter.set_parameter(get_param(params), val);
                        if let Ok(mut mgr) = ui_state.midi_device_manager.try_lock() {
                            mgr.set_oversampling(val);
                            mgr.save_config();
                        }
                        ui.close_menu();
                    }
                }
            });
    });
}
