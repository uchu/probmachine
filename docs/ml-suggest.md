# ML Beat & Pitch Suggestion

## Overview

Learns beat probability distributions and pitch/interval patterns from MIDI files. Provides:
- **Beat Suggest** — populates 152 beat probability parameters
- **Pitch Suggest** — populates NotePool with interval probabilities and biases
- **Both** — applies beat + pitch from the same MIDI bar
- **Performance Params** — sets Length page modifiers from MIDI statistics

No genre classification — all MIDI files treated as one pool. Feeds into the existing probability sequencer.

## Architecture

```
midi/**/*.mid → cargo run --bin midi_extract → *.bin files
                                                    ↓
                                    Built-in: include_bytes! (in binary)
                                    External: ~/.local/share/PhaseBurn/datasets/<name>/
                                                    ↓
                                    Arc<MlDataset> shared between threads
                                                    ↓
                                    UI: Suggest / Both buttons
                                                    ↓
                                    Beat: set_parameter() × 152
                                    Pitch: NotePool via SharedUiState
```

## Dataset Sources

| Source | Location | Size Limit |
|--------|----------|------------|
| Built-in | Compiled into binary | Practical (affects binary size) |
| External | `~/.local/share/PhaseBurn/datasets/<name>/` | None |

External datasets can be LZ4-compressed (`--compress` flag). Auto-detected by `"LZ4\0"` magic header.

## MIDI Extraction

**Binary:** `src/bin/midi_extract.rs`

Each MIDI track processed independently (multi-track files → one entry per track). Drum channel (10) skipped.

### Groove Filtering (per bar)

1. 4/4 time signature required
2. 3–20 notes per bar
3. Max 2 simultaneous onsets (skip chord blocks)
4. ≥75% pitch classes fit a known scale (8 templates × 12 roots)
5. Notes span ≥25% of bar

### Deduplication

Bars hashed by: 152 beat values (rounded), swing, root pitch class, pitch entries. Duplicates removed before downsampling.

### Downsampling

Stratified by file when exceeding `--max`: each file gets proportional share, small files keep all bars, large files subsampled.

### CLI

```
--dir <PATH>     MIDI input directory (default: midi/)
--max <N>        Max distributions (default: 50000)
--name <NAME>    Dataset name (for --install)
--install        Write to ~/.local/share/PhaseBurn/datasets/<name>/
--compress       LZ4 compress output
```

### Examples

```bash
# Embedded default dataset
cargo run --bin midi_extract -- --dir midi

# Genre-specific external dataset
cargo run --bin midi_extract -- --dir midi/jazz --name jazz --install --compress
```

## Binary Formats

### beat_data.bin

```
magic:   "BTDT"
version: u8 = 2
count:   u32 LE
data:    [f32; 152] × count (LE)
swing:   [f32] × count (LE, 50.0=straight)
```

### pitch_data.bin

```
magic:   "PTDT"
version: u8 = 2
count:   u32 LE
Per distribution:
  root_pitch_class: u8 (0-11)
  note_count: u8
  Per note (4 bytes):
    semitone_offset: i8
    chance: u8 (0-127)
    strength_bias: u8 (64=neutral)
    length_bias: u8 (64=neutral)
```

### melody_data.bin

```
magic:   "MLDT"
version: u8 = 1
count:   u32 LE
Per fragment:
  root_pitch_class: u8
  note_count: u8
  Per note (6 bytes):
    relative_pitch: i8 (-24..+24)
    start_time: u16 LE (0-10000 → 0.0-1.0)
    duration: u16 LE (0-10000 → 0.0-1.0)
    velocity: u8
```

### groups.bin

```
magic:   "GRDT"
version: u8 = 1
count:   u32 LE
Per group (5 bytes):
  start_index: u32 LE
  length: u8 (1-255)
```

### perf_data.bin

```
magic:   "PFDT"
version: u8 = 1
18 × f32 LE (72 bytes): len_mod_1/2 (target, amount, prob),
vel_strength/length (target, amount, prob), pos_mod_1/2 (target, shift, prob)
```

## Suggest Algorithms

### Beat Suggest

1. Pick random bar from dataset (filtered by min_notes and style if set)
2. Scale by density (1.0 = exact source, lower = fewer beats)
3. Clamp 0–127, zero below 5
4. Constraint normalization: overlapping beat probabilities scaled so sums ≤ 127 per time segment

**StyleFilter:** All (no filter), Straight (strong_ratio ≥ 0.55), Offbeat (strong_ratio < 0.45).

### Pitch Suggest

1. Pick random pitch distribution (same bar index when using "Both")
2. Scale chances by density, minimum 32 for root
3. Apply spread folding (1.0 = original multi-octave, 0.0 = fold to single octave)
4. Strength/length biases passed through from source

### Linked / Multi-Bar

"Both" button picks one shared bar index → beat + pitch from same MIDI bar. With multi-bar active, picks consecutive bars from groups.bin.

### Live Rescaling

After generation, adjusting Density/Spread re-applies from stored raw suggestion without re-rolling. Min Notes and Style only affect initial selection.

## UI Controls

**Beats page:** Density, Spread, Min slider, Style combo, Suggest, Both, Clear All, Dataset selector.

**Notes page:** Density, Spread, Suggest.

Each page has independent Density/Spread sliders.

## Key Files

| File | Purpose |
|------|---------|
| `src/bin/midi_extract.rs` | MIDI extraction binary |
| `src/sequencer/ml_dataset.rs` | Dataset loading, compression, listing |
| `src/sequencer/ml_suggest.rs` | Beat/Pitch suggesters, linked/multi-bar |
| `src/sequencer/melodic_engine.rs` | Melodic fragments + variation |
