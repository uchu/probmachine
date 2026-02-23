#![allow(clippy::too_many_arguments)]

use std::sync::Arc;
use nih_plug_egui::egui::{self, Color32};
use crate::params::DeviceParams;
use nih_plug::prelude::*;

const WAVE_BTN_SIZE: f32 = 56.0;
const WAVE_BTN_GAP: f32 = 8.0;
const WAVE_STROKE: f32 = 2.0;
const WAVE_COLOR: Color32 = Color32::from_rgb(160, 200, 240);
const WAVE_BG: Color32 = Color32::from_gray(40);
const WAVE_BG_SEL: Color32 = Color32::from_rgb(50, 70, 100);
const WAVE_COUNT: usize = 5;

const DIVISION_NAMES: [&str; 16] = [
    "1/1", "1/2", "1/4", "1/8", "1/16", "1/32",
    "1/2.", "1/4.", "1/8.", "1/16.",
    "1/2T", "1/4T", "1/8T", "1/16T",
    "2/1", "4/1",
];
const DIVISION_DISPLAY_ORDER: [usize; 16] = [
    15, 14, 0, 1, 2, 3, 4, 5,
    6, 7, 8, 9,
    10, 11, 12, 13,
];
const DEST_ENTRIES: [(&str, i32); 26] = [
    ("None", 0),
    // PLL
    ("PLL Damp", 1), ("PLL Infl", 2), ("PLL Track", 3), ("PLL FM", 4),
    ("PLL XFB", 5), ("PLL OT", 6), ("PLL Rng", 7), ("PLL Vol", 17),
    ("PLL Mult", 20), ("PLL Mult D", 21),
    // Sub
    ("Sub Vol", 19),
    // VPS
    ("VPS D", 8), ("VPS V", 9), ("VPS VΔ", 25), ("VPS DΔ", 23),
    ("VPS Fold", 24), ("VPS SHP", 22), ("VPS Vol", 18),
    // Filter
    ("Filt Cut", 10), ("Filt Res", 11), ("Filt Drv", 12),
    // Coloration
    ("Drift", 13), ("Tube", 14),
    // Reverb
    ("Rev Mix", 15), ("Rev Decay", 16),
];

const FONT: f32 = 17.0;
const COMBO_FONT: f32 = 18.0;
const HEADER_FONT: f32 = 19.0;
const LABEL_COLOR: Color32 = Color32::from_gray(120);
const DISABLED_COLOR: Color32 = Color32::from_gray(30);
const COMBO_BTN_HEIGHT: f32 = 38.0;
const SLIDER_RAIL: f32 = 18.0;
const AMOUNT_SLIDER_WIDTH: f32 = 200.0;
const DEST_COMBO_WIDTH: f32 = 170.0;

const COL_WIDTH: f32 = 330.0;
const COL_GAP: f32 = 25.0;
const COL_LEFT_PAD: f32 = 30.0;
const COL_SLIDER_WIDTH: f32 = 290.0;
const COL_DEST_COMBO: f32 = 130.0;

const RATE_COLOR: Color32 = Color32::from_rgb(80, 100, 80);
const PHASE_MOD_COLOR: Color32 = Color32::from_rgb(100, 60, 100);
const ROUTE_AMOUNT_COLOR: Color32 = Color32::from_rgb(60, 80, 120);

const STEP_BAR_WIDTH: f32 = 46.0;
const STEP_BAR_HEIGHT: f32 = 140.0;
const STEP_BAR_GAP: f32 = 4.0;
const TIE_BTN_SIZE: f32 = 20.0;
const STEP_COLOR_POS: Color32 = Color32::from_rgb(80, 140, 200);
const STEP_COLOR_NEG: Color32 = Color32::from_rgb(200, 100, 80);
const TIE_COLOR_ON: Color32 = Color32::from_rgb(200, 170, 60);
const TIE_COLOR_OFF: Color32 = Color32::from_gray(45);
const SLEW_COLOR: Color32 = Color32::from_rgb(120, 80, 60);

