#!/usr/bin/env python3
"""
Factory Preset Generator - Masterpiece Edition
Focus: Grooves, Melodies, Music Theory, VPS Sound, Minimal FX
"""

import json
import sys
sys.path.insert(0, '.')
from scripts.generate_presets import (
    create_default_preset, note_to_dict, Note,
    create_octave_randomization, create_strength_pattern,
    SCALE_DEFINITIONS, euclidean_rhythm
)

def n(midi, chance, beat=64, beat_len=64, oct_off=0):
    """Shorthand for creating note dict"""
    return note_to_dict(Note(midi, chance, beat, beat_len, oct_off))

def create_preset(name, author, description, data):
    return {"name": name, "author": author, "description": description, "data": data}

def make_strength_custom(accents):
    """Create strength pattern from accent positions (0-23 within bar, repeats 4x)"""
    s = [35] * 96
    for pos, strength in accents:
        for bar in range(4):
            s[bar * 24 + pos] = strength
    return s

# =============================================================================
# PRESET DEFINITIONS - Each one carefully crafted
# =============================================================================

def preset_01():
    """Breathing Space - Meditative whole/half notes, root-fifth stability"""
    d = create_default_preset()
    d["straight_1_1"] = [60.0]
    d["straight_1_2"] = [80.0, 40.0]
    d["straight_1_4"] = [30.0, 0.0, 20.0, 0.0]
    d["strength_values"] = make_strength_custom([(0, 100), (12, 70)])
    d["root_note"] = 48  # C3
    d["scale"] = "Major"
    d["stability_pattern"] = "Ambient"
    d["octave_randomization"] = create_octave_randomization(15, 100, 110, "Down")
    d["notes"] = [
        n(48, 127, 100, 120),      # Root - strong, long
        n(55, 90, 90, 100),        # Fifth - strong, long
        n(52, 50, 64, 80),         # Third - any, longish
        n(60, 30, 80, 90, 1),      # Octave up - strong, long
    ]
    d["synth_osc_d"] = 0.3
    d["synth_osc_v"] = 0.4
    d["synth_osc_volume"] = 0.7
    d["synth_filter_cutoff"] = 1500.0
    d["synth_vol_attack"] = 50.0
    d["synth_vol_decay"] = 800.0
    d["synth_vol_sustain"] = 0.7
    d["synth_vol_release"] = 1000.0
    d["note_length_percent"] = 100.0
    return create_preset("Breathing Space", "Factory",
        "Meditative pads - root and fifth anchor long sustained tones", d)

def preset_02():
    """Pocket Funk - Classic 16th funk with ghost notes"""
    d = create_default_preset()
    d["straight_1_16"] = [100.0, 30.0, 50.0, 35.0, 80.0, 25.0, 60.0, 40.0,
                          90.0, 20.0, 45.0, 55.0, 70.0, 35.0, 50.0, 30.0]
    d["strength_values"] = create_strength_pattern("funk")
    d["root_note"] = 43  # G2
    d["scale"] = "PentatonicMinor"
    d["stability_pattern"] = "Traditional"
    d["octave_randomization"] = create_octave_randomization(20, 30, 40, "Up")
    d["notes"] = [
        n(43, 127, 110, 90),       # Root - strong, medium
        n(50, 85, 80, 60),         # Fifth - strong, short
        n(46, 70, 100, 70),        # b3 - strong, medium
        n(48, 55, 40, 40),         # 4th - weak, short (ghost)
        n(55, 45, 30, 30),         # b7 - weak, short (ghost)
    ]
    d["synth_osc_d"] = 0.55
    d["synth_osc_v"] = 0.6
    d["synth_osc_volume"] = 0.75
    d["synth_filter_cutoff"] = 2500.0
    d["synth_vol_attack"] = 2.0
    d["synth_vol_decay"] = 150.0
    d["synth_vol_sustain"] = 0.3
    d["synth_vol_release"] = 100.0
    d["note_length_percent"] = 50.0
    d["swing_amount"] = 54.0
    return create_preset("Pocket Funk", "Factory",
        "Classic funk pocket - ghost notes dance around the one", d)

def preset_03():
    """Euclidean Five - E(8,5) rhythm, Lydian brightness"""
    d = create_default_preset()
    euc = euclidean_rhythm(8, 5)  # [0, 1, 3, 4, 6]
    probs = [90.0 if i in euc else 20.0 for i in range(8)]
    d["straight_1_8"] = probs
    d["strength_values"] = make_strength_custom([(0, 100), (3, 85), (6, 90), (9, 75), (12, 80), (15, 70), (18, 85), (21, 65)])
    d["root_note"] = 53  # F3
    d["scale"] = "Lydian"
    d["stability_pattern"] = "Melodic"
    d["octave_randomization"] = create_octave_randomization(25, 64, 90, "Up")
    d["notes"] = [
        n(53, 127, 64, 80),        # Root
        n(60, 80, 70, 90),         # Fifth
        n(57, 70, 80, 85),         # Third
        n(59, 55, 50, 70),         # #4 (Lydian char)
        n(65, 40, 40, 60, 1),      # Octave up
    ]
    d["synth_osc_d"] = 0.45
    d["synth_osc_v"] = 0.55
    d["synth_osc_volume"] = 0.72
    d["synth_filter_cutoff"] = 3000.0
    d["synth_vol_attack"] = 5.0
    d["synth_vol_decay"] = 300.0
    d["synth_vol_sustain"] = 0.5
    d["synth_vol_release"] = 400.0
    d["note_length_percent"] = 80.0
    return create_preset("Euclidean Five", "Factory",
        "Mathematical elegance - E(8,5) pattern in bright Lydian", d)

def preset_04():
    """Bass Cathedral - Low end worship, octave below dominance"""
    d = create_default_preset()
    d["straight_1_4"] = [100.0, 40.0, 70.0, 30.0]
    d["straight_1_8"] = [60.0, 0.0, 30.0, 0.0, 50.0, 0.0, 25.0, 0.0]
    d["strength_values"] = make_strength_custom([(0, 100), (6, 60), (12, 85), (18, 55)])
    d["root_note"] = 36  # C2
    d["scale"] = "Minor"
    d["stability_pattern"] = "BassHeavy"
    d["octave_randomization"] = create_octave_randomization(10, 120, 100, "Down")
    d["notes"] = [
        n(36, 127, 120, 110),      # Root - very strong, very long
        n(43, 90, 110, 100),       # Fifth - strong, long
        n(39, 60, 100, 90),        # b3 - strong, long
        n(41, 40, 90, 80),         # 4th - strong, medium
        n(48, 25, 70, 60, 1),      # Octave up - medium, medium
    ]
    d["synth_osc_d"] = 0.25
    d["synth_osc_v"] = 0.35
    d["synth_osc_octave"] = -1
    d["synth_osc_volume"] = 0.8
    d["synth_filter_cutoff"] = 800.0
    d["synth_vol_attack"] = 10.0
    d["synth_vol_decay"] = 500.0
    d["synth_vol_sustain"] = 0.8
    d["synth_vol_release"] = 600.0
    d["note_length_percent"] = 95.0
    return create_preset("Bass Cathedral", "Factory",
        "Deep low-end meditation - sub frequencies anchor the space", d)

def preset_05():
    """Chromatic Whisper - Sparse tension, short high notes"""
    d = create_default_preset()
    d["straight_1_8"] = [40.0, 0.0, 25.0, 0.0, 35.0, 0.0, 20.0, 0.0]
    d["straight_1_16"] = [0.0, 15.0, 0.0, 10.0, 0.0, 20.0, 0.0, 12.0,
                          0.0, 18.0, 0.0, 8.0, 0.0, 22.0, 0.0, 14.0]
    d["strength_values"] = make_strength_custom([(0, 70), (6, 50), (12, 65), (18, 45)])
    d["root_note"] = 60  # C4
    d["scale"] = "Chromatic"
    d["stability_pattern"] = "Tension"
    d["octave_randomization"] = create_octave_randomization(30, 30, 30, "Up")
    d["notes"] = [
        n(60, 127, 64, 64),        # Root
        n(67, 60, 50, 40),         # Fifth - weak, short
        n(61, 45, 35, 25),         # b2 - weak, very short
        n(63, 50, 40, 30),         # b3 - weak, short
        n(66, 40, 30, 25),         # b5 - weak, very short
        n(70, 35, 25, 20),         # b7 - weak, very short
    ]
    d["synth_osc_d"] = 0.6
    d["synth_osc_v"] = 0.7
    d["synth_osc_volume"] = 0.65
    d["synth_filter_cutoff"] = 4000.0
    d["synth_vol_attack"] = 3.0
    d["synth_vol_decay"] = 80.0
    d["synth_vol_sustain"] = 0.2
    d["synth_vol_release"] = 150.0
    d["note_length_percent"] = 35.0
    return create_preset("Chromatic Whisper", "Factory",
        "Sparse tension - chromatic passing tones flicker briefly", d)

def preset_06():
    """Seven Pulse - Implied 7/8 through accent pattern"""
    d = create_default_preset()
    d["straight_1_8"] = [100.0, 70.0, 85.0, 60.0, 90.0, 75.0, 80.0, 50.0]
    d["strength_values"] = make_strength_custom([
        (0, 100), (3, 85), (6, 90), (9, 70), (12, 95), (15, 80), (18, 75)
    ])
    d["root_note"] = 50  # D3
    d["scale"] = "Mixolydian"
    d["stability_pattern"] = "Traditional"
    d["octave_randomization"] = create_octave_randomization(20, 64, 64, "Both")
    d["notes"] = [
        n(50, 127, 100, 80),       # Root - strong
        n(57, 85, 90, 75),         # Fifth - strong
        n(54, 70, 80, 70),         # Third
        n(48, 55, 100, 90),        # b7 - strong, long (mixo char)
        n(52, 45, 60, 60),         # 4th
    ]
    d["synth_osc_d"] = 0.5
    d["synth_osc_v"] = 0.5
    d["synth_osc_volume"] = 0.72
    d["synth_filter_cutoff"] = 2200.0
    d["synth_vol_attack"] = 4.0
    d["synth_vol_decay"] = 200.0
    d["synth_vol_sustain"] = 0.45
    d["synth_vol_release"] = 250.0
    d["note_length_percent"] = 70.0
    return create_preset("Seven Pulse", "Factory",
        "Asymmetric groove - 7/8 feel implied through shifting accents", d)

def preset_07():
    """Call Response - Two-bar phrase, octave answers"""
    d = create_default_preset()
    d["straight_1_4"] = [90.0, 50.0, 70.0, 40.0]
    d["straight_1_8"] = [70.0, 30.0, 50.0, 25.0, 60.0, 35.0, 45.0, 20.0]
    s = [40] * 96
    for i in range(48):
        if i % 24 == 0: s[i] = 100
        elif i % 12 == 0: s[i] = 75
        elif i % 6 == 0: s[i] = 60
    for i in range(48, 96):
        if i % 24 == 0: s[i] = 70
        elif i % 12 == 0: s[i] = 85
        elif i % 6 == 0: s[i] = 65
    d["strength_values"] = s
    d["root_note"] = 48  # C3
    d["scale"] = "PentatonicMajor"
    d["stability_pattern"] = "Melodic"
    d["octave_randomization"] = create_octave_randomization(40, 40, 100, "Up")
    d["notes"] = [
        n(48, 127, 100, 70),       # Root
        n(55, 80, 80, 80),         # Fifth
        n(52, 70, 60, 90),         # Third - any, long (melodic)
        n(50, 60, 50, 85),         # 2nd - weak, long
        n(57, 50, 70, 75),         # 6th
        n(60, 35, 40, 100, 1),     # Octave - weak, very long (answer)
    ]
    d["synth_osc_d"] = 0.4
    d["synth_osc_v"] = 0.5
    d["synth_osc_volume"] = 0.7
    d["synth_filter_cutoff"] = 2800.0
    d["synth_vol_attack"] = 8.0
    d["synth_vol_decay"] = 350.0
    d["synth_vol_sustain"] = 0.55
    d["synth_vol_release"] = 400.0
    d["note_length_percent"] = 85.0
    return create_preset("Call Response", "Factory",
        "Conversational melody - phrases answer across octaves", d)

def preset_08():
    """Desert Wind - Phrygian ornaments, triplet flourishes"""
    d = create_default_preset()
    d["straight_1_8"] = [100.0, 40.0, 75.0, 35.0, 85.0, 45.0, 65.0, 30.0]
    d["triplet_1_8t"] = [0.0, 35.0, 25.0, 0.0, 40.0, 30.0, 0.0, 45.0, 20.0, 0.0, 35.0, 28.0]
    d["strength_values"] = make_strength_custom([
        (0, 100), (4, 70), (8, 85), (12, 90), (16, 75), (20, 65)
    ])
    d["root_note"] = 52  # E3
    d["scale"] = "Phrygian"
    d["stability_pattern"] = "Traditional"
    d["octave_randomization"] = create_octave_randomization(25, 40, 50, "Up")
    d["notes"] = [
        n(52, 127, 100, 85),       # Root (E)
        n(53, 70, 35, 30),         # b2 (F) - weak, short (Phrygian char)
        n(55, 65, 80, 70),         # b3 (G) - strong
        n(59, 80, 90, 80),         # 5th (B) - strong
        n(60, 55, 50, 45),         # b6 (C) - medium
        n(64, 40, 45, 40, 1),      # Octave - weak, short
    ]
    d["synth_osc_d"] = 0.5
    d["synth_osc_v"] = 0.6
    d["synth_osc_volume"] = 0.7
    d["synth_filter_cutoff"] = 3200.0
    d["synth_vol_attack"] = 5.0
    d["synth_vol_decay"] = 280.0
    d["synth_vol_sustain"] = 0.4
    d["synth_vol_release"] = 350.0
    d["note_length_percent"] = 75.0
    d["swing_amount"] = 52.0
    return create_preset("Desert Wind", "Factory",
        "Phrygian mode - the flat 2nd brings ancient desert colors", d)

