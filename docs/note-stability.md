# Note Stability Page

The Note Stability page controls which notes play and when, based on musical theory principles. It creates intelligent, style-aware melodies where note selection considers beat strength, note duration, and musical stability concepts.

## Overview

The system consists of five interconnected components:

1. **Scale Selection** - Which notes are available (harmonic palette)
2. **Stability Pattern** - How notes relate to beat strength/length (melodic shaping)
3. **Per-Note Parameters** - Fine-tuned control for each note
4. **Global Octave Randomization** - Post-processing melodic variation
5. **Style Patterns** - Post-processing pitch override for consecutive beats using musical idioms

## Core Concepts

### Musical Stability

In music theory, scale degrees have different levels of "stability":

| Degree | Name | Stability | Typical Role |
|--------|------|-----------|--------------|
| 1 | Root/Tonic | Most stable | Resolves phrases, foundation |
| 5 | Fifth/Dominant | Very stable | Reinforces tonality |
| 4 | Fourth/Subdominant | Stable | Strong, structural |
| 3 | Third | Moderately stable | Defines major/minor |
| 6 | Sixth | Less stable | Color, movement |
| 2 | Second | Unstable | Passing tone, tension |
| 7 | Seventh | Unstable | Leading tone, wants to resolve |

### How Note Selection Works

When the sequencer processes a beat:

1. Get beat's **strength** from Strength page (0-100)
2. Get beat's **note length** (short/medium/long)
3. For each enabled note, calculate weighted probability:
   - Base chance × strength match × length match
4. Select note via weighted random
5. Apply global octave randomization (post-processing)
6. Fallback to root if no suitable match

---

## ML Pitch Suggestion

As an alternative to manually selecting scales and patterns, the Notes page has a **Suggest** button that populates the note pool from MIDI-learned data. The Beats page has a **Both** button that applies beat and pitch suggestions together.

When ML suggestion is applied, the Scale and Pattern dropdowns automatically switch to **Custom**, reflecting that the generated intervals come from learned data rather than a predefined scale. The user can re-select a scale/pattern at any time to override the suggestion.

Controls: **Density** (0–1) scales interval chances, **Variation** (0–1) adds randomness to chances and biases.

See `docs/ml-beat-suggestion.md` for full details on the extraction pipeline and suggestion algorithms.

---

## Scale Selection Dropdown

Selecting a scale enables notes and sets their base chance values.

### Available Scales

| Scale | Intervals | Notes (C root) | Character |
|-------|-----------|----------------|-----------|
| **Major (Ionian)** | 1, 2, 3, 4, 5, 6, 7 | C D E F G A B | Bright, happy, resolved |
| **Minor (Aeolian)** | 1, 2, b3, 4, 5, b6, b7 | C D Eb F G Ab Bb | Dark, sad, introspective |
| **Dorian** | 1, 2, b3, 4, 5, 6, b7 | C D Eb F G A Bb | Minor but hopeful, jazzy |
| **Phrygian** | 1, b2, b3, 4, 5, b6, b7 | C Db Eb F G Ab Bb | Spanish, exotic, tense |
| **Lydian** | 1, 2, 3, #4, 5, 6, 7 | C D E F# G A B | Dreamy, floating, bright |
| **Mixolydian** | 1, 2, 3, 4, 5, 6, b7 | C D E F G A Bb | Bluesy major, rock |
| **Locrian** | 1, b2, b3, 4, b5, b6, b7 | C Db Eb F Gb Ab Bb | Diminished, unstable |
| **Harmonic Minor** | 1, 2, b3, 4, 5, b6, 7 | C D Eb F G Ab B | Classical, exotic |
| **Melodic Minor** | 1, 2, b3, 4, 5, 6, 7 | C D Eb F G A B | Jazz minor |
| **Pentatonic Major** | 1, 2, 3, 5, 6 | C D E G A | Simple, folk, universal |
| **Pentatonic Minor** | 1, b3, 4, 5, b7 | C Eb F G Bb | Blues, rock, universal |
| **Blues** | 1, b3, 4, b5, 5, b7 | C Eb F Gb G Bb | Blues with blue note |
| **Whole Tone** | 1, 2, 3, #4, #5, b7 | C D E F# G# Bb | Dreamy, ambiguous |
| **Chromatic** | All 12 semitones | All notes | Atonal, experimental |
| **Japanese (In)** | 1, b2, 4, 5, b6 | C Db F G Ab | Japanese traditional |
| **Arabic (Hijaz)** | 1, b2, 3, 4, 5, b6, b7 | C Db E F G Ab Bb | Middle Eastern |
| **Hungarian Minor** | 1, 2, b3, #4, 5, b6, 7 | C D Eb F# G Ab B | Eastern European |