pub fn render_ui(
    ui: &mut egui::Ui,
    params: &Arc<DeviceParams>,
    setter: &ParamSetter,
) {
    ui.add_space(16.0);

    let full_rect = ui.available_rect_before_wrap();

    ui.horizontal(|ui| {
        ui.add_space(COL_LEFT_PAD);

        let wave_row_width = WAVE_COUNT as f32 * WAVE_BTN_SIZE
            + (WAVE_COUNT as f32 - 1.0) * WAVE_BTN_GAP;
        let col_unused = COL_WIDTH - wave_row_width;
        let gap1 = COL_GAP + 25.0;
        let gap2 = COL_GAP + 25.0;

        render_lfo_column(ui, setter, 1,
            &params.lfo1_rate, &params.lfo1_waveform, &params.lfo1_tempo_sync,
            &params.lfo1_sync_division, &params.lfo1_sync_source, &params.lfo1_phase_mod,
            &params.lfo1_dest1, &params.lfo1_amount1, &params.lfo1_dest2, &params.lfo1_amount2);

        let right_edge = ui.cursor().left() - col_unused;
        let left_edge = ui.cursor().left() + gap1;
        let sep_x = (right_edge + left_edge) / 2.0;
        ui.painter().line_segment(
            [egui::pos2(sep_x, full_rect.top()), egui::pos2(sep_x, full_rect.bottom())],
            egui::Stroke::new(1.0, Color32::BLACK),
        );
        ui.add_space(gap1);

        render_lfo_column(ui, setter, 2,
            &params.lfo2_rate, &params.lfo2_waveform, &params.lfo2_tempo_sync,
            &params.lfo2_sync_division, &params.lfo2_sync_source, &params.lfo2_phase_mod,
            &params.lfo2_dest1, &params.lfo2_amount1, &params.lfo2_dest2, &params.lfo2_amount2);

        let right_edge = ui.cursor().left() - col_unused;
        let left_edge = ui.cursor().left() + gap2;
        let sep_x = (right_edge + left_edge) / 2.0;
        ui.painter().line_segment(
            [egui::pos2(sep_x, full_rect.top()), egui::pos2(sep_x, full_rect.bottom())],
            egui::Stroke::new(1.0, Color32::BLACK),
        );
        ui.add_space(gap2);

        render_lfo_column(ui, setter, 3,
            &params.lfo3_rate, &params.lfo3_waveform, &params.lfo3_tempo_sync,
            &params.lfo3_sync_division, &params.lfo3_sync_source, &params.lfo3_phase_mod,
            &params.lfo3_dest1, &params.lfo3_amount1, &params.lfo3_dest2, &params.lfo3_amount2);
    });
}

pub fn render_step_mod_ui(
    ui: &mut egui::Ui,
    params: &Arc<DeviceParams>,
    setter: &ParamSetter,
) {
    ui.add_space(6.0);
    render_step_seq_panel(ui, params, setter);
}

