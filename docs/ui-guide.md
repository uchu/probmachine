# Device - UI Technical Guide

## Overview

The Device synthesizer UI is a fixed 1280×720 egui-based interface using `egui_taffy` for flex layout. It has a navigation bar at the top and 6 switchable pages below.

**Stack**: Rust, nih-plug, egui, egui_taffy

---

## Window & Root Layout

### Window
- **Size**: 1280 × 720 (fixed, set via `EguiState::from_size(1280, 720)` in `src/params.rs`)
- **Editor**: `create_egui_editor()` in `src/lib.rs`

### Root Taffy Container (`src/lib.rs:90-108`)
```
Display: Flex
FlexDirection: Column
JustifyContent: SpaceBetween
AlignItems: Stretch
Padding: left=23.5, right=0, top=0, bottom=0
Gap: 0×0
```

The 23.5px left padding offsets all content from the window edge.

### Vertical Structure
1. **Navigation bar** — fixed height (~74px)
2. **Page content** — fills remaining space

---

## Navigation Bar (`src/ui/navigation.rs`)

### Frame
| Property | Value |
|----------|-------|
| outer_margin | left: -48, right: 0, top: -16, bottom: 0 |
| inner_margin | left: 48, right: 48, top: 16, bottom: 14 |
| fill | `Color32::BLACK` |
| min/max_width | 1280.0 |

The negative outer margins extend the black background to the window edges, while inner margins keep content aligned.

### Page Tab Buttons
| Property | Value |
|----------|-------|
| min_size | 96 × 56 |
| font size | 24.0 |
| pages | Presets, Synth, Beats, Notes, Strength, Length |

### Controls (after nav buttons, left-to-right)

#### Play/Stop Button
| Property | Value |
|----------|-------|
| min_size | 56 × 56 |
| icon size | 24.0 |
| playing icon | `\u{23F9}` (stop square) |
| stopped icon | `\u{25B6}` (play triangle) |
| playing color | RGB(80, 200, 80) |
| stopped color | RGB(160, 160, 160) |

#### Volume Slider
| Property | Value |
|----------|-------|
| slider_width | 220.0 |
| slider_rail_height | 14.0 |
| range | 0.0..=1.0 |
| trailing_fill | true |
| label | none (removed) |

#### Output Level Meter (5 boxes)
| Property | Value |
|----------|-------|
| box size | 10 × 26 |
| spacing | 3.0 |
| corner_radius | 2.0 |
| green (levels 0-2) | RGB(80, 200, 80) |
| yellow (level 3) | RGB(220, 200, 60) |
| red (level 4) | RGB(220, 80, 80) |
| off | RGB(40, 40, 40) |

---

## Common UI Patterns

### Standard Page Frame
Most pages wrap their main content in this frame:
```
fill: extreme_bg_color
inner_margin: 16.0
stroke: 1.0, window_stroke.color
corner_radius: 15.0
```

### State Management
All pages use egui's temporary memory storage:
```rust
let state_id = egui::Id::new("unique_page_id");
let mut state = ui.ctx().data_mut(|d|
    d.get_temp::<PageState>(state_id).unwrap_or_default()
);
// ... modify state ...
ui.ctx().data_mut(|d| d.insert_temp(state_id, state));
```

### Shared UI State (`src/ui/shared_state.rs`)
Thread-safe state shared between audio and UI threads via `Arc<SharedUiState>`:
- `note_pool` — `Mutex<NotePool>`
- `strength_values` — `Mutex<Vec<f32>>` (96 values, 0.0-1.0)
- `preset_manager` — `Mutex<PresetManager>`
- `preset_version` — `AtomicU32`
- `cpu_load` — `AtomicU32`
- `output_level` — `AtomicU32`
- `scale` — `Mutex<Scale>`
- `stability_pattern` — `Mutex<StabilityPattern>`
- `octave_randomization` — `Mutex<OctaveRandomization>`

### ComboBox Pattern
```rust
egui::ComboBox::from_id_salt("unique_id")
    .selected_text(RichText::new(label).size(16.0))
    .width(width)
    .height(max_dropdown_height)
    .show_ui(ui, |ui| {
        ui.style_mut().spacing.item_spacing.y = spacing;
        // buttons
    });
```

### Vertical Slider Pattern (Synth page)
```rust
ui.vertical(|ui| {
    ui.set_width(48.0);
    ui.style_mut().spacing.slider_width = 160.0;
    ui.style_mut().spacing.slider_rail_height = 14.0;
    // slider widget
    ui.label(RichText::new(name).size(15.0));
    ui.label(RichText::new(value).size(12.0).weak());
});
```

