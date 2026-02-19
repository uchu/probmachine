#![allow(clippy::too_many_arguments)]

use std::sync::Arc;
use nih_plug_egui::egui::{self, Color32};
use crate::params::DeviceParams;
use nih_plug::prelude::*;

const WAVEFORM_NAMES: [&str; 5] = ["Sine", "Tri", "Saw", "Sqr", "S&H"];
const DIVISION_NAMES: [&str; 14] = [
    "1/1", "1/2", "1/4", "1/8", "1/16", "1/32",
    "1/2.", "1/4.", "1/8.", "1/16.",
    "1/2T", "1/4T", "1/8T", "1/16T",
];
const DEST_NAMES: [&str; 20] = [
    "None",
    "PLL Damp", "PLL Infl", "PLL Track", "PLL FM", "PLL XFB", "PLL OT", "PLL Rng",
    "VPS D", "VPS V",
    "Filt Cut", "Filt Res", "Filt Drv",
    "Drift", "Tube",
    "Rev Mix", "Rev Decay",
    "PLL Vol", "VPS Vol", "Sub Vol",
];

const FONT: f32 = 17.0;
const COMBO_FONT: f32 = 18.0;
const HEADER_FONT: f32 = 19.0;
const LABEL_COLOR: Color32 = Color32::from_gray(120);
const DIM_COLOR: Color32 = Color32::from_gray(70);
const COMBO_WIDTH: f32 = 130.0;
const COMBO_BTN_HEIGHT: f32 = 38.0;
const SLIDER_RAIL: f32 = 18.0;
const RATE_SLIDER_WIDTH: f32 = 160.0;
const AMOUNT_SLIDER_WIDTH: f32 = 200.0;
const DEST_COMBO_WIDTH: f32 = 170.0;

const RATE_COLOR: Color32 = Color32::from_rgb(80, 100, 80);
const PHASE_MOD_COLOR: Color32 = Color32::from_rgb(100, 60, 100);
const ROUTE_AMOUNT_COLOR: Color32 = Color32::from_rgb(60, 80, 120);

pub fn render_ui(
    ui: &mut egui::Ui,
    params: &Arc<DeviceParams>,
    setter: &ParamSetter,
) {
    ui.add_space(6.0);

    render_lfo_panel(ui, setter, 1,
        &params.lfo1_rate, &params.lfo1_waveform, &params.lfo1_tempo_sync,
        &params.lfo1_sync_division, &params.lfo1_sync_source, &params.lfo1_phase_mod,
        &params.lfo1_dest1, &params.lfo1_amount1, &params.lfo1_dest2, &params.lfo1_amount2);

    render_separator(ui);

    render_lfo_panel(ui, setter, 2,
        &params.lfo2_rate, &params.lfo2_waveform, &params.lfo2_tempo_sync,
        &params.lfo2_sync_division, &params.lfo2_sync_source, &params.lfo2_phase_mod,
        &params.lfo2_dest1, &params.lfo2_amount1, &params.lfo2_dest2, &params.lfo2_amount2);

    render_separator(ui);

    render_lfo_panel(ui, setter, 3,
        &params.lfo3_rate, &params.lfo3_waveform, &params.lfo3_tempo_sync,
        &params.lfo3_sync_division, &params.lfo3_sync_source, &params.lfo3_phase_mod,
        &params.lfo3_dest1, &params.lfo3_amount1, &params.lfo3_dest2, &params.lfo3_amount2);
}

fn render_separator(ui: &mut egui::Ui) {
    ui.add_space(10.0);
    let rect = ui.available_rect_before_wrap();
    let y = rect.top();
    ui.painter().line_segment(
        [egui::pos2(rect.left() + 10.0, y), egui::pos2(rect.left() + 500.0, y)],
        egui::Stroke::new(1.0, DIM_COLOR),
    );
    ui.add_space(14.0);
}

