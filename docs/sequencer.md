# Device - Probability Sequencer

## Overview

Device uses a probability-based sequencer instead of traditional step patterns. Each beat position has a probability value (0-127), and the sequencer generates patterns by rolling against these probabilities each bar.

## Beat Divisions

### Straight Divisions
Standard binary subdivisions of a 4/4 bar.

| Division | Beats | Description |
|----------|-------|-------------|
| 1/1 | 1 | Whole bar |
| 1/2 | 2 | Half notes |
| 1/4 | 4 | Quarter notes |
| 1/8 | 8 | Eighth notes |
| 1/16 | 16 | Sixteenth notes |
| 1/32 | 32 | Thirty-second notes |

### Triplet Divisions
Three evenly spaced beats per reference note.

| Division | Beats | Description |
|----------|-------|-------------|
| 1/2T | 3 | Half-note triplets |
| 1/4T | 6 | Quarter-note triplets |
| 1/8T | 12 | Eighth-note triplets |
| 1/16T | 24 | Sixteenth-note triplets |

### Dotted Divisions
Notes 1.5× the length of their standard counterpart.

| Division | Beats | Description |
|----------|-------|-------------|
| 1/2D | 2 | Dotted half notes |
| 1/4D | 3 | Dotted quarter notes |
| 1/8D | 6 | Dotted eighth notes |
| 1/16D | 11 | Dotted sixteenth notes |
| 1/32D | 22 | Dotted thirty-second notes |

## Probability System

### How It Works

1. **Per-beat probabilities**: Each beat position has a 0-127 value
2. **Competition**: When beats from different divisions start at the same time, they compete
3. **Random selection**: A random value (0-127) determines which beat triggers
4. **Duration ownership**: When a beat wins, it occupies its full duration - no other beats can trigger until it ends
5. **Bar regeneration**: Patterns are generated at the start of each bar

### Probability Algorithm

```
occupied_until = 0
lost_beats = []  // (end_time, probability) pairs

For each unique start time (sorted):
  1. If start_time < occupied_until, skip (a beat is playing)
  2. Find all beats starting at this time (candidates)
  3. Sum their probabilities (total_probability)
  4. Calculate lost_probability from beats that lost earlier but would extend past this time
  5. remaining_space = 127 - lost_probability
  6. Roll random(0, remaining_space)
  7. If roll < total_probability:
     - Select winner proportionally
     - Trigger that beat's note
     - Set occupied_until = winner's end time
     - Add losing candidates to lost_beats
  8. Else:
     - Add all candidates to lost_beats (they lost this competition)
```

### Example: Pattern Selection

If 1/1 = 64 and all four 1/4 beats = 64 (total fills the space):

At time 0.0:
- Candidates: 1/1 (64), 1/4 beat 1 (64)
- Total = 128, remaining_space = 127
- Roll 0-127: either 1/1 or 1/4 beat 1 wins

**If 1/1 wins**: occupies entire bar, done.

**If 1/4 beat 1 wins**:
- 1/1 (64) added to lost_beats with end_time=1.0
- At time 0.25: remaining_space = 127 - 64 = 63
- 1/4 beat 2 (64) exceeds remaining_space → always triggers
- Same for beats 3 and 4

**Result**: Either 1/1 plays alone, OR all four 1/4 beats play. The probability determines which pattern, not whether individual beats trigger.

### Time Overlap Visualization

```
Bar: |========================================|
1/1: |████████████████████████████████████████|
1/2: |████████████████████|████████████████████|
1/4: |██████████|██████████|██████████|██████████|
1/8: |█████|█████|█████|█████|█████|█████|█████|█████|
T:   |███████|███████|███████|███████|███████|███████|
```

When multiple beats align at the same start time, they compete. Losing beats reduce the probability space for later times they would have covered, ensuring shorter beats fill in when longer beats lose.

## Note Selection

### Note Pool

Instead of a single pitch, notes are selected from a weighted pool.

**Per-Note Settings:**
| Setting | Range | Description |
|---------|-------|-------------|
| Chance | 0-127 | Base probability of selection |
| Strength Bias | -127 to +127 | Preference for weak/strong beats |

### Root Note

