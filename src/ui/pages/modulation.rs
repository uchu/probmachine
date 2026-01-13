use std::sync::Arc;
use egui_taffy::TuiBuilderLogic;
use nih_plug_egui::egui::{self, Color32};
use crate::params::DeviceParams;
use nih_plug::prelude::*;

const WAVEFORM_NAMES: [&str; 5] = ["Sine", "Tri", "Saw", "Sqr", "S&H"];
const DIVISION_NAMES: [&str; 14] = [
    "1/1", "1/2", "1/4", "1/8", "1/16", "1/32",
    "1/2.", "1/4.", "1/8.", "1/16.",
    "1/2T", "1/4T", "1/8T", "1/16T",
];
const DEST_NAMES: [&str; 27] = [
    "None", "PLL Damp", "PLL Infl", "PLL Track", "PLL FB", "PLL FM", "PLL PW",
    "PLL StPh", "PLL XFB", "PLL FMEnv",
    "VPS D", "VPS V", "Filt Cut", "Filt Res", "Filt Drv",
    "Fmt Vowel", "Fmt Shift",
    "Ring Mod", "Wavefold", "Drift", "Noise", "Tube",
    "Rev Mix", "Rev Decay",
    "PLL Vol", "VPS Vol", "Sub Vol",
];

pub fn render(
    tui: &mut egui_taffy::Tui,
    params: &Arc<DeviceParams>,
    setter: &ParamSetter,
) {
    tui.ui(|ui| {
        ui.add_space(12.0);
        ui.heading(egui::RichText::new("    Modulation").size(14.0));
        ui.add_space(8.0);
    });

    // LFO 1
    render_lfo_panel(tui, params, setter, 1,
        &params.lfo1_rate, &params.lfo1_waveform, &params.lfo1_tempo_sync,
        &params.lfo1_sync_division, &params.lfo1_sync_source, &params.lfo1_phase_mod,
        &params.lfo1_dest1, &params.lfo1_amount1, &params.lfo1_dest2, &params.lfo1_amount2);

    // LFO 2
    render_lfo_panel(tui, params, setter, 2,
        &params.lfo2_rate, &params.lfo2_waveform, &params.lfo2_tempo_sync,
        &params.lfo2_sync_division, &params.lfo2_sync_source, &params.lfo2_phase_mod,
        &params.lfo2_dest1, &params.lfo2_amount1, &params.lfo2_dest2, &params.lfo2_amount2);

    // LFO 3
    render_lfo_panel(tui, params, setter, 3,
        &params.lfo3_rate, &params.lfo3_waveform, &params.lfo3_tempo_sync,
        &params.lfo3_sync_division, &params.lfo3_sync_source, &params.lfo3_phase_mod,
        &params.lfo3_dest1, &params.lfo3_amount1, &params.lfo3_dest2, &params.lfo3_amount2);
}

