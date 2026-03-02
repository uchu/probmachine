# ML Beat & Pitch Suggestion

## Overview

Learn typical beat probability distributions and pitch/interval patterns from MIDI files placed in the `midi/` folder. The system provides:

- **Beat Suggest** — populates all 152 beat probability parameters with learned patterns
- **Pitch Suggest** — populates the note pool with interval probabilities, strength biases, and length biases
- **Both** — applies beat and pitch suggestions together (multi-bar aware)
- **Performance Params** — automatically sets Length page modifiers (velocity, note length, position) from MIDI performance statistics

No genre classification — all MIDI files are treated as one pool. The system does NOT replace the existing probability sequencer — it feeds into it.

## Architecture

```
midi/**/*.mid  →  cargo run --bin midi_extract  →  beat_data.bin
                                                    pitch_data.bin
                                                    melody_data.bin
                                                    groups.bin
                                                            ↓
                                              ┌─── include_bytes! (built-in)
                                              │    compiled into plugin binary
                                              │
                                   MlDataset ─┤
                                              │
                                              └─── External files (runtime)
                                                   ~/.local/share/PhaseBurn/datasets/<name>/
                                                            ↓
                                          Arc<MlDataset> shared between threads
                                                            ↓
                                          UI: Dataset selector + Suggest / Both buttons
                                                            ↓
                                          Beat: setter.set_parameter() × 152
                                          Pitch: NotePool via SharedUiState
                                          Multi-bar: BarSlot notes via MultiBarConfig
                                          Melody: MelodicConfig via SharedUiState
```

## Dataset Sources

### Built-in (embedded)
Default dataset compiled into the binary via `include_bytes!`. Always available.

### External (runtime)
Stored in `~/.local/share/PhaseBurn/datasets/<name>/` (Mac/Linux) or equivalent via `dirs::data_local_dir()`. Each subdirectory contains `beat_data.bin`, `pitch_data.bin`, `melody_data.bin`, and optionally `groups.bin`. Selected via UI dropdown. No binary size limits.

### Compression
External datasets can be LZ4-compressed (use `--compress` flag). Format: `"LZ4\0" + u32_le(uncompressed_size) + lz4_data`. Auto-detected on load.

## MIDI Extraction

### Overview

Place `.mid` / `.midi` files in a folder. Subfolders are supported. The extractor processes every file, treating each MIDI track as a separate source. Drum channel (MIDI channel 10) is skipped. Each track's bars are filtered for groove quality, producing one distribution per qualifying bar. Results are written to binary data files.

Two output modes:
- **Embedded** (default) — writes to `src/sequencer/`, compiled into the binary via `include_bytes!`
- **External** (`--install`) — writes to `~/.local/share/PhaseBurn/datasets/<name>/`, loaded at runtime

### Groove Filtering

Each bar is filtered before inclusion in the dataset. Bars must pass all criteria:

1. **4/4 time signature** — files with non-4/4 time signatures are skipped entirely
2. **Note count bounds** — 3 to 20 notes per bar (skip sparse intros and dense chord blocks)
3. **Onset polyphony check** — max 2 notes starting at the same beat position (skip chord-heavy bars). Legato sustain overlap is not counted — only simultaneous onsets matter, since the sequencer fires note-ons at discrete positions
4. **Scale coherence** — ≥75% of unique pitch classes fit a known scale (Major, Natural Minor, Harmonic Minor, Dorian, Mixolydian, Pentatonic Major, Pentatonic Minor, Blues). Best-fit root is used as the bar's `root_pitch_class`
5. **Rhythmic spread** — notes must span ≥ 25% of the bar (skip pickup measures and fills)

Filter statistics are printed at the end of extraction showing bars skipped per reason.

### Workflow

**1. Organize your MIDI files by genre:**
```
midi/
├── jazz/
│   ├── miles.mid
│   └── coltrane.mid
├── electronic/
│   └── techno-pack/
│       ├── track01.mid
│       └── track02.mid
└── classical/
    └── bach-inventions.mid
```