---

## Page Layouts

### 1. Beat Probability (`src/ui/pages/beat_probability.rs`)

#### Header
- "Clear All" button: 80 × 28, size 14.0 (right-aligned, offset 166.0)

#### Grid Container
| Property | Value |
|----------|-------|
| min_size | 1220 × 420 |
| max_width | 1220.0 |
| inner_margin | 0.0 |
| grid inner width | 1184.0 (padding 16.0 each side) |
| slider height | 388.0 |

#### Grid Lines
- Straight/Dotted: 33 vertical lines, spacing 32.0px
- Triplet: 25 vertical lines, spacing 24.0px
- Beat lines (every 8th): RGB(40, 40, 40)
- Half-beat (every 4th): RGB(25, 25, 25)
- Quarter-beat (every 2nd): RGB(20, 20, 20)
- Sub-division: RGB(15, 15, 15)
- 5 horizontal lines: RGB(20, 20, 20)

#### Division Colors
| Division | Color |
|----------|-------|
| 1/1 | RGBA(255, 100, 100, α) red |
| 1/2, 1/2T, 1/2D | RGBA(255, 150, 100, α) orange |
| 1/4, 1/4T, 1/4D | RGBA(255, 255, 100, α) yellow |
| 1/8, 1/8T, 1/8D | RGBA(100, 255, 100, α) green |
| 1/16, 1/16T, 1/16D | RGBA(100, 100, 255, α) blue |
| 1/32, 1/32D | RGBA(150, 100, 255, α) purple |

#### Bottom Controls
- Outer frame: fill RGB(30, 30, 30), inner_margin 12.0, stroke RGB(40, 40, 40), corner_radius 15.0
- Mode buttons (S, T, D): 96 × 48, size 20.0
- Division buttons: 96 × 48, size 20.0
- Length slider: 140.0 wide, rail 10.0
- Swing slider: 140.0 wide, rail 10.0
- DragValue fields: 70 × 32
- Legato button: 80 × 32

---

### 2. Length & Position (`src/ui/pages/length.rs`)

#### Header
- No page heading (removed for touch UI)

#### Layout
- item_spacing.y: 8.0
- Two frames side-by-side (Length Modifiers + Velocity Modifiers), each min_width 440.0
- Position frame below, min_width 440.0
- Spacing between horizontal frames: 24.0

#### All Frames
Standard page frame (inner_margin 16.0, corner_radius 15.0)

#### Modifier Row Pattern
```
[Label 24px] [Slider 140px] [gap 12] [Slider 100px] [gap 12] [Slider 60px]
```
- Section headers: size 18.0, RGB(180, 180, 180)
- Slider rail height: 10.0
- Label size: 16.0
- Sub-labels ("Weak", "Any", "Strong"): size 11.0

---

### 3. Notes / Piano (`src/ui/pages/notes.rs`)

#### Header
- No page heading (removed for touch UI)
- Scale ComboBox: width 220.0, button 200 × 32
- Pattern ComboBox: width 200.0, button 180 × 32

#### Piano Container
| Property | Value |
|----------|-------|
| max_width | 1220.0 |
| inner_margin | 16.0 |
| keyboard width | 1180.0 |
| white key height | 170.0 |
| visible white keys | 15 |
| white key width | ~78.67 |
| black key width | white × 0.6 (~47.2) |
| black key height | white height × 0.6 (102) |

#### Key Colors
| State | White Key | Black Key |
|-------|-----------|-----------|
| Normal | RGB(255, 255, 255) | RGB(30, 30, 30) |
| Hovered | RGB(220, 220, 220) | RGB(60, 60, 60) |
| Selected | RGB(150, 180, 220) | RGB(80, 120, 180) |
| Root stroke | RGB(255, 100, 100), 2.0 | RGB(255, 100, 100), 2.0 |

#### Key Indicator Colors (per-key vertical sections)
- Chance: RGBA(120, 120, 120, 150) / RGBA(80, 80, 80, 150)
- Strength: RGBA(100, 150, 255, 120) weak, RGBA(50, 100, 200, 120) strong
- Length: RGBA(100, 200, 100, 120) short, RGBA(50, 150, 50, 120) long

#### Scrollbar
- Width: 1180.0, Height: 24.0
- Handle: RGB(140, 140, 140), hovered RGB(120, 120, 120), dragged RGB(100, 100, 100)
- Handle stroke: 1.0, RGB(80, 80, 80)