def preset_09():
    """Polyrhythm Garden - 3 against 4 cross-rhythm"""
    d = create_default_preset()
    d["straight_1_4"] = [100.0, 0.0, 80.0, 0.0]
    d["triplet_1_4t"] = [90.0, 0.0, 75.0, 85.0, 0.0, 70.0]
    d["strength_values"] = create_strength_pattern("polyrhythm_3_4")
    d["root_note"] = 45  # A2
    d["scale"] = "Dorian"
    d["stability_pattern"] = "JazzMelodic"
    d["octave_randomization"] = create_octave_randomization(30, 64, 64, "Both")
    d["notes"] = [
        n(45, 127, 64, 70),        # Root (A)
        n(52, 85, 70, 75),         # 5th (E)
        n(48, 75, 80, 65),         # b3 (C)
        n(50, 65, 75, 80),         # 4th (D)
        n(54, 55, 60, 70),         # 6th (F#) - Dorian char
        n(43, 45, 90, 85),         # b7 (G)
    ]
    d["synth_osc_d"] = 0.45
    d["synth_osc_v"] = 0.52
    d["synth_osc_volume"] = 0.72
    d["synth_filter_cutoff"] = 2600.0
    d["synth_vol_attack"] = 6.0
    d["synth_vol_decay"] = 250.0
    d["synth_vol_sustain"] = 0.5
    d["synth_vol_release"] = 300.0
    d["note_length_percent"] = 75.0
    return create_preset("Polyrhythm Garden", "Factory",
        "3 against 4 - quarter and triplet patterns interlock", d)

def preset_10():
    """Sparse Fifths - Minimal, consonant, quarter notes"""
    d = create_default_preset()
    d["straight_1_4"] = [85.0, 30.0, 50.0, 25.0]
    d["straight_1_2"] = [60.0, 40.0]
    d["strength_values"] = make_strength_custom([(0, 100), (12, 60), (6, 40), (18, 35)])
    d["root_note"] = 41  # F2
    d["scale"] = "Major"
    d["stability_pattern"] = "Ambient"
    d["octave_randomization"] = create_octave_randomization(10, 110, 100, "Down")
    d["notes"] = [
        n(41, 127, 110, 110),      # Root - very strong, very long
        n(48, 100, 100, 100),      # 5th - strong, long
        n(53, 50, 90, 95, 1),      # Octave - strong, long
    ]
    d["synth_osc_d"] = 0.35
    d["synth_osc_v"] = 0.4
    d["synth_osc_volume"] = 0.7
    d["synth_filter_cutoff"] = 1200.0
    d["synth_vol_attack"] = 30.0
    d["synth_vol_decay"] = 600.0
    d["synth_vol_sustain"] = 0.75
    d["synth_vol_release"] = 800.0
    d["note_length_percent"] = 100.0
    return create_preset("Sparse Fifths", "Factory",
        "Minimal meditation - just root and fifth, pure consonance", d)

def preset_11():
    """Clave Drive - Son clave pattern drives the melody"""
    d = create_default_preset()
    clave_8 = [100.0, 0.0, 0.0, 80.0, 0.0, 0.0, 75.0, 0.0]
    clave_16 = [0.0, 0.0, 0.0, 0.0, 85.0, 0.0, 0.0, 0.0, 0.0, 0.0, 70.0, 0.0, 0.0, 0.0, 0.0, 0.0]
    d["straight_1_8"] = clave_8
    d["straight_1_16"] = clave_16
    d["strength_values"] = create_strength_pattern("latin")
    d["root_note"] = 48  # C3
    d["scale"] = "Minor"
    d["stability_pattern"] = "Traditional"
    d["octave_randomization"] = create_octave_randomization(25, 80, 60, "Up")
    d["notes"] = [
        n(48, 127, 100, 80),       # Root
        n(55, 85, 90, 75),         # 5th
        n(51, 70, 85, 70),         # b3
        n(53, 60, 75, 65),         # 4th
        n(58, 50, 70, 60),         # b6
    ]
    d["synth_osc_d"] = 0.52
    d["synth_osc_v"] = 0.58
    d["synth_osc_volume"] = 0.73
    d["synth_filter_cutoff"] = 2800.0
    d["synth_vol_attack"] = 3.0
    d["synth_vol_decay"] = 180.0
    d["synth_vol_sustain"] = 0.4
    d["synth_vol_release"] = 200.0
    d["note_length_percent"] = 65.0
    d["swing_amount"] = 52.0
    return create_preset("Clave Drive", "Factory",
        "Son clave foundation - 3-2 pattern anchors the groove", d)

def preset_12():
    """Glass Drops - High register, fast 16ths, sparse selection"""
    d = create_default_preset()
    d["straight_1_16"] = [60.0, 20.0, 35.0, 15.0, 50.0, 25.0, 40.0, 18.0,
                          55.0, 22.0, 30.0, 12.0, 45.0, 28.0, 38.0, 20.0]
    d["strength_values"] = make_strength_custom([
        (0, 80), (3, 55), (6, 70), (9, 50), (12, 75), (15, 60), (18, 65), (21, 45)
    ])
    d["root_note"] = 72  # C5
    d["scale"] = "Major"
    d["stability_pattern"] = "Even"
    d["octave_randomization"] = create_octave_randomization(15, 30, 30, "Down")
    d["notes"] = [
        n(72, 127, 64, 50),        # Root
        n(79, 80, 64, 45),         # 5th
        n(76, 70, 64, 50),         # 3rd
        n(74, 60, 64, 55),         # 2nd
        n(77, 55, 64, 45),         # 4th
        n(81, 45, 64, 40),         # 6th
    ]
    d["synth_osc_d"] = 0.65
    d["synth_osc_v"] = 0.75
    d["synth_osc_volume"] = 0.6
    d["synth_filter_cutoff"] = 6000.0
    d["synth_vol_attack"] = 1.0
    d["synth_vol_decay"] = 60.0
    d["synth_vol_sustain"] = 0.15
    d["synth_vol_release"] = 100.0
    d["note_length_percent"] = 30.0
    return create_preset("Glass Drops", "Factory",
        "High crystalline drops - major scale rains down in 16ths", d)

def preset_13():
    """Blues Truth - Dominant 7th and blue notes"""
    d = create_default_preset()
    d["straight_1_8"] = [100.0, 40.0, 70.0, 50.0, 85.0, 35.0, 60.0, 45.0]
    d["triplet_1_8t"] = [50.0, 30.0, 40.0, 45.0, 25.0, 35.0, 55.0, 28.0, 42.0, 48.0, 32.0, 38.0]
    d["strength_values"] = create_strength_pattern("shuffle")
    d["root_note"] = 40  # E2
    d["scale"] = "Blues"
    d["stability_pattern"] = "Traditional"
    d["octave_randomization"] = create_octave_randomization(30, 100, 80, "Up")
    d["notes"] = [
        n(40, 127, 110, 90),       # Root (E)
        n(43, 80, 90, 70),         # b3 (G)
        n(45, 70, 80, 75),         # 4th (A)
        n(46, 60, 50, 50),         # b5 (Bb) - blue note, weak, short
        n(47, 75, 85, 80),         # 5th (B)
        n(50, 55, 70, 65),         # b7 (D)
    ]
    d["synth_osc_d"] = 0.48
    d["synth_osc_v"] = 0.55
    d["synth_osc_volume"] = 0.75
    d["synth_filter_cutoff"] = 2400.0
    d["synth_vol_attack"] = 5.0
    d["synth_vol_decay"] = 220.0
    d["synth_vol_sustain"] = 0.45
    d["synth_vol_release"] = 280.0
    d["note_length_percent"] = 70.0
    d["swing_amount"] = 58.0
    return create_preset("Blues Truth", "Factory",
        "Six-note blues - the flat 5 brings that essential tension", d)

def preset_14():
    """Sakura Fall - Japanese scale, sparse beauty"""
    d = create_default_preset()
    d["straight_1_4"] = [80.0, 30.0, 60.0, 25.0]
    d["straight_1_8"] = [50.0, 0.0, 35.0, 20.0, 45.0, 0.0, 30.0, 15.0]
    d["strength_values"] = make_strength_custom([(0, 100), (6, 55), (12, 80), (18, 50)])
    d["root_note"] = 52  # E3
    d["scale"] = "Japanese"
    d["stability_pattern"] = "Ambient"
    d["octave_randomization"] = create_octave_randomization(20, 80, 100, "Up")
    d["notes"] = [
        n(52, 127, 100, 110),      # Root (E)
        n(53, 75, 60, 80),         # b2 (F) - characteristic
        n(57, 85, 90, 95),         # 4th (A)
        n(59, 70, 85, 90),         # 5th (B)
        n(60, 55, 55, 75),         # b6 (C)
    ]
    d["synth_osc_d"] = 0.38
    d["synth_osc_v"] = 0.45
    d["synth_osc_volume"] = 0.68
    d["synth_filter_cutoff"] = 2000.0
    d["synth_vol_attack"] = 15.0
    d["synth_vol_decay"] = 400.0
    d["synth_vol_sustain"] = 0.55
    d["synth_vol_release"] = 600.0
    d["note_length_percent"] = 90.0
    return create_preset("Sakura Fall", "Factory",
        "Japanese In scale - pentatonic with haunting half-steps", d)

def preset_15():
    """Driving Force - Relentless 8th notes, minor power"""
    d = create_default_preset()
    d["straight_1_8"] = [100.0, 85.0, 90.0, 80.0, 95.0, 82.0, 88.0, 78.0]
    d["strength_values"] = create_strength_pattern("driving")
    d["root_note"] = 45  # A2
    d["scale"] = "Minor"
    d["stability_pattern"] = "BassHeavy"
    d["octave_randomization"] = create_octave_randomization(15, 100, 70, "Down")
    d["notes"] = [
        n(45, 127, 110, 80),       # Root
        n(52, 90, 100, 75),        # 5th
        n(48, 75, 90, 70),         # b3
        n(50, 60, 85, 65),         # 4th
        n(53, 45, 75, 60),         # b6
    ]
    d["synth_osc_d"] = 0.5
    d["synth_osc_v"] = 0.55
    d["synth_osc_volume"] = 0.78
    d["synth_filter_cutoff"] = 2000.0
    d["synth_vol_attack"] = 2.0
    d["synth_vol_decay"] = 100.0
    d["synth_vol_sustain"] = 0.5
    d["synth_vol_release"] = 120.0
    d["note_length_percent"] = 60.0
    return create_preset("Driving Force", "Factory",
        "Relentless minor 8ths - propulsive energy with bass weight", d)

def preset_16():
    """Modal Jazz - Dorian flavor, swing feel"""
    d = create_default_preset()
    d["straight_1_4"] = [70.0, 40.0, 60.0, 35.0]
    d["straight_1_8"] = [55.0, 30.0, 45.0, 25.0, 50.0, 35.0, 40.0, 28.0]
    d["triplet_1_8t"] = [40.0, 20.0, 30.0, 35.0, 18.0, 28.0, 38.0, 22.0, 32.0, 42.0, 25.0, 35.0]
    d["strength_values"] = create_strength_pattern("jazz")
    d["root_note"] = 50  # D3
    d["scale"] = "Dorian"
    d["stability_pattern"] = "JazzMelodic"
    d["octave_randomization"] = create_octave_randomization(35, 50, 70, "Both")
    d["notes"] = [
        n(50, 127, 64, 70),        # Root (D)
        n(57, 85, 75, 75),         # 5th (A)
        n(53, 80, 80, 65),         # b3 (F)
        n(55, 70, 70, 80),         # 4th (G)
        n(59, 75, 65, 75),         # 6th (B) - Dorian char
        n(48, 55, 60, 85),         # b7 (C)
        n(52, 45, 55, 60),         # 2nd (E)
    ]
    d["synth_osc_d"] = 0.42
    d["synth_osc_v"] = 0.5
    d["synth_osc_volume"] = 0.7
    d["synth_filter_cutoff"] = 2600.0
    d["synth_vol_attack"] = 8.0
    d["synth_vol_decay"] = 300.0
    d["synth_vol_sustain"] = 0.5
    d["synth_vol_release"] = 350.0
    d["note_length_percent"] = 75.0
    d["swing_amount"] = 56.0
    return create_preset("Modal Jazz", "Factory",
        "Dorian mode with swing - the raised 6th opens up the harmony", d)


