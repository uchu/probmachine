#!/usr/bin/env python3
"""
Factory Preset Generator for Device Synthesizer
Generates 64 presets across 4 banks with diverse genres and styles

Note Stability System:
- Scale: Which notes are available (harmonic palette)
- Stability Pattern: How notes relate to beat strength/length
- Per-note parameters: Chance, Strength preference (beat), Length preference (beat_length)
- Octave Randomization: Post-processing melodic variation
"""

import json
from dataclasses import dataclass, field
from typing import List, Dict, Any, Optional, Tuple
import copy
import math

# =============================================================================
# SCALE DEFINITIONS
# =============================================================================

# Scale name -> (intervals from root, enum name for Rust)
SCALE_DEFINITIONS = {
    "Major": ([0, 2, 4, 5, 7, 9, 11], "Major"),
    "Minor": ([0, 2, 3, 5, 7, 8, 10], "Minor"),
    "Dorian": ([0, 2, 3, 5, 7, 9, 10], "Dorian"),
    "Phrygian": ([0, 1, 3, 5, 7, 8, 10], "Phrygian"),
    "Lydian": ([0, 2, 4, 6, 7, 9, 11], "Lydian"),
    "Mixolydian": ([0, 2, 4, 5, 7, 9, 10], "Mixolydian"),
    "Locrian": ([0, 1, 3, 5, 6, 8, 10], "Locrian"),
    "HarmonicMinor": ([0, 2, 3, 5, 7, 8, 11], "HarmonicMinor"),
    "MelodicMinor": ([0, 2, 3, 5, 7, 9, 11], "MelodicMinor"),
    "PentatonicMajor": ([0, 2, 4, 7, 9], "PentatonicMajor"),
    "PentatonicMinor": ([0, 3, 5, 7, 10], "PentatonicMinor"),
    "Blues": ([0, 3, 5, 6, 7, 10], "Blues"),
    "WholeTone": ([0, 2, 4, 6, 8, 10], "WholeTone"),
    "Chromatic": ([0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11], "Chromatic"),
    "Japanese": ([0, 1, 5, 7, 8], "Japanese"),
    "Arabic": ([0, 1, 4, 5, 7, 8, 10], "Arabic"),
    "Hungarian": ([0, 2, 3, 6, 7, 8, 11], "Hungarian"),
}

# Stability pattern name -> enum name for Rust
STABILITY_PATTERNS = {
    "Traditional": "Traditional",
    "JazzMelodic": "JazzMelodic",
    "Ambient": "Ambient",
    "BassHeavy": "BassHeavy",
    "Melodic": "Melodic",
    "Tension": "Tension",
    "Even": "Even",
    "Pentatonic": "Pentatonic",
}

# Octave direction enum values
OCTAVE_DIRECTION = {
    "Down": "Down",
    "Both": "Both",
    "Up": "Up",
}

# =============================================================================
# STABILITY PATTERN DEFINITIONS
# Per-degree settings: (strength_pref, length_pref, octaves)
# strength_pref/length_pref: 0=weak, 64=any, 127=strong/long
# =============================================================================

def get_stability_settings(pattern: str, degree: int) -> Tuple[int, int, List[int]]:
    """Get (strength_pref, length_pref, octaves) for a scale degree in a pattern"""

    if pattern == "Traditional":
        settings = {
            1: (64, 64, [-1, 0, 1]),    # Root - any
            5: (110, 100, [-1, 0]),      # Fifth - strong, long
            4: (100, 90, [0]),           # Fourth - strong
            3: (80, 64, [0]),            # Third - medium
            6: (70, 50, [0, 1]),         # Sixth - medium
            2: (30, 30, [0]),            # Second - weak, short
            7: (20, 20, [0, 1]),         # Seventh - weak, short
        }
    elif pattern == "JazzMelodic":
        settings = {
            1: (64, 64, [0]),
            5: (100, 90, [0]),
            3: (95, 70, [0, 1]),
            7: (90, 80, [0]),
            6: (70, 60, [0]),
            2: (64, 40, [0, 1]),
            4: (40, 30, [0]),
        }
    elif pattern == "Ambient":
        settings = {
            1: (120, 120, [-1, 0]),
            5: (115, 115, [-1, 0]),
            4: (100, 100, [0]),
            3: (30, 30, [0]),
            6: (30, 30, [0]),
            2: (30, 30, [0]),
            7: (30, 30, [0]),
        }
    elif pattern == "BassHeavy":
        settings = {
            1: (120, 110, [-1, 0]),
            5: (110, 100, [-1]),
            4: (90, 70, [-1, 0]),
            3: (64, 60, [0]),
            6: (30, 30, [0]),
            2: (30, 30, [0]),
            7: (30, 30, [0]),
        }
    elif pattern == "Melodic":
        settings = {
            1: (64, 64, [0, 1]),
            3: (100, 90, [0, 1]),
            5: (95, 85, [0, 1]),
            6: (75, 70, [0, 1]),
            2: (64, 60, [0, 1]),
            7: (70, 50, [0, 1]),
            4: (50, 40, [0]),
        }
    elif pattern == "Tension":
        settings = {
            1: (64, 64, [0]),
            7: (90, 80, [0, 1]),
            2: (85, 70, [0]),
            6: (75, 65, [0]),
            3: (70, 60, [0]),
            5: (50, 50, [0]),
            4: (45, 40, [0]),
        }
    elif pattern == "Even":
        settings = {i: (64, 64, [0]) for i in range(1, 8)}
    elif pattern == "Pentatonic":
        settings = {
            1: (64, 64, [-1, 0, 1]),
            5: (100, 90, [-1, 0]),
            3: (95, 75, [0, 1]),
            6: (80, 65, [0]),
            2: (80, 65, [0]),
            4: (80, 65, [0]),
            7: (80, 65, [0]),
        }
    else:
        settings = {i: (64, 64, [0]) for i in range(1, 8)}

    return settings.get(degree, (64, 64, [0]))


def get_base_chance_for_degree(scale: str, degree: int) -> int:
    """Get base chance (0-127) for a scale degree"""
    if scale in ["PentatonicMajor", "PentatonicMinor"]:
        chances = {1: 127, 2: 100, 3: 100, 4: 90, 5: 90}
        return chances.get(degree, 80)
    elif scale == "Blues":
        chances = {1: 127, 5: 100, 2: 85, 3: 85, 4: 75, 6: 70}
        return chances.get(degree, 70)
    elif scale == "Chromatic":
        return 70
    else:
        # Standard 7-note scale weighting
        chances = {
            1: 127,  # Root
            5: 100,  # Fifth
            4: 90,   # Fourth
            3: 80,   # Third
            6: 60,   # Sixth
            2: 45,   # Second
            7: 40,   # Seventh
        }
        return chances.get(degree, 50)


# =============================================================================
# NOTE DATA CLASS
# =============================================================================

@dataclass
class Note:
    midi_note: int
    chance: int
    beat: int = 64           # Strength preference: 0=weak, 64=any, 127=strong
    beat_length: int = 64    # Length preference: 0=short, 64=any, 127=long
    octave_offset: int = 0   # -1, 0, or 1 relative to root octave

def create_default_preset() -> Dict[str, Any]:
    """Create a default preset structure"""
    return {
        "straight_1_1": [0.0],
        "straight_1_2": [0.0, 0.0],
        "straight_1_4": [0.0, 0.0, 0.0, 0.0],
        "straight_1_8": [0.0] * 8,
        "straight_1_16": [0.0] * 16,
        "straight_1_32": [0.0] * 32,
        "triplet_1_2t": [0.0] * 3,
        "triplet_1_4t": [0.0] * 6,
        "triplet_1_8t": [0.0] * 12,
        "triplet_1_16t": [0.0] * 24,
        "dotted_1_2d": [0.0] * 2,
        "dotted_1_4d": [0.0] * 3,
        "dotted_1_8d": [0.0] * 6,
        "dotted_1_16d": [0.0] * 11,
        "dotted_1_32d": [0.0] * 22,
        "strength_values": [50] * 96,
        "root_note": 48,
        "notes": [],
        "scale": "Major",
        "stability_pattern": "Traditional",
        "octave_randomization": create_octave_randomization(0.0, 0.25, 0.25, "Both"),
        "synth_pll_track_speed": 0.5,
        "synth_pll_damping": 0.3,
        "synth_pll_influence": 0.5,
        "synth_pll_mult": 0,
        "synth_pll_colored": False,
        "synth_pll_mode": True,
        "synth_pll_ref_octave": 0,
        "synth_pll_ref_pulse_width": 0.5,
        "synth_pll_feedback": 0.0,
        "synth_pll_volume": 0.0,
        "synth_pll_stereo_damp_offset": 0.0,
        "synth_pll_glide": 0.0,
        "synth_pll_fm_amount": 0.0,
        "synth_pll_fm_ratio": 1,
        "synth_pll_retrigger": 0.0,
        "synth_pll_burst_threshold": 0.7,
        "synth_pll_burst_amount": 0.0,
        "synth_pll_loop_saturation": 1.0,
        "synth_pll_color_amount": 0.0,
        "synth_pll_edge_sensitivity": 0.5,
        "synth_pll_stereo_track_offset": 0.0,
        "synth_pll_stereo_phase": 0.0,
        "synth_pll_cross_feedback": 0.0,
        "synth_pll_fm_env_amount": 0.0,
        "synth_pll_enable": True,
        "synth_osc_octave": 0,
        "synth_osc_d": 0.5,
        "synth_osc_v": 0.5,
        "synth_osc_stereo_v_offset": 0.0,
        "synth_osc_volume": 0.7,
        "synth_sub_volume": 0.0,
        "synth_filter_enable": True,
        "synth_filter_cutoff": 2000.0,
        "synth_filter_resonance": 0.0,
        "synth_filter_env_amount": 0.0,
        "synth_filter_drive": 1.0,
        "synth_vol_attack": 5.0,
        "synth_vol_decay": 200.0,
        "synth_vol_sustain": 0.6,
        "synth_vol_release": 300.0,
        "synth_filt_attack": 5.0,
        "synth_filt_decay": 200.0,
        "synth_filt_sustain": 0.5,
        "synth_filt_release": 300.0,
        "synth_reverb_mix": 0.0,
        "synth_reverb_time_scale": 0.5,
        "synth_reverb_decay": 0.5,
        "synth_reverb_diffusion": 0.7,
        "synth_reverb_pre_delay": 20.0,
        "synth_reverb_mod_depth": 0.2,
        "synth_reverb_hpf": 100.0,
        "synth_reverb_lpf": 8000.0,
        "synth_reverb_ducking": 0.0,
        "synth_volume": 0.75,
        "lfo1_rate": 1.0,
        "lfo1_waveform": 0,
        "lfo1_tempo_sync": False,
        "lfo1_sync_division": 2,
        "lfo1_sync_source": -1,
        "lfo1_phase_mod": 0.0,
        "lfo1_dest1": 0,
        "lfo1_amount1": 0.0,
        "lfo1_dest2": 0,
        "lfo1_amount2": 0.0,
        "lfo2_rate": 1.0,
        "lfo2_waveform": 0,
        "lfo2_tempo_sync": False,
        "lfo2_sync_division": 2,
        "lfo2_sync_source": -1,
        "lfo2_phase_mod": 0.0,
        "lfo2_dest1": 0,
        "lfo2_amount1": 0.0,
        "lfo2_dest2": 0,
        "lfo2_amount2": 0.0,
        "lfo3_rate": 1.0,
        "lfo3_waveform": 0,
        "lfo3_tempo_sync": False,
        "lfo3_sync_division": 2,
        "lfo3_sync_source": -1,
        "lfo3_phase_mod": 0.0,
        "lfo3_dest1": 0,
        "lfo3_amount1": 0.0,
        "lfo3_dest2": 0,
        "lfo3_amount2": 0.0,
        "swing_amount": 50.0,
        "note_length_percent": 95.0,
        "len_mod_1_target": 0.0,
        "len_mod_1_amount": 100.0,
        "len_mod_1_prob": 0.0,
        "len_mod_2_target": 0.0,
        "len_mod_2_amount": 100.0,
        "len_mod_2_prob": 0.0,
        "decay_mod_1_target": 0.0,
        "decay_mod_1_amount": 100.0,
        "decay_mod_1_prob": 0.0,
        "decay_mod_2_target": 0.0,
        "decay_mod_2_amount": 100.0,
        "decay_mod_2_prob": 0.0,
        "pos_mod_1_target": 0.0,
        "pos_mod_1_shift": 0.0,
        "pos_mod_1_prob": 0.0,
        "pos_mod_2_target": 0.0,
        "pos_mod_2_shift": 0.0,
        "pos_mod_2_prob": 0.0,
        "synth_ring_mod": 0.0,
        "synth_wavefold": 0.0,
        "synth_drift_amount": 0.0,
        "synth_drift_rate": 0.5,
        "synth_noise_amount": 0.0,
        "synth_tube_drive": 1.0,
        "synth_vps_enable": True,
    }

def note_to_dict(n: Note) -> Dict:
    """Convert Note to dictionary for JSON serialization"""
    return {
        "midi_note": n.midi_note,
        "chance": n.chance,
        "beat": n.beat,
        "beat_length": n.beat_length,
        "octave_offset": n.octave_offset
    }


# =============================================================================
# OCTAVE RANDOMIZATION HELPERS
# =============================================================================

def create_octave_randomization(
    chance: float = 0,
    strength_pref: float = 0.25,
    length_pref: float = 0.25,
    direction: str = "Both"
) -> Dict:
    """Create octave randomization settings

    Args:
        chance: 0.0-1.0 probability of octave jump (converted to 0-255)
        strength_pref: 0.0-1.0 preference for stronger notes (converted to 0-255)
        length_pref: 0.0-1.0 preference for longer notes (converted to 0-255)
        direction: "up", "down", or "both"
    """
    return {
        "chance": int(chance * 255),
        "strength_pref": int(strength_pref * 255),
        "length_pref": int(length_pref * 255),
        "direction": direction
    }


# Preset octave randomization configurations (values in 0.0-1.0 range)
OCTAVE_RAND_PRESETS = {
    "off": create_octave_randomization(0.0, 0.25, 0.25, "Both"),
    "subtle": create_octave_randomization(0.08, 0.25, 0.25, "Both"),
    "climactic_peaks": create_octave_randomization(0.2, 0.43, 0.39, "Up"),
    "bass_drops": create_octave_randomization(0.16, 0.45, 0.43, "Down"),
    "quick_upper": create_octave_randomization(0.14, 0.12, 0.12, "Up"),
    "wide_range": create_octave_randomization(0.12, 0.25, 0.25, "Both"),
    "strong_up": create_octave_randomization(0.18, 0.39, 0.25, "Up"),
    "weak_down": create_octave_randomization(0.16, 0.12, 0.25, "Down"),
    "long_up": create_octave_randomization(0.14, 0.25, 0.39, "Up"),
    "short_down": create_octave_randomization(0.12, 0.25, 0.12, "Down"),
}


# =============================================================================
# SCALE-BASED NOTE GENERATION
# =============================================================================

def get_interval_degree(scale: str, interval: int) -> Optional[int]:
    """Get scale degree (1-7) for an interval, or None if not in scale"""
    if scale not in SCALE_DEFINITIONS:
        return None
    intervals = SCALE_DEFINITIONS[scale][0]
    if interval in intervals:
        return intervals.index(interval) + 1
    return None


def generate_scale_notes(
    root_note: int,
    scale: str,
    pattern: str = "Traditional",
    include_octave_variants: bool = True,
    chance_multiplier: float = 1.0,
    root_strength_pref: int = 64,
    root_length_pref: int = 64
) -> List[Note]:
    """
    Generate notes for a scale with stability pattern settings.

    Args:
        root_note: MIDI note number for root (e.g., 48 for C3)
        scale: Scale name from SCALE_DEFINITIONS
        pattern: Stability pattern from STABILITY_PATTERNS
        include_octave_variants: Whether to include -1/+1 octave variants
        chance_multiplier: Multiply all chances by this (for sparse/dense)
        root_strength_pref: Strength preference for root note (0-127)
        root_length_pref: Length preference for root note (0-127)

    Returns:
        List of Note objects including root with configurable biases
    """
    if scale not in SCALE_DEFINITIONS:
        # Fallback to root only
        return [Note(root_note, 127, root_strength_pref, root_length_pref, 0)]

    intervals, _ = SCALE_DEFINITIONS[scale]
    root_pitch_class = root_note % 12
    root_octave = root_note // 12

    notes = []

    for interval in intervals:
        degree = get_interval_degree(scale, interval)
        if degree is None:
            continue

        midi_note = root_octave * 12 + root_pitch_class + interval

        # Clamp to valid MIDI range
        if midi_note < 0 or midi_note > 127:
            continue

        # Get base chance
        base_chance = get_base_chance_for_degree(scale, degree)

        # Get stability settings
        strength_pref, length_pref, octaves = get_stability_settings(pattern, degree)

        # Apply chance multiplier
        final_chance = int(min(127, base_chance * chance_multiplier))

        # For root note, use custom biases; otherwise use pattern defaults
        if interval == 0:
            # Root note - use provided biases, chance always 127
            notes.append(Note(midi_note, 127, root_strength_pref, root_length_pref, 0))
        else:
            # Add main octave variant
            if 0 in octaves or not include_octave_variants:
                notes.append(Note(midi_note, final_chance, strength_pref, length_pref, 0))

            # Add octave variants if enabled
            if include_octave_variants:
                if -1 in octaves:
                    lower_note = midi_note - 12
                    if 0 <= lower_note <= 127:
                        # Lower octave often has stronger/longer preference
                        lower_chance = int(final_chance * 0.7)
                        notes.append(Note(midi_note, lower_chance, min(127, strength_pref + 10), min(127, length_pref + 10), -1))

                if 1 in octaves:
                    higher_note = midi_note + 12
                    if 0 <= higher_note <= 127:
                        # Higher octave often has weaker/shorter preference
                        higher_chance = int(final_chance * 0.6)
                        notes.append(Note(midi_note, higher_chance, max(0, strength_pref - 10), max(0, length_pref - 10), 1))

    return notes


def generate_simple_scale_notes(
    root_note: int,
    scale: str,
    chances: Optional[Dict[int, int]] = None,
    default_chance: int = 60
) -> List[Note]:
    """
    Generate simple scale notes without stability pattern complexity.
    Useful for quick preset creation with custom chance values.

    Args:
        root_note: MIDI note for root
        scale: Scale name
        chances: Dict mapping interval (0-11) to chance (0-127)
        default_chance: Default chance for notes not in chances dict
    """
    if scale not in SCALE_DEFINITIONS:
        return [Note(root_note, 127)]

    intervals, _ = SCALE_DEFINITIONS[scale]
    root_pitch_class = root_note % 12
    root_octave = root_note // 12

    notes = []

    for interval in intervals:
        midi_note = root_octave * 12 + root_pitch_class + interval
        if midi_note < 0 or midi_note > 127:
            continue

        if interval == 0:
            # Root always 127
            notes.append(Note(midi_note, 127))
        else:
            chance = chances.get(interval, default_chance) if chances else default_chance
            notes.append(Note(midi_note, chance))

    return notes


def create_custom_notes(
    root_note: int,
    note_specs: List[Tuple[int, int, int, int]]
) -> List[Note]:
    """
    Create custom notes from specifications.

    Args:
        root_note: MIDI note for root (automatically added with chance=127)
        note_specs: List of (interval_from_root, chance, strength_pref, length_pref)

    Example:
        create_custom_notes(48, [
            (7, 80, 100, 90),   # Fifth, high chance, prefer strong/long
            (5, 60, 64, 64),    # Fourth, medium chance, any
            (3, 40, 30, 30),    # Third, lower chance, prefer weak/short
        ])
    """
    notes = [Note(root_note, 127)]  # Root with default biases

    for interval, chance, strength, length in note_specs:
        midi_note = root_note + interval
        if 0 <= midi_note <= 127:
            notes.append(Note(midi_note, chance, strength, length, 0))

    return notes


def create_musical_preset(
    name: str,
    root_note: int = 48,
    scale: str = "Major",
    stability_pattern: str = "Traditional",
    octave_randomization: str = "off",
    root_strength_pref: int = 64,
    root_length_pref: int = 64,
    note_chance_multiplier: float = 1.0,
    include_octave_variants: bool = True,
    strength_pattern: str = "4_4_standard",
    **synth_overrides
) -> Dict[str, Any]:
    """
    Create a complete musical preset with Note Stability settings.

    This is the main helper for creating musically intelligent presets that use
    the full Note Stability system (scale, stability pattern, per-note biases).

    Args:
        name: Preset name
        root_note: MIDI note for root (default 48 = C3)
        scale: Scale from SCALE_DEFINITIONS (Major, Minor, Dorian, etc.)
        stability_pattern: Pattern from STABILITY_PATTERNS (Traditional, JazzMelodic, etc.)
        octave_randomization: Preset name from OCTAVE_RAND_PRESETS or custom dict
        root_strength_pref: Root note's strength preference (0=weak, 64=any, 127=strong)
        root_length_pref: Root note's length preference (0=short, 64=any, 127=long)
        note_chance_multiplier: Scale all note chances (0.5=sparse, 1.0=normal, 1.5=dense)
        include_octave_variants: Whether to include -1/+1 octave variants from pattern
        strength_pattern: Beat strength pattern type
        **synth_overrides: Override any synth parameter

    Returns:
        Complete preset dictionary ready for JSON export

    Example:
        # Ambient pad in D minor with gentle bass emphasis
        preset = create_musical_preset(
            name="Ambient Drift",
            root_note=50,  # D3
            scale="Minor",
            stability_pattern="Ambient",
            octave_randomization="subtle",
            root_strength_pref=100,  # Root prefers strong beats
            root_length_pref=110,    # Root prefers long notes
            synth_filter_cutoff=1200.0,
            synth_reverb_mix=0.4,
        )
    """
    preset = create_default_preset()
    preset["name"] = name
    preset["root_note"] = root_note

    # Set scale and stability pattern
    preset["scale"] = scale
    preset["stability_pattern"] = stability_pattern

    # Set octave randomization
    if isinstance(octave_randomization, str):
        preset["octave_randomization"] = OCTAVE_RAND_PRESETS.get(
            octave_randomization,
            OCTAVE_RAND_PRESETS["off"]
        )
    else:
        preset["octave_randomization"] = octave_randomization

    # Generate notes using the stability system
    notes = generate_scale_notes(
        root_note=root_note,
        scale=scale,
        pattern=stability_pattern,
        include_octave_variants=include_octave_variants,
        chance_multiplier=note_chance_multiplier,
        root_strength_pref=root_strength_pref,
        root_length_pref=root_length_pref
    )
    preset["notes"] = [note_to_dict(n) for n in notes]

    # Set strength pattern
    preset["strength_values"] = create_strength_pattern(strength_pattern)

    # Apply synth overrides
    for key, value in synth_overrides.items():
        if key in preset:
            preset[key] = value

    return preset


# Musical preset style presets - quick-start configurations
MUSICAL_STYLES = {
    "ambient_pad": {
        "scale": "Major",
        "stability_pattern": "Ambient",
        "octave_randomization": "subtle",
        "root_strength_pref": 100,
        "root_length_pref": 110,
        "strength_pattern": "sparse",
    },
    "bass_driven": {
        "scale": "Minor",
        "stability_pattern": "BassHeavy",
        "octave_randomization": "bass_drops",
        "root_strength_pref": 120,
        "root_length_pref": 100,
        "strength_pattern": "4_4_standard",
    },
    "jazz_melody": {
        "scale": "Dorian",
        "stability_pattern": "JazzMelodic",
        "octave_randomization": "wide_range",
        "root_strength_pref": 64,
        "root_length_pref": 64,
        "strength_pattern": "shuffle",
    },
    "tension_builder": {
        "scale": "HarmonicMinor",
        "stability_pattern": "Tension",
        "octave_randomization": "climactic_peaks",
        "root_strength_pref": 30,
        "root_length_pref": 30,
        "strength_pattern": "offbeat",
    },
    "pentatonic_lead": {
        "scale": "PentatonicMinor",
        "stability_pattern": "Pentatonic",
        "octave_randomization": "strong_up",
        "root_strength_pref": 64,
        "root_length_pref": 90,
        "strength_pattern": "backbeat",
    },
    "ethereal": {
        "scale": "Lydian",
        "stability_pattern": "Melodic",
        "octave_randomization": "long_up",
        "root_strength_pref": 80,
        "root_length_pref": 120,
        "strength_pattern": "triplet_feel",
    },
    "world_fusion": {
        "scale": "Arabic",
        "stability_pattern": "Traditional",
        "octave_randomization": "subtle",
        "root_strength_pref": 90,
        "root_length_pref": 80,
        "strength_pattern": "4_4_standard",
    },
    "dark_ambient": {
        "scale": "Phrygian",
        "stability_pattern": "Ambient",
        "octave_randomization": "bass_drops",
        "root_strength_pref": 110,
        "root_length_pref": 120,
        "strength_pattern": "sparse",
    },
}


def create_styled_preset(name: str, style: str, root_note: int = 48, **overrides) -> Dict[str, Any]:
    """
    Create a preset using a predefined musical style.

    Args:
        name: Preset name
        style: Style name from MUSICAL_STYLES
        root_note: MIDI note for root
        **overrides: Override any style or synth parameter

    Example:
        preset = create_styled_preset("Dark Cave", "dark_ambient", root_note=43)
    """
    if style not in MUSICAL_STYLES:
        raise ValueError(f"Unknown style: {style}. Available: {list(MUSICAL_STYLES.keys())}")

    style_settings = copy.deepcopy(MUSICAL_STYLES[style])
    style_settings.update(overrides)

    return create_musical_preset(name=name, root_note=root_note, **style_settings)


def euclidean_rhythm(steps: int, pulses: int) -> List[int]:
    """Generate euclidean rhythm pattern as list of step indices"""
    if pulses >= steps:
        return list(range(steps))
    if pulses == 0:
        return []
    pattern = []
    bucket = 0
    for i in range(steps):
        bucket += pulses
        if bucket >= steps:
            bucket -= steps
            pattern.append(i)
    return pattern

def create_strength_pattern(pattern_type: str) -> List[int]:
    """Create 96-position strength pattern for different feels"""
    s = [40] * 96

    if pattern_type == "4_4_standard":
        for i in range(96):
            beat_in_bar = i % 24
            if beat_in_bar == 0: s[i] = 100
            elif beat_in_bar == 12: s[i] = 80
            elif beat_in_bar == 6 or beat_in_bar == 18: s[i] = 65
            elif beat_in_bar % 3 == 0: s[i] = 55
            else: s[i] = 45

    elif pattern_type == "backbeat":
        for i in range(96):
            beat_in_bar = i % 24
            if beat_in_bar == 6 or beat_in_bar == 18: s[i] = 100
            elif beat_in_bar == 0 or beat_in_bar == 12: s[i] = 70
            else: s[i] = 40

    elif pattern_type == "offbeat":
        for i in range(96):
            beat_in_bar = i % 24
            if beat_in_bar == 3 or beat_in_bar == 9 or beat_in_bar == 15 or beat_in_bar == 21: s[i] = 100
            elif beat_in_bar == 0: s[i] = 60
            else: s[i] = 35

    elif pattern_type == "triplet_feel":
        for i in range(96):
            pos = i % 24
            if pos % 8 == 0: s[i] = 100
            elif pos % 8 == 4: s[i] = 70
            else: s[i] = 45

    elif pattern_type == "shuffle":
        for i in range(96):
            beat_in_bar = i % 24
            if beat_in_bar == 0: s[i] = 100
            elif beat_in_bar == 12: s[i] = 85
            elif beat_in_bar == 4 or beat_in_bar == 16: s[i] = 75
            elif beat_in_bar == 8 or beat_in_bar == 20: s[i] = 60
            else: s[i] = 40

    elif pattern_type == "sparse":
        for i in range(96):
            if i % 24 == 0: s[i] = 100
            elif i % 12 == 0: s[i] = 60
            else: s[i] = 30

    elif pattern_type == "dense":
        for i in range(96):
            beat = i % 24
            s[i] = 50 + int(30 * (1 - abs(beat - 12) / 12))
            if beat == 0: s[i] = 100

    elif pattern_type == "polyrhythm_3_4":
        for i in range(96):
            pos3 = i % 32
            pos4 = i % 24
            if pos4 == 0: s[i] = 100
            elif pos3 == 0: s[i] = 90
            elif pos4 == 12: s[i] = 75
            elif pos3 == 16: s[i] = 65
            else: s[i] = 40

    elif pattern_type == "african":
        for i in range(96):
            beat = i % 24
            if beat == 0: s[i] = 100
            elif beat == 7: s[i] = 90
            elif beat == 14: s[i] = 85
            elif beat == 21: s[i] = 70
            elif beat == 3 or beat == 10 or beat == 17: s[i] = 60
            else: s[i] = 35

    elif pattern_type == "reggae":
        for i in range(96):
            beat = i % 24
            if beat == 6 or beat == 18: s[i] = 100
            elif beat == 3 or beat == 9 or beat == 15 or beat == 21: s[i] = 80
            elif beat == 0 or beat == 12: s[i] = 50
            else: s[i] = 30

    elif pattern_type == "latin":
        clave = [0, 7, 12, 14, 19]
        for i in range(96):
            beat = i % 24
            if beat in clave: s[i] = 100 if beat == 0 else 85
            elif beat == 3 or beat == 10 or beat == 17: s[i] = 60
            else: s[i] = 35

    elif pattern_type == "funk":
        for i in range(96):
            beat = i % 24
            if beat == 0: s[i] = 100
            elif beat == 10: s[i] = 95
            elif beat == 6 or beat == 18: s[i] = 80
            elif beat == 3 or beat == 15 or beat == 21: s[i] = 70
            else: s[i] = 40

    elif pattern_type == "jazz":
        for i in range(96):
            beat = i % 24
            if beat == 0: s[i] = 90
            elif beat == 8 or beat == 16: s[i] = 80
            elif beat == 4 or beat == 12 or beat == 20: s[i] = 70
            else: s[i] = 50

    elif pattern_type == "ambient":
        import math
        for i in range(96):
            s[i] = int(40 + 30 * math.sin(i * math.pi / 24))

    elif pattern_type == "driving":
        for i in range(96):
            beat = i % 24
            if beat % 6 == 0: s[i] = 100
            elif beat % 3 == 0: s[i] = 75
            else: s[i] = 55

    return s

# Scales (MIDI intervals from root)
SCALES = {
    "minor_pent": [0, 3, 5, 7, 10],
    "major_pent": [0, 2, 4, 7, 9],
    "dorian": [0, 2, 3, 5, 7, 9, 10],
    "mixolydian": [0, 2, 4, 5, 7, 9, 10],
    "phrygian": [0, 1, 3, 5, 7, 8, 10],
    "harmonic_minor": [0, 2, 3, 5, 7, 8, 11],
    "blues": [0, 3, 5, 6, 7, 10],
    "whole_tone": [0, 2, 4, 6, 8, 10],
    "japanese": [0, 1, 5, 7, 8],
    "arabic": [0, 1, 4, 5, 7, 8, 11],
    "hungarian": [0, 2, 3, 6, 7, 8, 11],
    "african": [0, 2, 4, 5, 7, 9, 11],
    "chromatic": [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11],
}

def create_preset(name: str, author: str, description: str, data: Dict) -> Dict:
    return {
        "name": name,
        "author": author,
        "description": description,
        "data": data
    }

def create_bank_a() -> List[Dict]:
    """Bank A: World Rhythms & Ethnic - 16 presets"""
    presets = []

    # 1. Sahel Crossing - West African polyrhythm
    d = create_default_preset()
    d["straight_1_8"] = [100.0, 0.0, 75.0, 0.0, 85.0, 0.0, 70.0, 0.0]
    d["straight_1_16"] = [0.0, 60.0, 0.0, 50.0, 0.0, 65.0, 0.0, 45.0, 0.0, 55.0, 0.0, 70.0, 0.0, 40.0, 0.0, 60.0]
    d["triplet_1_8t"] = [80.0, 0.0, 50.0, 70.0, 0.0, 55.0, 75.0, 0.0, 45.0, 65.0, 0.0, 60.0]
    d["strength_values"] = create_strength_pattern("african")
    d["root_note"] = 48
    d["notes"] = [note_to_dict(Note(48, 100)), note_to_dict(Note(55, 60)), note_to_dict(Note(52, 45)),
                  note_to_dict(Note(60, 35)), note_to_dict(Note(50, 30))]
    d["synth_osc_d"] = 0.35
    d["synth_osc_v"] = 0.55
    d["synth_osc_volume"] = 0.75
    d["synth_sub_volume"] = 0.25
    d["synth_filter_cutoff"] = 3500.0
    d["synth_filter_resonance"] = 0.15
    d["synth_filter_env_amount"] = 1500.0
    d["synth_vol_attack"] = 3.0
    d["synth_vol_decay"] = 250.0
    d["synth_vol_sustain"] = 0.5
    d["synth_vol_release"] = 350.0
    d["swing_amount"] = 54.0
    d["note_length_percent"] = 75.0
    d["decay_mod_1_target"] = 0.8
    d["decay_mod_1_amount"] = 130.0
    d["decay_mod_1_prob"] = 0.4
    d["pos_mod_1_target"] = -0.5
    d["pos_mod_1_shift"] = 0.02
    d["pos_mod_1_prob"] = 0.25
    presets.append(create_preset("Sahel Crossing", "Factory",
        "West African polyrhythm - interlocking patterns over dusty bass", d))

    # 2. Marrakech Night - Oriental/Arabic feel
    d = create_default_preset()
    d["straight_1_8"] = [100.0, 40.0, 70.0, 35.0, 85.0, 45.0, 60.0, 50.0]
    d["straight_1_16"] = [0.0, 55.0, 40.0, 0.0, 0.0, 60.0, 35.0, 0.0, 0.0, 50.0, 45.0, 0.0, 0.0, 55.0, 30.0, 0.0]
    d["triplet_1_8t"] = [0.0, 45.0, 35.0, 0.0, 40.0, 30.0, 0.0, 50.0, 40.0, 0.0, 35.0, 45.0]
    d["strength_values"] = create_strength_pattern("arabic") if "arabic" in dir() else create_strength_pattern("4_4_standard")
    d["root_note"] = 50
    d["notes"] = [note_to_dict(Note(50, 100)), note_to_dict(Note(51, 55)), note_to_dict(Note(54, 50)),
                  note_to_dict(Note(57, 45)), note_to_dict(Note(58, 35)), note_to_dict(Note(61, 30))]
    d["synth_osc_d"] = 0.45
    d["synth_osc_v"] = 0.6
    d["synth_osc_stereo_v_offset"] = 0.08
    d["synth_osc_volume"] = 0.7
    d["synth_filter_cutoff"] = 2800.0
    d["synth_filter_resonance"] = 0.25
    d["synth_filter_env_amount"] = 2000.0
    d["synth_filt_decay"] = 350.0
    d["synth_vol_attack"] = 5.0
    d["synth_vol_decay"] = 400.0
    d["synth_vol_sustain"] = 0.45
    d["synth_reverb_mix"] = 0.18
    d["synth_reverb_decay"] = 0.55
    d["swing_amount"] = 52.0
    d["note_length_percent"] = 85.0
    d["lfo1_tempo_sync"] = True
    d["lfo1_sync_division"] = 1
    d["lfo1_waveform"] = 1
    d["lfo1_dest1"] = 12
    d["lfo1_amount1"] = 0.15
    presets.append(create_preset("Marrakech Night", "Factory",
        "Arabic maqam wanderings under desert stars - ornamental triplets weave through", d))

    # 3. Kingston Dub - Reggae skank
    d = create_default_preset()
    d["straight_1_8"] = [30.0, 100.0, 30.0, 100.0, 30.0, 100.0, 30.0, 100.0]
    d["straight_1_16"] = [0.0, 0.0, 45.0, 0.0, 0.0, 0.0, 50.0, 0.0, 0.0, 0.0, 40.0, 0.0, 0.0, 0.0, 55.0, 0.0]
    d["strength_values"] = create_strength_pattern("reggae")
    d["root_note"] = 43
    d["notes"] = [note_to_dict(Note(43, 100)), note_to_dict(Note(50, 50)), note_to_dict(Note(47, 40)),
                  note_to_dict(Note(55, 30))]
    d["synth_osc_d"] = 0.3
    d["synth_osc_v"] = 0.4
    d["synth_osc_volume"] = 0.65
    d["synth_sub_volume"] = 0.35
    d["synth_filter_cutoff"] = 1800.0
    d["synth_filter_resonance"] = 0.2
    d["synth_filter_env_amount"] = 800.0
    d["synth_vol_attack"] = 2.0
    d["synth_vol_decay"] = 150.0
    d["synth_vol_sustain"] = 0.3
    d["synth_vol_release"] = 200.0
    d["synth_reverb_mix"] = 0.22
    d["synth_reverb_decay"] = 0.6
    d["synth_reverb_pre_delay"] = 45.0
    d["swing_amount"] = 50.0
    d["note_length_percent"] = 45.0
    d["len_mod_1_target"] = 0.9
    d["len_mod_1_amount"] = 80.0
    d["len_mod_1_prob"] = 0.6
    presets.append(create_preset("Kingston Dub", "Factory",
        "Classic reggae skank - offbeat chops with deep sub pressure", d))

    # 4. Rio Carnival - Samba groove
    d = create_default_preset()
    d["straight_1_16"] = [100.0, 60.0, 75.0, 55.0, 90.0, 50.0, 70.0, 65.0, 85.0, 55.0, 80.0, 45.0, 95.0, 60.0, 65.0, 70.0]
    d["triplet_1_8t"] = [70.0, 45.0, 55.0, 65.0, 40.0, 50.0, 75.0, 50.0, 45.0, 60.0, 55.0, 40.0]
    d["strength_values"] = create_strength_pattern("latin")
    d["root_note"] = 52
    d["notes"] = [note_to_dict(Note(52, 100)), note_to_dict(Note(55, 55)), note_to_dict(Note(59, 50)),
                  note_to_dict(Note(57, 45)), note_to_dict(Note(64, 35))]
    d["synth_osc_d"] = 0.5
    d["synth_osc_v"] = 0.65
    d["synth_osc_volume"] = 0.72
    d["synth_filter_cutoff"] = 4500.0
    d["synth_filter_resonance"] = 0.1
    d["synth_filter_env_amount"] = 1200.0
    d["synth_vol_attack"] = 2.0
    d["synth_vol_decay"] = 120.0
    d["synth_vol_sustain"] = 0.4
    d["synth_vol_release"] = 180.0
    d["swing_amount"] = 56.0
    d["note_length_percent"] = 60.0
    d["pos_mod_1_target"] = -0.6
    d["pos_mod_1_shift"] = 0.015
    d["pos_mod_1_prob"] = 0.35
    d["pos_mod_2_target"] = 0.7
    d["pos_mod_2_shift"] = -0.01
    d["pos_mod_2_prob"] = 0.25
    presets.append(create_preset("Rio Carnival", "Factory",
        "Fast samba patterns - syncopated 16ths dance with triplet cross-rhythms", d))

    # 5. Bali Temple - Gamelan-inspired
    d = create_default_preset()
    d["straight_1_8"] = [100.0, 0.0, 60.0, 0.0, 80.0, 0.0, 55.0, 0.0]
    d["straight_1_16"] = [0.0, 70.0, 0.0, 50.0, 0.0, 65.0, 0.0, 45.0, 0.0, 75.0, 0.0, 40.0, 0.0, 60.0, 0.0, 55.0]
    euc = euclidean_rhythm(16, 7)
    for i in euc:
        d["straight_1_16"][i] = max(d["straight_1_16"][i], 80.0)
    d["strength_values"] = create_strength_pattern("polyrhythm_3_4")
    d["root_note"] = 60
    d["notes"] = [note_to_dict(Note(60, 100)), note_to_dict(Note(62, 65)), note_to_dict(Note(65, 55)),
                  note_to_dict(Note(67, 50)), note_to_dict(Note(72, 40))]
    d["synth_osc_d"] = 0.6
    d["synth_osc_v"] = 0.75
    d["synth_osc_stereo_v_offset"] = 0.12
    d["synth_osc_volume"] = 0.68
    d["synth_filter_cutoff"] = 5500.0
    d["synth_filter_resonance"] = 0.22
    d["synth_filter_env_amount"] = 2500.0
    d["synth_vol_attack"] = 1.0
    d["synth_vol_decay"] = 450.0
    d["synth_vol_sustain"] = 0.25
    d["synth_vol_release"] = 600.0
    d["synth_reverb_mix"] = 0.25
    d["synth_reverb_decay"] = 0.7
    d["synth_reverb_diffusion"] = 0.85
    d["note_length_percent"] = 120.0
    d["decay_mod_1_target"] = 0.7
    d["decay_mod_1_amount"] = 150.0
    d["decay_mod_1_prob"] = 0.5
    presets.append(create_preset("Bali Temple", "Factory",
        "Gamelan-inspired interlocking bells - shimmering metallic tones cascade", d))

    # 6. Flamenco Palmas - Spanish hand-clap rhythm
    d = create_default_preset()
    d["straight_1_8"] = [100.0, 0.0, 0.0, 85.0, 0.0, 90.0, 0.0, 75.0]
    d["triplet_1_4t"] = [70.0, 50.0, 60.0, 65.0, 45.0, 55.0]
    d["triplet_1_8t"] = [0.0, 40.0, 35.0, 0.0, 45.0, 30.0, 0.0, 50.0, 40.0, 0.0, 35.0, 45.0]
    d["strength_values"] = create_strength_pattern("triplet_feel")
    d["root_note"] = 52
    d["notes"] = [note_to_dict(Note(52, 100)), note_to_dict(Note(53, 50)), note_to_dict(Note(55, 55)),
                  note_to_dict(Note(59, 40)), note_to_dict(Note(60, 35))]
    d["synth_osc_d"] = 0.55
    d["synth_osc_v"] = 0.5
    d["synth_osc_volume"] = 0.7
    d["synth_filter_cutoff"] = 3200.0
    d["synth_filter_resonance"] = 0.18
    d["synth_filter_env_amount"] = 1800.0
    d["synth_vol_attack"] = 1.0
    d["synth_vol_decay"] = 180.0
    d["synth_vol_sustain"] = 0.35
    d["synth_vol_release"] = 250.0
    d["swing_amount"] = 58.0
    d["note_length_percent"] = 55.0
    d["len_mod_1_target"] = -0.7
    d["len_mod_1_amount"] = 70.0
    d["len_mod_1_prob"] = 0.4
    presets.append(create_preset("Flamenco Palmas", "Factory",
        "Spanish fire - triplet ornaments punctuate driving compas rhythm", d))

    # 7. Kora Dreams - West African harp
    d = create_default_preset()
    d["straight_1_8"] = [90.0, 55.0, 70.0, 45.0, 80.0, 50.0, 65.0, 60.0]
    d["straight_1_16"] = [0.0, 40.0, 35.0, 0.0, 0.0, 45.0, 30.0, 0.0, 0.0, 50.0, 40.0, 0.0, 0.0, 35.0, 45.0, 0.0]
    d["strength_values"] = create_strength_pattern("african")
    d["root_note"] = 55
    d["notes"] = [note_to_dict(Note(55, 100)), note_to_dict(Note(57, 55)), note_to_dict(Note(59, 60)),
                  note_to_dict(Note(62, 50)), note_to_dict(Note(64, 45)), note_to_dict(Note(67, 35))]
    d["synth_osc_d"] = 0.25
    d["synth_osc_v"] = 0.7
    d["synth_osc_stereo_v_offset"] = 0.1
    d["synth_osc_volume"] = 0.68
    d["synth_filter_cutoff"] = 4000.0
    d["synth_filter_resonance"] = 0.12
    d["synth_filter_env_amount"] = 1500.0
    d["synth_vol_attack"] = 2.0
    d["synth_vol_decay"] = 350.0
    d["synth_vol_sustain"] = 0.4
    d["synth_vol_release"] = 500.0
    d["synth_reverb_mix"] = 0.15
    d["synth_reverb_decay"] = 0.5
    d["swing_amount"] = 53.0
    d["note_length_percent"] = 90.0
    d["lfo1_tempo_sync"] = True
    d["lfo1_sync_division"] = 0
    d["lfo1_waveform"] = 1
    d["lfo1_dest1"] = 11
    d["lfo1_amount1"] = 0.08
    presets.append(create_preset("Kora Dreams", "Factory",
        "Gentle West African melody - cascading pentatonic lines float over steady pulse", d))

    # 8. Balkan Fire - Eastern European odd meter feel
    d = create_default_preset()
    # 7/8 feel via euclidean
    euc7 = euclidean_rhythm(8, 5)
    for i, val in enumerate(d["straight_1_8"]):
        d["straight_1_8"][i] = 95.0 if i in euc7 else 30.0
    d["straight_1_16"] = [0.0, 55.0, 0.0, 45.0, 0.0, 50.0, 0.0, 0.0, 0.0, 60.0, 0.0, 40.0, 0.0, 55.0, 0.0, 50.0]
    d["triplet_1_8t"] = [50.0, 0.0, 40.0, 45.0, 0.0, 35.0, 55.0, 0.0, 45.0, 40.0, 0.0, 50.0]
    d["strength_values"] = create_strength_pattern("driving")
    d["root_note"] = 50
    d["notes"] = [note_to_dict(Note(50, 100)), note_to_dict(Note(51, 45)), note_to_dict(Note(53, 55)),
                  note_to_dict(Note(55, 50)), note_to_dict(Note(58, 40)), note_to_dict(Note(62, 30))]
    d["synth_osc_d"] = 0.48
    d["synth_osc_v"] = 0.55
    d["synth_osc_volume"] = 0.72
    d["synth_filter_cutoff"] = 3000.0
    d["synth_filter_resonance"] = 0.2
    d["synth_filter_env_amount"] = 1600.0
    d["synth_vol_attack"] = 2.0
    d["synth_vol_decay"] = 200.0
    d["synth_vol_sustain"] = 0.45
    d["synth_vol_release"] = 280.0
    d["swing_amount"] = 50.0
    d["note_length_percent"] = 70.0
    presets.append(create_preset("Balkan Fire", "Factory",
        "Asymmetric dance - 7/8 euclidean pulse drives through minor harmonies", d))

    # 9. Tokyo Drift - Japanese scale, sparse
    d = create_default_preset()
    d["straight_1_4"] = [100.0, 0.0, 70.0, 0.0]
    d["straight_1_8"] = [0.0, 50.0, 0.0, 40.0, 0.0, 55.0, 0.0, 35.0]
    d["dotted_1_4d"] = [60.0, 45.0, 50.0]
    d["strength_values"] = create_strength_pattern("sparse")
    d["root_note"] = 52
    d["notes"] = [note_to_dict(Note(52, 100)), note_to_dict(Note(53, 50)), note_to_dict(Note(57, 55)),
                  note_to_dict(Note(59, 45)), note_to_dict(Note(60, 40))]
    d["synth_osc_d"] = 0.2
    d["synth_osc_v"] = 0.8
    d["synth_osc_volume"] = 0.65
    d["synth_filter_cutoff"] = 6000.0
    d["synth_filter_resonance"] = 0.15
    d["synth_filter_env_amount"] = 2000.0
    d["synth_vol_attack"] = 8.0
    d["synth_vol_decay"] = 500.0
    d["synth_vol_sustain"] = 0.35
    d["synth_vol_release"] = 800.0
    d["synth_reverb_mix"] = 0.2
    d["synth_reverb_decay"] = 0.65
    d["synth_reverb_diffusion"] = 0.8
    d["note_length_percent"] = 130.0
    d["decay_mod_1_target"] = 0.8
    d["decay_mod_1_amount"] = 140.0
    d["decay_mod_1_prob"] = 0.45
    presets.append(create_preset("Tokyo Drift", "Factory",
        "Japanese zen garden - sparse notes float in reverberant space", d))

    # 10. Havana Clave - Cuban son clave
    d = create_default_preset()
    # 3-2 son clave pattern
    d["straight_1_8"] = [100.0, 0.0, 0.0, 85.0, 0.0, 0.0, 90.0, 0.0]
    d["straight_1_16"] = [0.0, 0.0, 0.0, 0.0, 80.0, 0.0, 0.0, 0.0, 0.0, 0.0, 75.0, 0.0, 0.0, 0.0, 0.0, 0.0]
    d["triplet_1_8t"] = [55.0, 40.0, 45.0, 50.0, 35.0, 40.0, 60.0, 45.0, 50.0, 45.0, 40.0, 55.0]
    d["strength_values"] = create_strength_pattern("latin")
    d["root_note"] = 48
    d["notes"] = [note_to_dict(Note(48, 100)), note_to_dict(Note(50, 55)), note_to_dict(Note(52, 50)),
                  note_to_dict(Note(55, 60)), note_to_dict(Note(57, 45))]
    d["synth_osc_d"] = 0.42
    d["synth_osc_v"] = 0.58
    d["synth_osc_volume"] = 0.7
    d["synth_sub_volume"] = 0.2
    d["synth_filter_cutoff"] = 2500.0
    d["synth_filter_resonance"] = 0.18
    d["synth_filter_env_amount"] = 1200.0
    d["synth_vol_attack"] = 3.0
    d["synth_vol_decay"] = 180.0
    d["synth_vol_sustain"] = 0.5
    d["synth_vol_release"] = 250.0
    d["swing_amount"] = 55.0
    d["note_length_percent"] = 65.0
    presets.append(create_preset("Havana Clave", "Factory",
        "Cuban son rhythm - classic 3-2 clave drives syncopated melodies", d))

    # 11. Delhi Express - Indian rhythmic feel
    d = create_default_preset()
    d["straight_1_8"] = [100.0, 50.0, 70.0, 45.0, 85.0, 55.0, 60.0, 65.0]
    d["straight_1_16"] = [0.0, 45.0, 40.0, 35.0, 0.0, 50.0, 35.0, 40.0, 0.0, 55.0, 45.0, 30.0, 0.0, 40.0, 50.0, 35.0]
    d["triplet_1_4t"] = [75.0, 55.0, 65.0, 70.0, 50.0, 60.0]
    d["strength_values"] = create_strength_pattern("polyrhythm_3_4")
    d["root_note"] = 50
    d["notes"] = [note_to_dict(Note(50, 100)), note_to_dict(Note(52, 55)), note_to_dict(Note(53, 45)),
                  note_to_dict(Note(55, 60)), note_to_dict(Note(57, 50)), note_to_dict(Note(60, 35))]
    d["synth_osc_d"] = 0.38
    d["synth_osc_v"] = 0.62
    d["synth_osc_volume"] = 0.68
    d["synth_filter_cutoff"] = 2200.0
    d["synth_filter_resonance"] = 0.25
    d["synth_filter_env_amount"] = 1800.0
    d["synth_filt_decay"] = 300.0
    d["synth_vol_attack"] = 2.0
    d["synth_vol_decay"] = 220.0
    d["synth_vol_sustain"] = 0.55
    d["synth_vol_release"] = 320.0
    d["swing_amount"] = 52.0
    d["note_length_percent"] = 80.0
    d["pos_mod_1_target"] = -0.5
    d["pos_mod_1_shift"] = 0.025
    d["pos_mod_1_prob"] = 0.3
    presets.append(create_preset("Delhi Express", "Factory",
        "Tabla-inspired patterns - rapid ornaments over steady pulse", d))

    # 12. Celtic Reel - Irish dance
    d = create_default_preset()
    d["straight_1_8"] = [100.0, 70.0, 85.0, 65.0, 90.0, 75.0, 80.0, 70.0]
    d["straight_1_16"] = [0.0, 50.0, 45.0, 40.0, 0.0, 55.0, 40.0, 45.0, 0.0, 50.0, 55.0, 35.0, 0.0, 45.0, 50.0, 40.0]
    d["triplet_1_8t"] = [60.0, 45.0, 50.0, 55.0, 40.0, 45.0, 65.0, 50.0, 45.0, 50.0, 55.0, 40.0]
    d["strength_values"] = create_strength_pattern("shuffle")
    d["root_note"] = 50
    d["notes"] = [note_to_dict(Note(50, 100)), note_to_dict(Note(52, 60)), note_to_dict(Note(55, 55)),
                  note_to_dict(Note(57, 50)), note_to_dict(Note(59, 45)), note_to_dict(Note(62, 35))]
    d["synth_osc_d"] = 0.32
    d["synth_osc_v"] = 0.68
    d["synth_osc_volume"] = 0.72
    d["synth_filter_cutoff"] = 4200.0
    d["synth_filter_resonance"] = 0.1
    d["synth_filter_env_amount"] = 1000.0
    d["synth_vol_attack"] = 1.0
    d["synth_vol_decay"] = 140.0
    d["synth_vol_sustain"] = 0.45
    d["synth_vol_release"] = 200.0
    d["swing_amount"] = 54.0
    d["note_length_percent"] = 55.0
    presets.append(create_preset("Celtic Reel", "Factory",
        "Irish dance energy - fast ornamental runs over driving pulse", d))

    # 13. Gnawa Trance - Moroccan hypnotic
    d = create_default_preset()
    d["straight_1_4"] = [100.0, 80.0, 90.0, 75.0]
    d["straight_1_8"] = [0.0, 60.0, 0.0, 55.0, 0.0, 65.0, 0.0, 50.0]
    d["triplet_1_4t"] = [85.0, 60.0, 70.0, 80.0, 55.0, 65.0]
    d["strength_values"] = create_strength_pattern("driving")
    d["root_note"] = 45
    d["notes"] = [note_to_dict(Note(45, 100)), note_to_dict(Note(48, 60)), note_to_dict(Note(50, 55)),
                  note_to_dict(Note(52, 45))]
    d["synth_osc_d"] = 0.28
    d["synth_osc_v"] = 0.45
    d["synth_osc_volume"] = 0.68
    d["synth_sub_volume"] = 0.3
    d["synth_filter_cutoff"] = 1500.0
    d["synth_filter_resonance"] = 0.3
    d["synth_filter_env_amount"] = 600.0
    d["synth_vol_attack"] = 5.0
    d["synth_vol_decay"] = 300.0
    d["synth_vol_sustain"] = 0.6
    d["synth_vol_release"] = 400.0
    d["synth_reverb_mix"] = 0.2
    d["synth_reverb_decay"] = 0.55
    d["swing_amount"] = 50.0
    d["note_length_percent"] = 100.0
    d["lfo1_tempo_sync"] = True
    d["lfo1_sync_division"] = 3
    d["lfo1_waveform"] = 0
    d["lfo1_dest1"] = 12
    d["lfo1_amount1"] = 0.12
    presets.append(create_preset("Gnawa Trance", "Factory",
        "Moroccan hypnotic ritual - repetitive patterns induce deep trance", d))

    # 14. Steel Pan Yard - Caribbean steel drum
    d = create_default_preset()
    d["straight_1_8"] = [100.0, 55.0, 80.0, 50.0, 90.0, 60.0, 75.0, 65.0]
    d["straight_1_16"] = [0.0, 45.0, 40.0, 35.0, 0.0, 50.0, 45.0, 30.0, 0.0, 55.0, 40.0, 45.0, 0.0, 35.0, 50.0, 40.0]
    d["strength_values"] = create_strength_pattern("latin")
    d["root_note"] = 60
    d["notes"] = [note_to_dict(Note(60, 100)), note_to_dict(Note(62, 55)), note_to_dict(Note(64, 60)),
                  note_to_dict(Note(67, 50)), note_to_dict(Note(69, 45)), note_to_dict(Note(72, 35))]
    d["synth_osc_d"] = 0.65
    d["synth_osc_v"] = 0.72
    d["synth_osc_stereo_v_offset"] = 0.1
    d["synth_osc_volume"] = 0.7
    d["synth_filter_cutoff"] = 5000.0
    d["synth_filter_resonance"] = 0.2
    d["synth_filter_env_amount"] = 2200.0
    d["synth_vol_attack"] = 1.0
    d["synth_vol_decay"] = 400.0
    d["synth_vol_sustain"] = 0.3
    d["synth_vol_release"] = 500.0
    d["swing_amount"] = 56.0
    d["note_length_percent"] = 110.0
    d["decay_mod_1_target"] = 0.75
    d["decay_mod_1_amount"] = 135.0
    d["decay_mod_1_prob"] = 0.4
    presets.append(create_preset("Steel Pan Yard", "Factory",
        "Trinidad carnival - bright melodic patterns ring out in the sun", d))

    # 15. Didgeridoo Drone - Australian aboriginal
    d = create_default_preset()
    d["straight_1_1"] = [100.0]
    d["straight_1_2"] = [70.0, 60.0]
    d["straight_1_4"] = [50.0, 40.0, 45.0, 35.0]
    d["straight_1_8"] = [30.0, 25.0, 35.0, 20.0, 40.0, 30.0, 25.0, 35.0]
    d["strength_values"] = create_strength_pattern("ambient")
    d["root_note"] = 36
    d["notes"] = [note_to_dict(Note(36, 100)), note_to_dict(Note(43, 45)), note_to_dict(Note(48, 35))]
    d["synth_osc_d"] = 0.15
    d["synth_osc_v"] = 0.35
    d["synth_osc_volume"] = 0.65
    d["synth_sub_volume"] = 0.4
    d["synth_filter_cutoff"] = 800.0
    d["synth_filter_resonance"] = 0.35
    d["synth_filter_env_amount"] = 400.0
    d["synth_vol_attack"] = 50.0
    d["synth_vol_decay"] = 800.0
    d["synth_vol_sustain"] = 0.7
    d["synth_vol_release"] = 1000.0
    d["synth_reverb_mix"] = 0.2
    d["synth_reverb_decay"] = 0.6
    d["note_length_percent"] = 180.0
    d["lfo1_rate"] = 0.15
    d["lfo1_waveform"] = 0
    d["lfo1_dest1"] = 12
    d["lfo1_amount1"] = 0.2
    d["lfo1_dest2"] = 11
    d["lfo1_amount2"] = 0.1
    presets.append(create_preset("Didgeridoo Drone", "Factory",
        "Ancient breath - deep drone with subtle rhythmic undulations", d))

    # 16. Highlife Lagos - Nigerian dance
    d = create_default_preset()
    d["straight_1_8"] = [100.0, 65.0, 85.0, 55.0, 90.0, 70.0, 80.0, 60.0]
    d["straight_1_16"] = [0.0, 50.0, 45.0, 0.0, 0.0, 55.0, 40.0, 0.0, 0.0, 60.0, 50.0, 0.0, 0.0, 45.0, 55.0, 0.0]
    d["strength_values"] = create_strength_pattern("african")
    d["root_note"] = 55
    d["notes"] = [note_to_dict(Note(55, 100)), note_to_dict(Note(57, 60)), note_to_dict(Note(59, 55)),
                  note_to_dict(Note(62, 50)), note_to_dict(Note(64, 45)), note_to_dict(Note(66, 35))]
    d["synth_osc_d"] = 0.4
    d["synth_osc_v"] = 0.6
    d["synth_osc_volume"] = 0.72
    d["synth_filter_cutoff"] = 3800.0
    d["synth_filter_resonance"] = 0.12
    d["synth_filter_env_amount"] = 1400.0
    d["synth_vol_attack"] = 2.0
    d["synth_vol_decay"] = 180.0
    d["synth_vol_sustain"] = 0.5
    d["synth_vol_release"] = 250.0
    d["swing_amount"] = 55.0
    d["note_length_percent"] = 70.0
    d["pos_mod_1_target"] = -0.6
    d["pos_mod_1_shift"] = 0.018
    d["pos_mod_1_prob"] = 0.3
    presets.append(create_preset("Highlife Lagos", "Factory",
        "Nigerian dance party - bright melodies over infectious groove", d))

    # 17. Raga Dawn - North Indian morning raga
    d = create_default_preset()
    d["straight_1_4"] = [90.0, 50.0, 70.0, 45.0]
    d["straight_1_8"] = [60.0, 35.0, 50.0, 30.0, 55.0, 32.0, 48.0, 28.0]
    d["triplet_1_8t"] = [40.0, 25.0, 35.0, 20.0, 38.0, 22.0, 32.0, 18.0, 28.0, 15.0, 25.0, 12.0]
    d["strength_values"] = create_strength_pattern("indian")
    d["root_note"] = 48
    d["scale"] = "Major"
    d["stability_pattern"] = "Melodic"
    d["notes"] = [note_to_dict(Note(48, 127)), note_to_dict(Note(50, 70)), note_to_dict(Note(52, 80)),
                  note_to_dict(Note(55, 90)), note_to_dict(Note(57, 75)), note_to_dict(Note(59, 60))]
    d["synth_osc_d"] = 0.25
    d["synth_osc_v"] = 0.55
    d["synth_osc_volume"] = 0.68
    d["synth_pll_volume"] = 0.25
    d["synth_pll_track_speed"] = 0.4
    d["synth_filter_cutoff"] = 2800.0
    d["synth_filter_resonance"] = 0.15
    d["synth_vol_attack"] = 15.0
    d["synth_vol_decay"] = 400.0
    d["synth_vol_sustain"] = 0.5
    d["synth_vol_release"] = 600.0
    d["synth_reverb_mix"] = 0.18
    d["synth_reverb_decay"] = 0.55
    d["swing_amount"] = 52.0
    presets.append(create_preset("Raga Dawn", "Factory",
        "North Indian morning raga - ascending melodies with gentle ornaments", d))

    # 18. Celtic Jig - Irish dance rhythm
    d = create_default_preset()
    d["triplet_1_4t"] = [100.0, 70.0, 85.0, 90.0, 65.0, 80.0]
    d["triplet_1_8t"] = [80.0, 50.0, 60.0, 75.0, 45.0, 55.0, 70.0, 40.0, 52.0, 65.0, 38.0, 48.0]
    d["strength_values"] = create_strength_pattern("compound")
    d["root_note"] = 50
    d["scale"] = "Dorian"
    d["notes"] = [note_to_dict(Note(50, 127)), note_to_dict(Note(52, 75)), note_to_dict(Note(53, 70)),
                  note_to_dict(Note(55, 90)), note_to_dict(Note(57, 85)), note_to_dict(Note(59, 65)),
                  note_to_dict(Note(62, 55))]
    d["synth_osc_d"] = 0.35
    d["synth_osc_v"] = 0.5
    d["synth_osc_volume"] = 0.72
    d["synth_filter_cutoff"] = 3500.0
    d["synth_filter_env_amount"] = 1200.0
    d["synth_vol_attack"] = 2.0
    d["synth_vol_decay"] = 120.0
    d["synth_vol_sustain"] = 0.4
    d["synth_vol_release"] = 150.0
    d["note_length_percent"] = 65.0
    presets.append(create_preset("Celtic Jig", "Factory",
        "Irish jig in 6/8 - lively triplet dance with Dorian color", d))

    # 19. Gamelan Metalophone - Indonesian textures
    d = create_default_preset()
    d["straight_1_4"] = [85.0, 55.0, 70.0, 50.0]
    d["straight_1_8"] = [65.0, 40.0, 55.0, 35.0, 60.0, 38.0, 52.0, 32.0]
    d["dotted_1_8d"] = [50.0, 35.0, 45.0, 30.0, 42.0, 28.0]
    d["strength_values"] = create_strength_pattern("gamelan")
    d["root_note"] = 53
    d["scale"] = "PentatonicMinor"
    d["notes"] = [note_to_dict(Note(53, 127)), note_to_dict(Note(56, 85)), note_to_dict(Note(58, 90)),
                  note_to_dict(Note(60, 80)), note_to_dict(Note(63, 70)), note_to_dict(Note(65, 55))]
    d["synth_osc_d"] = 0.6
    d["synth_osc_v"] = 0.3
    d["synth_osc_volume"] = 0.6
    d["synth_pll_volume"] = 0.35
    d["synth_pll_track_speed"] = 0.55
    d["synth_pll_fm_amount"] = 0.12
    d["synth_pll_fm_ratio"] = 4
    d["synth_filter_cutoff"] = 4500.0
    d["synth_vol_attack"] = 3.0
    d["synth_vol_decay"] = 600.0
    d["synth_vol_sustain"] = 0.2
    d["synth_vol_release"] = 900.0
    d["synth_reverb_mix"] = 0.22
    d["synth_reverb_decay"] = 0.65
    presets.append(create_preset("Gamelan Bronze", "Factory",
        "Balinese metalophone - shimmering bell tones with interlocking patterns", d))

    # 20. Greek Sirtaki - Bouzouki dance
    d = create_default_preset()
    d["straight_1_8"] = [100.0, 60.0, 80.0, 50.0, 90.0, 55.0, 75.0, 45.0]
    d["straight_1_16"] = [0.0, 40.0, 35.0, 0.0, 0.0, 45.0, 30.0, 0.0, 0.0, 42.0, 38.0, 0.0, 0.0, 48.0, 32.0, 0.0]
    d["strength_values"] = create_strength_pattern("greek")
    d["root_note"] = 52
    d["scale"] = "HarmonicMinor"
    d["notes"] = [note_to_dict(Note(52, 127)), note_to_dict(Note(53, 65)), note_to_dict(Note(55, 80)),
                  note_to_dict(Note(57, 75)), note_to_dict(Note(59, 90)), note_to_dict(Note(60, 60)),
                  note_to_dict(Note(63, 70))]
    d["synth_osc_d"] = 0.45
    d["synth_osc_v"] = 0.55
    d["synth_osc_volume"] = 0.7
    d["synth_filter_cutoff"] = 3200.0
    d["synth_filter_resonance"] = 0.18
    d["synth_filter_env_amount"] = 1000.0
    d["synth_vol_attack"] = 2.0
    d["synth_vol_decay"] = 150.0
    d["synth_vol_sustain"] = 0.45
    d["synth_vol_release"] = 200.0
    d["swing_amount"] = 54.0
    d["note_length_percent"] = 70.0
    presets.append(create_preset("Greek Sirtaki", "Factory",
        "Hellenic dance - harmonic minor melodies with accelerating energy", d))

    # 21. Gnawa Trance - Moroccan ritual
    d = create_default_preset()
    d["straight_1_4"] = [100.0, 80.0, 90.0, 75.0]
    d["straight_1_8"] = [70.0, 55.0, 65.0, 50.0, 68.0, 52.0, 62.0, 48.0]
    d["triplet_1_8t"] = [60.0, 40.0, 50.0, 55.0, 38.0, 48.0, 52.0, 35.0, 45.0, 50.0, 32.0, 42.0]
    d["strength_values"] = create_strength_pattern("gnawa")
    d["root_note"] = 45
    d["scale"] = "Phrygian"
    d["stability_pattern"] = "BassHeavy"
    d["notes"] = [note_to_dict(Note(45, 127, 90, 100)), note_to_dict(Note(46, 60)), note_to_dict(Note(48, 75)),
                  note_to_dict(Note(50, 85)), note_to_dict(Note(52, 70)), note_to_dict(Note(53, 55))]
    d["synth_osc_d"] = 0.5
    d["synth_osc_v"] = 0.45
    d["synth_osc_volume"] = 0.65
    d["synth_sub_volume"] = 0.3
    d["synth_filter_cutoff"] = 1800.0
    d["synth_filter_resonance"] = 0.22
    d["synth_vol_attack"] = 5.0
    d["synth_vol_decay"] = 250.0
    d["synth_vol_sustain"] = 0.55
    d["synth_vol_release"] = 350.0
    d["synth_reverb_mix"] = 0.15
    d["swing_amount"] = 56.0
    presets.append(create_preset("Gnawa Trance", "Factory",
        "Moroccan ritual - hypnotic Phrygian bass grooves for spiritual journey", d))

    # 22. Koto Garden - Japanese elegance
    d = create_default_preset()
    d["straight_1_4"] = [80.0, 45.0, 60.0, 40.0]
    d["straight_1_8"] = [55.0, 30.0, 45.0, 25.0, 50.0, 28.0, 42.0, 22.0]
    d["dotted_1_4d"] = [50.0, 35.0, 40.0]
    d["strength_values"] = create_strength_pattern("sparse")
    d["root_note"] = 52
    d["scale"] = "Japanese"
    d["stability_pattern"] = "Melodic"
    d["notes"] = [note_to_dict(Note(52, 127)), note_to_dict(Note(53, 70)), note_to_dict(Note(57, 85)),
                  note_to_dict(Note(59, 80)), note_to_dict(Note(60, 65)), note_to_dict(Note(64, 55))]
    d["synth_osc_d"] = 0.55
    d["synth_osc_v"] = 0.35
    d["synth_osc_volume"] = 0.6
    d["synth_pll_volume"] = 0.3
    d["synth_pll_track_speed"] = 0.5
    d["synth_filter_cutoff"] = 4000.0
    d["synth_vol_attack"] = 5.0
    d["synth_vol_decay"] = 500.0
    d["synth_vol_sustain"] = 0.25
    d["synth_vol_release"] = 800.0
    d["synth_reverb_mix"] = 0.2
    d["synth_reverb_decay"] = 0.6
    d["note_length_percent"] = 90.0
    presets.append(create_preset("Koto Garden", "Factory",
        "Japanese zen garden - delicate plucked tones with contemplative space", d))

    # 23. Cuban Montuno - Piano pattern
    d = create_default_preset()
    d["straight_1_8"] = [100.0, 70.0, 85.0, 60.0, 95.0, 65.0, 80.0, 55.0]
    d["straight_1_16"] = [0.0, 50.0, 40.0, 55.0, 0.0, 45.0, 35.0, 50.0, 0.0, 48.0, 38.0, 52.0, 0.0, 42.0, 32.0, 48.0]
    d["strength_values"] = create_strength_pattern("clave")
    d["root_note"] = 48
    d["scale"] = "Mixolydian"
    d["notes"] = [note_to_dict(Note(48, 127)), note_to_dict(Note(50, 75)), note_to_dict(Note(52, 85)),
                  note_to_dict(Note(55, 95)), note_to_dict(Note(57, 70)), note_to_dict(Note(58, 60)),
                  note_to_dict(Note(60, 80))]
    d["synth_osc_d"] = 0.4
    d["synth_osc_v"] = 0.6
    d["synth_osc_volume"] = 0.72
    d["synth_filter_cutoff"] = 3600.0
    d["synth_filter_env_amount"] = 1200.0
    d["synth_vol_attack"] = 2.0
    d["synth_vol_decay"] = 140.0
    d["synth_vol_sustain"] = 0.4
    d["synth_vol_release"] = 180.0
    d["swing_amount"] = 55.0
    d["note_length_percent"] = 60.0
    presets.append(create_preset("Cuban Montuno", "Factory",
        "Havana piano pattern - syncopated Mixolydian runs over clave rhythm", d))

    # 24. Pygmy Polyphony - Central African vocal
    d = create_default_preset()
    d["straight_1_8"] = [90.0, 65.0, 80.0, 60.0, 85.0, 62.0, 78.0, 58.0]
    d["triplet_1_8t"] = [70.0, 50.0, 60.0, 65.0, 48.0, 58.0, 62.0, 45.0, 55.0, 60.0, 42.0, 52.0]
    d["strength_values"] = create_strength_pattern("african")
    d["root_note"] = 55
    d["scale"] = "PentatonicMajor"
    d["stability_pattern"] = "Melodic"
    d["octave_randomization"] = create_octave_randomization(0.16, 0.25, 0.25, "Both")
    d["notes"] = [note_to_dict(Note(55, 127)), note_to_dict(Note(57, 80)), note_to_dict(Note(59, 85)),
                  note_to_dict(Note(62, 90)), note_to_dict(Note(64, 75))]
    d["synth_osc_d"] = 0.2
    d["synth_osc_v"] = 0.6
    d["synth_osc_volume"] = 0.65
    d["synth_filter_cutoff"] = 3000.0
    d["synth_vol_attack"] = 10.0
    d["synth_vol_decay"] = 200.0
    d["synth_vol_sustain"] = 0.5
    d["synth_vol_release"] = 300.0
    d["synth_reverb_mix"] = 0.15
    d["note_length_percent"] = 80.0
    presets.append(create_preset("Pygmy Polyphony", "Factory",
        "Central African vocal style - interlocking pentatonic melodies", d))

    # 25. Nordic Hymn - Scandinavian folk
    d = create_default_preset()
    d["straight_1_2"] = [100.0, 80.0]
    d["straight_1_4"] = [70.0, 50.0, 60.0, 45.0]
    d["straight_1_8"] = [45.0, 30.0, 40.0, 25.0, 42.0, 28.0, 38.0, 22.0]
    d["strength_values"] = create_strength_pattern("sparse")
    d["root_note"] = 48
    d["scale"] = "Dorian"
    d["stability_pattern"] = "Ambient"
    d["notes"] = [note_to_dict(Note(48, 127)), note_to_dict(Note(50, 75)), note_to_dict(Note(51, 70)),
                  note_to_dict(Note(55, 90)), note_to_dict(Note(57, 80)), note_to_dict(Note(58, 60))]
    d["synth_osc_d"] = 0.15
    d["synth_osc_v"] = 0.55
    d["synth_osc_stereo_v_offset"] = 0.08
    d["synth_osc_volume"] = 0.6
    d["synth_pll_volume"] = 0.35
    d["synth_pll_track_speed"] = 0.35
    d["synth_filter_cutoff"] = 2200.0
    d["synth_vol_attack"] = 80.0
    d["synth_vol_decay"] = 600.0
    d["synth_vol_sustain"] = 0.65
    d["synth_vol_release"] = 900.0
    d["synth_reverb_mix"] = 0.22
    d["synth_reverb_decay"] = 0.7
    d["note_length_percent"] = 150.0
    presets.append(create_preset("Nordic Hymn", "Factory",
        "Scandinavian folk hymn - austere Dorian melodies with wide spaces", d))

    # 26. Bhangra Beat - Punjabi dance
    d = create_default_preset()
    d["straight_1_4"] = [100.0, 85.0, 95.0, 80.0]
    d["straight_1_8"] = [75.0, 55.0, 70.0, 50.0, 72.0, 52.0, 68.0, 48.0]
    d["straight_1_16"] = [0.0, 45.0, 40.0, 0.0, 0.0, 48.0, 35.0, 0.0, 0.0, 42.0, 38.0, 0.0, 0.0, 50.0, 32.0, 0.0]
    d["strength_values"] = create_strength_pattern("bhangra")
    d["root_note"] = 50
    d["scale"] = "Mixolydian"
    d["notes"] = [note_to_dict(Note(50, 127)), note_to_dict(Note(52, 80)), note_to_dict(Note(54, 85)),
                  note_to_dict(Note(57, 90)), note_to_dict(Note(59, 75)), note_to_dict(Note(60, 65))]
    d["synth_osc_d"] = 0.48
    d["synth_osc_v"] = 0.52
    d["synth_osc_volume"] = 0.7
    d["synth_sub_volume"] = 0.25
    d["synth_filter_cutoff"] = 2800.0
    d["synth_filter_resonance"] = 0.15
    d["synth_filter_env_amount"] = 1000.0
    d["synth_vol_attack"] = 2.0
    d["synth_vol_decay"] = 130.0
    d["synth_vol_sustain"] = 0.5
    d["synth_vol_release"] = 180.0
    d["swing_amount"] = 53.0
    d["note_length_percent"] = 65.0
    presets.append(create_preset("Bhangra Beat", "Factory",
        "Punjabi dance energy - driving rhythms with Mixolydian brightness", d))

    # 27. Hungarian Czardas - Gypsy violin
    d = create_default_preset()
    d["straight_1_8"] = [100.0, 55.0, 80.0, 50.0, 90.0, 52.0, 75.0, 48.0]
    d["straight_1_16"] = [0.0, 45.0, 40.0, 48.0, 0.0, 42.0, 38.0, 45.0, 0.0, 48.0, 42.0, 50.0, 0.0, 40.0, 35.0, 42.0]
    d["strength_values"] = create_strength_pattern("rubato")
    d["root_note"] = 52
    d["scale"] = "Hungarian"
    d["stability_pattern"] = "Melodic"
    d["notes"] = [note_to_dict(Note(52, 127)), note_to_dict(Note(54, 70)), note_to_dict(Note(55, 75)),
                  note_to_dict(Note(58, 90)), note_to_dict(Note(59, 85)), note_to_dict(Note(60, 65)),
                  note_to_dict(Note(63, 80))]
    d["synth_osc_d"] = 0.3
    d["synth_osc_v"] = 0.6
    d["synth_osc_volume"] = 0.68
    d["synth_filter_cutoff"] = 3500.0
    d["synth_filter_resonance"] = 0.12
    d["synth_vol_attack"] = 8.0
    d["synth_vol_decay"] = 200.0
    d["synth_vol_sustain"] = 0.55
    d["synth_vol_release"] = 280.0
    d["synth_reverb_mix"] = 0.12
    d["pos_mod_1_target"] = -0.5
    d["pos_mod_1_shift"] = 0.02
    d["pos_mod_1_prob"] = 0.25
    presets.append(create_preset("Hungarian Czardas", "Factory",
        "Gypsy dance - Hungarian minor with passionate rubato phrasing", d))

    # 28. Polynesian Drift - Pacific Island
    d = create_default_preset()
    d["straight_1_4"] = [80.0, 55.0, 70.0, 50.0]
    d["straight_1_8"] = [55.0, 35.0, 48.0, 30.0, 52.0, 32.0, 45.0, 28.0]
    d["triplet_1_8t"] = [45.0, 28.0, 38.0, 42.0, 25.0, 35.0, 40.0, 22.0, 32.0, 38.0, 20.0, 30.0]
    d["strength_values"] = create_strength_pattern("flowing")
    d["root_note"] = 48
    d["scale"] = "Major"
    d["stability_pattern"] = "Ambient"
    d["notes"] = [note_to_dict(Note(48, 127)), note_to_dict(Note(50, 70)), note_to_dict(Note(52, 80)),
                  note_to_dict(Note(55, 90)), note_to_dict(Note(57, 75)), note_to_dict(Note(59, 60))]
    d["synth_osc_d"] = 0.2
    d["synth_osc_v"] = 0.55
    d["synth_osc_stereo_v_offset"] = 0.1
    d["synth_osc_volume"] = 0.6
    d["synth_pll_volume"] = 0.3
    d["synth_filter_cutoff"] = 2500.0
    d["synth_vol_attack"] = 40.0
    d["synth_vol_decay"] = 400.0
    d["synth_vol_sustain"] = 0.55
    d["synth_vol_release"] = 600.0
    d["synth_reverb_mix"] = 0.2
    d["synth_reverb_decay"] = 0.6
    d["note_length_percent"] = 120.0
    presets.append(create_preset("Polynesian Drift", "Factory",
        "Pacific island calm - gentle major key waves with ocean-like flow", d))

    # 29. Tuvan Throat - Overtone singing
    d = create_default_preset()
    d["straight_1_1"] = [90.0]
    d["straight_1_2"] = [60.0, 50.0]
    d["straight_1_4"] = [40.0, 30.0, 35.0, 25.0]
    d["strength_values"] = create_strength_pattern("drone")
    d["root_note"] = 41
    d["scale"] = "PentatonicMinor"
    d["stability_pattern"] = "BassHeavy"
    d["notes"] = [note_to_dict(Note(41, 127, 100, 110)), note_to_dict(Note(44, 70)), note_to_dict(Note(46, 75)),
                  note_to_dict(Note(48, 85)), note_to_dict(Note(51, 65))]
    d["synth_osc_d"] = 0.65
    d["synth_osc_v"] = 0.25
    d["synth_osc_volume"] = 0.5
    d["synth_pll_volume"] = 0.45
    d["synth_pll_track_speed"] = 0.3
    d["synth_pll_damping"] = 0.2
    d["synth_pll_fm_amount"] = 0.15
    d["synth_pll_fm_ratio"] = 3
    d["synth_sub_volume"] = 0.35
    d["synth_filter_cutoff"] = 1500.0
    d["synth_filter_resonance"] = 0.25
    d["synth_vol_attack"] = 100.0
    d["synth_vol_decay"] = 800.0
    d["synth_vol_sustain"] = 0.7
    d["synth_vol_release"] = 1200.0
    d["synth_reverb_mix"] = 0.18
    d["synth_reverb_decay"] = 0.65
    d["note_length_percent"] = 200.0
    presets.append(create_preset("Tuvan Throat", "Factory",
        "Mongolian overtone singing - rich harmonics emerge from drone foundation", d))

    # 30. Persian Garden - Iranian classical
    d = create_default_preset()
    d["straight_1_4"] = [90.0, 55.0, 75.0, 50.0]
    d["straight_1_8"] = [65.0, 40.0, 55.0, 35.0, 60.0, 38.0, 52.0, 32.0]
    d["triplet_1_8t"] = [50.0, 30.0, 42.0, 48.0, 28.0, 40.0, 45.0, 25.0, 38.0, 42.0, 22.0, 35.0]
    d["strength_values"] = create_strength_pattern("persian")
    d["root_note"] = 50
    d["scale"] = "Arabic"
    d["stability_pattern"] = "Melodic"
    d["notes"] = [note_to_dict(Note(50, 127)), note_to_dict(Note(51, 65)), note_to_dict(Note(54, 85)),
                  note_to_dict(Note(55, 80)), note_to_dict(Note(57, 90)), note_to_dict(Note(58, 60)),
                  note_to_dict(Note(60, 70))]
    d["synth_osc_d"] = 0.3
    d["synth_osc_v"] = 0.55
    d["synth_osc_volume"] = 0.65
    d["synth_filter_cutoff"] = 3200.0
    d["synth_filter_resonance"] = 0.15
    d["synth_vol_attack"] = 12.0
    d["synth_vol_decay"] = 300.0
    d["synth_vol_sustain"] = 0.5
    d["synth_vol_release"] = 450.0
    d["synth_reverb_mix"] = 0.18
    d["synth_reverb_decay"] = 0.55
    d["swing_amount"] = 52.0
    presets.append(create_preset("Persian Garden", "Factory",
        "Iranian classical - ornate Arabic scale melodies with gentle flow", d))

    # 31. Cajun Stomp - Louisiana two-step
    d = create_default_preset()
    d["straight_1_4"] = [100.0, 90.0, 95.0, 85.0]
    d["straight_1_8"] = [80.0, 60.0, 75.0, 55.0, 78.0, 58.0, 72.0, 52.0]
    d["strength_values"] = create_strength_pattern("backbeat")
    d["root_note"] = 48
    d["scale"] = "Mixolydian"
    d["notes"] = [note_to_dict(Note(48, 127)), note_to_dict(Note(50, 75)), note_to_dict(Note(52, 85)),
                  note_to_dict(Note(55, 90)), note_to_dict(Note(57, 70)), note_to_dict(Note(58, 65)),
                  note_to_dict(Note(60, 80))]
    d["synth_osc_d"] = 0.42
    d["synth_osc_v"] = 0.58
    d["synth_osc_volume"] = 0.72
    d["synth_filter_cutoff"] = 3400.0
    d["synth_filter_resonance"] = 0.12
    d["synth_filter_env_amount"] = 1000.0
    d["synth_vol_attack"] = 2.0
    d["synth_vol_decay"] = 130.0
    d["synth_vol_sustain"] = 0.5
    d["synth_vol_release"] = 180.0
    d["swing_amount"] = 58.0
    d["note_length_percent"] = 65.0
    presets.append(create_preset("Cajun Stomp", "Factory",
        "Louisiana dance hall - swinging Mixolydian with accordion feel", d))

    # 32. Sufi Whirl - Mystical Turkish
    d = create_default_preset()
    d["straight_1_4"] = [95.0, 70.0, 85.0, 65.0]
    d["triplet_1_4t"] = [80.0, 55.0, 70.0, 75.0, 52.0, 68.0]
    d["triplet_1_8t"] = [60.0, 40.0, 52.0, 58.0, 38.0, 50.0, 55.0, 35.0, 48.0, 52.0, 32.0, 45.0]
    d["strength_values"] = create_strength_pattern("whirling")
    d["root_note"] = 48
    d["scale"] = "Phrygian"
    d["stability_pattern"] = "Traditional"
    d["notes"] = [note_to_dict(Note(48, 127)), note_to_dict(Note(49, 70)), note_to_dict(Note(51, 80)),
                  note_to_dict(Note(53, 85)), note_to_dict(Note(55, 90)), note_to_dict(Note(56, 65)),
                  note_to_dict(Note(58, 75))]
    d["synth_osc_d"] = 0.28
    d["synth_osc_v"] = 0.55
    d["synth_osc_volume"] = 0.65
    d["synth_pll_volume"] = 0.3
    d["synth_pll_track_speed"] = 0.4
    d["synth_filter_cutoff"] = 2800.0
    d["synth_filter_resonance"] = 0.18
    d["synth_vol_attack"] = 20.0
    d["synth_vol_decay"] = 350.0
    d["synth_vol_sustain"] = 0.55
    d["synth_vol_release"] = 500.0
    d["synth_reverb_mix"] = 0.2
    d["synth_reverb_decay"] = 0.6
    d["note_length_percent"] = 100.0
    presets.append(create_preset("Sufi Whirl", "Factory",
        "Turkish mystical - Phrygian rotation for meditative dance", d))

    return presets

def create_bank_b() -> List[Dict]:
    """Bank B: Electronic & Modern - 16 presets"""
    presets = []

    # 1. Berlin Pulse - Driving techno
    d = create_default_preset()
    d["straight_1_4"] = [100.0, 100.0, 100.0, 100.0]
    d["straight_1_8"] = [0.0, 60.0, 0.0, 55.0, 0.0, 65.0, 0.0, 50.0]
    d["straight_1_16"] = [0.0, 0.0, 45.0, 0.0, 0.0, 0.0, 50.0, 0.0, 0.0, 0.0, 40.0, 0.0, 0.0, 0.0, 55.0, 0.0]
    d["strength_values"] = create_strength_pattern("4_4_standard")
    d["root_note"] = 36
    d["notes"] = [note_to_dict(Note(36, 100)), note_to_dict(Note(43, 40)), note_to_dict(Note(48, 30))]
    d["synth_osc_d"] = 0.58
    d["synth_osc_v"] = 0.42
    d["synth_osc_volume"] = 0.7
    d["synth_sub_volume"] = 0.35
    d["synth_filter_cutoff"] = 1200.0
    d["synth_filter_resonance"] = 0.28
    d["synth_filter_env_amount"] = 800.0
    d["synth_vol_attack"] = 1.0
    d["synth_vol_decay"] = 150.0
    d["synth_vol_sustain"] = 0.4
    d["synth_vol_release"] = 180.0
    d["note_length_percent"] = 50.0
    d["lfo1_tempo_sync"] = True
    d["lfo1_sync_division"] = 2
    d["lfo1_waveform"] = 2
    d["lfo1_dest1"] = 12
    d["lfo1_amount1"] = 0.18
    presets.append(create_preset("Berlin Pulse", "Factory",
        "Four-on-floor drive - relentless kick pattern with filtered texture", d))

    # 2. Velvet House - Deep house groove
    d = create_default_preset()
    d["straight_1_4"] = [100.0, 70.0, 90.0, 75.0]
    d["straight_1_8"] = [0.0, 55.0, 0.0, 50.0, 0.0, 60.0, 0.0, 45.0]
    d["straight_1_16"] = [0.0, 40.0, 35.0, 0.0, 0.0, 45.0, 30.0, 0.0, 0.0, 50.0, 40.0, 0.0, 0.0, 35.0, 45.0, 0.0]
    d["strength_values"] = create_strength_pattern("shuffle")
    d["root_note"] = 43
    d["notes"] = [note_to_dict(Note(43, 100)), note_to_dict(Note(48, 55)), note_to_dict(Note(50, 50)),
                  note_to_dict(Note(55, 40))]
    d["synth_osc_d"] = 0.28
    d["synth_osc_v"] = 0.55
    d["synth_osc_volume"] = 0.68
    d["synth_sub_volume"] = 0.28
    d["synth_filter_cutoff"] = 1800.0
    d["synth_filter_resonance"] = 0.2
    d["synth_filter_env_amount"] = 1000.0
    d["synth_vol_attack"] = 5.0
    d["synth_vol_decay"] = 280.0
    d["synth_vol_sustain"] = 0.5
    d["synth_vol_release"] = 350.0
    d["synth_reverb_mix"] = 0.15
    d["synth_reverb_decay"] = 0.45
    d["swing_amount"] = 58.0
    d["note_length_percent"] = 80.0
    presets.append(create_preset("Velvet House", "Factory",
        "Smooth deep house - swung bass with warm filtered pads", d))

    # 3. Breakbeat Science - Broken beat patterns
    d = create_default_preset()
    euc_break = euclidean_rhythm(16, 9)
    for i in range(16):
        d["straight_1_16"][i] = 90.0 if i in euc_break else 35.0
    d["straight_1_8"] = [80.0, 50.0, 60.0, 70.0, 75.0, 45.0, 65.0, 55.0]
    d["strength_values"] = create_strength_pattern("funk")
    d["root_note"] = 41
    d["notes"] = [note_to_dict(Note(41, 100)), note_to_dict(Note(43, 55)), note_to_dict(Note(48, 50)),
                  note_to_dict(Note(53, 40))]
    d["synth_osc_d"] = 0.52
    d["synth_osc_v"] = 0.48
    d["synth_osc_volume"] = 0.72
    d["synth_filter_cutoff"] = 2200.0
    d["synth_filter_resonance"] = 0.22
    d["synth_filter_env_amount"] = 1400.0
    d["synth_vol_attack"] = 1.0
    d["synth_vol_decay"] = 120.0
    d["synth_vol_sustain"] = 0.35
    d["synth_vol_release"] = 150.0
    d["swing_amount"] = 52.0
    d["note_length_percent"] = 55.0
    d["pos_mod_1_target"] = -0.7
    d["pos_mod_1_shift"] = 0.02
    d["pos_mod_1_prob"] = 0.35
    presets.append(create_preset("Breakbeat Science", "Factory",
        "Broken beat complexity - euclidean breaks meet funk syncopation", d))

    # 4. Ambient Drift - Evolving pad
    d = create_default_preset()
    d["straight_1_1"] = [100.0]
    d["straight_1_2"] = [60.0, 55.0]
    d["straight_1_4"] = [45.0, 35.0, 40.0, 30.0]
    d["strength_values"] = create_strength_pattern("ambient")
    d["root_note"] = 48
    d["notes"] = [note_to_dict(Note(48, 100)), note_to_dict(Note(55, 60)), note_to_dict(Note(52, 55)),
                  note_to_dict(Note(60, 45)), note_to_dict(Note(64, 35))]
    d["synth_osc_d"] = 0.15
    d["synth_osc_v"] = 0.75
    d["synth_osc_stereo_v_offset"] = 0.15
    d["synth_osc_volume"] = 0.6
    d["synth_filter_cutoff"] = 3500.0
    d["synth_filter_resonance"] = 0.1
    d["synth_filter_env_amount"] = 500.0
    d["synth_vol_attack"] = 200.0
    d["synth_vol_decay"] = 1500.0
    d["synth_vol_sustain"] = 0.65
    d["synth_vol_release"] = 2000.0
    d["synth_reverb_mix"] = 0.2
    d["synth_reverb_decay"] = 0.75
    d["synth_reverb_diffusion"] = 0.9
    d["note_length_percent"] = 200.0
    d["lfo1_rate"] = 0.08
    d["lfo1_waveform"] = 0
    d["lfo1_dest1"] = 11
    d["lfo1_amount1"] = 0.15
    d["lfo1_dest2"] = 12
    d["lfo1_amount2"] = 0.1
    presets.append(create_preset("Ambient Drift", "Factory",
        "Slowly evolving textures - long notes shimmer in deep space", d))

    # 5. Glitch Grid - IDM fragmented
    d = create_default_preset()
    d["straight_1_16"] = [95.0, 30.0, 45.0, 80.0, 35.0, 70.0, 25.0, 85.0, 40.0, 75.0, 50.0, 20.0, 90.0, 35.0, 55.0, 65.0]
    d["straight_1_32"] = [50.0, 0.0, 40.0, 0.0, 55.0, 0.0, 35.0, 0.0, 45.0, 0.0, 60.0, 0.0, 30.0, 0.0, 50.0, 0.0,
                          40.0, 0.0, 55.0, 0.0, 35.0, 0.0, 45.0, 0.0, 60.0, 0.0, 25.0, 0.0, 50.0, 0.0, 40.0, 0.0]
    d["strength_values"] = create_strength_pattern("dense")
    d["root_note"] = 45
    d["notes"] = [note_to_dict(Note(45, 100)), note_to_dict(Note(48, 50)), note_to_dict(Note(52, 45)),
                  note_to_dict(Note(57, 40)), note_to_dict(Note(60, 35))]
    d["synth_osc_d"] = 0.68
    d["synth_osc_v"] = 0.35
    d["synth_osc_volume"] = 0.7
    d["synth_filter_cutoff"] = 4500.0
    d["synth_filter_resonance"] = 0.25
    d["synth_filter_env_amount"] = 2500.0
    d["synth_vol_attack"] = 1.0
    d["synth_vol_decay"] = 80.0
    d["synth_vol_sustain"] = 0.2
    d["synth_vol_release"] = 100.0
    d["note_length_percent"] = 35.0
    d["len_mod_1_target"] = -0.8
    d["len_mod_1_amount"] = 50.0
    d["len_mod_1_prob"] = 0.5
    d["pos_mod_1_target"] = 0.0
    d["pos_mod_1_shift"] = 0.03
    d["pos_mod_1_prob"] = 0.4
    presets.append(create_preset("Glitch Grid", "Factory",
        "Fragmented IDM - unpredictable note bursts stutter across the grid", d))

    # 6. Minimal Loop - Hypnotic repetition
    d = create_default_preset()
    d["straight_1_8"] = [100.0, 75.0, 90.0, 70.0, 95.0, 80.0, 85.0, 65.0]
    d["strength_values"] = create_strength_pattern("driving")
    d["root_note"] = 41
    d["notes"] = [note_to_dict(Note(41, 100)), note_to_dict(Note(43, 45)), note_to_dict(Note(48, 35))]
    d["synth_osc_d"] = 0.45
    d["synth_osc_v"] = 0.5
    d["synth_osc_volume"] = 0.7
    d["synth_sub_volume"] = 0.25
    d["synth_filter_cutoff"] = 1500.0
    d["synth_filter_resonance"] = 0.32
    d["synth_filter_env_amount"] = 700.0
    d["synth_vol_attack"] = 2.0
    d["synth_vol_decay"] = 180.0
    d["synth_vol_sustain"] = 0.45
    d["synth_vol_release"] = 220.0
    d["note_length_percent"] = 65.0
    d["lfo1_tempo_sync"] = True
    d["lfo1_sync_division"] = 0
    d["lfo1_waveform"] = 1
    d["lfo1_dest1"] = 12
    d["lfo1_amount1"] = 0.22
    presets.append(create_preset("Minimal Loop", "Factory",
        "Hypnotic repetition - slowly shifting filter over steady pulse", d))

    # 7. Neon Arp - Synthwave style
    d = create_default_preset()
    d["straight_1_16"] = [100.0, 70.0, 85.0, 65.0, 95.0, 75.0, 80.0, 60.0, 90.0, 70.0, 85.0, 55.0, 100.0, 65.0, 75.0, 70.0]
    d["strength_values"] = create_strength_pattern("4_4_standard")
    d["root_note"] = 48
    d["notes"] = [note_to_dict(Note(48, 100)), note_to_dict(Note(52, 65)), note_to_dict(Note(55, 60)),
                  note_to_dict(Note(60, 55)), note_to_dict(Note(64, 45))]
    d["synth_osc_d"] = 0.35
    d["synth_osc_v"] = 0.7
    d["synth_osc_stereo_v_offset"] = 0.1
    d["synth_osc_volume"] = 0.72
    d["synth_filter_cutoff"] = 3200.0
    d["synth_filter_resonance"] = 0.18
    d["synth_filter_env_amount"] = 1800.0
    d["synth_vol_attack"] = 1.0
    d["synth_vol_decay"] = 200.0
    d["synth_vol_sustain"] = 0.35
    d["synth_vol_release"] = 280.0
    d["synth_reverb_mix"] = 0.2
    d["synth_reverb_decay"] = 0.5
    d["note_length_percent"] = 70.0
    presets.append(create_preset("Neon Arp", "Factory",
        "80s synthwave arpeggios - bright cascading patterns under neon lights", d))

    # 8. Liquid DnB - Flowing drum and bass
    d = create_default_preset()
    d["straight_1_8"] = [100.0, 40.0, 50.0, 90.0, 45.0, 85.0, 55.0, 70.0]
    d["straight_1_16"] = [0.0, 60.0, 45.0, 0.0, 0.0, 55.0, 50.0, 0.0, 0.0, 65.0, 40.0, 0.0, 0.0, 50.0, 60.0, 0.0]
    d["triplet_1_8t"] = [50.0, 35.0, 40.0, 45.0, 30.0, 35.0, 55.0, 40.0, 35.0, 40.0, 45.0, 30.0]
    d["strength_values"] = create_strength_pattern("backbeat")
    d["root_note"] = 45
    d["notes"] = [note_to_dict(Note(45, 100)), note_to_dict(Note(48, 55)), note_to_dict(Note(52, 50)),
                  note_to_dict(Note(57, 45)), note_to_dict(Note(60, 40))]
    d["synth_osc_d"] = 0.32
    d["synth_osc_v"] = 0.62
    d["synth_osc_volume"] = 0.7
    d["synth_sub_volume"] = 0.22
    d["synth_filter_cutoff"] = 2800.0
    d["synth_filter_resonance"] = 0.15
    d["synth_filter_env_amount"] = 1500.0
    d["synth_vol_attack"] = 2.0
    d["synth_vol_decay"] = 160.0
    d["synth_vol_sustain"] = 0.45
    d["synth_vol_release"] = 220.0
    d["synth_reverb_mix"] = 0.18
    d["synth_reverb_decay"] = 0.5
    d["swing_amount"] = 52.0
    d["note_length_percent"] = 60.0
    presets.append(create_preset("Liquid DnB", "Factory",
        "Flowing fast rhythms - smooth melodic lines over rolling breaks", d))

    # 9. Dub Techno Echo - Dubby techno
    d = create_default_preset()
    d["straight_1_4"] = [100.0, 75.0, 90.0, 70.0]
    d["straight_1_8"] = [0.0, 50.0, 0.0, 45.0, 0.0, 55.0, 0.0, 40.0]
    d["dotted_1_8d"] = [60.0, 45.0, 50.0, 55.0, 40.0, 45.0]
    d["strength_values"] = create_strength_pattern("4_4_standard")
    d["root_note"] = 38
    d["notes"] = [note_to_dict(Note(38, 100)), note_to_dict(Note(45, 50)), note_to_dict(Note(50, 40))]
    d["synth_osc_d"] = 0.38
    d["synth_osc_v"] = 0.48
    d["synth_osc_volume"] = 0.65
    d["synth_sub_volume"] = 0.3
    d["synth_filter_cutoff"] = 1400.0
    d["synth_filter_resonance"] = 0.35
    d["synth_filter_env_amount"] = 600.0
    d["synth_vol_attack"] = 8.0
    d["synth_vol_decay"] = 350.0
    d["synth_vol_sustain"] = 0.55
    d["synth_vol_release"] = 450.0
    d["synth_reverb_mix"] = 0.2
    d["synth_reverb_decay"] = 0.7
    d["synth_reverb_pre_delay"] = 60.0
    d["synth_reverb_diffusion"] = 0.85
    d["note_length_percent"] = 95.0
    d["lfo1_tempo_sync"] = True
    d["lfo1_sync_division"] = 7
    d["lfo1_waveform"] = 0
    d["lfo1_dest1"] = 12
    d["lfo1_amount1"] = 0.15
    presets.append(create_preset("Dub Techno Echo", "Factory",
        "Hazy dub chords - filtered pads swim in cavernous reverb", d))

    # 10. Acid Squelch - 303-style bass
    d = create_default_preset()
    d["straight_1_16"] = [100.0, 65.0, 80.0, 55.0, 95.0, 70.0, 75.0, 60.0, 90.0, 65.0, 85.0, 50.0, 100.0, 60.0, 70.0, 75.0]
    d["strength_values"] = create_strength_pattern("driving")
    d["root_note"] = 36
    d["notes"] = [note_to_dict(Note(36, 100)), note_to_dict(Note(39, 50)), note_to_dict(Note(43, 45)),
                  note_to_dict(Note(48, 40))]
    d["synth_osc_d"] = 0.72
    d["synth_osc_v"] = 0.3
    d["synth_osc_volume"] = 0.75
    d["synth_filter_cutoff"] = 800.0
    d["synth_filter_resonance"] = 0.55
    d["synth_filter_env_amount"] = 4000.0
    d["synth_filt_decay"] = 150.0
    d["synth_filt_sustain"] = 0.2
    d["synth_vol_attack"] = 1.0
    d["synth_vol_decay"] = 100.0
    d["synth_vol_sustain"] = 0.5
    d["synth_vol_release"] = 120.0
    d["note_length_percent"] = 55.0
    d["len_mod_1_target"] = 0.8
    d["len_mod_1_amount"] = 150.0
    d["len_mod_1_prob"] = 0.4
    presets.append(create_preset("Acid Squelch", "Factory",
        "Classic 303 acid - resonant filter sweeps over driving bassline", d))

    # 11. Progressive Build - Building patterns
    d = create_default_preset()
    d["straight_1_4"] = [100.0, 80.0, 90.0, 85.0]
    d["straight_1_8"] = [0.0, 65.0, 0.0, 60.0, 0.0, 70.0, 0.0, 55.0]
    d["straight_1_16"] = [0.0, 45.0, 40.0, 0.0, 0.0, 50.0, 35.0, 0.0, 0.0, 55.0, 45.0, 0.0, 0.0, 40.0, 50.0, 0.0]
    d["strength_values"] = create_strength_pattern("4_4_standard")
    d["root_note"] = 45
    d["notes"] = [note_to_dict(Note(45, 100)), note_to_dict(Note(48, 55)), note_to_dict(Note(52, 50)),
                  note_to_dict(Note(57, 45)), note_to_dict(Note(60, 40))]
    d["synth_osc_d"] = 0.4
    d["synth_osc_v"] = 0.6
    d["synth_osc_stereo_v_offset"] = 0.08
    d["synth_osc_volume"] = 0.7
    d["synth_filter_cutoff"] = 2500.0
    d["synth_filter_resonance"] = 0.18
    d["synth_filter_env_amount"] = 1200.0
    d["synth_vol_attack"] = 5.0
    d["synth_vol_decay"] = 300.0
    d["synth_vol_sustain"] = 0.55
    d["synth_vol_release"] = 400.0
    d["synth_reverb_mix"] = 0.15
    d["synth_reverb_decay"] = 0.45
    d["note_length_percent"] = 85.0
    d["lfo1_tempo_sync"] = True
    d["lfo1_sync_division"] = 0
    d["lfo1_waveform"] = 1
    d["lfo1_dest1"] = 12
    d["lfo1_amount1"] = 0.12
    d["lfo2_tempo_sync"] = True
    d["lfo2_sync_division"] = 1
    d["lfo2_waveform"] = 0
    d["lfo2_dest1"] = 11
    d["lfo2_amount1"] = 0.08
    presets.append(create_preset("Progressive Build", "Factory",
        "Layered progression - patterns build density through the bar", d))

    # 12. Lo-Fi Tape - Dusty beats
    d = create_default_preset()
    d["straight_1_8"] = [100.0, 55.0, 75.0, 50.0, 90.0, 60.0, 70.0, 65.0]
    d["straight_1_16"] = [0.0, 40.0, 35.0, 0.0, 0.0, 45.0, 30.0, 0.0, 0.0, 50.0, 40.0, 0.0, 0.0, 35.0, 45.0, 0.0]
    d["strength_values"] = create_strength_pattern("backbeat")
    d["root_note"] = 48
    d["notes"] = [note_to_dict(Note(48, 100)), note_to_dict(Note(51, 50)), note_to_dict(Note(55, 55)),
                  note_to_dict(Note(58, 40))]
    d["synth_osc_d"] = 0.25
    d["synth_osc_v"] = 0.55
    d["synth_osc_volume"] = 0.65
    d["synth_filter_cutoff"] = 2200.0
    d["synth_filter_resonance"] = 0.12
    d["synth_filter_env_amount"] = 800.0
    d["synth_vol_attack"] = 3.0
    d["synth_vol_decay"] = 250.0
    d["synth_vol_sustain"] = 0.45
    d["synth_vol_release"] = 320.0
    d["synth_reverb_mix"] = 0.12
    d["synth_reverb_decay"] = 0.4
    d["swing_amount"] = 62.0
    d["note_length_percent"] = 75.0
    d["pos_mod_1_target"] = 0.0
    d["pos_mod_1_shift"] = 0.015
    d["pos_mod_1_prob"] = 0.4
    presets.append(create_preset("Lo-Fi Tape", "Factory",
        "Dusty tape vibes - swung beats with warm analog character", d))

    # 13. Future Pluck - Modern bass music
    d = create_default_preset()
    d["straight_1_8"] = [100.0, 60.0, 0.0, 85.0, 0.0, 75.0, 55.0, 90.0]
    d["straight_1_16"] = [0.0, 50.0, 65.0, 0.0, 45.0, 0.0, 55.0, 0.0, 60.0, 0.0, 40.0, 70.0, 0.0, 50.0, 0.0, 55.0]
    d["strength_values"] = create_strength_pattern("funk")
    d["root_note"] = 43
    d["notes"] = [note_to_dict(Note(43, 100)), note_to_dict(Note(48, 55)), note_to_dict(Note(50, 50)),
                  note_to_dict(Note(55, 45)), note_to_dict(Note(60, 35))]
    d["synth_osc_d"] = 0.58
    d["synth_osc_v"] = 0.45
    d["synth_osc_volume"] = 0.72
    d["synth_sub_volume"] = 0.2
    d["synth_filter_cutoff"] = 3500.0
    d["synth_filter_resonance"] = 0.2
    d["synth_filter_env_amount"] = 2000.0
    d["synth_vol_attack"] = 1.0
    d["synth_vol_decay"] = 120.0
    d["synth_vol_sustain"] = 0.3
    d["synth_vol_release"] = 150.0
    d["swing_amount"] = 54.0
    d["note_length_percent"] = 50.0
    presets.append(create_preset("Future Pluck", "Factory",
        "Sharp modern bass - punchy plucks with syncopated patterns", d))

    # 14. Trance Gate - Uplifting trance
    d = create_default_preset()
    d["straight_1_16"] = [100.0, 80.0, 90.0, 75.0, 95.0, 85.0, 88.0, 70.0, 100.0, 82.0, 92.0, 78.0, 96.0, 80.0, 85.0, 72.0]
    d["strength_values"] = create_strength_pattern("driving")
    d["root_note"] = 48
    d["notes"] = [note_to_dict(Note(48, 100)), note_to_dict(Note(52, 60)), note_to_dict(Note(55, 55)),
                  note_to_dict(Note(60, 50)), note_to_dict(Note(64, 45))]
    d["synth_osc_d"] = 0.3
    d["synth_osc_v"] = 0.72
    d["synth_osc_stereo_v_offset"] = 0.12
    d["synth_osc_volume"] = 0.7
    d["synth_filter_cutoff"] = 4000.0
    d["synth_filter_resonance"] = 0.15
    d["synth_filter_env_amount"] = 1500.0
    d["synth_vol_attack"] = 1.0
    d["synth_vol_decay"] = 150.0
    d["synth_vol_sustain"] = 0.4
    d["synth_vol_release"] = 200.0
    d["synth_reverb_mix"] = 0.22
    d["synth_reverb_decay"] = 0.55
    d["note_length_percent"] = 60.0
    presets.append(create_preset("Trance Gate", "Factory",
        "Euphoric gated patterns - driving 16ths build energy", d))

    # 15. Chillwave Haze - Dreamy slow
    d = create_default_preset()
    d["straight_1_4"] = [100.0, 65.0, 80.0, 60.0]
    d["straight_1_8"] = [0.0, 45.0, 0.0, 40.0, 0.0, 50.0, 0.0, 35.0]
    d["dotted_1_4d"] = [55.0, 40.0, 45.0]
    d["strength_values"] = create_strength_pattern("ambient")
    d["root_note"] = 52
    d["notes"] = [note_to_dict(Note(52, 100)), note_to_dict(Note(55, 55)), note_to_dict(Note(59, 50)),
                  note_to_dict(Note(64, 45)), note_to_dict(Note(67, 35))]
    d["synth_osc_d"] = 0.18
    d["synth_osc_v"] = 0.68
    d["synth_osc_stereo_v_offset"] = 0.15
    d["synth_osc_volume"] = 0.62
    d["synth_filter_cutoff"] = 2800.0
    d["synth_filter_resonance"] = 0.1
    d["synth_filter_env_amount"] = 600.0
    d["synth_vol_attack"] = 80.0
    d["synth_vol_decay"] = 600.0
    d["synth_vol_sustain"] = 0.55
    d["synth_vol_release"] = 800.0
    d["synth_reverb_mix"] = 0.22
    d["synth_reverb_decay"] = 0.7
    d["synth_reverb_diffusion"] = 0.85
    d["note_length_percent"] = 140.0
    d["lfo1_rate"] = 0.12
    d["lfo1_waveform"] = 0
    d["lfo1_dest1"] = 11
    d["lfo1_amount1"] = 0.12
    presets.append(create_preset("Chillwave Haze", "Factory",
        "Dreamy slow motion - hazy pads drift through soft reverb", d))

    # 16. Glitch Hop Funk - Funky glitches
    d = create_default_preset()
    d["straight_1_8"] = [100.0, 55.0, 70.0, 85.0, 60.0, 90.0, 50.0, 75.0]
    d["straight_1_16"] = [0.0, 65.0, 45.0, 0.0, 55.0, 0.0, 60.0, 40.0, 0.0, 50.0, 70.0, 0.0, 45.0, 0.0, 55.0, 65.0]
    d["strength_values"] = create_strength_pattern("funk")
    d["root_note"] = 43
    d["notes"] = [note_to_dict(Note(43, 100)), note_to_dict(Note(45, 50)), note_to_dict(Note(48, 55)),
                  note_to_dict(Note(52, 45)), note_to_dict(Note(55, 40))]
    d["synth_osc_d"] = 0.55
    d["synth_osc_v"] = 0.5
    d["synth_osc_volume"] = 0.72
    d["synth_sub_volume"] = 0.18
    d["synth_filter_cutoff"] = 2600.0
    d["synth_filter_resonance"] = 0.22
    d["synth_filter_env_amount"] = 1600.0
    d["synth_vol_attack"] = 1.0
    d["synth_vol_decay"] = 140.0
    d["synth_vol_sustain"] = 0.4
    d["synth_vol_release"] = 180.0
    d["swing_amount"] = 56.0
    d["note_length_percent"] = 55.0
    d["len_mod_1_target"] = -0.6
    d["len_mod_1_amount"] = 60.0
    d["len_mod_1_prob"] = 0.45
    d["pos_mod_1_target"] = -0.5
    d["pos_mod_1_shift"] = 0.02
    d["pos_mod_1_prob"] = 0.35
    presets.append(create_preset("Glitch Hop Funk", "Factory",
        "Funky micro-edits - syncopated bass with stuttered accents", d))

    # 17. Acid Squelch - 303 worship
    d = create_default_preset()
    d["straight_1_16"] = [100.0, 70.0, 85.0, 60.0, 95.0, 65.0, 80.0, 55.0, 90.0, 68.0, 82.0, 58.0, 88.0, 62.0, 78.0, 52.0]
    d["strength_values"] = create_strength_pattern("4_4_standard")
    d["root_note"] = 36
    d["scale"] = "Minor"
    d["notes"] = [note_to_dict(Note(36, 127)), note_to_dict(Note(39, 80)), note_to_dict(Note(43, 70)), note_to_dict(Note(48, 60))]
    d["synth_osc_d"] = 0.55
    d["synth_osc_v"] = 0.45
    d["synth_osc_volume"] = 0.75
    d["synth_filter_cutoff"] = 600.0
    d["synth_filter_resonance"] = 0.78
    d["synth_filter_env_amount"] = 3500.0
    d["synth_filt_decay"] = 120.0
    d["synth_filt_sustain"] = 0.15
    d["synth_vol_attack"] = 1.0
    d["synth_vol_decay"] = 100.0
    d["synth_vol_sustain"] = 0.55
    d["synth_tube_drive"] = 2.2
    d["synth_reverb_mix"] = 0.08
    d["swing_amount"] = 52.0
    presets.append(create_preset("Acid Squelch", "Factory",
        "Classic 303 acid - high resonance filter sweeps with tube warmth", d))

    # 18. Ambient Drone - Vast textures
    d = create_default_preset()
    d["straight_1_1"] = [90.0]
    d["straight_1_2"] = [50.0, 40.0]
    d["strength_values"] = create_strength_pattern("sparse")
    d["root_note"] = 36
    d["scale"] = "Minor"
    d["stability_pattern"] = "Ambient"
    d["notes"] = [note_to_dict(Note(36, 127)), note_to_dict(Note(43, 90)), note_to_dict(Note(48, 80)), note_to_dict(Note(55, 60))]
    d["synth_osc_d"] = 0.15
    d["synth_osc_v"] = 0.5
    d["synth_osc_stereo_v_offset"] = 0.12
    d["synth_osc_volume"] = 0.55
    d["synth_pll_volume"] = 0.4
    d["synth_pll_track_speed"] = 0.25
    d["synth_pll_damping"] = 0.15
    d["synth_filter_cutoff"] = 1200.0
    d["synth_vol_attack"] = 600.0
    d["synth_vol_decay"] = 2000.0
    d["synth_vol_sustain"] = 0.7
    d["synth_vol_release"] = 3000.0
    d["synth_drift_amount"] = 0.18
    d["synth_drift_rate"] = 0.25
    d["synth_reverb_mix"] = 0.35
    d["synth_reverb_decay"] = 0.8
    d["note_length_percent"] = 250.0
    presets.append(create_preset("Ambient Drone", "Factory",
        "Vast atmospheric texture - slow evolving drones with pitch drift", d))

    # 19. Trance Lead - Soaring anthem
    d = create_default_preset()
    d["straight_1_8"] = [100.0, 70.0, 85.0, 60.0, 95.0, 65.0, 80.0, 55.0]
    d["straight_1_16"] = [0.0, 50.0, 40.0, 0.0, 0.0, 45.0, 35.0, 0.0, 0.0, 48.0, 38.0, 0.0, 0.0, 42.0, 32.0, 0.0]
    d["strength_values"] = create_strength_pattern("4_4_standard")
    d["root_note"] = 60
    d["scale"] = "Minor"
    d["notes"] = [note_to_dict(Note(60, 127)), note_to_dict(Note(63, 90)), note_to_dict(Note(67, 95)), note_to_dict(Note(70, 80))]
    d["synth_osc_d"] = 0.4
    d["synth_osc_v"] = 0.55
    d["synth_osc_volume"] = 0.65
    d["synth_pll_volume"] = 0.35
    d["synth_pll_track_speed"] = 0.5
    d["synth_filter_cutoff"] = 3500.0
    d["synth_filter_resonance"] = 0.2
    d["synth_vol_attack"] = 10.0
    d["synth_vol_decay"] = 200.0
    d["synth_vol_sustain"] = 0.6
    d["synth_vol_release"] = 350.0
    d["synth_reverb_mix"] = 0.22
    d["synth_reverb_decay"] = 0.6
    d["lfo1_rate"] = 5.5
    d["lfo1_dest1"] = 11
    d["lfo1_amount1"] = 0.06
    presets.append(create_preset("Trance Lead", "Factory",
        "Soaring anthem lead - bright minor key with gentle vibrato", d))

    # 20. Industrial Noise - Harsh textures
    d = create_default_preset()
    d["straight_1_8"] = [100.0, 85.0, 95.0, 80.0, 98.0, 82.0, 92.0, 78.0]
    d["straight_1_16"] = [70.0, 55.0, 65.0, 50.0, 68.0, 52.0, 62.0, 48.0, 72.0, 58.0, 68.0, 54.0, 66.0, 50.0, 60.0, 45.0]
    d["strength_values"] = create_strength_pattern("industrial")
    d["root_note"] = 36
    d["scale"] = "Chromatic"
    d["notes"] = [note_to_dict(Note(36, 127)), note_to_dict(Note(37, 60)), note_to_dict(Note(42, 70)), note_to_dict(Note(43, 80))]
    d["synth_osc_d"] = 0.7
    d["synth_osc_v"] = 0.3
    d["synth_osc_volume"] = 0.6
    d["synth_pll_volume"] = 0.4
    d["synth_pll_track_speed"] = 0.85
    d["synth_pll_feedback"] = 0.25
    d["synth_pll_burst_amount"] = 2.5
    d["synth_noise_amount"] = 0.3
    d["synth_filter_cutoff"] = 2000.0
    d["synth_filter_drive"] = 4.0
    d["synth_color_distortion_amount"] = 0.5
    d["synth_vol_attack"] = 1.0
    d["synth_vol_decay"] = 80.0
    d["synth_vol_sustain"] = 0.5
    d["synth_reverb_mix"] = 0.12
    presets.append(create_preset("Industrial Noise", "Factory",
        "Harsh mechanical - distorted rhythms with noise injection", d))

    # 21. Lo-Fi Beats - Dusty hip-hop
    d = create_default_preset()
    d["straight_1_4"] = [100.0, 75.0, 90.0, 70.0]
    d["straight_1_8"] = [60.0, 40.0, 55.0, 35.0, 58.0, 38.0, 52.0, 32.0]
    d["strength_values"] = create_strength_pattern("hip_hop")
    d["root_note"] = 41
    d["scale"] = "PentatonicMinor"
    d["notes"] = [note_to_dict(Note(41, 127)), note_to_dict(Note(44, 80)), note_to_dict(Note(46, 85)), note_to_dict(Note(48, 70)), note_to_dict(Note(51, 60))]
    d["synth_osc_d"] = 0.35
    d["synth_osc_v"] = 0.55
    d["synth_osc_volume"] = 0.65
    d["synth_filter_cutoff"] = 2200.0
    d["synth_filter_resonance"] = 0.12
    d["synth_vol_attack"] = 5.0
    d["synth_vol_decay"] = 180.0
    d["synth_vol_sustain"] = 0.45
    d["synth_vol_release"] = 250.0
    d["synth_reverb_mix"] = 0.15
    d["synth_reverb_lpf"] = 4000.0
    d["swing_amount"] = 58.0
    d["note_length_percent"] = 75.0
    presets.append(create_preset("Lo-Fi Beats", "Factory",
        "Dusty hip-hop vibes - warm pentatonic melodies with lazy swing", d))

    # 22. Synthwave Bass - Retro drive
    d = create_default_preset()
    d["straight_1_8"] = [100.0, 60.0, 80.0, 50.0, 90.0, 55.0, 75.0, 45.0]
    d["strength_values"] = create_strength_pattern("4_4_standard")
    d["root_note"] = 36
    d["scale"] = "Minor"
    d["notes"] = [note_to_dict(Note(36, 127)), note_to_dict(Note(39, 80)), note_to_dict(Note(43, 90)), note_to_dict(Note(46, 70))]
    d["synth_osc_d"] = 0.5
    d["synth_osc_v"] = 0.5
    d["synth_osc_volume"] = 0.7
    d["synth_sub_volume"] = 0.35
    d["synth_filter_cutoff"] = 1400.0
    d["synth_filter_resonance"] = 0.3
    d["synth_filter_env_amount"] = 1800.0
    d["synth_vol_attack"] = 2.0
    d["synth_vol_decay"] = 120.0
    d["synth_vol_sustain"] = 0.5
    d["synth_tube_drive"] = 1.8
    d["synth_reverb_mix"] = 0.15
    presets.append(create_preset("Synthwave Bass", "Factory",
        "80s retro bass - punchy filtered synth with sub weight", d))

    # 23. Breakcore Chaos - Frantic edits
    d = create_default_preset()
    d["straight_1_16"] = [95.0, 70.0, 85.0, 60.0, 90.0, 65.0, 80.0, 55.0, 92.0, 68.0, 82.0, 58.0, 88.0, 62.0, 78.0, 52.0]
    d["straight_1_32"] = [60.0, 40.0, 55.0, 35.0, 58.0, 38.0, 52.0, 32.0, 62.0, 42.0, 57.0, 37.0, 55.0, 35.0, 50.0, 30.0,
                          58.0, 38.0, 53.0, 33.0, 56.0, 36.0, 51.0, 31.0, 60.0, 40.0, 55.0, 35.0, 54.0, 34.0, 49.0, 29.0]
    d["strength_values"] = create_strength_pattern("dense")
    d["root_note"] = 48
    d["scale"] = "Chromatic"
    d["notes"] = [note_to_dict(Note(48, 127)), note_to_dict(Note(49, 60)), note_to_dict(Note(51, 70)), note_to_dict(Note(54, 65)), note_to_dict(Note(55, 80))]
    d["synth_osc_d"] = 0.6
    d["synth_osc_v"] = 0.4
    d["synth_osc_volume"] = 0.7
    d["synth_pll_volume"] = 0.3
    d["synth_pll_track_speed"] = 0.75
    d["synth_filter_cutoff"] = 3000.0
    d["synth_filter_env_amount"] = 2000.0
    d["synth_vol_attack"] = 0.5
    d["synth_vol_decay"] = 50.0
    d["synth_vol_sustain"] = 0.3
    d["synth_reverb_mix"] = 0.1
    d["note_length_percent"] = 35.0
    presets.append(create_preset("Breakcore Chaos", "Factory",
        "Frantic edits - rapid-fire chromatic bursts at maximum density", d))

    # 24. Downtempo Pad - Slow and lush
    d = create_default_preset()
    d["straight_1_2"] = [90.0, 70.0]
    d["straight_1_4"] = [60.0, 45.0, 55.0, 40.0]
    d["strength_values"] = create_strength_pattern("sparse")
    d["root_note"] = 48
    d["scale"] = "Dorian"
    d["stability_pattern"] = "Ambient"
    d["notes"] = [note_to_dict(Note(48, 127)), note_to_dict(Note(50, 85)), note_to_dict(Note(51, 80)), note_to_dict(Note(55, 95)), note_to_dict(Note(57, 70))]
    d["synth_osc_d"] = 0.2
    d["synth_osc_v"] = 0.55
    d["synth_osc_stereo_v_offset"] = 0.1
    d["synth_osc_volume"] = 0.6
    d["synth_pll_volume"] = 0.35
    d["synth_filter_cutoff"] = 2000.0
    d["synth_vol_attack"] = 200.0
    d["synth_vol_decay"] = 800.0
    d["synth_vol_sustain"] = 0.65
    d["synth_vol_release"] = 1200.0
    d["synth_reverb_mix"] = 0.28
    d["synth_reverb_decay"] = 0.7
    d["note_length_percent"] = 160.0
    presets.append(create_preset("Downtempo Pad", "Factory",
        "Slow lush texture - Dorian warmth with long sustaining notes", d))

    # 25. Nu Disco Groove - Funky modern
    d = create_default_preset()
    d["straight_1_4"] = [100.0, 85.0, 95.0, 80.0]
    d["straight_1_8"] = [70.0, 55.0, 68.0, 50.0, 72.0, 58.0, 65.0, 48.0]
    d["straight_1_16"] = [0.0, 45.0, 40.0, 0.0, 0.0, 48.0, 35.0, 0.0, 0.0, 42.0, 38.0, 0.0, 0.0, 50.0, 32.0, 0.0]
    d["strength_values"] = create_strength_pattern("disco")
    d["root_note"] = 43
    d["scale"] = "Dorian"
    d["notes"] = [note_to_dict(Note(43, 127)), note_to_dict(Note(45, 75)), note_to_dict(Note(47, 85)), note_to_dict(Note(50, 90)), note_to_dict(Note(52, 70))]
    d["synth_osc_d"] = 0.45
    d["synth_osc_v"] = 0.55
    d["synth_osc_volume"] = 0.7
    d["synth_sub_volume"] = 0.25
    d["synth_filter_cutoff"] = 2800.0
    d["synth_filter_resonance"] = 0.18
    d["synth_filter_env_amount"] = 1200.0
    d["synth_vol_attack"] = 2.0
    d["synth_vol_decay"] = 140.0
    d["synth_vol_sustain"] = 0.5
    d["synth_reverb_mix"] = 0.12
    d["swing_amount"] = 54.0
    d["note_length_percent"] = 70.0
    presets.append(create_preset("Nu Disco Groove", "Factory",
        "Modern funky bass - Dorian grooves with disco energy", d))

    # 26. Psytrance Riff - Driving triplets
    d = create_default_preset()
    d["straight_1_16"] = [100.0, 75.0, 90.0, 65.0, 95.0, 70.0, 85.0, 60.0, 98.0, 72.0, 88.0, 62.0, 92.0, 68.0, 82.0, 58.0]
    d["triplet_1_16t"] = [70.0, 50.0, 60.0, 65.0, 48.0, 58.0, 68.0, 52.0, 62.0, 66.0, 50.0, 56.0, 64.0, 46.0, 54.0, 62.0, 48.0, 58.0, 60.0, 44.0, 52.0, 58.0, 42.0, 50.0]
    d["strength_values"] = create_strength_pattern("4_4_standard")
    d["root_note"] = 45
    d["scale"] = "Phrygian"
    d["notes"] = [note_to_dict(Note(45, 127)), note_to_dict(Note(46, 70)), note_to_dict(Note(48, 85)), note_to_dict(Note(50, 80)), note_to_dict(Note(52, 75))]
    d["synth_osc_d"] = 0.55
    d["synth_osc_v"] = 0.45
    d["synth_osc_volume"] = 0.7
    d["synth_filter_cutoff"] = 2200.0
    d["synth_filter_resonance"] = 0.35
    d["synth_filter_env_amount"] = 1500.0
    d["synth_vol_attack"] = 1.0
    d["synth_vol_decay"] = 80.0
    d["synth_vol_sustain"] = 0.4
    d["synth_tube_drive"] = 1.5
    d["synth_reverb_mix"] = 0.1
    d["note_length_percent"] = 50.0
    presets.append(create_preset("Psytrance Riff", "Factory",
        "Driving Phrygian patterns - rolling triplets with acidic filter", d))

    # 27. Garage 2-Step - UK shuffle
    d = create_default_preset()
    d["straight_1_4"] = [100.0, 70.0, 85.0, 65.0]
    d["straight_1_8"] = [0.0, 80.0, 0.0, 55.0, 0.0, 75.0, 0.0, 50.0]
    d["triplet_1_8t"] = [0.0, 60.0, 50.0, 0.0, 55.0, 45.0, 0.0, 58.0, 48.0, 0.0, 52.0, 42.0]
    d["strength_values"] = create_strength_pattern("shuffle")
    d["root_note"] = 48
    d["scale"] = "Minor"
    d["notes"] = [note_to_dict(Note(48, 127)), note_to_dict(Note(51, 85)), note_to_dict(Note(55, 90)), note_to_dict(Note(58, 75))]
    d["synth_osc_d"] = 0.38
    d["synth_osc_v"] = 0.52
    d["synth_osc_volume"] = 0.68
    d["synth_filter_cutoff"] = 2600.0
    d["synth_filter_resonance"] = 0.15
    d["synth_vol_attack"] = 3.0
    d["synth_vol_decay"] = 120.0
    d["synth_vol_sustain"] = 0.45
    d["synth_vol_release"] = 180.0
    d["synth_reverb_mix"] = 0.18
    d["swing_amount"] = 60.0
    d["note_length_percent"] = 60.0
    presets.append(create_preset("Garage 2-Step", "Factory",
        "UK garage shuffle - skippy rhythms with soulful minor key", d))

    # 28. Hardstyle Kick - Punishing bass
    d = create_default_preset()
    d["straight_1_4"] = [127.0, 127.0, 127.0, 127.0]
    d["straight_1_8"] = [0.0, 60.0, 0.0, 55.0, 0.0, 65.0, 0.0, 50.0]
    d["strength_values"] = create_strength_pattern("4_4_standard")
    d["root_note"] = 36
    d["notes"] = [note_to_dict(Note(36, 127)), note_to_dict(Note(43, 50))]
    d["synth_osc_d"] = 0.65
    d["synth_osc_v"] = 0.35
    d["synth_osc_volume"] = 0.8
    d["synth_sub_volume"] = 0.5
    d["synth_filter_cutoff"] = 400.0
    d["synth_filter_env_amount"] = 3500.0
    d["synth_filter_drive"] = 3.0
    d["synth_color_distortion_amount"] = 0.4
    d["synth_vol_attack"] = 0.5
    d["synth_vol_decay"] = 150.0
    d["synth_vol_sustain"] = 0.0
    d["synth_filt_decay"] = 50.0
    d["synth_reverb_mix"] = 0.05
    presets.append(create_preset("Hardstyle Kick", "Factory",
        "Punishing four-on-floor - massive distorted kick with tail", d))

    # 29. Minimal Techno - Hypnotic loop
    d = create_default_preset()
    d["straight_1_8"] = [100.0, 55.0, 70.0, 45.0, 90.0, 50.0, 65.0, 40.0]
    d["straight_1_16"] = [0.0, 40.0, 35.0, 0.0, 0.0, 45.0, 30.0, 0.0, 0.0, 38.0, 33.0, 0.0, 0.0, 42.0, 28.0, 0.0]
    d["strength_values"] = create_strength_pattern("4_4_standard")
    d["root_note"] = 48
    d["scale"] = "Minor"
    d["notes"] = [note_to_dict(Note(48, 127)), note_to_dict(Note(51, 70)), note_to_dict(Note(55, 80))]
    d["synth_osc_d"] = 0.42
    d["synth_osc_v"] = 0.48
    d["synth_osc_volume"] = 0.65
    d["synth_filter_cutoff"] = 1800.0
    d["synth_filter_resonance"] = 0.28
    d["synth_vol_attack"] = 2.0
    d["synth_vol_decay"] = 100.0
    d["synth_vol_sustain"] = 0.4
    d["synth_reverb_mix"] = 0.15
    d["note_length_percent"] = 55.0
    d["lfo1_tempo_sync"] = True
    d["lfo1_sync_division"] = 1
    d["lfo1_waveform"] = 2
    d["lfo1_dest1"] = 12
    d["lfo1_amount1"] = 0.12
    presets.append(create_preset("Minimal Techno", "Factory",
        "Hypnotic loop - sparse minor key with synced filter sweep", d))

    # 30. IDM Glitch - Algorithmic complexity
    d = create_default_preset()
    d["straight_1_16"] = [85.0, 55.0, 70.0, 80.0, 60.0, 90.0, 50.0, 75.0, 82.0, 58.0, 72.0, 65.0, 88.0, 52.0, 68.0, 78.0]
    d["triplet_1_8t"] = [60.0, 40.0, 50.0, 55.0, 45.0, 62.0, 38.0, 58.0, 48.0, 65.0, 42.0, 52.0]
    d["strength_values"] = create_strength_pattern("dense")
    d["root_note"] = 48
    d["scale"] = "WholeTone"
    d["notes"] = [note_to_dict(Note(48, 127)), note_to_dict(Note(50, 70)), note_to_dict(Note(52, 75)), note_to_dict(Note(54, 80)), note_to_dict(Note(56, 65))]
    d["synth_osc_d"] = 0.5
    d["synth_osc_v"] = 0.5
    d["synth_osc_volume"] = 0.6
    d["synth_pll_volume"] = 0.35
    d["synth_pll_track_speed"] = 0.6
    d["synth_pll_fm_amount"] = 0.15
    d["synth_filter_cutoff"] = 3200.0
    d["synth_filter_env_amount"] = 1500.0
    d["synth_vol_attack"] = 2.0
    d["synth_vol_decay"] = 120.0
    d["synth_vol_sustain"] = 0.35
    d["synth_reverb_mix"] = 0.18
    d["note_length_percent"] = 45.0
    d["len_mod_1_target"] = -0.5
    d["len_mod_1_amount"] = 50.0
    d["len_mod_1_prob"] = 0.4
    presets.append(create_preset("IDM Glitch", "Factory",
        "Algorithmic complexity - whole tone FM with variable lengths", d))

    # 31. Electroclash Stab - Punky synth
    d = create_default_preset()
    d["straight_1_4"] = [100.0, 90.0, 95.0, 85.0]
    d["straight_1_8"] = [70.0, 55.0, 65.0, 50.0, 72.0, 58.0, 68.0, 52.0]
    d["strength_values"] = create_strength_pattern("punk")
    d["root_note"] = 48
    d["scale"] = "Minor"
    d["notes"] = [note_to_dict(Note(48, 127)), note_to_dict(Note(51, 90)), note_to_dict(Note(55, 95)), note_to_dict(Note(58, 80))]
    d["synth_osc_d"] = 0.55
    d["synth_osc_v"] = 0.45
    d["synth_osc_volume"] = 0.72
    d["synth_filter_cutoff"] = 2400.0
    d["synth_filter_resonance"] = 0.22
    d["synth_filter_env_amount"] = 1200.0
    d["synth_vol_attack"] = 1.0
    d["synth_vol_decay"] = 80.0
    d["synth_vol_sustain"] = 0.45
    d["synth_tube_drive"] = 2.0
    d["synth_reverb_mix"] = 0.12
    d["note_length_percent"] = 50.0
    presets.append(create_preset("Electroclash Stab", "Factory",
        "Punky synth stabs - raw minor chords with attitude", d))

    # 32. Future Bass - Pitched chords
    d = create_default_preset()
    d["straight_1_4"] = [100.0, 75.0, 90.0, 70.0]
    d["straight_1_8"] = [65.0, 45.0, 60.0, 40.0, 68.0, 48.0, 62.0, 42.0]
    d["strength_values"] = create_strength_pattern("half_time")
    d["root_note"] = 48
    d["scale"] = "Lydian"
    d["notes"] = [note_to_dict(Note(48, 127)), note_to_dict(Note(52, 95)), note_to_dict(Note(55, 100)), note_to_dict(Note(59, 85)), note_to_dict(Note(60, 75))]
    d["synth_osc_d"] = 0.3
    d["synth_osc_v"] = 0.55
    d["synth_osc_stereo_v_offset"] = 0.08
    d["synth_osc_volume"] = 0.65
    d["synth_pll_volume"] = 0.3
    d["synth_filter_cutoff"] = 3500.0
    d["synth_filter_resonance"] = 0.12
    d["synth_vol_attack"] = 8.0
    d["synth_vol_decay"] = 200.0
    d["synth_vol_sustain"] = 0.55
    d["synth_vol_release"] = 350.0
    d["synth_reverb_mix"] = 0.22
    d["synth_reverb_decay"] = 0.6
    d["note_length_percent"] = 80.0
    d["lfo1_rate"] = 4.0
    d["lfo1_dest1"] = 11
    d["lfo1_amount1"] = 0.05
    presets.append(create_preset("Future Bass", "Factory",
        "Bright Lydian chords - modern electronic with pitched textures", d))

    return presets

def create_bank_c() -> List[Dict]:
    """Bank C: Classic Genres - 32 presets"""
    presets = []

    # 1. Jazz Walk - Walking bass feel
    d = create_default_preset()
    d["straight_1_4"] = [100.0, 85.0, 90.0, 80.0]
    d["straight_1_8"] = [0.0, 50.0, 0.0, 45.0, 0.0, 55.0, 0.0, 40.0]
    d["triplet_1_4t"] = [60.0, 45.0, 50.0, 55.0, 40.0, 45.0]
    d["strength_values"] = create_strength_pattern("jazz")
    d["root_note"] = 36
    d["notes"] = [note_to_dict(Note(36, 100)), note_to_dict(Note(38, 55)), note_to_dict(Note(40, 50)),
                  note_to_dict(Note(43, 60)), note_to_dict(Note(45, 45))]
    d["synth_osc_d"] = 0.22
    d["synth_osc_v"] = 0.45
    d["synth_osc_volume"] = 0.68
    d["synth_sub_volume"] = 0.32
    d["synth_filter_cutoff"] = 1600.0
    d["synth_filter_resonance"] = 0.15
    d["synth_filter_env_amount"] = 500.0
    d["synth_vol_attack"] = 3.0
    d["synth_vol_decay"] = 280.0
    d["synth_vol_sustain"] = 0.55
    d["synth_vol_release"] = 350.0
    d["swing_amount"] = 60.0
    d["note_length_percent"] = 85.0
    d["pos_mod_1_target"] = -0.4
    d["pos_mod_1_shift"] = 0.018
    d["pos_mod_1_prob"] = 0.3
    presets.append(create_preset("Jazz Walk", "Factory",
        "Walking bass lines - swung quarter notes stroll through changes", d))

    # 2. Blues Shuffle - 12-bar feel
    d = create_default_preset()
    d["triplet_1_4t"] = [100.0, 0.0, 75.0, 95.0, 0.0, 70.0]
    d["triplet_1_8t"] = [0.0, 50.0, 45.0, 0.0, 55.0, 40.0, 0.0, 45.0, 50.0, 0.0, 40.0, 55.0]
    d["strength_values"] = create_strength_pattern("shuffle")
    d["root_note"] = 40
    d["notes"] = [note_to_dict(Note(40, 100)), note_to_dict(Note(43, 55)), note_to_dict(Note(45, 50)),
                  note_to_dict(Note(46, 40)), note_to_dict(Note(47, 45))]
    d["synth_osc_d"] = 0.35
    d["synth_osc_v"] = 0.5
    d["synth_osc_volume"] = 0.7
    d["synth_sub_volume"] = 0.25
    d["synth_filter_cutoff"] = 2000.0
    d["synth_filter_resonance"] = 0.18
    d["synth_filter_env_amount"] = 800.0
    d["synth_vol_attack"] = 2.0
    d["synth_vol_decay"] = 200.0
    d["synth_vol_sustain"] = 0.5
    d["synth_vol_release"] = 280.0
    d["swing_amount"] = 66.0
    d["note_length_percent"] = 70.0
    presets.append(create_preset("Blues Shuffle", "Factory",
        "Classic 12-bar feel - triplet shuffle with blue notes", d))

    # 3. Rock Solid - Driving rock
    d = create_default_preset()
    d["straight_1_4"] = [100.0, 90.0, 95.0, 85.0]
    d["straight_1_8"] = [0.0, 70.0, 0.0, 65.0, 0.0, 75.0, 0.0, 60.0]
    d["strength_values"] = create_strength_pattern("4_4_standard")
    d["root_note"] = 40
    d["notes"] = [note_to_dict(Note(40, 100)), note_to_dict(Note(45, 55)), note_to_dict(Note(47, 50)),
                  note_to_dict(Note(52, 40))]
    d["synth_osc_d"] = 0.48
    d["synth_osc_v"] = 0.42
    d["synth_osc_volume"] = 0.72
    d["synth_sub_volume"] = 0.3
    d["synth_filter_cutoff"] = 1800.0
    d["synth_filter_resonance"] = 0.2
    d["synth_filter_env_amount"] = 600.0
    d["synth_vol_attack"] = 2.0
    d["synth_vol_decay"] = 180.0
    d["synth_vol_sustain"] = 0.55
    d["synth_vol_release"] = 220.0
    d["note_length_percent"] = 75.0
    presets.append(create_preset("Rock Solid", "Factory",
        "Driving rock foundation - solid downbeats anchor the groove", d))

    # 4. Funk Machine - Tight syncopation
    d = create_default_preset()
    d["straight_1_16"] = [100.0, 0.0, 65.0, 80.0, 0.0, 70.0, 55.0, 0.0, 90.0, 0.0, 60.0, 75.0, 0.0, 85.0, 50.0, 0.0]
    d["strength_values"] = create_strength_pattern("funk")
    d["root_note"] = 36
    d["notes"] = [note_to_dict(Note(36, 100)), note_to_dict(Note(38, 50)), note_to_dict(Note(41, 55)),
                  note_to_dict(Note(43, 45)), note_to_dict(Note(48, 35))]
    d["synth_osc_d"] = 0.55
    d["synth_osc_v"] = 0.48
    d["synth_osc_volume"] = 0.72
    d["synth_sub_volume"] = 0.22
    d["synth_filter_cutoff"] = 2400.0
    d["synth_filter_resonance"] = 0.25
    d["synth_filter_env_amount"] = 1400.0
    d["synth_vol_attack"] = 1.0
    d["synth_vol_decay"] = 120.0
    d["synth_vol_sustain"] = 0.35
    d["synth_vol_release"] = 150.0
    d["swing_amount"] = 54.0
    d["note_length_percent"] = 45.0
    d["len_mod_1_target"] = 0.8
    d["len_mod_1_amount"] = 140.0
    d["len_mod_1_prob"] = 0.4
    presets.append(create_preset("Funk Machine", "Factory",
        "Tight funk bass - syncopated 16ths with percussive attack", d))

    # 5. Motown Soul - Classic soul groove
    d = create_default_preset()
    d["straight_1_8"] = [100.0, 60.0, 80.0, 55.0, 90.0, 65.0, 75.0, 70.0]
    d["straight_1_16"] = [0.0, 45.0, 0.0, 40.0, 0.0, 50.0, 0.0, 35.0, 0.0, 55.0, 0.0, 45.0, 0.0, 40.0, 0.0, 50.0]
    d["strength_values"] = create_strength_pattern("backbeat")
    d["root_note"] = 41
    d["notes"] = [note_to_dict(Note(41, 100)), note_to_dict(Note(43, 55)), note_to_dict(Note(45, 50)),
                  note_to_dict(Note(48, 45)), note_to_dict(Note(53, 35))]
    d["synth_osc_d"] = 0.28
    d["synth_osc_v"] = 0.52
    d["synth_osc_volume"] = 0.68
    d["synth_sub_volume"] = 0.28
    d["synth_filter_cutoff"] = 2200.0
    d["synth_filter_resonance"] = 0.15
    d["synth_filter_env_amount"] = 900.0
    d["synth_vol_attack"] = 3.0
    d["synth_vol_decay"] = 220.0
    d["synth_vol_sustain"] = 0.5
    d["synth_vol_release"] = 300.0
    d["swing_amount"] = 56.0
    d["note_length_percent"] = 80.0
    presets.append(create_preset("Motown Soul", "Factory",
        "Classic Detroit soul - warm bass with soulful groove", d))

    # 6. Disco Fever - Four-on-floor disco
    d = create_default_preset()
    d["straight_1_4"] = [100.0, 100.0, 100.0, 100.0]
    d["straight_1_8"] = [0.0, 85.0, 0.0, 80.0, 0.0, 90.0, 0.0, 75.0]
    d["straight_1_16"] = [0.0, 55.0, 60.0, 0.0, 0.0, 50.0, 65.0, 0.0, 0.0, 60.0, 55.0, 0.0, 0.0, 45.0, 70.0, 0.0]
    d["strength_values"] = create_strength_pattern("driving")
    d["root_note"] = 43
    d["notes"] = [note_to_dict(Note(43, 100)), note_to_dict(Note(45, 50)), note_to_dict(Note(48, 55)),
                  note_to_dict(Note(50, 45)), note_to_dict(Note(55, 40))]
    d["synth_osc_d"] = 0.42
    d["synth_osc_v"] = 0.58
    d["synth_osc_volume"] = 0.7
    d["synth_sub_volume"] = 0.25
    d["synth_filter_cutoff"] = 2800.0
    d["synth_filter_resonance"] = 0.18
    d["synth_filter_env_amount"] = 1200.0
    d["synth_vol_attack"] = 2.0
    d["synth_vol_decay"] = 160.0
    d["synth_vol_sustain"] = 0.45
    d["synth_vol_release"] = 200.0
    d["note_length_percent"] = 65.0
    presets.append(create_preset("Disco Fever", "Factory",
        "Classic disco bass - four-on-floor drive with octave jumps", d))

    # 7. New Wave Chop - 80s angular style
    d = create_default_preset()
    d["straight_1_8"] = [100.0, 70.0, 0.0, 85.0, 75.0, 0.0, 90.0, 65.0]
    d["straight_1_16"] = [0.0, 55.0, 45.0, 0.0, 0.0, 60.0, 0.0, 50.0, 0.0, 45.0, 55.0, 0.0, 0.0, 65.0, 0.0, 40.0]
    d["strength_values"] = create_strength_pattern("offbeat")
    d["root_note"] = 45
    d["notes"] = [note_to_dict(Note(45, 100)), note_to_dict(Note(48, 55)), note_to_dict(Note(52, 50)),
                  note_to_dict(Note(57, 40))]
    d["synth_osc_d"] = 0.6
    d["synth_osc_v"] = 0.4
    d["synth_osc_volume"] = 0.72
    d["synth_filter_cutoff"] = 3200.0
    d["synth_filter_resonance"] = 0.22
    d["synth_filter_env_amount"] = 1600.0
    d["synth_vol_attack"] = 1.0
    d["synth_vol_decay"] = 140.0
    d["synth_vol_sustain"] = 0.35
    d["synth_vol_release"] = 180.0
    d["note_length_percent"] = 50.0
    presets.append(create_preset("New Wave Chop", "Factory",
        "Angular 80s style - choppy rhythms with bright attack", d))

    # 8. Punk Drive - Fast aggressive
    d = create_default_preset()
    d["straight_1_8"] = [100.0, 90.0, 95.0, 85.0, 100.0, 88.0, 92.0, 82.0]
    d["strength_values"] = create_strength_pattern("driving")
    d["root_note"] = 40
    d["notes"] = [note_to_dict(Note(40, 100)), note_to_dict(Note(43, 50)), note_to_dict(Note(45, 45)),
                  note_to_dict(Note(47, 40))]
    d["synth_osc_d"] = 0.65
    d["synth_osc_v"] = 0.35
    d["synth_osc_volume"] = 0.75
    d["synth_distortion_amount"] = 0.15
    d["synth_filter_cutoff"] = 2000.0
    d["synth_filter_resonance"] = 0.15
    d["synth_filter_env_amount"] = 400.0
    d["synth_vol_attack"] = 1.0
    d["synth_vol_decay"] = 100.0
    d["synth_vol_sustain"] = 0.6
    d["synth_vol_release"] = 120.0
    d["note_length_percent"] = 60.0
    presets.append(create_preset("Punk Drive", "Factory",
        "Fast aggressive energy - relentless eighth notes with grit", d))

    # 9. R&B Smooth - Slow sensual groove
    d = create_default_preset()
    d["straight_1_4"] = [100.0, 70.0, 85.0, 65.0]
    d["straight_1_8"] = [0.0, 50.0, 0.0, 45.0, 0.0, 55.0, 0.0, 40.0]
    d["straight_1_16"] = [0.0, 35.0, 30.0, 0.0, 0.0, 40.0, 25.0, 0.0, 0.0, 45.0, 35.0, 0.0, 0.0, 30.0, 40.0, 0.0]
    d["strength_values"] = create_strength_pattern("backbeat")
    d["root_note"] = 43
    d["notes"] = [note_to_dict(Note(43, 100)), note_to_dict(Note(47, 55)), note_to_dict(Note(50, 50)),
                  note_to_dict(Note(55, 45)), note_to_dict(Note(59, 35))]
    d["synth_osc_d"] = 0.2
    d["synth_osc_v"] = 0.6
    d["synth_osc_volume"] = 0.65
    d["synth_sub_volume"] = 0.3
    d["synth_filter_cutoff"] = 1800.0
    d["synth_filter_resonance"] = 0.12
    d["synth_filter_env_amount"] = 600.0
    d["synth_vol_attack"] = 8.0
    d["synth_vol_decay"] = 350.0
    d["synth_vol_sustain"] = 0.55
    d["synth_vol_release"] = 450.0
    d["swing_amount"] = 58.0
    d["note_length_percent"] = 100.0
    presets.append(create_preset("R&B Smooth", "Factory",
        "Slow sensual groove - legato lines with deep sub weight", d))

    # 10. Gospel Lift - Uplifting triplet feel
    d = create_default_preset()
    d["triplet_1_4t"] = [100.0, 70.0, 80.0, 95.0, 65.0, 75.0]
    d["triplet_1_8t"] = [0.0, 50.0, 45.0, 0.0, 55.0, 40.0, 0.0, 60.0, 50.0, 0.0, 45.0, 55.0]
    d["strength_values"] = create_strength_pattern("triplet_feel")
    d["root_note"] = 48
    d["notes"] = [note_to_dict(Note(48, 100)), note_to_dict(Note(52, 60)), note_to_dict(Note(55, 55)),
                  note_to_dict(Note(60, 50)), note_to_dict(Note(64, 40))]
    d["synth_osc_d"] = 0.25
    d["synth_osc_v"] = 0.65
    d["synth_osc_stereo_v_offset"] = 0.1
    d["synth_osc_volume"] = 0.68
    d["synth_filter_cutoff"] = 3000.0
    d["synth_filter_resonance"] = 0.12
    d["synth_filter_env_amount"] = 1000.0
    d["synth_vol_attack"] = 5.0
    d["synth_vol_decay"] = 280.0
    d["synth_vol_sustain"] = 0.5
    d["synth_vol_release"] = 380.0
    d["synth_reverb_mix"] = 0.18
    d["synth_reverb_decay"] = 0.5
    d["swing_amount"] = 62.0
    d["note_length_percent"] = 90.0
    presets.append(create_preset("Gospel Lift", "Factory",
        "Uplifting triplet feel - joyful patterns with call and response", d))

    # 11. Country Roots - Simple effective
    d = create_default_preset()
    d["straight_1_4"] = [100.0, 80.0, 90.0, 75.0]
    d["straight_1_8"] = [0.0, 55.0, 0.0, 50.0, 0.0, 60.0, 0.0, 45.0]
    d["strength_values"] = create_strength_pattern("4_4_standard")
    d["root_note"] = 40
    d["notes"] = [note_to_dict(Note(40, 100)), note_to_dict(Note(44, 55)), note_to_dict(Note(47, 50)),
                  note_to_dict(Note(52, 40))]
    d["synth_osc_d"] = 0.3
    d["synth_osc_v"] = 0.55
    d["synth_osc_volume"] = 0.7
    d["synth_sub_volume"] = 0.2
    d["synth_filter_cutoff"] = 2400.0
    d["synth_filter_resonance"] = 0.1
    d["synth_filter_env_amount"] = 700.0
    d["synth_vol_attack"] = 2.0
    d["synth_vol_decay"] = 200.0
    d["synth_vol_sustain"] = 0.5
    d["synth_vol_release"] = 280.0
    d["note_length_percent"] = 75.0
    presets.append(create_preset("Country Roots", "Factory",
        "Honest country bass - simple patterns with major feel", d))

    # 12. Ska Bounce - Extreme offbeat
    d = create_default_preset()
    d["straight_1_8"] = [20.0, 100.0, 20.0, 100.0, 20.0, 100.0, 20.0, 100.0]
    d["straight_1_16"] = [0.0, 0.0, 55.0, 0.0, 0.0, 0.0, 50.0, 0.0, 0.0, 0.0, 60.0, 0.0, 0.0, 0.0, 45.0, 0.0]
    d["strength_values"] = create_strength_pattern("offbeat")
    d["root_note"] = 45
    d["notes"] = [note_to_dict(Note(45, 100)), note_to_dict(Note(48, 50)), note_to_dict(Note(50, 55)),
                  note_to_dict(Note(52, 45))]
    d["synth_osc_d"] = 0.45
    d["synth_osc_v"] = 0.5
    d["synth_osc_volume"] = 0.7
    d["synth_filter_cutoff"] = 2600.0
    d["synth_filter_resonance"] = 0.18
    d["synth_filter_env_amount"] = 1200.0
    d["synth_vol_attack"] = 1.0
    d["synth_vol_decay"] = 130.0
    d["synth_vol_sustain"] = 0.3
    d["synth_vol_release"] = 160.0
    d["note_length_percent"] = 40.0
    presets.append(create_preset("Ska Bounce", "Factory",
        "Offbeat bounce - staccato chops on every and", d))

    # 13. Bebop Run - Fast jazz lines
    d = create_default_preset()
    d["straight_1_8"] = [100.0, 80.0, 90.0, 75.0, 95.0, 85.0, 88.0, 70.0]
    d["straight_1_16"] = [0.0, 55.0, 50.0, 0.0, 0.0, 60.0, 45.0, 0.0, 0.0, 50.0, 55.0, 0.0, 0.0, 65.0, 40.0, 0.0]
    d["triplet_1_8t"] = [65.0, 50.0, 55.0, 60.0, 45.0, 50.0, 70.0, 55.0, 50.0, 55.0, 60.0, 45.0]
    d["strength_values"] = create_strength_pattern("jazz")
    d["root_note"] = 45
    d["notes"] = [note_to_dict(Note(45, 100)), note_to_dict(Note(47, 55)), note_to_dict(Note(48, 50)),
                  note_to_dict(Note(50, 60)), note_to_dict(Note(52, 45)), note_to_dict(Note(55, 40))]
    d["synth_osc_d"] = 0.32
    d["synth_osc_v"] = 0.58
    d["synth_osc_volume"] = 0.68
    d["synth_filter_cutoff"] = 3500.0
    d["synth_filter_resonance"] = 0.12
    d["synth_filter_env_amount"] = 1000.0
    d["synth_vol_attack"] = 1.0
    d["synth_vol_decay"] = 150.0
    d["synth_vol_sustain"] = 0.4
    d["synth_vol_release"] = 200.0
    d["swing_amount"] = 58.0
    d["note_length_percent"] = 55.0
    presets.append(create_preset("Bebop Run", "Factory",
        "Fast jazz runs - chromatic lines dance through changes", d))

    # 14. Soul Ballad - Slow emotional
    d = create_default_preset()
    d["straight_1_2"] = [100.0, 85.0]
    d["straight_1_4"] = [0.0, 60.0, 0.0, 55.0]
    d["straight_1_8"] = [0.0, 40.0, 0.0, 35.0, 0.0, 45.0, 0.0, 30.0]
    d["strength_values"] = create_strength_pattern("sparse")
    d["root_note"] = 43
    d["notes"] = [note_to_dict(Note(43, 100)), note_to_dict(Note(48, 60)), note_to_dict(Note(50, 55)),
                  note_to_dict(Note(55, 50)), note_to_dict(Note(60, 40))]
    d["synth_osc_d"] = 0.18
    d["synth_osc_v"] = 0.65
    d["synth_osc_stereo_v_offset"] = 0.1
    d["synth_osc_volume"] = 0.62
    d["synth_sub_volume"] = 0.28
    d["synth_filter_cutoff"] = 2000.0
    d["synth_filter_resonance"] = 0.1
    d["synth_filter_env_amount"] = 500.0
    d["synth_vol_attack"] = 20.0
    d["synth_vol_decay"] = 500.0
    d["synth_vol_sustain"] = 0.6
    d["synth_vol_release"] = 700.0
    d["synth_reverb_mix"] = 0.2
    d["synth_reverb_decay"] = 0.55
    d["note_length_percent"] = 150.0
    presets.append(create_preset("Soul Ballad", "Factory",
        "Slow emotional expression - long notes breathe with feeling", d))

    # 15. Classic Rock Drive - Driving 8ths
    d = create_default_preset()
    d["straight_1_8"] = [100.0, 85.0, 95.0, 80.0, 100.0, 88.0, 92.0, 78.0]
    d["straight_1_16"] = [0.0, 50.0, 0.0, 45.0, 0.0, 55.0, 0.0, 40.0, 0.0, 50.0, 0.0, 45.0, 0.0, 60.0, 0.0, 35.0]
    d["strength_values"] = create_strength_pattern("4_4_standard")
    d["root_note"] = 40
    d["notes"] = [note_to_dict(Note(40, 100)), note_to_dict(Note(43, 55)), note_to_dict(Note(45, 50)),
                  note_to_dict(Note(47, 45)), note_to_dict(Note(52, 35))]
    d["synth_osc_d"] = 0.45
    d["synth_osc_v"] = 0.45
    d["synth_osc_volume"] = 0.72
    d["synth_sub_volume"] = 0.25
    d["synth_filter_cutoff"] = 2200.0
    d["synth_filter_resonance"] = 0.18
    d["synth_filter_env_amount"] = 800.0
    d["synth_vol_attack"] = 2.0
    d["synth_vol_decay"] = 180.0
    d["synth_vol_sustain"] = 0.5
    d["synth_vol_release"] = 220.0
    d["note_length_percent"] = 70.0
    presets.append(create_preset("Classic Rock Drive", "Factory",
        "Driving rock bass - consistent eighth notes power the track", d))

    # 16. Prog Odd - Complex meter feel
    d = create_default_preset()
    euc_prog = euclidean_rhythm(16, 11)
    for i in range(16):
        d["straight_1_16"][i] = 90.0 if i in euc_prog else 30.0
    d["triplet_1_8t"] = [70.0, 45.0, 55.0, 65.0, 40.0, 50.0, 75.0, 50.0, 45.0, 60.0, 55.0, 40.0]
    d["strength_values"] = create_strength_pattern("polyrhythm_3_4")
    d["root_note"] = 43
    d["notes"] = [note_to_dict(Note(43, 100)), note_to_dict(Note(45, 55)), note_to_dict(Note(48, 50)),
                  note_to_dict(Note(50, 60)), note_to_dict(Note(52, 45)), note_to_dict(Note(55, 40))]
    d["synth_osc_d"] = 0.38
    d["synth_osc_v"] = 0.52
    d["synth_osc_volume"] = 0.7
    d["synth_filter_cutoff"] = 2600.0
    d["synth_filter_resonance"] = 0.2
    d["synth_filter_env_amount"] = 1200.0
    d["synth_vol_attack"] = 2.0
    d["synth_vol_decay"] = 200.0
    d["synth_vol_sustain"] = 0.45
    d["synth_vol_release"] = 280.0
    d["note_length_percent"] = 65.0
    presets.append(create_preset("Prog Odd", "Factory",
        "Complex meter exploration - euclidean meets triplet polyrhythm", d))

    # 17. Delta Blues - Slide guitar feel
    d = create_default_preset()
    d["straight_1_4"] = [100.0, 70.0, 85.0, 65.0]
    d["triplet_1_4t"] = [70.0, 50.0, 60.0, 65.0, 45.0, 55.0]
    d["strength_values"] = create_strength_pattern("shuffle")
    d["root_note"] = 40
    d["scale"] = "Blues"
    d["notes"] = [note_to_dict(Note(40, 127)), note_to_dict(Note(43, 85)), note_to_dict(Note(45, 80)), note_to_dict(Note(46, 70)), note_to_dict(Note(47, 90)), note_to_dict(Note(50, 65))]
    d["synth_osc_d"] = 0.35
    d["synth_osc_v"] = 0.55
    d["synth_osc_volume"] = 0.68
    d["synth_pll_volume"] = 0.25
    d["synth_pll_glide"] = 80.0
    d["synth_filter_cutoff"] = 2400.0
    d["synth_vol_attack"] = 8.0
    d["synth_vol_decay"] = 300.0
    d["synth_vol_sustain"] = 0.5
    d["synth_reverb_mix"] = 0.15
    d["swing_amount"] = 62.0
    presets.append(create_preset("Delta Blues", "Factory",
        "Mississippi slide - blue notes with heavy triplet swing", d))

    # 18. Bebop Run - Fast jazz lines
    d = create_default_preset()
    d["straight_1_8"] = [90.0, 65.0, 80.0, 60.0, 85.0, 62.0, 78.0, 58.0]
    d["straight_1_16"] = [70.0, 45.0, 60.0, 40.0, 65.0, 42.0, 58.0, 38.0, 68.0, 44.0, 62.0, 42.0, 66.0, 40.0, 56.0, 35.0]
    d["strength_values"] = create_strength_pattern("jazz")
    d["root_note"] = 48
    d["scale"] = "MelodicMinor"
    d["stability_pattern"] = "JazzMelodic"
    d["notes"] = [note_to_dict(Note(48, 127)), note_to_dict(Note(50, 75)), note_to_dict(Note(51, 80)), note_to_dict(Note(55, 90)), note_to_dict(Note(57, 85)), note_to_dict(Note(59, 70))]
    d["synth_osc_d"] = 0.3
    d["synth_osc_v"] = 0.55
    d["synth_osc_volume"] = 0.65
    d["synth_filter_cutoff"] = 3200.0
    d["synth_vol_attack"] = 2.0
    d["synth_vol_decay"] = 100.0
    d["synth_vol_sustain"] = 0.4
    d["synth_reverb_mix"] = 0.12
    d["swing_amount"] = 56.0
    d["note_length_percent"] = 55.0
    presets.append(create_preset("Bebop Run", "Factory",
        "Fast jazz lines - melodic minor runs with swing phrasing", d))

    # 19. Soul Ballad - Slow R&B
    d = create_default_preset()
    d["straight_1_2"] = [100.0, 85.0]
    d["straight_1_4"] = [70.0, 55.0, 65.0, 50.0]
    d["triplet_1_4t"] = [50.0, 35.0, 45.0, 48.0, 32.0, 42.0]
    d["strength_values"] = create_strength_pattern("ballad")
    d["root_note"] = 43
    d["scale"] = "Dorian"
    d["notes"] = [note_to_dict(Note(43, 127)), note_to_dict(Note(45, 80)), note_to_dict(Note(46, 75)), note_to_dict(Note(50, 90)), note_to_dict(Note(52, 70))]
    d["synth_osc_d"] = 0.2
    d["synth_osc_v"] = 0.6
    d["synth_osc_volume"] = 0.6
    d["synth_sub_volume"] = 0.3
    d["synth_filter_cutoff"] = 2000.0
    d["synth_vol_attack"] = 30.0
    d["synth_vol_decay"] = 400.0
    d["synth_vol_sustain"] = 0.6
    d["synth_vol_release"] = 600.0
    d["synth_reverb_mix"] = 0.2
    d["swing_amount"] = 54.0
    d["note_length_percent"] = 120.0
    presets.append(create_preset("Soul Ballad", "Factory",
        "Slow soul ballad - warm Dorian bass with expressive phrasing", d))

    # 20. Funk Slap - Popping bass
    d = create_default_preset()
    d["straight_1_16"] = [100.0, 50.0, 70.0, 85.0, 55.0, 90.0, 45.0, 75.0, 95.0, 48.0, 68.0, 82.0, 52.0, 88.0, 42.0, 72.0]
    d["strength_values"] = create_strength_pattern("funk")
    d["root_note"] = 36
    d["scale"] = "Dorian"
    d["notes"] = [note_to_dict(Note(36, 127)), note_to_dict(Note(38, 75)), note_to_dict(Note(41, 85)), note_to_dict(Note(43, 90)), note_to_dict(Note(48, 70))]
    d["synth_osc_d"] = 0.5
    d["synth_osc_v"] = 0.45
    d["synth_osc_volume"] = 0.75
    d["synth_filter_cutoff"] = 2500.0
    d["synth_filter_resonance"] = 0.2
    d["synth_filter_env_amount"] = 1500.0
    d["synth_vol_attack"] = 1.0
    d["synth_vol_decay"] = 80.0
    d["synth_vol_sustain"] = 0.35
    d["synth_tube_drive"] = 1.5
    d["synth_reverb_mix"] = 0.08
    d["swing_amount"] = 55.0
    d["note_length_percent"] = 50.0
    presets.append(create_preset("Funk Slap", "Factory",
        "Popping funk bass - syncopated slaps with ghost notes", d))

    # 21. Country Twang - Nashville bass
    d = create_default_preset()
    d["straight_1_4"] = [100.0, 80.0, 90.0, 75.0]
    d["straight_1_8"] = [60.0, 45.0, 55.0, 40.0, 58.0, 42.0, 52.0, 38.0]
    d["strength_values"] = create_strength_pattern("country")
    d["root_note"] = 40
    d["scale"] = "Major"
    d["notes"] = [note_to_dict(Note(40, 127)), note_to_dict(Note(42, 80)), note_to_dict(Note(44, 85)), note_to_dict(Note(47, 95)), note_to_dict(Note(49, 70))]
    d["synth_osc_d"] = 0.35
    d["synth_osc_v"] = 0.55
    d["synth_osc_volume"] = 0.7
    d["synth_filter_cutoff"] = 2800.0
    d["synth_filter_env_amount"] = 800.0
    d["synth_vol_attack"] = 3.0
    d["synth_vol_decay"] = 150.0
    d["synth_vol_sustain"] = 0.45
    d["synth_reverb_mix"] = 0.1
    d["swing_amount"] = 52.0
    d["note_length_percent"] = 70.0
    presets.append(create_preset("Country Twang", "Factory",
        "Nashville walking bass - major key country with bounce", d))

    # 22. Reggae Skank - Off-beat chords
    d = create_default_preset()
    d["straight_1_4"] = [60.0, 100.0, 55.0, 95.0]
    d["straight_1_8"] = [40.0, 80.0, 35.0, 75.0, 38.0, 78.0, 32.0, 72.0]
    d["strength_values"] = create_strength_pattern("reggae")
    d["root_note"] = 48
    d["scale"] = "Minor"
    d["notes"] = [note_to_dict(Note(48, 127)), note_to_dict(Note(51, 95)), note_to_dict(Note(55, 100)), note_to_dict(Note(58, 80))]
    d["synth_osc_d"] = 0.4
    d["synth_osc_v"] = 0.5
    d["synth_osc_volume"] = 0.65
    d["synth_filter_cutoff"] = 2200.0
    d["synth_filter_resonance"] = 0.15
    d["synth_vol_attack"] = 3.0
    d["synth_vol_decay"] = 100.0
    d["synth_vol_sustain"] = 0.4
    d["synth_reverb_mix"] = 0.18
    d["swing_amount"] = 52.0
    d["note_length_percent"] = 55.0
    presets.append(create_preset("Reggae Skank", "Factory",
        "Off-beat reggae chords - emphasis on beats 2 and 4", d))

    # 23. Gospel Shout - Church organ feel
    d = create_default_preset()
    d["straight_1_4"] = [100.0, 85.0, 95.0, 80.0]
    d["triplet_1_4t"] = [75.0, 55.0, 65.0, 70.0, 50.0, 60.0]
    d["strength_values"] = create_strength_pattern("gospel")
    d["root_note"] = 48
    d["scale"] = "Major"
    d["notes"] = [note_to_dict(Note(48, 127)), note_to_dict(Note(52, 100)), note_to_dict(Note(55, 100)), note_to_dict(Note(59, 85)), note_to_dict(Note(60, 75))]
    d["synth_osc_d"] = 0.25
    d["synth_osc_v"] = 0.55
    d["synth_osc_volume"] = 0.65
    d["synth_pll_volume"] = 0.3
    d["synth_filter_cutoff"] = 3000.0
    d["synth_vol_attack"] = 15.0
    d["synth_vol_decay"] = 250.0
    d["synth_vol_sustain"] = 0.6
    d["synth_reverb_mix"] = 0.22
    d["synth_reverb_decay"] = 0.6
    d["swing_amount"] = 58.0
    presets.append(create_preset("Gospel Shout", "Factory",
        "Church organ chords - triumphant major key progressions", d))

    # 24. Motown Bass - Classic R&B
    d = create_default_preset()
    d["straight_1_8"] = [100.0, 65.0, 80.0, 55.0, 90.0, 60.0, 75.0, 50.0]
    d["straight_1_16"] = [0.0, 45.0, 40.0, 0.0, 0.0, 50.0, 35.0, 0.0, 0.0, 42.0, 38.0, 0.0, 0.0, 48.0, 32.0, 0.0]
    d["strength_values"] = create_strength_pattern("motown")
    d["root_note"] = 36
    d["scale"] = "Major"
    d["notes"] = [note_to_dict(Note(36, 127)), note_to_dict(Note(38, 75)), note_to_dict(Note(40, 85)), note_to_dict(Note(43, 95)), note_to_dict(Note(45, 70))]
    d["synth_osc_d"] = 0.3
    d["synth_osc_v"] = 0.55
    d["synth_osc_volume"] = 0.7
    d["synth_sub_volume"] = 0.25
    d["synth_filter_cutoff"] = 2000.0
    d["synth_filter_env_amount"] = 800.0
    d["synth_vol_attack"] = 3.0
    d["synth_vol_decay"] = 150.0
    d["synth_vol_sustain"] = 0.5
    d["synth_tube_drive"] = 1.3
    d["synth_reverb_mix"] = 0.1
    d["swing_amount"] = 54.0
    presets.append(create_preset("Motown Bass", "Factory",
        "Classic Detroit bass - warm melodic lines with subtle swing", d))

    # 25. Punk Rock - Fast aggressive
    d = create_default_preset()
    d["straight_1_8"] = [100.0, 95.0, 100.0, 90.0, 100.0, 92.0, 100.0, 88.0]
    d["strength_values"] = create_strength_pattern("punk")
    d["root_note"] = 40
    d["scale"] = "Minor"
    d["notes"] = [note_to_dict(Note(40, 127)), note_to_dict(Note(43, 90)), note_to_dict(Note(47, 95))]
    d["synth_osc_d"] = 0.55
    d["synth_osc_v"] = 0.45
    d["synth_osc_volume"] = 0.75
    d["synth_filter_cutoff"] = 2000.0
    d["synth_filter_drive"] = 2.0
    d["synth_vol_attack"] = 1.0
    d["synth_vol_decay"] = 80.0
    d["synth_vol_sustain"] = 0.55
    d["synth_tube_drive"] = 2.0
    d["synth_reverb_mix"] = 0.08
    d["note_length_percent"] = 60.0
    presets.append(create_preset("Punk Rock", "Factory",
        "Fast aggressive punk - relentless eighth notes with attitude", d))

    # 26. Swing Era - Big band bass
    d = create_default_preset()
    d["straight_1_4"] = [100.0, 85.0, 95.0, 80.0]
    d["triplet_1_4t"] = [70.0, 50.0, 60.0, 68.0, 48.0, 58.0]
    d["strength_values"] = create_strength_pattern("swing")
    d["root_note"] = 36
    d["scale"] = "Major"
    d["notes"] = [note_to_dict(Note(36, 127)), note_to_dict(Note(38, 75)), note_to_dict(Note(40, 85)), note_to_dict(Note(43, 95)), note_to_dict(Note(45, 70))]
    d["synth_osc_d"] = 0.25
    d["synth_osc_v"] = 0.58
    d["synth_osc_volume"] = 0.65
    d["synth_sub_volume"] = 0.3
    d["synth_filter_cutoff"] = 2200.0
    d["synth_vol_attack"] = 5.0
    d["synth_vol_decay"] = 180.0
    d["synth_vol_sustain"] = 0.5
    d["synth_reverb_mix"] = 0.12
    d["swing_amount"] = 62.0
    d["note_length_percent"] = 80.0
    presets.append(create_preset("Swing Era", "Factory",
        "Big band walking bass - classic swing feel with triplet pulse", d))

    # 27. Metal Chug - Heavy palm mutes
    d = create_default_preset()
    d["straight_1_8"] = [100.0, 90.0, 95.0, 85.0, 100.0, 88.0, 92.0, 82.0]
    d["straight_1_16"] = [80.0, 60.0, 75.0, 55.0, 78.0, 58.0, 72.0, 52.0, 82.0, 62.0, 77.0, 57.0, 76.0, 55.0, 70.0, 50.0]
    d["strength_values"] = create_strength_pattern("metal")
    d["root_note"] = 36
    d["scale"] = "Phrygian"
    d["notes"] = [note_to_dict(Note(36, 127)), note_to_dict(Note(37, 70)), note_to_dict(Note(41, 85)), note_to_dict(Note(43, 80))]
    d["synth_osc_d"] = 0.6
    d["synth_osc_v"] = 0.4
    d["synth_osc_volume"] = 0.75
    d["synth_filter_cutoff"] = 1500.0
    d["synth_filter_drive"] = 3.0
    d["synth_color_distortion_amount"] = 0.3
    d["synth_vol_attack"] = 1.0
    d["synth_vol_decay"] = 60.0
    d["synth_vol_sustain"] = 0.4
    d["synth_reverb_mix"] = 0.05
    d["note_length_percent"] = 45.0
    presets.append(create_preset("Metal Chug", "Factory",
        "Heavy Phrygian riffs - tight palm muted patterns", d))

    # 28. Bossa Nova - Brazilian jazz
    d = create_default_preset()
    d["straight_1_8"] = [80.0, 50.0, 65.0, 45.0, 75.0, 48.0, 62.0, 42.0]
    d["straight_1_16"] = [55.0, 35.0, 48.0, 30.0, 52.0, 32.0, 45.0, 28.0, 58.0, 38.0, 50.0, 32.0, 54.0, 34.0, 46.0, 26.0]
    d["strength_values"] = create_strength_pattern("bossa")
    d["root_note"] = 43
    d["scale"] = "Dorian"
    d["stability_pattern"] = "JazzMelodic"
    d["notes"] = [note_to_dict(Note(43, 127)), note_to_dict(Note(45, 80)), note_to_dict(Note(46, 75)), note_to_dict(Note(50, 90)), note_to_dict(Note(52, 70)), note_to_dict(Note(55, 60))]
    d["synth_osc_d"] = 0.25
    d["synth_osc_v"] = 0.55
    d["synth_osc_volume"] = 0.62
    d["synth_filter_cutoff"] = 2600.0
    d["synth_vol_attack"] = 5.0
    d["synth_vol_decay"] = 200.0
    d["synth_vol_sustain"] = 0.45
    d["synth_reverb_mix"] = 0.15
    d["swing_amount"] = 52.0
    d["note_length_percent"] = 75.0
    presets.append(create_preset("Bossa Nova", "Factory",
        "Brazilian sophistication - syncopated Dorian bass with jazz colors", d))

    # 29. Disco Octave - Classic disco bass
    d = create_default_preset()
    d["straight_1_8"] = [100.0, 75.0, 90.0, 70.0, 95.0, 72.0, 88.0, 68.0]
    d["strength_values"] = create_strength_pattern("disco")
    d["root_note"] = 36
    d["scale"] = "Minor"
    d["octave_randomization"] = create_octave_randomization(0.235, 0.12, 0.12, "Up")
    d["notes"] = [note_to_dict(Note(36, 127)), note_to_dict(Note(39, 85)), note_to_dict(Note(43, 90))]
    d["synth_osc_d"] = 0.4
    d["synth_osc_v"] = 0.5
    d["synth_osc_volume"] = 0.7
    d["synth_sub_volume"] = 0.3
    d["synth_filter_cutoff"] = 2400.0
    d["synth_filter_resonance"] = 0.2
    d["synth_filter_env_amount"] = 1200.0
    d["synth_vol_attack"] = 2.0
    d["synth_vol_decay"] = 100.0
    d["synth_vol_sustain"] = 0.5
    d["synth_reverb_mix"] = 0.1
    d["note_length_percent"] = 65.0
    presets.append(create_preset("Disco Octave", "Factory",
        "Classic disco bass - octave jumps drive the dance floor", d))

    # 30. Latin Tumbao - Afro-Cuban bass
    d = create_default_preset()
    d["straight_1_8"] = [100.0, 55.0, 75.0, 90.0, 50.0, 85.0, 60.0, 80.0]
    d["strength_values"] = create_strength_pattern("clave")
    d["root_note"] = 41
    d["scale"] = "Minor"
    d["notes"] = [note_to_dict(Note(41, 127)), note_to_dict(Note(43, 80)), note_to_dict(Note(46, 85)), note_to_dict(Note(48, 90)), note_to_dict(Note(53, 70))]
    d["synth_osc_d"] = 0.35
    d["synth_osc_v"] = 0.55
    d["synth_osc_volume"] = 0.68
    d["synth_filter_cutoff"] = 2500.0
    d["synth_vol_attack"] = 3.0
    d["synth_vol_decay"] = 130.0
    d["synth_vol_sustain"] = 0.45
    d["synth_reverb_mix"] = 0.1
    d["swing_amount"] = 52.0
    d["note_length_percent"] = 70.0
    presets.append(create_preset("Latin Tumbao", "Factory",
        "Afro-Cuban bass pattern - syncopated tumbao over clave", d))

    # 31. New Orleans Funk - Second line groove
    d = create_default_preset()
    d["straight_1_8"] = [90.0, 60.0, 75.0, 55.0, 85.0, 58.0, 72.0, 52.0]
    d["straight_1_16"] = [65.0, 40.0, 55.0, 35.0, 62.0, 38.0, 52.0, 32.0, 68.0, 42.0, 58.0, 38.0, 60.0, 35.0, 50.0, 30.0]
    d["strength_values"] = create_strength_pattern("second_line")
    d["root_note"] = 43
    d["scale"] = "Mixolydian"
    d["notes"] = [note_to_dict(Note(43, 127)), note_to_dict(Note(45, 75)), note_to_dict(Note(47, 85)), note_to_dict(Note(50, 90)), note_to_dict(Note(52, 70))]
    d["synth_osc_d"] = 0.38
    d["synth_osc_v"] = 0.52
    d["synth_osc_volume"] = 0.68
    d["synth_filter_cutoff"] = 2600.0
    d["synth_filter_env_amount"] = 900.0
    d["synth_vol_attack"] = 2.0
    d["synth_vol_decay"] = 140.0
    d["synth_vol_sustain"] = 0.48
    d["synth_reverb_mix"] = 0.12
    d["swing_amount"] = 56.0
    presets.append(create_preset("New Orleans Funk", "Factory",
        "Second line groove - Mixolydian bounce with parade energy", d))

    # 32. Surf Rock - Reverb-drenched twang
    d = create_default_preset()
    d["straight_1_8"] = [100.0, 75.0, 90.0, 70.0, 95.0, 72.0, 88.0, 68.0]
    d["triplet_1_8t"] = [60.0, 40.0, 50.0, 58.0, 38.0, 48.0, 55.0, 35.0, 45.0, 52.0, 32.0, 42.0]
    d["strength_values"] = create_strength_pattern("surf")
    d["root_note"] = 40
    d["scale"] = "Minor"
    d["notes"] = [note_to_dict(Note(40, 127)), note_to_dict(Note(43, 85)), note_to_dict(Note(47, 90)), note_to_dict(Note(52, 75))]
    d["synth_osc_d"] = 0.45
    d["synth_osc_v"] = 0.5
    d["synth_osc_volume"] = 0.68
    d["synth_filter_cutoff"] = 3200.0
    d["synth_filter_resonance"] = 0.15
    d["synth_vol_attack"] = 2.0
    d["synth_vol_decay"] = 120.0
    d["synth_vol_sustain"] = 0.45
    d["synth_reverb_mix"] = 0.28
    d["synth_reverb_decay"] = 0.65
    d["note_length_percent"] = 70.0
    presets.append(create_preset("Surf Rock", "Factory",
        "Beach party bass - reverb-soaked minor key twang", d))

    return presets

def create_bank_d() -> List[Dict]:
    """Bank D: Experimental & Chill - 32 presets"""
    presets = []

    # 1. Euclidean Garden - Pure euclidean patterns
    d = create_default_preset()
    euc5_8 = euclidean_rhythm(8, 5)
    euc7_16 = euclidean_rhythm(16, 7)
    euc3_8 = euclidean_rhythm(8, 3)
    for i in range(8):
        d["straight_1_8"][i] = 95.0 if i in euc5_8 else 25.0
    for i in range(16):
        d["straight_1_16"][i] = 70.0 if i in euc7_16 else 20.0
    d["triplet_1_8t"] = [80.0, 0.0, 60.0, 75.0, 0.0, 55.0, 85.0, 0.0, 50.0, 70.0, 0.0, 65.0]
    d["strength_values"] = create_strength_pattern("polyrhythm_3_4")
    d["root_note"] = 48
    d["notes"] = [note_to_dict(Note(48, 100)), note_to_dict(Note(52, 55)), note_to_dict(Note(55, 50)),
                  note_to_dict(Note(60, 45)), note_to_dict(Note(64, 35))]
    d["synth_osc_d"] = 0.35
    d["synth_osc_v"] = 0.6
    d["synth_osc_volume"] = 0.7
    d["synth_filter_cutoff"] = 3000.0
    d["synth_filter_resonance"] = 0.15
    d["synth_filter_env_amount"] = 1200.0
    d["synth_vol_attack"] = 3.0
    d["synth_vol_decay"] = 250.0
    d["synth_vol_sustain"] = 0.45
    d["synth_vol_release"] = 350.0
    d["note_length_percent"] = 75.0
    presets.append(create_preset("Euclidean Garden", "Factory",
        "Interlocking euclidean cycles - mathematical beauty in motion", d))

    # 2. Polyrhythm Drift - Overlapping cycles
    d = create_default_preset()
    d["straight_1_4"] = [100.0, 75.0, 85.0, 70.0]
    d["triplet_1_4t"] = [90.0, 60.0, 70.0, 85.0, 55.0, 65.0]
    d["dotted_1_4d"] = [80.0, 55.0, 65.0]
    d["strength_values"] = create_strength_pattern("polyrhythm_3_4")
    d["root_note"] = 45
    d["notes"] = [note_to_dict(Note(45, 100)), note_to_dict(Note(48, 55)), note_to_dict(Note(52, 50)),
                  note_to_dict(Note(57, 45)), note_to_dict(Note(60, 40))]
    d["synth_osc_d"] = 0.28
    d["synth_osc_v"] = 0.65
    d["synth_osc_stereo_v_offset"] = 0.12
    d["synth_osc_volume"] = 0.68
    d["synth_filter_cutoff"] = 2500.0
    d["synth_filter_resonance"] = 0.12
    d["synth_filter_env_amount"] = 800.0
    d["synth_vol_attack"] = 5.0
    d["synth_vol_decay"] = 300.0
    d["synth_vol_sustain"] = 0.5
    d["synth_vol_release"] = 400.0
    d["synth_reverb_mix"] = 0.18
    d["synth_reverb_decay"] = 0.5
    d["note_length_percent"] = 90.0
    presets.append(create_preset("Polyrhythm Drift", "Factory",
        "Overlapping cycles - different divisions phase in and out", d))

    # 3. Generative Sparse - Very sparse random
    d = create_default_preset()
    d["straight_1_1"] = [60.0]
    d["straight_1_2"] = [45.0, 40.0]
    d["straight_1_4"] = [35.0, 25.0, 30.0, 20.0]
    d["straight_1_8"] = [25.0, 15.0, 20.0, 10.0, 30.0, 18.0, 22.0, 12.0]
    d["strength_values"] = create_strength_pattern("sparse")
    d["root_note"] = 43
    d["notes"] = [note_to_dict(Note(43, 100)), note_to_dict(Note(48, 60)), note_to_dict(Note(50, 55)),
                  note_to_dict(Note(55, 50)), note_to_dict(Note(60, 45)), note_to_dict(Note(67, 35))]
    d["synth_osc_d"] = 0.2
    d["synth_osc_v"] = 0.72
    d["synth_osc_stereo_v_offset"] = 0.15
    d["synth_osc_volume"] = 0.6
    d["synth_filter_cutoff"] = 4000.0
    d["synth_filter_resonance"] = 0.1
    d["synth_filter_env_amount"] = 1000.0
    d["synth_vol_attack"] = 30.0
    d["synth_vol_decay"] = 800.0
    d["synth_vol_sustain"] = 0.5
    d["synth_vol_release"] = 1200.0
    d["synth_reverb_mix"] = 0.25
    d["synth_reverb_decay"] = 0.75
    d["synth_reverb_diffusion"] = 0.9
    d["note_length_percent"] = 180.0
    presets.append(create_preset("Generative Sparse", "Factory",
        "Minimal generative - rare events bloom in vast space", d))

    # 4. Micro Timing Lab - Humanization focus
    d = create_default_preset()
    d["straight_1_8"] = [100.0, 70.0, 85.0, 65.0, 95.0, 75.0, 80.0, 60.0]
    d["straight_1_16"] = [0.0, 45.0, 40.0, 0.0, 0.0, 50.0, 35.0, 0.0, 0.0, 55.0, 45.0, 0.0, 0.0, 40.0, 50.0, 0.0]
    d["strength_values"] = create_strength_pattern("jazz")
    d["root_note"] = 48
    d["notes"] = [note_to_dict(Note(48, 100)), note_to_dict(Note(50, 55)), note_to_dict(Note(52, 50)),
                  note_to_dict(Note(55, 45)), note_to_dict(Note(57, 40))]
    d["synth_osc_d"] = 0.35
    d["synth_osc_v"] = 0.55
    d["synth_osc_volume"] = 0.68
    d["synth_filter_cutoff"] = 2800.0
    d["synth_filter_resonance"] = 0.15
    d["synth_filter_env_amount"] = 1000.0
    d["synth_vol_attack"] = 2.0
    d["synth_vol_decay"] = 220.0
    d["synth_vol_sustain"] = 0.5
    d["synth_vol_release"] = 300.0
    d["swing_amount"] = 56.0
    d["note_length_percent"] = 80.0
    d["pos_mod_1_target"] = 0.7
    d["pos_mod_1_shift"] = -0.025
    d["pos_mod_1_prob"] = 0.5
    d["pos_mod_2_target"] = -0.6
    d["pos_mod_2_shift"] = 0.02
    d["pos_mod_2_prob"] = 0.45
    presets.append(create_preset("Micro Timing Lab", "Factory",
        "Human feel study - position modifiers create organic push-pull", d))

    # 5. Probability Cascade - Cascading chances
    d = create_default_preset()
    d["straight_1_4"] = [100.0, 85.0, 90.0, 80.0]
    d["straight_1_8"] = [75.0, 65.0, 70.0, 60.0, 80.0, 55.0, 72.0, 50.0]
    d["straight_1_16"] = [55.0, 45.0, 50.0, 40.0, 60.0, 35.0, 52.0, 30.0, 58.0, 42.0, 48.0, 38.0, 62.0, 32.0, 55.0, 28.0]
    d["strength_values"] = create_strength_pattern("dense")
    d["root_note"] = 45
    d["notes"] = [note_to_dict(Note(45, 100)), note_to_dict(Note(48, 60)), note_to_dict(Note(50, 55)),
                  note_to_dict(Note(52, 50)), note_to_dict(Note(55, 45)), note_to_dict(Note(57, 40))]
    d["synth_osc_d"] = 0.42
    d["synth_osc_v"] = 0.52
    d["synth_osc_volume"] = 0.7
    d["synth_filter_cutoff"] = 3200.0
    d["synth_filter_resonance"] = 0.18
    d["synth_filter_env_amount"] = 1400.0
    d["synth_vol_attack"] = 2.0
    d["synth_vol_decay"] = 180.0
    d["synth_vol_sustain"] = 0.45
    d["synth_vol_release"] = 250.0
    d["note_length_percent"] = 65.0
    presets.append(create_preset("Probability Cascade", "Factory",
        "Layered density - probabilities compound into complex results", d))

    # 6. Quantum Bounce - Unpredictable
    d = create_default_preset()
    d["straight_1_8"] = [80.0, 45.0, 60.0, 70.0, 50.0, 75.0, 40.0, 65.0]
    d["straight_1_16"] = [60.0, 30.0, 45.0, 55.0, 35.0, 50.0, 25.0, 65.0, 55.0, 40.0, 50.0, 35.0, 70.0, 28.0, 58.0, 42.0]
    d["triplet_1_8t"] = [55.0, 40.0, 50.0, 60.0, 35.0, 45.0, 65.0, 30.0, 55.0, 50.0, 45.0, 40.0]
    d["strength_values"] = create_strength_pattern("funk")
    d["root_note"] = 43
    d["notes"] = [note_to_dict(Note(43, 100)), note_to_dict(Note(45, 55)), note_to_dict(Note(48, 60)),
                  note_to_dict(Note(50, 50)), note_to_dict(Note(55, 45))]
    d["synth_osc_d"] = 0.5
    d["synth_osc_v"] = 0.48
    d["synth_osc_volume"] = 0.72
    d["synth_filter_cutoff"] = 2600.0
    d["synth_filter_resonance"] = 0.22
    d["synth_filter_env_amount"] = 1500.0
    d["synth_vol_attack"] = 1.0
    d["synth_vol_decay"] = 140.0
    d["synth_vol_sustain"] = 0.4
    d["synth_vol_release"] = 180.0
    d["swing_amount"] = 52.0
    d["note_length_percent"] = 55.0
    presets.append(create_preset("Quantum Bounce", "Factory",
        "Unpredictable energy - competing probabilities create surprise", d))

    # 7. Tape Loop Mantra - Hypnotic repetition
    d = create_default_preset()
    d["straight_1_4"] = [100.0, 95.0, 100.0, 90.0]
    d["straight_1_8"] = [0.0, 65.0, 0.0, 60.0, 0.0, 70.0, 0.0, 55.0]
    d["strength_values"] = create_strength_pattern("driving")
    d["root_note"] = 41
    d["notes"] = [note_to_dict(Note(41, 100)), note_to_dict(Note(48, 40)), note_to_dict(Note(53, 35))]
    d["synth_osc_d"] = 0.32
    d["synth_osc_v"] = 0.48
    d["synth_osc_volume"] = 0.65
    d["synth_sub_volume"] = 0.25
    d["synth_filter_cutoff"] = 1600.0
    d["synth_filter_resonance"] = 0.28
    d["synth_filter_env_amount"] = 500.0
    d["synth_vol_attack"] = 5.0
    d["synth_vol_decay"] = 350.0
    d["synth_vol_sustain"] = 0.6
    d["synth_vol_release"] = 450.0
    d["note_length_percent"] = 100.0
    d["lfo1_tempo_sync"] = True
    d["lfo1_sync_division"] = 0
    d["lfo1_waveform"] = 0
    d["lfo1_dest1"] = 12
    d["lfo1_amount1"] = 0.18
    presets.append(create_preset("Tape Loop Mantra", "Factory",
        "Hypnotic tape feel - endless repetition with slow filter drift", d))

    # 8. Crystal Lattice - Bell-like geometric
    d = create_default_preset()
    euc_crystal = euclidean_rhythm(16, 5)
    for i in range(16):
        d["straight_1_16"][i] = 100.0 if i in euc_crystal else 40.0
    d["dotted_1_8d"] = [80.0, 55.0, 65.0, 75.0, 50.0, 60.0]
    d["strength_values"] = create_strength_pattern("sparse")
    d["root_note"] = 60
    d["notes"] = [note_to_dict(Note(60, 100)), note_to_dict(Note(64, 60)), note_to_dict(Note(67, 55)),
                  note_to_dict(Note(72, 50)), note_to_dict(Note(76, 40))]
    d["synth_osc_d"] = 0.58
    d["synth_osc_v"] = 0.75
    d["synth_osc_stereo_v_offset"] = 0.12
    d["synth_osc_volume"] = 0.68
    d["synth_filter_cutoff"] = 5500.0
    d["synth_filter_resonance"] = 0.2
    d["synth_filter_env_amount"] = 2500.0
    d["synth_vol_attack"] = 1.0
    d["synth_vol_decay"] = 500.0
    d["synth_vol_sustain"] = 0.25
    d["synth_vol_release"] = 700.0
    d["synth_reverb_mix"] = 0.2
    d["synth_reverb_decay"] = 0.7
    d["synth_reverb_diffusion"] = 0.85
    d["note_length_percent"] = 130.0
    presets.append(create_preset("Crystal Lattice", "Factory",
        "Geometric bells - crystalline tones in precise patterns", d))

    # 9. Dream State - Floaty ethereal
    d = create_default_preset()
    d["straight_1_2"] = [100.0, 75.0]
    d["straight_1_4"] = [60.0, 45.0, 55.0, 40.0]
    d["dotted_1_4d"] = [50.0, 35.0, 40.0]
    d["strength_values"] = create_strength_pattern("ambient")
    d["root_note"] = 48
    d["notes"] = [note_to_dict(Note(48, 100)), note_to_dict(Note(52, 60)), note_to_dict(Note(55, 55)),
                  note_to_dict(Note(60, 50)), note_to_dict(Note(64, 45)), note_to_dict(Note(67, 35))]
    d["synth_osc_d"] = 0.15
    d["synth_osc_v"] = 0.78
    d["synth_osc_stereo_v_offset"] = 0.18
    d["synth_osc_volume"] = 0.58
    d["synth_filter_cutoff"] = 3500.0
    d["synth_filter_resonance"] = 0.08
    d["synth_filter_env_amount"] = 400.0
    d["synth_vol_attack"] = 150.0
    d["synth_vol_decay"] = 1200.0
    d["synth_vol_sustain"] = 0.6
    d["synth_vol_release"] = 1800.0
    d["synth_reverb_mix"] = 0.22
    d["synth_reverb_decay"] = 0.8
    d["synth_reverb_diffusion"] = 0.92
    d["note_length_percent"] = 200.0
    d["lfo1_rate"] = 0.05
    d["lfo1_waveform"] = 0
    d["lfo1_dest1"] = 11
    d["lfo1_amount1"] = 0.15
    presets.append(create_preset("Dream State", "Factory",
        "Ethereal floating - very slow notes dissolve into space", d))

    # 10. Psychedelic Swirl - Trippy modulation
    d = create_default_preset()
    d["straight_1_8"] = [100.0, 65.0, 80.0, 55.0, 90.0, 70.0, 75.0, 60.0]
    d["triplet_1_8t"] = [60.0, 45.0, 50.0, 55.0, 40.0, 45.0, 65.0, 50.0, 45.0, 50.0, 55.0, 40.0]
    d["strength_values"] = create_strength_pattern("triplet_feel")
    d["root_note"] = 45
    d["notes"] = [note_to_dict(Note(45, 100)), note_to_dict(Note(48, 55)), note_to_dict(Note(52, 50)),
                  note_to_dict(Note(57, 45)), note_to_dict(Note(60, 40))]
    d["synth_osc_d"] = 0.45
    d["synth_osc_v"] = 0.6
    d["synth_osc_stereo_v_offset"] = 0.1
    d["synth_osc_volume"] = 0.7
    d["synth_filter_cutoff"] = 2400.0
    d["synth_filter_resonance"] = 0.25
    d["synth_filter_env_amount"] = 1500.0
    d["synth_vol_attack"] = 5.0
    d["synth_vol_decay"] = 280.0
    d["synth_vol_sustain"] = 0.5
    d["synth_vol_release"] = 380.0
    d["synth_reverb_mix"] = 0.25
    d["synth_reverb_decay"] = 0.6
    d["note_length_percent"] = 85.0
    d["lfo1_tempo_sync"] = True
    d["lfo1_sync_division"] = 7
    d["lfo1_waveform"] = 0
    d["lfo1_dest1"] = 12
    d["lfo1_amount1"] = 0.3
    d["lfo2_tempo_sync"] = True
    d["lfo2_sync_division"] = 10
    d["lfo2_waveform"] = 1
    d["lfo2_dest1"] = 11
    d["lfo2_amount1"] = 0.15
    d["lfo3_tempo_sync"] = True
    d["lfo3_sync_division"] = 12
    d["lfo3_waveform"] = 0
    d["lfo3_dest1"] = 10
    d["lfo3_amount1"] = 0.1
    presets.append(create_preset("Psychedelic Swirl", "Factory",
        "Trippy modulation fest - multiple LFOs create shifting textures", d))

    # 11. Cosmic Drone - Deep sustained
    d = create_default_preset()
    d["straight_1_1"] = [100.0]
    d["straight_1_2"] = [50.0, 45.0]
    d["straight_1_4"] = [30.0, 20.0, 25.0, 15.0]
    d["strength_values"] = create_strength_pattern("sparse")
    d["root_note"] = 36
    d["notes"] = [note_to_dict(Note(36, 100)), note_to_dict(Note(43, 50)), note_to_dict(Note(48, 40))]
    d["synth_osc_d"] = 0.12
    d["synth_osc_v"] = 0.38
    d["synth_osc_volume"] = 0.6
    d["synth_sub_volume"] = 0.45
    d["synth_filter_cutoff"] = 600.0
    d["synth_filter_resonance"] = 0.4
    d["synth_filter_env_amount"] = 200.0
    d["synth_vol_attack"] = 200.0
    d["synth_vol_decay"] = 2000.0
    d["synth_vol_sustain"] = 0.7
    d["synth_vol_release"] = 3000.0
    d["synth_reverb_mix"] = 0.25
    d["synth_reverb_decay"] = 0.7
    d["note_length_percent"] = 200.0
    d["lfo1_rate"] = 0.03
    d["lfo1_waveform"] = 0
    d["lfo1_dest1"] = 12
    d["lfo1_amount1"] = 0.25
    presets.append(create_preset("Cosmic Drone", "Factory",
        "Deep space bass - sustained tones with very slow movement", d))

    # 12. Fractal Pattern - Self-similar
    d = create_default_preset()
    d["straight_1_2"] = [100.0, 80.0]
    d["straight_1_4"] = [90.0, 70.0, 85.0, 65.0]
    d["straight_1_8"] = [80.0, 60.0, 75.0, 55.0, 85.0, 65.0, 70.0, 50.0]
    d["straight_1_16"] = [70.0, 50.0, 65.0, 45.0, 75.0, 55.0, 60.0, 40.0, 72.0, 52.0, 68.0, 48.0, 78.0, 58.0, 62.0, 42.0]
    d["strength_values"] = create_strength_pattern("4_4_standard")
    d["root_note"] = 48
    d["notes"] = [note_to_dict(Note(48, 100)), note_to_dict(Note(50, 55)), note_to_dict(Note(52, 50)),
                  note_to_dict(Note(55, 60)), note_to_dict(Note(57, 45))]
    d["synth_osc_d"] = 0.4
    d["synth_osc_v"] = 0.55
    d["synth_osc_volume"] = 0.7
    d["synth_filter_cutoff"] = 2800.0
    d["synth_filter_resonance"] = 0.15
    d["synth_filter_env_amount"] = 1100.0
    d["synth_vol_attack"] = 2.0
    d["synth_vol_decay"] = 200.0
    d["synth_vol_sustain"] = 0.5
    d["synth_vol_release"] = 280.0
    d["note_length_percent"] = 70.0
    presets.append(create_preset("Fractal Pattern", "Factory",
        "Self-similar rhythms - patterns echo across time scales", d))

    # 13. Dawn Chorus - Birdsong inspired
    d = create_default_preset()
    d["straight_1_16"] = [60.0, 30.0, 45.0, 20.0, 55.0, 25.0, 50.0, 35.0, 65.0, 28.0, 40.0, 22.0, 58.0, 32.0, 48.0, 38.0]
    d["straight_1_32"] = [35.0, 15.0, 25.0, 10.0, 40.0, 20.0, 30.0, 18.0, 38.0, 12.0, 28.0, 8.0, 42.0, 22.0, 32.0, 16.0,
                          36.0, 14.0, 26.0, 12.0, 44.0, 18.0, 34.0, 20.0, 40.0, 16.0, 30.0, 10.0, 46.0, 24.0, 36.0, 22.0]
    d["strength_values"] = create_strength_pattern("ambient")
    d["root_note"] = 60
    d["notes"] = [note_to_dict(Note(60, 100)), note_to_dict(Note(64, 60)), note_to_dict(Note(67, 55)),
                  note_to_dict(Note(71, 50)), note_to_dict(Note(72, 45)), note_to_dict(Note(76, 35))]
    d["synth_osc_d"] = 0.55
    d["synth_osc_v"] = 0.72
    d["synth_osc_stereo_v_offset"] = 0.15
    d["synth_osc_volume"] = 0.65
    d["synth_filter_cutoff"] = 6000.0
    d["synth_filter_resonance"] = 0.12
    d["synth_filter_env_amount"] = 2000.0
    d["synth_vol_attack"] = 1.0
    d["synth_vol_decay"] = 120.0
    d["synth_vol_sustain"] = 0.2
    d["synth_vol_release"] = 180.0
    d["synth_reverb_mix"] = 0.2
    d["synth_reverb_decay"] = 0.6
    d["note_length_percent"] = 40.0
    presets.append(create_preset("Dawn Chorus", "Factory",
        "Birdsong territory - rapid high notes flutter unpredictably", d))

    # 14. Ocean Waves - Flowing organic
    d = create_default_preset()
    d["straight_1_2"] = [100.0, 85.0]
    d["straight_1_4"] = [70.0, 55.0, 65.0, 50.0]
    d["straight_1_8"] = [50.0, 35.0, 45.0, 30.0, 55.0, 40.0, 48.0, 32.0]
    d["dotted_1_4d"] = [60.0, 45.0, 50.0]
    d["strength_values"] = create_strength_pattern("ambient")
    d["root_note"] = 43
    d["notes"] = [note_to_dict(Note(43, 100)), note_to_dict(Note(48, 60)), note_to_dict(Note(50, 55)),
                  note_to_dict(Note(55, 50)), note_to_dict(Note(60, 40))]
    d["synth_osc_d"] = 0.2
    d["synth_osc_v"] = 0.65
    d["synth_osc_stereo_v_offset"] = 0.12
    d["synth_osc_volume"] = 0.62
    d["synth_sub_volume"] = 0.2
    d["synth_filter_cutoff"] = 2200.0
    d["synth_filter_resonance"] = 0.1
    d["synth_filter_env_amount"] = 600.0
    d["synth_vol_attack"] = 80.0
    d["synth_vol_decay"] = 700.0
    d["synth_vol_sustain"] = 0.55
    d["synth_vol_release"] = 1000.0
    d["synth_reverb_mix"] = 0.2
    d["synth_reverb_decay"] = 0.65
    d["note_length_percent"] = 150.0
    d["lfo1_rate"] = 0.08
    d["lfo1_waveform"] = 0
    d["lfo1_dest1"] = 25
    d["lfo1_amount1"] = 0.2
    presets.append(create_preset("Ocean Waves", "Factory",
        "Flowing like water - slow swells build and recede", d))

    # 15. Neural Net - Complex learning-like
    d = create_default_preset()
    d["straight_1_8"] = [90.0, 55.0, 70.0, 80.0, 60.0, 85.0, 50.0, 75.0]
    d["straight_1_16"] = [70.0, 40.0, 55.0, 65.0, 45.0, 60.0, 35.0, 72.0, 68.0, 42.0, 58.0, 48.0, 75.0, 38.0, 62.0, 52.0]
    d["triplet_1_8t"] = [60.0, 35.0, 50.0, 55.0, 40.0, 45.0, 65.0, 32.0, 55.0, 50.0, 38.0, 58.0]
    d["strength_values"] = create_strength_pattern("dense")
    d["root_note"] = 45
    d["notes"] = [note_to_dict(Note(45, 100)), note_to_dict(Note(47, 50)), note_to_dict(Note(48, 55)),
                  note_to_dict(Note(50, 60)), note_to_dict(Note(52, 45)), note_to_dict(Note(55, 40)),
                  note_to_dict(Note(57, 35))]
    d["synth_osc_d"] = 0.48
    d["synth_osc_v"] = 0.52
    d["synth_osc_volume"] = 0.7
    d["synth_filter_cutoff"] = 3000.0
    d["synth_filter_resonance"] = 0.2
    d["synth_filter_env_amount"] = 1400.0
    d["synth_vol_attack"] = 2.0
    d["synth_vol_decay"] = 180.0
    d["synth_vol_sustain"] = 0.45
    d["synth_vol_release"] = 250.0
    d["swing_amount"] = 52.0
    d["note_length_percent"] = 60.0
    presets.append(create_preset("Neural Net", "Factory",
        "Complex adaptive - patterns that seem to think and respond", d))

    # 16. Time Stretch - Slow stretched feel
    d = create_default_preset()
    d["straight_1_1"] = [100.0]
    d["straight_1_2"] = [80.0, 70.0]
    d["straight_1_4"] = [55.0, 40.0, 50.0, 35.0]
    d["dotted_1_2d"] = [65.0, 50.0]
    d["dotted_1_4d"] = [45.0, 30.0, 38.0]
    d["strength_values"] = create_strength_pattern("sparse")
    d["root_note"] = 41
    d["notes"] = [note_to_dict(Note(41, 100)), note_to_dict(Note(48, 60)), note_to_dict(Note(53, 55)),
                  note_to_dict(Note(60, 45)), note_to_dict(Note(65, 35))]
    d["synth_osc_d"] = 0.18
    d["synth_osc_v"] = 0.7
    d["synth_osc_stereo_v_offset"] = 0.15
    d["synth_osc_volume"] = 0.6
    d["synth_sub_volume"] = 0.2
    d["synth_filter_cutoff"] = 2500.0
    d["synth_filter_resonance"] = 0.1
    d["synth_filter_env_amount"] = 500.0
    d["synth_vol_attack"] = 100.0
    d["synth_vol_decay"] = 1000.0
    d["synth_vol_sustain"] = 0.6
    d["synth_vol_release"] = 1500.0
    d["synth_reverb_mix"] = 0.22
    d["synth_reverb_decay"] = 0.72
    d["synth_reverb_diffusion"] = 0.88
    d["note_length_percent"] = 180.0
    presets.append(create_preset("Time Stretch", "Factory",
        "Dilated time - slow events stretch perception of rhythm", d))

    # 17. Granular Fields - Scattered particles
    d = create_default_preset()
    d["straight_1_16"] = [45.0, 30.0, 40.0, 25.0, 50.0, 35.0, 38.0, 28.0, 48.0, 32.0, 42.0, 22.0, 52.0, 38.0, 45.0, 30.0]
    d["straight_1_32"] = [30.0, 15.0, 25.0, 12.0, 35.0, 18.0, 28.0, 10.0, 32.0, 20.0, 22.0, 8.0, 38.0, 22.0, 30.0, 15.0,
                          28.0, 14.0, 24.0, 10.0, 33.0, 16.0, 26.0, 12.0, 30.0, 18.0, 20.0, 10.0, 36.0, 20.0, 28.0, 14.0]
    d["strength_values"] = create_strength_pattern("ambient")
    d["root_note"] = 55
    d["notes"] = [note_to_dict(Note(55, 100)), note_to_dict(Note(60, 60)), note_to_dict(Note(62, 55)),
                  note_to_dict(Note(67, 50)), note_to_dict(Note(72, 40))]
    d["synth_osc_d"] = 0.52
    d["synth_osc_v"] = 0.68
    d["synth_osc_stereo_v_offset"] = 0.18
    d["synth_osc_volume"] = 0.55
    d["synth_filter_cutoff"] = 5000.0
    d["synth_filter_resonance"] = 0.12
    d["synth_filter_env_amount"] = 1800.0
    d["synth_vol_attack"] = 1.0
    d["synth_vol_decay"] = 100.0
    d["synth_vol_sustain"] = 0.15
    d["synth_vol_release"] = 150.0
    d["synth_reverb_mix"] = 0.22
    d["synth_reverb_decay"] = 0.65
    d["note_length_percent"] = 30.0
    presets.append(create_preset("Granular Fields", "Factory",
        "Scattered particles - tiny grains swarm like dust in sunlight", d))

    # 18. Meditative Pulse - Breathing rhythm
    d = create_default_preset()
    d["straight_1_2"] = [100.0, 90.0]
    d["straight_1_4"] = [0.0, 0.0, 0.0, 0.0]
    d["strength_values"] = create_strength_pattern("sparse")
    d["root_note"] = 36
    d["notes"] = [note_to_dict(Note(36, 100)), note_to_dict(Note(43, 50)), note_to_dict(Note(48, 45))]
    d["synth_osc_d"] = 0.1
    d["synth_osc_v"] = 0.4
    d["synth_osc_volume"] = 0.55
    d["synth_sub_volume"] = 0.35
    d["synth_filter_cutoff"] = 800.0
    d["synth_filter_resonance"] = 0.25
    d["synth_filter_env_amount"] = 300.0
    d["synth_vol_attack"] = 300.0
    d["synth_vol_decay"] = 1500.0
    d["synth_vol_sustain"] = 0.7
    d["synth_vol_release"] = 2500.0
    d["synth_reverb_mix"] = 0.18
    d["synth_reverb_decay"] = 0.6
    d["note_length_percent"] = 200.0
    d["lfo1_rate"] = 0.08
    d["lfo1_waveform"] = 0
    d["lfo1_dest1"] = 12
    d["lfo1_amount1"] = 0.15
    presets.append(create_preset("Meditative Pulse", "Factory",
        "Breathing rhythm - slow inhale exhale cycle for focus", d))

    # 19. Glitch Garden - Digital errors
    d = create_default_preset()
    d["straight_1_8"] = [95.0, 0.0, 80.0, 0.0, 90.0, 0.0, 75.0, 0.0]
    d["straight_1_32"] = [50.0, 45.0, 0.0, 0.0, 55.0, 40.0, 0.0, 0.0, 48.0, 42.0, 0.0, 0.0, 52.0, 38.0, 0.0, 0.0,
                          60.0, 50.0, 0.0, 0.0, 45.0, 35.0, 0.0, 0.0, 55.0, 48.0, 0.0, 0.0, 50.0, 40.0, 0.0, 0.0]
    d["strength_values"] = create_strength_pattern("syncopated")
    d["root_note"] = 48
    d["notes"] = [note_to_dict(Note(48, 100)), note_to_dict(Note(51, 60)), note_to_dict(Note(53, 55)),
                  note_to_dict(Note(60, 45)), note_to_dict(Note(63, 35))]
    d["synth_osc_d"] = 0.7
    d["synth_osc_v"] = 0.3
    d["synth_osc_volume"] = 0.72
    d["synth_filter_cutoff"] = 4500.0
    d["synth_filter_resonance"] = 0.35
    d["synth_filter_env_amount"] = 3000.0
    d["synth_vol_attack"] = 0.5
    d["synth_vol_decay"] = 50.0
    d["synth_vol_sustain"] = 0.1
    d["synth_vol_release"] = 80.0
    d["note_length_percent"] = 20.0
    presets.append(create_preset("Glitch Garden", "Factory",
        "Digital debris - micro-edits and stutters bloom into pattern", d))

    # 20. Slow Cinema - Atmospheric storytelling
    d = create_default_preset()
    d["straight_1_1"] = [80.0]
    d["straight_1_2"] = [60.0, 50.0]
    d["straight_1_4"] = [40.0, 30.0, 35.0, 25.0]
    d["strength_values"] = create_strength_pattern("sparse")
    d["root_note"] = 41
    d["notes"] = [note_to_dict(Note(41, 100)), note_to_dict(Note(48, 55)), note_to_dict(Note(53, 50)),
                  note_to_dict(Note(56, 45)), note_to_dict(Note(60, 40))]
    d["synth_osc_d"] = 0.22
    d["synth_osc_v"] = 0.62
    d["synth_osc_stereo_v_offset"] = 0.12
    d["synth_osc_volume"] = 0.58
    d["synth_filter_cutoff"] = 2000.0
    d["synth_filter_resonance"] = 0.08
    d["synth_filter_env_amount"] = 400.0
    d["synth_vol_attack"] = 120.0
    d["synth_vol_decay"] = 800.0
    d["synth_vol_sustain"] = 0.55
    d["synth_vol_release"] = 1200.0
    d["synth_reverb_mix"] = 0.2
    d["synth_reverb_decay"] = 0.68
    d["note_length_percent"] = 180.0
    presets.append(create_preset("Slow Cinema", "Factory",
        "Atmospheric score - sparse melodies for contemplative scenes", d))

    # 21. Mutation Engine - Evolving patterns
    d = create_default_preset()
    d["straight_1_8"] = [85.0, 60.0, 70.0, 50.0, 80.0, 55.0, 75.0, 45.0]
    d["straight_1_16"] = [55.0, 40.0, 50.0, 35.0, 60.0, 45.0, 48.0, 30.0, 58.0, 42.0, 52.0, 38.0, 62.0, 48.0, 55.0, 32.0]
    d["triplet_1_8t"] = [45.0, 35.0, 40.0, 50.0, 30.0, 45.0, 55.0, 38.0, 42.0, 48.0, 32.0, 40.0]
    d["strength_values"] = create_strength_pattern("polyrhythm_3_4")
    d["root_note"] = 45
    d["notes"] = [note_to_dict(Note(45, 100)), note_to_dict(Note(48, 60)), note_to_dict(Note(50, 55)),
                  note_to_dict(Note(52, 50)), note_to_dict(Note(55, 45)), note_to_dict(Note(57, 40))]
    d["octave_randomization"] = create_octave_randomization(0.3, 0.4, 0.3, "Up")
    d["synth_osc_d"] = 0.45
    d["synth_osc_v"] = 0.55
    d["synth_osc_volume"] = 0.68
    d["synth_filter_cutoff"] = 2800.0
    d["synth_filter_resonance"] = 0.18
    d["synth_filter_env_amount"] = 1200.0
    d["synth_vol_attack"] = 3.0
    d["synth_vol_decay"] = 200.0
    d["synth_vol_sustain"] = 0.45
    d["synth_vol_release"] = 280.0
    d["note_length_percent"] = 70.0
    presets.append(create_preset("Mutation Engine", "Factory",
        "Evolving creature - octave jumps create genetic variations", d))

    # 22. Static Field - White noise textures
    d = create_default_preset()
    d["straight_1_4"] = [100.0, 85.0, 90.0, 80.0]
    d["straight_1_8"] = [70.0, 55.0, 65.0, 50.0, 75.0, 60.0, 68.0, 52.0]
    d["strength_values"] = create_strength_pattern("driving")
    d["root_note"] = 36
    d["notes"] = [note_to_dict(Note(36, 100)), note_to_dict(Note(43, 50))]
    d["synth_osc_d"] = 0.6
    d["synth_osc_v"] = 0.25
    d["synth_osc_volume"] = 0.5
    d["synth_sub_volume"] = 0.35
    d["synth_noise_amount"] = 0.35
    d["synth_filter_cutoff"] = 1500.0
    d["synth_filter_resonance"] = 0.45
    d["synth_filter_env_amount"] = 800.0
    d["synth_vol_attack"] = 5.0
    d["synth_vol_decay"] = 150.0
    d["synth_vol_sustain"] = 0.35
    d["synth_vol_release"] = 200.0
    d["note_length_percent"] = 60.0
    presets.append(create_preset("Static Field", "Factory",
        "Noisy foundation - filtered noise creates textural rhythm", d))

    # 23. Harmonic Series - Overtone exploration
    d = create_default_preset()
    d["straight_1_4"] = [100.0, 80.0, 90.0, 75.0]
    d["straight_1_8"] = [65.0, 45.0, 55.0, 40.0, 70.0, 50.0, 60.0, 42.0]
    d["strength_values"] = create_strength_pattern("4_4_standard")
    d["root_note"] = 36
    d["notes"] = [note_to_dict(Note(36, 100)), note_to_dict(Note(43, 55)), note_to_dict(Note(48, 50)),
                  note_to_dict(Note(52, 45)), note_to_dict(Note(55, 40)), note_to_dict(Note(57, 35)),
                  note_to_dict(Note(60, 30))]
    d["synth_osc_d"] = 0.3
    d["synth_osc_v"] = 0.65
    d["synth_osc_volume"] = 0.65
    d["synth_sub_volume"] = 0.3
    d["synth_filter_cutoff"] = 3500.0
    d["synth_filter_resonance"] = 0.12
    d["synth_filter_env_amount"] = 1000.0
    d["synth_vol_attack"] = 2.0
    d["synth_vol_decay"] = 250.0
    d["synth_vol_sustain"] = 0.5
    d["synth_vol_release"] = 350.0
    d["note_length_percent"] = 90.0
    presets.append(create_preset("Harmonic Series", "Factory",
        "Overtone study - notes follow natural harmonic relationships", d))

    # 24. Broken Clock - Stuttering time
    d = create_default_preset()
    d["straight_1_8"] = [100.0, 95.0, 0.0, 0.0, 100.0, 0.0, 90.0, 0.0]
    d["straight_1_16"] = [0.0, 0.0, 85.0, 80.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 75.0, 70.0, 0.0, 0.0, 0.0, 0.0]
    d["strength_values"] = create_strength_pattern("syncopated")
    d["root_note"] = 48
    d["notes"] = [note_to_dict(Note(48, 100)), note_to_dict(Note(52, 55)), note_to_dict(Note(55, 50)),
                  note_to_dict(Note(60, 45))]
    d["synth_osc_d"] = 0.55
    d["synth_osc_v"] = 0.45
    d["synth_osc_volume"] = 0.72
    d["synth_filter_cutoff"] = 3200.0
    d["synth_filter_resonance"] = 0.22
    d["synth_filter_env_amount"] = 1600.0
    d["synth_vol_attack"] = 1.0
    d["synth_vol_decay"] = 100.0
    d["synth_vol_sustain"] = 0.3
    d["synth_vol_release"] = 150.0
    d["note_length_percent"] = 45.0
    presets.append(create_preset("Broken Clock", "Factory",
        "Mechanical malfunction - clock ticks skip and stutter", d))

    # 25. Vapor Trail - Misty atmospherics
    d = create_default_preset()
    d["straight_1_2"] = [90.0, 75.0]
    d["straight_1_4"] = [55.0, 40.0, 50.0, 35.0]
    d["dotted_1_4d"] = [45.0, 30.0, 38.0]
    d["strength_values"] = create_strength_pattern("ambient")
    d["root_note"] = 48
    d["notes"] = [note_to_dict(Note(48, 100)), note_to_dict(Note(52, 58)), note_to_dict(Note(55, 55)),
                  note_to_dict(Note(60, 50)), note_to_dict(Note(64, 45)), note_to_dict(Note(67, 38))]
    d["synth_osc_d"] = 0.18
    d["synth_osc_v"] = 0.72
    d["synth_osc_stereo_v_offset"] = 0.15
    d["synth_osc_volume"] = 0.55
    d["synth_filter_cutoff"] = 2800.0
    d["synth_filter_resonance"] = 0.08
    d["synth_filter_env_amount"] = 350.0
    d["synth_vol_attack"] = 180.0
    d["synth_vol_decay"] = 1000.0
    d["synth_vol_sustain"] = 0.6
    d["synth_vol_release"] = 1500.0
    d["synth_reverb_mix"] = 0.25
    d["synth_reverb_decay"] = 0.75
    d["synth_reverb_diffusion"] = 0.9
    d["note_length_percent"] = 200.0
    presets.append(create_preset("Vapor Trail", "Factory",
        "Misty echoes - notes dissolve into foggy atmosphere", d))

    # 26. Cellular Automata - Rule-based patterns
    d = create_default_preset()
    d["straight_1_16"] = [100.0, 0.0, 80.0, 0.0, 0.0, 85.0, 0.0, 75.0, 90.0, 0.0, 0.0, 70.0, 0.0, 95.0, 0.0, 0.0]
    d["strength_values"] = create_strength_pattern("dense")
    d["root_note"] = 48
    d["notes"] = [note_to_dict(Note(48, 100)), note_to_dict(Note(50, 55)), note_to_dict(Note(52, 50)),
                  note_to_dict(Note(55, 45)), note_to_dict(Note(57, 40))]
    d["synth_osc_d"] = 0.48
    d["synth_osc_v"] = 0.52
    d["synth_osc_volume"] = 0.7
    d["synth_filter_cutoff"] = 3800.0
    d["synth_filter_resonance"] = 0.2
    d["synth_filter_env_amount"] = 1400.0
    d["synth_vol_attack"] = 1.0
    d["synth_vol_decay"] = 120.0
    d["synth_vol_sustain"] = 0.35
    d["synth_vol_release"] = 180.0
    d["note_length_percent"] = 50.0
    presets.append(create_preset("Cellular Automata", "Factory",
        "Rule-based life - patterns evolve by simple rules into complexity", d))

    # 27. Submarine - Deep underwater
    d = create_default_preset()
    d["straight_1_2"] = [100.0, 85.0]
    d["straight_1_4"] = [70.0, 55.0, 65.0, 50.0]
    d["strength_values"] = create_strength_pattern("sparse")
    d["root_note"] = 33
    d["notes"] = [note_to_dict(Note(33, 100)), note_to_dict(Note(40, 55)), note_to_dict(Note(45, 50))]
    d["synth_osc_d"] = 0.08
    d["synth_osc_v"] = 0.35
    d["synth_osc_volume"] = 0.55
    d["synth_sub_volume"] = 0.45
    d["synth_filter_cutoff"] = 400.0
    d["synth_filter_resonance"] = 0.5
    d["synth_filter_env_amount"] = 200.0
    d["synth_vol_attack"] = 150.0
    d["synth_vol_decay"] = 1200.0
    d["synth_vol_sustain"] = 0.65
    d["synth_vol_release"] = 2000.0
    d["synth_reverb_mix"] = 0.2
    d["synth_reverb_decay"] = 0.7
    d["note_length_percent"] = 200.0
    d["lfo1_rate"] = 0.04
    d["lfo1_waveform"] = 0
    d["lfo1_dest1"] = 12
    d["lfo1_amount1"] = 0.2
    presets.append(create_preset("Submarine", "Factory",
        "Deep diving - pressure and darkness in the abyss", d))

    # 28. Fireflies - Sporadic sparkles
    d = create_default_preset()
    d["straight_1_8"] = [40.0, 25.0, 35.0, 20.0, 45.0, 30.0, 38.0, 22.0]
    d["straight_1_16"] = [30.0, 15.0, 25.0, 12.0, 35.0, 20.0, 28.0, 10.0, 32.0, 18.0, 22.0, 8.0, 38.0, 22.0, 30.0, 14.0]
    d["strength_values"] = create_strength_pattern("ambient")
    d["root_note"] = 60
    d["notes"] = [note_to_dict(Note(60, 100)), note_to_dict(Note(64, 60)), note_to_dict(Note(67, 55)),
                  note_to_dict(Note(72, 50)), note_to_dict(Note(76, 45)), note_to_dict(Note(79, 35))]
    d["octave_randomization"] = create_octave_randomization(0.4, 0.6, 0.4, "Up")
    d["synth_osc_d"] = 0.6
    d["synth_osc_v"] = 0.75
    d["synth_osc_stereo_v_offset"] = 0.2
    d["synth_osc_volume"] = 0.6
    d["synth_filter_cutoff"] = 6500.0
    d["synth_filter_resonance"] = 0.15
    d["synth_filter_env_amount"] = 2500.0
    d["synth_vol_attack"] = 0.5
    d["synth_vol_decay"] = 80.0
    d["synth_vol_sustain"] = 0.1
    d["synth_vol_release"] = 200.0
    d["synth_reverb_mix"] = 0.22
    d["synth_reverb_decay"] = 0.6
    d["note_length_percent"] = 35.0
    presets.append(create_preset("Fireflies", "Factory",
        "Summer night sparkles - brief flashes dance randomly", d))

    # 29. Gravity Well - Pulled toward center
    d = create_default_preset()
    d["straight_1_4"] = [100.0, 80.0, 90.0, 75.0]
    d["straight_1_8"] = [65.0, 50.0, 60.0, 45.0, 70.0, 55.0, 62.0, 48.0]
    d["straight_1_16"] = [45.0, 35.0, 42.0, 30.0, 50.0, 38.0, 45.0, 32.0, 48.0, 36.0, 44.0, 33.0, 52.0, 40.0, 47.0, 35.0]
    d["strength_values"] = create_strength_pattern("bass_heavy")
    d["root_note"] = 36
    d["notes"] = [note_to_dict(Note(36, 100)), note_to_dict(Note(41, 55)), note_to_dict(Note(43, 50)),
                  note_to_dict(Note(48, 45)), note_to_dict(Note(53, 40))]
    d["synth_osc_d"] = 0.35
    d["synth_osc_v"] = 0.42
    d["synth_osc_volume"] = 0.7
    d["synth_sub_volume"] = 0.35
    d["synth_filter_cutoff"] = 1800.0
    d["synth_filter_resonance"] = 0.3
    d["synth_filter_env_amount"] = 600.0
    d["synth_vol_attack"] = 5.0
    d["synth_vol_decay"] = 300.0
    d["synth_vol_sustain"] = 0.5
    d["synth_vol_release"] = 400.0
    d["note_length_percent"] = 90.0
    presets.append(create_preset("Gravity Well", "Factory",
        "Massive bass pull - everything falls toward the fundamental", d))

    # 30. Northern Lights - Shimmering aurora
    d = create_default_preset()
    d["straight_1_2"] = [100.0, 80.0]
    d["straight_1_4"] = [60.0, 45.0, 55.0, 40.0]
    d["straight_1_8"] = [35.0, 25.0, 30.0, 20.0, 40.0, 28.0, 32.0, 22.0]
    d["strength_values"] = create_strength_pattern("ambient")
    d["root_note"] = 55
    d["notes"] = [note_to_dict(Note(55, 100)), note_to_dict(Note(60, 60)), note_to_dict(Note(62, 55)),
                  note_to_dict(Note(64, 50)), note_to_dict(Note(67, 45)), note_to_dict(Note(72, 35))]
    d["synth_osc_d"] = 0.25
    d["synth_osc_v"] = 0.78
    d["synth_osc_stereo_v_offset"] = 0.22
    d["synth_osc_volume"] = 0.55
    d["synth_filter_cutoff"] = 4500.0
    d["synth_filter_resonance"] = 0.1
    d["synth_filter_env_amount"] = 600.0
    d["synth_vol_attack"] = 200.0
    d["synth_vol_decay"] = 1500.0
    d["synth_vol_sustain"] = 0.65
    d["synth_vol_release"] = 2500.0
    d["synth_reverb_mix"] = 0.25
    d["synth_reverb_decay"] = 0.8
    d["note_length_percent"] = 200.0
    d["lfo1_rate"] = 0.06
    d["lfo1_waveform"] = 1
    d["lfo1_dest1"] = 10
    d["lfo1_amount1"] = 0.15
    d["lfo2_rate"] = 0.03
    d["lfo2_waveform"] = 0
    d["lfo2_dest1"] = 11
    d["lfo2_amount1"] = 0.12
    presets.append(create_preset("Northern Lights", "Factory",
        "Arctic shimmer - slow dancing waves of colored light", d))

    # 31. Clockwork Orange - Mechanical quirk
    d = create_default_preset()
    euc_clock = euclidean_rhythm(16, 7)
    for i in range(16):
        d["straight_1_16"][i] = 90.0 if i in euc_clock else 35.0
    d["triplet_1_8t"] = [70.0, 0.0, 55.0, 65.0, 0.0, 50.0, 75.0, 0.0, 60.0, 68.0, 0.0, 52.0]
    d["strength_values"] = create_strength_pattern("syncopated")
    d["root_note"] = 48
    d["notes"] = [note_to_dict(Note(48, 100)), note_to_dict(Note(51, 55)), note_to_dict(Note(55, 50)),
                  note_to_dict(Note(58, 45)), note_to_dict(Note(60, 40))]
    d["synth_osc_d"] = 0.58
    d["synth_osc_v"] = 0.48
    d["synth_osc_volume"] = 0.7
    d["synth_filter_cutoff"] = 3500.0
    d["synth_filter_resonance"] = 0.25
    d["synth_filter_env_amount"] = 1800.0
    d["synth_vol_attack"] = 1.0
    d["synth_vol_decay"] = 100.0
    d["synth_vol_sustain"] = 0.3
    d["synth_vol_release"] = 150.0
    d["swing_amount"] = 55.0
    d["note_length_percent"] = 50.0
    presets.append(create_preset("Clockwork Orange", "Factory",
        "Mechanical oddity - intricate euclidean gears interlock", d))

    # 32. Event Horizon - Black hole edge
    d = create_default_preset()
    d["straight_1_1"] = [100.0]
    d["straight_1_2"] = [75.0, 65.0]
    d["straight_1_4"] = [50.0, 40.0, 45.0, 35.0]
    d["straight_1_8"] = [35.0, 25.0, 30.0, 20.0, 38.0, 28.0, 32.0, 22.0]
    d["strength_values"] = create_strength_pattern("sparse")
    d["root_note"] = 33
    d["notes"] = [note_to_dict(Note(33, 100)), note_to_dict(Note(40, 50)), note_to_dict(Note(45, 45)),
                  note_to_dict(Note(52, 40))]
    d["synth_osc_d"] = 0.15
    d["synth_osc_v"] = 0.3
    d["synth_osc_volume"] = 0.6
    d["synth_sub_volume"] = 0.5
    d["synth_filter_cutoff"] = 500.0
    d["synth_filter_resonance"] = 0.55
    d["synth_filter_env_amount"] = 150.0
    d["synth_vol_attack"] = 250.0
    d["synth_vol_decay"] = 2000.0
    d["synth_vol_sustain"] = 0.6
    d["synth_vol_release"] = 3500.0
    d["synth_reverb_mix"] = 0.22
    d["synth_reverb_decay"] = 0.75
    d["note_length_percent"] = 200.0
    d["lfo1_rate"] = 0.02
    d["lfo1_waveform"] = 0
    d["lfo1_dest1"] = 13
    d["lfo1_amount1"] = 0.25
    presets.append(create_preset("Event Horizon", "Factory",
        "Crossing point - time stretches at the edge of oblivion", d))

    return presets

def create_bank_e() -> List[Dict]:
    """Bank E: Psychedelic & Space - 32 presets"""
    presets = []

    # 1. Acid Trip - Classic 303-inspired
    d = create_default_preset()
    d["straight_1_16"] = [100.0, 75.0, 85.0, 70.0, 95.0, 72.0, 80.0, 65.0, 90.0, 78.0, 82.0, 68.0, 88.0, 74.0, 86.0, 60.0]
    d["strength_values"] = create_strength_pattern("driving")
    d["root_note"] = 36
    d["notes"] = [note_to_dict(Note(36, 100)), note_to_dict(Note(39, 60)), note_to_dict(Note(41, 55)),
                  note_to_dict(Note(43, 50)), note_to_dict(Note(48, 45))]
    d["octave_randomization"] = create_octave_randomization(0.25, 0.5, 0.3, "Up")
    d["synth_osc_d"] = 0.65
    d["synth_osc_v"] = 0.35
    d["synth_osc_volume"] = 0.75
    d["synth_filter_cutoff"] = 800.0
    d["synth_filter_resonance"] = 0.7
    d["synth_filter_env_amount"] = 4000.0
    d["synth_vol_attack"] = 1.0
    d["synth_vol_decay"] = 150.0
    d["synth_vol_sustain"] = 0.3
    d["synth_vol_release"] = 180.0
    d["note_length_percent"] = 60.0
    d["lfo1_tempo_sync"] = True
    d["lfo1_sync_division"] = 2
    d["lfo1_waveform"] = 2
    d["lfo1_dest1"] = 12
    d["lfo1_amount1"] = 0.35
    presets.append(create_preset("Acid Trip", "Factory",
        "Classic squelch - resonant filter sweeps that melt reality", d))

    # 2. Cosmic Rays - Particle streams
    d = create_default_preset()
    d["straight_1_32"] = [60.0, 35.0, 50.0, 30.0, 55.0, 40.0, 45.0, 25.0, 58.0, 38.0, 52.0, 32.0, 62.0, 42.0, 48.0, 28.0,
                          65.0, 36.0, 54.0, 34.0, 57.0, 44.0, 46.0, 26.0, 60.0, 40.0, 50.0, 30.0, 68.0, 45.0, 55.0, 35.0]
    d["strength_values"] = create_strength_pattern("ambient")
    d["root_note"] = 60
    d["notes"] = [note_to_dict(Note(60, 100)), note_to_dict(Note(64, 55)), note_to_dict(Note(67, 50)),
                  note_to_dict(Note(72, 45)), note_to_dict(Note(76, 40)), note_to_dict(Note(79, 35))]
    d["octave_randomization"] = create_octave_randomization(0.5, 0.7, 0.5, "Both")
    d["synth_osc_d"] = 0.55
    d["synth_osc_v"] = 0.72
    d["synth_osc_stereo_v_offset"] = 0.2
    d["synth_osc_volume"] = 0.6
    d["synth_filter_cutoff"] = 5500.0
    d["synth_filter_resonance"] = 0.15
    d["synth_filter_env_amount"] = 2200.0
    d["synth_vol_attack"] = 0.5
    d["synth_vol_decay"] = 60.0
    d["synth_vol_sustain"] = 0.1
    d["synth_vol_release"] = 150.0
    d["synth_reverb_mix"] = 0.22
    d["synth_reverb_decay"] = 0.65
    d["note_length_percent"] = 25.0
    presets.append(create_preset("Cosmic Rays", "Factory",
        "High energy particles - rapid bright notes streak across space", d))

    # 3. Mushroom Forest - Organic and weird
    d = create_default_preset()
    d["straight_1_8"] = [90.0, 60.0, 75.0, 50.0, 85.0, 55.0, 70.0, 45.0]
    d["triplet_1_8t"] = [55.0, 40.0, 50.0, 60.0, 35.0, 45.0, 58.0, 42.0, 48.0, 52.0, 38.0, 55.0]
    d["strength_values"] = create_strength_pattern("polyrhythm_3_4")
    d["root_note"] = 41
    d["notes"] = [note_to_dict(Note(41, 100)), note_to_dict(Note(44, 55)), note_to_dict(Note(48, 50)),
                  note_to_dict(Note(51, 45)), note_to_dict(Note(53, 40)), note_to_dict(Note(56, 35))]
    d["synth_osc_d"] = 0.38
    d["synth_osc_v"] = 0.58
    d["synth_osc_stereo_v_offset"] = 0.15
    d["synth_osc_volume"] = 0.68
    d["synth_filter_cutoff"] = 2200.0
    d["synth_filter_resonance"] = 0.28
    d["synth_filter_env_amount"] = 1400.0
    d["synth_vol_attack"] = 8.0
    d["synth_vol_decay"] = 280.0
    d["synth_vol_sustain"] = 0.45
    d["synth_vol_release"] = 400.0
    d["synth_reverb_mix"] = 0.18
    d["synth_reverb_decay"] = 0.55
    d["note_length_percent"] = 85.0
    d["lfo1_rate"] = 0.15
    d["lfo1_waveform"] = 4
    d["lfo1_dest1"] = 11
    d["lfo1_amount1"] = 0.2
    presets.append(create_preset("Mushroom Forest", "Factory",
        "Organic weirdness - strange shapes grow in rhythm", d))

    # 4. Solar Wind - Sustained brightness
    d = create_default_preset()
    d["straight_1_2"] = [100.0, 85.0]
    d["straight_1_4"] = [70.0, 55.0, 65.0, 50.0]
    d["straight_1_8"] = [45.0, 30.0, 40.0, 25.0, 50.0, 35.0, 42.0, 28.0]
    d["strength_values"] = create_strength_pattern("ambient")
    d["root_note"] = 60
    d["notes"] = [note_to_dict(Note(60, 100)), note_to_dict(Note(64, 60)), note_to_dict(Note(67, 55)),
                  note_to_dict(Note(72, 50)), note_to_dict(Note(76, 40))]
    d["synth_osc_d"] = 0.25
    d["synth_osc_v"] = 0.8
    d["synth_osc_stereo_v_offset"] = 0.18
    d["synth_osc_volume"] = 0.55
    d["synth_filter_cutoff"] = 4500.0
    d["synth_filter_resonance"] = 0.08
    d["synth_filter_env_amount"] = 500.0
    d["synth_vol_attack"] = 150.0
    d["synth_vol_decay"] = 1200.0
    d["synth_vol_sustain"] = 0.65
    d["synth_vol_release"] = 2000.0
    d["synth_reverb_mix"] = 0.25
    d["synth_reverb_decay"] = 0.78
    d["synth_reverb_diffusion"] = 0.9
    d["note_length_percent"] = 200.0
    presets.append(create_preset("Solar Wind", "Factory",
        "Stellar pressure - sustained bright tones push through void", d))

    # 5. DMT Entity - Alien encounter
    d = create_default_preset()
    d["straight_1_8"] = [100.0, 0.0, 80.0, 0.0, 90.0, 0.0, 75.0, 0.0]
    d["straight_1_16"] = [65.0, 50.0, 0.0, 45.0, 70.0, 55.0, 0.0, 40.0, 60.0, 48.0, 0.0, 52.0, 68.0, 58.0, 0.0, 42.0]
    d["triplet_1_16t"] = [55.0, 40.0, 0.0, 50.0, 35.0, 0.0, 58.0, 42.0, 0.0, 52.0, 38.0, 0.0,
                          60.0, 45.0, 0.0, 55.0, 40.0, 0.0, 62.0, 48.0, 0.0, 50.0, 35.0, 0.0]
    d["strength_values"] = create_strength_pattern("syncopated")
    d["root_note"] = 48
    d["notes"] = [note_to_dict(Note(48, 100)), note_to_dict(Note(51, 60)), note_to_dict(Note(54, 55)),
                  note_to_dict(Note(57, 50)), note_to_dict(Note(60, 45)), note_to_dict(Note(63, 40))]
    d["synth_osc_d"] = 0.62
    d["synth_osc_v"] = 0.45
    d["synth_osc_stereo_v_offset"] = 0.22
    d["synth_osc_volume"] = 0.7
    d["synth_filter_cutoff"] = 3500.0
    d["synth_filter_resonance"] = 0.35
    d["synth_filter_env_amount"] = 2500.0
    d["synth_vol_attack"] = 2.0
    d["synth_vol_decay"] = 150.0
    d["synth_vol_sustain"] = 0.35
    d["synth_vol_release"] = 220.0
    d["synth_reverb_mix"] = 0.2
    d["synth_reverb_decay"] = 0.6
    d["note_length_percent"] = 55.0
    d["lfo1_tempo_sync"] = True
    d["lfo1_sync_division"] = 5
    d["lfo1_waveform"] = 4
    d["lfo1_dest1"] = 10
    d["lfo1_amount1"] = 0.25
    presets.append(create_preset("DMT Entity", "Factory",
        "Alien presence - fractalized patterns communicate in code", d))

    # 6. Astral Projection - Out of body
    d = create_default_preset()
    d["straight_1_1"] = [100.0]
    d["straight_1_2"] = [75.0, 65.0]
    d["straight_1_4"] = [45.0, 35.0, 40.0, 30.0]
    d["strength_values"] = create_strength_pattern("sparse")
    d["root_note"] = 48
    d["notes"] = [note_to_dict(Note(48, 100)), note_to_dict(Note(52, 55)), note_to_dict(Note(55, 50)),
                  note_to_dict(Note(60, 45)), note_to_dict(Note(64, 40)), note_to_dict(Note(67, 35))]
    d["synth_osc_d"] = 0.15
    d["synth_osc_v"] = 0.75
    d["synth_osc_stereo_v_offset"] = 0.2
    d["synth_osc_volume"] = 0.55
    d["synth_filter_cutoff"] = 3000.0
    d["synth_filter_resonance"] = 0.1
    d["synth_filter_env_amount"] = 400.0
    d["synth_vol_attack"] = 200.0
    d["synth_vol_decay"] = 1500.0
    d["synth_vol_sustain"] = 0.6
    d["synth_vol_release"] = 2500.0
    d["synth_reverb_mix"] = 0.25
    d["synth_reverb_decay"] = 0.82
    d["synth_reverb_diffusion"] = 0.92
    d["note_length_percent"] = 200.0
    d["lfo1_rate"] = 0.04
    d["lfo1_waveform"] = 0
    d["lfo1_dest1"] = 11
    d["lfo1_amount1"] = 0.15
    presets.append(create_preset("Astral Projection", "Factory",
        "Floating free - consciousness detaches from form", d))

    # 7. Wormhole Transit - Interdimensional travel
    d = create_default_preset()
    d["straight_1_4"] = [100.0, 90.0, 95.0, 85.0]
    d["straight_1_8"] = [80.0, 70.0, 75.0, 65.0, 85.0, 72.0, 78.0, 68.0]
    d["straight_1_16"] = [60.0, 50.0, 55.0, 45.0, 65.0, 52.0, 58.0, 48.0, 62.0, 54.0, 57.0, 47.0, 68.0, 55.0, 60.0, 50.0]
    d["strength_values"] = create_strength_pattern("driving")
    d["root_note"] = 36
    d["notes"] = [note_to_dict(Note(36, 100)), note_to_dict(Note(41, 55)), note_to_dict(Note(43, 50)),
                  note_to_dict(Note(48, 45))]
    d["synth_osc_d"] = 0.5
    d["synth_osc_v"] = 0.4
    d["synth_osc_volume"] = 0.72
    d["synth_sub_volume"] = 0.28
    d["synth_filter_cutoff"] = 1500.0
    d["synth_filter_resonance"] = 0.4
    d["synth_filter_env_amount"] = 2000.0
    d["synth_vol_attack"] = 1.0
    d["synth_vol_decay"] = 200.0
    d["synth_vol_sustain"] = 0.5
    d["synth_vol_release"] = 280.0
    d["note_length_percent"] = 80.0
    d["lfo1_tempo_sync"] = True
    d["lfo1_sync_division"] = 0
    d["lfo1_waveform"] = 2
    d["lfo1_dest1"] = 12
    d["lfo1_amount1"] = 0.4
    presets.append(create_preset("Wormhole Transit", "Factory",
        "Tunnel vision - accelerating through spacetime fabric", d))

    # 8. Kaleidoscope - Fractured visuals
    d = create_default_preset()
    euc_kaleid = euclidean_rhythm(16, 9)
    for i in range(16):
        d["straight_1_16"][i] = 85.0 if i in euc_kaleid else 30.0
    d["triplet_1_8t"] = [70.0, 45.0, 55.0, 75.0, 40.0, 60.0, 68.0, 48.0, 52.0, 72.0, 42.0, 58.0]
    d["strength_values"] = create_strength_pattern("polyrhythm_3_4")
    d["root_note"] = 53
    d["notes"] = [note_to_dict(Note(53, 100)), note_to_dict(Note(56, 55)), note_to_dict(Note(60, 50)),
                  note_to_dict(Note(63, 45)), note_to_dict(Note(65, 40)), note_to_dict(Note(68, 35))]
    d["synth_osc_d"] = 0.48
    d["synth_osc_v"] = 0.62
    d["synth_osc_stereo_v_offset"] = 0.18
    d["synth_osc_volume"] = 0.68
    d["synth_filter_cutoff"] = 3200.0
    d["synth_filter_resonance"] = 0.22
    d["synth_filter_env_amount"] = 1600.0
    d["synth_vol_attack"] = 2.0
    d["synth_vol_decay"] = 180.0
    d["synth_vol_sustain"] = 0.4
    d["synth_vol_release"] = 250.0
    d["synth_reverb_mix"] = 0.18
    d["synth_reverb_decay"] = 0.55
    d["note_length_percent"] = 65.0
    presets.append(create_preset("Kaleidoscope", "Factory",
        "Fractured symmetry - patterns reflect and rotate endlessly", d))

    # 9. Nebula Drift - Interstellar clouds
    d = create_default_preset()
    d["straight_1_2"] = [100.0, 80.0]
    d["straight_1_4"] = [55.0, 40.0, 50.0, 35.0]
    d["dotted_1_4d"] = [45.0, 30.0, 38.0]
    d["strength_values"] = create_strength_pattern("ambient")
    d["root_note"] = 43
    d["notes"] = [note_to_dict(Note(43, 100)), note_to_dict(Note(48, 60)), note_to_dict(Note(50, 55)),
                  note_to_dict(Note(55, 50)), note_to_dict(Note(60, 45)), note_to_dict(Note(67, 35))]
    d["synth_osc_d"] = 0.18
    d["synth_osc_v"] = 0.72
    d["synth_osc_stereo_v_offset"] = 0.15
    d["synth_osc_volume"] = 0.55
    d["synth_filter_cutoff"] = 2800.0
    d["synth_filter_resonance"] = 0.08
    d["synth_filter_env_amount"] = 400.0
    d["synth_vol_attack"] = 180.0
    d["synth_vol_decay"] = 1200.0
    d["synth_vol_sustain"] = 0.6
    d["synth_vol_release"] = 1800.0
    d["synth_reverb_mix"] = 0.25
    d["synth_reverb_decay"] = 0.8
    d["synth_reverb_diffusion"] = 0.92
    d["note_length_percent"] = 200.0
    d["lfo1_rate"] = 0.03
    d["lfo1_waveform"] = 0
    d["lfo1_dest1"] = 25
    d["lfo1_amount1"] = 0.18
    presets.append(create_preset("Nebula Drift", "Factory",
        "Cosmic fog - slow currents carry through stellar nurseries", d))

    # 10. Synaptic Fire - Neural electricity
    d = create_default_preset()
    d["straight_1_16"] = [80.0, 45.0, 60.0, 35.0, 75.0, 50.0, 55.0, 30.0, 85.0, 48.0, 65.0, 38.0, 70.0, 52.0, 58.0, 32.0]
    d["straight_1_32"] = [45.0, 25.0, 35.0, 18.0, 50.0, 30.0, 40.0, 20.0, 48.0, 28.0, 38.0, 22.0, 52.0, 32.0, 42.0, 25.0,
                          47.0, 27.0, 37.0, 20.0, 55.0, 33.0, 43.0, 23.0, 50.0, 30.0, 40.0, 24.0, 58.0, 35.0, 45.0, 28.0]
    d["strength_values"] = create_strength_pattern("dense")
    d["root_note"] = 55
    d["notes"] = [note_to_dict(Note(55, 100)), note_to_dict(Note(58, 55)), note_to_dict(Note(60, 50)),
                  note_to_dict(Note(62, 45)), note_to_dict(Note(65, 40)), note_to_dict(Note(67, 35))]
    d["synth_osc_d"] = 0.58
    d["synth_osc_v"] = 0.52
    d["synth_osc_volume"] = 0.68
    d["synth_filter_cutoff"] = 4500.0
    d["synth_filter_resonance"] = 0.2
    d["synth_filter_env_amount"] = 2000.0
    d["synth_vol_attack"] = 0.5
    d["synth_vol_decay"] = 80.0
    d["synth_vol_sustain"] = 0.15
    d["synth_vol_release"] = 120.0
    d["synth_reverb_mix"] = 0.15
    d["synth_reverb_decay"] = 0.5
    d["note_length_percent"] = 30.0
    presets.append(create_preset("Synaptic Fire", "Factory",
        "Brain sparks - rapid neural cascades create thought", d))

    # 11. Void Walker - Empty space traveler
    d = create_default_preset()
    d["straight_1_1"] = [100.0]
    d["straight_1_2"] = [60.0, 50.0]
    d["straight_1_4"] = [35.0, 25.0, 30.0, 20.0]
    d["strength_values"] = create_strength_pattern("sparse")
    d["root_note"] = 33
    d["notes"] = [note_to_dict(Note(33, 100)), note_to_dict(Note(40, 50)), note_to_dict(Note(45, 45)),
                  note_to_dict(Note(52, 40))]
    d["synth_osc_d"] = 0.12
    d["synth_osc_v"] = 0.32
    d["synth_osc_volume"] = 0.6
    d["synth_sub_volume"] = 0.5
    d["synth_filter_cutoff"] = 450.0
    d["synth_filter_resonance"] = 0.55
    d["synth_filter_env_amount"] = 150.0
    d["synth_vol_attack"] = 250.0
    d["synth_vol_decay"] = 2000.0
    d["synth_vol_sustain"] = 0.65
    d["synth_vol_release"] = 3500.0
    d["synth_reverb_mix"] = 0.22
    d["synth_reverb_decay"] = 0.75
    d["note_length_percent"] = 200.0
    presets.append(create_preset("Void Walker", "Factory",
        "Absolute emptiness - solitary steps through nothing", d))

    # 12. Time Dilation - Relativity effects
    d = create_default_preset()
    d["straight_1_4"] = [100.0, 80.0, 90.0, 70.0]
    d["straight_1_8"] = [60.0, 45.0, 55.0, 40.0, 65.0, 50.0, 58.0, 42.0]
    d["triplet_1_4t"] = [75.0, 55.0, 65.0, 70.0, 50.0, 60.0]
    d["strength_values"] = create_strength_pattern("polyrhythm_3_4")
    d["root_note"] = 41
    d["notes"] = [note_to_dict(Note(41, 100)), note_to_dict(Note(45, 55)), note_to_dict(Note(48, 50)),
                  note_to_dict(Note(52, 45)), note_to_dict(Note(55, 40))]
    d["synth_osc_d"] = 0.3
    d["synth_osc_v"] = 0.6
    d["synth_osc_stereo_v_offset"] = 0.12
    d["synth_osc_volume"] = 0.65
    d["synth_filter_cutoff"] = 2400.0
    d["synth_filter_resonance"] = 0.15
    d["synth_filter_env_amount"] = 900.0
    d["synth_vol_attack"] = 10.0
    d["synth_vol_decay"] = 350.0
    d["synth_vol_sustain"] = 0.5
    d["synth_vol_release"] = 500.0
    d["synth_reverb_mix"] = 0.2
    d["synth_reverb_decay"] = 0.65
    d["note_length_percent"] = 100.0
    presets.append(create_preset("Time Dilation", "Factory",
        "Stretched moments - near lightspeed time perception shift", d))

    # 13. Plasma Storm - Charged particles
    d = create_default_preset()
    d["straight_1_8"] = [95.0, 70.0, 80.0, 60.0, 90.0, 75.0, 85.0, 65.0]
    d["straight_1_16"] = [70.0, 50.0, 60.0, 45.0, 75.0, 55.0, 65.0, 48.0, 72.0, 52.0, 62.0, 47.0, 78.0, 58.0, 68.0, 50.0]
    d["strength_values"] = create_strength_pattern("driving")
    d["root_note"] = 36
    d["notes"] = [note_to_dict(Note(36, 100)), note_to_dict(Note(39, 55)), note_to_dict(Note(43, 50)),
                  note_to_dict(Note(48, 45)), note_to_dict(Note(51, 40))]
    d["synth_osc_d"] = 0.65
    d["synth_osc_v"] = 0.38
    d["synth_osc_volume"] = 0.72
    d["synth_sub_volume"] = 0.2
    d["synth_filter_cutoff"] = 2000.0
    d["synth_filter_resonance"] = 0.38
    d["synth_filter_env_amount"] = 2200.0
    d["synth_vol_attack"] = 1.0
    d["synth_vol_decay"] = 150.0
    d["synth_vol_sustain"] = 0.4
    d["synth_vol_release"] = 200.0
    d["note_length_percent"] = 65.0
    d["lfo1_tempo_sync"] = True
    d["lfo1_sync_division"] = 3
    d["lfo1_waveform"] = 3
    d["lfo1_dest1"] = 13
    d["lfo1_amount1"] = 0.3
    presets.append(create_preset("Plasma Storm", "Factory",
        "Ionized fury - electromagnetic chaos erupts", d))

    # 14. Ego Death - Dissolution of self
    d = create_default_preset()
    d["straight_1_2"] = [100.0, 75.0]
    d["straight_1_4"] = [50.0, 35.0, 45.0, 30.0]
    d["dotted_1_4d"] = [40.0, 25.0, 32.0]
    d["strength_values"] = create_strength_pattern("ambient")
    d["root_note"] = 45
    d["notes"] = [note_to_dict(Note(45, 100)), note_to_dict(Note(48, 55)), note_to_dict(Note(52, 50)),
                  note_to_dict(Note(57, 45)), note_to_dict(Note(60, 40)), note_to_dict(Note(64, 35))]
    d["synth_osc_d"] = 0.2
    d["synth_osc_v"] = 0.7
    d["synth_osc_stereo_v_offset"] = 0.18
    d["synth_osc_volume"] = 0.55
    d["synth_filter_cutoff"] = 2200.0
    d["synth_filter_resonance"] = 0.1
    d["synth_filter_env_amount"] = 350.0
    d["synth_vol_attack"] = 250.0
    d["synth_vol_decay"] = 1500.0
    d["synth_vol_sustain"] = 0.55
    d["synth_vol_release"] = 2500.0
    d["synth_reverb_mix"] = 0.25
    d["synth_reverb_decay"] = 0.8
    d["synth_reverb_diffusion"] = 0.92
    d["note_length_percent"] = 200.0
    presets.append(create_preset("Ego Death", "Factory",
        "Identity dissolves - boundaries melt into unity", d))

    # 15. Quantum Foam - Spacetime texture
    d = create_default_preset()
    d["straight_1_32"] = [40.0, 25.0, 35.0, 20.0, 45.0, 30.0, 38.0, 22.0, 42.0, 28.0, 36.0, 24.0, 48.0, 32.0, 40.0, 26.0,
                          44.0, 27.0, 37.0, 21.0, 46.0, 31.0, 39.0, 23.0, 43.0, 29.0, 35.0, 25.0, 50.0, 33.0, 41.0, 28.0]
    d["strength_values"] = create_strength_pattern("ambient")
    d["root_note"] = 60
    d["notes"] = [note_to_dict(Note(60, 100)), note_to_dict(Note(62, 55)), note_to_dict(Note(64, 50)),
                  note_to_dict(Note(67, 45)), note_to_dict(Note(69, 40)), note_to_dict(Note(72, 35))]
    d["octave_randomization"] = create_octave_randomization(0.4, 0.6, 0.4, "Both")
    d["synth_osc_d"] = 0.55
    d["synth_osc_v"] = 0.65
    d["synth_osc_stereo_v_offset"] = 0.2
    d["synth_osc_volume"] = 0.58
    d["synth_filter_cutoff"] = 5500.0
    d["synth_filter_resonance"] = 0.12
    d["synth_filter_env_amount"] = 2000.0
    d["synth_vol_attack"] = 0.5
    d["synth_vol_decay"] = 60.0
    d["synth_vol_sustain"] = 0.1
    d["synth_vol_release"] = 120.0
    d["synth_reverb_mix"] = 0.2
    d["synth_reverb_decay"] = 0.6
    d["note_length_percent"] = 25.0
    presets.append(create_preset("Quantum Foam", "Factory",
        "Planck scale bubbles - spacetime itself fizzes", d))

    # 16. Third Eye - Inner vision
    d = create_default_preset()
    d["straight_1_4"] = [100.0, 85.0, 90.0, 80.0]
    d["straight_1_8"] = [65.0, 50.0, 60.0, 45.0, 70.0, 55.0, 62.0, 48.0]
    d["triplet_1_8t"] = [55.0, 40.0, 48.0, 58.0, 42.0, 50.0, 52.0, 38.0, 45.0, 60.0, 44.0, 52.0]
    d["strength_values"] = create_strength_pattern("triplet_feel")
    d["root_note"] = 48
    d["notes"] = [note_to_dict(Note(48, 100)), note_to_dict(Note(51, 55)), note_to_dict(Note(55, 50)),
                  note_to_dict(Note(58, 45)), note_to_dict(Note(60, 40)), note_to_dict(Note(63, 35))]
    d["synth_osc_d"] = 0.42
    d["synth_osc_v"] = 0.58
    d["synth_osc_stereo_v_offset"] = 0.15
    d["synth_osc_volume"] = 0.68
    d["synth_filter_cutoff"] = 2800.0
    d["synth_filter_resonance"] = 0.22
    d["synth_filter_env_amount"] = 1400.0
    d["synth_vol_attack"] = 5.0
    d["synth_vol_decay"] = 250.0
    d["synth_vol_sustain"] = 0.45
    d["synth_vol_release"] = 350.0
    d["synth_reverb_mix"] = 0.2
    d["synth_reverb_decay"] = 0.62
    d["note_length_percent"] = 80.0
    d["lfo1_tempo_sync"] = True
    d["lfo1_sync_division"] = 6
    d["lfo1_waveform"] = 0
    d["lfo1_dest1"] = 11
    d["lfo1_amount1"] = 0.18
    presets.append(create_preset("Third Eye", "Factory",
        "Inner sight - perception beyond ordinary senses", d))

    # 17. Hyperdrive - FTL engagement
    d = create_default_preset()
    d["straight_1_8"] = [100.0, 90.0, 95.0, 85.0, 100.0, 88.0, 92.0, 82.0]
    d["straight_1_16"] = [80.0, 70.0, 75.0, 65.0, 85.0, 72.0, 78.0, 68.0, 82.0, 74.0, 77.0, 67.0, 88.0, 75.0, 80.0, 70.0]
    d["strength_values"] = create_strength_pattern("driving")
    d["root_note"] = 36
    d["notes"] = [note_to_dict(Note(36, 100)), note_to_dict(Note(43, 50)), note_to_dict(Note(48, 45))]
    d["synth_osc_d"] = 0.55
    d["synth_osc_v"] = 0.42
    d["synth_osc_volume"] = 0.75
    d["synth_sub_volume"] = 0.35
    d["synth_filter_cutoff"] = 1200.0
    d["synth_filter_resonance"] = 0.35
    d["synth_filter_env_amount"] = 2500.0
    d["synth_vol_attack"] = 0.5
    d["synth_vol_decay"] = 100.0
    d["synth_vol_sustain"] = 0.4
    d["synth_vol_release"] = 150.0
    d["note_length_percent"] = 55.0
    d["lfo1_tempo_sync"] = True
    d["lfo1_sync_division"] = 1
    d["lfo1_waveform"] = 2
    d["lfo1_dest1"] = 12
    d["lfo1_amount1"] = 0.45
    presets.append(create_preset("Hyperdrive", "Factory",
        "Light speed jump - stars streak past in tunnels", d))

    # 18. Bardo State - Between worlds
    d = create_default_preset()
    d["straight_1_2"] = [100.0, 70.0]
    d["straight_1_4"] = [45.0, 30.0, 40.0, 25.0]
    d["triplet_1_4t"] = [50.0, 35.0, 42.0, 48.0, 32.0, 40.0]
    d["strength_values"] = create_strength_pattern("sparse")
    d["root_note"] = 45
    d["notes"] = [note_to_dict(Note(45, 100)), note_to_dict(Note(48, 55)), note_to_dict(Note(52, 50)),
                  note_to_dict(Note(55, 45)), note_to_dict(Note(60, 40)), note_to_dict(Note(64, 35))]
    d["synth_osc_d"] = 0.18
    d["synth_osc_v"] = 0.68
    d["synth_osc_stereo_v_offset"] = 0.15
    d["synth_osc_volume"] = 0.55
    d["synth_filter_cutoff"] = 2400.0
    d["synth_filter_resonance"] = 0.1
    d["synth_filter_env_amount"] = 400.0
    d["synth_vol_attack"] = 200.0
    d["synth_vol_decay"] = 1200.0
    d["synth_vol_sustain"] = 0.55
    d["synth_vol_release"] = 2000.0
    d["synth_reverb_mix"] = 0.25
    d["synth_reverb_decay"] = 0.78
    d["synth_reverb_diffusion"] = 0.9
    d["note_length_percent"] = 180.0
    presets.append(create_preset("Bardo State", "Factory",
        "Liminal space - hovering between states of being", d))

    # 19. Fractal Zoom - Infinite detail
    d = create_default_preset()
    d["straight_1_4"] = [100.0, 85.0, 90.0, 80.0]
    d["straight_1_8"] = [75.0, 60.0, 70.0, 55.0, 80.0, 65.0, 72.0, 58.0]
    d["straight_1_16"] = [55.0, 40.0, 50.0, 35.0, 60.0, 45.0, 52.0, 38.0, 58.0, 42.0, 48.0, 36.0, 62.0, 47.0, 54.0, 40.0]
    d["strength_values"] = create_strength_pattern("4_4_standard")
    d["root_note"] = 48
    d["notes"] = [note_to_dict(Note(48, 100)), note_to_dict(Note(50, 55)), note_to_dict(Note(52, 50)),
                  note_to_dict(Note(55, 60)), note_to_dict(Note(57, 45)), note_to_dict(Note(60, 40))]
    d["synth_osc_d"] = 0.45
    d["synth_osc_v"] = 0.55
    d["synth_osc_volume"] = 0.68
    d["synth_filter_cutoff"] = 3000.0
    d["synth_filter_resonance"] = 0.18
    d["synth_filter_env_amount"] = 1200.0
    d["synth_vol_attack"] = 2.0
    d["synth_vol_decay"] = 200.0
    d["synth_vol_sustain"] = 0.45
    d["synth_vol_release"] = 280.0
    d["note_length_percent"] = 70.0
    presets.append(create_preset("Fractal Zoom", "Factory",
        "Self-similar journey - patterns repeat at every scale", d))

    # 20. Dark Matter - Unseen mass
    d = create_default_preset()
    d["straight_1_2"] = [100.0, 80.0]
    d["straight_1_4"] = [55.0, 40.0, 50.0, 35.0]
    d["strength_values"] = create_strength_pattern("bass_heavy")
    d["root_note"] = 29
    d["notes"] = [note_to_dict(Note(29, 100)), note_to_dict(Note(36, 55)), note_to_dict(Note(41, 50)),
                  note_to_dict(Note(43, 45))]
    d["synth_osc_d"] = 0.08
    d["synth_osc_v"] = 0.28
    d["synth_osc_volume"] = 0.55
    d["synth_sub_volume"] = 0.55
    d["synth_filter_cutoff"] = 350.0
    d["synth_filter_resonance"] = 0.6
    d["synth_filter_env_amount"] = 100.0
    d["synth_vol_attack"] = 300.0
    d["synth_vol_decay"] = 2500.0
    d["synth_vol_sustain"] = 0.7
    d["synth_vol_release"] = 4000.0
    d["synth_reverb_mix"] = 0.2
    d["synth_reverb_decay"] = 0.7
    d["note_length_percent"] = 200.0
    presets.append(create_preset("Dark Matter", "Factory",
        "Hidden gravity - invisible forces shape visible universe", d))

    # 21. Synesthesia - Crossed senses
    d = create_default_preset()
    d["straight_1_8"] = [85.0, 60.0, 70.0, 50.0, 80.0, 55.0, 75.0, 45.0]
    d["straight_1_16"] = [55.0, 40.0, 50.0, 35.0, 60.0, 45.0, 48.0, 30.0, 58.0, 42.0, 52.0, 38.0, 62.0, 48.0, 55.0, 32.0]
    d["triplet_1_8t"] = [48.0, 35.0, 42.0, 52.0, 38.0, 45.0, 50.0, 32.0, 40.0, 55.0, 40.0, 48.0]
    d["strength_values"] = create_strength_pattern("triplet_feel")
    d["root_note"] = 53
    d["notes"] = [note_to_dict(Note(53, 100)), note_to_dict(Note(56, 55)), note_to_dict(Note(58, 50)),
                  note_to_dict(Note(60, 60)), note_to_dict(Note(63, 45)), note_to_dict(Note(65, 40))]
    d["synth_osc_d"] = 0.52
    d["synth_osc_v"] = 0.62
    d["synth_osc_stereo_v_offset"] = 0.18
    d["synth_osc_volume"] = 0.68
    d["synth_filter_cutoff"] = 3500.0
    d["synth_filter_resonance"] = 0.2
    d["synth_filter_env_amount"] = 1500.0
    d["synth_vol_attack"] = 3.0
    d["synth_vol_decay"] = 220.0
    d["synth_vol_sustain"] = 0.45
    d["synth_vol_release"] = 300.0
    d["synth_reverb_mix"] = 0.18
    d["synth_reverb_decay"] = 0.58
    d["note_length_percent"] = 75.0
    d["lfo1_tempo_sync"] = True
    d["lfo1_sync_division"] = 4
    d["lfo1_waveform"] = 1
    d["lfo1_dest1"] = 10
    d["lfo1_amount1"] = 0.2
    d["lfo2_tempo_sync"] = True
    d["lfo2_sync_division"] = 7
    d["lfo2_waveform"] = 0
    d["lfo2_dest1"] = 11
    d["lfo2_amount1"] = 0.15
    presets.append(create_preset("Synesthesia", "Factory",
        "Sense blending - hear colors, see sounds", d))

    # 22. Quasar Pulse - Distant beacons
    d = create_default_preset()
    d["straight_1_4"] = [100.0, 0.0, 95.0, 0.0]
    d["straight_1_8"] = [0.0, 75.0, 0.0, 70.0, 0.0, 80.0, 0.0, 65.0]
    d["strength_values"] = create_strength_pattern("syncopated")
    d["root_note"] = 36
    d["notes"] = [note_to_dict(Note(36, 100)), note_to_dict(Note(43, 55)), note_to_dict(Note(48, 50)),
                  note_to_dict(Note(55, 45))]
    d["synth_osc_d"] = 0.48
    d["synth_osc_v"] = 0.45
    d["synth_osc_volume"] = 0.72
    d["synth_sub_volume"] = 0.3
    d["synth_filter_cutoff"] = 1800.0
    d["synth_filter_resonance"] = 0.32
    d["synth_filter_env_amount"] = 1800.0
    d["synth_vol_attack"] = 1.0
    d["synth_vol_decay"] = 180.0
    d["synth_vol_sustain"] = 0.35
    d["synth_vol_release"] = 250.0
    d["synth_reverb_mix"] = 0.2
    d["synth_reverb_decay"] = 0.65
    d["note_length_percent"] = 60.0
    presets.append(create_preset("Quasar Pulse", "Factory",
        "Cosmic lighthouse - billion light year pulses", d))

    # 23. Machine Elf - McKenna entities
    d = create_default_preset()
    d["straight_1_16"] = [70.0, 50.0, 60.0, 40.0, 75.0, 55.0, 65.0, 45.0, 72.0, 52.0, 62.0, 42.0, 78.0, 58.0, 68.0, 48.0]
    d["triplet_1_16t"] = [50.0, 35.0, 42.0, 55.0, 38.0, 45.0, 52.0, 32.0, 40.0, 58.0, 40.0, 48.0,
                          48.0, 33.0, 40.0, 53.0, 36.0, 43.0, 50.0, 30.0, 38.0, 56.0, 38.0, 46.0]
    d["strength_values"] = create_strength_pattern("dense")
    d["root_note"] = 55
    d["notes"] = [note_to_dict(Note(55, 100)), note_to_dict(Note(58, 55)), note_to_dict(Note(60, 50)),
                  note_to_dict(Note(63, 45)), note_to_dict(Note(67, 40)), note_to_dict(Note(70, 35))]
    d["octave_randomization"] = create_octave_randomization(0.35, 0.5, 0.35, "Up")
    d["synth_osc_d"] = 0.58
    d["synth_osc_v"] = 0.55
    d["synth_osc_stereo_v_offset"] = 0.2
    d["synth_osc_volume"] = 0.65
    d["synth_filter_cutoff"] = 4000.0
    d["synth_filter_resonance"] = 0.25
    d["synth_filter_env_amount"] = 2000.0
    d["synth_vol_attack"] = 1.0
    d["synth_vol_decay"] = 100.0
    d["synth_vol_sustain"] = 0.3
    d["synth_vol_release"] = 150.0
    d["synth_reverb_mix"] = 0.18
    d["synth_reverb_decay"] = 0.55
    d["note_length_percent"] = 45.0
    presets.append(create_preset("Machine Elf", "Factory",
        "Self-transforming - jeweled entities present gifts of language", d))

    # 24. Stellar Nursery - Star birth
    d = create_default_preset()
    d["straight_1_2"] = [100.0, 85.0]
    d["straight_1_4"] = [65.0, 50.0, 60.0, 45.0]
    d["straight_1_8"] = [40.0, 30.0, 35.0, 25.0, 45.0, 32.0, 38.0, 28.0]
    d["strength_values"] = create_strength_pattern("ambient")
    d["root_note"] = 48
    d["notes"] = [note_to_dict(Note(48, 100)), note_to_dict(Note(52, 58)), note_to_dict(Note(55, 55)),
                  note_to_dict(Note(60, 50)), note_to_dict(Note(64, 45)), note_to_dict(Note(67, 40))]
    d["synth_osc_d"] = 0.22
    d["synth_osc_v"] = 0.72
    d["synth_osc_stereo_v_offset"] = 0.15
    d["synth_osc_volume"] = 0.58
    d["synth_filter_cutoff"] = 3200.0
    d["synth_filter_resonance"] = 0.1
    d["synth_filter_env_amount"] = 500.0
    d["synth_vol_attack"] = 180.0
    d["synth_vol_decay"] = 1200.0
    d["synth_vol_sustain"] = 0.6
    d["synth_vol_release"] = 1800.0
    d["synth_reverb_mix"] = 0.22
    d["synth_reverb_decay"] = 0.72
    d["synth_reverb_diffusion"] = 0.88
    d["note_length_percent"] = 180.0
    presets.append(create_preset("Stellar Nursery", "Factory",
        "Cosmic womb - gas clouds collapse into new suns", d))

    # 25. Peak Experience - Summit of consciousness
    d = create_default_preset()
    d["straight_1_4"] = [100.0, 90.0, 95.0, 85.0]
    d["straight_1_8"] = [70.0, 60.0, 65.0, 55.0, 75.0, 62.0, 68.0, 58.0]
    d["triplet_1_8t"] = [55.0, 45.0, 50.0, 58.0, 48.0, 52.0, 52.0, 42.0, 48.0, 60.0, 50.0, 55.0]
    d["strength_values"] = create_strength_pattern("4_4_standard")
    d["root_note"] = 55
    d["notes"] = [note_to_dict(Note(55, 100)), note_to_dict(Note(58, 55)), note_to_dict(Note(60, 50)),
                  note_to_dict(Note(62, 60)), note_to_dict(Note(65, 45)), note_to_dict(Note(67, 40))]
    d["synth_osc_d"] = 0.4
    d["synth_osc_v"] = 0.65
    d["synth_osc_stereo_v_offset"] = 0.12
    d["synth_osc_volume"] = 0.68
    d["synth_filter_cutoff"] = 3500.0
    d["synth_filter_resonance"] = 0.18
    d["synth_filter_env_amount"] = 1400.0
    d["synth_vol_attack"] = 5.0
    d["synth_vol_decay"] = 280.0
    d["synth_vol_sustain"] = 0.5
    d["synth_vol_release"] = 400.0
    d["synth_reverb_mix"] = 0.2
    d["synth_reverb_decay"] = 0.65
    d["note_length_percent"] = 85.0
    presets.append(create_preset("Peak Experience", "Factory",
        "Transcendent moment - unity with the infinite", d))

    # 26. Spore Cloud - Fungal dispersal
    d = create_default_preset()
    d["straight_1_8"] = [55.0, 40.0, 50.0, 35.0, 60.0, 45.0, 48.0, 32.0]
    d["straight_1_16"] = [35.0, 22.0, 30.0, 18.0, 40.0, 28.0, 32.0, 20.0, 38.0, 25.0, 33.0, 22.0, 42.0, 30.0, 35.0, 24.0]
    d["strength_values"] = create_strength_pattern("ambient")
    d["root_note"] = 50
    d["notes"] = [note_to_dict(Note(50, 100)), note_to_dict(Note(53, 55)), note_to_dict(Note(57, 50)),
                  note_to_dict(Note(60, 45)), note_to_dict(Note(62, 40)), note_to_dict(Note(65, 35))]
    d["octave_randomization"] = create_octave_randomization(0.4, 0.5, 0.4, "Both")
    d["synth_osc_d"] = 0.35
    d["synth_osc_v"] = 0.6
    d["synth_osc_stereo_v_offset"] = 0.18
    d["synth_osc_volume"] = 0.58
    d["synth_filter_cutoff"] = 3800.0
    d["synth_filter_resonance"] = 0.12
    d["synth_filter_env_amount"] = 1200.0
    d["synth_vol_attack"] = 10.0
    d["synth_vol_decay"] = 200.0
    d["synth_vol_sustain"] = 0.35
    d["synth_vol_release"] = 350.0
    d["synth_reverb_mix"] = 0.2
    d["synth_reverb_decay"] = 0.6
    d["note_length_percent"] = 65.0
    presets.append(create_preset("Spore Cloud", "Factory",
        "Mycological mist - billions of spores drift seeking ground", d))

    # 27. Singularity - Point of infinite density
    d = create_default_preset()
    d["straight_1_4"] = [100.0, 85.0, 90.0, 80.0]
    d["straight_1_8"] = [70.0, 55.0, 65.0, 50.0, 75.0, 60.0, 68.0, 52.0]
    d["straight_1_16"] = [50.0, 40.0, 45.0, 35.0, 55.0, 42.0, 48.0, 38.0, 52.0, 43.0, 46.0, 36.0, 58.0, 45.0, 50.0, 40.0]
    d["strength_values"] = create_strength_pattern("bass_heavy")
    d["root_note"] = 33
    d["notes"] = [note_to_dict(Note(33, 100)), note_to_dict(Note(40, 55)), note_to_dict(Note(45, 50)),
                  note_to_dict(Note(48, 45))]
    d["synth_osc_d"] = 0.45
    d["synth_osc_v"] = 0.35
    d["synth_osc_volume"] = 0.72
    d["synth_sub_volume"] = 0.4
    d["synth_filter_cutoff"] = 1000.0
    d["synth_filter_resonance"] = 0.45
    d["synth_filter_env_amount"] = 800.0
    d["synth_vol_attack"] = 2.0
    d["synth_vol_decay"] = 300.0
    d["synth_vol_sustain"] = 0.5
    d["synth_vol_release"] = 450.0
    d["note_length_percent"] = 90.0
    d["lfo1_rate"] = 0.05
    d["lfo1_waveform"] = 0
    d["lfo1_dest1"] = 13
    d["lfo1_amount1"] = 0.2
    presets.append(create_preset("Singularity", "Factory",
        "Infinite collapse - all matter drawn to center point", d))

    # 28. Gamma Burst - Cosmic explosion
    d = create_default_preset()
    d["straight_1_8"] = [100.0, 85.0, 90.0, 80.0, 95.0, 82.0, 88.0, 75.0]
    d["straight_1_16"] = [75.0, 60.0, 70.0, 55.0, 80.0, 65.0, 72.0, 58.0, 78.0, 62.0, 68.0, 56.0, 82.0, 68.0, 75.0, 60.0]
    d["straight_1_32"] = [55.0, 40.0, 50.0, 35.0, 60.0, 45.0, 52.0, 38.0, 58.0, 42.0, 48.0, 36.0, 62.0, 48.0, 55.0, 40.0,
                          56.0, 41.0, 51.0, 36.0, 61.0, 46.0, 53.0, 39.0, 59.0, 43.0, 49.0, 37.0, 63.0, 49.0, 56.0, 41.0]
    d["strength_values"] = create_strength_pattern("driving")
    d["root_note"] = 43
    d["notes"] = [note_to_dict(Note(43, 100)), note_to_dict(Note(48, 55)), note_to_dict(Note(50, 50)),
                  note_to_dict(Note(55, 45)), note_to_dict(Note(60, 40))]
    d["synth_osc_d"] = 0.62
    d["synth_osc_v"] = 0.48
    d["synth_osc_volume"] = 0.72
    d["synth_filter_cutoff"] = 2500.0
    d["synth_filter_resonance"] = 0.28
    d["synth_filter_env_amount"] = 2000.0
    d["synth_vol_attack"] = 0.5
    d["synth_vol_decay"] = 150.0
    d["synth_vol_sustain"] = 0.4
    d["synth_vol_release"] = 200.0
    d["note_length_percent"] = 55.0
    presets.append(create_preset("Gamma Burst", "Factory",
        "Cosmic violence - most energetic events in universe", d))

    # 29. Spirit Molecule - 5-MeO essence
    d = create_default_preset()
    d["straight_1_4"] = [100.0, 75.0, 85.0, 65.0]
    d["straight_1_8"] = [55.0, 40.0, 50.0, 35.0, 60.0, 45.0, 52.0, 38.0]
    d["triplet_1_8t"] = [45.0, 32.0, 38.0, 50.0, 35.0, 42.0, 48.0, 30.0, 40.0, 52.0, 38.0, 45.0]
    d["strength_values"] = create_strength_pattern("ambient")
    d["root_note"] = 48
    d["notes"] = [note_to_dict(Note(48, 100)), note_to_dict(Note(52, 55)), note_to_dict(Note(55, 50)),
                  note_to_dict(Note(60, 45)), note_to_dict(Note(64, 40)), note_to_dict(Note(67, 35))]
    d["synth_osc_d"] = 0.25
    d["synth_osc_v"] = 0.7
    d["synth_osc_stereo_v_offset"] = 0.15
    d["synth_osc_volume"] = 0.58
    d["synth_filter_cutoff"] = 3000.0
    d["synth_filter_resonance"] = 0.12
    d["synth_filter_env_amount"] = 600.0
    d["synth_vol_attack"] = 120.0
    d["synth_vol_decay"] = 800.0
    d["synth_vol_sustain"] = 0.55
    d["synth_vol_release"] = 1200.0
    d["synth_reverb_mix"] = 0.22
    d["synth_reverb_decay"] = 0.7
    d["note_length_percent"] = 150.0
    presets.append(create_preset("Spirit Molecule", "Factory",
        "Inner light - the key that unlocks perception", d))

    # 30. Multiverse Gate - Parallel realities
    d = create_default_preset()
    euc_multi = euclidean_rhythm(16, 11)
    for i in range(16):
        d["straight_1_16"][i] = 80.0 if i in euc_multi else 25.0
    d["triplet_1_8t"] = [65.0, 45.0, 55.0, 70.0, 48.0, 58.0, 62.0, 42.0, 52.0, 68.0, 50.0, 60.0]
    d["strength_values"] = create_strength_pattern("polyrhythm_3_4")
    d["root_note"] = 45
    d["notes"] = [note_to_dict(Note(45, 100)), note_to_dict(Note(48, 55)), note_to_dict(Note(52, 50)),
                  note_to_dict(Note(55, 45)), note_to_dict(Note(57, 40)), note_to_dict(Note(60, 35))]
    d["synth_osc_d"] = 0.48
    d["synth_osc_v"] = 0.55
    d["synth_osc_stereo_v_offset"] = 0.15
    d["synth_osc_volume"] = 0.68
    d["synth_filter_cutoff"] = 3200.0
    d["synth_filter_resonance"] = 0.22
    d["synth_filter_env_amount"] = 1500.0
    d["synth_vol_attack"] = 3.0
    d["synth_vol_decay"] = 200.0
    d["synth_vol_sustain"] = 0.45
    d["synth_vol_release"] = 280.0
    d["synth_reverb_mix"] = 0.2
    d["synth_reverb_decay"] = 0.6
    d["note_length_percent"] = 70.0
    presets.append(create_preset("Multiverse Gate", "Factory",
        "Portal opened - glimpsing infinite branching realities", d))

    # 31. Cosmic Serpent - DNA wisdom
    d = create_default_preset()
    d["straight_1_8"] = [90.0, 65.0, 75.0, 55.0, 85.0, 60.0, 80.0, 50.0]
    d["straight_1_16"] = [60.0, 45.0, 55.0, 40.0, 65.0, 48.0, 58.0, 42.0, 62.0, 47.0, 56.0, 43.0, 68.0, 50.0, 60.0, 45.0]
    d["triplet_1_8t"] = [50.0, 38.0, 45.0, 55.0, 40.0, 48.0, 52.0, 35.0, 42.0, 58.0, 42.0, 50.0]
    d["strength_values"] = create_strength_pattern("triplet_feel")
    d["root_note"] = 41
    d["notes"] = [note_to_dict(Note(41, 100)), note_to_dict(Note(44, 55)), note_to_dict(Note(48, 50)),
                  note_to_dict(Note(53, 45)), note_to_dict(Note(56, 40)), note_to_dict(Note(60, 35))]
    d["synth_osc_d"] = 0.4
    d["synth_osc_v"] = 0.58
    d["synth_osc_stereo_v_offset"] = 0.12
    d["synth_osc_volume"] = 0.68
    d["synth_filter_cutoff"] = 2600.0
    d["synth_filter_resonance"] = 0.2
    d["synth_filter_env_amount"] = 1200.0
    d["synth_vol_attack"] = 5.0
    d["synth_vol_decay"] = 250.0
    d["synth_vol_sustain"] = 0.48
    d["synth_vol_release"] = 350.0
    d["synth_reverb_mix"] = 0.18
    d["synth_reverb_decay"] = 0.58
    d["note_length_percent"] = 80.0
    d["lfo1_tempo_sync"] = True
    d["lfo1_sync_division"] = 5
    d["lfo1_waveform"] = 0
    d["lfo1_dest1"] = 11
    d["lfo1_amount1"] = 0.15
    presets.append(create_preset("Cosmic Serpent", "Factory",
        "Coiled helix - ancient knowledge encoded in every cell", d))

    # 32. Omega Point - Final convergence
    d = create_default_preset()
    d["straight_1_2"] = [100.0, 90.0]
    d["straight_1_4"] = [75.0, 65.0, 70.0, 60.0]
    d["straight_1_8"] = [50.0, 40.0, 45.0, 35.0, 55.0, 42.0, 48.0, 38.0]
    d["strength_values"] = create_strength_pattern("4_4_standard")
    d["root_note"] = 48
    d["notes"] = [note_to_dict(Note(48, 100)), note_to_dict(Note(52, 58)), note_to_dict(Note(55, 55)),
                  note_to_dict(Note(60, 50)), note_to_dict(Note(64, 45)), note_to_dict(Note(67, 40))]
    d["synth_osc_d"] = 0.32
    d["synth_osc_v"] = 0.68
    d["synth_osc_stereo_v_offset"] = 0.15
    d["synth_osc_volume"] = 0.62
    d["synth_filter_cutoff"] = 3200.0
    d["synth_filter_resonance"] = 0.15
    d["synth_filter_env_amount"] = 800.0
    d["synth_vol_attack"] = 80.0
    d["synth_vol_decay"] = 600.0
    d["synth_vol_sustain"] = 0.55
    d["synth_vol_release"] = 1000.0
    d["synth_reverb_mix"] = 0.22
    d["synth_reverb_decay"] = 0.7
    d["synth_reverb_diffusion"] = 0.88
    d["note_length_percent"] = 150.0
    presets.append(create_preset("Omega Point", "Factory",
        "Final destination - all evolution converges here", d))

    return presets

def create_bank_f() -> List[Dict]:
    """Bank F: Trance & Progressive - 32 presets"""
    presets = []

    # 1. Classic Trance Lead - Supersaw inspired
    d = create_default_preset()
    d["straight_1_4"] = [100.0, 90.0, 95.0, 85.0]
    d["straight_1_8"] = [75.0, 65.0, 70.0, 60.0, 80.0, 68.0, 72.0, 62.0]
    d["straight_1_16"] = [50.0, 40.0, 45.0, 35.0, 55.0, 42.0, 48.0, 38.0, 52.0, 44.0, 46.0, 36.0, 58.0, 45.0, 50.0, 40.0]
    d["strength_values"] = create_strength_pattern("driving")
    d["root_note"] = 48
    d["notes"] = [note_to_dict(Note(48, 100)), note_to_dict(Note(52, 60)), note_to_dict(Note(55, 55)),
                  note_to_dict(Note(60, 50)), note_to_dict(Note(64, 45))]
    d["synth_osc_d"] = 0.45
    d["synth_osc_v"] = 0.6
    d["synth_osc_stereo_v_offset"] = 0.15
    d["synth_osc_volume"] = 0.72
    d["synth_filter_cutoff"] = 3500.0
    d["synth_filter_resonance"] = 0.15
    d["synth_filter_env_amount"] = 1500.0
    d["synth_vol_attack"] = 5.0
    d["synth_vol_decay"] = 300.0
    d["synth_vol_sustain"] = 0.55
    d["synth_vol_release"] = 400.0
    d["synth_reverb_mix"] = 0.15
    d["synth_reverb_decay"] = 0.5
    d["note_length_percent"] = 85.0
    presets.append(create_preset("Classic Trance Lead", "Factory",
        "Melodic euphoria - soaring leads that lift the spirit", d))

    # 2. Uplifting Arp - Rising arpeggios
    d = create_default_preset()
    d["straight_1_16"] = [100.0, 85.0, 90.0, 80.0, 95.0, 82.0, 88.0, 75.0, 92.0, 78.0, 86.0, 72.0, 98.0, 80.0, 84.0, 70.0]
    d["strength_values"] = create_strength_pattern("driving")
    d["root_note"] = 48
    d["notes"] = [note_to_dict(Note(48, 100)), note_to_dict(Note(52, 55)), note_to_dict(Note(55, 50)),
                  note_to_dict(Note(60, 45)), note_to_dict(Note(64, 40)), note_to_dict(Note(67, 35))]
    d["synth_osc_d"] = 0.5
    d["synth_osc_v"] = 0.55
    d["synth_osc_stereo_v_offset"] = 0.12
    d["synth_osc_volume"] = 0.7
    d["synth_filter_cutoff"] = 4000.0
    d["synth_filter_resonance"] = 0.18
    d["synth_filter_env_amount"] = 1800.0
    d["synth_vol_attack"] = 1.0
    d["synth_vol_decay"] = 150.0
    d["synth_vol_sustain"] = 0.4
    d["synth_vol_release"] = 200.0
    d["synth_reverb_mix"] = 0.18
    d["synth_reverb_decay"] = 0.55
    d["note_length_percent"] = 55.0
    presets.append(create_preset("Uplifting Arp", "Factory",
        "Rising cascade - arpeggios climb toward the drop", d))

    # 3. Goa Psy - Trippy acid
    d = create_default_preset()
    d["straight_1_16"] = [100.0, 75.0, 85.0, 70.0, 95.0, 78.0, 80.0, 65.0, 90.0, 72.0, 82.0, 68.0, 88.0, 76.0, 78.0, 62.0]
    d["triplet_1_16t"] = [60.0, 45.0, 52.0, 65.0, 48.0, 55.0, 58.0, 42.0, 50.0, 62.0, 46.0, 54.0,
                          55.0, 40.0, 48.0, 60.0, 44.0, 52.0, 56.0, 38.0, 46.0, 58.0, 42.0, 50.0]
    d["strength_values"] = create_strength_pattern("driving")
    d["root_note"] = 36
    d["notes"] = [note_to_dict(Note(36, 100)), note_to_dict(Note(39, 60)), note_to_dict(Note(43, 55)),
                  note_to_dict(Note(48, 50)), note_to_dict(Note(51, 45))]
    d["octave_randomization"] = create_octave_randomization(0.2, 0.5, 0.3, "Up")
    d["synth_osc_d"] = 0.65
    d["synth_osc_v"] = 0.35
    d["synth_osc_volume"] = 0.75
    d["synth_filter_cutoff"] = 900.0
    d["synth_filter_resonance"] = 0.65
    d["synth_filter_env_amount"] = 3500.0
    d["synth_vol_attack"] = 0.5
    d["synth_vol_decay"] = 120.0
    d["synth_vol_sustain"] = 0.3
    d["synth_vol_release"] = 150.0
    d["note_length_percent"] = 50.0
    d["lfo1_tempo_sync"] = True
    d["lfo1_sync_division"] = 2
    d["lfo1_waveform"] = 2
    d["lfo1_dest1"] = 12
    d["lfo1_amount1"] = 0.3
    presets.append(create_preset("Goa Psy", "Factory",
        "Morning sunshine - acid lines twist through consciousness", d))

    # 4. Progressive Pluck - Deep house plucks
    d = create_default_preset()
    d["straight_1_8"] = [100.0, 80.0, 85.0, 70.0, 95.0, 75.0, 88.0, 65.0]
    d["straight_1_16"] = [55.0, 40.0, 48.0, 35.0, 60.0, 45.0, 52.0, 38.0, 58.0, 42.0, 50.0, 36.0, 62.0, 48.0, 54.0, 40.0]
    d["strength_values"] = create_strength_pattern("syncopated")
    d["root_note"] = 41
    d["notes"] = [note_to_dict(Note(41, 100)), note_to_dict(Note(45, 55)), note_to_dict(Note(48, 50)),
                  note_to_dict(Note(53, 45)), note_to_dict(Note(57, 40))]
    d["synth_osc_d"] = 0.55
    d["synth_osc_v"] = 0.5
    d["synth_osc_volume"] = 0.7
    d["synth_filter_cutoff"] = 4500.0
    d["synth_filter_resonance"] = 0.22
    d["synth_filter_env_amount"] = 2500.0
    d["synth_vol_attack"] = 0.5
    d["synth_vol_decay"] = 100.0
    d["synth_vol_sustain"] = 0.2
    d["synth_vol_release"] = 180.0
    d["synth_reverb_mix"] = 0.2
    d["synth_reverb_decay"] = 0.55
    d["note_length_percent"] = 40.0
    presets.append(create_preset("Progressive Pluck", "Factory",
        "Deep melodic - plucked strings carry the groove", d))

    # 5. Trance Gate - Rhythmic tremolo
    d = create_default_preset()
    d["straight_1_16"] = [100.0, 0.0, 90.0, 0.0, 95.0, 0.0, 85.0, 0.0, 100.0, 0.0, 88.0, 0.0, 92.0, 0.0, 82.0, 0.0]
    d["strength_values"] = create_strength_pattern("driving")
    d["root_note"] = 48
    d["notes"] = [note_to_dict(Note(48, 100)), note_to_dict(Note(52, 55)), note_to_dict(Note(55, 50)),
                  note_to_dict(Note(60, 45))]
    d["synth_osc_d"] = 0.4
    d["synth_osc_v"] = 0.62
    d["synth_osc_stereo_v_offset"] = 0.15
    d["synth_osc_volume"] = 0.7
    d["synth_filter_cutoff"] = 3000.0
    d["synth_filter_resonance"] = 0.12
    d["synth_filter_env_amount"] = 1000.0
    d["synth_vol_attack"] = 1.0
    d["synth_vol_decay"] = 80.0
    d["synth_vol_sustain"] = 0.35
    d["synth_vol_release"] = 120.0
    d["synth_reverb_mix"] = 0.18
    d["synth_reverb_decay"] = 0.5
    d["note_length_percent"] = 45.0
    presets.append(create_preset("Trance Gate", "Factory",
        "Pumping rhythm - gated pads pulse with energy", d))

    # 6. Euro Energy - Hard trance stab
    d = create_default_preset()
    d["straight_1_4"] = [100.0, 95.0, 100.0, 90.0]
    d["straight_1_8"] = [85.0, 75.0, 80.0, 70.0, 88.0, 78.0, 82.0, 72.0]
    d["strength_values"] = create_strength_pattern("driving")
    d["root_note"] = 41
    d["notes"] = [note_to_dict(Note(41, 100)), note_to_dict(Note(45, 55)), note_to_dict(Note(48, 50)),
                  note_to_dict(Note(53, 45))]
    d["synth_osc_d"] = 0.6
    d["synth_osc_v"] = 0.4
    d["synth_osc_volume"] = 0.75
    d["synth_sub_volume"] = 0.25
    d["synth_filter_cutoff"] = 2500.0
    d["synth_filter_resonance"] = 0.3
    d["synth_filter_env_amount"] = 2000.0
    d["synth_vol_attack"] = 0.5
    d["synth_vol_decay"] = 150.0
    d["synth_vol_sustain"] = 0.4
    d["synth_vol_release"] = 200.0
    d["note_length_percent"] = 65.0
    presets.append(create_preset("Euro Energy", "Factory",
        "Hard and fast - relentless driving energy", d))

    # 7. Ambient Breakdown - Ethereal pad
    d = create_default_preset()
    d["straight_1_2"] = [100.0, 80.0]
    d["straight_1_4"] = [60.0, 45.0, 55.0, 40.0]
    d["dotted_1_4d"] = [50.0, 35.0, 42.0]
    d["strength_values"] = create_strength_pattern("ambient")
    d["root_note"] = 48
    d["notes"] = [note_to_dict(Note(48, 100)), note_to_dict(Note(52, 60)), note_to_dict(Note(55, 55)),
                  note_to_dict(Note(60, 50)), note_to_dict(Note(64, 45)), note_to_dict(Note(67, 38))]
    d["synth_osc_d"] = 0.2
    d["synth_osc_v"] = 0.75
    d["synth_osc_stereo_v_offset"] = 0.2
    d["synth_osc_volume"] = 0.55
    d["synth_filter_cutoff"] = 3200.0
    d["synth_filter_resonance"] = 0.08
    d["synth_filter_env_amount"] = 400.0
    d["synth_vol_attack"] = 200.0
    d["synth_vol_decay"] = 1500.0
    d["synth_vol_sustain"] = 0.65
    d["synth_vol_release"] = 2500.0
    d["synth_reverb_mix"] = 0.25
    d["synth_reverb_decay"] = 0.8
    d["synth_reverb_diffusion"] = 0.9
    d["note_length_percent"] = 200.0
    presets.append(create_preset("Ambient Breakdown", "Factory",
        "Floating respite - ethereal pads for emotional breaks", d))

    # 8. Tech Trance - Minimal grooves
    d = create_default_preset()
    euc_tech = euclidean_rhythm(16, 7)
    for i in range(16):
        d["straight_1_16"][i] = 90.0 if i in euc_tech else 30.0
    d["strength_values"] = create_strength_pattern("syncopated")
    d["root_note"] = 36
    d["notes"] = [note_to_dict(Note(36, 100)), note_to_dict(Note(41, 55)), note_to_dict(Note(43, 50)),
                  note_to_dict(Note(48, 45))]
    d["synth_osc_d"] = 0.52
    d["synth_osc_v"] = 0.45
    d["synth_osc_volume"] = 0.72
    d["synth_sub_volume"] = 0.3
    d["synth_filter_cutoff"] = 2200.0
    d["synth_filter_resonance"] = 0.35
    d["synth_filter_env_amount"] = 1800.0
    d["synth_vol_attack"] = 1.0
    d["synth_vol_decay"] = 120.0
    d["synth_vol_sustain"] = 0.35
    d["synth_vol_release"] = 160.0
    d["note_length_percent"] = 50.0
    presets.append(create_preset("Tech Trance", "Factory",
        "Hypnotic minimal - stripped-back grooves lock you in", d))

    # 9. Anjuna Vibes - Melodic progressive
    d = create_default_preset()
    d["straight_1_8"] = [100.0, 75.0, 85.0, 65.0, 95.0, 70.0, 80.0, 60.0]
    d["straight_1_16"] = [50.0, 35.0, 45.0, 30.0, 55.0, 40.0, 48.0, 32.0, 52.0, 38.0, 46.0, 34.0, 58.0, 42.0, 50.0, 36.0]
    d["strength_values"] = create_strength_pattern("4_4_standard")
    d["root_note"] = 48
    d["notes"] = [note_to_dict(Note(48, 100)), note_to_dict(Note(52, 58)), note_to_dict(Note(55, 55)),
                  note_to_dict(Note(60, 50)), note_to_dict(Note(64, 45)), note_to_dict(Note(67, 38))]
    d["synth_osc_d"] = 0.38
    d["synth_osc_v"] = 0.65
    d["synth_osc_stereo_v_offset"] = 0.12
    d["synth_osc_volume"] = 0.68
    d["synth_filter_cutoff"] = 3800.0
    d["synth_filter_resonance"] = 0.15
    d["synth_filter_env_amount"] = 1200.0
    d["synth_vol_attack"] = 5.0
    d["synth_vol_decay"] = 250.0
    d["synth_vol_sustain"] = 0.5
    d["synth_vol_release"] = 350.0
    d["synth_reverb_mix"] = 0.18
    d["synth_reverb_decay"] = 0.58
    d["note_length_percent"] = 80.0
    presets.append(create_preset("Anjuna Vibes", "Factory",
        "Warm melodic - sunlit progressive house journey", d))

    # 10. Full On - Peak time psy
    d = create_default_preset()
    d["straight_1_16"] = [100.0, 80.0, 90.0, 75.0, 95.0, 82.0, 88.0, 70.0, 98.0, 78.0, 85.0, 72.0, 92.0, 85.0, 86.0, 68.0]
    d["strength_values"] = create_strength_pattern("driving")
    d["root_note"] = 36
    d["notes"] = [note_to_dict(Note(36, 100)), note_to_dict(Note(39, 55)), note_to_dict(Note(41, 50)),
                  note_to_dict(Note(43, 60)), note_to_dict(Note(48, 45))]
    d["synth_osc_d"] = 0.62
    d["synth_osc_v"] = 0.38
    d["synth_osc_volume"] = 0.75
    d["synth_sub_volume"] = 0.25
    d["synth_filter_cutoff"] = 1200.0
    d["synth_filter_resonance"] = 0.55
    d["synth_filter_env_amount"] = 3000.0
    d["synth_vol_attack"] = 0.5
    d["synth_vol_decay"] = 100.0
    d["synth_vol_sustain"] = 0.35
    d["synth_vol_release"] = 140.0
    d["note_length_percent"] = 45.0
    d["lfo1_tempo_sync"] = True
    d["lfo1_sync_division"] = 1
    d["lfo1_waveform"] = 2
    d["lfo1_dest1"] = 12
    d["lfo1_amount1"] = 0.4
    presets.append(create_preset("Full On", "Factory",
        "Peak time power - maximum energy for the dance floor", d))

    # 11. Vocal Trance Pad - Ethereal voices
    d = create_default_preset()
    d["straight_1_2"] = [100.0, 85.0]
    d["straight_1_4"] = [65.0, 50.0, 60.0, 45.0]
    d["strength_values"] = create_strength_pattern("4_4_standard")
    d["root_note"] = 55
    d["notes"] = [note_to_dict(Note(55, 100)), note_to_dict(Note(60, 60)), note_to_dict(Note(64, 55)),
                  note_to_dict(Note(67, 50)), note_to_dict(Note(72, 45))]
    d["synth_osc_d"] = 0.28
    d["synth_osc_v"] = 0.72
    d["synth_osc_stereo_v_offset"] = 0.18
    d["synth_osc_volume"] = 0.6
    d["synth_filter_cutoff"] = 3500.0
    d["synth_filter_resonance"] = 0.1
    d["synth_filter_env_amount"] = 500.0
    d["synth_vol_attack"] = 120.0
    d["synth_vol_decay"] = 800.0
    d["synth_vol_sustain"] = 0.6
    d["synth_vol_release"] = 1200.0
    d["synth_reverb_mix"] = 0.22
    d["synth_reverb_decay"] = 0.68
    d["note_length_percent"] = 150.0
    presets.append(create_preset("Vocal Trance Pad", "Factory",
        "Angelic presence - choir-like textures support melodies", d))

    # 12. Acid House - Classic squelch
    d = create_default_preset()
    d["straight_1_8"] = [100.0, 85.0, 90.0, 80.0, 95.0, 82.0, 88.0, 75.0]
    d["straight_1_16"] = [70.0, 55.0, 65.0, 50.0, 75.0, 58.0, 68.0, 52.0, 72.0, 56.0, 66.0, 48.0, 78.0, 60.0, 70.0, 55.0]
    d["strength_values"] = create_strength_pattern("driving")
    d["root_note"] = 36
    d["notes"] = [note_to_dict(Note(36, 100)), note_to_dict(Note(39, 55)), note_to_dict(Note(41, 50)),
                  note_to_dict(Note(43, 45)), note_to_dict(Note(48, 40))]
    d["synth_osc_d"] = 0.68
    d["synth_osc_v"] = 0.32
    d["synth_osc_volume"] = 0.75
    d["synth_filter_cutoff"] = 600.0
    d["synth_filter_resonance"] = 0.72
    d["synth_filter_env_amount"] = 4500.0
    d["synth_vol_attack"] = 0.5
    d["synth_vol_decay"] = 140.0
    d["synth_vol_sustain"] = 0.3
    d["synth_vol_release"] = 180.0
    d["note_length_percent"] = 55.0
    presets.append(create_preset("Acid House", "Factory",
        "Chicago squelch - warehouse energy from the 303", d))

    # 13. Beatport Progressive - Festival anthem
    d = create_default_preset()
    d["straight_1_4"] = [100.0, 90.0, 95.0, 85.0]
    d["straight_1_8"] = [70.0, 60.0, 65.0, 55.0, 75.0, 62.0, 68.0, 58.0]
    d["strength_values"] = create_strength_pattern("driving")
    d["root_note"] = 45
    d["notes"] = [note_to_dict(Note(45, 100)), note_to_dict(Note(48, 55)), note_to_dict(Note(52, 50)),
                  note_to_dict(Note(57, 45)), note_to_dict(Note(60, 40))]
    d["synth_osc_d"] = 0.42
    d["synth_osc_v"] = 0.58
    d["synth_osc_stereo_v_offset"] = 0.12
    d["synth_osc_volume"] = 0.7
    d["synth_filter_cutoff"] = 3200.0
    d["synth_filter_resonance"] = 0.18
    d["synth_filter_env_amount"] = 1400.0
    d["synth_vol_attack"] = 3.0
    d["synth_vol_decay"] = 220.0
    d["synth_vol_sustain"] = 0.48
    d["synth_vol_release"] = 300.0
    d["synth_reverb_mix"] = 0.15
    d["synth_reverb_decay"] = 0.52
    d["note_length_percent"] = 75.0
    presets.append(create_preset("Beatport Progressive", "Factory",
        "Big room energy - festival-ready progressive drops", d))

    # 14. Dark Psy - Forest frequencies
    d = create_default_preset()
    d["straight_1_16"] = [100.0, 70.0, 85.0, 65.0, 95.0, 72.0, 80.0, 60.0, 90.0, 68.0, 82.0, 62.0, 88.0, 75.0, 78.0, 58.0]
    d["triplet_1_16t"] = [55.0, 40.0, 48.0, 60.0, 42.0, 50.0, 52.0, 38.0, 45.0, 58.0, 44.0, 52.0,
                          50.0, 36.0, 44.0, 55.0, 38.0, 46.0, 48.0, 34.0, 42.0, 54.0, 40.0, 48.0]
    d["strength_values"] = create_strength_pattern("driving")
    d["root_note"] = 33
    d["notes"] = [note_to_dict(Note(33, 100)), note_to_dict(Note(36, 55)), note_to_dict(Note(39, 50)),
                  note_to_dict(Note(45, 45)), note_to_dict(Note(48, 40))]
    d["synth_osc_d"] = 0.7
    d["synth_osc_v"] = 0.28
    d["synth_osc_volume"] = 0.75
    d["synth_sub_volume"] = 0.3
    d["synth_filter_cutoff"] = 700.0
    d["synth_filter_resonance"] = 0.6
    d["synth_filter_env_amount"] = 3200.0
    d["synth_vol_attack"] = 0.5
    d["synth_vol_decay"] = 110.0
    d["synth_vol_sustain"] = 0.28
    d["synth_vol_release"] = 150.0
    d["note_length_percent"] = 42.0
    d["lfo1_tempo_sync"] = True
    d["lfo1_sync_division"] = 3
    d["lfo1_waveform"] = 4
    d["lfo1_dest1"] = 12
    d["lfo1_amount1"] = 0.35
    presets.append(create_preset("Dark Psy", "Factory",
        "Night forest - twisted frequencies from the shadows", d))

    # 15. 138 BPM Lead - High energy trance
    d = create_default_preset()
    d["straight_1_8"] = [100.0, 90.0, 95.0, 85.0, 100.0, 88.0, 92.0, 82.0]
    d["straight_1_16"] = [75.0, 65.0, 70.0, 60.0, 80.0, 68.0, 72.0, 62.0, 78.0, 66.0, 74.0, 64.0, 82.0, 70.0, 76.0, 66.0]
    d["strength_values"] = create_strength_pattern("driving")
    d["root_note"] = 48
    d["notes"] = [note_to_dict(Note(48, 100)), note_to_dict(Note(52, 55)), note_to_dict(Note(55, 50)),
                  note_to_dict(Note(60, 45)), note_to_dict(Note(64, 40))]
    d["synth_osc_d"] = 0.48
    d["synth_osc_v"] = 0.55
    d["synth_osc_stereo_v_offset"] = 0.1
    d["synth_osc_volume"] = 0.72
    d["synth_filter_cutoff"] = 3800.0
    d["synth_filter_resonance"] = 0.2
    d["synth_filter_env_amount"] = 1600.0
    d["synth_vol_attack"] = 2.0
    d["synth_vol_decay"] = 180.0
    d["synth_vol_sustain"] = 0.45
    d["synth_vol_release"] = 250.0
    d["synth_reverb_mix"] = 0.15
    d["synth_reverb_decay"] = 0.5
    d["note_length_percent"] = 70.0
    presets.append(create_preset("138 BPM Lead", "Factory",
        "High octane - racing melodies at peak tempo", d))

    # 16. Balearic Sunset - Ibiza vibes
    d = create_default_preset()
    d["straight_1_4"] = [100.0, 80.0, 85.0, 70.0]
    d["straight_1_8"] = [55.0, 40.0, 50.0, 35.0, 60.0, 45.0, 52.0, 38.0]
    d["triplet_1_8t"] = [45.0, 32.0, 38.0, 48.0, 35.0, 40.0, 42.0, 30.0, 36.0, 50.0, 38.0, 44.0]
    d["strength_values"] = create_strength_pattern("triplet_feel")
    d["root_note"] = 48
    d["notes"] = [note_to_dict(Note(48, 100)), note_to_dict(Note(52, 58)), note_to_dict(Note(55, 55)),
                  note_to_dict(Note(60, 50)), note_to_dict(Note(64, 45)), note_to_dict(Note(67, 38))]
    d["synth_osc_d"] = 0.32
    d["synth_osc_v"] = 0.68
    d["synth_osc_stereo_v_offset"] = 0.15
    d["synth_osc_volume"] = 0.62
    d["synth_filter_cutoff"] = 3500.0
    d["synth_filter_resonance"] = 0.12
    d["synth_filter_env_amount"] = 700.0
    d["synth_vol_attack"] = 20.0
    d["synth_vol_decay"] = 350.0
    d["synth_vol_sustain"] = 0.55
    d["synth_vol_release"] = 500.0
    d["synth_reverb_mix"] = 0.2
    d["synth_reverb_decay"] = 0.62
    d["note_length_percent"] = 100.0
    presets.append(create_preset("Balearic Sunset", "Factory",
        "Golden hour - warm tones for sunset sessions", d))

    # 17. Hardstyle Lead - Euphoric screeches
    d = create_default_preset()
    d["straight_1_4"] = [100.0, 0.0, 95.0, 0.0]
    d["straight_1_8"] = [0.0, 90.0, 0.0, 85.0, 0.0, 88.0, 0.0, 80.0]
    d["strength_values"] = create_strength_pattern("syncopated")
    d["root_note"] = 48
    d["notes"] = [note_to_dict(Note(48, 100)), note_to_dict(Note(52, 55)), note_to_dict(Note(55, 50)),
                  note_to_dict(Note(60, 45))]
    d["synth_osc_d"] = 0.72
    d["synth_osc_v"] = 0.35
    d["synth_osc_volume"] = 0.78
    d["synth_filter_cutoff"] = 2000.0
    d["synth_filter_resonance"] = 0.4
    d["synth_filter_env_amount"] = 3500.0
    d["synth_vol_attack"] = 0.5
    d["synth_vol_decay"] = 200.0
    d["synth_vol_sustain"] = 0.5
    d["synth_vol_release"] = 280.0
    d["note_length_percent"] = 90.0
    d["lfo1_tempo_sync"] = True
    d["lfo1_sync_division"] = 4
    d["lfo1_waveform"] = 2
    d["lfo1_dest1"] = 10
    d["lfo1_amount1"] = 0.25
    presets.append(create_preset("Hardstyle Lead", "Factory",
        "Euphoric screech - massive leads that dominate", d))

    # 18. Melodic Techno - Afterlife style
    d = create_default_preset()
    d["straight_1_8"] = [100.0, 80.0, 85.0, 70.0, 95.0, 75.0, 88.0, 65.0]
    d["straight_1_16"] = [55.0, 40.0, 50.0, 35.0, 60.0, 45.0, 52.0, 38.0, 58.0, 42.0, 48.0, 36.0, 62.0, 48.0, 55.0, 40.0]
    d["strength_values"] = create_strength_pattern("syncopated")
    d["root_note"] = 41
    d["notes"] = [note_to_dict(Note(41, 100)), note_to_dict(Note(44, 55)), note_to_dict(Note(48, 50)),
                  note_to_dict(Note(53, 45)), note_to_dict(Note(56, 40))]
    d["synth_osc_d"] = 0.35
    d["synth_osc_v"] = 0.6
    d["synth_osc_stereo_v_offset"] = 0.12
    d["synth_osc_volume"] = 0.68
    d["synth_filter_cutoff"] = 3000.0
    d["synth_filter_resonance"] = 0.18
    d["synth_filter_env_amount"] = 1100.0
    d["synth_vol_attack"] = 5.0
    d["synth_vol_decay"] = 250.0
    d["synth_vol_sustain"] = 0.45
    d["synth_vol_release"] = 350.0
    d["synth_reverb_mix"] = 0.18
    d["synth_reverb_decay"] = 0.58
    d["note_length_percent"] = 80.0
    presets.append(create_preset("Melodic Techno", "Factory",
        "Emotional driving - introspective grooves with depth", d))

    # 19. Psytrance Bassline - Twisted low end
    d = create_default_preset()
    d["straight_1_16"] = [100.0, 75.0, 90.0, 70.0, 95.0, 78.0, 85.0, 65.0, 92.0, 72.0, 88.0, 68.0, 98.0, 80.0, 82.0, 62.0]
    d["strength_values"] = create_strength_pattern("driving")
    d["root_note"] = 33
    d["notes"] = [note_to_dict(Note(33, 100)), note_to_dict(Note(36, 55)), note_to_dict(Note(39, 50)),
                  note_to_dict(Note(41, 45))]
    d["synth_osc_d"] = 0.55
    d["synth_osc_v"] = 0.35
    d["synth_osc_volume"] = 0.7
    d["synth_sub_volume"] = 0.4
    d["synth_filter_cutoff"] = 1000.0
    d["synth_filter_resonance"] = 0.5
    d["synth_filter_env_amount"] = 2500.0
    d["synth_vol_attack"] = 0.5
    d["synth_vol_decay"] = 100.0
    d["synth_vol_sustain"] = 0.35
    d["synth_vol_release"] = 140.0
    d["note_length_percent"] = 50.0
    d["lfo1_tempo_sync"] = True
    d["lfo1_sync_division"] = 2
    d["lfo1_waveform"] = 2
    d["lfo1_dest1"] = 12
    d["lfo1_amount1"] = 0.25
    presets.append(create_preset("Psytrance Bassline", "Factory",
        "Rolling foundation - twisted bass carries the beat", d))

    # 20. Dreamstate Pad - ASOT vibes
    d = create_default_preset()
    d["straight_1_2"] = [100.0, 85.0]
    d["straight_1_4"] = [60.0, 48.0, 55.0, 42.0]
    d["strength_values"] = create_strength_pattern("ambient")
    d["root_note"] = 48
    d["notes"] = [note_to_dict(Note(48, 100)), note_to_dict(Note(52, 60)), note_to_dict(Note(55, 55)),
                  note_to_dict(Note(60, 50)), note_to_dict(Note(64, 45)), note_to_dict(Note(67, 40))]
    d["synth_osc_d"] = 0.22
    d["synth_osc_v"] = 0.75
    d["synth_osc_stereo_v_offset"] = 0.2
    d["synth_osc_volume"] = 0.58
    d["synth_filter_cutoff"] = 3200.0
    d["synth_filter_resonance"] = 0.08
    d["synth_filter_env_amount"] = 350.0
    d["synth_vol_attack"] = 180.0
    d["synth_vol_decay"] = 1200.0
    d["synth_vol_sustain"] = 0.62
    d["synth_vol_release"] = 2000.0
    d["synth_reverb_mix"] = 0.25
    d["synth_reverb_decay"] = 0.75
    d["synth_reverb_diffusion"] = 0.9
    d["note_length_percent"] = 200.0
    presets.append(create_preset("Dreamstate Pad", "Factory",
        "Trance heaven - lush pads for emotional moments", d))

    # 21. Driving Prog Bass - Rolling grooves
    d = create_default_preset()
    d["straight_1_8"] = [100.0, 85.0, 90.0, 75.0, 95.0, 80.0, 88.0, 70.0]
    d["straight_1_16"] = [60.0, 48.0, 55.0, 42.0, 65.0, 52.0, 58.0, 45.0, 62.0, 50.0, 56.0, 44.0, 68.0, 54.0, 60.0, 48.0]
    d["strength_values"] = create_strength_pattern("driving")
    d["root_note"] = 36
    d["notes"] = [note_to_dict(Note(36, 100)), note_to_dict(Note(41, 55)), note_to_dict(Note(43, 50)),
                  note_to_dict(Note(48, 45))]
    d["synth_osc_d"] = 0.45
    d["synth_osc_v"] = 0.42
    d["synth_osc_volume"] = 0.72
    d["synth_sub_volume"] = 0.32
    d["synth_filter_cutoff"] = 1800.0
    d["synth_filter_resonance"] = 0.28
    d["synth_filter_env_amount"] = 1400.0
    d["synth_vol_attack"] = 1.0
    d["synth_vol_decay"] = 150.0
    d["synth_vol_sustain"] = 0.42
    d["synth_vol_release"] = 200.0
    d["note_length_percent"] = 60.0
    presets.append(create_preset("Driving Prog Bass", "Factory",
        "Relentless push - bass drives the journey forward", d))

    # 22. Trancey Sequence - Hypnotic loops
    d = create_default_preset()
    d["straight_1_16"] = [100.0, 80.0, 90.0, 75.0, 95.0, 82.0, 85.0, 70.0, 92.0, 78.0, 88.0, 72.0, 98.0, 84.0, 86.0, 68.0]
    d["strength_values"] = create_strength_pattern("driving")
    d["root_note"] = 48
    d["notes"] = [note_to_dict(Note(48, 100)), note_to_dict(Note(50, 55)), note_to_dict(Note(52, 50)),
                  note_to_dict(Note(55, 60)), note_to_dict(Note(57, 45))]
    d["synth_osc_d"] = 0.5
    d["synth_osc_v"] = 0.52
    d["synth_osc_volume"] = 0.7
    d["synth_filter_cutoff"] = 3500.0
    d["synth_filter_resonance"] = 0.2
    d["synth_filter_env_amount"] = 1600.0
    d["synth_vol_attack"] = 1.0
    d["synth_vol_decay"] = 140.0
    d["synth_vol_sustain"] = 0.38
    d["synth_vol_release"] = 180.0
    d["synth_reverb_mix"] = 0.15
    d["synth_reverb_decay"] = 0.48
    d["note_length_percent"] = 55.0
    presets.append(create_preset("Trancey Sequence", "Factory",
        "Looping mantra - hypnotic sequences entrance the mind", d))

    # 23. Festival Anthem - Main stage power
    d = create_default_preset()
    d["straight_1_4"] = [100.0, 95.0, 100.0, 90.0]
    d["straight_1_8"] = [80.0, 70.0, 75.0, 65.0, 85.0, 72.0, 78.0, 68.0]
    d["strength_values"] = create_strength_pattern("driving")
    d["root_note"] = 45
    d["notes"] = [note_to_dict(Note(45, 100)), note_to_dict(Note(48, 55)), note_to_dict(Note(52, 50)),
                  note_to_dict(Note(57, 45)), note_to_dict(Note(60, 40))]
    d["synth_osc_d"] = 0.5
    d["synth_osc_v"] = 0.55
    d["synth_osc_stereo_v_offset"] = 0.12
    d["synth_osc_volume"] = 0.75
    d["synth_filter_cutoff"] = 3500.0
    d["synth_filter_resonance"] = 0.18
    d["synth_filter_env_amount"] = 1500.0
    d["synth_vol_attack"] = 2.0
    d["synth_vol_decay"] = 200.0
    d["synth_vol_sustain"] = 0.5
    d["synth_vol_release"] = 280.0
    d["synth_reverb_mix"] = 0.15
    d["synth_reverb_decay"] = 0.5
    d["note_length_percent"] = 75.0
    presets.append(create_preset("Festival Anthem", "Factory",
        "Arms in air - massive drops for thousand-strong crowds", d))

    # 24. Zenonesque - Organic psy
    d = create_default_preset()
    d["straight_1_8"] = [95.0, 70.0, 80.0, 60.0, 90.0, 65.0, 85.0, 55.0]
    d["straight_1_16"] = [55.0, 40.0, 50.0, 35.0, 60.0, 45.0, 52.0, 38.0, 58.0, 42.0, 48.0, 36.0, 62.0, 48.0, 55.0, 40.0]
    d["triplet_1_8t"] = [48.0, 35.0, 42.0, 52.0, 38.0, 45.0, 50.0, 32.0, 40.0, 55.0, 40.0, 48.0]
    d["strength_values"] = create_strength_pattern("triplet_feel")
    d["root_note"] = 41
    d["notes"] = [note_to_dict(Note(41, 100)), note_to_dict(Note(44, 55)), note_to_dict(Note(48, 50)),
                  note_to_dict(Note(53, 45)), note_to_dict(Note(56, 40))]
    d["synth_osc_d"] = 0.4
    d["synth_osc_v"] = 0.55
    d["synth_osc_stereo_v_offset"] = 0.12
    d["synth_osc_volume"] = 0.68
    d["synth_filter_cutoff"] = 2400.0
    d["synth_filter_resonance"] = 0.25
    d["synth_filter_env_amount"] = 1500.0
    d["synth_vol_attack"] = 3.0
    d["synth_vol_decay"] = 200.0
    d["synth_vol_sustain"] = 0.4
    d["synth_vol_release"] = 280.0
    d["synth_reverb_mix"] = 0.18
    d["synth_reverb_decay"] = 0.55
    d["note_length_percent"] = 70.0
    d["lfo1_rate"] = 0.2
    d["lfo1_waveform"] = 4
    d["lfo1_dest1"] = 11
    d["lfo1_amount1"] = 0.15
    presets.append(create_preset("Zenonesque", "Factory",
        "Organic intelligence - natural grooves with depth", d))

    # 25. Neelix Style - Melodic psy
    d = create_default_preset()
    d["straight_1_16"] = [100.0, 78.0, 88.0, 72.0, 95.0, 82.0, 85.0, 68.0, 92.0, 76.0, 86.0, 70.0, 98.0, 80.0, 84.0, 65.0]
    d["strength_values"] = create_strength_pattern("driving")
    d["root_note"] = 41
    d["notes"] = [note_to_dict(Note(41, 100)), note_to_dict(Note(44, 55)), note_to_dict(Note(48, 50)),
                  note_to_dict(Note(53, 45)), note_to_dict(Note(56, 40)), note_to_dict(Note(60, 35))]
    d["synth_osc_d"] = 0.52
    d["synth_osc_v"] = 0.5
    d["synth_osc_stereo_v_offset"] = 0.1
    d["synth_osc_volume"] = 0.72
    d["synth_filter_cutoff"] = 2800.0
    d["synth_filter_resonance"] = 0.28
    d["synth_filter_env_amount"] = 2000.0
    d["synth_vol_attack"] = 1.0
    d["synth_vol_decay"] = 140.0
    d["synth_vol_sustain"] = 0.38
    d["synth_vol_release"] = 180.0
    d["synth_reverb_mix"] = 0.15
    d["synth_reverb_decay"] = 0.5
    d["note_length_percent"] = 55.0
    d["lfo1_tempo_sync"] = True
    d["lfo1_sync_division"] = 2
    d["lfo1_waveform"] = 2
    d["lfo1_dest1"] = 12
    d["lfo1_amount1"] = 0.22
    presets.append(create_preset("Neelix Style", "Factory",
        "Progressive psy - melodic energy that moves feet", d))

    # 26. Tiesto Classic - Old school anthem
    d = create_default_preset()
    d["straight_1_4"] = [100.0, 90.0, 95.0, 85.0]
    d["straight_1_8"] = [70.0, 60.0, 65.0, 55.0, 75.0, 62.0, 68.0, 58.0]
    d["strength_values"] = create_strength_pattern("driving")
    d["root_note"] = 48
    d["notes"] = [note_to_dict(Note(48, 100)), note_to_dict(Note(52, 60)), note_to_dict(Note(55, 55)),
                  note_to_dict(Note(60, 50)), note_to_dict(Note(64, 45))]
    d["synth_osc_d"] = 0.45
    d["synth_osc_v"] = 0.6
    d["synth_osc_stereo_v_offset"] = 0.15
    d["synth_osc_volume"] = 0.72
    d["synth_filter_cutoff"] = 3800.0
    d["synth_filter_resonance"] = 0.15
    d["synth_filter_env_amount"] = 1400.0
    d["synth_vol_attack"] = 5.0
    d["synth_vol_decay"] = 280.0
    d["synth_vol_sustain"] = 0.52
    d["synth_vol_release"] = 380.0
    d["synth_reverb_mix"] = 0.18
    d["synth_reverb_decay"] = 0.55
    d["note_length_percent"] = 85.0
    presets.append(create_preset("Tiesto Classic", "Factory",
        "Golden era trance - melodies that defined a generation", d))

    # 27. Hybrid Progressive - Modern fusion
    d = create_default_preset()
    d["straight_1_8"] = [100.0, 80.0, 85.0, 70.0, 95.0, 75.0, 88.0, 65.0]
    d["straight_1_16"] = [55.0, 42.0, 50.0, 38.0, 60.0, 45.0, 52.0, 40.0, 58.0, 44.0, 48.0, 36.0, 62.0, 48.0, 55.0, 42.0]
    d["triplet_1_8t"] = [45.0, 32.0, 38.0, 48.0, 35.0, 42.0, 42.0, 30.0, 36.0, 50.0, 38.0, 44.0]
    d["strength_values"] = create_strength_pattern("syncopated")
    d["root_note"] = 41
    d["notes"] = [note_to_dict(Note(41, 100)), note_to_dict(Note(44, 55)), note_to_dict(Note(48, 50)),
                  note_to_dict(Note(53, 45)), note_to_dict(Note(56, 40)), note_to_dict(Note(60, 35))]
    d["synth_osc_d"] = 0.38
    d["synth_osc_v"] = 0.62
    d["synth_osc_stereo_v_offset"] = 0.12
    d["synth_osc_volume"] = 0.68
    d["synth_filter_cutoff"] = 3200.0
    d["synth_filter_resonance"] = 0.18
    d["synth_filter_env_amount"] = 1200.0
    d["synth_vol_attack"] = 3.0
    d["synth_vol_decay"] = 220.0
    d["synth_vol_sustain"] = 0.48
    d["synth_vol_release"] = 300.0
    d["synth_reverb_mix"] = 0.18
    d["synth_reverb_decay"] = 0.58
    d["note_length_percent"] = 75.0
    presets.append(create_preset("Hybrid Progressive", "Factory",
        "Modern crossover - blending genres for new territory", d))

    # 28. Offbeat Bass - Pumping grooves
    d = create_default_preset()
    d["straight_1_8"] = [0.0, 100.0, 0.0, 95.0, 0.0, 100.0, 0.0, 90.0]
    d["strength_values"] = create_strength_pattern("syncopated")
    d["root_note"] = 36
    d["notes"] = [note_to_dict(Note(36, 100)), note_to_dict(Note(41, 55)), note_to_dict(Note(43, 50)),
                  note_to_dict(Note(48, 45))]
    d["synth_osc_d"] = 0.5
    d["synth_osc_v"] = 0.4
    d["synth_osc_volume"] = 0.72
    d["synth_sub_volume"] = 0.35
    d["synth_filter_cutoff"] = 1500.0
    d["synth_filter_resonance"] = 0.35
    d["synth_filter_env_amount"] = 1800.0
    d["synth_vol_attack"] = 0.5
    d["synth_vol_decay"] = 150.0
    d["synth_vol_sustain"] = 0.4
    d["synth_vol_release"] = 200.0
    d["note_length_percent"] = 65.0
    presets.append(create_preset("Offbeat Bass", "Factory",
        "Classic pump - offbeat groove that moves bodies", d))

    # 29. Hi-Tech Minimal - Twisted micro sounds
    d = create_default_preset()
    d["straight_1_32"] = [60.0, 40.0, 52.0, 35.0, 58.0, 42.0, 48.0, 30.0, 62.0, 44.0, 55.0, 38.0, 56.0, 40.0, 50.0, 32.0,
                          64.0, 46.0, 54.0, 36.0, 60.0, 42.0, 52.0, 34.0, 58.0, 40.0, 48.0, 30.0, 66.0, 48.0, 56.0, 38.0]
    d["strength_values"] = create_strength_pattern("dense")
    d["root_note"] = 36
    d["notes"] = [note_to_dict(Note(36, 100)), note_to_dict(Note(39, 55)), note_to_dict(Note(41, 50)),
                  note_to_dict(Note(43, 45)), note_to_dict(Note(48, 40))]
    d["synth_osc_d"] = 0.68
    d["synth_osc_v"] = 0.32
    d["synth_osc_volume"] = 0.72
    d["synth_filter_cutoff"] = 2500.0
    d["synth_filter_resonance"] = 0.4
    d["synth_filter_env_amount"] = 2500.0
    d["synth_vol_attack"] = 0.5
    d["synth_vol_decay"] = 60.0
    d["synth_vol_sustain"] = 0.2
    d["synth_vol_release"] = 100.0
    d["note_length_percent"] = 30.0
    d["lfo1_tempo_sync"] = True
    d["lfo1_sync_division"] = 4
    d["lfo1_waveform"] = 4
    d["lfo1_dest1"] = 12
    d["lfo1_amount1"] = 0.35
    presets.append(create_preset("Hi-Tech Minimal", "Factory",
        "Microscopic detail - tiny sounds create complex patterns", d))

    # 30. Anjuna Deep - Warm and emotional
    d = create_default_preset()
    d["straight_1_8"] = [100.0, 75.0, 85.0, 65.0, 95.0, 70.0, 80.0, 60.0]
    d["straight_1_16"] = [50.0, 35.0, 45.0, 30.0, 55.0, 40.0, 48.0, 32.0, 52.0, 38.0, 46.0, 34.0, 58.0, 42.0, 50.0, 36.0]
    d["strength_values"] = create_strength_pattern("4_4_standard")
    d["root_note"] = 41
    d["notes"] = [note_to_dict(Note(41, 100)), note_to_dict(Note(45, 58)), note_to_dict(Note(48, 55)),
                  note_to_dict(Note(53, 50)), note_to_dict(Note(57, 45)), note_to_dict(Note(60, 38))]
    d["synth_osc_d"] = 0.32
    d["synth_osc_v"] = 0.68
    d["synth_osc_stereo_v_offset"] = 0.15
    d["synth_osc_volume"] = 0.62
    d["synth_filter_cutoff"] = 3500.0
    d["synth_filter_resonance"] = 0.12
    d["synth_filter_env_amount"] = 800.0
    d["synth_vol_attack"] = 15.0
    d["synth_vol_decay"] = 300.0
    d["synth_vol_sustain"] = 0.52
    d["synth_vol_release"] = 450.0
    d["synth_reverb_mix"] = 0.2
    d["synth_reverb_decay"] = 0.62
    d["note_length_percent"] = 95.0
    presets.append(create_preset("Anjuna Deep", "Factory",
        "Emotional warmth - deep grooves that touch the heart", d))

    # 31. UK Trance - Classic sound
    d = create_default_preset()
    d["straight_1_4"] = [100.0, 90.0, 95.0, 85.0]
    d["straight_1_8"] = [75.0, 65.0, 70.0, 60.0, 80.0, 68.0, 72.0, 62.0]
    d["straight_1_16"] = [50.0, 40.0, 45.0, 35.0, 55.0, 42.0, 48.0, 38.0, 52.0, 44.0, 46.0, 36.0, 58.0, 45.0, 50.0, 40.0]
    d["strength_values"] = create_strength_pattern("driving")
    d["root_note"] = 48
    d["notes"] = [note_to_dict(Note(48, 100)), note_to_dict(Note(52, 58)), note_to_dict(Note(55, 55)),
                  note_to_dict(Note(60, 50)), note_to_dict(Note(64, 45))]
    d["synth_osc_d"] = 0.48
    d["synth_osc_v"] = 0.58
    d["synth_osc_stereo_v_offset"] = 0.12
    d["synth_osc_volume"] = 0.72
    d["synth_filter_cutoff"] = 4000.0
    d["synth_filter_resonance"] = 0.18
    d["synth_filter_env_amount"] = 1600.0
    d["synth_vol_attack"] = 3.0
    d["synth_vol_decay"] = 250.0
    d["synth_vol_sustain"] = 0.5
    d["synth_vol_release"] = 350.0
    d["synth_reverb_mix"] = 0.18
    d["synth_reverb_decay"] = 0.55
    d["note_length_percent"] = 80.0
    presets.append(create_preset("UK Trance", "Factory",
        "British anthems - Gatecrasher and Godskitchen vibes", d))

    # 32. Closing Set - Journey's end
    d = create_default_preset()
    d["straight_1_2"] = [100.0, 85.0]
    d["straight_1_4"] = [65.0, 50.0, 60.0, 45.0]
    d["straight_1_8"] = [40.0, 30.0, 35.0, 25.0, 45.0, 32.0, 38.0, 28.0]
    d["strength_values"] = create_strength_pattern("ambient")
    d["root_note"] = 48
    d["notes"] = [note_to_dict(Note(48, 100)), note_to_dict(Note(52, 60)), note_to_dict(Note(55, 55)),
                  note_to_dict(Note(60, 50)), note_to_dict(Note(64, 45)), note_to_dict(Note(67, 40))]
    d["synth_osc_d"] = 0.25
    d["synth_osc_v"] = 0.72
    d["synth_osc_stereo_v_offset"] = 0.18
    d["synth_osc_volume"] = 0.58
    d["synth_filter_cutoff"] = 3000.0
    d["synth_filter_resonance"] = 0.1
    d["synth_filter_env_amount"] = 500.0
    d["synth_vol_attack"] = 150.0
    d["synth_vol_decay"] = 1000.0
    d["synth_vol_sustain"] = 0.6
    d["synth_vol_release"] = 1800.0
    d["synth_reverb_mix"] = 0.25
    d["synth_reverb_decay"] = 0.78
    d["synth_reverb_diffusion"] = 0.9
    d["note_length_percent"] = 180.0
    presets.append(create_preset("Closing Set", "Factory",
        "Dawn breaks - gentle melodies as the night winds down", d))

    return presets

def create_bank_g() -> List[Dict]:
    """Bank G: Cinematic & Classic - 32 presets"""
    presets = []

    # 1. Epic Dawn - Orchestral opening
    d = create_default_preset()
    d["straight_1_1"] = [100.0]
    d["straight_1_2"] = [80.0, 70.0]
    d["straight_1_4"] = [55.0, 45.0, 50.0, 40.0]
    d["strength_values"] = create_strength_pattern("ambient")
    d["root_note"] = 36
    d["notes"] = [note_to_dict(Note(36, 100)), note_to_dict(Note(43, 60)), note_to_dict(Note(48, 55)),
                  note_to_dict(Note(55, 50)), note_to_dict(Note(60, 45))]
    d["synth_osc_d"] = 0.2
    d["synth_osc_v"] = 0.75
    d["synth_osc_stereo_v_offset"] = 0.18
    d["synth_osc_volume"] = 0.58
    d["synth_filter_cutoff"] = 2800.0
    d["synth_filter_resonance"] = 0.08
    d["synth_filter_env_amount"] = 600.0
    d["synth_vol_attack"] = 300.0
    d["synth_vol_decay"] = 1500.0
    d["synth_vol_sustain"] = 0.65
    d["synth_vol_release"] = 2500.0
    d["synth_reverb_mix"] = 0.22
    d["synth_reverb_decay"] = 0.75
    d["synth_reverb_diffusion"] = 0.88
    d["note_length_percent"] = 200.0
    presets.append(create_preset("Epic Dawn", "Factory",
        "Cinematic opening - massive orchestral swells herald the story", d))

    # 2. Tension Rising - Suspense build
    d = create_default_preset()
    d["straight_1_4"] = [100.0, 85.0, 90.0, 80.0]
    d["straight_1_8"] = [70.0, 55.0, 65.0, 50.0, 75.0, 60.0, 68.0, 52.0]
    d["straight_1_16"] = [45.0, 35.0, 40.0, 30.0, 48.0, 38.0, 42.0, 32.0, 46.0, 36.0, 41.0, 31.0, 50.0, 40.0, 44.0, 34.0]
    d["strength_values"] = create_strength_pattern("driving")
    d["root_note"] = 41
    d["notes"] = [note_to_dict(Note(41, 100)), note_to_dict(Note(44, 65)), note_to_dict(Note(47, 60)),
                  note_to_dict(Note(48, 55)), note_to_dict(Note(53, 50))]
    d["synth_osc_d"] = 0.42
    d["synth_osc_v"] = 0.55
    d["synth_osc_volume"] = 0.65
    d["synth_filter_cutoff"] = 2200.0
    d["synth_filter_resonance"] = 0.2
    d["synth_filter_env_amount"] = 1200.0
    d["synth_vol_attack"] = 10.0
    d["synth_vol_decay"] = 400.0
    d["synth_vol_sustain"] = 0.5
    d["synth_vol_release"] = 500.0
    d["synth_reverb_mix"] = 0.18
    d["synth_reverb_decay"] = 0.55
    d["note_length_percent"] = 90.0
    d["lfo1_rate"] = 0.08
    d["lfo1_waveform"] = 2
    d["lfo1_dest1"] = 12
    d["lfo1_amount1"] = 0.15
    presets.append(create_preset("Tension Rising", "Factory",
        "Building dread - chromatic movement creates unease", d))

    # 3. Romantic Theme - Love story
    d = create_default_preset()
    d["straight_1_2"] = [100.0, 85.0]
    d["straight_1_4"] = [70.0, 55.0, 65.0, 50.0]
    d["dotted_1_4d"] = [60.0, 45.0, 52.0]
    d["strength_values"] = create_strength_pattern("ambient")
    d["root_note"] = 53
    d["notes"] = [note_to_dict(Note(53, 100)), note_to_dict(Note(57, 60)), note_to_dict(Note(60, 55)),
                  note_to_dict(Note(65, 50)), note_to_dict(Note(69, 45)), note_to_dict(Note(72, 40))]
    d["synth_osc_d"] = 0.25
    d["synth_osc_v"] = 0.72
    d["synth_osc_stereo_v_offset"] = 0.15
    d["synth_osc_volume"] = 0.55
    d["synth_filter_cutoff"] = 3500.0
    d["synth_filter_resonance"] = 0.1
    d["synth_filter_env_amount"] = 500.0
    d["synth_vol_attack"] = 150.0
    d["synth_vol_decay"] = 1200.0
    d["synth_vol_sustain"] = 0.6
    d["synth_vol_release"] = 2000.0
    d["synth_reverb_mix"] = 0.2
    d["synth_reverb_decay"] = 0.68
    d["synth_reverb_diffusion"] = 0.85
    d["note_length_percent"] = 180.0
    presets.append(create_preset("Romantic Theme", "Factory",
        "Sweeping romance - lush strings carry the heart", d))

    # 4. Action Sequence - Chase scene
    d = create_default_preset()
    d["straight_1_8"] = [100.0, 90.0, 95.0, 85.0, 100.0, 88.0, 92.0, 82.0]
    d["straight_1_16"] = [75.0, 65.0, 70.0, 60.0, 78.0, 68.0, 72.0, 62.0, 76.0, 66.0, 71.0, 61.0, 80.0, 70.0, 74.0, 64.0]
    d["strength_values"] = create_strength_pattern("driving")
    d["root_note"] = 36
    d["notes"] = [note_to_dict(Note(36, 100)), note_to_dict(Note(41, 55)), note_to_dict(Note(43, 50)),
                  note_to_dict(Note(48, 45))]
    d["synth_osc_d"] = 0.55
    d["synth_osc_v"] = 0.42
    d["synth_osc_volume"] = 0.72
    d["synth_sub_volume"] = 0.25
    d["synth_filter_cutoff"] = 1800.0
    d["synth_filter_resonance"] = 0.25
    d["synth_filter_env_amount"] = 1500.0
    d["synth_vol_attack"] = 2.0
    d["synth_vol_decay"] = 150.0
    d["synth_vol_sustain"] = 0.45
    d["synth_vol_release"] = 200.0
    d["note_length_percent"] = 70.0
    presets.append(create_preset("Action Sequence", "Factory",
        "Relentless pursuit - driving ostinatos fuel the chase", d))

    # 5. Mystery Fog - Noir detective
    d = create_default_preset()
    d["straight_1_4"] = [100.0, 0.0, 80.0, 0.0]
    d["straight_1_8"] = [0.0, 60.0, 0.0, 55.0, 0.0, 65.0, 0.0, 50.0]
    d["triplet_1_8t"] = [50.0, 0.0, 45.0, 55.0, 0.0, 48.0, 52.0, 0.0, 42.0, 58.0, 0.0, 46.0]
    d["strength_values"] = create_strength_pattern("jazz")
    d["root_note"] = 41
    d["notes"] = [note_to_dict(Note(41, 100)), note_to_dict(Note(44, 60)), note_to_dict(Note(47, 55)),
                  note_to_dict(Note(50, 50)), note_to_dict(Note(53, 45))]
    d["synth_osc_d"] = 0.35
    d["synth_osc_v"] = 0.62
    d["synth_osc_stereo_v_offset"] = 0.12
    d["synth_osc_volume"] = 0.6
    d["synth_filter_cutoff"] = 2500.0
    d["synth_filter_resonance"] = 0.15
    d["synth_filter_env_amount"] = 800.0
    d["synth_vol_attack"] = 20.0
    d["synth_vol_decay"] = 350.0
    d["synth_vol_sustain"] = 0.5
    d["synth_vol_release"] = 500.0
    d["synth_reverb_mix"] = 0.2
    d["synth_reverb_decay"] = 0.6
    d["note_length_percent"] = 85.0
    presets.append(create_preset("Mystery Fog", "Factory",
        "Smoky noir - jazz-tinged shadows and intrigue", d))

    # 6. Sci-Fi Horizon - Space exploration
    d = create_default_preset()
    d["straight_1_2"] = [100.0, 75.0]
    d["straight_1_4"] = [55.0, 40.0, 50.0, 35.0]
    d["straight_1_8"] = [35.0, 25.0, 30.0, 20.0, 38.0, 28.0, 32.0, 22.0]
    d["strength_values"] = create_strength_pattern("ambient")
    d["root_note"] = 48
    d["notes"] = [note_to_dict(Note(48, 100)), note_to_dict(Note(52, 55)), note_to_dict(Note(55, 50)),
                  note_to_dict(Note(60, 45)), note_to_dict(Note(64, 40)), note_to_dict(Note(67, 35))]
    d["octave_randomization"] = create_octave_randomization(0.14, 0.31, 0.27, "Both")
    d["synth_osc_d"] = 0.3
    d["synth_osc_v"] = 0.7
    d["synth_osc_stereo_v_offset"] = 0.2
    d["synth_osc_volume"] = 0.55
    d["synth_filter_cutoff"] = 4000.0
    d["synth_filter_resonance"] = 0.12
    d["synth_filter_env_amount"] = 600.0
    d["synth_vol_attack"] = 200.0
    d["synth_vol_decay"] = 1500.0
    d["synth_vol_sustain"] = 0.6
    d["synth_vol_release"] = 2500.0
    d["synth_reverb_mix"] = 0.25
    d["synth_reverb_decay"] = 0.8
    d["synth_reverb_diffusion"] = 0.92
    d["note_length_percent"] = 200.0
    d["lfo1_rate"] = 0.05
    d["lfo1_waveform"] = 0
    d["lfo1_dest1"] = 11
    d["lfo1_amount1"] = 0.12
    presets.append(create_preset("Sci-Fi Horizon", "Factory",
        "Vast cosmos - ethereal pads drift through infinite space", d))

    # 7. Classical Waltz - Three-four elegance
    d = create_default_preset()
    d["triplet_1_4t"] = [100.0, 70.0, 75.0, 95.0, 65.0, 72.0]
    d["triplet_1_8t"] = [55.0, 40.0, 45.0, 50.0, 38.0, 42.0, 58.0, 42.0, 48.0, 52.0, 40.0, 44.0]
    d["strength_values"] = create_strength_pattern("triplet_feel")
    d["root_note"] = 48
    d["notes"] = [note_to_dict(Note(48, 100)), note_to_dict(Note(52, 60)), note_to_dict(Note(55, 55)),
                  note_to_dict(Note(60, 50)), note_to_dict(Note(64, 45))]
    d["synth_osc_d"] = 0.28
    d["synth_osc_v"] = 0.68
    d["synth_osc_volume"] = 0.62
    d["synth_filter_cutoff"] = 3200.0
    d["synth_filter_resonance"] = 0.1
    d["synth_filter_env_amount"] = 700.0
    d["synth_vol_attack"] = 8.0
    d["synth_vol_decay"] = 300.0
    d["synth_vol_sustain"] = 0.5
    d["synth_vol_release"] = 400.0
    d["synth_reverb_mix"] = 0.18
    d["synth_reverb_decay"] = 0.55
    d["note_length_percent"] = 75.0
    presets.append(create_preset("Classical Waltz", "Factory",
        "Ballroom elegance - graceful three-four time swirls", d))

    # 8. Horror Stinger - Jump scare
    d = create_default_preset()
    d["straight_1_16"] = [100.0, 95.0, 90.0, 85.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]
    d["straight_1_32"] = [80.0, 75.0, 70.0, 65.0, 60.0, 55.0, 50.0, 45.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
                          0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]
    d["strength_values"] = create_strength_pattern("sparse")
    d["root_note"] = 36
    d["notes"] = [note_to_dict(Note(36, 100)), note_to_dict(Note(37, 70)), note_to_dict(Note(42, 65)),
                  note_to_dict(Note(43, 60)), note_to_dict(Note(48, 55))]
    d["synth_osc_d"] = 0.6
    d["synth_osc_v"] = 0.35
    d["synth_osc_volume"] = 0.78
    d["synth_filter_cutoff"] = 1500.0
    d["synth_filter_resonance"] = 0.35
    d["synth_filter_env_amount"] = 2500.0
    d["synth_vol_attack"] = 0.5
    d["synth_vol_decay"] = 80.0
    d["synth_vol_sustain"] = 0.2
    d["synth_vol_release"] = 150.0
    d["synth_reverb_mix"] = 0.15
    d["synth_reverb_decay"] = 0.45
    d["note_length_percent"] = 40.0
    presets.append(create_preset("Horror Stinger", "Factory",
        "Sudden terror - dissonant stabs pierce the silence", d))

    # 9. Period Drama - Historical elegance
    d = create_default_preset()
    d["straight_1_4"] = [100.0, 80.0, 90.0, 75.0]
    d["straight_1_8"] = [60.0, 45.0, 55.0, 40.0, 65.0, 48.0, 58.0, 42.0]
    d["strength_values"] = create_strength_pattern("4_4_standard")
    d["root_note"] = 53
    d["notes"] = [note_to_dict(Note(53, 100)), note_to_dict(Note(57, 60)), note_to_dict(Note(60, 55)),
                  note_to_dict(Note(65, 50)), note_to_dict(Note(69, 45))]
    d["synth_osc_d"] = 0.22
    d["synth_osc_v"] = 0.7
    d["synth_osc_stereo_v_offset"] = 0.1
    d["synth_osc_volume"] = 0.6
    d["synth_filter_cutoff"] = 3000.0
    d["synth_filter_resonance"] = 0.08
    d["synth_filter_env_amount"] = 400.0
    d["synth_vol_attack"] = 50.0
    d["synth_vol_decay"] = 500.0
    d["synth_vol_sustain"] = 0.55
    d["synth_vol_release"] = 700.0
    d["synth_reverb_mix"] = 0.18
    d["synth_reverb_decay"] = 0.58
    d["note_length_percent"] = 95.0
    presets.append(create_preset("Period Drama", "Factory",
        "Victorian grace - refined melodies from another era", d))

    # 10. Battle March - Epic warfare
    d = create_default_preset()
    d["straight_1_4"] = [100.0, 95.0, 100.0, 90.0]
    d["straight_1_8"] = [85.0, 75.0, 80.0, 70.0, 88.0, 78.0, 82.0, 72.0]
    d["strength_values"] = create_strength_pattern("driving")
    d["root_note"] = 36
    d["notes"] = [note_to_dict(Note(36, 100)), note_to_dict(Note(41, 55)), note_to_dict(Note(43, 50)),
                  note_to_dict(Note(48, 45)), note_to_dict(Note(53, 40))]
    d["synth_osc_d"] = 0.5
    d["synth_osc_v"] = 0.45
    d["synth_osc_volume"] = 0.75
    d["synth_sub_volume"] = 0.3
    d["synth_filter_cutoff"] = 2000.0
    d["synth_filter_resonance"] = 0.2
    d["synth_filter_env_amount"] = 1000.0
    d["synth_vol_attack"] = 5.0
    d["synth_vol_decay"] = 200.0
    d["synth_vol_sustain"] = 0.55
    d["synth_vol_release"] = 300.0
    d["note_length_percent"] = 80.0
    presets.append(create_preset("Battle March", "Factory",
        "Drums of war - relentless rhythm drives the charge", d))

    # 11. Funeral Dirge - Solemn mourning
    d = create_default_preset()
    d["straight_1_1"] = [100.0]
    d["straight_1_2"] = [70.0, 60.0]
    d["straight_1_4"] = [45.0, 35.0, 40.0, 30.0]
    d["strength_values"] = create_strength_pattern("sparse")
    d["root_note"] = 41
    d["notes"] = [note_to_dict(Note(41, 100)), note_to_dict(Note(44, 60)), note_to_dict(Note(48, 55)),
                  note_to_dict(Note(53, 50)), note_to_dict(Note(56, 45))]
    d["synth_osc_d"] = 0.18
    d["synth_osc_v"] = 0.75
    d["synth_osc_stereo_v_offset"] = 0.12
    d["synth_osc_volume"] = 0.52
    d["synth_filter_cutoff"] = 2200.0
    d["synth_filter_resonance"] = 0.06
    d["synth_filter_env_amount"] = 300.0
    d["synth_vol_attack"] = 250.0
    d["synth_vol_decay"] = 1800.0
    d["synth_vol_sustain"] = 0.55
    d["synth_vol_release"] = 3000.0
    d["synth_reverb_mix"] = 0.22
    d["synth_reverb_decay"] = 0.72
    d["synth_reverb_diffusion"] = 0.88
    d["note_length_percent"] = 200.0
    presets.append(create_preset("Funeral Dirge", "Factory",
        "Final farewell - heavy tones mark the passing", d))

    # 12. Discovery Theme - Wonder and awe
    d = create_default_preset()
    d["straight_1_4"] = [100.0, 85.0, 90.0, 80.0]
    d["straight_1_8"] = [65.0, 50.0, 60.0, 45.0, 70.0, 55.0, 62.0, 48.0]
    d["dotted_1_4d"] = [55.0, 40.0, 48.0]
    d["strength_values"] = create_strength_pattern("4_4_standard")
    d["root_note"] = 48
    d["notes"] = [note_to_dict(Note(48, 100)), note_to_dict(Note(52, 60)), note_to_dict(Note(55, 55)),
                  note_to_dict(Note(60, 50)), note_to_dict(Note(64, 45)), note_to_dict(Note(67, 40))]
    d["octave_randomization"] = create_octave_randomization(0.12, 0.35, 0.31, "Up")
    d["synth_osc_d"] = 0.32
    d["synth_osc_v"] = 0.65
    d["synth_osc_stereo_v_offset"] = 0.15
    d["synth_osc_volume"] = 0.62
    d["synth_filter_cutoff"] = 3800.0
    d["synth_filter_resonance"] = 0.12
    d["synth_filter_env_amount"] = 800.0
    d["synth_vol_attack"] = 30.0
    d["synth_vol_decay"] = 600.0
    d["synth_vol_sustain"] = 0.55
    d["synth_vol_release"] = 900.0
    d["synth_reverb_mix"] = 0.2
    d["synth_reverb_decay"] = 0.62
    d["note_length_percent"] = 110.0
    presets.append(create_preset("Discovery Theme", "Factory",
        "New horizons - uplifting melody captures wonder", d))

    # 13. Villain Motif - Dark antagonist
    d = create_default_preset()
    d["straight_1_4"] = [100.0, 0.0, 90.0, 0.0]
    d["straight_1_8"] = [80.0, 70.0, 0.0, 65.0, 85.0, 72.0, 0.0, 68.0]
    d["strength_values"] = create_strength_pattern("driving")
    d["root_note"] = 36
    d["notes"] = [note_to_dict(Note(36, 100)), note_to_dict(Note(39, 65)), note_to_dict(Note(42, 60)),
                  note_to_dict(Note(45, 55)), note_to_dict(Note(48, 50))]
    d["synth_osc_d"] = 0.58
    d["synth_osc_v"] = 0.38
    d["synth_osc_volume"] = 0.7
    d["synth_sub_volume"] = 0.25
    d["synth_filter_cutoff"] = 1600.0
    d["synth_filter_resonance"] = 0.28
    d["synth_filter_env_amount"] = 1200.0
    d["synth_vol_attack"] = 8.0
    d["synth_vol_decay"] = 250.0
    d["synth_vol_sustain"] = 0.45
    d["synth_vol_release"] = 350.0
    d["synth_reverb_mix"] = 0.15
    d["synth_reverb_decay"] = 0.5
    d["note_length_percent"] = 75.0
    presets.append(create_preset("Villain Motif", "Factory",
        "Dark presence - menacing theme stalks the hero", d))

    # 14. Triumph Fanfare - Victory celebration
    d = create_default_preset()
    d["straight_1_4"] = [100.0, 90.0, 95.0, 85.0]
    d["straight_1_8"] = [75.0, 60.0, 70.0, 55.0, 80.0, 65.0, 72.0, 58.0]
    d["strength_values"] = create_strength_pattern("4_4_standard")
    d["root_note"] = 48
    d["notes"] = [note_to_dict(Note(48, 100)), note_to_dict(Note(52, 60)), note_to_dict(Note(55, 55)),
                  note_to_dict(Note(60, 50)), note_to_dict(Note(64, 45))]
    d["synth_osc_d"] = 0.45
    d["synth_osc_v"] = 0.52
    d["synth_osc_volume"] = 0.72
    d["synth_filter_cutoff"] = 4200.0
    d["synth_filter_resonance"] = 0.15
    d["synth_filter_env_amount"] = 1000.0
    d["synth_vol_attack"] = 3.0
    d["synth_vol_decay"] = 300.0
    d["synth_vol_sustain"] = 0.5
    d["synth_vol_release"] = 400.0
    d["synth_reverb_mix"] = 0.18
    d["synth_reverb_decay"] = 0.55
    d["note_length_percent"] = 85.0
    presets.append(create_preset("Triumph Fanfare", "Factory",
        "Glorious victory - brass-like tones herald success", d))

    # 15. Alien Planet - Otherworldly texture
    d = create_default_preset()
    d["straight_1_8"] = [100.0, 0.0, 75.0, 0.0, 85.0, 0.0, 70.0, 0.0]
    d["triplet_1_8t"] = [60.0, 45.0, 0.0, 55.0, 40.0, 0.0, 62.0, 48.0, 0.0, 58.0, 42.0, 0.0]
    d["strength_values"] = create_strength_pattern("polyrhythm_3_4")
    d["root_note"] = 48
    d["notes"] = [note_to_dict(Note(48, 100)), note_to_dict(Note(50, 60)), note_to_dict(Note(54, 55)),
                  note_to_dict(Note(56, 50)), note_to_dict(Note(60, 45)), note_to_dict(Note(62, 40))]
    d["octave_randomization"] = create_octave_randomization(0.16, 0.25, 0.25, "Both")
    d["synth_osc_d"] = 0.48
    d["synth_osc_v"] = 0.58
    d["synth_osc_stereo_v_offset"] = 0.22
    d["synth_osc_volume"] = 0.6
    d["synth_filter_cutoff"] = 3200.0
    d["synth_filter_resonance"] = 0.25
    d["synth_filter_env_amount"] = 1500.0
    d["synth_vol_attack"] = 15.0
    d["synth_vol_decay"] = 350.0
    d["synth_vol_sustain"] = 0.45
    d["synth_vol_release"] = 500.0
    d["synth_reverb_mix"] = 0.22
    d["synth_reverb_decay"] = 0.65
    d["note_length_percent"] = 70.0
    d["lfo1_rate"] = 0.12
    d["lfo1_waveform"] = 4
    d["lfo1_dest1"] = 10
    d["lfo1_amount1"] = 0.18
    presets.append(create_preset("Alien Planet", "Factory",
        "Strange world - exotic scales hint at alien life", d))

    # 16. Minimalist Piano - Philip Glass style
    d = create_default_preset()
    d["straight_1_8"] = [100.0, 85.0, 90.0, 80.0, 95.0, 82.0, 88.0, 78.0]
    d["straight_1_16"] = [65.0, 55.0, 60.0, 50.0, 68.0, 58.0, 62.0, 52.0, 66.0, 56.0, 61.0, 51.0, 70.0, 60.0, 64.0, 54.0]
    d["strength_values"] = create_strength_pattern("4_4_standard")
    d["root_note"] = 48
    d["notes"] = [note_to_dict(Note(48, 100)), note_to_dict(Note(52, 55)), note_to_dict(Note(55, 50)),
                  note_to_dict(Note(60, 45)), note_to_dict(Note(64, 40))]
    d["synth_osc_d"] = 0.35
    d["synth_osc_v"] = 0.6
    d["synth_osc_volume"] = 0.65
    d["synth_filter_cutoff"] = 4500.0
    d["synth_filter_resonance"] = 0.08
    d["synth_filter_env_amount"] = 500.0
    d["synth_vol_attack"] = 2.0
    d["synth_vol_decay"] = 400.0
    d["synth_vol_sustain"] = 0.35
    d["synth_vol_release"] = 600.0
    d["synth_reverb_mix"] = 0.18
    d["synth_reverb_decay"] = 0.52
    d["note_length_percent"] = 65.0
    presets.append(create_preset("Minimalist Piano", "Factory",
        "Repetitive beauty - arpeggios cycle in shifting patterns", d))

    # 17. Baroque Sequence - Bach-inspired
    d = create_default_preset()
    d["straight_1_16"] = [100.0, 80.0, 85.0, 75.0, 95.0, 78.0, 82.0, 72.0, 90.0, 76.0, 84.0, 70.0, 88.0, 74.0, 80.0, 68.0]
    d["strength_values"] = create_strength_pattern("4_4_standard")
    d["root_note"] = 48
    d["notes"] = [note_to_dict(Note(48, 100)), note_to_dict(Note(52, 60)), note_to_dict(Note(55, 55)),
                  note_to_dict(Note(59, 50)), note_to_dict(Note(60, 45)), note_to_dict(Note(64, 40))]
    d["synth_osc_d"] = 0.3
    d["synth_osc_v"] = 0.65
    d["synth_osc_volume"] = 0.62
    d["synth_filter_cutoff"] = 4000.0
    d["synth_filter_resonance"] = 0.1
    d["synth_filter_env_amount"] = 600.0
    d["synth_vol_attack"] = 1.0
    d["synth_vol_decay"] = 200.0
    d["synth_vol_sustain"] = 0.4
    d["synth_vol_release"] = 300.0
    d["synth_reverb_mix"] = 0.15
    d["synth_reverb_decay"] = 0.48
    d["note_length_percent"] = 55.0
    presets.append(create_preset("Baroque Sequence", "Factory",
        "Counterpoint dance - interlocking lines weave together", d))

    # 18. Underwater World - Aquatic ambience
    d = create_default_preset()
    d["straight_1_4"] = [100.0, 70.0, 80.0, 60.0]
    d["straight_1_8"] = [50.0, 35.0, 45.0, 30.0, 55.0, 40.0, 48.0, 32.0]
    d["dotted_1_8d"] = [45.0, 30.0, 38.0, 48.0, 32.0, 40.0]
    d["strength_values"] = create_strength_pattern("ambient")
    d["root_note"] = 48
    d["notes"] = [note_to_dict(Note(48, 100)), note_to_dict(Note(52, 55)), note_to_dict(Note(55, 50)),
                  note_to_dict(Note(60, 45)), note_to_dict(Note(64, 40)), note_to_dict(Note(67, 35))]
    d["synth_osc_d"] = 0.22
    d["synth_osc_v"] = 0.72
    d["synth_osc_stereo_v_offset"] = 0.18
    d["synth_osc_volume"] = 0.55
    d["synth_filter_cutoff"] = 2800.0
    d["synth_filter_resonance"] = 0.15
    d["synth_filter_env_amount"] = 500.0
    d["synth_vol_attack"] = 100.0
    d["synth_vol_decay"] = 800.0
    d["synth_vol_sustain"] = 0.55
    d["synth_vol_release"] = 1500.0
    d["synth_reverb_mix"] = 0.25
    d["synth_reverb_decay"] = 0.75
    d["synth_reverb_diffusion"] = 0.9
    d["note_length_percent"] = 150.0
    d["lfo1_rate"] = 0.08
    d["lfo1_waveform"] = 0
    d["lfo1_dest1"] = 12
    d["lfo1_amount1"] = 0.1
    presets.append(create_preset("Underwater World", "Factory",
        "Deep blue - filtered sounds drift through the depths", d))

    # 19. Heroic Journey - Adventure theme
    d = create_default_preset()
    d["straight_1_4"] = [100.0, 85.0, 95.0, 80.0]
    d["straight_1_8"] = [70.0, 55.0, 65.0, 50.0, 75.0, 60.0, 68.0, 52.0]
    d["strength_values"] = create_strength_pattern("4_4_standard")
    d["root_note"] = 48
    d["notes"] = [note_to_dict(Note(48, 100)), note_to_dict(Note(52, 60)), note_to_dict(Note(55, 55)),
                  note_to_dict(Note(60, 50)), note_to_dict(Note(64, 45))]
    d["octave_randomization"] = create_octave_randomization(0.1, 0.39, 0.33, "Up")
    d["synth_osc_d"] = 0.4
    d["synth_osc_v"] = 0.55
    d["synth_osc_volume"] = 0.68
    d["synth_filter_cutoff"] = 3500.0
    d["synth_filter_resonance"] = 0.12
    d["synth_filter_env_amount"] = 900.0
    d["synth_vol_attack"] = 10.0
    d["synth_vol_decay"] = 400.0
    d["synth_vol_sustain"] = 0.5
    d["synth_vol_release"] = 550.0
    d["synth_reverb_mix"] = 0.18
    d["synth_reverb_decay"] = 0.55
    d["note_length_percent"] = 90.0
    presets.append(create_preset("Heroic Journey", "Factory",
        "Quest begins - stirring melody calls to adventure", d))

    # 20. Night Forest - Mysterious nature
    d = create_default_preset()
    d["straight_1_8"] = [100.0, 0.0, 70.0, 0.0, 80.0, 0.0, 65.0, 0.0]
    d["triplet_1_8t"] = [55.0, 40.0, 45.0, 50.0, 35.0, 42.0, 58.0, 42.0, 48.0, 52.0, 38.0, 44.0]
    d["strength_values"] = create_strength_pattern("sparse")
    d["root_note"] = 41
    d["notes"] = [note_to_dict(Note(41, 100)), note_to_dict(Note(44, 58)), note_to_dict(Note(48, 52)),
                  note_to_dict(Note(53, 46)), note_to_dict(Note(56, 40))]
    d["synth_osc_d"] = 0.28
    d["synth_osc_v"] = 0.68
    d["synth_osc_stereo_v_offset"] = 0.15
    d["synth_osc_volume"] = 0.58
    d["synth_filter_cutoff"] = 2500.0
    d["synth_filter_resonance"] = 0.12
    d["synth_filter_env_amount"] = 600.0
    d["synth_vol_attack"] = 50.0
    d["synth_vol_decay"] = 600.0
    d["synth_vol_sustain"] = 0.5
    d["synth_vol_release"] = 900.0
    d["synth_reverb_mix"] = 0.22
    d["synth_reverb_decay"] = 0.65
    d["note_length_percent"] = 120.0
    presets.append(create_preset("Night Forest", "Factory",
        "Enchanted woods - mysterious sounds from shadows", d))

    # 21. Space Station - Orbital ambience
    d = create_default_preset()
    d["straight_1_2"] = [100.0, 80.0]
    d["straight_1_4"] = [60.0, 45.0, 55.0, 40.0]
    d["straight_1_8"] = [40.0, 30.0, 35.0, 25.0, 42.0, 32.0, 38.0, 28.0]
    d["strength_values"] = create_strength_pattern("ambient")
    d["root_note"] = 48
    d["notes"] = [note_to_dict(Note(48, 100)), note_to_dict(Note(52, 55)), note_to_dict(Note(55, 50)),
                  note_to_dict(Note(60, 45)), note_to_dict(Note(64, 40))]
    d["synth_osc_d"] = 0.25
    d["synth_osc_v"] = 0.72
    d["synth_osc_stereo_v_offset"] = 0.2
    d["synth_osc_volume"] = 0.55
    d["synth_filter_cutoff"] = 3500.0
    d["synth_filter_resonance"] = 0.1
    d["synth_filter_env_amount"] = 400.0
    d["synth_vol_attack"] = 200.0
    d["synth_vol_decay"] = 1500.0
    d["synth_vol_sustain"] = 0.6
    d["synth_vol_release"] = 2500.0
    d["synth_reverb_mix"] = 0.25
    d["synth_reverb_decay"] = 0.78
    d["synth_reverb_diffusion"] = 0.9
    d["note_length_percent"] = 200.0
    d["lfo1_rate"] = 0.03
    d["lfo1_waveform"] = 0
    d["lfo1_dest1"] = 11
    d["lfo1_amount1"] = 0.1
    presets.append(create_preset("Space Station", "Factory",
        "Orbital drift - gentle tones float in zero gravity", d))

    # 22. Medieval Fair - Renaissance feel
    d = create_default_preset()
    d["straight_1_8"] = [100.0, 75.0, 85.0, 70.0, 95.0, 72.0, 82.0, 68.0]
    d["triplet_1_8t"] = [60.0, 45.0, 52.0, 55.0, 42.0, 48.0, 62.0, 48.0, 55.0, 58.0, 44.0, 50.0]
    d["strength_values"] = create_strength_pattern("4_4_standard")
    d["root_note"] = 50
    d["notes"] = [note_to_dict(Note(50, 100)), note_to_dict(Note(53, 58)), note_to_dict(Note(57, 52)),
                  note_to_dict(Note(62, 46)), note_to_dict(Note(65, 40))]
    d["synth_osc_d"] = 0.35
    d["synth_osc_v"] = 0.62
    d["synth_osc_volume"] = 0.62
    d["synth_filter_cutoff"] = 3800.0
    d["synth_filter_resonance"] = 0.12
    d["synth_filter_env_amount"] = 700.0
    d["synth_vol_attack"] = 5.0
    d["synth_vol_decay"] = 300.0
    d["synth_vol_sustain"] = 0.45
    d["synth_vol_release"] = 400.0
    d["synth_reverb_mix"] = 0.18
    d["synth_reverb_decay"] = 0.52
    d["note_length_percent"] = 70.0
    presets.append(create_preset("Medieval Fair", "Factory",
        "Olde tymes - lute-like patterns from the past", d))

    # 23. Cyber City - Blade Runner vibes
    d = create_default_preset()
    d["straight_1_8"] = [100.0, 85.0, 90.0, 80.0, 95.0, 82.0, 88.0, 78.0]
    d["straight_1_16"] = [60.0, 50.0, 55.0, 45.0, 65.0, 52.0, 58.0, 48.0, 62.0, 52.0, 56.0, 46.0, 68.0, 55.0, 60.0, 50.0]
    d["strength_values"] = create_strength_pattern("driving")
    d["root_note"] = 41
    d["notes"] = [note_to_dict(Note(41, 100)), note_to_dict(Note(44, 60)), note_to_dict(Note(48, 55)),
                  note_to_dict(Note(53, 50)), note_to_dict(Note(56, 45))]
    d["synth_osc_d"] = 0.45
    d["synth_osc_v"] = 0.55
    d["synth_osc_stereo_v_offset"] = 0.15
    d["synth_osc_volume"] = 0.65
    d["synth_filter_cutoff"] = 2800.0
    d["synth_filter_resonance"] = 0.2
    d["synth_filter_env_amount"] = 1200.0
    d["synth_vol_attack"] = 10.0
    d["synth_vol_decay"] = 350.0
    d["synth_vol_sustain"] = 0.5
    d["synth_vol_release"] = 450.0
    d["synth_reverb_mix"] = 0.2
    d["synth_reverb_decay"] = 0.58
    d["note_length_percent"] = 85.0
    d["lfo1_rate"] = 0.15
    d["lfo1_waveform"] = 2
    d["lfo1_dest1"] = 12
    d["lfo1_amount1"] = 0.12
    presets.append(create_preset("Cyber City", "Factory",
        "Neon rain - synthetic noir for dystopian nights", d))

    # 24. Fairy Tale - Magical storytelling
    d = create_default_preset()
    d["straight_1_4"] = [100.0, 80.0, 90.0, 75.0]
    d["straight_1_8"] = [60.0, 45.0, 55.0, 40.0, 65.0, 48.0, 58.0, 42.0]
    d["dotted_1_8d"] = [50.0, 35.0, 42.0, 55.0, 38.0, 45.0]
    d["strength_values"] = create_strength_pattern("4_4_standard")
    d["root_note"] = 53
    d["notes"] = [note_to_dict(Note(53, 100)), note_to_dict(Note(57, 60)), note_to_dict(Note(60, 55)),
                  note_to_dict(Note(65, 50)), note_to_dict(Note(69, 45)), note_to_dict(Note(72, 40))]
    d["synth_osc_d"] = 0.28
    d["synth_osc_v"] = 0.68
    d["synth_osc_stereo_v_offset"] = 0.12
    d["synth_osc_volume"] = 0.58
    d["synth_filter_cutoff"] = 4200.0
    d["synth_filter_resonance"] = 0.1
    d["synth_filter_env_amount"] = 600.0
    d["synth_vol_attack"] = 15.0
    d["synth_vol_decay"] = 400.0
    d["synth_vol_sustain"] = 0.5
    d["synth_vol_release"] = 600.0
    d["synth_reverb_mix"] = 0.2
    d["synth_reverb_decay"] = 0.6
    d["note_length_percent"] = 90.0
    presets.append(create_preset("Fairy Tale", "Factory",
        "Once upon - twinkling melody spins enchantment", d))

    # 25. Apocalypse Now - Dark finale
    d = create_default_preset()
    d["straight_1_4"] = [100.0, 90.0, 95.0, 85.0]
    d["straight_1_8"] = [75.0, 65.0, 70.0, 60.0, 80.0, 68.0, 72.0, 62.0]
    d["strength_values"] = create_strength_pattern("driving")
    d["root_note"] = 36
    d["notes"] = [note_to_dict(Note(36, 100)), note_to_dict(Note(39, 65)), note_to_dict(Note(42, 60)),
                  note_to_dict(Note(48, 55)), note_to_dict(Note(51, 50))]
    d["synth_osc_d"] = 0.55
    d["synth_osc_v"] = 0.4
    d["synth_osc_volume"] = 0.72
    d["synth_sub_volume"] = 0.28
    d["synth_filter_cutoff"] = 1500.0
    d["synth_filter_resonance"] = 0.3
    d["synth_filter_env_amount"] = 1500.0
    d["synth_vol_attack"] = 5.0
    d["synth_vol_decay"] = 250.0
    d["synth_vol_sustain"] = 0.5
    d["synth_vol_release"] = 350.0
    d["note_length_percent"] = 80.0
    presets.append(create_preset("Apocalypse Now", "Factory",
        "End times - crushing weight of impending doom", d))

    # 26. Dream Sequence - Surreal state
    d = create_default_preset()
    d["straight_1_2"] = [100.0, 75.0]
    d["straight_1_4"] = [55.0, 40.0, 50.0, 35.0]
    d["dotted_1_4d"] = [45.0, 30.0, 38.0]
    d["strength_values"] = create_strength_pattern("ambient")
    d["root_note"] = 48
    d["notes"] = [note_to_dict(Note(48, 100)), note_to_dict(Note(52, 55)), note_to_dict(Note(55, 50)),
                  note_to_dict(Note(60, 45)), note_to_dict(Note(64, 40)), note_to_dict(Note(67, 35))]
    d["octave_randomization"] = create_octave_randomization(0.16, 0.25, 0.25, "Both")
    d["synth_osc_d"] = 0.2
    d["synth_osc_v"] = 0.75
    d["synth_osc_stereo_v_offset"] = 0.2
    d["synth_osc_volume"] = 0.52
    d["synth_filter_cutoff"] = 3000.0
    d["synth_filter_resonance"] = 0.1
    d["synth_filter_env_amount"] = 400.0
    d["synth_vol_attack"] = 200.0
    d["synth_vol_decay"] = 1500.0
    d["synth_vol_sustain"] = 0.6
    d["synth_vol_release"] = 2500.0
    d["synth_reverb_mix"] = 0.25
    d["synth_reverb_decay"] = 0.78
    d["synth_reverb_diffusion"] = 0.92
    d["note_length_percent"] = 200.0
    d["lfo1_rate"] = 0.06
    d["lfo1_waveform"] = 0
    d["lfo1_dest1"] = 11
    d["lfo1_amount1"] = 0.15
    presets.append(create_preset("Dream Sequence", "Factory",
        "Subconscious drift - hazy tones blur reality", d))

    # 27. Car Chase - High speed pursuit
    d = create_default_preset()
    d["straight_1_16"] = [100.0, 90.0, 95.0, 85.0, 100.0, 88.0, 92.0, 82.0, 98.0, 86.0, 94.0, 80.0, 100.0, 84.0, 90.0, 78.0]
    d["strength_values"] = create_strength_pattern("driving")
    d["root_note"] = 36
    d["notes"] = [note_to_dict(Note(36, 100)), note_to_dict(Note(41, 55)), note_to_dict(Note(43, 50)),
                  note_to_dict(Note(48, 45))]
    d["synth_osc_d"] = 0.52
    d["synth_osc_v"] = 0.42
    d["synth_osc_volume"] = 0.75
    d["synth_sub_volume"] = 0.25
    d["synth_filter_cutoff"] = 2000.0
    d["synth_filter_resonance"] = 0.25
    d["synth_filter_env_amount"] = 1800.0
    d["synth_vol_attack"] = 1.0
    d["synth_vol_decay"] = 100.0
    d["synth_vol_sustain"] = 0.4
    d["synth_vol_release"] = 150.0
    d["note_length_percent"] = 60.0
    presets.append(create_preset("Car Chase", "Factory",
        "Burning rubber - relentless 16ths drive the pursuit", d))

    # 28. Gentle Rain - Peaceful nature
    d = create_default_preset()
    d["straight_1_8"] = [100.0, 70.0, 80.0, 60.0, 90.0, 65.0, 75.0, 55.0]
    d["straight_1_16"] = [45.0, 30.0, 40.0, 25.0, 50.0, 35.0, 42.0, 28.0, 48.0, 32.0, 38.0, 26.0, 52.0, 38.0, 44.0, 30.0]
    d["strength_values"] = create_strength_pattern("ambient")
    d["root_note"] = 53
    d["notes"] = [note_to_dict(Note(53, 100)), note_to_dict(Note(57, 55)), note_to_dict(Note(60, 50)),
                  note_to_dict(Note(65, 45)), note_to_dict(Note(69, 40)), note_to_dict(Note(72, 35))]
    d["synth_osc_d"] = 0.25
    d["synth_osc_v"] = 0.7
    d["synth_osc_stereo_v_offset"] = 0.15
    d["synth_osc_volume"] = 0.55
    d["synth_filter_cutoff"] = 3500.0
    d["synth_filter_resonance"] = 0.08
    d["synth_filter_env_amount"] = 400.0
    d["synth_vol_attack"] = 30.0
    d["synth_vol_decay"] = 500.0
    d["synth_vol_sustain"] = 0.5
    d["synth_vol_release"] = 800.0
    d["synth_reverb_mix"] = 0.22
    d["synth_reverb_decay"] = 0.65
    d["note_length_percent"] = 100.0
    presets.append(create_preset("Gentle Rain", "Factory",
        "Soft droplets - peaceful patter soothes the soul", d))

    # 29. Tension Break - Release moment
    d = create_default_preset()
    d["straight_1_1"] = [100.0]
    d["straight_1_2"] = [75.0, 65.0]
    d["straight_1_4"] = [50.0, 40.0, 45.0, 35.0]
    d["strength_values"] = create_strength_pattern("sparse")
    d["root_note"] = 48
    d["notes"] = [note_to_dict(Note(48, 100)), note_to_dict(Note(52, 60)), note_to_dict(Note(55, 55)),
                  note_to_dict(Note(60, 50)), note_to_dict(Note(64, 45))]
    d["synth_osc_d"] = 0.22
    d["synth_osc_v"] = 0.72
    d["synth_osc_stereo_v_offset"] = 0.18
    d["synth_osc_volume"] = 0.58
    d["synth_filter_cutoff"] = 3200.0
    d["synth_filter_resonance"] = 0.1
    d["synth_filter_env_amount"] = 500.0
    d["synth_vol_attack"] = 150.0
    d["synth_vol_decay"] = 1200.0
    d["synth_vol_sustain"] = 0.6
    d["synth_vol_release"] = 2000.0
    d["synth_reverb_mix"] = 0.22
    d["synth_reverb_decay"] = 0.7
    d["synth_reverb_diffusion"] = 0.88
    d["note_length_percent"] = 180.0
    presets.append(create_preset("Tension Break", "Factory",
        "Sweet release - resolution brings peaceful calm", d))

    # 30. Orchestral Stab - Dramatic accent
    d = create_default_preset()
    d["straight_1_4"] = [100.0, 0.0, 95.0, 0.0]
    d["straight_1_8"] = [0.0, 0.0, 80.0, 0.0, 0.0, 0.0, 75.0, 0.0]
    d["strength_values"] = create_strength_pattern("sparse")
    d["root_note"] = 41
    d["notes"] = [note_to_dict(Note(41, 100)), note_to_dict(Note(45, 60)), note_to_dict(Note(48, 55)),
                  note_to_dict(Note(53, 50)), note_to_dict(Note(57, 45))]
    d["synth_osc_d"] = 0.45
    d["synth_osc_v"] = 0.5
    d["synth_osc_volume"] = 0.72
    d["synth_filter_cutoff"] = 3000.0
    d["synth_filter_resonance"] = 0.15
    d["synth_filter_env_amount"] = 1200.0
    d["synth_vol_attack"] = 2.0
    d["synth_vol_decay"] = 200.0
    d["synth_vol_sustain"] = 0.4
    d["synth_vol_release"] = 300.0
    d["synth_reverb_mix"] = 0.18
    d["synth_reverb_decay"] = 0.5
    d["note_length_percent"] = 60.0
    presets.append(create_preset("Orchestral Stab", "Factory",
        "Dramatic punch - powerful accents mark the moment", d))

    # 31. Time Lapse - Passing of time
    d = create_default_preset()
    d["straight_1_4"] = [100.0, 85.0, 90.0, 80.0]
    d["straight_1_8"] = [65.0, 50.0, 60.0, 45.0, 70.0, 55.0, 62.0, 48.0]
    d["straight_1_16"] = [40.0, 30.0, 35.0, 25.0, 45.0, 32.0, 38.0, 28.0, 42.0, 32.0, 36.0, 26.0, 48.0, 35.0, 40.0, 30.0]
    d["strength_values"] = create_strength_pattern("4_4_standard")
    d["root_note"] = 48
    d["notes"] = [note_to_dict(Note(48, 100)), note_to_dict(Note(52, 55)), note_to_dict(Note(55, 50)),
                  note_to_dict(Note(60, 45)), note_to_dict(Note(64, 40))]
    d["synth_osc_d"] = 0.35
    d["synth_osc_v"] = 0.6
    d["synth_osc_volume"] = 0.62
    d["synth_filter_cutoff"] = 3500.0
    d["synth_filter_resonance"] = 0.1
    d["synth_filter_env_amount"] = 600.0
    d["synth_vol_attack"] = 20.0
    d["synth_vol_decay"] = 400.0
    d["synth_vol_sustain"] = 0.5
    d["synth_vol_release"] = 600.0
    d["synth_reverb_mix"] = 0.18
    d["synth_reverb_decay"] = 0.55
    d["note_length_percent"] = 85.0
    presets.append(create_preset("Time Lapse", "Factory",
        "Hours pass - gentle progression marks the moments", d))

    # 32. Credits Roll - The end
    d = create_default_preset()
    d["straight_1_2"] = [100.0, 85.0]
    d["straight_1_4"] = [70.0, 55.0, 65.0, 50.0]
    d["straight_1_8"] = [45.0, 32.0, 40.0, 28.0, 48.0, 35.0, 42.0, 30.0]
    d["strength_values"] = create_strength_pattern("ambient")
    d["root_note"] = 48
    d["notes"] = [note_to_dict(Note(48, 100)), note_to_dict(Note(52, 60)), note_to_dict(Note(55, 55)),
                  note_to_dict(Note(60, 50)), note_to_dict(Note(64, 45)), note_to_dict(Note(67, 40))]
    d["synth_osc_d"] = 0.25
    d["synth_osc_v"] = 0.72
    d["synth_osc_stereo_v_offset"] = 0.18
    d["synth_osc_volume"] = 0.55
    d["synth_filter_cutoff"] = 3200.0
    d["synth_filter_resonance"] = 0.08
    d["synth_filter_env_amount"] = 400.0
    d["synth_vol_attack"] = 150.0
    d["synth_vol_decay"] = 1200.0
    d["synth_vol_sustain"] = 0.6
    d["synth_vol_release"] = 2000.0
    d["synth_reverb_mix"] = 0.22
    d["synth_reverb_decay"] = 0.7
    d["synth_reverb_diffusion"] = 0.88
    d["note_length_percent"] = 180.0
    presets.append(create_preset("Credits Roll", "Factory",
        "The end - reflective melody closes the story", d))

    return presets

def create_bank_h() -> List[Dict]:
    """Bank H: Bass & Rhythm - 32 presets"""
    presets = []

    # 1. Deep House Sub - Classic deep bass
    d = create_default_preset()
    d["straight_1_4"] = [100.0, 0.0, 85.0, 0.0]
    d["straight_1_8"] = [0.0, 70.0, 0.0, 65.0, 0.0, 75.0, 0.0, 60.0]
    d["strength_values"] = create_strength_pattern("4_4_standard")
    d["root_note"] = 36
    d["notes"] = [note_to_dict(Note(36, 100)), note_to_dict(Note(41, 55)), note_to_dict(Note(43, 50)),
                  note_to_dict(Note(48, 45))]
    d["synth_osc_d"] = 0.5
    d["synth_osc_v"] = 0.4
    d["synth_osc_volume"] = 0.72
    d["synth_sub_volume"] = 0.35
    d["synth_filter_cutoff"] = 800.0
    d["synth_filter_resonance"] = 0.2
    d["synth_filter_env_amount"] = 1200.0
    d["synth_vol_attack"] = 2.0
    d["synth_vol_decay"] = 200.0
    d["synth_vol_sustain"] = 0.5
    d["synth_vol_release"] = 250.0
    d["note_length_percent"] = 75.0
    presets.append(create_preset("Deep House Sub", "Factory",
        "Underground foundation - warm sub bass drives the groove", d))

    # 2. DnB Reese - Classic jungle bass
    d = create_default_preset()
    d["straight_1_8"] = [100.0, 0.0, 0.0, 85.0, 0.0, 90.0, 0.0, 0.0]
    d["straight_1_16"] = [0.0, 75.0, 70.0, 0.0, 80.0, 0.0, 65.0, 72.0, 0.0, 78.0, 68.0, 0.0, 82.0, 0.0, 60.0, 74.0]
    d["strength_values"] = create_strength_pattern("driving")
    d["root_note"] = 36
    d["notes"] = [note_to_dict(Note(36, 100)), note_to_dict(Note(39, 60)), note_to_dict(Note(43, 55)),
                  note_to_dict(Note(48, 50))]
    d["synth_osc_d"] = 0.55
    d["synth_osc_v"] = 0.45
    d["synth_osc_stereo_v_offset"] = 0.1
    d["synth_osc_volume"] = 0.75
    d["synth_sub_volume"] = 0.3
    d["synth_filter_cutoff"] = 1200.0
    d["synth_filter_resonance"] = 0.35
    d["synth_filter_env_amount"] = 1800.0
    d["synth_vol_attack"] = 1.0
    d["synth_vol_decay"] = 150.0
    d["synth_vol_sustain"] = 0.4
    d["synth_vol_release"] = 200.0
    d["note_length_percent"] = 60.0
    d["lfo1_rate"] = 2.5
    d["lfo1_waveform"] = 0
    d["lfo1_dest1"] = 11
    d["lfo1_amount1"] = 0.15
    presets.append(create_preset("DnB Reese", "Factory",
        "Jungle pressure - detuned bass warps and growls", d))

    # 3. Dubstep Wobble - Classic LFO bass
    d = create_default_preset()
    d["straight_1_4"] = [100.0, 95.0, 100.0, 90.0]
    d["straight_1_8"] = [85.0, 80.0, 88.0, 75.0, 82.0, 78.0, 86.0, 72.0]
    d["strength_values"] = create_strength_pattern("driving")
    d["root_note"] = 36
    d["notes"] = [note_to_dict(Note(36, 100)), note_to_dict(Note(41, 55)), note_to_dict(Note(43, 50)),
                  note_to_dict(Note(48, 45))]
    d["synth_osc_d"] = 0.6
    d["synth_osc_v"] = 0.35
    d["synth_osc_volume"] = 0.78
    d["synth_sub_volume"] = 0.25
    d["synth_filter_cutoff"] = 600.0
    d["synth_filter_resonance"] = 0.55
    d["synth_filter_env_amount"] = 2500.0
    d["synth_vol_attack"] = 1.0
    d["synth_vol_decay"] = 200.0
    d["synth_vol_sustain"] = 0.55
    d["synth_vol_release"] = 250.0
    d["note_length_percent"] = 85.0
    d["lfo1_tempo_sync"] = True
    d["lfo1_sync_division"] = 3
    d["lfo1_waveform"] = 2
    d["lfo1_dest1"] = 12
    d["lfo1_amount1"] = 0.45
    presets.append(create_preset("Dubstep Wobble", "Factory",
        "Filter mayhem - aggressive LFO tears through the mix", d))

    # 4. UK Garage Bass - 2-step foundation
    d = create_default_preset()
    d["straight_1_8"] = [100.0, 0.0, 0.0, 80.0, 0.0, 85.0, 0.0, 0.0]
    d["straight_1_16"] = [0.0, 0.0, 70.0, 0.0, 75.0, 0.0, 0.0, 65.0, 0.0, 0.0, 72.0, 0.0, 78.0, 0.0, 0.0, 60.0]
    d["strength_values"] = create_strength_pattern("shuffle")
    d["root_note"] = 36
    d["notes"] = [note_to_dict(Note(36, 100)), note_to_dict(Note(41, 55)), note_to_dict(Note(43, 50)),
                  note_to_dict(Note(48, 45))]
    d["synth_osc_d"] = 0.48
    d["synth_osc_v"] = 0.5
    d["synth_osc_volume"] = 0.7
    d["synth_sub_volume"] = 0.32
    d["synth_filter_cutoff"] = 1000.0
    d["synth_filter_resonance"] = 0.25
    d["synth_filter_env_amount"] = 1500.0
    d["synth_vol_attack"] = 1.0
    d["synth_vol_decay"] = 120.0
    d["synth_vol_sustain"] = 0.35
    d["synth_vol_release"] = 180.0
    d["swing_amount"] = 58.0
    d["note_length_percent"] = 55.0
    presets.append(create_preset("UK Garage Bass", "Factory",
        "2-step bounce - skippy bass grooves with swing", d))

    # 5. Breakbeat Stab - Jungle stab bass
    d = create_default_preset()
    d["straight_1_8"] = [100.0, 0.0, 90.0, 0.0, 0.0, 85.0, 0.0, 80.0]
    d["straight_1_16"] = [0.0, 70.0, 0.0, 0.0, 75.0, 0.0, 65.0, 0.0, 72.0, 0.0, 0.0, 68.0, 0.0, 74.0, 0.0, 0.0]
    d["strength_values"] = create_strength_pattern("funk")
    d["root_note"] = 36
    d["notes"] = [note_to_dict(Note(36, 100)), note_to_dict(Note(41, 58)), note_to_dict(Note(43, 52)),
                  note_to_dict(Note(48, 46))]
    d["synth_osc_d"] = 0.52
    d["synth_osc_v"] = 0.45
    d["synth_osc_volume"] = 0.72
    d["synth_filter_cutoff"] = 1800.0
    d["synth_filter_resonance"] = 0.3
    d["synth_filter_env_amount"] = 2200.0
    d["synth_vol_attack"] = 0.5
    d["synth_vol_decay"] = 100.0
    d["synth_vol_sustain"] = 0.3
    d["synth_vol_release"] = 150.0
    d["note_length_percent"] = 45.0
    presets.append(create_preset("Breakbeat Stab", "Factory",
        "Chopped funk - stabby bass cuts through breaks", d))

    # 6. Hip Hop Boom - 808 style
    d = create_default_preset()
    d["straight_1_4"] = [100.0, 0.0, 0.0, 80.0]
    d["straight_1_8"] = [0.0, 0.0, 70.0, 0.0, 0.0, 75.0, 0.0, 0.0]
    d["strength_values"] = create_strength_pattern("backbeat")
    d["root_note"] = 36
    d["notes"] = [note_to_dict(Note(36, 100)), note_to_dict(Note(41, 55)), note_to_dict(Note(43, 50)),
                  note_to_dict(Note(48, 45))]
    d["synth_osc_d"] = 0.4
    d["synth_osc_v"] = 0.55
    d["synth_osc_volume"] = 0.68
    d["synth_sub_volume"] = 0.4
    d["synth_filter_cutoff"] = 500.0
    d["synth_filter_resonance"] = 0.15
    d["synth_filter_env_amount"] = 800.0
    d["synth_vol_attack"] = 1.0
    d["synth_vol_decay"] = 350.0
    d["synth_vol_sustain"] = 0.4
    d["synth_vol_release"] = 500.0
    d["note_length_percent"] = 100.0
    presets.append(create_preset("Hip Hop Boom", "Factory",
        "808 weight - long decay sub bass hits hard", d))

    # 7. Trap Sub Drop - Modern trap bass
    d = create_default_preset()
    d["straight_1_4"] = [100.0, 0.0, 90.0, 0.0]
    d["straight_1_8"] = [0.0, 0.0, 0.0, 75.0, 0.0, 0.0, 80.0, 0.0]
    d["triplet_1_8t"] = [70.0, 0.0, 0.0, 65.0, 0.0, 0.0, 72.0, 0.0, 0.0, 68.0, 0.0, 0.0]
    d["strength_values"] = create_strength_pattern("sparse")
    d["root_note"] = 36
    d["notes"] = [note_to_dict(Note(36, 100)), note_to_dict(Note(38, 60)), note_to_dict(Note(41, 55)),
                  note_to_dict(Note(43, 50))]
    d["synth_osc_d"] = 0.35
    d["synth_osc_v"] = 0.6
    d["synth_osc_volume"] = 0.65
    d["synth_sub_volume"] = 0.45
    d["synth_filter_cutoff"] = 400.0
    d["synth_filter_resonance"] = 0.1
    d["synth_filter_env_amount"] = 600.0
    d["synth_vol_attack"] = 0.5
    d["synth_vol_decay"] = 500.0
    d["synth_vol_sustain"] = 0.35
    d["synth_vol_release"] = 700.0
    d["note_length_percent"] = 120.0
    presets.append(create_preset("Trap Sub Drop", "Factory",
        "Modern weight - deep 808 slides and drops", d))

    # 8. Techno Kick Bass - Four to the floor
    d = create_default_preset()
    d["straight_1_4"] = [100.0, 100.0, 100.0, 100.0]
    d["straight_1_8"] = [0.0, 70.0, 0.0, 65.0, 0.0, 72.0, 0.0, 68.0]
    d["strength_values"] = create_strength_pattern("4_4_standard")
    d["root_note"] = 36
    d["notes"] = [note_to_dict(Note(36, 100)), note_to_dict(Note(41, 50)), note_to_dict(Note(43, 45))]
    d["synth_osc_d"] = 0.55
    d["synth_osc_v"] = 0.4
    d["synth_osc_volume"] = 0.75
    d["synth_sub_volume"] = 0.3
    d["synth_filter_cutoff"] = 600.0
    d["synth_filter_resonance"] = 0.25
    d["synth_filter_env_amount"] = 1500.0
    d["synth_vol_attack"] = 0.5
    d["synth_vol_decay"] = 80.0
    d["synth_vol_sustain"] = 0.25
    d["synth_vol_release"] = 120.0
    d["note_length_percent"] = 50.0
    presets.append(create_preset("Techno Kick Bass", "Factory",
        "Industrial pulse - punchy kick bass drives the floor", d))

    # 9. Rolling DnB - Liquid bassline
    d = create_default_preset()
    d["straight_1_16"] = [100.0, 75.0, 85.0, 70.0, 90.0, 72.0, 80.0, 65.0, 95.0, 78.0, 82.0, 68.0, 88.0, 74.0, 78.0, 62.0]
    d["strength_values"] = create_strength_pattern("driving")
    d["root_note"] = 36
    d["notes"] = [note_to_dict(Note(36, 100)), note_to_dict(Note(41, 58)), note_to_dict(Note(43, 52)),
                  note_to_dict(Note(48, 46)), note_to_dict(Note(53, 40))]
    d["synth_osc_d"] = 0.5
    d["synth_osc_v"] = 0.48
    d["synth_osc_volume"] = 0.72
    d["synth_sub_volume"] = 0.28
    d["synth_filter_cutoff"] = 1500.0
    d["synth_filter_resonance"] = 0.28
    d["synth_filter_env_amount"] = 1800.0
    d["synth_vol_attack"] = 1.0
    d["synth_vol_decay"] = 120.0
    d["synth_vol_sustain"] = 0.4
    d["synth_vol_release"] = 180.0
    d["note_length_percent"] = 55.0
    presets.append(create_preset("Rolling DnB", "Factory",
        "Liquid flow - smooth 16th basslines roll endlessly", d))

    # 10. Acid Squelch - 303 tribute
    d = create_default_preset()
    d["straight_1_16"] = [100.0, 80.0, 85.0, 75.0, 95.0, 78.0, 82.0, 70.0, 90.0, 76.0, 84.0, 72.0, 88.0, 74.0, 80.0, 68.0]
    d["strength_values"] = create_strength_pattern("driving")
    d["root_note"] = 36
    d["notes"] = [note_to_dict(Note(36, 100)), note_to_dict(Note(39, 60)), note_to_dict(Note(41, 55)),
                  note_to_dict(Note(43, 50)), note_to_dict(Note(48, 45))]
    d["octave_randomization"] = create_octave_randomization(0.12, 0.2, 0.16, "Up")
    d["synth_osc_d"] = 0.65
    d["synth_osc_v"] = 0.32
    d["synth_osc_volume"] = 0.75
    d["synth_filter_cutoff"] = 700.0
    d["synth_filter_resonance"] = 0.68
    d["synth_filter_env_amount"] = 4000.0
    d["synth_vol_attack"] = 0.5
    d["synth_vol_decay"] = 150.0
    d["synth_vol_sustain"] = 0.3
    d["synth_vol_release"] = 180.0
    d["note_length_percent"] = 55.0
    presets.append(create_preset("Acid Squelch", "Factory",
        "303 worship - screaming resonance tears through acid", d))

    # 11. Funk Slap Bass - Classic funk
    d = create_default_preset()
    d["straight_1_16"] = [100.0, 0.0, 75.0, 70.0, 0.0, 85.0, 0.0, 65.0, 90.0, 0.0, 72.0, 68.0, 0.0, 80.0, 0.0, 62.0]
    d["strength_values"] = create_strength_pattern("funk")
    d["root_note"] = 36
    d["notes"] = [note_to_dict(Note(36, 100)), note_to_dict(Note(41, 58)), note_to_dict(Note(43, 52)),
                  note_to_dict(Note(48, 46)), note_to_dict(Note(53, 40))]
    d["synth_osc_d"] = 0.45
    d["synth_osc_v"] = 0.52
    d["synth_osc_volume"] = 0.7
    d["synth_filter_cutoff"] = 2500.0
    d["synth_filter_resonance"] = 0.2
    d["synth_filter_env_amount"] = 2000.0
    d["synth_vol_attack"] = 0.5
    d["synth_vol_decay"] = 80.0
    d["synth_vol_sustain"] = 0.25
    d["synth_vol_release"] = 120.0
    d["swing_amount"] = 55.0
    d["note_length_percent"] = 40.0
    presets.append(create_preset("Funk Slap Bass", "Factory",
        "Popping groove - tight slaps drive the funk", d))

    # 12. Minimal Techno - Hypnotic pulse
    d = create_default_preset()
    d["straight_1_8"] = [100.0, 85.0, 90.0, 80.0, 95.0, 82.0, 88.0, 78.0]
    d["strength_values"] = create_strength_pattern("driving")
    d["root_note"] = 36
    d["notes"] = [note_to_dict(Note(36, 100)), note_to_dict(Note(41, 50)), note_to_dict(Note(43, 45))]
    d["synth_osc_d"] = 0.52
    d["synth_osc_v"] = 0.42
    d["synth_osc_volume"] = 0.72
    d["synth_sub_volume"] = 0.3
    d["synth_filter_cutoff"] = 900.0
    d["synth_filter_resonance"] = 0.3
    d["synth_filter_env_amount"] = 1200.0
    d["synth_vol_attack"] = 1.0
    d["synth_vol_decay"] = 150.0
    d["synth_vol_sustain"] = 0.4
    d["synth_vol_release"] = 200.0
    d["note_length_percent"] = 65.0
    d["lfo1_tempo_sync"] = True
    d["lfo1_sync_division"] = 4
    d["lfo1_waveform"] = 0
    d["lfo1_dest1"] = 12
    d["lfo1_amount1"] = 0.12
    presets.append(create_preset("Minimal Techno", "Factory",
        "Hypnotic loop - repetitive bass mesmerizes the floor", d))

    # 13. Garage Bassline - Speed garage
    d = create_default_preset()
    d["straight_1_8"] = [100.0, 0.0, 80.0, 0.0, 90.0, 0.0, 75.0, 0.0]
    d["straight_1_16"] = [0.0, 65.0, 0.0, 60.0, 0.0, 70.0, 0.0, 55.0, 0.0, 68.0, 0.0, 62.0, 0.0, 72.0, 0.0, 58.0]
    d["strength_values"] = create_strength_pattern("shuffle")
    d["root_note"] = 36
    d["notes"] = [note_to_dict(Note(36, 100)), note_to_dict(Note(41, 55)), note_to_dict(Note(43, 50)),
                  note_to_dict(Note(48, 45))]
    d["synth_osc_d"] = 0.48
    d["synth_osc_v"] = 0.48
    d["synth_osc_volume"] = 0.7
    d["synth_sub_volume"] = 0.3
    d["synth_filter_cutoff"] = 1200.0
    d["synth_filter_resonance"] = 0.25
    d["synth_filter_env_amount"] = 1600.0
    d["synth_vol_attack"] = 1.0
    d["synth_vol_decay"] = 130.0
    d["synth_vol_sustain"] = 0.38
    d["synth_vol_release"] = 180.0
    d["swing_amount"] = 56.0
    d["note_length_percent"] = 60.0
    presets.append(create_preset("Garage Bassline", "Factory",
        "Speed bounce - energetic bass pushes the tempo", d))

    # 14. Dub Reggae - Roots bass
    d = create_default_preset()
    d["straight_1_4"] = [100.0, 0.0, 80.0, 0.0]
    d["straight_1_8"] = [0.0, 70.0, 0.0, 0.0, 0.0, 75.0, 0.0, 0.0]
    d["strength_values"] = create_strength_pattern("reggae")
    d["root_note"] = 36
    d["notes"] = [note_to_dict(Note(36, 100)), note_to_dict(Note(41, 55)), note_to_dict(Note(43, 50)),
                  note_to_dict(Note(48, 45))]
    d["synth_osc_d"] = 0.35
    d["synth_osc_v"] = 0.6
    d["synth_osc_volume"] = 0.65
    d["synth_sub_volume"] = 0.38
    d["synth_filter_cutoff"] = 600.0
    d["synth_filter_resonance"] = 0.12
    d["synth_filter_env_amount"] = 500.0
    d["synth_vol_attack"] = 5.0
    d["synth_vol_decay"] = 300.0
    d["synth_vol_sustain"] = 0.5
    d["synth_vol_release"] = 400.0
    d["synth_reverb_mix"] = 0.18
    d["synth_reverb_decay"] = 0.55
    d["note_length_percent"] = 85.0
    presets.append(create_preset("Dub Reggae", "Factory",
        "Roots pressure - deep bass shakes the speaker box", d))

    # 15. Neuro Bass - Heavy DnB
    d = create_default_preset()
    d["straight_1_8"] = [100.0, 90.0, 0.0, 85.0, 95.0, 0.0, 88.0, 80.0]
    d["straight_1_16"] = [75.0, 70.0, 0.0, 65.0, 78.0, 0.0, 68.0, 72.0, 0.0, 74.0, 66.0, 0.0, 76.0, 0.0, 70.0, 62.0]
    d["strength_values"] = create_strength_pattern("driving")
    d["root_note"] = 36
    d["notes"] = [note_to_dict(Note(36, 100)), note_to_dict(Note(39, 60)), note_to_dict(Note(42, 55)),
                  note_to_dict(Note(48, 50))]
    d["synth_osc_d"] = 0.58
    d["synth_osc_v"] = 0.38
    d["synth_osc_stereo_v_offset"] = 0.12
    d["synth_osc_volume"] = 0.78
    d["synth_sub_volume"] = 0.25
    d["synth_filter_cutoff"] = 1000.0
    d["synth_filter_resonance"] = 0.45
    d["synth_filter_env_amount"] = 2500.0
    d["synth_vol_attack"] = 0.5
    d["synth_vol_decay"] = 120.0
    d["synth_vol_sustain"] = 0.35
    d["synth_vol_release"] = 180.0
    d["note_length_percent"] = 55.0
    d["lfo1_rate"] = 4.0
    d["lfo1_waveform"] = 4
    d["lfo1_dest1"] = 12
    d["lfo1_amount1"] = 0.25
    presets.append(create_preset("Neuro Bass", "Factory",
        "Brain damage - aggressive modulated bass attacks", d))

    # 16. House Organ - Classic house bass
    d = create_default_preset()
    d["straight_1_8"] = [100.0, 80.0, 85.0, 75.0, 95.0, 78.0, 82.0, 72.0]
    d["strength_values"] = create_strength_pattern("4_4_standard")
    d["root_note"] = 36
    d["notes"] = [note_to_dict(Note(36, 100)), note_to_dict(Note(41, 55)), note_to_dict(Note(43, 50)),
                  note_to_dict(Note(48, 45)), note_to_dict(Note(53, 40))]
    d["synth_osc_d"] = 0.42
    d["synth_osc_v"] = 0.55
    d["synth_osc_volume"] = 0.68
    d["synth_filter_cutoff"] = 1800.0
    d["synth_filter_resonance"] = 0.18
    d["synth_filter_env_amount"] = 1200.0
    d["synth_vol_attack"] = 2.0
    d["synth_vol_decay"] = 200.0
    d["synth_vol_sustain"] = 0.48
    d["synth_vol_release"] = 280.0
    d["note_length_percent"] = 70.0
    presets.append(create_preset("House Organ", "Factory",
        "Chicago soul - warm organ bass carries the groove", d))

    # 17. Halftime DnB - Modern halftime
    d = create_default_preset()
    d["straight_1_4"] = [100.0, 0.0, 90.0, 0.0]
    d["straight_1_8"] = [0.0, 80.0, 0.0, 75.0, 0.0, 85.0, 0.0, 70.0]
    d["straight_1_16"] = [65.0, 0.0, 60.0, 0.0, 68.0, 0.0, 55.0, 0.0, 70.0, 0.0, 62.0, 0.0, 72.0, 0.0, 58.0, 0.0]
    d["strength_values"] = create_strength_pattern("backbeat")
    d["root_note"] = 36
    d["notes"] = [note_to_dict(Note(36, 100)), note_to_dict(Note(39, 58)), note_to_dict(Note(43, 52)),
                  note_to_dict(Note(48, 46))]
    d["synth_osc_d"] = 0.55
    d["synth_osc_v"] = 0.42
    d["synth_osc_volume"] = 0.75
    d["synth_sub_volume"] = 0.3
    d["synth_filter_cutoff"] = 800.0
    d["synth_filter_resonance"] = 0.35
    d["synth_filter_env_amount"] = 1800.0
    d["synth_vol_attack"] = 1.0
    d["synth_vol_decay"] = 180.0
    d["synth_vol_sustain"] = 0.42
    d["synth_vol_release"] = 250.0
    d["note_length_percent"] = 70.0
    presets.append(create_preset("Halftime DnB", "Factory",
        "Slow heavy - weighted bass at half tempo", d))

    # 18. Electro Zap - Electro bass
    d = create_default_preset()
    d["straight_1_8"] = [100.0, 0.0, 85.0, 0.0, 90.0, 0.0, 80.0, 0.0]
    d["straight_1_16"] = [0.0, 70.0, 0.0, 65.0, 0.0, 75.0, 0.0, 60.0, 0.0, 72.0, 0.0, 68.0, 0.0, 78.0, 0.0, 62.0]
    d["strength_values"] = create_strength_pattern("driving")
    d["root_note"] = 36
    d["notes"] = [note_to_dict(Note(36, 100)), note_to_dict(Note(41, 55)), note_to_dict(Note(43, 50)),
                  note_to_dict(Note(48, 45))]
    d["synth_osc_d"] = 0.6
    d["synth_osc_v"] = 0.35
    d["synth_osc_volume"] = 0.75
    d["synth_filter_cutoff"] = 1200.0
    d["synth_filter_resonance"] = 0.4
    d["synth_filter_env_amount"] = 2800.0
    d["synth_vol_attack"] = 0.5
    d["synth_vol_decay"] = 100.0
    d["synth_vol_sustain"] = 0.28
    d["synth_vol_release"] = 150.0
    d["note_length_percent"] = 45.0
    presets.append(create_preset("Electro Zap", "Factory",
        "Robot funk - zappy bass with sharp envelope", d))

    # 19. Afrobeat Pulse - African rhythm bass
    d = create_default_preset()
    d["straight_1_8"] = [100.0, 75.0, 85.0, 70.0, 90.0, 72.0, 80.0, 65.0]
    d["triplet_1_8t"] = [60.0, 50.0, 55.0, 65.0, 48.0, 52.0, 58.0, 45.0, 50.0, 62.0, 52.0, 55.0]
    d["strength_values"] = create_strength_pattern("african")
    d["root_note"] = 36
    d["notes"] = [note_to_dict(Note(36, 100)), note_to_dict(Note(41, 58)), note_to_dict(Note(43, 52)),
                  note_to_dict(Note(48, 46)), note_to_dict(Note(53, 40))]
    d["synth_osc_d"] = 0.45
    d["synth_osc_v"] = 0.52
    d["synth_osc_volume"] = 0.68
    d["synth_sub_volume"] = 0.25
    d["synth_filter_cutoff"] = 1400.0
    d["synth_filter_resonance"] = 0.2
    d["synth_filter_env_amount"] = 1000.0
    d["synth_vol_attack"] = 2.0
    d["synth_vol_decay"] = 180.0
    d["synth_vol_sustain"] = 0.45
    d["synth_vol_release"] = 250.0
    d["note_length_percent"] = 65.0
    presets.append(create_preset("Afrobeat Pulse", "Factory",
        "Lagos groove - polyrhythmic bass drives the beat", d))

    # 20. Synthwave Bass - Retro 80s
    d = create_default_preset()
    d["straight_1_8"] = [100.0, 85.0, 90.0, 80.0, 95.0, 82.0, 88.0, 78.0]
    d["strength_values"] = create_strength_pattern("4_4_standard")
    d["root_note"] = 36
    d["notes"] = [note_to_dict(Note(36, 100)), note_to_dict(Note(41, 55)), note_to_dict(Note(43, 50)),
                  note_to_dict(Note(48, 45)), note_to_dict(Note(53, 40))]
    d["synth_osc_d"] = 0.5
    d["synth_osc_v"] = 0.45
    d["synth_osc_stereo_v_offset"] = 0.08
    d["synth_osc_volume"] = 0.72
    d["synth_filter_cutoff"] = 1600.0
    d["synth_filter_resonance"] = 0.22
    d["synth_filter_env_amount"] = 1400.0
    d["synth_vol_attack"] = 1.0
    d["synth_vol_decay"] = 200.0
    d["synth_vol_sustain"] = 0.45
    d["synth_vol_release"] = 280.0
    d["synth_reverb_mix"] = 0.15
    d["synth_reverb_decay"] = 0.45
    d["note_length_percent"] = 70.0
    presets.append(create_preset("Synthwave Bass", "Factory",
        "Neon nights - retro analog bass for outrun vibes", d))

    # 21. Grime Bass - UK underground
    d = create_default_preset()
    d["straight_1_8"] = [100.0, 0.0, 0.0, 85.0, 0.0, 90.0, 0.0, 0.0]
    d["straight_1_16"] = [0.0, 0.0, 75.0, 0.0, 70.0, 0.0, 0.0, 65.0, 0.0, 0.0, 78.0, 0.0, 72.0, 0.0, 0.0, 60.0]
    d["strength_values"] = create_strength_pattern("driving")
    d["root_note"] = 36
    d["notes"] = [note_to_dict(Note(36, 100)), note_to_dict(Note(39, 58)), note_to_dict(Note(43, 52)),
                  note_to_dict(Note(48, 46))]
    d["synth_osc_d"] = 0.58
    d["synth_osc_v"] = 0.38
    d["synth_osc_volume"] = 0.75
    d["synth_sub_volume"] = 0.3
    d["synth_filter_cutoff"] = 900.0
    d["synth_filter_resonance"] = 0.38
    d["synth_filter_env_amount"] = 2000.0
    d["synth_vol_attack"] = 0.5
    d["synth_vol_decay"] = 120.0
    d["synth_vol_sustain"] = 0.35
    d["synth_vol_release"] = 180.0
    d["note_length_percent"] = 55.0
    presets.append(create_preset("Grime Bass", "Factory",
        "East London - aggressive skippy bass patterns", d))

    # 22. Latin Tumbao - Salsa bass
    d = create_default_preset()
    d["straight_1_8"] = [100.0, 0.0, 75.0, 80.0, 0.0, 85.0, 0.0, 70.0]
    d["strength_values"] = create_strength_pattern("latin")
    d["root_note"] = 36
    d["notes"] = [note_to_dict(Note(36, 100)), note_to_dict(Note(41, 58)), note_to_dict(Note(43, 52)),
                  note_to_dict(Note(48, 46)), note_to_dict(Note(53, 40))]
    d["synth_osc_d"] = 0.42
    d["synth_osc_v"] = 0.55
    d["synth_osc_volume"] = 0.68
    d["synth_filter_cutoff"] = 2000.0
    d["synth_filter_resonance"] = 0.15
    d["synth_filter_env_amount"] = 1000.0
    d["synth_vol_attack"] = 2.0
    d["synth_vol_decay"] = 150.0
    d["synth_vol_sustain"] = 0.4
    d["synth_vol_release"] = 200.0
    d["swing_amount"] = 54.0
    d["note_length_percent"] = 60.0
    presets.append(create_preset("Latin Tumbao", "Factory",
        "Havana rhythm - syncopated bass drives the clave", d))

    # 23. Industrial Grind - Harsh bass
    d = create_default_preset()
    d["straight_1_8"] = [100.0, 95.0, 90.0, 85.0, 100.0, 92.0, 88.0, 82.0]
    d["straight_1_16"] = [75.0, 70.0, 72.0, 65.0, 78.0, 68.0, 74.0, 62.0, 76.0, 72.0, 70.0, 66.0, 80.0, 74.0, 76.0, 60.0]
    d["strength_values"] = create_strength_pattern("driving")
    d["root_note"] = 36
    d["notes"] = [note_to_dict(Note(36, 100)), note_to_dict(Note(39, 60)), note_to_dict(Note(42, 55)),
                  note_to_dict(Note(48, 50))]
    d["synth_osc_d"] = 0.62
    d["synth_osc_v"] = 0.32
    d["synth_osc_volume"] = 0.78
    d["synth_filter_cutoff"] = 1400.0
    d["synth_filter_resonance"] = 0.42
    d["synth_filter_env_amount"] = 2200.0
    d["synth_filter_drive"] = 1.8
    d["synth_vol_attack"] = 0.5
    d["synth_vol_decay"] = 100.0
    d["synth_vol_sustain"] = 0.35
    d["synth_vol_release"] = 150.0
    d["note_length_percent"] = 55.0
    presets.append(create_preset("Industrial Grind", "Factory",
        "Machine noise - distorted bass grinds relentlessly", d))

    # 24. Jazz Walking - Upright style
    d = create_default_preset()
    d["straight_1_4"] = [100.0, 90.0, 95.0, 85.0]
    d["triplet_1_4t"] = [70.0, 60.0, 65.0, 75.0, 62.0, 68.0]
    d["strength_values"] = create_strength_pattern("jazz")
    d["root_note"] = 36
    d["notes"] = [note_to_dict(Note(36, 100)), note_to_dict(Note(41, 60)), note_to_dict(Note(43, 55)),
                  note_to_dict(Note(48, 50)), note_to_dict(Note(53, 45)), note_to_dict(Note(55, 40))]
    d["synth_osc_d"] = 0.35
    d["synth_osc_v"] = 0.62
    d["synth_osc_volume"] = 0.62
    d["synth_filter_cutoff"] = 1200.0
    d["synth_filter_resonance"] = 0.1
    d["synth_filter_env_amount"] = 600.0
    d["synth_vol_attack"] = 5.0
    d["synth_vol_decay"] = 250.0
    d["synth_vol_sustain"] = 0.5
    d["synth_vol_release"] = 350.0
    d["swing_amount"] = 58.0
    d["note_length_percent"] = 80.0
    presets.append(create_preset("Jazz Walking", "Factory",
        "Acoustic feel - walking bass swings through changes", d))

    # 25. Brostep Growl - Heavy drops
    d = create_default_preset()
    d["straight_1_4"] = [100.0, 0.0, 95.0, 0.0]
    d["straight_1_8"] = [90.0, 85.0, 0.0, 80.0, 88.0, 82.0, 0.0, 75.0]
    d["strength_values"] = create_strength_pattern("driving")
    d["root_note"] = 36
    d["notes"] = [note_to_dict(Note(36, 100)), note_to_dict(Note(39, 58)), note_to_dict(Note(43, 52)),
                  note_to_dict(Note(48, 46))]
    d["synth_osc_d"] = 0.62
    d["synth_osc_v"] = 0.32
    d["synth_osc_stereo_v_offset"] = 0.15
    d["synth_osc_volume"] = 0.8
    d["synth_sub_volume"] = 0.22
    d["synth_filter_cutoff"] = 500.0
    d["synth_filter_resonance"] = 0.58
    d["synth_filter_env_amount"] = 3000.0
    d["synth_vol_attack"] = 0.5
    d["synth_vol_decay"] = 150.0
    d["synth_vol_sustain"] = 0.45
    d["synth_vol_release"] = 200.0
    d["note_length_percent"] = 75.0
    d["lfo1_tempo_sync"] = True
    d["lfo1_sync_division"] = 2
    d["lfo1_waveform"] = 4
    d["lfo1_dest1"] = 12
    d["lfo1_amount1"] = 0.5
    presets.append(create_preset("Brostep Growl", "Factory",
        "Filthy drops - aggressive modulated bass destroys", d))

    # 26. Disco Octave - Classic disco
    d = create_default_preset()
    d["straight_1_8"] = [100.0, 85.0, 90.0, 80.0, 95.0, 82.0, 88.0, 78.0]
    d["strength_values"] = create_strength_pattern("4_4_standard")
    d["root_note"] = 36
    d["notes"] = [note_to_dict(Note(36, 100)), note_to_dict(Note(41, 55)), note_to_dict(Note(43, 50)),
                  note_to_dict(Note(48, 60)), note_to_dict(Note(53, 45))]
    d["octave_randomization"] = create_octave_randomization(0.16, 0.27, 0.2, "Up")
    d["synth_osc_d"] = 0.45
    d["synth_osc_v"] = 0.52
    d["synth_osc_volume"] = 0.7
    d["synth_filter_cutoff"] = 2200.0
    d["synth_filter_resonance"] = 0.18
    d["synth_filter_env_amount"] = 1200.0
    d["synth_vol_attack"] = 1.0
    d["synth_vol_decay"] = 150.0
    d["synth_vol_sustain"] = 0.42
    d["synth_vol_release"] = 200.0
    d["note_length_percent"] = 60.0
    presets.append(create_preset("Disco Octave", "Factory",
        "Studio 54 - bouncy octave bass drives the dance", d))

    # 27. Footwork Juke - Chicago style
    d = create_default_preset()
    d["straight_1_8"] = [100.0, 0.0, 0.0, 85.0, 0.0, 0.0, 90.0, 0.0]
    d["straight_1_16"] = [0.0, 70.0, 0.0, 0.0, 75.0, 0.0, 0.0, 65.0, 0.0, 72.0, 0.0, 0.0, 78.0, 0.0, 0.0, 60.0]
    d["triplet_1_16t"] = [55.0, 0.0, 50.0, 0.0, 58.0, 0.0, 52.0, 0.0, 48.0, 0.0, 60.0, 0.0,
                          54.0, 0.0, 46.0, 0.0, 56.0, 0.0, 50.0, 0.0, 44.0, 0.0, 58.0, 0.0]
    d["strength_values"] = create_strength_pattern("driving")
    d["root_note"] = 36
    d["notes"] = [note_to_dict(Note(36, 100)), note_to_dict(Note(41, 55)), note_to_dict(Note(43, 50)),
                  note_to_dict(Note(48, 45))]
    d["synth_osc_d"] = 0.52
    d["synth_osc_v"] = 0.45
    d["synth_osc_volume"] = 0.72
    d["synth_sub_volume"] = 0.28
    d["synth_filter_cutoff"] = 1000.0
    d["synth_filter_resonance"] = 0.3
    d["synth_filter_env_amount"] = 1600.0
    d["synth_vol_attack"] = 0.5
    d["synth_vol_decay"] = 100.0
    d["synth_vol_sustain"] = 0.32
    d["synth_vol_release"] = 150.0
    d["note_length_percent"] = 45.0
    presets.append(create_preset("Footwork Juke", "Factory",
        "Chi-town heat - rapid bass for 160 BPM dance", d))

    # 28. Ambient Sub - Atmospheric bass
    d = create_default_preset()
    d["straight_1_2"] = [100.0, 80.0]
    d["straight_1_4"] = [60.0, 45.0, 55.0, 40.0]
    d["strength_values"] = create_strength_pattern("ambient")
    d["root_note"] = 36
    d["notes"] = [note_to_dict(Note(36, 100)), note_to_dict(Note(41, 55)), note_to_dict(Note(43, 50)),
                  note_to_dict(Note(48, 45))]
    d["synth_osc_d"] = 0.3
    d["synth_osc_v"] = 0.65
    d["synth_osc_volume"] = 0.55
    d["synth_sub_volume"] = 0.4
    d["synth_filter_cutoff"] = 500.0
    d["synth_filter_resonance"] = 0.08
    d["synth_filter_env_amount"] = 300.0
    d["synth_vol_attack"] = 100.0
    d["synth_vol_decay"] = 800.0
    d["synth_vol_sustain"] = 0.55
    d["synth_vol_release"] = 1200.0
    d["synth_reverb_mix"] = 0.2
    d["synth_reverb_decay"] = 0.65
    d["note_length_percent"] = 150.0
    presets.append(create_preset("Ambient Sub", "Factory",
        "Deep space - slow moving bass for atmospheric tracks", d))

    # 29. Jungle Amen - Classic break bass
    d = create_default_preset()
    d["straight_1_8"] = [100.0, 0.0, 85.0, 0.0, 0.0, 90.0, 0.0, 80.0]
    d["straight_1_16"] = [0.0, 70.0, 0.0, 65.0, 75.0, 0.0, 60.0, 0.0, 72.0, 0.0, 68.0, 0.0, 0.0, 78.0, 0.0, 55.0]
    d["strength_values"] = create_strength_pattern("funk")
    d["root_note"] = 36
    d["notes"] = [note_to_dict(Note(36, 100)), note_to_dict(Note(41, 58)), note_to_dict(Note(43, 52)),
                  note_to_dict(Note(48, 46))]
    d["synth_osc_d"] = 0.52
    d["synth_osc_v"] = 0.45
    d["synth_osc_volume"] = 0.72
    d["synth_sub_volume"] = 0.28
    d["synth_filter_cutoff"] = 1400.0
    d["synth_filter_resonance"] = 0.3
    d["synth_filter_env_amount"] = 1800.0
    d["synth_vol_attack"] = 0.5
    d["synth_vol_decay"] = 120.0
    d["synth_vol_sustain"] = 0.38
    d["synth_vol_release"] = 180.0
    d["note_length_percent"] = 55.0
    presets.append(create_preset("Jungle Amen", "Factory",
        "Break science - classic jungle bass for chopped beats", d))

    # 30. Psytrance Pulse - Goa bass
    d = create_default_preset()
    d["straight_1_16"] = [100.0, 80.0, 85.0, 75.0, 95.0, 78.0, 82.0, 70.0, 90.0, 76.0, 84.0, 72.0, 88.0, 74.0, 80.0, 68.0]
    d["strength_values"] = create_strength_pattern("driving")
    d["root_note"] = 36
    d["notes"] = [note_to_dict(Note(36, 100)), note_to_dict(Note(39, 55)), note_to_dict(Note(43, 50)),
                  note_to_dict(Note(48, 45))]
    d["synth_osc_d"] = 0.58
    d["synth_osc_v"] = 0.38
    d["synth_osc_volume"] = 0.75
    d["synth_sub_volume"] = 0.25
    d["synth_filter_cutoff"] = 800.0
    d["synth_filter_resonance"] = 0.45
    d["synth_filter_env_amount"] = 2500.0
    d["synth_vol_attack"] = 0.5
    d["synth_vol_decay"] = 120.0
    d["synth_vol_sustain"] = 0.35
    d["synth_vol_release"] = 160.0
    d["note_length_percent"] = 50.0
    d["lfo1_tempo_sync"] = True
    d["lfo1_sync_division"] = 4
    d["lfo1_waveform"] = 2
    d["lfo1_dest1"] = 12
    d["lfo1_amount1"] = 0.2
    presets.append(create_preset("Psytrance Pulse", "Factory",
        "Morning madness - rolling 16th bass for outdoor raves", d))

    # 31. Modern R&B - Smooth bass
    d = create_default_preset()
    d["straight_1_4"] = [100.0, 0.0, 80.0, 0.0]
    d["straight_1_8"] = [0.0, 70.0, 0.0, 65.0, 0.0, 75.0, 0.0, 60.0]
    d["strength_values"] = create_strength_pattern("backbeat")
    d["root_note"] = 36
    d["notes"] = [note_to_dict(Note(36, 100)), note_to_dict(Note(41, 58)), note_to_dict(Note(43, 52)),
                  note_to_dict(Note(48, 46)), note_to_dict(Note(53, 40))]
    d["synth_osc_d"] = 0.4
    d["synth_osc_v"] = 0.58
    d["synth_osc_volume"] = 0.65
    d["synth_sub_volume"] = 0.35
    d["synth_filter_cutoff"] = 800.0
    d["synth_filter_resonance"] = 0.12
    d["synth_filter_env_amount"] = 600.0
    d["synth_vol_attack"] = 3.0
    d["synth_vol_decay"] = 250.0
    d["synth_vol_sustain"] = 0.48
    d["synth_vol_release"] = 350.0
    d["note_length_percent"] = 80.0
    presets.append(create_preset("Modern R&B", "Factory",
        "Smooth vibes - warm bass supports silky grooves", d))

    # 32. Warehouse Techno - Raw underground
    d = create_default_preset()
    d["straight_1_4"] = [100.0, 95.0, 100.0, 90.0]
    d["straight_1_8"] = [80.0, 75.0, 82.0, 70.0, 78.0, 72.0, 85.0, 68.0]
    d["strength_values"] = create_strength_pattern("driving")
    d["root_note"] = 36
    d["notes"] = [note_to_dict(Note(36, 100)), note_to_dict(Note(41, 50)), note_to_dict(Note(43, 45))]
    d["synth_osc_d"] = 0.55
    d["synth_osc_v"] = 0.4
    d["synth_osc_volume"] = 0.75
    d["synth_sub_volume"] = 0.3
    d["synth_filter_cutoff"] = 700.0
    d["synth_filter_resonance"] = 0.35
    d["synth_filter_env_amount"] = 1800.0
    d["synth_vol_attack"] = 0.5
    d["synth_vol_decay"] = 100.0
    d["synth_vol_sustain"] = 0.35
    d["synth_vol_release"] = 150.0
    d["note_length_percent"] = 55.0
    presets.append(create_preset("Warehouse Techno", "Factory",
        "Raw power - stripped back bass for dark floors", d))

    return presets

def generate_all_banks():
    """Generate all 8 banks and save to JSON files"""
    import os

    script_dir = os.path.dirname(os.path.abspath(__file__))
    output_dir = os.path.join(script_dir, "..", "assets", "presets")

    banks = [
        ("A", "World & Ethnic", create_bank_a),
        ("B", "Electronic", create_bank_b),
        ("C", "Classic Genres", create_bank_c),
        ("D", "Experimental", create_bank_d),
        ("E", "Psychedelic & Space", create_bank_e),
        ("F", "Trance & Progressive", create_bank_f),
        ("G", "Cinematic & Classic", create_bank_g),
        ("H", "Bass & Rhythm", create_bank_h),
    ]

    for bank_letter, bank_name, create_func in banks:
        presets = create_func()

        if len(presets) != 32:
            print(f"Warning: Bank {bank_letter} has {len(presets)} presets, expected 32")

        bank_data = {
            "name": bank_letter,
            "presets": presets
        }

        output_path = os.path.join(output_dir, f"factory_bank_{bank_letter.lower()}.json")
        with open(output_path, "w") as f:
            json.dump(bank_data, f, indent=2)

        print(f"Generated Bank {bank_letter} ({bank_name}): {len(presets)} presets -> {output_path}")

if __name__ == "__main__":
    generate_all_banks()
    print("\nAll factory presets generated successfully!")