fn render_lfo_column(
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
    ui.vertical(|ui| {
        ui.set_width(COL_WIDTH);

        ui.label(egui::RichText::new(format!("LFO {}", lfo_num))
            .size(HEADER_FONT).strong());
        ui.add_space(14.0);

        let is_synced = tempo_sync.value();

        ui.horizontal(|ui| {
            render_waveform_buttons(ui, lfo_num, waveform.value() as usize,
                |i| setter.set_parameter(waveform, i));
        });
        ui.add_space(20.0);

        // RATE label
        ui.label(egui::RichText::new("RATE").size(FONT).color(LABEL_COLOR));
        ui.add_space(6.0);

        // RATE slider
        ui.horizontal(|ui| {
            if is_synced {
                set_disabled_slider_color(ui);
            } else {
                set_slider_color(ui, RATE_COLOR);
            }
            let mut rate_val = rate.modulated_plain_value();
            ui.style_mut().spacing.slider_width = COL_SLIDER_WIDTH;
            ui.style_mut().spacing.slider_rail_height = SLIDER_RAIL;
            let slider = egui::Slider::new(&mut rate_val, 0.01..=50.0)
                .logarithmic(true)
                .show_value(false)
                .clamping(egui::SliderClamping::Always);
            if ui.add_enabled(!is_synced, slider).changed() {
                setter.set_parameter(rate, rate_val);
            }
        });
        ui.add_space(20.0);

        // SYNC + division dropdown on same row
        ui.horizontal(|ui| {
            let mut sync = is_synced;
            render_toggle(ui, &mut sync, "SYNC");
            if sync != is_synced {
                setter.set_parameter(tempo_sync, sync);
            }
            ui.add_space(41.0);
            ui.add_enabled_ui(is_synced, |ui| {
                render_division_combo(ui, &format!("lfo{}_div", lfo_num), 110.0,
                    sync_division.value() as usize,
                    |i| setter.set_parameter(sync_division, i as i32));
            });
        });
        ui.add_space(20.0);

        // FROM
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("FROM").size(FONT).color(LABEL_COLOR));
            ui.add_space(4.0);
            render_combo(ui, &format!("lfo{}_src", lfo_num), 110.0,
                &["None", "LFO 1", "LFO 2", "LFO 3"],
                source_to_index(sync_source.value(), lfo_num),
                |i| setter.set_parameter(sync_source, index_to_source(i, lfo_num)));
        });
        ui.add_space(20.0);

        ui.horizontal(|ui| {
            let has_source = sync_source.value() >= 0;
            if has_source {
                set_slider_color(ui, PHASE_MOD_COLOR);
            } else {
                set_disabled_slider_color(ui);
            }
            let mut pm_val = phase_mod.modulated_plain_value();
            ui.style_mut().spacing.slider_width = COL_SLIDER_WIDTH;
            ui.style_mut().spacing.slider_rail_height = SLIDER_RAIL;
            let slider = egui::Slider::new(&mut pm_val, 0.0..=1.0)
                .clamping(egui::SliderClamping::Always)
                .show_value(false);
            if ui.add_enabled(has_source, slider).changed() {
                setter.set_parameter(phase_mod, pm_val);
            }
        });
        ui.add_space(22.0);

        let id_prefix = format!("lfo{}", lfo_num);
        render_route_slot(ui, setter, &id_prefix, 1, dest1, amount1);
        ui.add_space(18.0);
        render_route_slot(ui, setter, &id_prefix, 2, dest2, amount2);
    });
}

fn render_waveform_buttons<F: FnOnce(i32)>(
    ui: &mut egui::Ui,
    lfo_num: usize,
    current: usize,
    on_select: F,
) {
    let mut clicked = None;
    for i in 0..WAVE_COUNT {
        if i > 0 {
            ui.add_space(WAVE_BTN_GAP);
        }
        let size = egui::vec2(WAVE_BTN_SIZE, WAVE_BTN_SIZE);
        let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click());
        let selected = i == current;
        let bg = if selected { WAVE_BG_SEL } else { WAVE_BG };
        let rounding = 4.0;
        ui.painter().rect_filled(rect, rounding, bg);
        if selected {
            ui.painter().rect_stroke(rect, rounding, egui::Stroke::new(1.5, WAVE_COLOR), egui::epaint::StrokeKind::Inside);
        }
        paint_waveform(ui.painter(), rect.shrink(8.0), i);
        if response.clicked() {
            clicked = Some(i);
        }
        let id = egui::Id::new(format!("lfo{}_wave_{}", lfo_num, i));
        let hover_t = ui.ctx().animate_bool_with_time(id, response.hovered(), 0.1);
        if hover_t > 0.0 && !selected {
            let hover_bg = WAVE_BG.lerp_to_gamma(WAVE_BG_SEL, hover_t);
            ui.painter().rect_filled(rect, rounding, hover_bg);
            paint_waveform(ui.painter(), rect.shrink(8.0), i);
        }
    }
    if let Some(i) = clicked {
        on_select(i as i32);
    }
}