def preset_17():
    """Tidal Drift - Slow evolving, whole notes with subtle movement"""
    d = create_default_preset()
    d["straight_1_1"] = [100.0]
    d["straight_1_2"] = [50.0, 30.0]
    d["straight_1_4"] = [20.0, 0.0, 15.0, 0.0]
    s = [30] * 96
    for i in range(96):
        wave = int(30 + 25 * (1 + __import__('math').sin(i * 3.14159 / 48)))
        s[i] = wave
    s[0] = 100
    s[48] = 90
    d["strength_values"] = s
    d["root_note"] = 36  # C2
    d["scale"] = "Major"
    d["stability_pattern"] = "Ambient"
    d["octave_randomization"] = create_octave_randomization(8, 120, 127, "Down")
    d["notes"] = [
        n(36, 127, 110, 127),      # Root - strongest, longest
        n(43, 85, 100, 120),       # 5th
        n(48, 60, 90, 110, 1),     # Octave
        n(40, 40, 80, 100),        # 3rd
    ]
    d["synth_osc_d"] = 0.28
    d["synth_osc_v"] = 0.35
    d["synth_osc_volume"] = 0.72
    d["synth_filter_cutoff"] = 900.0
    d["synth_vol_attack"] = 100.0
    d["synth_vol_decay"] = 1500.0
    d["synth_vol_sustain"] = 0.8
    d["synth_vol_release"] = 2000.0
    d["note_length_percent"] = 100.0
    return create_preset("Tidal Drift", "Factory",
        "Oceanic slowness - whole notes breathe like waves", d)

def preset_18():
    """Staccato Minor - Short punchy notes, minor bite"""
    d = create_default_preset()
    d["straight_1_8"] = [100.0, 0.0, 80.0, 0.0, 90.0, 0.0, 75.0, 0.0]
    d["straight_1_16"] = [0.0, 50.0, 0.0, 40.0, 0.0, 55.0, 0.0, 35.0,
                          0.0, 45.0, 0.0, 38.0, 0.0, 52.0, 0.0, 42.0]
    d["strength_values"] = make_strength_custom([
        (0, 100), (3, 70), (6, 85), (9, 65), (12, 90), (15, 75), (18, 80), (21, 60)
    ])
    d["root_note"] = 48  # C3
    d["scale"] = "HarmonicMinor"
    d["stability_pattern"] = "Traditional"
    d["octave_randomization"] = create_octave_randomization(25, 30, 20, "Up")
    d["notes"] = [
        n(48, 127, 100, 40),       # Root - strong, SHORT
        n(55, 90, 90, 35),         # 5th - strong, short
        n(51, 75, 80, 45),         # b3
        n(53, 60, 70, 40),         # 4th
        n(56, 50, 60, 35),         # b6
        n(59, 65, 75, 30),         # 7 (raised) - harmonic minor char
    ]
    d["synth_osc_d"] = 0.58
    d["synth_osc_v"] = 0.65
    d["synth_osc_volume"] = 0.74
    d["synth_filter_cutoff"] = 3500.0
    d["synth_vol_attack"] = 1.0
    d["synth_vol_decay"] = 50.0
    d["synth_vol_sustain"] = 0.1
    d["synth_vol_release"] = 80.0
    d["note_length_percent"] = 25.0
    return create_preset("Staccato Minor", "Factory",
        "Sharp minor stabs - harmonic minor with that raised 7th edge", d)

def preset_19():
    """Triplet Flow - Pure triplet groove, flowing 8ths"""
    d = create_default_preset()
    d["triplet_1_4t"] = [90.0, 50.0, 70.0, 85.0, 45.0, 65.0]
    d["triplet_1_8t"] = [75.0, 35.0, 55.0, 70.0, 30.0, 50.0, 72.0, 38.0, 58.0, 68.0, 32.0, 48.0]
    d["strength_values"] = create_strength_pattern("triplet_feel")
    d["root_note"] = 53  # F3
    d["scale"] = "Major"
    d["stability_pattern"] = "Melodic"
    d["octave_randomization"] = create_octave_randomization(20, 64, 80, "Both")
    d["notes"] = [
        n(53, 127, 80, 85),        # Root
        n(60, 85, 75, 90),         # 5th
        n(57, 75, 70, 85),         # 3rd
        n(55, 65, 65, 80),         # 2nd
        n(58, 55, 60, 75),         # 4th
        n(62, 50, 55, 70),         # 6th
        n(65, 40, 50, 65, 1),      # Octave
    ]
    d["synth_osc_d"] = 0.45
    d["synth_osc_v"] = 0.52
    d["synth_osc_volume"] = 0.71
    d["synth_filter_cutoff"] = 2700.0
    d["synth_vol_attack"] = 6.0
    d["synth_vol_decay"] = 250.0
    d["synth_vol_sustain"] = 0.5
    d["synth_vol_release"] = 300.0
    d["note_length_percent"] = 80.0
    return create_preset("Triplet Flow", "Factory",
        "Pure triplet motion - no straight notes, all threes", d)

def preset_20():
    """Offbeat Kingdom - Reggae-inspired offbeat emphasis"""
    d = create_default_preset()
    d["straight_1_8"] = [30.0, 100.0, 25.0, 95.0, 28.0, 98.0, 22.0, 92.0]
    d["strength_values"] = create_strength_pattern("reggae")
    d["root_note"] = 41  # F2
    d["scale"] = "Minor"
    d["stability_pattern"] = "Traditional"
    d["octave_randomization"] = create_octave_randomization(15, 100, 80, "Down")
    d["notes"] = [
        n(41, 127, 30, 60),        # Root - plays on offbeats (weak), medium length
        n(48, 90, 40, 55),         # 5th
        n(44, 70, 35, 50),         # b3
        n(46, 55, 25, 45),         # 4th
        n(53, 40, 20, 40, 1),      # Octave
    ]
    d["synth_osc_d"] = 0.38
    d["synth_osc_v"] = 0.45
    d["synth_osc_volume"] = 0.7
    d["synth_filter_cutoff"] = 2000.0
    d["synth_vol_attack"] = 3.0
    d["synth_vol_decay"] = 120.0
    d["synth_vol_sustain"] = 0.35
    d["synth_vol_release"] = 150.0
    d["note_length_percent"] = 45.0
    return create_preset("Offbeat Kingdom", "Factory",
        "Offbeat emphasis - the and beats carry the groove", d)

def preset_21():
    """Euclidean Seven - E(16,7) complex rhythm"""
    d = create_default_preset()
    euc = euclidean_rhythm(16, 7)
    probs = [85.0 if i in euc else 15.0 for i in range(16)]
    d["straight_1_16"] = probs
    d["strength_values"] = make_strength_custom([
        (0, 100), (3, 75), (6, 85), (9, 70), (12, 90), (15, 65), (18, 80), (21, 72)
    ])
    d["root_note"] = 45  # A2
    d["scale"] = "PentatonicMinor"
    d["stability_pattern"] = "Pentatonic"
    d["octave_randomization"] = create_octave_randomization(30, 70, 60, "Both")
    d["notes"] = [
        n(45, 127, 90, 75),        # Root
        n(52, 85, 80, 70),         # 5th
        n(48, 75, 75, 65),         # b3
        n(50, 65, 70, 70),         # 4th
        n(55, 55, 65, 60),         # b7
    ]
    d["synth_osc_d"] = 0.52
    d["synth_osc_v"] = 0.58
    d["synth_osc_volume"] = 0.73
    d["synth_filter_cutoff"] = 2400.0
    d["synth_vol_attack"] = 4.0
    d["synth_vol_decay"] = 180.0
    d["synth_vol_sustain"] = 0.4
    d["synth_vol_release"] = 220.0
    d["note_length_percent"] = 65.0
    return create_preset("Euclidean Seven", "Factory",
        "E(16,7) pattern - mathematically distributed in pentatonic", d)

def preset_22():
    """Wide Open - Very sparse, octave jumps, major space"""
    d = create_default_preset()
    d["straight_1_2"] = [70.0, 30.0]
    d["straight_1_1"] = [50.0]
    d["strength_values"] = make_strength_custom([(0, 100), (12, 50)])
    d["root_note"] = 48  # C3
    d["scale"] = "Major"
    d["stability_pattern"] = "Ambient"
    d["octave_randomization"] = create_octave_randomization(40, 90, 100, "Both")
    d["notes"] = [
        n(48, 127, 100, 110),      # Root
        n(55, 80, 85, 100),        # 5th
        n(60, 60, 75, 95, 1),      # Octave up
        n(36, 50, 110, 120, -1),   # Octave down
    ]
    d["synth_osc_d"] = 0.32
    d["synth_osc_v"] = 0.4
    d["synth_osc_volume"] = 0.68
    d["synth_filter_cutoff"] = 1400.0
    d["synth_vol_attack"] = 40.0
    d["synth_vol_decay"] = 700.0
    d["synth_vol_sustain"] = 0.7
    d["synth_vol_release"] = 900.0
    d["note_length_percent"] = 100.0
    return create_preset("Wide Open", "Factory",
        "Vast spaciousness - octaves span the register with long decay", d)

def preset_23():
    """Groove Machine - Dense 16ths with velocity layers"""
    d = create_default_preset()
    d["straight_1_16"] = [100.0, 55.0, 75.0, 45.0, 90.0, 50.0, 70.0, 48.0,
                          95.0, 52.0, 72.0, 42.0, 88.0, 58.0, 68.0, 40.0]
    d["strength_values"] = create_strength_pattern("4_4_standard")
    d["root_note"] = 43  # G2
    d["scale"] = "Minor"
    d["stability_pattern"] = "BassHeavy"
    d["octave_randomization"] = create_octave_randomization(20, 110, 50, "Down")
    d["notes"] = [
        n(43, 127, 115, 70),       # Root
        n(50, 95, 100, 65),        # 5th
        n(46, 80, 90, 60),         # b3
        n(48, 70, 85, 55),         # 4th
        n(51, 55, 75, 50),         # b6
        n(55, 45, 70, 45, 1),      # Octave
    ]
    d["synth_osc_d"] = 0.55
    d["synth_osc_v"] = 0.6
    d["synth_osc_volume"] = 0.76
    d["synth_filter_cutoff"] = 2200.0
    d["synth_vol_attack"] = 2.0
    d["synth_vol_decay"] = 130.0
    d["synth_vol_sustain"] = 0.35
    d["synth_vol_release"] = 160.0
    d["note_length_percent"] = 55.0
    return create_preset("Groove Machine", "Factory",
        "Dense driving 16ths - layered dynamics create depth", d)

def preset_24():
    """Hungarian Dance - Exotic scale, strong accents"""
    d = create_default_preset()
    d["straight_1_8"] = [100.0, 35.0, 80.0, 45.0, 90.0, 40.0, 75.0, 50.0]
    d["straight_1_16"] = [0.0, 30.0, 0.0, 25.0, 0.0, 35.0, 0.0, 20.0,
                          0.0, 28.0, 0.0, 32.0, 0.0, 22.0, 0.0, 38.0]
    d["strength_values"] = make_strength_custom([
        (0, 100), (4, 60), (6, 90), (10, 55), (12, 85), (16, 70), (18, 80), (22, 50)
    ])
    d["root_note"] = 50  # D3
    d["scale"] = "Hungarian"
    d["stability_pattern"] = "Traditional"
    d["octave_randomization"] = create_octave_randomization(25, 80, 70, "Up")
    d["notes"] = [
        n(50, 127, 100, 80),       # Root (D)
        n(52, 65, 75, 60),         # 2nd (E)
        n(53, 80, 85, 75),         # b3 (F)
        n(56, 70, 70, 65),         # #4 (G#) - characteristic
        n(57, 90, 95, 85),         # 5th (A)
        n(58, 55, 60, 55),         # b6 (Bb)
        n(61, 75, 80, 70),         # 7 (C#)
    ]
    d["synth_osc_d"] = 0.5
    d["synth_osc_v"] = 0.58
    d["synth_osc_volume"] = 0.72
    d["synth_filter_cutoff"] = 3000.0
    d["synth_vol_attack"] = 4.0
    d["synth_vol_decay"] = 200.0
    d["synth_vol_sustain"] = 0.45
    d["synth_vol_release"] = 250.0
    d["note_length_percent"] = 70.0
    d["swing_amount"] = 52.0
    return create_preset("Hungarian Dance", "Factory",
        "Hungarian minor - augmented intervals create drama", d)