### Scale Selection Behavior

When a scale is selected:
- Notes IN the scale: chance > 0 (based on interval from root, not positional degree)
- Notes OUTSIDE the scale: chance = 0 (disabled)
- Root note: always chance = 127 (guaranteed available)

**Interval-based matching:** Chance values are determined by each note's semitone interval from the root (0-11), not by its positional index in the scale array. This ensures musical correctness for all scale sizes — e.g., the 5th (interval 7) gets proper importance in pentatonic and other non-7-note scales.

Scale-specific chance values:
- **Pentatonic Major**: Root=127, 5th(7)=105, 3rd(4)=95, 6th(9)=90, 2nd(2)=85
- **Pentatonic Minor**: Root=127, 5th(7)=105, b3(3)=95, 4th(5)=90, b7(10)=85
- **Blues**: Root=127, 5th(7)=100, b3(3)=90, b7(10)=85, 4th(5)=80, tritone(6)=65
- **Whole Tone**: Root=127, all others=75 (equidistant intervals)
- **Chromatic**: Root=127, all others=70
- **Japanese**: Root=127, 5th(7)=100, 4th(5)=85, b6(8)=65, b2(1)=55
- **Hungarian**: Root=127, 5th(7)=100, b3(3)=80, #4(6)=65, b6(8)=60, 2nd(2)=45, 7th(11)=40
- **Arabic**: Root=127, 5th(7)=100, 4th(5)=90, 3rd(4)=85, b6(8)=60, b2(1)=50, b7(10)=45
- **Default 7-note**: Root=127, 5th(7)=100, 4th(5)=90, 3rd(3|4)=80, 6th(8|9)=60, 2nd(1|2)=45, 7th(10|11)=40, tritone(6)=35

### Root Note Behavior

The root note has special handling:

| Parameter | Editable? | Value | Reason |
|-----------|-----------|-------|--------|
| **Chance** | Yes | 0-127 (default 127) | Editable; root still serves as fallback if nothing else qualifies |
| **Strength Preference** | Yes | 0-127 | Root can prefer strong/weak beats |
| **Length Preference** | Yes | 0-127 | Root can prefer short/long notes |

**Editable root chance:** The root note's chance slider is now fully editable. Lowering it reduces the root's weight in the probability pool, making it less likely to be selected through normal weighted random selection. However, the root always serves as a **fallback** — if no other note qualifies (all weighted chances are zero), the sequencer falls back to the root note regardless of its chance value.

A "(fallback)" label appears next to the chance slider when the root is selected, reminding the user of this safety net.

**Why root note biases matter:**

The root note competes with other notes through weighted probability. Its strength and length preferences affect *when* it's most likely to play:

| Configuration | Musical Effect |
|---------------|----------------|
| Chance=127, Strength=127, Length=127 | Root anchors downbeats with sustained notes |
| Chance=60, Strength=0, Length=0 | Root is a soft presence, fills gaps as quick passing notes |
| Chance=127, Strength=64, Length=64 | Root equally likely everywhere (default) |
| Chance=30, Strength=127, Length=0 | Root rarely selected, but when it is, on punchy downbeat accents |