fn paint_waveform(painter: &egui::Painter, rect: egui::Rect, wave: usize) {
    let stroke = egui::Stroke::new(WAVE_STROKE, WAVE_COLOR);
    let l = rect.left();
    let r = rect.right();
    let t = rect.top();
    let b = rect.bottom();
    let cy = rect.center().y;
    let cx = rect.center().x;
    let w = rect.width();
    let h = rect.height();

    match wave {
        0 => {
            // Sine
            let steps = 24;
            let points: Vec<egui::Pos2> = (0..=steps)
                .map(|i| {
                    let frac = i as f32 / steps as f32;
                    let x = l + frac * w;
                    let y = cy - (frac * std::f32::consts::TAU).sin() * (h / 2.0);
                    egui::pos2(x, y)
                })
                .collect();
            for pair in points.windows(2) {
                painter.line_segment([pair[0], pair[1]], stroke);
            }
        }
        1 => {
            // Triangle
            painter.line_segment([egui::pos2(l, cy), egui::pos2(l + w * 0.25, t)], stroke);
            painter.line_segment([egui::pos2(l + w * 0.25, t), egui::pos2(l + w * 0.75, b)], stroke);
            painter.line_segment([egui::pos2(l + w * 0.75, b), egui::pos2(r, cy)], stroke);
        }
        2 => {
            // Saw
            painter.line_segment([egui::pos2(l, b), egui::pos2(cx, t)], stroke);
            painter.line_segment([egui::pos2(cx, t), egui::pos2(cx, b)], stroke);
            painter.line_segment([egui::pos2(cx, b), egui::pos2(r, t)], stroke);
        }
        3 => {
            // Square
            painter.line_segment([egui::pos2(l, t), egui::pos2(cx, t)], stroke);
            painter.line_segment([egui::pos2(cx, t), egui::pos2(cx, b)], stroke);
            painter.line_segment([egui::pos2(cx, b), egui::pos2(r, b)], stroke);
        }
        4 => {
            // S&H (stepped random-ish)
            let levels = [0.3, 0.8, 0.1, 0.6, 0.9, 0.2];
            let step_w = w / levels.len() as f32;
            for (i, &level) in levels.iter().enumerate() {
                let x0 = l + i as f32 * step_w;
                let x1 = x0 + step_w;
                let y = t + (1.0 - level) * h;
                painter.line_segment([egui::pos2(x0, y), egui::pos2(x1, y)], stroke);
                if i > 0 {
                    let prev_y = t + (1.0 - levels[i - 1]) * h;
                    painter.line_segment([egui::pos2(x0, prev_y), egui::pos2(x0, y)], stroke);
                }
            }
        }
        _ => {}
    }
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

fn set_disabled_slider_color(ui: &mut egui::Ui) {
    ui.style_mut().visuals.widgets.inactive.bg_fill = DISABLED_COLOR;
    ui.style_mut().visuals.widgets.hovered.bg_fill = DISABLED_COLOR;
    ui.style_mut().visuals.widgets.active.bg_fill = DISABLED_COLOR;
    ui.style_mut().visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, Color32::from_gray(50));
    ui.style_mut().visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.0, Color32::from_gray(50));
}