def preset_25():
    """Whole Tone Dream - Symmetrical scale, floating quality"""
    d = create_default_preset()
    d["straight_1_4"] = [70.0, 40.0, 60.0, 35.0]
    d["straight_1_8"] = [50.0, 25.0, 40.0, 30.0, 55.0, 28.0, 45.0, 22.0]
    s = [40] * 96
    for i in range(96):
        s[i] = 40 + int(20 * __import__('math').sin(i * 3.14159 / 16))
    s[0] = 90
    s[24] = 85
    s[48] = 90
    s[72] = 85
    d["strength_values"] = s
    d["root_note"] = 48  # C3
    d["scale"] = "WholeTone"
    d["stability_pattern"] = "Even"
    d["octave_randomization"] = create_octave_randomization(30, 64, 90, "Both")
    d["notes"] = [
        n(48, 127, 64, 80),        # Root (C)
        n(50, 80, 64, 75),         # D
        n(52, 75, 64, 80),         # E
        n(54, 70, 64, 75),         # F#
        n(56, 65, 64, 80),         # G#
        n(58, 60, 64, 75),         # A#
    ]
    d["synth_osc_d"] = 0.4
    d["synth_osc_v"] = 0.48
    d["synth_osc_volume"] = 0.68
    d["synth_filter_cutoff"] = 2200.0
    d["synth_vol_attack"] = 20.0
    d["synth_vol_decay"] = 450.0
    d["synth_vol_sustain"] = 0.6
    d["synth_vol_release"] = 550.0
    d["note_length_percent"] = 85.0
    return create_preset("Whole Tone Dream", "Factory",
        "Whole tone scale - no gravity, pure symmetrical floating", d)

def preset_26():
    """Tension Release - Build up and resolve pattern"""
    d = create_default_preset()
    d["straight_1_8"] = [80.0, 40.0, 65.0, 45.0, 75.0, 50.0, 60.0, 55.0]
    s = [40] * 96
    for i in range(96):
        if i < 72:
            s[i] = 40 + int(i * 0.7)
        else:
            s[i] = 90 - int((i - 72) * 2)
    s[0] = 100
    s[72] = 100
    d["strength_values"] = s
    d["root_note"] = 52  # E3
    d["scale"] = "HarmonicMinor"
    d["stability_pattern"] = "Tension"
    d["octave_randomization"] = create_octave_randomization(35, 40, 50, "Up")
    d["notes"] = [
        n(52, 127, 30, 100),       # Root - weak (tension), long (resolve)
        n(59, 85, 90, 40),         # 5th - strong (tension point)
        n(55, 75, 80, 50),         # b3
        n(63, 90, 100, 35),        # 7 (leading tone) - very strong, short
        n(57, 65, 70, 60),         # 4th
        n(60, 55, 60, 45),         # b6
    ]
    d["synth_osc_d"] = 0.5
    d["synth_osc_v"] = 0.55
    d["synth_osc_volume"] = 0.72
    d["synth_filter_cutoff"] = 2600.0
    d["synth_vol_attack"] = 5.0
    d["synth_vol_decay"] = 200.0
    d["synth_vol_sustain"] = 0.4
    d["synth_vol_release"] = 300.0
    d["note_length_percent"] = 70.0
    return create_preset("Tension Release", "Factory",
        "Harmonic minor drama - leading tone creates pull to root", d)

def preset_27():
    """Dotted Dance - Dotted rhythms create forward motion"""
    d = create_default_preset()
    d["dotted_1_4d"] = [85.0, 60.0, 70.0]
    d["dotted_1_8d"] = [75.0, 45.0, 55.0, 70.0, 40.0, 50.0]
    d["straight_1_4"] = [40.0, 20.0, 30.0, 15.0]
    d["strength_values"] = make_strength_custom([
        (0, 100), (9, 85), (18, 75), (6, 60), (15, 55)
    ])
    d["root_note"] = 55  # G3
    d["scale"] = "Major"
    d["stability_pattern"] = "Melodic"
    d["octave_randomization"] = create_octave_randomization(20, 70, 85, "Up")
    d["notes"] = [
        n(55, 127, 90, 90),        # Root
        n(62, 85, 80, 85),         # 5th
        n(59, 75, 75, 80),         # 3rd
        n(57, 65, 70, 75),         # 2nd
        n(60, 55, 65, 70),         # 4th
        n(67, 45, 60, 65, 1),      # Octave
    ]
    d["synth_osc_d"] = 0.48
    d["synth_osc_v"] = 0.54
    d["synth_osc_volume"] = 0.71
    d["synth_filter_cutoff"] = 2800.0
    d["synth_vol_attack"] = 6.0
    d["synth_vol_decay"] = 280.0
    d["synth_vol_sustain"] = 0.5
    d["synth_vol_release"] = 340.0
    d["note_length_percent"] = 78.0
    return create_preset("Dotted Dance", "Factory",
        "Dotted note propulsion - the long-short pattern drives forward", d)

def preset_28():
    """Ghost Notes - Very quiet secondary hits"""
    d = create_default_preset()
    d["straight_1_16"] = [100.0, 25.0, 15.0, 20.0, 85.0, 18.0, 12.0, 22.0,
                          90.0, 20.0, 14.0, 18.0, 80.0, 22.0, 16.0, 25.0]
    d["strength_values"] = create_strength_pattern("funk")
    d["root_note"] = 40  # E2
    d["scale"] = "PentatonicMinor"
    d["stability_pattern"] = "Traditional"
    d["octave_randomization"] = create_octave_randomization(15, 25, 30, "Up")
    d["notes"] = [
        n(40, 127, 100, 70),       # Root - strong main hits
        n(47, 80, 90, 60),         # 5th
        n(43, 65, 85, 55),         # b3
        n(45, 50, 25, 25),         # 4th - ghost note (weak, short)
        n(50, 40, 20, 20),         # b7 - ghost note
        n(52, 30, 15, 15, 1),      # Octave - ghost
    ]
    d["synth_osc_d"] = 0.55
    d["synth_osc_v"] = 0.6
    d["synth_osc_volume"] = 0.74
    d["synth_filter_cutoff"] = 2500.0
    d["synth_vol_attack"] = 2.0
    d["synth_vol_decay"] = 100.0
    d["synth_vol_sustain"] = 0.25
    d["synth_vol_release"] = 130.0
    d["note_length_percent"] = 45.0
    d["swing_amount"] = 55.0
    return create_preset("Ghost Notes", "Factory",
        "Funk ghosts - quiet notes fill the spaces between hits", d)

def preset_29():
    """Arabic Nights - Maqam-inspired melodic movement"""
    d = create_default_preset()
    d["straight_1_8"] = [100.0, 45.0, 75.0, 40.0, 85.0, 50.0, 70.0, 35.0]
    d["triplet_1_8t"] = [0.0, 30.0, 25.0, 0.0, 35.0, 28.0, 0.0, 32.0, 22.0, 0.0, 38.0, 30.0]
    d["strength_values"] = make_strength_custom([
        (0, 100), (4, 65), (7, 85), (12, 90), (16, 70), (19, 80)
    ])
    d["root_note"] = 48  # C3
    d["scale"] = "Arabic"
    d["stability_pattern"] = "Traditional"
    d["octave_randomization"] = create_octave_randomization(25, 60, 70, "Up")
    d["notes"] = [
        n(48, 127, 100, 85),       # Root (C)
        n(49, 70, 50, 45),         # b2 (Db) - characteristic
        n(52, 80, 85, 75),         # 3 (E) - raised
        n(53, 65, 75, 70),         # 4 (F)
        n(55, 90, 95, 85),         # 5 (G)
        n(56, 60, 55, 50),         # b6 (Ab)
        n(58, 55, 65, 60),         # b7 (Bb)
    ]
    d["synth_osc_d"] = 0.48
    d["synth_osc_v"] = 0.55
    d["synth_osc_volume"] = 0.7
    d["synth_filter_cutoff"] = 2800.0
    d["synth_vol_attack"] = 5.0
    d["synth_vol_decay"] = 280.0
    d["synth_vol_sustain"] = 0.45
    d["synth_vol_release"] = 350.0
    d["note_length_percent"] = 75.0
    d["swing_amount"] = 52.0
    return create_preset("Arabic Nights", "Factory",
        "Arabic maqam - the flat 2 and raised 3 create exotic color", d)

def preset_30():
    """Pedal Point - Root drones while melody moves"""
    d = create_default_preset()
    d["straight_1_4"] = [100.0, 50.0, 70.0, 40.0]
    d["straight_1_8"] = [80.0, 30.0, 50.0, 25.0, 60.0, 35.0, 45.0, 20.0]
    d["strength_values"] = make_strength_custom([
        (0, 100), (6, 70), (12, 90), (18, 65)
    ])
    d["root_note"] = 36  # C2
    d["scale"] = "Minor"
    d["stability_pattern"] = "BassHeavy"
    d["octave_randomization"] = create_octave_randomization(35, 30, 80, "Up")
    d["notes"] = [
        n(36, 127, 127, 127),      # Root pedal - ALWAYS strong, ALWAYS long
        n(48, 70, 50, 50, 1),      # Upper root - weaker
        n(55, 75, 60, 60, 1),      # 5th
        n(51, 65, 55, 55, 1),      # b3
        n(53, 55, 50, 50, 1),      # 4th
        n(58, 45, 45, 45, 1),      # b6
    ]
    d["synth_osc_d"] = 0.35
    d["synth_osc_v"] = 0.42
    d["synth_osc_octave"] = 0
    d["synth_osc_volume"] = 0.74
    d["synth_filter_cutoff"] = 1800.0
    d["synth_vol_attack"] = 10.0
    d["synth_vol_decay"] = 400.0
    d["synth_vol_sustain"] = 0.7
    d["synth_vol_release"] = 500.0
    d["note_length_percent"] = 90.0
    return create_preset("Pedal Point", "Factory",
        "Drone foundation - bass root anchors while upper voices move", d)

def preset_31():
    """Quick Silver - Very fast 32nds, sparse selection"""
    d = create_default_preset()
    d["straight_1_32"] = [50.0, 15.0, 25.0, 10.0, 40.0, 12.0, 20.0, 8.0,
                          45.0, 14.0, 22.0, 9.0, 38.0, 11.0, 18.0, 7.0,
                          48.0, 13.0, 24.0, 10.0, 42.0, 12.0, 21.0, 8.0,
                          46.0, 15.0, 23.0, 11.0, 40.0, 13.0, 19.0, 9.0]
    d["strength_values"] = make_strength_custom([
        (0, 100), (6, 80), (12, 90), (18, 75)
    ])
    d["root_note"] = 60  # C4
    d["scale"] = "PentatonicMajor"
    d["stability_pattern"] = "Even"
    d["octave_randomization"] = create_octave_randomization(20, 40, 25, "Both")
    d["notes"] = [
        n(60, 127, 64, 40),        # Root
        n(67, 80, 64, 35),         # 5th
        n(64, 70, 64, 38),         # 3rd
        n(62, 60, 64, 35),         # 2nd
        n(69, 50, 64, 32),         # 6th
    ]
    d["synth_osc_d"] = 0.6
    d["synth_osc_v"] = 0.7
    d["synth_osc_volume"] = 0.65
    d["synth_filter_cutoff"] = 5000.0
    d["synth_vol_attack"] = 0.5
    d["synth_vol_decay"] = 30.0
    d["synth_vol_sustain"] = 0.05
    d["synth_vol_release"] = 50.0
    d["note_length_percent"] = 20.0
    return create_preset("Quick Silver", "Factory",
        "32nd note flurries - rapid pentatonic sparkles", d)

def preset_32():
    """Minor Second - Dissonant cluster potential"""
    d = create_default_preset()
    d["straight_1_8"] = [90.0, 40.0, 70.0, 35.0, 80.0, 45.0, 65.0, 30.0]
    d["straight_1_16"] = [0.0, 30.0, 20.0, 25.0, 0.0, 35.0, 22.0, 28.0,
                          0.0, 32.0, 18.0, 24.0, 0.0, 38.0, 25.0, 30.0]
    d["strength_values"] = make_strength_custom([
        (0, 100), (3, 60), (6, 85), (9, 55), (12, 90), (15, 65), (18, 75), (21, 50)
    ])
    d["root_note"] = 52  # E3
    d["scale"] = "Chromatic"
    d["stability_pattern"] = "Tension"
    d["octave_randomization"] = create_octave_randomization(15, 50, 40, "Up")
    d["notes"] = [
        n(52, 127, 100, 80),       # Root (E)
        n(53, 60, 40, 35),         # b2 (F) - dissonant, weak, short
        n(59, 85, 90, 75),         # 5th (B)
        n(55, 70, 75, 65),         # b3 (G)
        n(54, 50, 35, 30),         # 2 (F#) - cluster potential
        n(58, 45, 45, 40),         # 4th (A)
    ]
    d["synth_osc_d"] = 0.52
    d["synth_osc_v"] = 0.58
    d["synth_osc_volume"] = 0.7
    d["synth_filter_cutoff"] = 2600.0
    d["synth_vol_attack"] = 4.0
    d["synth_vol_decay"] = 180.0
    d["synth_vol_sustain"] = 0.35
    d["synth_vol_release"] = 220.0
    d["note_length_percent"] = 60.0
    return create_preset("Minor Second", "Factory",
        "Chromatic tension - minor 2nds available for cluster effects", d)


