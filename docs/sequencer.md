# Probability Sequencer

## Beat Grid

152 beat slots across three timing modes per bar:

| Mode | Divisions | Slots |
|------|-----------|-------|
| Straight | 1, 2, 4, 8, 16, 32 | 63 |
| Triplet | 3, 6, 12, 24 | 45 |
| Dotted | 2, 3, 6, 11, 22 | 44 |

Each slot: probability 0–127. Values below 5 treated as off.

### Slot Index Layout (flat 0..151)

```
[0]        Straight 1/1
[1..2]     Straight 1/2
[3..6]     Straight 1/4
[7..14]    Straight 1/8
[15..30]   Straight 1/16
[31..62]   Straight 1/32
[63..65]   Triplet 1/2T
[66..71]   Triplet 1/4T
[72..83]   Triplet 1/8T
[84..107]  Triplet 1/16T
[108..109] Dotted 1/2D
[110..112] Dotted 1/4D
[113..118] Dotted 1/8D
[119..129] Dotted 1/16D
[130..151] Dotted 1/32D
```

## Probability Resolution

When multiple slots overlap at the same time point:

1. All slots with prob > 0 compete
2. Probabilities summed → random roll selects one winner (weighted)
3. Winner occupies its full duration — no other beats trigger until it ends
4. Losers tracked — their lost probability suppresses later overlapping slots
5. `remaining_space = 127 - lost_probability` for subsequent competitions

Probability is relative: slot 60 vs slot 120 at the same time → 33%/67%. Slot 60 alone → 47% (60/127).

### Pattern Generation

- Patterns regenerated at bar start (double-buffered: current plays while next prepares)
- Beat parameter hash detects changes → triggers regeneration
- Bar-level event list sorted by time, resolved sequentially

## Beat Links

Forward connections between slots: when source triggers, target is forced regardless of its own roll. Chains resolve recursively (A→B→C all trigger if A wins).

- Forced beats use source beat's note → creates legato slides
- Target slots still need nonzero probability for normal competition
- Best as short chains (2–3 beats) within the same division

## Swing

Global 50–75% affects eighth-note timing within each quarter note. At 50% both eighths are equal; at 66% first eighth takes 66% of the quarter-note duration.

## Strength Grid

96-position grid per bar (LCM of 32 straight + 24 triplet positions). Each position: value 0–100 controlling note intensity.

Two display modes: Straight (32 positions) and Triplet (24 positions). Values from the inactive mode shown as reference lines.

15 built-in presets: 4/4 Standard, Backbeat, Offbeat, Triplet Feel, Shuffle, Sparse, Dense, Polyrhythm 3:4, African, Reggae, Latin, Funk, Jazz, Ambient, Driving.

Strength values drive:
- Note selection (strength-biased notes prefer strong/weak positions)
- Velocity modulation
- Length/position modifier targeting

## Note Selection

### Note Pool

Each note has: chance (0–127), strength preference (0–127, 64=neutral), length preference (0–127, 64=neutral), octave offset (-1/0/+1).

Root note: always serves as fallback if no other note qualifies. Chance is editable (default 127).

### Selection Algorithm (per beat)

1. Get beat strength from strength grid, determine note length
2. For each note: `weight = chance × strength_match × length_match`
3. Weighted random selection from all notes
4. Fallback to root if nothing qualifies

**Strength/length match formula:**
- Preference 64 (neutral) → multiplier 1.0
- Higher preference + strong beat → boost up to 2.0×
- Higher preference + weak beat → reduce to 0.1×
- `match = clamp(1.0 + pref_norm × (beat_norm - 0.5) × 2.0, 0.1, 2.0)`

### Note Pool Sources

**Path A — Scale + Stability Pattern:** Select a scale (17 available) and a stability pattern (8 available). The scale determines which notes are enabled; the pattern sets strength/length preferences per scale degree.

**Path B — ML Suggestion:** "Suggest" on Notes page or "Both" on Beats page populates from MIDI-learned data. Scale/Pattern dropdowns switch to "Custom".

Both paths produce a NotePool. Per-note parameters are always manually adjustable after.

### Scales (17)

Major, Minor, Dorian, Phrygian, Lydian, Mixolydian, Locrian, Harmonic Minor, Melodic Minor, Pentatonic Major, Pentatonic Minor, Blues, Whole Tone, Chromatic, Japanese (In), Arabic (Hijaz), Hungarian Minor.

Chance values use interval-based matching (semitone interval from root, not positional index), ensuring correct behavior across all scale sizes.

### Stability Patterns (8)