**The math behind it:**

With bias system that only *boosts* (never reduces below base chance):
- Root at Chance=127, Strength=127 on a strong beat: effective chance = 127 × 2.0 = 254
- Root at Chance=60, Strength=127 on a weak beat: effective chance = 60 × 1.0 = 60
- Other notes still compete based on their own weights

The fallback mechanism ensures root plays if no other note qualifies, but with adjustable chance, the root's participation in weighted selection is fully configurable.

---

## Stability Pattern Dropdown

Selecting a stability pattern adjusts how notes respond to beat strength and length.

Patterns use **interval-based functional degree mapping** internally: each note's semitone interval from root is converted to a functional degree (Root, 2nd, 3rd, 4th, 5th, 6th, 7th, Tritone) for pattern lookup. This ensures correct behavior across all scale sizes.

Tritone (interval 6) handling varies by pattern:
- **Traditional**: Very unstable (20/20, octave 0)
- **Jazz Melodic**: Used in dominant chords (75/50, octave 0)
- **Melodic**: Passing tone (40/30, octave 0)
- **Tension**: Maximum tension creator (90/80, octaves 0,+1)
- **Ambient/BassHeavy**: Falls into catch-all

### Available Patterns

#### Traditional/Classical
Standard Western music theory approach.

| Degree | Chance | Strength Pref | Length Pref | Octaves |
|--------|--------|---------------|-------------|---------|
| 1 Root | 127 | Any (64) | Any (64) | -1, 0, +1 |
| 5th | 100 | Strong (110) | Long (100) | -1, 0 |
| 4th | 90 | Strong (100) | Long (90) | 0 |
| 3rd | 80 | Medium (80) | Medium (64) | 0 |
| 6th | 60 | Medium (70) | Medium (50) | 0, +1 |
| 2nd | 40 | Weak (30) | Short (30) | 0 |
| 7th | 35 | Weak (20) | Short (20) | 0, +1 |

*Stable notes on strong/long beats, passing tones on weak/short beats.*

#### Jazz Melodic
Jazz harmony preferences with emphasis on extensions.

| Degree | Chance | Strength Pref | Length Pref | Octaves |
|--------|--------|---------------|-------------|---------|
| 1 Root | 127 | Any (64) | Any (64) | 0 |
| 5th | 90 | Strong (100) | Long (90) | 0 |
| 3rd | 95 | Strong (95) | Medium (70) | 0, +1 |
| 7th | 85 | Strong (90) | Long (80) | 0 |
| 6th | 70 | Medium (70) | Medium (60) | 0 |
| 2nd | 60 | Any (64) | Short (40) | 0, +1 |
| 4th | 40 | Weak (40) | Short (30) | 0 |

*Jazz loves 7ths and 3rds on strong beats; avoids 4th (bebop tradition).*

#### Ambient/Drone
Emphasis on root and fifth for sustained, meditative feel.

| Degree | Chance | Strength Pref | Length Pref | Octaves |
|--------|--------|---------------|-------------|---------|
| 1 Root | 127 | Strong (120) | Long (120) | -1, 0 |
| 5th | 110 | Strong (115) | Long (115) | -1, 0 |
| 4th | 80 | Strong (100) | Long (100) | 0 |
| 3rd | 40 | Weak (40) | Short (40) | 0 |
| 6th | 30 | Weak (30) | Short (30) | 0 |
| 2nd | 20 | Weak (20) | Short (20) | 0 |
| 7th | 20 | Weak (20) | Short (20) | 0 |

*Root and 5th dominate; other notes are sparse ornaments.*

#### Bass Heavy
Low register emphasis for driving rhythms.

| Degree | Chance | Strength Pref | Length Pref | Octaves |
|--------|--------|---------------|-------------|---------|
| 1 Root | 127 | Strong (120) | Long (110) | -1, 0 |
| 5th | 100 | Strong (110) | Long (100) | -1 |
| 4th | 80 | Strong (90) | Medium (70) | -1, 0 |
| 3rd | 60 | Any (64) | Medium (60) | 0 |
| Others | 30 | Weak (30) | Short (30) | 0 |

