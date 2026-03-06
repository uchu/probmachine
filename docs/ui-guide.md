# PhaseBurn - UI Guide

## Screen & Platform

- **Window**: 1280×720 fixed, non-resizable
- **Targets**: Desktop VST3/CLAP/Standalone + 5-inch Raspberry Pi touch screen
- **Stack**: egui, egui_taffy, nih-plug
- **No DPR scaling** — all pixel values are absolute

The small touch screen is the most constraining target. Every control must be tap-friendly, every label readable at arm's length.

---

## Layout Philosophy

### Flex layout is a loose guide, not pixel-precise
egui_taffy provides flex layout (like CSS flexbox), but in practice alignment and distribution are unreliable for precise positioning. The layout approach is:

1. **Use flex for high-level structure** — column/row direction, flex_grow for filling space
2. **Use manual spacing for precision** — `ui.add_space()`, fixed widths, calculated rects
3. **Custom paint when needed** — `ui.painter()` for grids, keyboards, backgrounds

### Positioning toolkit (in order of preference)
| Technique | When to use |
|-----------|------------|
| `ui.horizontal()` / `ui.vertical()` | Basic flow layout |
| `ui.add_space(N)` | Gaps between elements |
| `ui.set_min_width()` / `ui.set_max_width()` | Section width constraints |
| `ui.style_mut().spacing.slider_width = N` | Control sizing |
| `Frame::inner_margin()` | Padding inside sections |
| `ui.allocate_exact_size()` / `ui.allocate_rect()` | Custom-painted widgets |
| `ui.allocate_new_ui()` + calculated rect | Full custom layout (Presets pattern) |

### Temporary position checks during development
When building or adjusting layout and you're unsure where elements land, temporarily check positions mid-render (`ui.cursor()`, `ui.available_width()`, `ui.max_rect()`). Use these to figure out the right spacing values, then hardcode them and remove the checks. Don't leave position checks in production code.

---

## Touch-Friendly Rules

### Minimum interactive sizes
| Element | Min size |
|---------|----------|
| Tap button | 56×48 |
| Toggle | 80×32 |
| Nav tab | 96×56 |
| Combo box | 80×32 |
| Drag value | 70×32 |

### Spacing
- Between buttons in a row: 8px minimum, 12px+ preferred
- Between toggle buttons: 8px+ gap
- Between sections: 8-16px

### Font sizes
| Context | Size |
|---------|------|
| Page/section headers | 18-20px |
| Control labels | 14-16px |
| Sub-labels / hints | 12px (absolute minimum 11px) |
| Value displays | 12-15px |
| Nav tabs | 20-24px |

---

## Page Structure

### Root (src/lib.rs)
```
Column flex, left padding 23.5px
├── Navigation bar (~74px, black background)
└── Page content (flex_grow: 1.0, ~646px available)
```

### Navigation bar
Horizontal: Page tabs (96×56 each) → Play button (56×56) → Volume slider (220px) → Level meter (5 boxes)

Pages: Presets, Synth, Beats, Notes, Strength, Length

### Standard page frame
```rust
egui::Frame {
    fill: extreme_bg_color,
    inner_margin: 16.0,
    stroke: (1.0, window_stroke.color),
    corner_radius: 15.0,
}
```

### Available content widths
After accounting for root padding, page frame margins, and tab bars:

| Page type | Available width |
|-----------|----------------|
| Standard page (no tab bar) | ~1200px |
| Synth sub-tabs (after 52px tab bar) | ~1100px |

When laying out columns, verify: `left_pad + N×col_width + (N-1)×gap` fits the available width.

### Sub-pages / tabs (Synth pattern)
Store tab selection in egui temp memory:
```rust
let tab = ui.memory_mut(|mem| *mem.data.get_temp_mut_or(id, 0u8));
```
Switch tab on click:
```rust
ui.memory_mut(|mem| mem.data.insert_temp(id, new_tab));
```

---

## Layout Patterns

### Pattern 1: Custom-painted full page (Presets — best example)
Skip Frame entirely. Paint background with `ui.painter().rect_filled()`, then use `ui.allocate_new_ui()` with a calculated content rect. Calculate button sizes dynamically from available space.

**When to use**: Full-page grids, responsive layouts that need precise control.

### Pattern 2: Vertical slider columns (Synth — good pattern)
```rust
ui.vertical(|ui| {
    ui.set_width(52.0);
    ui.style_mut().spacing.slider_width = 215.0;
    ui.style_mut().spacing.slider_rail_height = 18.0;
    // slider widget
    ui.label(RichText::new(name).size(19.0));
});
```
Pack multiple columns in a `ui.horizontal()`. Each column is self-contained.

### Pattern 3: Custom-painted grid (Beats/Strength — good pattern)
Allocate a rect, then paint grid lines, slider tracks, and handles with the painter. Handle mouse interaction via the response from `allocate_rect`.

### Pattern 4: Horizontal slider rows (Notes/Length)
```rust
ui.horizontal(|ui| {
    ui.add_sized(vec2(label_width, 20.0), Label::new(RichText::new("Label:").size(16.0)));
    ui.style_mut().spacing.slider_width = 280.0;
    ui.style_mut().spacing.slider_rail_height = 10.0;
    ui.add(Slider::new(&mut value, range));
});
```
Use fixed `label_width` for alignment across multiple rows.