fn render_lfo_panel(
    ui: &mut egui::Ui,
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
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(format!("LFO {}", lfo_num))
            .size(HEADER_FONT).strong());

        ui.add_space(24.0);

        ui.label(egui::RichText::new("WAVE").size(FONT).color(LABEL_COLOR));
        ui.add_space(4.0);
        render_combo(ui, &format!("lfo{}_wf", lfo_num), COMBO_WIDTH,
            &WAVEFORM_NAMES, waveform.value() as usize,
            |i| setter.set_parameter(waveform, i as i32));

        ui.add_space(20.0);

        let mut sync = tempo_sync.value();
        render_toggle(ui, &mut sync, "SYNC");
        if sync != tempo_sync.value() {
            setter.set_parameter(tempo_sync, sync);
        }

        ui.add_space(20.0);

        if tempo_sync.value() {
            ui.label(egui::RichText::new("DIV").size(FONT).color(LABEL_COLOR));
            ui.add_space(4.0);
            render_combo(ui, &format!("lfo{}_div", lfo_num), 100.0,
                &DIVISION_NAMES, sync_division.value() as usize,
                |i| setter.set_parameter(sync_division, i as i32));
        } else {
            ui.label(egui::RichText::new("RATE").size(FONT).color(LABEL_COLOR));
            ui.add_space(4.0);
            set_slider_color(ui, RATE_COLOR);
            let mut rate_val = rate.modulated_plain_value();
            ui.style_mut().spacing.slider_width = RATE_SLIDER_WIDTH;
            ui.style_mut().spacing.slider_rail_height = SLIDER_RAIL;
            let slider = egui::Slider::new(&mut rate_val, 0.01..=50.0)
                .logarithmic(true)
                .suffix(" Hz")
                .clamping(egui::SliderClamping::Always);
            if ui.add(slider).changed() {
                setter.set_parameter(rate, rate_val);
            }
        }

        ui.add_space(20.0);

        ui.label(egui::RichText::new("FROM").size(FONT).color(LABEL_COLOR));
        ui.add_space(4.0);
        render_combo(ui, &format!("lfo{}_src", lfo_num), 110.0,
            &["None", "LFO 1", "LFO 2", "LFO 3"],
            source_to_index(sync_source.value(), lfo_num),
            |i| setter.set_parameter(sync_source, index_to_source(i, lfo_num)));

        if sync_source.value() >= 0 {
            ui.add_space(12.0);
            ui.label(egui::RichText::new("AMT").size(FONT).color(LABEL_COLOR));
            ui.add_space(4.0);
            set_slider_color(ui, PHASE_MOD_COLOR);
            let mut pm_val = phase_mod.modulated_plain_value();
            ui.style_mut().spacing.slider_width = 80.0;
            ui.style_mut().spacing.slider_rail_height = SLIDER_RAIL;
            let slider = egui::Slider::new(&mut pm_val, 0.0..=1.0)
                .clamping(egui::SliderClamping::Always)
                .show_value(false);
            if ui.add(slider).changed() {
                setter.set_parameter(phase_mod, pm_val);
            }
        }
    });

    ui.add_space(12.0);

    ui.horizontal(|ui| {
        ui.add_space(10.0);
        render_route_slot(ui, setter, lfo_num, 1, dest1, amount1);
        ui.add_space(40.0);
        render_route_slot(ui, setter, lfo_num, 2, dest2, amount2);
    });
}

fn source_to_index(src: i32, lfo_num: usize) -> usize {
    if src < 0 { return 0; }
    let mut idx = 1;
    for i in 0..3 {
        if i + 1 == lfo_num { continue; }
        if i as i32 == src { return idx; }
        idx += 1;
    }
    0
}

fn index_to_source(idx: usize, lfo_num: usize) -> i32 {
    if idx == 0 { return -1; }
    let mut count = 1;
    for i in 0..3 {
        if i + 1 == lfo_num { continue; }
        if count == idx { return i as i32; }
        count += 1;
    }
    -1
}