def preset_33():
    """Ascending Line - Stepwise melodic motion up"""
    d = create_default_preset()
    d["straight_1_8"] = [90.0, 50.0, 75.0, 45.0, 85.0, 55.0, 70.0, 40.0]
    s = [40] * 96
    for i in range(96):
        s[i] = 40 + int((i % 24) * 2.5)
    s[0] = 100
    s[24] = 100
    s[48] = 100
    s[72] = 100
    d["strength_values"] = s
    d["root_note"] = 48  # C3
    d["scale"] = "Major"
    d["stability_pattern"] = "Melodic"
    d["octave_randomization"] = create_octave_randomization(15, 30, 60, "Up")
    d["notes"] = [
        n(48, 127, 100, 70),       # Root C
        n(50, 85, 60, 75),         # D - 2nd
        n(52, 80, 55, 80),         # E - 3rd
        n(53, 75, 50, 85),         # F - 4th
        n(55, 90, 45, 90),         # G - 5th
        n(57, 70, 40, 85),         # A - 6th
        n(59, 65, 35, 80),         # B - 7th
        n(60, 60, 30, 95, 1),      # C octave - weak, very long
    ]
    d["synth_osc_d"] = 0.45
    d["synth_osc_v"] = 0.52
    d["synth_osc_volume"] = 0.7
    d["synth_filter_cutoff"] = 2600.0
    d["synth_vol_attack"] = 6.0
    d["synth_vol_decay"] = 300.0
    d["synth_vol_sustain"] = 0.55
    d["synth_vol_release"] = 350.0
    d["note_length_percent"] = 80.0
    return create_preset("Ascending Line", "Factory",
        "Stepwise melody climbing - each degree leads to the next", d)

def preset_34():
    """Root Power - Only root and fifth, maximum stability"""
    d = create_default_preset()
    d["straight_1_4"] = [100.0, 60.0, 80.0, 50.0]
    d["straight_1_8"] = [70.0, 35.0, 55.0, 30.0, 65.0, 40.0, 50.0, 25.0]
    d["strength_values"] = create_strength_pattern("4_4_standard")
    d["root_note"] = 36  # C2
    d["scale"] = "Major"
    d["stability_pattern"] = "BassHeavy"
    d["octave_randomization"] = create_octave_randomization(20, 110, 90, "Down")
    d["notes"] = [
        n(36, 127, 120, 100),      # Root
        n(43, 100, 110, 90),       # 5th
        n(48, 70, 90, 80, 1),      # Octave up
        n(24, 50, 127, 110, -1),   # Octave down (very low)
    ]
    d["synth_osc_d"] = 0.3
    d["synth_osc_v"] = 0.38
    d["synth_osc_octave"] = -1
    d["synth_osc_volume"] = 0.8
    d["synth_filter_cutoff"] = 1000.0
    d["synth_vol_attack"] = 8.0
    d["synth_vol_decay"] = 450.0
    d["synth_vol_sustain"] = 0.75
    d["synth_vol_release"] = 550.0
    d["note_length_percent"] = 95.0
    return create_preset("Root Power", "Factory",
        "Maximum stability - only root and fifth, pure power", d)

def preset_35():
    """Syncopation City - Off-beat emphasis throughout"""
    d = create_default_preset()
    d["straight_1_8"] = [40.0, 90.0, 35.0, 85.0, 45.0, 88.0, 38.0, 82.0]
    d["straight_1_16"] = [0.0, 0.0, 60.0, 0.0, 0.0, 0.0, 55.0, 0.0,
                          0.0, 0.0, 65.0, 0.0, 0.0, 0.0, 50.0, 0.0]
    d["strength_values"] = create_strength_pattern("offbeat")
    d["root_note"] = 45  # A2
    d["scale"] = "Dorian"
    d["stability_pattern"] = "JazzMelodic"
    d["octave_randomization"] = create_octave_randomization(25, 40, 50, "Up")
    d["notes"] = [
        n(45, 127, 40, 60),        # Root - weak beat preference!
        n(52, 85, 35, 55),         # 5th
        n(48, 75, 30, 50),         # b3
        n(50, 65, 45, 55),         # 4th
        n(54, 70, 50, 60),         # 6th (Dorian)
        n(43, 55, 35, 50),         # b7
    ]
    d["synth_osc_d"] = 0.52
    d["synth_osc_v"] = 0.58
    d["synth_osc_volume"] = 0.72
    d["synth_filter_cutoff"] = 2500.0
    d["synth_vol_attack"] = 3.0
    d["synth_vol_decay"] = 150.0
    d["synth_vol_sustain"] = 0.4
    d["synth_vol_release"] = 180.0
    d["note_length_percent"] = 55.0
    d["swing_amount"] = 54.0
    return create_preset("Syncopation City", "Factory",
        "Off-beat emphasis - the weak beats become strong", d)

def preset_36():
    """Lydian Light - Bright #4 character"""
    d = create_default_preset()
    d["straight_1_4"] = [80.0, 45.0, 65.0, 40.0]
    d["straight_1_8"] = [60.0, 30.0, 48.0, 25.0, 55.0, 35.0, 45.0, 28.0]
    d["strength_values"] = make_strength_custom([
        (0, 100), (6, 75), (12, 90), (18, 70)
    ])
    d["root_note"] = 53  # F3
    d["scale"] = "Lydian"
    d["stability_pattern"] = "Melodic"
    d["octave_randomization"] = create_octave_randomization(25, 50, 80, "Up")
    d["notes"] = [
        n(53, 127, 80, 85),        # Root F
        n(60, 85, 75, 80),         # 5th C
        n(57, 80, 70, 85),         # 3rd A
        n(59, 90, 85, 90),         # #4 B - THE Lydian note, make it prominent
        n(55, 65, 65, 75),         # 2nd G
        n(62, 55, 60, 70),         # 6th D
        n(64, 50, 55, 75),         # 7th E
    ]
    d["synth_osc_d"] = 0.48
    d["synth_osc_v"] = 0.55
    d["synth_osc_volume"] = 0.7
    d["synth_filter_cutoff"] = 3200.0
    d["synth_vol_attack"] = 10.0
    d["synth_vol_decay"] = 350.0
    d["synth_vol_sustain"] = 0.55
    d["synth_vol_release"] = 400.0
    d["note_length_percent"] = 82.0
    return create_preset("Lydian Light", "Factory",
        "Lydian brightness - the raised 4th lifts everything up", d)

def preset_37():
    """Swing Time - Classic shuffle feel"""
    d = create_default_preset()
    d["straight_1_8"] = [100.0, 55.0, 80.0, 50.0, 90.0, 52.0, 75.0, 48.0]
    d["triplet_1_8t"] = [70.0, 30.0, 50.0, 65.0, 28.0, 45.0, 68.0, 32.0, 48.0, 62.0, 25.0, 42.0]
    d["strength_values"] = create_strength_pattern("shuffle")
    d["root_note"] = 48  # C3
    d["scale"] = "PentatonicMajor"
    d["stability_pattern"] = "Traditional"
    d["octave_randomization"] = create_octave_randomization(20, 80, 70, "Both")
    d["notes"] = [
        n(48, 127, 90, 75),        # Root
        n(55, 85, 85, 70),         # 5th
        n(52, 75, 80, 75),         # 3rd
        n(50, 65, 75, 70),         # 2nd
        n(57, 60, 70, 65),         # 6th
    ]
    d["synth_osc_d"] = 0.5
    d["synth_osc_v"] = 0.55
    d["synth_osc_volume"] = 0.72
    d["synth_filter_cutoff"] = 2800.0
    d["synth_vol_attack"] = 4.0
    d["synth_vol_decay"] = 200.0
    d["synth_vol_sustain"] = 0.45
    d["synth_vol_release"] = 250.0
    d["note_length_percent"] = 70.0
    d["swing_amount"] = 62.0
    return create_preset("Swing Time", "Factory",
        "Classic shuffle - triplet feel meets straight 8ths", d)

def preset_38():
    """Two Bar Phrase - Different patterns each bar"""
    d = create_default_preset()
    d["straight_1_8"] = [100.0, 50.0, 70.0, 40.0, 85.0, 55.0, 65.0, 35.0]
    s = [40] * 96
    for i in range(48):
        beat = i % 24
        if beat == 0: s[i] = 100
        elif beat == 6: s[i] = 80
        elif beat == 12: s[i] = 85
        elif beat == 18: s[i] = 70
        else: s[i] = 45
    for i in range(48, 96):
        beat = i % 24
        if beat == 0: s[i] = 90
        elif beat == 3: s[i] = 75
        elif beat == 9: s[i] = 80
        elif beat == 15: s[i] = 85
        elif beat == 21: s[i] = 70
        else: s[i] = 40
    d["strength_values"] = s
    d["root_note"] = 50  # D3
    d["scale"] = "Minor"
    d["stability_pattern"] = "Traditional"
    d["octave_randomization"] = create_octave_randomization(30, 64, 75, "Both")
    d["notes"] = [
        n(50, 127, 80, 75),        # Root
        n(57, 85, 75, 70),         # 5th
        n(53, 75, 70, 75),         # b3
        n(55, 65, 65, 70),         # 4th
        n(58, 55, 60, 65),         # b6
        n(48, 50, 55, 80),         # b7
    ]
    d["synth_osc_d"] = 0.48
    d["synth_osc_v"] = 0.54
    d["synth_osc_volume"] = 0.71
    d["synth_filter_cutoff"] = 2400.0
    d["synth_vol_attack"] = 5.0
    d["synth_vol_decay"] = 220.0
    d["synth_vol_sustain"] = 0.45
    d["synth_vol_release"] = 280.0
    d["note_length_percent"] = 72.0
    return create_preset("Two Bar Phrase", "Factory",
        "Phrase variation - first bar statement, second bar answer", d)

def preset_39():
    """Minor Blues - Classic minor pentatonic with blue note"""
    d = create_default_preset()
    d["straight_1_8"] = [100.0, 45.0, 75.0, 40.0, 88.0, 50.0, 70.0, 35.0]
    d["triplet_1_8t"] = [55.0, 28.0, 40.0, 50.0, 25.0, 38.0, 58.0, 30.0, 42.0, 52.0, 22.0, 35.0]
    d["strength_values"] = create_strength_pattern("shuffle")
    d["root_note"] = 45  # A2
    d["scale"] = "Blues"
    d["stability_pattern"] = "Traditional"
    d["octave_randomization"] = create_octave_randomization(25, 90, 70, "Up")
    d["notes"] = [
        n(45, 127, 100, 85),       # Root A
        n(48, 80, 85, 70),         # b3 C
        n(50, 70, 80, 75),         # 4th D
        n(51, 55, 45, 40),         # b5 Eb - blue note (weak, short)
        n(52, 85, 90, 80),         # 5th E
        n(55, 65, 75, 70),         # b7 G
    ]
    d["synth_osc_d"] = 0.5
    d["synth_osc_v"] = 0.56
    d["synth_osc_volume"] = 0.74
    d["synth_filter_cutoff"] = 2600.0
    d["synth_vol_attack"] = 4.0
    d["synth_vol_decay"] = 200.0
    d["synth_vol_sustain"] = 0.4
    d["synth_vol_release"] = 260.0
    d["note_length_percent"] = 68.0
    d["swing_amount"] = 56.0
    return create_preset("Minor Blues", "Factory",
        "Classic blues scale - that flat 5 provides the grit", d)

def preset_40():
    """Euclidean Three - E(8,3) sparse pattern"""
    d = create_default_preset()
    euc = euclidean_rhythm(8, 3)
    probs = [95.0 if i in euc else 10.0 for i in range(8)]
    d["straight_1_8"] = probs
    d["strength_values"] = make_strength_custom([
        (0, 100), (9, 90), (18, 85)
    ])
    d["root_note"] = 48  # C3
    d["scale"] = "Minor"
    d["stability_pattern"] = "Ambient"
    d["octave_randomization"] = create_octave_randomization(20, 100, 110, "Down")
    d["notes"] = [
        n(48, 127, 110, 115),      # Root - strong, long
        n(55, 90, 100, 110),       # 5th
        n(51, 70, 90, 100),        # b3
        n(53, 55, 80, 90),         # 4th
    ]
    d["synth_osc_d"] = 0.38
    d["synth_osc_v"] = 0.45
    d["synth_osc_volume"] = 0.7
    d["synth_filter_cutoff"] = 1800.0
    d["synth_vol_attack"] = 15.0
    d["synth_vol_decay"] = 500.0
    d["synth_vol_sustain"] = 0.65
    d["synth_vol_release"] = 600.0
    d["note_length_percent"] = 95.0
    return create_preset("Euclidean Three", "Factory",
        "E(8,3) minimal - maximum space between hits", d)

def preset_41():
    """Melodic Minor Rise - Ascending melodic minor**"""
    d = create_default_preset()
    d["straight_1_8"] = [85.0, 40.0, 68.0, 35.0, 78.0, 45.0, 62.0, 30.0]
    d["strength_values"] = make_strength_custom([
        (0, 100), (4, 65), (8, 80), (12, 90), (16, 70), (20, 75)
    ])
    d["root_note"] = 45  # A2
    d["scale"] = "MelodicMinor"
    d["stability_pattern"] = "Melodic"
    d["octave_randomization"] = create_octave_randomization(20, 45, 75, "Up")
    d["notes"] = [
        n(45, 127, 85, 80),        # Root A
        n(47, 65, 55, 70),         # 2nd B
        n(48, 75, 70, 75),         # b3 C
        n(50, 60, 60, 70),         # 4th D
        n(52, 85, 80, 80),         # 5th E
        n(54, 70, 65, 75),         # 6th F# (raised)
        n(56, 75, 70, 70),         # 7th G# (raised)
        n(57, 55, 50, 85, 1),      # Octave
    ]
    d["synth_osc_d"] = 0.46
    d["synth_osc_v"] = 0.52
    d["synth_osc_volume"] = 0.7
    d["synth_filter_cutoff"] = 2700.0
    d["synth_vol_attack"] = 6.0
    d["synth_vol_decay"] = 280.0
    d["synth_vol_sustain"] = 0.5
    d["synth_vol_release"] = 320.0
    d["note_length_percent"] = 78.0
    return create_preset("Melodic Minor Rise", "Factory",
        "Melodic minor - raised 6th and 7th create upward pull", d)

