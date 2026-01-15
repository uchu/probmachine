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
4. **Bar regeneration**: Patterns are generated at the start of each bar

### Probability Algorithm

```
For each unique start time:
  1. Find all beats starting at this time
  2. Sum their probabilities
  3. Roll random(0, 127)
  4. If roll < total_probability:
     - Select winner proportionally
     - Trigger that beat's note
```

### Example

If at time 0.0 (bar start):
- 1/1 beat has probability 50
- 1/4 beat 1 has probability 30

Total = 80. Roll a random number:
- Roll 0-30: 1/4 beat 1 triggers
- Roll 30-80: 1/1 beat triggers
- Roll 80-127: Nothing triggers

### Time Overlap Visualization

```
Bar: |========================================|
1/1: |████████████████████████████████████████|
1/2: |████████████████████|████████████████████|
1/4: |██████████|██████████|██████████|██████████|
1/8: |█████|█████|█████|█████|█████|█████|█████|█████|
T:   |███████|███████|███████|███████|███████|███████|
```

When multiple beats align at the same start time, they compete.

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
| Amount | -100 to +100 | Duration modifier (-100=shortest, 0=no change, +100=2× length) |
| Probability | 0-127 | Chance of modifier applying |

**How Target Matching Works:**
- Value near 0: Modifier is off
- Negative values: Targets weak beats (lower strength)
- Positive values: Targets strong beats (higher strength)
- Extreme values require extreme strength to match

## Decay Modifiers

Same structure as length modifiers, but affect the volume envelope decay time.

| Param | Range | Description |
|-------|-------|-------------|
| Target | -100 to +100 | Which beats to affect |
| Amount | -100 to +100 | Decay modifier (-100=shortest, 0=no change, +100=2× decay) |
| Probability | 0-127 | Chance of modifier applying |

**Use Cases:**
- Longer decay on downbeats for emphasis
- Shorter decay on weak beats for tighter rhythm

## Position Modifiers (Humanization)

Shift notes slightly from their exact grid position.

| Param | Range | Description |
|-------|-------|-------------|
| Target | -100 to +100 | Which beats to affect |
| Shift | -50% to +50% | Position shift as % of beat duration |
| Probability | 0-127 | Chance of modifier applying |

**Use Cases:**
- Pull weak beats slightly early for drive
- Push strong beats slightly late for laid-back feel
- Random micro-timing for organic feel

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

Controls for note duration and modifiers:
- Base length percentage
- Two length modifier slots
- Two decay modifier slots
- Two position modifier slots

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