*Low octave variants preferred for foundational bass.*

#### Melodic/Vocal
Smooth melodic lines with upper register activity.

| Degree | Chance | Strength Pref | Length Pref | Octaves |
|--------|--------|---------------|-------------|---------|
| 1 Root | 127 | Any (64) | Any (64) | 0, +1 |
| 3rd | 100 | Strong (100) | Long (90) | 0, +1 |
| 5th | 90 | Strong (95) | Long (85) | 0, +1 |
| 6th | 80 | Medium (75) | Medium (70) | 0, +1 |
| 2nd | 70 | Any (64) | Medium (60) | 0, +1 |
| 7th | 60 | Medium (70) | Short (50) | 0, +1 |
| 4th | 50 | Weak (50) | Short (40) | 0 |

*Upper octaves for singing quality; 3rd prominent for expressiveness.*

#### Tension/Chromatic
More dissonance and harmonic tension.

| Degree | Chance | Strength Pref | Length Pref | Octaves |
|--------|--------|---------------|-------------|---------|
| 1 Root | 127 | Any (64) | Any (64) | 0 |
| 7th | 90 | Strong (90) | Long (80) | 0, +1 |
| 2nd | 85 | Strong (85) | Medium (70) | 0 |
| 6th | 80 | Medium (75) | Medium (65) | 0 |
| 3rd | 70 | Medium (70) | Medium (60) | 0 |
| 5th | 60 | Weak (50) | Short (50) | 0 |
| 4th | 50 | Weak (45) | Short (40) | 0 |

*Unstable intervals get prominence; creates tension.*

#### Even/Balanced
Equal weighting for all notes.

| Degree | Chance | Strength Pref | Length Pref | Octaves |
|--------|--------|---------------|-------------|---------|
| All | 80 | Any (64) | Any (64) | 0 |
| Root | 127 | Any (64) | Any (64) | 0 |

*Democratic distribution; chance-based only.*

#### Pentatonic Focus
Optimized for pentatonic scales (no avoid notes).

| Degree | Chance | Strength Pref | Length Pref | Octaves |
|--------|--------|---------------|-------------|---------|
| 1 Root | 127 | Any (64) | Any (64) | -1, 0, +1 |
| 5th | 100 | Strong (100) | Long (90) | -1, 0 |
| 3rd/b3 | 95 | Strong (95) | Medium (75) | 0, +1 |
| 6th/b7 | 85 | Medium (80) | Medium (65) | 0 |
| 2nd/4th | 75 | Any (64) | Any (64) | 0 |

*All pentatonic notes relatively stable; wider octave range.*

---

## Per-Note Parameters

Each note has individual parameters that can be fine-tuned after scale/pattern selection.

### Chance (0-127)

Base probability weight for the note.

| Value | Meaning |
|-------|---------|
| 0 | Disabled (never plays) |
| 1-40 | Rare (ornamental) |
| 41-80 | Moderate (supporting) |
| 81-126 | Common (prominent) |
| 127 | Maximum |

**Note:** Root note's chance defaults to 127 but is now editable. The root always serves as a fallback if no other note qualifies, regardless of its chance value.

### Beat Strength Preference (0-127)

How the note responds to beat strength from Strength page. **Editable for all notes including root.**

| Value | Name | Behavior |
|-------|------|----------|
| 0-30 | Weak | Prefers weak beats (low strength positions) |
| 31-50 | Weak-Medium | Slight weak preference |
| 51-76 | Any | Unaffected by strength (pure chance) |
| 77-100 | Medium-Strong | Slight strong preference |
| 101-127 | Strong | Prefers strong beats (high strength positions) |

**64 = Any (neutral, default)**

### Beat Length Preference (0-127)

How the note responds to note duration. **Editable for all notes including root.**