fn paint_arrow(ui: &mut egui::Ui) {
    let arrow_size = egui::vec2(18.0, 18.0);
    let (arrow_rect, _) = ui.allocate_exact_size(arrow_size, egui::Sense::hover());
    let arrow_color = Color32::from_rgb(80, 120, 170);
    let cx = arrow_rect.center().x;
    let cy = arrow_rect.center().y;
    let half = 7.0;
    ui.painter().line_segment(
        [egui::pos2(cx - half, cy), egui::pos2(cx + half, cy)],
        egui::Stroke::new(2.0, arrow_color),
    );
    ui.painter().line_segment(
        [egui::pos2(cx + 2.0, cy - 5.0), egui::pos2(cx + half, cy)],
        egui::Stroke::new(2.0, arrow_color),
    );
    ui.painter().line_segment(
        [egui::pos2(cx + 2.0, cy + 5.0), egui::pos2(cx + half, cy)],
        egui::Stroke::new(2.0, arrow_color),
    );
    ui.add_space(4.0);
}

fn render_route_slot(
    ui: &mut egui::Ui,
    setter: &ParamSetter,
    id_prefix: &str,
    slot: usize,
    dest: &IntParam,
    amount: &FloatParam,
) {
    ui.horizontal(|ui| {
        paint_arrow(ui);
        render_dest_combo(ui, &format!("{}_{}_dest", id_prefix, slot), COL_DEST_COMBO,
            dest.value(), |v| setter.set_parameter(dest, v));
    });
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        ui.add_space(4.0);
        set_slider_color(ui, ROUTE_AMOUNT_COLOR);
        let mut amt_val = amount.modulated_plain_value();
        ui.style_mut().spacing.slider_width = COL_SLIDER_WIDTH;
        ui.style_mut().spacing.slider_rail_height = SLIDER_RAIL;
        let slider = egui::Slider::new(&mut amt_val, -1.0..=1.0)
            .clamping(egui::SliderClamping::Always)
            .show_value(false);
        if ui.add(slider).changed() {
            setter.set_parameter(amount, amt_val);
        }
    });
}

fn render_route_slot_horizontal(
    ui: &mut egui::Ui,
    setter: &ParamSetter,
    id_prefix: &str,
    slot: usize,
    dest: &IntParam,
    amount: &FloatParam,
    dest_width: f32,
    amount_width: f32,
) {
    paint_arrow(ui);

    render_dest_combo(ui, &format!("{}_{}_dest", id_prefix, slot), dest_width,
        dest.value(), |v| setter.set_parameter(dest, v));

    ui.add_space(8.0);

    set_slider_color(ui, ROUTE_AMOUNT_COLOR);
    let mut amt_val = amount.modulated_plain_value();
    ui.style_mut().spacing.slider_width = amount_width;
    ui.style_mut().spacing.slider_rail_height = SLIDER_RAIL;
    let slider = egui::Slider::new(&mut amt_val, -1.0..=1.0)
        .clamping(egui::SliderClamping::Always)
        .show_value(true);
    if ui.add(slider).changed() {
        setter.set_parameter(amount, amt_val);
    }
}