#### Bottom Panels (side-by-side)
- Selected Note Info: max_width 450.0
- Octave Randomization: max_width 450.0
- Slider label width: 80.0, slider width: 280.0
- Direction buttons: 60 × 28, size 14.0

---

### 4. Strength (`src/ui/pages/strength.rs`)

#### Header
- No page heading (removed for touch UI)
- Style ComboBox: width 180.0, button 160 × 40

#### Grid
Same structure as Beat Probability grid (1220 × 420, 1184 inner width).

#### Opposite Mode Indicators
- Stroke: 2.0, RGBA(200, 100, 100, 150)

#### Bottom Controls
- Mode buttons (S, T): 96 × 48, size 20.0

---

### 5. Synth (`src/ui/pages/synth.rs`)

#### Header
- No page heading (removed for touch UI)
- Tab buttons: "Sound" 80 × 28, "Envelopes & FX" 140 × 28, "Mod" 80 × 28
- Tab corner_radius: top-left/right 6.0, bottom 0
- Selected tab color: RGB(60, 100, 160)

#### Tab 0: Sound

**PLL Oscillator Frame** — standard frame + sliders

| Slider | Color |
|--------|-------|
| Oct, Tune, Fine | RGB(80, 80, 40) |
| Trk, Dmp, Inf | RGB(40, 40, 80) |
| StΔ | RGB(80, 40, 80) |
| Mlt | RGB(40, 40, 80) |
| OT | RGB(100, 80, 60) |
| Sat | RGB(60, 80, 100) |
| Rng | RGB(60, 100, 80) |
| XFB | RGB(100, 60, 80) |
| FM, Rat | RGB(100, 60, 100) |
| Vol | RGB(40, 80, 40) |

**Moog Filter** — inner_margin 16, sliders: Cut/Res RGB(180, 120, 60), Env/Drv RGB(140, 100, 80)

**Sub** — inner_margin left 16, right 8, top/bottom 16. Single Vol slider RGB(40, 80, 40)

**VPS OSC** — inner_margin left 16, right 8, top/bottom 16

**Color** — inner_margin 16. Sliders: Drft/Rate RGB(80, 100, 80), Nois RGB(100, 100, 60), Tube RGB(140, 80, 80), Dist RGB(180, 60, 60)

#### Tab 1: Envelopes & FX

**VOL ENV / FILT ENV** — 4 sliders each (A, D, S, R), Exponential(2.0) scale, spacing 4.0

**REVERB** — 9 sliders, Dry/Wet color RGB(100, 80, 140)

**PERF Frame** — inner_margin left/right 16, top/bottom 12, corner_radius 10.0
- CPU color thresholds: >80% red, >50% yellow, <50% green
- Bypass checkboxes: PLL, VPS, Color, Reverb (size 14.0)
- Oversampling ComboBox: width 70.0
- Option buttons: 60 × 36

#### Tab 2: Mod (`src/ui/pages/modulation.rs`)

Rendered via `modulation::render(tui, params, setter)` when tab 2 is selected.

**LFO Panel (×3)** — Standard page frame. Spacing between panels: 12.0

**Controls per LFO**:
- Title: "LFO X" size 18.0, bold
- Waveform ComboBox: width 100.0, button 90 × 36
- Sync checkbox: "Sync" size 14.0
- Rate slider (free): width 120.0, logarithmic, suffix " Hz"
- Division ComboBox (synced): width 90.0, button 80 × 36
- Sync source ComboBox: width 95.0, button 85 × 36
- Phase mod slider: width 80.0

**Mod Slots (2 per LFO)**:
- Arrow indicator: "→" size 18.0, RGB(100, 150, 200)
- Destination ComboBox: width 130.0, button 120 × 32
- Amount slider: width 140.0, range -1.0..=1.0
- Spacing between slots: 40.0

---

### 6. Presets (`src/ui/pages/presets.rs`)

#### Header
- No page heading (removed for touch UI)

#### Panel Layout
No egui Frame is used. Background is painted manually edge-to-edge via `ui.painter().rect_filled()`.
Content is positioned using `ui.allocate_new_ui()` with a precise content rect.

| Property | Value |
|----------|-------|
| background | extreme_bg_color, painted edge-to-edge |
| padding | 20px from screen edges (all sides) |
| content top offset | +2px additional |
| flex | `flex_grow: 1.0` fills remaining vertical space |

#### Section Toggle
- Factory/User buttons: 120 × 48, size 18.0 bold
- Factory selected: RGB(60, 100, 160)
- User selected: RGB(100, 80, 60)
- Unselected: RGB(50, 50, 50)

