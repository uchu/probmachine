use nih_plug_egui::egui::{self, Color32, RichText};

const FONT: f32 = 15.0;
const SEARCH_FONT: f32 = 16.0;
const GROUP_FONT: f32 = 12.0;
const BTN_HEIGHT: f32 = 38.0;
const BTN_GAP: f32 = 5.0;
const ROW_GAP: f32 = 5.0;
const PANEL_PADDING: f32 = 16.0;
const SEARCH_HEIGHT: f32 = 34.0;
const CLOSE_BTN_SIZE: f32 = 34.0;
const TOP_BAR_HEIGHT: f32 = 44.0;
const CORNER_RADIUS: f32 = 6.0;
const BTN_CORNER: f32 = 4.0;
const PANEL_BG: Color32 = Color32::from_rgb(24, 24, 28);
const BTN_BG: Color32 = Color32::from_rgb(40, 40, 48);
const BTN_HOVER: Color32 = Color32::from_rgb(55, 55, 65);
const BTN_SELECTED: Color32 = Color32::from_rgb(80, 120, 180);
const SEARCH_BG: Color32 = Color32::from_rgb(16, 16, 20);
const BORDER_COLOR: Color32 = Color32::from_rgb(50, 50, 58);
const GROUP_LABEL_COLOR: Color32 = Color32::from_gray(90);
const CLOSE_COLOR: Color32 = Color32::from_gray(120);
const CLOSE_HOVER_COLOR: Color32 = Color32::from_gray(200);

pub struct GridPickerGroup {
    pub name: &'static str,
    pub tint: Color32,
    pub entries: &'static [(&'static str, i32)],
}

#[derive(Clone, Default)]
struct PickerState {
    open: bool,
    search: String,
}

const CONTENT_RECT_ID: &str = "grid_picker_content_rect";

pub fn set_content_rect(ui: &mut egui::Ui, rect: egui::Rect) {
    ui.memory_mut(|mem| {
        mem.data.insert_temp(egui::Id::new(CONTENT_RECT_ID), rect);
    });
}

fn get_content_rect(ui: &egui::Ui) -> egui::Rect {
    ui.memory(|mem| {
        mem.data.get_temp::<egui::Rect>(egui::Id::new(CONTENT_RECT_ID))
    }).unwrap_or(ui.ctx().screen_rect())
}

pub fn grid_picker_button(
    ui: &mut egui::Ui,
    id: &str,
    width: f32,
    current_value: i32,
    groups: &[GridPickerGroup],
) -> Option<i32> {
    let picker_id = egui::Id::new(id);

    let current_name = groups.iter()
        .flat_map(|g| g.entries.iter())
        .find(|(_, v)| *v == current_value)
        .map(|(n, _)| *n)
        .unwrap_or("?");

    let is_open = ui.memory(|mem| {
        mem.data.get_temp::<PickerState>(picker_id)
            .map(|s| s.open)
            .unwrap_or(false)
    });

    let btn_size = egui::vec2(width, BTN_HEIGHT);
    let (btn_rect, btn_response) = ui.allocate_exact_size(btn_size, egui::Sense::click());

    let bg = if is_open {
        BTN_SELECTED
    } else if btn_response.hovered() {
        BTN_HOVER
    } else {
        BTN_BG
    };
    let text_col = if is_open { Color32::WHITE } else { Color32::from_gray(180) };

    ui.painter().rect_filled(btn_rect, BTN_CORNER, bg);
    ui.painter().rect_stroke(
        btn_rect, BTN_CORNER,
        egui::Stroke::new(1.0, BORDER_COLOR),
        egui::epaint::StrokeKind::Inside,
    );

    let font = egui::FontId::proportional(FONT);
    let galley = ui.painter().layout_no_wrap(current_name.to_string(), font, text_col);
    let text_pos = egui::pos2(
        btn_rect.left() + 10.0,
        btn_rect.center().y - galley.size().y / 2.0,
    );
    ui.painter().galley(text_pos, galley, text_col);

    paint_chevron(ui, btn_rect, is_open);

    if btn_response.clicked() {
        ui.memory_mut(|mem| {
            let state = mem.data.get_temp_mut_or_default::<PickerState>(picker_id);
            state.open = !state.open;
            state.search.clear();
        });
    }

    let mut result = None;

    if is_open {
        result = render_overlay(ui, picker_id, current_value, groups);
    }

    result
}

fn paint_chevron(ui: &mut egui::Ui, btn_rect: egui::Rect, is_open: bool) {
    let cx = btn_rect.right() - 14.0;
    let cy = btn_rect.center().y;
    let color = Color32::from_gray(120);
    let (y_from, y_to) = if is_open { (2.0, -2.0) } else { (-2.0, 2.0) };
    ui.painter().line_segment(
        [egui::pos2(cx - 4.0, cy + y_from), egui::pos2(cx, cy + y_to)],
        egui::Stroke::new(1.5, color),
    );
    ui.painter().line_segment(
        [egui::pos2(cx, cy + y_to), egui::pos2(cx + 4.0, cy + y_from)],
        egui::Stroke::new(1.5, color),
    );
}