def preset_42():
    """Dense Texture - Very active 16ths, many notes"""
    d = create_default_preset()
    d["straight_1_16"] = [100.0, 65.0, 80.0, 55.0, 92.0, 60.0, 78.0, 58.0,
                          95.0, 62.0, 82.0, 52.0, 88.0, 68.0, 75.0, 50.0]
    d["strength_values"] = create_strength_pattern("dense")
    d["root_note"] = 48  # C3
    d["scale"] = "Major"
    d["stability_pattern"] = "Even"
    d["octave_randomization"] = create_octave_randomization(30, 64, 50, "Both")
    d["notes"] = [
        n(48, 127, 64, 55),        # Root C
        n(55, 90, 64, 50),         # 5th G
        n(52, 85, 64, 55),         # 3rd E
        n(50, 75, 64, 50),         # 2nd D
        n(53, 70, 64, 55),         # 4th F
        n(57, 65, 64, 50),         # 6th A
        n(59, 60, 64, 45),         # 7th B
    ]
    d["synth_osc_d"] = 0.55
    d["synth_osc_v"] = 0.62
    d["synth_osc_volume"] = 0.72
    d["synth_filter_cutoff"] = 3200.0
    d["synth_vol_attack"] = 2.0
    d["synth_vol_decay"] = 100.0
    d["synth_vol_sustain"] = 0.3
    d["synth_vol_release"] = 120.0
    d["note_length_percent"] = 50.0
    return create_preset("Dense Texture", "Factory",
        "Maximum activity - every 16th has a chance", d)

def preset_43():
    """Locrian Darkness - Diminished flavor"""
    d = create_default_preset()
    d["straight_1_8"] = [100.0, 40.0, 75.0, 45.0, 85.0, 38.0, 70.0, 42.0]
    d["strength_values"] = make_strength_custom([
        (0, 100), (6, 85), (12, 95), (18, 80)
    ])
    d["root_note"] = 47  # B2
    d["scale"] = "Locrian"
    d["stability_pattern"] = "Tension"
    d["octave_randomization"] = create_octave_randomization(20, 70, 60, "Up")
    d["notes"] = [
        n(47, 127, 90, 80),        # Root B
        n(48, 80, 60, 50),         # b2 C - Locrian char
        n(50, 70, 75, 65),         # b3 D
        n(52, 60, 70, 60),         # 4th E
        n(53, 75, 55, 50),         # b5 F - another dissonance
        n(55, 65, 80, 70),         # b6 G
        n(57, 55, 65, 55),         # b7 A
    ]
    d["synth_osc_d"] = 0.5
    d["synth_osc_v"] = 0.55
    d["synth_osc_volume"] = 0.7
    d["synth_filter_cutoff"] = 2400.0
    d["synth_vol_attack"] = 5.0
    d["synth_vol_decay"] = 220.0
    d["synth_vol_sustain"] = 0.4
    d["synth_vol_release"] = 280.0
    d["note_length_percent"] = 65.0
    return create_preset("Locrian Darkness", "Factory",
        "Locrian mode - diminished root triad creates instability", d)

def preset_44():
    """African Bell - 12/8 feel bell pattern"""
    d = create_default_preset()
    bell_pattern = [100.0, 0.0, 70.0, 80.0, 0.0, 65.0, 85.0, 0.0, 60.0, 75.0, 0.0, 55.0]
    d["triplet_1_4t"] = [90.0, 50.0, 70.0, 85.0, 45.0, 65.0]
    d["triplet_1_8t"] = bell_pattern
    d["strength_values"] = create_strength_pattern("african")
    d["root_note"] = 55  # G3
    d["scale"] = "PentatonicMajor"
    d["stability_pattern"] = "Traditional"
    d["octave_randomization"] = create_octave_randomization(25, 80, 65, "Both")
    d["notes"] = [
        n(55, 127, 95, 70),        # Root G
        n(62, 85, 85, 65),         # 5th D
        n(59, 75, 80, 70),         # 3rd B
        n(57, 65, 75, 65),         # 2nd A
        n(64, 55, 70, 60),         # 6th E
    ]
    d["synth_osc_d"] = 0.6
    d["synth_osc_v"] = 0.68
    d["synth_osc_volume"] = 0.7
    d["synth_filter_cutoff"] = 4000.0
    d["synth_vol_attack"] = 1.0
    d["synth_vol_decay"] = 80.0
    d["synth_vol_sustain"] = 0.2
    d["synth_vol_release"] = 120.0
    d["note_length_percent"] = 45.0
    return create_preset("African Bell", "Factory",
        "12/8 bell pattern - interlocking triplet feel", d)