#[allow(clippy::too_many_arguments)]
fn render_lfo_panel(
    tui: &mut egui_taffy::Tui,
    _params: &Arc<DeviceParams>,
    setter: &ParamSetter,
    lfo_num: usize,
    rate: &FloatParam,
    waveform: &IntParam,
    tempo_sync: &BoolParam,
    sync_division: &IntParam,
    sync_source: &IntParam,
    phase_mod: &FloatParam,
    dest1: &IntParam,
    amount1: &FloatParam,
    dest2: &IntParam,
    amount2: &FloatParam,
) {
    tui.ui(|ui| {
        egui::Frame::default()
            .fill(ui.visuals().extreme_bg_color)
            .inner_margin(10.0)
            .stroke(egui::Stroke::new(1.0, Color32::from_rgb(60, 80, 120)))
            .corner_radius(10.0)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(format!("LFO {}", lfo_num)).size(12.0).strong());

                    ui.add_space(15.0);

                    // Waveform selector
                    ui.label(egui::RichText::new("Wave:").size(10.0));
                    let wf_idx = waveform.value() as usize;
                    egui::ComboBox::from_id_salt(format!("lfo{}_wf", lfo_num))
                        .width(50.0)
                        .selected_text(WAVEFORM_NAMES.get(wf_idx).copied().unwrap_or("?"))
                        .show_ui(ui, |ui| {
                            for (i, name) in WAVEFORM_NAMES.iter().enumerate() {
                                if ui.selectable_label(wf_idx == i, *name).clicked() {
                                    setter.set_parameter(waveform, i as i32);
                                }
                            }
                        });

                    ui.add_space(10.0);

                    // Tempo sync toggle
                    let mut sync = tempo_sync.value();
                    if ui.checkbox(&mut sync, "Sync").changed() {
                        setter.set_parameter(tempo_sync, sync);
                    }

                    ui.add_space(10.0);

                    // Rate or division based on sync
                    if tempo_sync.value() {
                        ui.label(egui::RichText::new("Div:").size(10.0));
                        let div_idx = sync_division.value() as usize;
                        egui::ComboBox::from_id_salt(format!("lfo{}_div", lfo_num))
                            .width(50.0)
                            .selected_text(DIVISION_NAMES.get(div_idx).copied().unwrap_or("?"))
                            .show_ui(ui, |ui| {
                                for (i, name) in DIVISION_NAMES.iter().enumerate() {
                                    if ui.selectable_label(div_idx == i, *name).clicked() {
                                        setter.set_parameter(sync_division, i as i32);
                                    }
                                }
                            });
                    } else {
                        ui.label(egui::RichText::new("Rate:").size(10.0));
                        let mut rate_val = rate.modulated_plain_value();
                        let slider = egui::Slider::new(&mut rate_val, 0.01..=50.0)
                            .logarithmic(true)
                            .suffix(" Hz")
                            .clamping(egui::SliderClamping::Always);
                        if ui.add(slider).changed() {
                            setter.set_parameter(rate, rate_val);
                        }
                    }

                    ui.add_space(10.0);

                    // Sync source
                    ui.label(egui::RichText::new("From:").size(10.0));
                    let src = sync_source.value();
                    let src_name = match src {
                        -1 => "None",
                        0 => "LFO 1",
                        1 => "LFO 2",
                        2 => "LFO 3",
                        _ => "?",
                    };
                    egui::ComboBox::from_id_salt(format!("lfo{}_src", lfo_num))
                        .width(55.0)
                        .selected_text(src_name)
                        .show_ui(ui, |ui| {
                            if ui.selectable_label(src == -1, "None").clicked() {
                                setter.set_parameter(sync_source, -1);
                            }
                            for i in 0..3 {
                                if i + 1 != lfo_num {
                                    let label = format!("LFO {}", i + 1);
                                    if ui.selectable_label(src == i as i32, &label).clicked() {
                                        setter.set_parameter(sync_source, i as i32);
                                    }
                                }
                            }
                        });

                    // Phase mod amount (only show if sync source is set)
                    if sync_source.value() >= 0 {
                        ui.label(egui::RichText::new("Amt:").size(10.0));
                        let mut pm_val = phase_mod.modulated_plain_value();
                        let slider = egui::Slider::new(&mut pm_val, 0.0..=1.0)
                            .clamping(egui::SliderClamping::Always)
                            .show_value(false);
                        if ui.add_sized([60.0, 18.0], slider).changed() {
                            setter.set_parameter(phase_mod, pm_val);
                        }
                    }
                });

                ui.add_space(5.0);

                // Modulation slots
                ui.horizontal(|ui| {
                    // Slot 1
                    render_mod_slot(ui, setter, lfo_num, 1, dest1, amount1);

                    ui.add_space(20.0);

                    // Slot 2
                    render_mod_slot(ui, setter, lfo_num, 2, dest2, amount2);
                });
            });
    });

    tui.ui(|ui| { ui.add_space(8.0); });
}

fn render_mod_slot(
    ui: &mut egui::Ui,
    setter: &ParamSetter,
    lfo_num: usize,
    slot: usize,
    dest: &IntParam,
    amount: &FloatParam,
) {
    ui.label(egui::RichText::new("→").size(12.0).color(Color32::from_rgb(100, 150, 200)));

    let dest_idx = dest.value() as usize;
    egui::ComboBox::from_id_salt(format!("lfo{}_{}_dest", lfo_num, slot))
        .width(75.0)
        .selected_text(DEST_NAMES.get(dest_idx).copied().unwrap_or("?"))
        .show_ui(ui, |ui| {
            for (i, name) in DEST_NAMES.iter().enumerate() {
                if ui.selectable_label(dest_idx == i, *name).clicked() {
                    setter.set_parameter(dest, i as i32);
                }
            }
        });

    let mut amt_val = amount.modulated_plain_value();
    let slider = egui::Slider::new(&mut amt_val, -1.0..=1.0)
        .clamping(egui::SliderClamping::Always)
        .show_value(true);
    if ui.add_sized([100.0, 18.0], slider).changed() {
        setter.set_parameter(amount, amt_val);
    }
}