**2. Build the embedded default dataset (genre fusion):**
```
cargo run --bin midi_extract -- --dir midi
cargo build
```
This processes all MIDI files under `midi/`, filters for groove quality, caps at 50K distributions, and writes to `src/sequencer/`. The result is compiled into the plugin binary.

**3. Build genre-specific external datasets:**
```
cargo run --bin midi_extract -- --dir midi/jazz --name jazz --install --compress
cargo run --bin midi_extract -- --dir midi/electronic --name electronic --install --compress
cargo run --bin midi_extract -- --dir midi/classical --name classical --install --compress
```
Each command creates a separate dataset in `~/.local/share/PhaseBurn/datasets/<name>/`. External datasets have no practical size limit — they load from disk at runtime. Use `--compress` to save disk space (LZ4, auto-decompressed on load).

**4. User selects datasets in the UI** via the dropdown on the Beats page. "Built-in" is always available; external datasets appear after extraction with `--install`.

### Deduplication

After collecting all bar distributions, the extractor removes duplicates before downsampling:

1. Each bar is hashed combining: all 152 beat distribution f32 values (rounded to integer), rounded swing value, pitch root pitch class, and each pitch entry's semitone offset + chance
2. Only the first occurrence of each unique hash is kept — bars with identical rhythms but different melodies are preserved
3. File ranges, bar numbers, and consecutive bar groups are rebuilt from the deduplicated indices
4. Stats are printed: `"Deduplication: N -> M unique distributions"`

This prevents duplicate patterns from different MIDI bars inflating the dataset and skewing random selection.

### Downsampling

When the total number of extracted bars exceeds `--max`, **stratified downsampling** preserves variety:

1. Each MIDI file gets a proportional share of the budget
2. Small files keep all their bars (their full contribution is preserved)
3. Large files are randomly subsampled from their surplus
4. Result: every file is represented, no single large file dominates
5. Consecutive bar groups are rebuilt from surviving indices after downsampling

### CLI Options

```
--dir <PATH>     MIDI input directory (default: midi/)
--max <N>        Max distributions to keep (default: 50000)
--name <NAME>    Dataset name (used with --install)
--install        Write to ~/.local/share/PhaseBurn/datasets/<name>/)
--compress       Compress output with LZ4
```

### Examples

```bash
# Embedded fusion dataset from all genres (default cap 50K)
cargo run --bin midi_extract -- --dir midi

# Large external jazz dataset, no practical cap
cargo run --bin midi_extract -- --dir midi/jazz --name jazz --install --compress --max 500000

# Small focused dataset
cargo run --bin midi_extract -- --dir midi/ambient --name ambient --install --compress

# Custom embedded cap for smaller binary
cargo run --bin midi_extract -- --dir midi --max 20000
```

## Extraction Pipeline

**Binary**: `src/bin/midi_extract.rs`

### Per-Track Processing

Each MIDI track is processed independently as a separate source. This ensures that different instruments in a multi-track MIDI file (e.g., piano and bass) produce separate distributions rather than being merged. Multi-track files output one labeled entry per track (e.g., `song.mid [track 1]`, `song.mid [track 2]`). Single-track files output just the filename.

**Drum channel skip**: MIDI channel 10 (GM drums, channel index 9) is skipped during note extraction. Only melodic/harmonic content is collected.

### Beat Extraction

1. Recursively finds all MIDI files in `midi/`
2. Parses each file with the `midly` crate
3. **Filters** files for 4/4 time signature
4. Extracts note-on/note-off pairs from each track separately, skipping drum channel (channel 10)
5. **Within-bar dedup**: notes with the same MIDI pitch at the same position (within 0.015 threshold) are merged — keeps max velocity and longest duration
6. **Bar filtering**: note count (3–20), onset polyphony (max 2 notes starting at the same position — chord detection, not sustain overlap), scale coherence (≥75%), rhythmic spread (≥25% of bar)
7. Sorts bars by bar number within each track
8. Quantizes each note-on to the coarsest matching beat slot (within 0.015 threshold), skipping 1/1 and 1/2 divisions (minimum resolution is 1/4). **Velocity-weighted**: each slot gets the maximum velocity across all notes that quantize to it (range 1–127), so accented beats produce stronger probability values than ghost notes
9. Normalizes per-bar so max value = 127
10. Detects swing per bar by measuring "and" beat displacement within quarter notes (50=straight, up to 75=hard swing)
11. Writes all bar distributions + swing values to `src/sequencer/beat_data.bin` (v2 format)