fn render_overlay(
    ui: &mut egui::Ui,
    picker_id: egui::Id,
    current_value: i32,
    groups: &[GridPickerGroup],
) -> Option<i32> {
    let content_rect = get_content_rect(ui);
    let mut result = None;
    let search_id = picker_id.with("search");

    let search = ui.memory(|mem| {
        mem.data.get_temp::<PickerState>(picker_id)
            .map(|s| s.search.clone())
            .unwrap_or_default()
    });

    let overlay_layer = egui::LayerId::new(egui::Order::Foreground, picker_id.with("overlay"));
    let painter = ui.ctx().layer_painter(overlay_layer);

    painter.rect_filled(content_rect, CORNER_RADIUS, PANEL_BG);
    painter.rect_stroke(
        content_rect, CORNER_RADIUS,
        egui::Stroke::new(1.0, BORDER_COLOR),
        egui::epaint::StrokeKind::Inside,
    );

    let inner_rect = content_rect.shrink(PANEL_PADDING);
    let mut overlay_ui = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(inner_rect)
            .layer_id(overlay_layer),
    );

    let avail_w = inner_rect.width();

    let top_bar_rect = egui::Rect::from_min_size(
        overlay_ui.cursor().left_top(),
        egui::vec2(avail_w, TOP_BAR_HEIGHT),
    );
    overlay_ui.allocate_rect(top_bar_rect, egui::Sense::hover());

    let search_w = 260.0_f32.min(avail_w - CLOSE_BTN_SIZE - 16.0);
    let search_rect = egui::Rect::from_min_size(
        top_bar_rect.left_top(),
        egui::vec2(search_w, SEARCH_HEIGHT),
    );
    let top_painter = ui.ctx().layer_painter(overlay_layer);
    top_painter.rect_filled(search_rect, 4.0, SEARCH_BG);
    top_painter.rect_stroke(
        search_rect, 4.0,
        egui::Stroke::new(1.0, Color32::from_rgb(50, 50, 58)),
        egui::epaint::StrokeKind::Inside,
    );

    let text_edit_rect = search_rect.shrink2(egui::vec2(8.0, 4.0));
    let mut new_search = search.clone();
    let mut text_area = overlay_ui.new_child(
        egui::UiBuilder::new()
            .max_rect(text_edit_rect)
            .layout(egui::Layout::left_to_right(egui::Align::Center)),
    );
    let te_response = text_area.add(
        egui::TextEdit::singleline(&mut new_search)
            .font(egui::FontId::proportional(SEARCH_FONT))
            .text_color(Color32::WHITE)
            .frame(false)
            .desired_width(text_edit_rect.width())
            .hint_text(RichText::new("Search...").color(Color32::from_gray(60)))
            .id(search_id),
    );

    if !te_response.has_focus() {
        te_response.request_focus();
    }

    if new_search != search {
        ui.memory_mut(|mem| {
            let state = mem.data.get_temp_mut_or_default::<PickerState>(picker_id);
            state.search = new_search.clone();
        });
    }

    let close_rect = egui::Rect::from_min_size(
        egui::pos2(top_bar_rect.right() - CLOSE_BTN_SIZE, top_bar_rect.top()),
        egui::vec2(CLOSE_BTN_SIZE, CLOSE_BTN_SIZE),
    );
    let close_id = picker_id.with("close");
    let close_response = overlay_ui.interact(close_rect, close_id, egui::Sense::click());
    let close_color = if close_response.hovered() { CLOSE_HOVER_COLOR } else { CLOSE_COLOR };

    let close_painter = ui.ctx().layer_painter(overlay_layer);
    let cc = close_rect.center();
    let s = 8.0;
    close_painter.line_segment(
        [egui::pos2(cc.x - s, cc.y - s), egui::pos2(cc.x + s, cc.y + s)],
        egui::Stroke::new(2.0, close_color),
    );
    close_painter.line_segment(
        [egui::pos2(cc.x + s, cc.y - s), egui::pos2(cc.x - s, cc.y + s)],
        egui::Stroke::new(2.0, close_color),
    );

    if close_response.clicked() {
        close_picker(ui, picker_id);
        return result;
    }

    let grid_top = top_bar_rect.bottom() + 8.0;
    let grid_rect = egui::Rect::from_min_max(
        egui::pos2(inner_rect.left(), grid_top),
        inner_rect.max,
    );

    let search_lower = new_search.to_lowercase();
    let cols = compute_columns(grid_rect.width());
    let btn_w = (grid_rect.width() - (cols - 1) as f32 * BTN_GAP) / cols as f32;

    let group_header_h = 22.0;
    let mut cursor_y = grid_rect.top();

    let filtered_any = groups.iter().any(|g| {
        if search_lower.is_empty() { !g.entries.is_empty() }
        else { g.entries.iter().any(|(name, _)| name.to_lowercase().contains(&search_lower)) }
    });

    for group in groups {
        let filtered_entries: Vec<_> = if search_lower.is_empty() {
            group.entries.iter().collect()
        } else {
            group.entries.iter()
                .filter(|(name, _)| name.to_lowercase().contains(&search_lower))
                .collect()
        };
        if filtered_entries.is_empty() { continue; }

        if !group.name.is_empty() {
            let label_painter = ui.ctx().layer_painter(overlay_layer);
            label_painter.text(
                egui::pos2(grid_rect.left() + 2.0, cursor_y + group_header_h / 2.0),
                egui::Align2::LEFT_CENTER,
                group.name,
                egui::FontId::proportional(GROUP_FONT),
                GROUP_LABEL_COLOR,
            );
            cursor_y += group_header_h;
        }

        let rows = (filtered_entries.len() + cols - 1) / cols;
        let grid_h = rows as f32 * (BTN_HEIGHT + ROW_GAP) - ROW_GAP;

        if group.tint != Color32::TRANSPARENT {
            let tint_rect = egui::Rect::from_min_size(
                egui::pos2(grid_rect.left() - 4.0, cursor_y - 2.0),
                egui::vec2(grid_rect.width() + 8.0, grid_h + 4.0),
            );
            let bg_painter = ui.ctx().layer_painter(
                egui::LayerId::new(egui::Order::Foreground, picker_id.with("tint")),
            );
            bg_painter.rect_filled(tint_rect, 4.0, group.tint);
        }

        for (idx, &(name, value)) in filtered_entries.iter().enumerate() {
            let col = idx % cols;
            let row = idx / cols;
            let x = grid_rect.left() + col as f32 * (btn_w + BTN_GAP);
            let y = cursor_y + row as f32 * (BTN_HEIGHT + ROW_GAP);
            let entry_rect = egui::Rect::from_min_size(
                egui::pos2(x, y),
                egui::vec2(btn_w, BTN_HEIGHT),
            );

            if entry_rect.bottom() > grid_rect.bottom() { break; }

            let btn_id = picker_id.with(("entry", value));
            let response = overlay_ui.interact(entry_rect, btn_id, egui::Sense::click());

            let is_selected = *value == current_value;
            let bg = if is_selected {
                BTN_SELECTED
            } else if response.hovered() {
                BTN_HOVER
            } else {
                BTN_BG
            };
            let text_col = if is_selected { Color32::WHITE } else { Color32::from_gray(180) };

            let entry_painter = ui.ctx().layer_painter(overlay_layer);
            entry_painter.rect_filled(entry_rect, BTN_CORNER, bg);
            if is_selected {
                entry_painter.rect_stroke(
                    entry_rect, BTN_CORNER,
                    egui::Stroke::new(1.5, Color32::from_rgb(100, 150, 220)),
                    egui::epaint::StrokeKind::Inside,
                );
            }

            let font = egui::FontId::proportional(FONT);
            let galley = entry_painter.layout_no_wrap(name.to_string(), font, text_col);
            let text_pos = entry_rect.center() - galley.size() / 2.0;
            entry_painter.galley(text_pos, galley, text_col);

            if response.clicked() {
                result = Some(*value);
                close_picker(ui, picker_id);
                return result;
            }
        }

        cursor_y += grid_h + 12.0;
    }

    if !filtered_any {
        let no_match_painter = ui.ctx().layer_painter(overlay_layer);
        no_match_painter.text(
            egui::pos2(grid_rect.center().x, grid_rect.top() + 40.0),
            egui::Align2::CENTER_CENTER,
            "No matches",
            egui::FontId::proportional(FONT),
            Color32::from_gray(60),
        );
    }

    let close = ui.input(|i| {
        i.key_pressed(egui::Key::Escape)
            || (i.pointer.any_click()
                && !content_rect.contains(i.pointer.interact_pos().unwrap_or_default()))
    });

    if close {
        close_picker(ui, picker_id);
    }

    result
}

fn close_picker(ui: &mut egui::Ui, picker_id: egui::Id) {
    ui.memory_mut(|mem| {
        let state = mem.data.get_temp_mut_or_default::<PickerState>(picker_id);
        state.open = false;
        state.search.clear();
    });
}

fn compute_columns(available_width: f32) -> usize {
    let min_btn_w = 90.0;
    let max_cols = 8;
    let cols = ((available_width + BTN_GAP) / (min_btn_w + BTN_GAP)).floor() as usize;
    cols.clamp(3, max_cols)
}
