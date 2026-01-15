# Strength Page

The Strength page controls the velocity/intensity of notes at different beat positions within a bar. This affects how strongly notes are played depending on their timing.

## Strength Values (0-100)

- **0**: Weakest - notes at this position play very softly
- **50**: Medium - normal intensity
- **100**: Strongest - notes at this position play at full strength

Higher strength values result in louder, more prominent notes at that beat position.

## Grid System

The strength grid uses 96 positions per bar, which is the LCM (Least Common Multiple) of:
- 32 (for straight 1/32 notes)
- 24 (for triplet divisions)

This allows both straight and triplet timing to be represented precisely.

## Modes

### Straight Mode (S)
- Displays 32 positions per bar
- Each slider represents a 1/32 note position
- Grid lines indicate quarter notes, eighth notes, and sixteenth notes

### Triplet Mode (T)
- Displays 24 positions per bar
- Each slider represents a triplet subdivision
- Grid lines indicate triplet groupings

When in one mode, values from the other mode are shown as semi-transparent red lines.

## Strength Style Presets

The dropdown menu provides pre-configured strength patterns:

| Style | Description |
|-------|-------------|
| 4/4 Standard | Traditional 4/4 with strong downbeats |
| Backbeat | Emphasis on beats 2 and 4 (rock/pop feel) |
| Offbeat | Emphasis on off-beats (ska/reggae feel) |
| Triplet Feel | Swung triplet emphasis |
| Shuffle | Blues shuffle pattern |
| Sparse | Minimal, only main beats emphasized |
| Dense | Gradual build towards bar center |
| Polyrhythm 3:4 | Combined 3 and 4 feel |
| African | West African polyrhythmic pattern |
| Reggae | Classic reggae skank pattern |
| Latin | Son clave-inspired pattern |
| Funk | Syncopated funk groove |
| Jazz | Swing jazz feel |
| Ambient | Smooth sine-wave variation |
| Driving | Straight, energetic pattern |

## Preset Storage

Strength values are stored in presets as an array of 96 bytes (0-100 range). When loading a preset, the strength pattern is applied to the grid. When saving, the current grid values are stored in the preset.

## Integration with Sequencer

The sequencer uses strength values to modulate note velocity:
- Notes triggering at positions with high strength play louder
- Notes at low-strength positions play softer
- This creates natural-feeling rhythmic dynamics

## Swing Interaction

The strength grid visually adjusts to show swing when enabled. Swing affects the timing display but not the underlying strength values - each position maintains its assigned strength regardless of swing amount.
