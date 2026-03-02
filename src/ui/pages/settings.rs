use crate::params::DeviceParams;
use crate::ui::SharedUiState;
use crate::midi_modes::MidiInputMode;
use egui_taffy::TuiBuilderLogic;
use nih_plug::prelude::ParamSetter;
use nih_plug_egui::egui;
use nih_plug_egui::egui::Color32;
use std::sync::Arc;
use std::sync::atomic::Ordering;

const HEADER_FONT: f32 = 18.0;
const UI_FONT: f32 = 16.0;
const SECTION_SPACING: f32 = 16.0;

pub fn render(
    tui: &mut egui_taffy::Tui,
    params: &Arc<DeviceParams>,
    setter: &ParamSetter,
    ui_state: &Arc<SharedUiState>,
) {
    tui.ui(|ui| {
        egui::Frame::NONE
            .inner_margin(egui::Margin::same(16))
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    render_midi_section(ui, ui_state);
                    ui.add_space(SECTION_SPACING);
                    ui.separator();
                    ui.add_space(SECTION_SPACING);
                    render_performance_section(ui, params, setter, ui_state);
                });
            });
    });
}

fn render_midi_section(ui: &mut egui::Ui, ui_state: &Arc<SharedUiState>) {
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

    let mut color = params.synth_coloration_enable.value();
    if ui
        .checkbox(&mut color, egui::RichText::new("Coloration").size(UI_FONT))
        .changed()
    {
        setter.set_parameter(&params.synth_coloration_enable, color);
    }

    ui.add_space(8.0);

    ui.label(egui::RichText::new("Oversampling").size(UI_FONT).strong());
    ui.add_space(4.0);

    render_os_combo(ui, "PLL:", "os_pll", params, setter, |p| &p.synth_pll_oversampling);
    render_os_combo(ui, "SAW:", "os_saw", params, setter, |p| &p.synth_saw_oversampling);
    render_os_combo(ui, "VPS:", "os_vps", params, setter, |p| &p.synth_vps_oversampling);
}

fn os_label(val: i32) -> &'static str {
    match val { 0 => "1x (off)", 1 => "2x", 2 => "4x", 3 => "8x", 4 => "16x", 5 => "32x", 6 => "64x", _ => "128x" }
}

fn render_os_combo(
    ui: &mut egui::Ui,
    label: &str,
    id: &str,
    params: &Arc<DeviceParams>,
    setter: &ParamSetter,
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
                for (val, lbl) in [(0, "1x (off)"), (1, "2x"), (2, "4x"), (3, "8x"), (4, "16x"), (5, "32x"), (6, "64x"), (7, "128x")] {
                    let btn = egui::Button::new(egui::RichText::new(lbl).size(UI_FONT))
                        .min_size(egui::vec2(80.0, 36.0))
                        .selected(current == val);
                    if ui.add(btn).clicked() {
                        setter.set_parameter(get_param(params), val);
                        ui.close_menu();
                    }
                }
            });
    });
}