A designated root note always has 100% chance and serves as a fallback if no other note is selected.

### Strength-Based Selection

The Strength page defines a 96-position grid (0.0-1.0 per position) that affects note selection:

- **Positive Bias**: Note prefers strong beats (high strength positions)
- **Negative Bias**: Note prefers weak beats (low strength positions)
- **Zero Bias**: No preference

**Selection Algorithm:**
```
effective_chance = base_chance × strength_modifier

where strength_modifier =
  if bias > 0: 1.0 + (bias × position_strength)
  if bias < 0: 1.0 + (-bias × (1.0 - position_strength))
  else: 1.0
```

## Note Duration

### Base Length

| Param | Range | Description |
|-------|-------|-------------|
| Note Length % | 1-200% | Base duration as percentage of beat |

### Length Modifiers

Two modifier slots, each with:

| Param | Range | Description |
|-------|-------|-------------|
| Target | -100 to +100 | Which beats to affect (neg=weak, pos=strong) |
| Amount | 0-200% | Duration multiplier (100%=no change, 200%=2× length) |
| Probability | 0-127 | Chance of modifier applying |

**"Up To" Behavior:**
When a modifier applies, it uses a random value between the base (100%) and the configured amount:
- Amount 150%: applies random multiplier between 1.0× and 1.5×
- Amount 50%: applies random multiplier between 0.5× and 1.0×
- Amount 100%: no change

**How Target Matching Works:**
- Value at center (Any): Affects ALL notes
- Negative values (toward Weak): Targets weak beats (lower strength) - more extreme = only weakest
- Positive values (toward Strong): Targets strong beats (higher strength) - more extreme = only strongest

**Relative Targeting:**
Target matching uses RELATIVE values based on the actual distribution of strengths/lengths in your configuration:

- The system finds the min and max strength values in your Strength grid
- Each note's strength is normalized to this range (0 = minimum, 1 = maximum)
- Similarly for length: finds min/max duration from enabled beat divisions

**Example:** If your Strength grid has values 0.0, 0.3, and 0.5:
- A note at strength 0.0 → relative value 0.0 (weakest)
- A note at strength 0.3 → relative value 0.6 (middle)
- A note at strength 0.5 → relative value 1.0 (strongest)
- At target -100 (Weak): only the 0.0 strength notes are affected
- At target +100 (Strong): only the 0.5 strength notes are affected

**Target Matching Scale (after normalization):**
| Target Value | Notes Affected |
|--------------|----------------|
| -100 (Weak) | Only notes with relative value < 0.15 |
| -50 | Notes with relative value < ~0.60 |
| 0 (Any) | All notes |
| +50 | Notes with relative value > ~0.40 |
| +100 (Strong) | Only notes with relative value > 0.85 |

## Velocity Modifiers

Control MIDI velocity based on beat characteristics. Two modifier slots with target-based selection.

### Strength-Based Velocity

| Param | Range | Description |
|-------|-------|-------------|
| Target | -100 to +100 | Which beats to affect (Weak/Any/Strong) |
| Amount | -99 to +27 | Velocity offset (base 100, range 1-127) |
| Probability | 0-127 | Chance of modifier applying |

Modify velocity based on beat strength from the Strength grid.

### Length-Based Velocity

| Param | Range | Description |
|-------|-------|-------------|
| Target | -100 to +100 | Which notes to affect (Short/Any/Long) |
| Amount | -99 to +27 | Velocity offset (base 100, range 1-127) |
| Probability | 0-127 | Chance of modifier applying |

Modify velocity based on beat duration. Duration is normalized using log scale:
- Shortest beats (1/32): length value = 0.0
- Longest beats (1/1): length value = 1.0

**"Up To" Behavior:**
When a modifier applies, it uses a random value between 0 and the configured amount:
- Amount +20: applies random offset between 0 and +20
- Amount -30: applies random offset between -30 and 0
- Amount 0: no change

**How It Works:**
1. Start with base velocity = 100
2. If strength target matches and probability check passes, add random value up to strength amount
3. If length target matches and probability check passes, add random value up to length amount
4. Clamp result to 1-127