### Pitch Extraction

From the same per-track notes, for each qualifying bar:

1. Collects note-on/note-off pairs to compute durations
2. Detects root pitch class via scale coherence (best-fit root across 8 scale templates × 12 roots, fallback to most frequent low-register note)
3. Finds root octave (most common octave for root pitch class in the bar)
4. Computes per-note semitone offset from root MIDI note (preserving octave — e.g., +12 = octave above, -7 = fifth below):
   - **Chance** — occurrence count normalized to 0-127
   - **Strength bias** — ratio of notes on strong beat positions (0.0, 0.25, 0.5, 0.75) mapped to 0-127 (64=neutral)
   - **Length bias** — log-scaled duration ratio vs bar average mapped to 0-127 (64=average)
5. Writes all pitch distributions to `src/sequencer/pitch_data.bin`

Multi-octave voicings from the source MIDI are preserved. At runtime, offsets are mapped relative to the user's root note.

Each bar produces one beat distribution, one pitch distribution, AND one melodic fragment (same index = same bar).

### Melodic Fragment Extraction

From the same per-track notes, for each qualifying bar:

1. Takes matched note-on/note-off pairs (same events used for pitch extraction)
2. Sorts notes by start time within bar
3. Computes relative pitch from detected root (preserving octave — full semitone offset from reference octave 4)
4. Normalizes start_time and duration to 0.0-1.0 bar fractions
5. Caps at 32 notes per fragment
6. Writes all fragments to `src/sequencer/melody_data.bin`

### Consecutive Bar Groups

After filtering, the extractor identifies runs of consecutive qualifying bars from the same file and writes `groups.bin`. This enables the multi-bar suggest feature.

### Beat Binary Format (`beat_data.bin`)

```
magic:   [u8; 4] = "BTDT"
version: u8 = 2
count:   u32 (LE) — number of bar distributions
data:    [f32; 152] × count — each distribution (LE f32)
swing:   [f32] × count — per-bar swing values (LE f32, 50.0=straight, up to 75.0)
```

Version 1 files (no swing section) are supported — swing defaults to 50.0 for all bars.

### Pitch Binary Format (`pitch_data.bin`)

```
magic:   [u8; 4] = "PTDT"
version: u8 = 2
count:   u32 (LE) — number of pitch distributions

Per distribution (variable length):
  root_pitch_class: u8          (0-11)
  note_count:       u8
  Per note (4 bytes × note_count):
    semitone_offset: i8         (offset from root, e.g. -12 = octave below, +7 = fifth above)
    chance:          u8         (0-127)
    strength_bias:   u8         (0-127, 64=neutral)
    length_bias:     u8         (0-127, 64=neutral)
```

Version 1 files (fixed 12-interval format) are supported for backward compatibility — intervals are treated as offsets 0-11 (single octave).

### Melodic Binary Format (`melody_data.bin`)

```
magic:   [u8; 4] = "MLDT"
version: u8 = 1
count:   u32 (LE) — number of melodic fragments

Per fragment:
  root_pitch_class: u8          (0-11)
  note_count: u8
  Per note (6 bytes × note_count):
    relative_pitch: i8          (-24..+24 semitones from reference octave)
    start_time: u16 LE          (0-10000 → 0.0-1.0 bar position)
    duration: u16 LE            (0-10000 → 0.0-1.0 bar fraction)
    velocity: u8
```

### Groups Binary Format (`groups.bin`)

```
magic:   [u8; 4] = "GRDT"
version: u8 = 1
count:   u32 (LE) — number of groups

Per group (5 bytes):
  start_index: u32 (LE) — index into distribution arrays
  length:      u8        — consecutive bars (1-255)
```

### Performance Extraction

From the same qualifying bars, aggregate performance statistics are computed across all accepted notes:

1. Classify notes: "strong" = bar positions near {0.0, 0.25, 0.5, 0.75} (within 0.03), "weak" = all others
2. Classify by duration: "long" = above median duration, "short" = below median
3. Compute velocity averages for strong/weak/long/short groups
4. Compute duration ratios for strong/weak beats relative to median
5. Map to 18 Length page modifier parameters:
   - **Velocity by strength**: if strong/weak velocity difference > 3, set target toward strong/weak, amount = half the difference
   - **Velocity by length**: same logic for long/short notes
   - **Length modifier 1** (weak beats): if weak-beat duration differs from median by > 5%, set amount proportionally
   - **Length modifier 2** (strong beats): same for strong beats
   - **Position modifiers**: left at defaults (swing extraction covers timing variation)

### Performance Binary Format (`perf_data.bin`)

```
magic:   [u8; 4] = "PFDT"
version: u8 = 1
18 × f32 LE (72 bytes) in order:
  len_mod_1_target, len_mod_1_amount, len_mod_1_prob,
  len_mod_2_target, len_mod_2_amount, len_mod_2_prob,
  vel_strength_target, vel_strength_amount, vel_strength_prob,
  vel_length_target, vel_length_amount, vel_length_prob,
  pos_mod_1_target, pos_mod_1_shift, pos_mod_1_prob,
  pos_mod_2_target, pos_mod_2_shift, pos_mod_2_prob,
Total: 77 bytes
```

Optional file — datasets without `perf_data.bin` skip performance parameter application.

### Slot Ordering (flat index 0..151)

```
[0]        Straight 1/1 beat 0
[1..2]     Straight 1/2 beats 0-1
[3..6]     Straight 1/4 beats 0-3
[7..14]    Straight 1/8 beats 0-7
[15..30]   Straight 1/16 beats 0-15
[31..62]   Straight 1/32 beats 0-31
[63..65]   Triplet 1/2T beats 0-2
[66..71]   Triplet 1/4T beats 0-5
[72..83]   Triplet 1/8T beats 0-11
[84..107]  Triplet 1/16T beats 0-23
[108..109] Dotted 1/2D beats 0-1
[110..112] Dotted 1/4D beats 0-2
[113..118] Dotted 1/8D beats 0-5
[119..129] Dotted 1/16D beats 0-10
[130..151] Dotted 1/32D beats 0-21
```

## Suggest Modules

### MlDataset (`src/sequencer/ml_dataset.rs`)

- `MlDataset::builtin()` — creates from embedded data via `include_bytes!`
- `MlDataset::load_from_dir(path, name)` — loads from external directory (auto-decompresses LZ4)
- `list_datasets()` — scans data directory for available datasets
- `load_dataset(name)` — loads named dataset (or "Built-in")
- `groups: Vec<(usize, usize)>` — consecutive bar groups (start_index, length)
- `performance: Option<PerformanceParams>` — 18 modifier parameters extracted from MIDI velocity/duration statistics
- Shared between threads as `Arc<MlDataset>` (all methods are `&self`)

### BeatSuggester (`src/sequencer/ml_suggest.rs`)

- `BeatSuggester::new()` — loads `beat_data.bin` via `include_bytes!`
- `BeatSuggester::from_data(data)` — loads from arbitrary byte slice, computes per-distribution metadata
- `suggest(density, rng)` — returns `BeatSuggestion` (beats + swing)
- `suggest_with_index(density, bar_index)` — same but uses specific bar index
- `suggest_filtered(density, min_notes, style, rng)` — returns `BeatSuggestion` filtered by minimum complexity and style
- `suggest_filtered_with_index(density, min_notes, style, rng)` — same, also returns the chosen index
- `max_active_slots()` / `min_active_slots()` — range of note counts across all distributions
- `distribution_count()` — number of available bar distributions
- `flat_index(mode, count, index)` — maps division params to flat array position

Per-distribution metadata (`DistributionMeta`):
- `active_slots: u8` — count of non-zero slots in the distribution
- `strong_ratio: f32` — sum of values at strong beat positions (0.0, 0.25, 0.5, 0.75) divided by total sum