### Pattern 5: Framed sections side-by-side (Length page)
```rust
ui.horizontal(|ui| {
    Frame::NONE.inner_margin(16.0).show(ui, |ui| {
        ui.set_min_width(440.0);
        // section A content
    });
    ui.add_space(24.0);
    Frame::NONE.inner_margin(16.0).show(ui, |ui| {
        ui.set_min_width(440.0);
        // section B content
    });
});
```

### Pattern 6: Manual section backgrounds (Sound/OSCs — best approach)
When you need tinted backgrounds behind sections in a horizontal layout, **do not use `Frame::fill()`** — it creates gaps between frames due to layout spacing. Instead:

1. Render all content frames without fill
2. Capture boundary positions (e.g., separator line x-coordinates)
3. Paint backgrounds on `Order::Background` layer using those boundaries

```rust
// Capture positions during layout
let line_rect = ui.available_rect_before_wrap(); // between sections

// After all content is rendered, paint backgrounds on bg layer
let bg_painter = ui.painter().clone().with_layer_id(
    egui::LayerId::new(egui::Order::Background, egui::Id::new("section_bg")),
);
let sep_x = line_rect.left() - 15.0; // separator line position
bg_painter.rect_filled(
    egui::Rect::from_min_max(egui::pos2(left, top), egui::pos2(sep_x, bottom)),
    0.0,
    Color32::from_rgba_premultiplied(6, 0, 0, 5), // very subtle tint
);
```

Key points:
- Use shared x-coordinates (separator positions) as boundaries — guarantees no gaps
- Paint on `Order::Background` layer so tints render behind content
- Use `from_rgba_premultiplied` with very low values (3-8 range) for subtle tints
- Separator lines paint on the normal foreground layer, on top of backgrounds

---

## Anti-Patterns

| Don't | Do instead |
|-------|-----------|
| Negative margins (`margin: left: -29px`) | Reposition the parent container |
| Hard-coded large offsets (`add_space(460.0)`) | Calculate from available space |
| Fonts below 11px | Use 12px minimum for any text |
| Rely on flex alignment for pixel-precise positioning | Use manual spacing + position checks |
| Use Frame for full-page backgrounds | Use `painter.rect_filled()` edge-to-edge |
| Pack too many controls horizontally | Split into sub-pages/tabs when space is tight |
| Use `Frame::fill()` for adjacent section backgrounds | Paint manually on `Order::Background` layer |
| Use `outer_margin` with negative values to extend bg | It moves content too; use manual `rect_filled` instead |
| Use `Frame::fill()` + `outer_margin` to close gaps | Gaps are inherent in layout spacing; manual painting avoids them |

---

## Page Quality Reference

| Page | Rating | Notes |
|------|--------|-------|
| Presets | Excellent | Responsive grid, dynamic sizing, clean painting |
| Beats | Good | Custom grid, well-sized controls (96×48 buttons) |
| Sound/OSCs | Good | Slider columns, section bg tints via manual painting, separator lines |
| Strength | Adequate | Reuses Beats grid pattern |
| Length | Adequate | Side-by-side frames, hard-coded widths |
| Notes (piano) | Good | Proportional key sizing, custom painting |
| Notes (controls) | Poor | Cramped panels, 10-11px fonts, too many sections side-by-side |

---

## Color Palette

### Semantic
| Purpose | Color |
|---------|-------|
| Active/On | RGB(80, 200, 80) |
| Warning/Clip | RGB(220, 80, 80) |
| Caution | RGB(220, 200, 60) |
| Disabled | RGB(160, 160, 160) |
| Factory selection | RGB(60, 100, 160) |
| User selection | RGB(100, 80, 60) |

### Backgrounds
| Element | Color |
|---------|-------|
| Nav bar | BLACK |
| Control panels | RGB(30, 30, 30) |
| Panel stroke | RGB(40, 40, 40) |
| Page content | extreme_bg_color |

### Synth slider categories
| Category | Color |
|----------|-------|
| Pitch | RGB(80, 80, 40) |
| Tracking/PLL | RGB(40, 40, 80) |
| Stereo | RGB(80, 40, 80) |
| Overtone | RGB(100, 80, 60) |
| FM/Modulation | RGB(100, 60, 100) |
| Volume/Mix | RGB(40, 80, 40) |

### Section background tints (OSCs tab)
Very subtle tints to visually differentiate oscillator sections:
| Section | Color (premultiplied RGBA) |
|---------|---------------------------|
| PLL OSC | (4, 3, 0, 3) — warm yellow |
| SUB | (6, 0, 0, 5) — red |
| SAW | (0, 3, 0, 2) — green |
| VPS | (1, 1, 8, 7) — blue |

### Logo
| Element | Style |
|---------|-------|
| "phaseburn" | 22pt bold, white |
| "ONE" | 20pt bold, RGB(230, 140, 40) orange, positioned below phaseburn, right-aligned |

### Beat division colors
| Division | Color |
|----------|-------|
| 1/1 | Red RGBA(255, 100, 100, α) |
| 1/2 variants | Orange RGBA(255, 150, 100, α) |
| 1/4 variants | Yellow RGBA(255, 255, 100, α) |
| 1/8 variants | Green RGBA(100, 255, 100, α) |
| 1/16 variants | Blue RGBA(100, 100, 255, α) |
| 1/32 variants | Purple RGBA(150, 100, 255, α) |