| Value | Name | Behavior |
|-------|------|----------|
| 0-30 | Short | Prefers short/quick notes |
| 31-50 | Short-Medium | Slight short preference |
| 51-76 | Any | Unaffected by length (pure chance) |
| 77-100 | Medium-Long | Slight long preference |
| 101-127 | Long | Prefers long/sustained notes |

**64 = Any (neutral, default)**

### Octave (Relative to Root)

Which octave(s) this note can appear in.

| Value | Meaning |
|-------|---------|
| -1 | One octave below root |
| 0 | Same octave as root (main) |
| +1 | One octave above root |

Notes can exist in multiple octaves with **different stability settings** per octave.

Example:
```
Root (octave 0):  Strength=Any, Length=Any      → Plays anywhere
Root (octave -1): Strength=Strong, Length=Long  → Bass on downbeats
Root (octave +1): Strength=Weak, Length=Short   → High ornament on upbeats
```

---

## Octave Variants

### Music Theory Basis

The same pitch in different octaves has different musical functions:

| Octave | Character | Musical Use |
|--------|-----------|-------------|
| **-1 (Lower)** | Grounding, bass-like, foundational | Strong beats, root/5th reinforcement, bass lines |
| **0 (Main)** | Standard melodic range, balanced | Default melodic content |
| **+1 (Upper)** | Soaring, bright, climactic | Melodic peaks, ornaments, tension |

### Per-Octave Stability

Each octave variant can have independent stability settings:

```
5th (octave 0):  Chance=100, Strength=Strong(110), Length=Long(100)
5th (octave -1): Chance=80,  Strength=Strong(120), Length=Long(120)  ← bass 5th
5th (octave +1): Chance=60,  Strength=Any(64),     Length=Short(40)  ← melodic 5th
```

This allows musically intelligent octave placement:
- Low 5th reinforces bass on strong, long beats
- Mid 5th is standard melodic
- High 5th appears on quicker passages

---

## Global Octave Randomization

A post-processing system that adds melodic contour variation to ANY selected note.

### Parameters

| Parameter | Range | Default | Function |
|-----------|-------|---------|----------|
| **Chance** | 0-127 | 0 (off) | Probability that octave shift occurs |
| **Strength Preference** | Weak/Any/Strong | Any (64) | When shift occurs based on beat strength |
| **Length Preference** | Short/Any/Long | Any (64) | When shift occurs based on note length |
| **Direction** | Down/Both/Up | Both | Which direction(s) to shift |

### How It Works

After a note is selected through normal probability:

1. **Chance Check**: Roll against Chance parameter
2. **Strength Check**: Does current beat strength match preference?
3. **Length Check**: Does current note length match preference?
4. If all pass → shift octave in specified direction

### Example Configurations

#### "Subtle Variation"
```
Chance: 20
Strength: Any (64)
Length: Any (64)
Direction: Both
```
→ 20% of all notes randomly shift ±1 octave, creating gentle melodic movement.

#### "Climactic Peaks"
```
Chance: 50
Strength: Strong (110)
Length: Long (100)
Direction: Up
```
→ Strong, long notes have 50% chance to jump UP an octave = exciting peaks.

#### "Bass Drops"
```
Chance: 40
Strength: Strong (115)
Length: Long (110)
Direction: Down
```
→ Strong, sustained notes occasionally drop an octave for bass weight.

#### "Quick Upper Register"
```
Chance: 35
Strength: Weak (30)
Length: Short (30)
Direction: Up
```
→ Weak, quick notes sometimes pop up an octave = sparkling ornaments.

#### "Wide Range Melody"
```
Chance: 30
Strength: Any (64)
Length: Any (64)
Direction: Both
```
→ Melodic line spans wider range with organic octave jumps.

---

## Processing Flow

Complete algorithm for note selection:

```
┌─────────────────────────────────────────────────────────────────┐
│ STEP 1: NOTE POOL SETUP (choose one path)                        │
│                                                                  │
│ Path A: Scale + Pattern                                          │
│   Select scale (e.g., "Dorian") → enables notes with chances    │
│   Select pattern (e.g., "Jazz Melodic") → sets strength/length  │
│                                                                  │
│ Path B: ML Suggestion                                            │
│   Click "Suggest" on Notes page (or "Both" on Beats page)       │
│   → Populates notes from MIDI-learned interval distributions    │
│   → Sets chances, strength biases, and length biases per note   │
│   → Scale/Pattern dropdowns switch to "Custom"                  │
│                                                                  │
│ Either path produces a NotePool with per-note chance,            │
│ strength bias, and length bias. Both can be fine-tuned after.    │
└─────────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────┐
│ STEP 2: USER FINE-TUNING (optional)                              │
│                                                                  │
│ User can manually adjust any per-note parameter                  │
│ → Override chance, strength pref, length pref                    │
│ → Enable/disable specific octave variants                        │
└─────────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────┐
│ STEP 4: CONFIGURE GLOBAL OCTAVE RANDOMIZATION                    │
│                                                                  │
│ User sets post-processing octave shift behavior                  │
│ → Chance, Strength pref, Length pref, Direction                  │
└─────────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────┐
│ STEP 4b: CONFIGURE STYLE PATTERNS (optional)                     │
│                                                                  │
│ User selects a musical style and configures:                     │
│ → Style (Classical, Blues, Jazz, Rock, etc.)                     │
│ → Chance (0-127): probability of pattern triggering per beat     │
│ → Complexity (1-20): limits available patterns (simple→complex)  │
│ → Max Notes (1-10): max consecutive beats covered (loops)        │
│ → Mode (Replace/Finish): pattern interruption behavior           │
└─────────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────┐
│ STEP 5: SEQUENCER RUNTIME (per beat)                             │
│                                                                  │
│ For each beat position:                                          │
│                                                                  │
│ a. Read beat strength from Strength page (0-100)                 │
│ b. Determine note length for this beat                           │
│                                                                  │
│ c. For each note (including ROOT and octave variants):           │
│    weight = base_chance                                          │
│           × calculate_strength_match(beat_strength, note_pref)   │
│           × calculate_length_match(note_length, note_pref)       │
│                                                                  │
│    ROOT NOTE participates with:                                  │
│    - base_chance = 127 (fixed, maximum)                          │
│    - strength_pref = configurable (can prefer strong/weak)       │
│    - length_pref = configurable (can prefer short/long)          │
│                                                                  │
│ d. Normalize weights to probabilities                            │
│ e. Weighted random selection (root competes with other notes)    │
│ f. Fallback: If weighted_notes empty → Root frequency used       │
│    (rare edge case, root is normally in the selection pool)      │
└─────────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────┐
│ STEP 6: GLOBAL OCTAVE RANDOMIZATION (post-processing)            │
│                                                                  │
│ Selected note: e.g., 5th (octave 0)                              │
│                                                                  │
│ a. Roll against Chance parameter                                 │
│    → If fails, keep original octave                              │
│                                                                  │
│ b. Check Strength preference vs current beat strength            │
│    → If mismatch, keep original octave                           │
│                                                                  │
│ c. Check Length preference vs current note length                │
│    → If mismatch, keep original octave                           │
│                                                                  │
│ d. If all pass → Apply octave shift (Direction parameter)        │
│    → Clamp to valid MIDI range                                   │
└─────────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────┐
│ STEP 7: STYLE PATTERN APPLICATION (post-processing)              │
│                                                                  │
│ After all beat events are generated for a bar:                   │
│                                                                  │
│ a. Sort events by time                                           │
│ b. Walk through events in order                                  │
│ c. If an active pattern exists, apply its next pitch             │
│ d. Roll against Style Chance for a new pattern trigger           │
│    → Replace mode: rolls every beat (may interrupt active)       │
│    → Finish mode: only rolls when no pattern is active           │
│    → If triggered, select random pattern (limited by Complexity) │
│    → Build pitch sequence from enabled notes + start pitch       │
│    → Override pitches of this and subsequent consecutive beats   │
│ e. Max Notes limits pattern length (loops if exceeded)           │
└─────────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────┐
│ STEP 8: OUTPUT                                                   │
│                                                                  │
│ Final note with:                                                 │
│ - Pitch (note + octave, possibly style-pattern-overridden)       │
│ - Velocity (derived from beat strength)                          │
│ - Duration (note length)                                         │
└─────────────────────────────────────────────────────────────────┘
```