`StyleFilter` enum:
- `All` — no filtering
- `Straight` — `strong_ratio >= 0.55` (on-beat patterns)
- `Offbeat` — `strong_ratio < 0.45` (syncopated patterns)

Algorithm:
1. Pick a random bar distribution from the dataset (or use provided index)
2. Scale by `density` (1.0 = exact source, lower = fewer beats via deadzone)
3. Clamp to 0–127, zero values below 5
4. **Constraint normalization**: for each time sub-segment, if overlapping beat probabilities sum > 127, scale them proportionally so their sum = 127. Each beat uses its minimum scale factor across all segments it participates in. This ensures ML suggestions respect the same probability budget as the manual UI.
5. Return `BeatSuggestion` with beats array and per-bar swing value (detected from MIDI source)

### PitchSuggester (`src/sequencer/ml_suggest.rs`)

- `PitchSuggester::new()` — loads `pitch_data.bin` via `include_bytes!`
- `PitchSuggester::from_data(data)` — loads from arbitrary byte slice
- `suggest_pitch(density, spread, rng)` — returns `PitchSuggestion`
- `suggest_pitch_with_index(density, spread, bar_index)` — same but uses specific bar index
- `distribution_count()` — number of available pitch distributions
- `apply_pitch_suggestion(suggestion, root_note, ui_state)` — applies to NotePool
- `rescale_beat_suggestion(raw, density)` — re-scales a stored raw beat suggestion with new density
- `rescale_pitch_suggestion(raw, density, spread)` — re-scales a stored raw pitch suggestion with new density/spread

Algorithm:
1. Pick a random pitch distribution from the dataset (or use provided index)
2. Scale note chances by `density` (1.0 = exact source values)
3. Clamp to 0–127, zero chances below 8
4. Strength and length biases are passed through unmodified from the source data
5. Root (offset 0) uses source data chance scaled by density, with a minimum of 32
6. Apply `spread` folding (1.0 = original multi-octave, 0.0 = fold to single octave). Duplicate offsets after folding are merged (max chance, averaged biases)
7. Notes are stored as semitone offsets from root, preserving multi-octave voicings when spread=1.0

### Linked Suggestion (`src/sequencer/ml_suggest.rs`)

- `suggest_linked(beat, pitch, density, spread, rng)` — picks one shared bar index from `min(beat.count(), pitch.count())` and calls both suggesters with the same index
- `suggest_linked_filtered(beat, pitch, density, spread, min_notes, style, rng)` — same but filters qualifying indices by beat metadata (active_slots >= min_notes and style)
- Used by the "Both" button to ensure beat and pitch come from the same MIDI bar, preserving musical coherence

### Multi-Bar Suggestion (`src/sequencer/ml_suggest.rs`)

- `suggest_multi_bar(beat, pitch, groups, count, density, spread, rng)` — picks a random group with `length >= count`, returns `count` consecutive beat+pitch suggestions
- `suggest_multi_bar_filtered(beat, pitch, groups, count, density, spread, min_notes, style, rng)` — same but filters groups whose first bar has active_slots >= min_notes and matches style
- Used by the "Both" button when multi-bar is active — fills all bar slots with consecutive grooves from the same source
- Falls back to single-bar linked suggest if no qualifying group has enough consecutive bars

### MelodySuggester (`src/sequencer/melodic_engine.rs`)

- `MelodySuggester::new()` — loads `melody_data.bin` via `include_bytes!`
- `generate_varied(fragment_index, config, target_root, rng)` — returns varied melodic notes
- `fragment_count()` — number of available fragments
- Loaded once, lives on the Sequencer struct
- Fragment selection and variation applied each bar generation

### PitchSuggestion

```rust
pub struct PitchNoteEntry {
    pub semitone_offset: i8,    // offset from root (-48..+48)
    pub chance: u8,             // 0-127
    pub strength_bias: u8,      // 0-127, 64=neutral
    pub length_bias: u8,        // 0-127, 64=neutral
}

pub struct PitchSuggestion {
    pub notes: Vec<PitchNoteEntry>,
}
```