fn render_dest_combo<F: FnOnce(i32)>(
    ui: &mut egui::Ui,
    id: &str,
    width: f32,
    current_value: i32,
    on_select: F,
) {
    let current_name = DEST_ENTRIES.iter()
        .find(|(_, v)| *v == current_value)
        .map(|(n, _)| *n)
        .unwrap_or("?");

    let mut selected = None;
    ui.style_mut().spacing.button_padding = egui::vec2(8.0, 6.0);
    egui::ComboBox::from_id_salt(id)
        .width(width)
        .height(420.0)
        .selected_text(egui::RichText::new(current_name).size(COMBO_FONT))
        .show_ui(ui, |ui| {
            ui.style_mut().spacing.item_spacing.y = 3.0;
            for &(name, value) in &DEST_ENTRIES {
                let btn = egui::Button::new(egui::RichText::new(name).size(COMBO_FONT))
                    .min_size(egui::vec2(width - 10.0, COMBO_BTN_HEIGHT))
                    .selected(current_value == value);
                if ui.add(btn).clicked() {
                    selected = Some(value);
                    ui.close_menu();
                }
            }
        });
    if let Some(v) = selected {
        on_select(v);
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

fn render_division_combo<F: FnOnce(usize)>(
    ui: &mut egui::Ui,
    id: &str,
    width: f32,
    current: usize,
    on_select: F,
) {
    let current_name = DIVISION_NAMES.get(current).copied().unwrap_or("?");
    let mut selected = None;
    ui.style_mut().spacing.button_padding = egui::vec2(8.0, 6.0);
    egui::ComboBox::from_id_salt(id)
        .width(width)
        .height(420.0)
        .selected_text(egui::RichText::new(current_name).size(COMBO_FONT))
        .show_ui(ui, |ui| {
            ui.style_mut().spacing.item_spacing.y = 3.0;
            for &idx in &DIVISION_DISPLAY_ORDER {
                let name = DIVISION_NAMES[idx];
                let btn = egui::Button::new(egui::RichText::new(name).size(COMBO_FONT))
                    .min_size(egui::vec2(width - 10.0, COMBO_BTN_HEIGHT))
                    .selected(current == idx);
                if ui.add(btn).clicked() {
                    selected = Some(idx);
                    ui.close_menu();
                }
            }
        });
    if let Some(idx) = selected {
        on_select(idx);
    }
}

fn get_step_params(params: &Arc<DeviceParams>) -> [&FloatParam; 16] {
    [
        &params.mseq_step_1, &params.mseq_step_2, &params.mseq_step_3, &params.mseq_step_4,
        &params.mseq_step_5, &params.mseq_step_6, &params.mseq_step_7, &params.mseq_step_8,
        &params.mseq_step_9, &params.mseq_step_10, &params.mseq_step_11, &params.mseq_step_12,
        &params.mseq_step_13, &params.mseq_step_14, &params.mseq_step_15, &params.mseq_step_16,
    ]
}

fn render_step_seq_panel(
    ui: &mut egui::Ui,
    params: &Arc<DeviceParams>,
    setter: &ParamSetter,
) {
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("STEP SEQ")
            .size(HEADER_FONT).strong());

        ui.add_space(24.0);

        ui.label(egui::RichText::new("RATE").size(FONT).color(LABEL_COLOR));
        ui.add_space(4.0);
        render_division_combo(ui, "mseq_div", 100.0,
            params.mseq_division.value() as usize,
            |i| setter.set_parameter(&params.mseq_division, i as i32));

        ui.add_space(20.0);

        ui.label(egui::RichText::new("SLEW").size(FONT).color(LABEL_COLOR));
        ui.add_space(4.0);
        set_slider_color(ui, SLEW_COLOR);
        let mut slew_val = params.mseq_slew.modulated_plain_value();
        ui.style_mut().spacing.slider_width = 140.0;
        ui.style_mut().spacing.slider_rail_height = SLIDER_RAIL;
        let slider = egui::Slider::new(&mut slew_val, 0.0..=200.0)
            .suffix(" ms")
            .clamping(egui::SliderClamping::Always);
        if ui.add(slider).changed() {
            setter.set_parameter(&params.mseq_slew, slew_val);
        }
    });

    ui.add_space(10.0);

    let step_params = get_step_params(params);
    let ties = params.mseq_ties.value() as u16;

    ui.horizontal(|ui| {
        ui.add_space(10.0);
        for (i, step_param) in step_params.iter().enumerate() {
            render_step_bar(ui, setter, i, step_param);
        }
    });

    ui.add_space(4.0);

    ui.horizontal(|ui| {
        ui.add_space(10.0);
        for i in 0..16 {
            let tied = (ties >> i) & 1 == 1;
            let new_tied = render_tie_button(ui, i, tied);
            if new_tied != tied {
                let new_ties = if new_tied {
                    ties | (1 << i)
                } else {
                    ties & !(1 << i)
                };
                setter.set_parameter(&params.mseq_ties, new_ties as i32);
            }
        }
    });

    ui.add_space(12.0);

    ui.horizontal(|ui| {
        ui.add_space(10.0);
        render_route_slot_horizontal(ui, setter, "mseq", 1, &params.mseq_dest1, &params.mseq_amount1, DEST_COMBO_WIDTH, AMOUNT_SLIDER_WIDTH);
        ui.add_space(40.0);
        render_route_slot_horizontal(ui, setter, "mseq", 2, &params.mseq_dest2, &params.mseq_amount2, DEST_COMBO_WIDTH, AMOUNT_SLIDER_WIDTH);
    });
}