def preset_45():
    """Question Answer - Call in first half, response in second"""
    d = create_default_preset()
    d["straight_1_4"] = [90.0, 50.0, 70.0, 40.0]
    d["straight_1_8"] = [75.0, 30.0, 55.0, 25.0, 65.0, 45.0, 50.0, 35.0]
    s = [35] * 96
    for i in range(48):
        if i % 6 == 0: s[i] = 90 + (6 - (i % 24) // 6) * 2
        elif i % 3 == 0: s[i] = 60
    for i in range(48, 96):
        beat = (i - 48) % 24
        if beat in [3, 9, 15]: s[i] = 85
        elif beat in [6, 12, 18]: s[i] = 70
        elif beat == 21: s[i] = 95
    d["strength_values"] = s
    d["root_note"] = 52  # E3
    d["scale"] = "PentatonicMinor"
    d["stability_pattern"] = "Melodic"
    d["octave_randomization"] = create_octave_randomization(35, 50, 90, "Up")
    d["notes"] = [
        n(52, 127, 90, 70),        # Root - strong, medium
        n(59, 80, 70, 80),         # 5th - medium, long
        n(55, 75, 75, 85),         # b3
        n(57, 70, 65, 90),         # 4th - longer on answers
        n(62, 55, 55, 95),         # b7 - weak, very long
        n(64, 45, 45, 100, 1),     # Octave - answer note
    ]
    d["synth_osc_d"] = 0.45
    d["synth_osc_v"] = 0.52
    d["synth_osc_volume"] = 0.7
    d["synth_filter_cutoff"] = 2600.0
    d["synth_vol_attack"] = 8.0
    d["synth_vol_decay"] = 350.0
    d["synth_vol_sustain"] = 0.55
    d["synth_vol_release"] = 400.0
    d["note_length_percent"] = 80.0
    return create_preset("Question Answer", "Factory",
        "Call and response - phrases dialogue across the bar", d)

def preset_46():
    """Mixolydian Groove - Dominant 7th funk"""
    d = create_default_preset()
    d["straight_1_16"] = [100.0, 40.0, 65.0, 35.0, 85.0, 45.0, 60.0, 38.0,
                          90.0, 42.0, 70.0, 32.0, 80.0, 48.0, 55.0, 30.0]
    d["strength_values"] = create_strength_pattern("funk")
    d["root_note"] = 43  # G2
    d["scale"] = "Mixolydian"
    d["stability_pattern"] = "Traditional"
    d["octave_randomization"] = create_octave_randomization(20, 100, 60, "Down")
    d["notes"] = [
        n(43, 127, 110, 75),       # Root G
        n(50, 90, 95, 70),         # 5th D
        n(47, 80, 90, 65),         # 3rd B
        n(45, 70, 85, 70),         # 2nd A
        n(48, 65, 80, 65),         # 4th C
        n(52, 60, 75, 60),         # 6th E
        n(53, 75, 70, 55),         # b7 F - Mixolydian char
    ]
    d["synth_osc_d"] = 0.54
    d["synth_osc_v"] = 0.6
    d["synth_osc_volume"] = 0.75
    d["synth_filter_cutoff"] = 2800.0
    d["synth_vol_attack"] = 2.0
    d["synth_vol_decay"] = 140.0
    d["synth_vol_sustain"] = 0.35
    d["synth_vol_release"] = 170.0
    d["note_length_percent"] = 55.0
    d["swing_amount"] = 53.0
    return create_preset("Mixolydian Groove", "Factory",
        "Dominant 7th flavor - the flat 7 gives that funk edge", d)

def preset_47():
    """Half Time Feel - Quarter notes dominate"""
    d = create_default_preset()
    d["straight_1_2"] = [100.0, 70.0]
    d["straight_1_4"] = [85.0, 40.0, 75.0, 35.0]
    d["straight_1_8"] = [30.0, 15.0, 25.0, 12.0, 28.0, 18.0, 22.0, 10.0]
    d["strength_values"] = make_strength_custom([
        (0, 100), (12, 85), (6, 50), (18, 45)
    ])
    d["root_note"] = 41  # F2
    d["scale"] = "Minor"
    d["stability_pattern"] = "BassHeavy"
    d["octave_randomization"] = create_octave_randomization(15, 110, 100, "Down")
    d["notes"] = [
        n(41, 127, 115, 110),      # Root - very strong, very long
        n(48, 90, 100, 100),       # 5th
        n(44, 75, 95, 95),         # b3
        n(46, 60, 90, 90),         # 4th
        n(53, 45, 85, 85, 1),      # Octave
    ]
    d["synth_osc_d"] = 0.35
    d["synth_osc_v"] = 0.42
    d["synth_osc_volume"] = 0.74
    d["synth_filter_cutoff"] = 1600.0
    d["synth_vol_attack"] = 15.0
    d["synth_vol_decay"] = 500.0
    d["synth_vol_sustain"] = 0.7
    d["synth_vol_release"] = 600.0
    d["note_length_percent"] = 92.0
    return create_preset("Half Time Feel", "Factory",
        "Half time heaviness - quarter notes anchor the groove", d)

def preset_48():
    """Chromatic Run - All 12 notes available"""
    d = create_default_preset()
    d["straight_1_16"] = [80.0, 35.0, 55.0, 30.0, 70.0, 40.0, 50.0, 28.0,
                          75.0, 38.0, 52.0, 32.0, 65.0, 42.0, 48.0, 25.0]
    d["strength_values"] = make_strength_custom([
        (0, 100), (3, 60), (6, 80), (9, 55), (12, 90), (15, 65), (18, 75), (21, 50)
    ])
    d["root_note"] = 60  # C4
    d["scale"] = "Chromatic"
    d["stability_pattern"] = "Even"
    d["octave_randomization"] = create_octave_randomization(25, 50, 40, "Both")
    d["notes"] = [
        n(60, 127, 64, 60),        # C (root)
        n(67, 70, 64, 55),         # G (5th)
        n(64, 65, 64, 55),         # E (3rd)
        n(61, 40, 64, 40),         # Db
        n(62, 50, 64, 45),         # D
        n(63, 45, 64, 42),         # Eb
        n(65, 55, 64, 48),         # F
        n(66, 42, 64, 38),         # F#
        n(68, 48, 64, 45),         # Ab
        n(69, 52, 64, 48),         # A
        n(70, 45, 64, 40),         # Bb
        n(71, 50, 64, 45),         # B
    ]
    d["synth_osc_d"] = 0.55
    d["synth_osc_v"] = 0.62
    d["synth_osc_volume"] = 0.68
    d["synth_filter_cutoff"] = 3500.0
    d["synth_vol_attack"] = 3.0
    d["synth_vol_decay"] = 120.0
    d["synth_vol_sustain"] = 0.3
    d["synth_vol_release"] = 150.0
    d["note_length_percent"] = 50.0
    return create_preset("Chromatic Run", "Factory",
        "All 12 notes - chromatic passing tones for color", d)


def preset_49():
    """Tremolo Gates - Rapid 32nd rhythmic gating"""
    d = create_default_preset()
    d["straight_1_32"] = [100.0, 0.0, 0.0, 0.0, 85.0, 0.0, 0.0, 0.0,
                          90.0, 0.0, 0.0, 0.0, 80.0, 0.0, 0.0, 0.0,
                          95.0, 0.0, 0.0, 0.0, 82.0, 0.0, 0.0, 0.0,
                          88.0, 0.0, 0.0, 0.0, 78.0, 0.0, 0.0, 0.0]
    d["strength_values"] = create_strength_pattern("4_4_standard")
    d["root_note"] = 48  # C3
    d["scale"] = "Minor"
    d["stability_pattern"] = "Traditional"
    d["octave_randomization"] = create_octave_randomization(10, 90, 50, "Both")
    d["notes"] = [
        n(48, 127, 95, 50),        # Root
        n(55, 85, 90, 45),         # 5th
        n(51, 70, 85, 48),         # b3
        n(53, 60, 80, 45),         # 4th
    ]
    d["synth_osc_d"] = 0.5
    d["synth_osc_v"] = 0.55
    d["synth_osc_volume"] = 0.72
    d["synth_filter_cutoff"] = 2800.0
    d["synth_vol_attack"] = 1.0
    d["synth_vol_decay"] = 40.0
    d["synth_vol_sustain"] = 0.2
    d["synth_vol_release"] = 60.0
    d["note_length_percent"] = 25.0
    return create_preset("Tremolo Gates", "Factory",
        "Rhythmic gating - 32nd pulses create tremolo effect", d)

def preset_50():
    """Suspended Fourth - Sus4 chord movement"""
    d = create_default_preset()
    d["straight_1_4"] = [100.0, 60.0, 80.0, 50.0]
    d["straight_1_8"] = [70.0, 35.0, 55.0, 30.0, 65.0, 40.0, 50.0, 25.0]
    d["strength_values"] = make_strength_custom([
        (0, 100), (6, 75), (12, 90), (18, 70)
    ])
    d["root_note"] = 48  # C3
    d["scale"] = "Major"
    d["stability_pattern"] = "Ambient"
    d["octave_randomization"] = create_octave_randomization(20, 100, 95, "Both")
    d["notes"] = [
        n(48, 127, 100, 100),      # Root C
        n(55, 95, 95, 95),         # 5th G
        n(53, 90, 90, 90),         # Sus4 F (instead of E)
        n(52, 40, 50, 60),         # 3rd E (lower chance - resolution)
        n(60, 50, 85, 85, 1),      # Octave
    ]
    d["synth_osc_d"] = 0.38
    d["synth_osc_v"] = 0.45
    d["synth_osc_volume"] = 0.7
    d["synth_filter_cutoff"] = 2000.0
    d["synth_vol_attack"] = 20.0
    d["synth_vol_decay"] = 500.0
    d["synth_vol_sustain"] = 0.65
    d["synth_vol_release"] = 600.0
    d["note_length_percent"] = 90.0
    return create_preset("Suspended Fourth", "Factory",
        "Sus4 tension - the fourth wants to resolve to the third", d)

def preset_51():
    """Tresillo Beat - 3+3+2 Latin pattern"""
    d = create_default_preset()
    tresillo = [100.0, 0.0, 0.0, 85.0, 0.0, 0.0, 90.0, 0.0]
    d["straight_1_8"] = tresillo
    d["straight_1_16"] = [0.0, 40.0, 35.0, 0.0, 45.0, 30.0, 0.0, 50.0,
                          0.0, 38.0, 32.0, 0.0, 42.0, 28.0, 0.0, 48.0]
    d["strength_values"] = create_strength_pattern("latin")
    d["root_note"] = 50  # D3
    d["scale"] = "Minor"
    d["stability_pattern"] = "Traditional"
    d["octave_randomization"] = create_octave_randomization(25, 85, 65, "Up")
    d["notes"] = [
        n(50, 127, 100, 75),       # Root D
        n(57, 90, 90, 70),         # 5th A
        n(53, 80, 85, 65),         # b3 F
        n(55, 70, 80, 70),         # 4th G
        n(58, 55, 70, 60),         # b6 Bb
    ]
    d["synth_osc_d"] = 0.52
    d["synth_osc_v"] = 0.58
    d["synth_osc_volume"] = 0.73
    d["synth_filter_cutoff"] = 2600.0
    d["synth_vol_attack"] = 3.0
    d["synth_vol_decay"] = 160.0
    d["synth_vol_sustain"] = 0.4
    d["synth_vol_release"] = 200.0
    d["note_length_percent"] = 60.0
    d["swing_amount"] = 52.0
    return create_preset("Tresillo Beat", "Factory",
        "3+3+2 pattern - the heartbeat of Latin music", d)

def preset_52():
    """Contrary Motion - High and low diverge"""
    d = create_default_preset()
    d["straight_1_8"] = [90.0, 45.0, 70.0, 40.0, 80.0, 50.0, 65.0, 35.0]
    s = [40] * 96
    for i in range(96):
        pos = i % 24
        if pos < 12:
            s[i] = 50 + int(pos * 4)
        else:
            s[i] = 100 - int((pos - 12) * 4)
    d["strength_values"] = s
    d["root_note"] = 48  # C3
    d["scale"] = "Minor"
    d["stability_pattern"] = "Melodic"
    d["octave_randomization"] = create_octave_randomization(40, 64, 64, "Both")
    d["notes"] = [
        n(48, 127, 50, 70),        # Root - middle ground
        n(55, 75, 100, 75, 1),     # 5th high - prefers strong
        n(43, 80, 30, 85, -1),     # 5th low - prefers weak
        n(51, 60, 80, 65, 1),      # b3 high
        n(39, 65, 40, 80, -1),     # b3 low
    ]
    d["synth_osc_d"] = 0.45
    d["synth_osc_v"] = 0.52
    d["synth_osc_volume"] = 0.7
    d["synth_filter_cutoff"] = 2400.0
    d["synth_vol_attack"] = 8.0
    d["synth_vol_decay"] = 300.0
    d["synth_vol_sustain"] = 0.5
    d["synth_vol_release"] = 400.0
    d["note_length_percent"] = 80.0
    return create_preset("Contrary Motion", "Factory",
        "Diverging lines - high notes on strong, low on weak beats", d)

def preset_53():
    """Ionian Purity - Simple major, clean intervals"""
    d = create_default_preset()
    d["straight_1_4"] = [100.0, 50.0, 75.0, 45.0]
    d["straight_1_8"] = [60.0, 30.0, 45.0, 25.0, 55.0, 35.0, 40.0, 20.0]
    d["strength_values"] = make_strength_custom([
        (0, 100), (6, 70), (12, 90), (18, 65)
    ])
    d["root_note"] = 60  # C4
    d["scale"] = "Major"
    d["stability_pattern"] = "Traditional"
    d["octave_randomization"] = create_octave_randomization(15, 80, 85, "Both")
    d["notes"] = [
        n(60, 127, 95, 90),        # Root C
        n(67, 90, 90, 85),         # 5th G
        n(64, 80, 85, 80),         # 3rd E
        n(65, 65, 75, 75),         # 4th F
        n(62, 55, 70, 70),         # 2nd D
    ]
    d["synth_osc_d"] = 0.42
    d["synth_osc_v"] = 0.5
    d["synth_osc_volume"] = 0.7
    d["synth_filter_cutoff"] = 2800.0
    d["synth_vol_attack"] = 6.0
    d["synth_vol_decay"] = 280.0
    d["synth_vol_sustain"] = 0.55
    d["synth_vol_release"] = 350.0
    d["note_length_percent"] = 80.0
    return create_preset("Ionian Purity", "Factory",
        "Pure major mode - traditional note hierarchy", d)

def preset_54():
    """Pulse Width - Alternating long and short"""
    d = create_default_preset()
    d["straight_1_8"] = [100.0, 70.0, 90.0, 65.0, 95.0, 68.0, 85.0, 62.0]
    d["strength_values"] = make_strength_custom([
        (0, 100), (3, 50), (6, 95), (9, 45), (12, 100), (15, 55), (18, 90), (21, 48)
    ])
    d["root_note"] = 45  # A2
    d["scale"] = "PentatonicMinor"
    d["stability_pattern"] = "Traditional"
    d["octave_randomization"] = create_octave_randomization(20, 100, 30, "Both")
    d["notes"] = [
        n(45, 127, 110, 100),      # Root - long on strong
        n(45, 80, 30, 40),         # Root alt - short on weak
        n(52, 75, 95, 90),         # 5th - long on strong
        n(52, 60, 35, 45),         # 5th alt - short
        n(48, 65, 90, 85),         # b3 - long
        n(48, 50, 40, 50),         # b3 alt - short
    ]
    d["synth_osc_d"] = 0.5
    d["synth_osc_v"] = 0.55
    d["synth_osc_volume"] = 0.72
    d["synth_filter_cutoff"] = 2500.0
    d["synth_vol_attack"] = 3.0
    d["synth_vol_decay"] = 150.0
    d["synth_vol_sustain"] = 0.4
    d["synth_vol_release"] = 200.0
    d["note_length_percent"] = 65.0
    return create_preset("Pulse Width", "Factory",
        "Long-short alternation - dynamic length contrast", d)

def preset_55():
    """Fifth Drone - Constant fifth with moving melody"""
    d = create_default_preset()
    d["straight_1_2"] = [90.0, 70.0]
    d["straight_1_4"] = [60.0, 40.0, 55.0, 35.0]
    d["straight_1_8"] = [45.0, 25.0, 38.0, 20.0, 42.0, 28.0, 35.0, 18.0]
    d["strength_values"] = make_strength_custom([
        (0, 100), (12, 85), (6, 60), (18, 55)
    ])
    d["root_note"] = 41  # F2
    d["scale"] = "Major"
    d["stability_pattern"] = "BassHeavy"
    d["octave_randomization"] = create_octave_randomization(15, 30, 100, "Up")
    d["notes"] = [
        n(48, 127, 127, 127),      # 5th C - ALWAYS present (drone)
        n(41, 90, 100, 90),        # Root F
        n(53, 70, 50, 70, 1),      # Octave F
        n(45, 60, 60, 75),         # 3rd A
        n(46, 50, 55, 70),         # 4th Bb
        n(50, 45, 50, 65),         # 6th D
    ]
    d["synth_osc_d"] = 0.35
    d["synth_osc_v"] = 0.42
    d["synth_osc_volume"] = 0.72
    d["synth_filter_cutoff"] = 1600.0
    d["synth_vol_attack"] = 12.0
    d["synth_vol_decay"] = 450.0
    d["synth_vol_sustain"] = 0.7
    d["synth_vol_release"] = 550.0
    d["note_length_percent"] = 95.0
    return create_preset("Fifth Drone", "Factory",
        "Constant fifth - the 5th drones while other notes move", d)

def preset_56():
    """Aeolian Night - Natural minor melancholy"""
    d = create_default_preset()
    d["straight_1_8"] = [90.0, 40.0, 70.0, 35.0, 80.0, 45.0, 65.0, 30.0]
    d["strength_values"] = make_strength_custom([
        (0, 100), (4, 65), (8, 80), (12, 90), (16, 70), (20, 75)
    ])
    d["root_note"] = 45  # A2
    d["scale"] = "Minor"
    d["stability_pattern"] = "Traditional"
    d["octave_randomization"] = create_octave_randomization(25, 70, 85, "Up")
    d["notes"] = [
        n(45, 127, 100, 85),       # Root A
        n(52, 90, 90, 80),         # 5th E
        n(48, 80, 85, 75),         # b3 C
        n(50, 70, 80, 80),         # 4th D
        n(53, 60, 70, 70),         # b6 F
        n(55, 55, 65, 65),         # b7 G
        n(47, 50, 60, 75),         # 2nd B
    ]
    d["synth_osc_d"] = 0.45
    d["synth_osc_v"] = 0.52
    d["synth_osc_volume"] = 0.7
    d["synth_filter_cutoff"] = 2400.0
    d["synth_vol_attack"] = 8.0
    d["synth_vol_decay"] = 320.0
    d["synth_vol_sustain"] = 0.5
    d["synth_vol_release"] = 380.0
    d["note_length_percent"] = 78.0
    return create_preset("Aeolian Night", "Factory",
        "Natural minor - the classic minor scale sound", d)

def preset_57():
    """Double Time - Very fast feel, sparse hits"""
    d = create_default_preset()
    d["straight_1_16"] = [100.0, 30.0, 40.0, 25.0, 85.0, 28.0, 35.0, 22.0,
                          90.0, 32.0, 38.0, 20.0, 80.0, 35.0, 42.0, 28.0]
    d["straight_1_32"] = [60.0, 15.0, 20.0, 12.0, 50.0, 18.0, 22.0, 10.0,
                          55.0, 16.0, 18.0, 14.0, 48.0, 20.0, 25.0, 12.0,
                          58.0, 14.0, 19.0, 11.0, 52.0, 17.0, 21.0, 13.0,
                          54.0, 15.0, 20.0, 10.0, 50.0, 18.0, 24.0, 15.0]
    d["strength_values"] = create_strength_pattern("driving")
    d["root_note"] = 48  # C3
    d["scale"] = "PentatonicMinor"
    d["stability_pattern"] = "Traditional"
    d["octave_randomization"] = create_octave_randomization(30, 40, 35, "Both")
    d["notes"] = [
        n(48, 127, 90, 50),        # Root
        n(55, 85, 85, 45),         # 5th
        n(51, 70, 80, 50),         # b3
        n(53, 60, 75, 45),         # 4th
        n(58, 50, 70, 40),         # b7
    ]
    d["synth_osc_d"] = 0.55
    d["synth_osc_v"] = 0.62
    d["synth_osc_volume"] = 0.7
    d["synth_filter_cutoff"] = 3200.0
    d["synth_vol_attack"] = 1.0
    d["synth_vol_decay"] = 60.0
    d["synth_vol_sustain"] = 0.2
    d["synth_vol_release"] = 80.0
    d["note_length_percent"] = 35.0
    return create_preset("Double Time", "Factory",
        "High speed feel - 16ths and 32nds for rapid motion", d)

def preset_58():
    """Octave Leap - Wide intervals, dramatic jumps"""
    d = create_default_preset()
    d["straight_1_4"] = [100.0, 55.0, 80.0, 45.0]
    d["straight_1_8"] = [70.0, 30.0, 50.0, 25.0, 65.0, 35.0, 45.0, 20.0]
    d["strength_values"] = make_strength_custom([
        (0, 100), (6, 80), (12, 95), (18, 75)
    ])
    d["root_note"] = 48  # C3
    d["scale"] = "Minor"
    d["stability_pattern"] = "Melodic"
    d["octave_randomization"] = create_octave_randomization(50, 64, 80, "Both")
    d["notes"] = [
        n(48, 127, 80, 75),        # Root C3
        n(36, 80, 110, 100, -1),   # Root C2 (octave down)
        n(60, 75, 50, 85, 1),      # Root C4 (octave up)
        n(55, 70, 90, 70),         # 5th G3
        n(43, 60, 105, 90, -1),    # 5th G2
        n(67, 55, 45, 80, 1),      # 5th G4
    ]
    d["synth_osc_d"] = 0.42
    d["synth_osc_v"] = 0.5
    d["synth_osc_volume"] = 0.72
    d["synth_filter_cutoff"] = 2600.0
    d["synth_vol_attack"] = 5.0
    d["synth_vol_decay"] = 280.0
    d["synth_vol_sustain"] = 0.5
    d["synth_vol_release"] = 350.0
    d["note_length_percent"] = 82.0
    return create_preset("Octave Leap", "Factory",
        "Dramatic octave jumps - wide register spanning", d)

def preset_59():
    """Additive Build - Gradually increasing density"""
    d = create_default_preset()
    d["straight_1_1"] = [30.0]
    d["straight_1_2"] = [45.0, 35.0]
    d["straight_1_4"] = [60.0, 40.0, 55.0, 38.0]
    d["straight_1_8"] = [75.0, 50.0, 65.0, 45.0, 70.0, 52.0, 62.0, 42.0]
    d["straight_1_16"] = [85.0, 55.0, 70.0, 48.0, 80.0, 58.0, 68.0, 45.0,
                          82.0, 52.0, 72.0, 50.0, 78.0, 55.0, 65.0, 48.0]
    s = [35] * 96
    for i in range(96):
        s[i] = 35 + int(i * 0.65)
    s[0] = 100
    d["strength_values"] = s
    d["root_note"] = 50  # D3
    d["scale"] = "Major"
    d["stability_pattern"] = "Melodic"
    d["octave_randomization"] = create_octave_randomization(25, 50, 60, "Both")
    d["notes"] = [
        n(50, 127, 70, 80),        # Root
        n(57, 85, 65, 75),         # 5th
        n(54, 75, 60, 70),         # 3rd
        n(52, 65, 55, 75),         # 2nd
        n(55, 60, 50, 70),         # 4th
        n(59, 50, 45, 65),         # 6th
    ]
    d["synth_osc_d"] = 0.48
    d["synth_osc_v"] = 0.55
    d["synth_osc_volume"] = 0.7
    d["synth_filter_cutoff"] = 2800.0
    d["synth_vol_attack"] = 5.0
    d["synth_vol_decay"] = 250.0
    d["synth_vol_sustain"] = 0.5
    d["synth_vol_release"] = 300.0
    d["note_length_percent"] = 75.0
    return create_preset("Additive Build", "Factory",
        "Gradual density - all divisions active, building intensity", d)

def preset_60():
    """Falling Thirds - Descending third intervals"""
    d = create_default_preset()
    d["straight_1_8"] = [90.0, 50.0, 75.0, 45.0, 85.0, 55.0, 70.0, 40.0]
    s = [40] * 96
    for i in range(96):
        s[i] = 100 - int((i % 24) * 2.5)
    s[0] = 100
    d["strength_values"] = s
    d["root_note"] = 60  # C4
    d["scale"] = "Minor"
    d["stability_pattern"] = "Melodic"
    d["octave_randomization"] = create_octave_randomization(20, 100, 50, "Down")
    d["notes"] = [
        n(60, 127, 30, 90),        # C4 - weak (descending)
        n(57, 85, 45, 85),         # A3
        n(55, 80, 55, 80),         # G3
        n(53, 75, 65, 75),         # F3
        n(51, 70, 75, 70),         # Eb3
        n(48, 90, 90, 65),         # C3 - arrives strong
    ]
    d["synth_osc_d"] = 0.45
    d["synth_osc_v"] = 0.52
    d["synth_osc_volume"] = 0.7
    d["synth_filter_cutoff"] = 2600.0
    d["synth_vol_attack"] = 6.0
    d["synth_vol_decay"] = 300.0
    d["synth_vol_sustain"] = 0.5
    d["synth_vol_release"] = 350.0
    d["note_length_percent"] = 80.0
    return create_preset("Falling Thirds", "Factory",
        "Descending sequence - thirds cascade downward", d)

def preset_61():
    """Backbeat Heavy - Emphasis on 2 and 4"""
    d = create_default_preset()
    d["straight_1_4"] = [60.0, 100.0, 55.0, 95.0]
    d["straight_1_8"] = [50.0, 85.0, 40.0, 80.0, 48.0, 88.0, 42.0, 78.0]
    d["strength_values"] = create_strength_pattern("backbeat")
    d["root_note"] = 43  # G2
    d["scale"] = "PentatonicMinor"
    d["stability_pattern"] = "Traditional"
    d["octave_randomization"] = create_octave_randomization(20, 35, 60, "Up")
    d["notes"] = [
        n(43, 127, 35, 70),        # Root - prefers weak (2 and 4)
        n(50, 85, 40, 65),         # 5th
        n(46, 75, 30, 60),         # b3
        n(48, 65, 38, 65),         # 4th
        n(53, 55, 45, 55),         # b7
    ]
    d["synth_osc_d"] = 0.52
    d["synth_osc_v"] = 0.58
    d["synth_osc_volume"] = 0.74
    d["synth_filter_cutoff"] = 2500.0
    d["synth_vol_attack"] = 3.0
    d["synth_vol_decay"] = 160.0
    d["synth_vol_sustain"] = 0.4
    d["synth_vol_release"] = 200.0
    d["note_length_percent"] = 60.0
    return create_preset("Backbeat Heavy", "Factory",
        "Backbeat emphasis - beats 2 and 4 dominate", d)

def preset_62():
    """Sparse Tension - Minimal dissonance"""
    d = create_default_preset()
    d["straight_1_2"] = [60.0, 35.0]
    d["straight_1_4"] = [45.0, 25.0, 40.0, 20.0]
    d["strength_values"] = make_strength_custom([
        (0, 100), (12, 70), (6, 45), (18, 40)
    ])
    d["root_note"] = 52  # E3
    d["scale"] = "HarmonicMinor"
    d["stability_pattern"] = "Tension"
    d["octave_randomization"] = create_octave_randomization(15, 90, 110, "Both")
    d["notes"] = [
        n(52, 127, 95, 100),       # Root E
        n(59, 80, 85, 95),         # 5th B
        n(63, 70, 100, 50),        # Leading tone D# - strong, short
        n(55, 55, 75, 85),         # b3 G
        n(60, 45, 50, 40),         # b6 C - weak, short
    ]
    d["synth_osc_d"] = 0.4
    d["synth_osc_v"] = 0.48
    d["synth_osc_volume"] = 0.68
    d["synth_filter_cutoff"] = 2000.0
    d["synth_vol_attack"] = 18.0
    d["synth_vol_decay"] = 450.0
    d["synth_vol_sustain"] = 0.6
    d["synth_vol_release"] = 550.0
    d["note_length_percent"] = 90.0
    return create_preset("Sparse Tension", "Factory",
        "Minimal drama - leading tone creates pull in sparse texture", d)

def preset_63():
    """Motor Rhythm - Constant 16th motion"""
    d = create_default_preset()
    d["straight_1_16"] = [100.0, 75.0, 85.0, 70.0, 95.0, 72.0, 82.0, 68.0,
                          98.0, 78.0, 88.0, 65.0, 92.0, 70.0, 80.0, 72.0]
    d["strength_values"] = create_strength_pattern("driving")
    d["root_note"] = 45  # A2
    d["scale"] = "Minor"
    d["stability_pattern"] = "BassHeavy"
    d["octave_randomization"] = create_octave_randomization(15, 110, 60, "Down")
    d["notes"] = [
        n(45, 127, 115, 65),       # Root
        n(52, 95, 105, 60),        # 5th
        n(48, 80, 95, 55),         # b3
        n(50, 70, 90, 60),         # 4th
        n(53, 55, 80, 50),         # b6
    ]
    d["synth_osc_d"] = 0.55
    d["synth_osc_v"] = 0.6
    d["synth_osc_volume"] = 0.75
    d["synth_filter_cutoff"] = 2400.0
    d["synth_vol_attack"] = 1.0
    d["synth_vol_decay"] = 80.0
    d["synth_vol_sustain"] = 0.35
    d["synth_vol_release"] = 100.0
    d["note_length_percent"] = 50.0
    return create_preset("Motor Rhythm", "Factory",
        "Relentless 16ths - constant motion drives forward", d)

def preset_64():
    """Final Resolution - Perfect cadence feel"""
    d = create_default_preset()
    d["straight_1_2"] = [100.0, 60.0]
    d["straight_1_4"] = [80.0, 45.0, 70.0, 40.0]
    d["straight_1_8"] = [55.0, 30.0, 45.0, 25.0, 50.0, 35.0, 40.0, 20.0]
    s = [40] * 96
    for i in range(72):
        if i % 24 == 0: s[i] = 100 - int(i / 3)
        elif i % 12 == 0: s[i] = 80 - int(i / 4)
        elif i % 6 == 0: s[i] = 60
    for i in range(72, 96):
        s[i] = 50 + int((i - 72) * 2)
    s[72] = 100
    d["strength_values"] = s
    d["root_note"] = 48  # C3
    d["scale"] = "Major"
    d["stability_pattern"] = "Traditional"
    d["octave_randomization"] = create_octave_randomization(20, 100, 110, "Down")
    d["notes"] = [
        n(48, 127, 100, 120),      # Root C - final resolution
        n(55, 90, 85, 80),         # 5th G (dominant)
        n(52, 75, 80, 75),         # 3rd E
        n(53, 55, 60, 50),         # 4th F (subdominant approach)
        n(59, 65, 70, 55),         # 7th B (leading tone)
    ]
    d["synth_osc_d"] = 0.4
    d["synth_osc_v"] = 0.48
    d["synth_osc_volume"] = 0.72
    d["synth_filter_cutoff"] = 2400.0
    d["synth_vol_attack"] = 10.0
    d["synth_vol_decay"] = 400.0
    d["synth_vol_sustain"] = 0.6
    d["synth_vol_release"] = 500.0
    d["note_length_percent"] = 88.0
    return create_preset("Final Resolution", "Factory",
        "Perfect cadence - dominant to tonic resolution", d)


# Collect all preset functions
ALL_PRESETS = [
    preset_01, preset_02, preset_03, preset_04, preset_05, preset_06,
    preset_07, preset_08, preset_09, preset_10, preset_11, preset_12,
    preset_13, preset_14, preset_15, preset_16, preset_17, preset_18,
    preset_19, preset_20, preset_21, preset_22, preset_23, preset_24,
    preset_25, preset_26, preset_27, preset_28, preset_29, preset_30,
    preset_31, preset_32, preset_33, preset_34, preset_35, preset_36,
    preset_37, preset_38, preset_39, preset_40, preset_41, preset_42,
    preset_43, preset_44, preset_45, preset_46, preset_47, preset_48,
    preset_49, preset_50, preset_51, preset_52, preset_53, preset_54,
    preset_55, preset_56, preset_57, preset_58, preset_59, preset_60,
    preset_61, preset_62, preset_63, preset_64,
]

def generate_banks():
    """Generate all 4 banks as JSON files"""
    import os

    presets = [fn() for fn in ALL_PRESETS]

    bank_names = {
        'a': 'Factory Bank A',
        'b': 'Factory Bank B',
        'c': 'Factory Bank C',
        'd': 'Factory Bank D',
    }

    banks = {
        'a': presets[0:16],
        'b': presets[16:32],
        'c': presets[32:48],
        'd': presets[48:64],
    }

    output_dir = os.path.join(os.path.dirname(__file__), '..', 'assets', 'presets')
    os.makedirs(output_dir, exist_ok=True)

    for bank_letter, bank_presets in banks.items():
        filename = f'factory_bank_{bank_letter}.json'
        filepath = os.path.join(output_dir, filename)

        # Create PresetBank format: {"name": "...", "presets": [...]}
        bank_data = {
            "name": bank_names[bank_letter],
            "presets": bank_presets
        }

        with open(filepath, 'w') as f:
            json.dump(bank_data, f, indent=2)

        print(f"Generated {filename} with {len(bank_presets)} presets:")
        for i, p in enumerate(bank_presets):
            print(f"  {i+1}. {p['name']}")

    print(f"\nTotal: {len(presets)} presets across 4 banks")

if __name__ == "__main__":
    generate_banks()