fn set_slider_color(ui: &mut egui::Ui, color: Color32) {
    ui.style_mut().visuals.widgets.inactive.bg_fill = color;
    ui.style_mut().visuals.widgets.hovered.bg_fill = color;
    ui.style_mut().visuals.widgets.active.bg_fill = color;
}

fn render_route_slot(
    ui: &mut egui::Ui,
    setter: &ParamSetter,
    lfo_num: usize,
    slot: usize,
    dest: &IntParam,
    amount: &FloatParam,
) {
    ui.label(egui::RichText::new("→").size(FONT).color(Color32::from_rgb(80, 120, 170)));
    ui.add_space(6.0);

    render_combo(ui, &format!("lfo{}_{}_dest", lfo_num, slot), DEST_COMBO_WIDTH,
        &DEST_NAMES, dest.value() as usize,
        |i| setter.set_parameter(dest, i as i32));

    ui.add_space(8.0);

    set_slider_color(ui, ROUTE_AMOUNT_COLOR);
    let mut amt_val = amount.modulated_plain_value();
    ui.style_mut().spacing.slider_width = AMOUNT_SLIDER_WIDTH;
    ui.style_mut().spacing.slider_rail_height = SLIDER_RAIL;
    let slider = egui::Slider::new(&mut amt_val, -1.0..=1.0)
        .clamping(egui::SliderClamping::Always)
        .show_value(true);
    if ui.add(slider).changed() {
        setter.set_parameter(amount, amt_val);
    }
}

fn render_combo<F: FnOnce(usize)>(
    ui: &mut egui::Ui,
    id: &str,
    width: f32,
    names: &[&str],
    current: usize,
    on_select: F,
) {
    let mut selected = None;
    ui.style_mut().spacing.button_padding = egui::vec2(8.0, 6.0);
    egui::ComboBox::from_id_salt(id)
        .width(width)
        .height(420.0)
        .selected_text(egui::RichText::new(
            names.get(current).copied().unwrap_or("?")).size(COMBO_FONT))
        .show_ui(ui, |ui| {
            ui.style_mut().spacing.item_spacing.y = 3.0;
            for (i, name) in names.iter().enumerate() {
                let btn = egui::Button::new(egui::RichText::new(*name).size(COMBO_FONT))
                    .min_size(egui::vec2(width - 10.0, COMBO_BTN_HEIGHT))
                    .selected(current == i);
                if ui.add(btn).clicked() {
                    selected = Some(i);
                    ui.close_menu();
                }
            }
        });
    if let Some(i) = selected {
        on_select(i);
    }
}

fn render_toggle(ui: &mut egui::Ui, value: &mut bool, label: &str) {
    let desired_size = egui::vec2(44.0, 22.0);
    let (alloc_rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::click());
    if response.clicked() {
        *value = !*value;
    }

    let rect = alloc_rect;
    let anim_t = ui.ctx().animate_bool_with_time(response.id, *value, 0.15);

    let bg_color = Color32::from_gray(50)
        .lerp_to_gamma(Color32::from_rgb(80, 130, 190), anim_t);
    let circle_x = egui::lerp(rect.left() + 11.0..=rect.right() - 11.0, anim_t);
    let circle_color = Color32::from_gray(220)
        .lerp_to_gamma(Color32::WHITE, anim_t);

    ui.painter().rect_filled(rect, rect.height() / 2.0, bg_color);
    ui.painter().circle_filled(egui::pos2(circle_x, rect.center().y), 8.0, circle_color);

    let text_color = if *value { Color32::WHITE } else { Color32::from_gray(140) };
    ui.painter().text(
        egui::pos2(rect.right() + 6.0, rect.center().y),
        egui::Align2::LEFT_CENTER,
        label,
        egui::FontId::proportional(FONT),
        text_color,
    );
}