fn render_step_bar(
    ui: &mut egui::Ui,
    setter: &ParamSetter,
    _step_idx: usize,
    param: &FloatParam,
) {
    let total_width = STEP_BAR_WIDTH + STEP_BAR_GAP;
    let desired_size = egui::vec2(total_width, STEP_BAR_HEIGHT);
    let (rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::click_and_drag());

    let bar_rect = egui::Rect::from_min_size(
        rect.min,
        egui::vec2(STEP_BAR_WIDTH, STEP_BAR_HEIGHT),
    );

    let value = param.value();
    let center_y = bar_rect.center().y;

    ui.painter().rect_filled(bar_rect, 2.0, Color32::from_gray(35));

    ui.painter().line_segment(
        [egui::pos2(bar_rect.left(), center_y), egui::pos2(bar_rect.right(), center_y)],
        egui::Stroke::new(1.0, Color32::from_gray(55)),
    );

    if value.abs() > 0.005 {
        let fill_color = if value > 0.0 { STEP_COLOR_POS } else { STEP_COLOR_NEG };
        let fill_top;
        let fill_bottom;
        if value > 0.0 {
            fill_top = center_y - (value * (STEP_BAR_HEIGHT / 2.0));
            fill_bottom = center_y;
        } else {
            fill_top = center_y;
            fill_bottom = center_y + (-value * (STEP_BAR_HEIGHT / 2.0));
        }
        let fill_rect = egui::Rect::from_min_max(
            egui::pos2(bar_rect.left() + 1.0, fill_top),
            egui::pos2(bar_rect.right() - 1.0, fill_bottom),
        );
        ui.painter().rect_filled(fill_rect, 0.0, fill_color);
    }

    if response.dragged() {
        let drag_delta = response.drag_delta();
        let sensitivity = 1.0 / STEP_BAR_HEIGHT;
        let new_value = (value - drag_delta.y * sensitivity).clamp(-1.0, 1.0);
        setter.set_parameter(param, new_value);
    }

    if response.double_clicked() {
        setter.set_parameter(param, 0.0);
    }
}

fn render_tie_button(
    ui: &mut egui::Ui,
    step_idx: usize,
    tied: bool,
) -> bool {
    let total_width = STEP_BAR_WIDTH + STEP_BAR_GAP;
    let desired_size = egui::vec2(total_width, TIE_BTN_SIZE);
    let (rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::click());

    let btn_rect = egui::Rect::from_min_size(
        rect.min,
        egui::vec2(STEP_BAR_WIDTH, TIE_BTN_SIZE),
    );

    let bg = if tied { TIE_COLOR_ON } else { TIE_COLOR_OFF };
    ui.painter().rect_filled(btn_rect, 2.0, bg);

    if tied && step_idx < 15 {
        let y = btn_rect.center().y;
        ui.painter().line_segment(
            [egui::pos2(btn_rect.right() - 2.0, y), egui::pos2(btn_rect.right() + STEP_BAR_GAP + 2.0, y)],
            egui::Stroke::new(2.0, TIE_COLOR_ON),
        );
    }

    let label = if tied { "T" } else { "" };
    if !label.is_empty() {
        ui.painter().text(
            btn_rect.center(),
            egui::Align2::CENTER_CENTER,
            label,
            egui::FontId::proportional(11.0),
            Color32::from_gray(30),
        );
    }

    if response.clicked() { !tied } else { tied }
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