| Pattern | Character |
|---------|-----------|
| Traditional | Stable notes on strong beats, passing tones on weak |
| Jazz Melodic | 7ths/3rds prominent, 4th avoided (bebop tradition) |
| Ambient/Drone | Root/5th dominate, sparse other notes |
| Bass Heavy | Low octave emphasis, driving rhythms |
| Melodic/Vocal | Upper register, 3rd prominent |
| Tension/Chromatic | Unstable intervals get prominence |
| Even/Balanced | Equal weighting except root |
| Pentatonic Focus | All pentatonic notes stable, wide octave range |

Patterns use functional degree mapping: each note's semitone interval → functional degree (Root, 2nd, 3rd, 4th, 5th, 6th, 7th, Tritone).

### Octave Variants

Same pitch in different octaves gets independent stability settings. Low octave = grounding/bass, main = standard melodic, upper = bright/climactic.

### Global Octave Randomization (post-processing)

After note selection, optionally shift ±1 octave. Controlled by: chance (0–127), strength preference, length preference, direction (Down/Both/Up). All conditions must pass for shift to occur.

## Style Patterns (post-processing)

Override pitch for consecutive beats using musical idioms. Applied after all beat events are generated.

**Parameters:** Style (12 styles), Chance (0–127), Complexity (1–20 limits pattern pool), Max Notes (1–10), Mode (Replace interrupts/Finish waits).

Patterns are step sequences through enabled notes (relative offsets like +1, -1, +3). Wraps octaves when stepping beyond available notes.

**12 Styles:** Classical, Blues, Jazz, Rock, Latin, Techno, Ambient, Reggae, Dubstep, Funk, Middle Eastern, Celtic. Each has 20 patterns sorted simple→complex.

## Note Duration

Base: `note_length_percent` (1–200%). Modified by 2 length modifier slots with target (Weak/Any/Strong), amount, and probability. Modifiers use "up to" randomization between base and configured amount.

Target matching uses relative values normalized to the actual min/max strength distribution in the grid.

## Velocity Modifiers

Two slots: strength-based and length-based. Each has target, amount (-99 to +27 from base 100), and probability. "Up to" randomization applies.

## Position Modifiers (Humanization)

Two slots shifting notes from grid positions. Target + shift (-50% to +50%) + probability. "Up to" randomization.

## Legato Mode

When on: consecutive notes glide without envelope retrigger. Configurable glide time (1–500ms).

## Multi-Bar Sequences

Up to 8 bars, each with independent NotePool, root note, and strength grid. Beat probabilities remain global.

**Ordering modes:** Sequential (1→2→3→4→1...), Ping-Pong (1→2→3→4→3→2→1...), Random, Weighted (per-bar weight).

At each bar boundary: next slot selected → NotePool and strength swap → new bar generates.

## Algorithmic Groove ("Groove" button)

Generates beats algorithmically without a dataset. Produces patterns using templates, random variation, and automatic linking.

**6 groove families** (weighted selection): Eighth (25%), Sixteenth (20%), Triplet (20%), Sparse Legato (10%), Dotted (10%), Polyrhythmic (15%).

Each family uses pre-designed templates with ±15% random variation. Builds in layers: foundation (anchor beats in lower division) → primary → ghost (sparse high-division notes).

**Link generation:** Chain links (consecutive beats, probability-based) and phrase links (longer legato passages). Each family tunes link parameters independently.

**Strength grid:** Each groove generates a matching 96-point grid from 6 shape types (Arc, Crescendo, Accented downbeats, Two-bar feel, Organic noise, Flat).

## Melodic Fragment System

Extracts real note sequences from MIDI files and applies generative variation.

**Parameters:** Blend (0=pure melody, 1=pure probability), Pitch Variation, Rhythm Variation, Note Drop, Octave Variation.

At each beat event: with probability `(1-blend)`, nearest fragment note's pitch is used; otherwise NotePool selection applies.

## Key Files

| File | Purpose |
|------|---------|
| `src/sequencer/mod.rs` | Sequencer engine, probability resolution, BeatLinks |
| `src/sequencer/note_utils.rs` | NotePool, note selection |
| `src/sequencer/scales.rs` | Scale definitions, StabilityPattern presets |
| `src/sequencer/styles.rs` | Style patterns (12×20) |
| `src/sequencer/algo_suggest.rs` | Algorithmic groove generator |
| `src/sequencer/ml_suggest.rs` | ML data-driven suggest |
| `src/sequencer/ml_dataset.rs` | Dataset loading, compression |
| `src/sequencer/melodic_engine.rs` | Melodic fragments + variation |
| `src/sequencer/multi_bar.rs` | Multi-bar config, ordering modes |
| `src/ui/pages/beat_probability.rs` | Beats page UI |
| `src/ui/pages/notes.rs` | Notes page UI |
| `src/ui/pages/strength.rs` | Strength page UI |
| `src/ui/pages/length.rs` | Length page UI |