---

## Probability Calculation

### Strength Match Formula

```
strength_match(beat_strength, note_pref) =
    if note_pref == 64 (Any):
        return 1.0  // No modification

    // Normalize beat_strength (0-100) to 0.0-1.0
    beat_norm = beat_strength / 100.0

    // Normalize note_pref (0-127) to -1.0 to +1.0
    // 0 = -1.0 (wants weak), 64 = 0.0 (any), 127 = +1.0 (wants strong)
    pref_norm = (note_pref - 64) / 63.0

    // Calculate match
    // If pref is positive (wants strong) and beat is strong → boost
    // If pref is positive (wants strong) and beat is weak → reduce
    // Vice versa for negative pref

    match = 1.0 + pref_norm * (beat_norm - 0.5) * 2.0
    return clamp(match, 0.1, 2.0)
```

### Length Match Formula

```
length_match(note_length, note_pref) =
    // Same formula as strength_match
    // note_length normalized from actual duration
    // note_pref: 0=short, 64=any, 127=long
```

### Combined Weight

```
final_weight = base_chance * strength_match * length_match
```

---

## Preset Storage

### Data Structure

```rust
struct NoteStabilityData {
    // Scale and pattern selection
    scale: Scale,
    stability_pattern: StabilityPattern,

    // Per-note settings (can override pattern defaults)
    notes: Vec<NoteSettings>,

    // Global octave randomization
    octave_rand_chance: u8,        // 0-127, default 0
    octave_rand_strength: u8,      // 0-127, default 64 (Any)
    octave_rand_length: u8,        // 0-127, default 64 (Any)
    octave_rand_direction: i8,     // -1=Down, 0=Both, 1=Up

    // Style patterns
    style_config: StyleConfigPresetData,
}

struct NoteSettings {
    midi_note: u8,                 // Absolute MIDI note
    octave_offset: i8,             // -1, 0, or +1 relative to root
    chance: u8,                    // 0-127
    strength_pref: u8,             // 0-127
    length_pref: u8,               // 0-127
    enabled: bool,
}

struct StyleConfigPresetData {
    style: StylePattern,           // None, Classical, Blues, Jazz, etc.
    chance: u8,                    // 0-127, default 0
    complexity: u8,                // 1-20, default 10
    max_notes: u8,                 // 1-10, default 4
}
```

### JSON Format

```json
{
    "scale": "Dorian",
    "stability_pattern": "JazzMelodic",
    "notes": [
        {"midi_note": 48, "octave_offset": 0, "chance": 127, "strength_pref": 64, "length_pref": 64, "enabled": true},
        {"midi_note": 50, "octave_offset": 0, "chance": 60, "strength_pref": 64, "length_pref": 40, "enabled": true}
    ],
    "octave_rand_chance": 20,
    "octave_rand_strength": 64,
    "octave_rand_length": 64,
    "octave_rand_direction": 0,
    "style_config": {
        "style": "Blues",
        "chance": 80,
        "complexity": 10,
        "max_notes": 4
    }
}
```

---

## Integration with Other Pages

### Strength Page Connection

The Strength page defines **which beat positions are strong/weak** (0-100).
The Note Stability page defines **which notes prefer which strength levels**.

Together they create rhythmic-melodic relationships:
- Strong beats → Stable notes (root, 5th)
- Weak beats → Passing tones (2nd, 7th)