Notes are root-relative semitone offsets preserving octave (e.g., +12 = octave above, -7 = fifth below). The caller maps to MIDI notes as `root_note + offset`.

## UI

### Beats Page (`beat_probability.rs`)

Layout: `[Density] [Spread] [Min: slider] [Style: combo] [Suggest] [Both] [Clear All] | [Dataset]`

- **Density slider** (0.0–1.0) — controls pattern fidelity (1.0 = exact source, lower = sparser). Live: adjusting after generation re-applies from stored raw suggestion
- **Spread slider** (0.0–1.0) — controls pitch octave spread (1.0 = original multi-octave voicing from source, 0.0 = fold everything into one octave, good for monophonic synth). Live: adjusting after generation re-applies from stored raw suggestion
- **Min slider** (min_active_slots–max_active_slots) — sets minimum active slots in suggested patterns. Default = dataset minimum. Higher values produce denser patterns with more notes
- **Style combo** (All / Straight / Offbeat) — filters by strong-beat ratio. Straight selects on-beat patterns (strong_ratio >= 0.55), Offbeat selects syncopated patterns (strong_ratio < 0.45), All applies no filter
- **Suggest button** — generates and applies a filtered beat pattern, sets swing from MIDI source, resets note length to 95%, applies performance params. Stores raw suggestion for live slider adjustment
- **Both button** — generates and applies filtered beat + pitch patterns together, sets swing, resets note length, applies performance params. Uses filtered linked/multi-bar methods. Stores raw suggestions for live slider adjustment
- **Clear All button** — resets all 152 beats to 0

Notes and Style controls affect pattern selection only — they do not trigger live-rescale like Density/Spread.

### Notes Page (`notes.rs`)

- **Density slider** (0.0–1.0) — controls how many intervals are active. Live: adjusting re-applies from stored raw
- **Spread slider** (0.0–1.0) — controls pitch octave spread. Live: adjusting re-applies from stored raw
- **Suggest button** — generates and applies a pitch pattern. Stores raw for live slider adjustment

Each page has its own independent Density slider. The Beats page slider controls beat suggestion (and both beat+pitch when using "Both"). The Notes page slider controls pitch-only suggestion.

### Scale/Pattern Override

When any pitch suggestion is applied (Suggest on Notes page, or Both on Beats page), the Scale and Stability Pattern dropdowns automatically switch to "Custom". This reflects that the ML-generated intervals don't correspond to any predefined scale. The user can re-select a scale/pattern at any time to override the ML suggestion.

### State Flow

All suggestion runs on UI thread. Beat changes picked up via hash-based detection. Pitch changes applied directly to NotePool via SharedUiState. Multi-bar pitch suggestions apply to BarSlot notes in MultiBarConfig. The extracted `root_pitch_class` from MIDI (detected via scale coherence) is stored in the binary but unused at runtime — intervals are always mapped relative to the user's current root note.

## Files

| File | Purpose |
|------|---------|
| `src/bin/midi_extract.rs` | MIDI extraction binary (beat + pitch + melody + groups + performance) |
| `src/sequencer/ml_dataset.rs` | MlDataset, DatasetInfo, PerformanceParams, groups loading, list/load/decompress |
| `src/sequencer/ml_suggest.rs` | BeatSuggester + PitchSuggester + linked/multi-bar suggestion |
| `src/sequencer/melodic_engine.rs` | MelodySuggester + MelodicConfig + variation |
| `src/sequencer/beat_data.bin` | Generated beat distribution data |
| `src/sequencer/pitch_data.bin` | Generated pitch distribution data |
| `src/sequencer/melody_data.bin` | Generated melodic fragment data |
| `src/sequencer/groups.bin` | Generated consecutive bar group data |
| `src/sequencer/perf_data.bin` | Generated performance parameter data |
| `src/sequencer/multi_bar.rs` | Multi-bar config, bar ordering modes |
| `src/ui/pages/beat_probability.rs` | Beats page UI (Suggest, Both, Clear, Dataset selector) |
| `src/ui/pages/notes.rs` | Notes page UI (Suggest, multi-bar, melody) |
| `src/ui/page.rs` | Page routing (passes ui_state to all pages) |