#### Bank Buttons (2 columns × 4 rows, alongside preset grid)
- Size: dynamic (calculated to fill available space), size 22.0 bold
- corner_radius: 4.0 (matches preset buttons)
- Spacing: 6.0 (item_spacing.x)
- Selected: same hue as unselected but with 1.6× saturation boost
- Unselected: unique hue per bank (low saturation)

| Bank | Color (r, g, b) | Hue |
|------|-----------------|-----|
| 0 | (65, 45, 45) | warm red |
| 1 | (60, 55, 38) | amber |
| 2 | (40, 58, 45) | forest |
| 3 | (40, 52, 62) | steel blue |
| 4 | (52, 42, 62) | violet |
| 5 | (62, 42, 55) | rose |
| 6 | (42, 58, 58) | teal |
| 7 | (55, 50, 42) | sand |

#### Preset Grid (8 columns × 4 rows)
- Button size: dynamic, calculated to fill available space
  - `btn_w = (content_w - 9 * spacing - bank_gap) / 10`
  - `btn_h = (grid_h - 3 * row_gap) / 4 - 3`
- corner_radius: 4.0
- item_spacing.x: 6.0 (overridden per row)
- Row spacing: 6.0
- Bank-to-preset gap: 8px (add_space) + item_spacing
- Header-to-grid gap: 22px

##### Preset Button Content
- **Number**: top-left, 18px proportional, RGB(160, 160, 160)
- **Star** (favorite): top-right corner, 14px, RGB(220, 180, 60)
- **Name**: below number (y offset 32px), 18px proportional, RGB(220, 220, 220), wrapping with hidden overflow (clip rect)
- Group start: every 4th preset (i % 4 == 0), slightly darker

##### Preset Button Colors
| State | Color |
|-------|-------|
| Current (loaded) | RGB(80, 140, 80) |
| Selected (Factory) | RGB(70, 100, 140) |
| Selected (User) | RGB(100, 80, 60) |
| Empty (group start) | RGB(28, 28, 28) |
| Empty | RGB(35, 35, 35) |
| Normal | tinted to match current bank color (~35% blend from base 36) |
| Group start | tinted to match current bank color (~30% blend from base 36) |

#### Action Area (Browse mode)
- Preset name: size 18.0 bold
- Favorite button: 48 × 48, filled RGB(180, 140, 40), empty RGB(60, 60, 60)
- Init button: 80 × 48, RGB(100, 70, 70)
- Save button: 80 × 48, RGB(80, 100, 60)

#### Action Area (Save mode)
- Name field: width 180.0
- Author field: width 120.0
- Cancel button: 90 × 48
- Confirm button: 100 × 48, RGB(80, 120, 80)

---

## Dimension Reference

| Element | Width | Height | Font |
|---------|-------|--------|------|
| Window | 1280 | 720 | — |
| Nav tab button | 96 | 56 | 20 |
| Play button | 56 | 56 | 24 |
| Level meter box | 10 | 26 | — |
| Volume slider | 220 | — | — |
| Beat/Strength grid | 1220 | 420 | — |
| Grid inner | 1184 | 388 | — |
| Piano keyboard | 1180 | 170 | — |
| Preset button | dynamic | dynamic | 18 |
| Bank button | dynamic | dynamic | 22 |
| Synth vertical slider | 48 | — | 15 |
| Standard DragValue | 70 | 32 | — |

## Spacing Conventions

| Context | Value |
|---------|-------|
| Section gap | 8–16 |
| Widget gap | 4–12 |
| Frame inner_margin (standard) | 16 |
| Presets panel padding | 20px from screen edges (no Frame) |
| Frame corner_radius | 15 (standard), 10 (small) |
| Frame stroke width | 1.0 |
| Heading indent | 4 spaces ("    ") |
| Heading top spacing | 12–16 |
| Heading bottom spacing | 8–12 |

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

### Synth Slider Categories
| Category | Color |
|----------|-------|
| Pitch (Oct/Tune/Fine) | RGB(80, 80, 40) |
| Tracking (Trk/Dmp/Inf) | RGB(40, 40, 80) |
| Stereo | RGB(80, 40, 80) |
| Overtone | RGB(100, 80, 60) |
| FM/Modulation | RGB(100, 60, 100) |
| Volume/Mix | RGB(40, 80, 40) |
| Filter | RGB(180, 120, 60) |
| Reverb | RGB(100, 80, 140) |
| Noise/Drift | RGB(80, 100, 80) / RGB(100, 100, 60) |
| Distortion | RGB(140, 80, 80) / RGB(180, 60, 60) |