**Use Cases:**
- Target Strong + positive amount: Emphasize downbeats (with variation)
- Target Weak + negative amount: Soften off-beats (with variation)
- Target Long + positive amount: Weight on sustained notes
- Target Short + negative amount: Quieter ghost notes

## Position Modifiers (Humanization)

Shift notes slightly from their exact grid position. Two modifier slots available.

| Param | Range | Description |
|-------|-------|-------------|
| Target | -100 to +100 | Which beats to affect (Weak/Any/Strong) |
| Shift | -50% to +50% | Maximum position shift as % of beat duration |
| Probability | 0-127 | Chance of modifier applying |

Uses the same target matching as length modifiers - center (Any) affects all notes, toward Weak affects only weaker beats, toward Strong affects only stronger beats.

**"Up To" Behavior:**
When a modifier applies, it uses a random value between 0 and the configured shift:
- Shift +10%: applies random shift between 0% and +10%
- Shift -15%: applies random shift between -15% and 0%
- Shift 0%: no shift

**Use Cases:**
- Target Weak + negative shift: Pull weak beats early for drive (with variation)
- Target Strong + positive shift: Push strong beats late for laid-back feel (with variation)
- Target Any + small shift: Random micro-timing for organic feel

## Swing

Global swing affects eighth-note timing.

| Value | Feel |
|-------|------|
| 50% | Straight |
| 66% | Triplet swing |
| 75% | Hard swing |

**How Swing Works:**
Within each quarter note, the second eighth is delayed:
- At 50%: Both eighths equal duration
- At 66%: First eighth is 66% of quarter, second is 34%
- At 75%: First eighth is 75%, second is 25%

## Legato Mode

Legato mode changes how consecutive notes are played.

| Param | Range | Description |
|-------|-------|-------------|
| Legato | On/Off | Toggle legato behavior |
| Time | 1-500ms | Glide time between notes |

**Behavior:**
- **Legato Off**: Each note triggers envelopes from the start, creating distinct attacks
- **Legato On**: When a new note plays while the previous is still sounding, envelopes continue without retriggering and pitch glides smoothly to the new note

**Use Cases:**
- Smooth melodic lines without repeated attacks
- Portamento/glide effects at slower time settings
- Bass lines with connected phrasing
- Fast passages that maintain smooth timbre

## Tempo

Currently fixed at 120 BPM in 4/4 time.

**Note:** Future versions may support:
- Variable tempo
- Different time signatures
- Host tempo sync

## Pattern Generation

### Bar-Level Processing

1. Hash all beat probability parameters
2. If hash changed, regenerate next bar
3. Generate events for all beats whose roll succeeds
4. Double-buffer: Current bar plays while next bar is prepared

### Double Buffering

```
Bar N playing → Bar N+1 generating
                     ↓
Bar N+1 playing ← Pattern swap at bar boundary
```

This ensures smooth transitions without glitches.

## UI Pages

### Beats Page

Grid of probability sliders for each beat position. Organized by division type with color coding:
- Straight: Blue shades
- Triplet: Orange shades
- Dotted: Green shades
- Swing control

### Notes Page

Piano roll interface for note pool configuration:
- Click to add/remove notes
- Sliders for chance and strength bias
- Root note indicator

### Strength Page

96-position grid for beat strength values:
- Displayed as vertical sliders
- Grouped by beat position
- Values affect note selection and modifiers

### Length Page

Controls for note duration, velocity, and position:
- Base length percentage
- Two length modifier slots with target sliders (Weak/Any/Strong)
- Velocity modifiers:
  - Strength-based (target: Weak/Any/Strong)
  - Length-based (target: Short/Any/Long)
- Two position modifier slots with target sliders (Weak/Any/Strong)

## Creative Applications

### Generative Patterns

Set low probabilities across many divisions for ever-changing patterns that never repeat exactly.

### Polyrhythmic Textures

Combine triplet and straight divisions with moderate probabilities for complex rhythmic interplay.

### Dynamic Emphasis

Use strength-based note selection and decay modifiers to create naturally emphasizing patterns.

### Controlled Chaos

High probabilities on downbeats, low on off-beats creates patterns that are chaotic but anchored.

### Groove Templates

Use position modifiers to create specific timing feels (swing, push, pull) that vary based on beat strength.