### Length Page Connection

The Length page (or note length parameter) defines **how long notes sustain**.
The Note Stability page defines **which notes prefer which lengths**.

Together they create durational-melodic relationships:
- Long notes → Structural pitches (root, 5th, 3rd)
- Short notes → Ornamental pitches (2nd, 7th)

---

## UI Layout

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              NOTE STABILITY                                  │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  Scale: [Dorian ▼]  Pattern: [Jazz ▼]  Style: [Blues ▼] │ Density:[==] Var:[==] [Suggest] │
│                                                                              │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  ┌───────────────────────────────────────────────────────────────────────┐  │
│  │                          NOTE GRID                                    │  │
│  │  (Piano roll with enabled notes, click to add/remove)                 │  │
│  │                                                                        │  │
│  │  [Root] ████████████████████████████ 127  Any   Any                   │  │
│  │  [2nd]  ██████████░░░░░░░░░░░░░░░░░░  60  Weak  Short                 │  │
│  │  [b3rd] ████████████████░░░░░░░░░░░░  95  Str   Med                   │  │
│  │  [4th]  ██████░░░░░░░░░░░░░░░░░░░░░░  40  Weak  Short                 │  │
│  │  [5th]  ██████████████████████░░░░░░  90  Str   Long                  │  │
│  │  [6th]  ██████████████░░░░░░░░░░░░░░  70  Med   Med                   │  │
│  │  [b7th] ██████████████████░░░░░░░░░░  85  Str   Long                  │  │
│  └───────────────────────────────────────────────────────────────────────┘  │
│                                                                              │
├─────────────────────────────────────────────────────────────────────────────┤
│                        BOTTOM ROW (3 panels)                                 │
│                                                                              │
│  ┌─────────────────┐  ┌──────────────────────┐  ┌──────────────────────┐   │
│  │ SELECTED NOTE    │  │ OCTAVE RANDOMIZATION │  │ STYLE PATTERN        │   │
│  │                  │  │                      │  │                      │   │
│  │ Chance:  [====]  │  │ Chance:  [░░░░] 0    │  │ Chance:  [████] 80   │   │
│  │ Strength:[====]  │  │ Strength:[Wk|Any|St] │  │ Complexity:[██] 10   │   │
│  │ Length:  [====]  │  │ Length:  [Sh|Any|Lg]  │  │ Max Notes: [██] 4   │   │
│  │                  │  │ Direction:[Dn|Bo|Up]  │  │ Mode:[Replace|Finish]│   │
│  │ Effective Weight │  │                      │  │                      │   │
│  │ Weak:  [████] 80%│  │                      │  │                      │   │
│  │ Mid:   [██] 100% │  │                      │  │                      │   │
│  │ Strong:[██] 120% │  │                      │  │                      │   │
│  └─────────────────┘  └──────────────────────┘  └──────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Musical Examples

### Example 1: Jazz Ballad

```
Scale: Major
Pattern: Jazz Melodic
Octave Rand: Chance=15, Strength=Any, Length=Long, Direction=Up
```

Result: Smooth melodic lines with 7ths on strong beats, occasional upper octave jumps on sustained notes.

### Example 2: Ambient Drone

```
Scale: Pentatonic Minor
Pattern: Ambient/Drone
Octave Rand: Chance=10, Strength=Strong, Length=Long, Direction=Down
```

Result: Root and 5th dominate, mostly in lower octave, sparse other notes.

### Example 3: Funky Bass

```
Scale: Blues
Pattern: Bass Heavy
Octave Rand: Chance=25, Strength=Weak, Length=Short, Direction=Up
```

Result: Heavy bass on downbeats, quick upper octave pops on offbeats.

### Example 4: Tension Build

```
Scale: Harmonic Minor
Pattern: Tension/Chromatic
Octave Rand: Chance=40, Strength=Any, Length=Any, Direction=Both
```

Result: Dissonant, wide-ranging melody with lots of 7ths and 2nds.
