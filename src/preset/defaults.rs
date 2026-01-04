#![allow(clippy::field_reassign_with_default)]

use super::data::{Preset, PresetBank, PresetData, NotePresetData};

pub fn create_default_presets() -> [PresetBank; 4] {
    [
        create_bank_a(),
        create_bank_b(),
        create_bank_c(),
        create_bank_d(),
    ]
}

fn create_bank_a() -> PresetBank {
    let mut bank = PresetBank::new("A");

    bank.presets[0] = create_four_on_floor();
    bank.presets[1] = create_basic_8th();
    bank.presets[2] = create_offbeat();
    bank.presets[3] = create_shuffle();
    bank.presets[4] = create_breakbeat();
    bank.presets[5] = create_halftime();
    bank.presets[6] = create_syncopated();
    bank.presets[7] = create_polyrhythm();
    bank.presets[8] = create_triplet_groove();
    bank.presets[9] = create_dotted_bounce();
    bank.presets[10] = create_sparse_minimal();
    bank.presets[11] = create_busy_16th();
    bank.presets[12] = create_kick_snare();
    bank.presets[13] = create_rolling_hats();
    bank.presets[14] = create_swing_feel();
    bank.presets[15] = create_random_hits();

    bank
}

fn create_bank_b() -> PresetBank {
    let mut bank = PresetBank::new("B");

    bank.presets[0] = create_techno_driving();
    bank.presets[1] = create_house_classic();
    bank.presets[2] = create_minimal_techno();
    bank.presets[3] = create_acid_line();
    bank.presets[4] = create_trance_gate();
    bank.presets[5] = create_industrial();
    bank.presets[6] = create_detroit();
    bank.presets[7] = create_berlin();
    bank.presets[8] = create_dub_techno();
    bank.presets[9] = create_hard_techno();
    bank.presets[10] = create_deep_house();
    bank.presets[11] = create_progressive();
    bank.presets[12] = create_electro();
    bank.presets[13] = create_ebm();
    bank.presets[14] = create_dark_wave();
    bank.presets[15] = create_synth_pop();

    bank
}

fn create_bank_c() -> PresetBank {
    let mut bank = PresetBank::new("C");

    bank.presets[0] = create_glitch_stutter();
    bank.presets[1] = create_idm_complex();
    bank.presets[2] = create_broken_beat();
    bank.presets[3] = create_micro_rhythm();
    bank.presets[4] = create_generative();
    bank.presets[5] = create_euclidean_5();
    bank.presets[6] = create_euclidean_7();
    bank.presets[7] = create_euclidean_11();
    bank.presets[8] = create_noise_burst();
    bank.presets[9] = create_granular_feel();
    bank.presets[10] = create_polymetric();
    bank.presets[11] = create_chaos_theory();
    bank.presets[12] = create_stuttered_gate();
    bank.presets[13] = create_bit_crush();
    bank.presets[14] = create_circuit_bent();
    bank.presets[15] = create_data_stream();

    bank
}

fn create_bank_d() -> PresetBank {
    let mut bank = PresetBank::new("D");

    bank.presets[0] = create_ambient_pulse();
    bank.presets[1] = create_drone_evolve();
    bank.presets[2] = create_meditation();
    bank.presets[3] = create_breath();
    bank.presets[4] = create_sparse_bells();
    bank.presets[5] = create_underwater();
    bank.presets[6] = create_space();
    bank.presets[7] = create_forest();
    bank.presets[8] = create_minimal_piano();
    bank.presets[9] = create_slow_motion();
    bank.presets[10] = create_dreamscape();
    bank.presets[11] = create_whisper();
    bank.presets[12] = create_heartbeat();
    bank.presets[13] = create_time_stretch();
    bank.presets[14] = create_frozen();
    bank.presets[15] = create_void();

    bank
}

fn create_four_on_floor() -> Preset {
    let mut data = PresetData::default();
    data.straight_1_4 = [127.0, 127.0, 127.0, 127.0];
    data.root_note = 36;
    data.synth_pll_volume = 0.8;
    data.synth_pll_track_speed = 0.6;
    data.synth_vol_attack = 5.0;
    data.synth_vol_decay = 50.0;
    data.synth_vol_sustain = 0.3;
    data.synth_vol_release = 100.0;
    Preset::with_data("4 On Floor", data)
}

fn create_basic_8th() -> Preset {
    let mut data = PresetData::default();
    data.straight_1_8 = [127.0, 80.0, 127.0, 80.0, 127.0, 80.0, 127.0, 80.0];
    data.root_note = 48;
    data.synth_pll_volume = 0.7;
    data.synth_pll_track_speed = 0.5;
    data.synth_vol_attack = 2.0;
    data.synth_vol_decay = 80.0;
    data.synth_vol_sustain = 0.4;
    data.synth_vol_release = 150.0;
    Preset::with_data("Basic 8th", data)
}

fn create_offbeat() -> Preset {
    let mut data = PresetData::default();
    data.straight_1_8 = [0.0, 127.0, 0.0, 127.0, 0.0, 127.0, 0.0, 127.0];
    data.root_note = 60;
    data.notes = vec![
        NotePresetData { midi_note: 64, chance: 80, beat: 64, beat_length: 64 },
        NotePresetData { midi_note: 67, chance: 60, beat: 80, beat_length: 64 },
    ];
    data.synth_pll_volume = 0.7;
    data.synth_pll_track_speed = 0.4;
    data.synth_vol_attack = 1.0;
    data.synth_vol_decay = 60.0;
    data.synth_vol_sustain = 0.2;
    data.synth_vol_release = 100.0;
    Preset::with_data("Offbeat", data)
}

fn create_shuffle() -> Preset {
    let mut data = PresetData::default();
    data.triplet_1_8t = [127.0, 0.0, 100.0, 127.0, 0.0, 100.0, 127.0, 0.0, 100.0, 127.0, 0.0, 100.0];
    data.root_note = 48;
    data.synth_pll_volume = 0.75;
    data.synth_pll_track_speed = 0.55;
    data.synth_vol_attack = 3.0;
    data.synth_vol_decay = 70.0;
    data.synth_vol_sustain = 0.35;
    data.synth_vol_release = 120.0;
    Preset::with_data("Shuffle", data)
}

fn create_breakbeat() -> Preset {
    let mut data = PresetData::default();
    data.straight_1_16 = [
        127.0, 0.0, 60.0, 0.0, 0.0, 127.0, 0.0, 80.0,
        127.0, 0.0, 0.0, 60.0, 0.0, 127.0, 60.0, 0.0
    ];
    data.root_note = 36;
    data.notes = vec![
        NotePresetData { midi_note: 38, chance: 100, beat: 90, beat_length: 80 },
        NotePresetData { midi_note: 42, chance: 70, beat: 50, beat_length: 100 },
    ];
    data.synth_pll_volume = 0.8;
    data.synth_pll_track_speed = 0.7;
    data.synth_vol_attack = 1.0;
    data.synth_vol_decay = 40.0;
    data.synth_vol_sustain = 0.2;
    data.synth_vol_release = 80.0;
    Preset::with_data("Breakbeat", data)
}

fn create_halftime() -> Preset {
    let mut data = PresetData::default();
    data.straight_1_2 = [127.0, 100.0];
    data.root_note = 36;
    data.synth_pll_volume = 0.85;
    data.synth_pll_track_speed = 0.3;
    data.synth_vol_attack = 10.0;
    data.synth_vol_decay = 200.0;
    data.synth_vol_sustain = 0.5;
    data.synth_vol_release = 300.0;
    Preset::with_data("Halftime", data)
}

fn create_syncopated() -> Preset {
    let mut data = PresetData::default();
    data.straight_1_16 = [
        127.0, 0.0, 0.0, 100.0, 0.0, 0.0, 127.0, 0.0,
        0.0, 100.0, 0.0, 0.0, 127.0, 0.0, 100.0, 0.0
    ];
    data.root_note = 48;
    data.synth_pll_volume = 0.7;
    data.synth_pll_track_speed = 0.5;
    data.synth_vol_attack = 2.0;
    data.synth_vol_decay = 60.0;
    data.synth_vol_sustain = 0.3;
    data.synth_vol_release = 100.0;
    Preset::with_data("Syncopated", data)
}

fn create_polyrhythm() -> Preset {
    let mut data = PresetData::default();
    data.straight_1_4 = [127.0, 80.0, 127.0, 80.0];
    data.triplet_1_4t = [100.0, 70.0, 100.0, 70.0, 100.0, 70.0];
    data.root_note = 48;
    data.notes = vec![
        NotePresetData { midi_note: 55, chance: 80, beat: 40, beat_length: 64 },
    ];
    data.synth_pll_volume = 0.65;
    data.synth_pll_track_speed = 0.45;
    data.synth_vol_attack = 5.0;
    data.synth_vol_decay = 100.0;
    data.synth_vol_sustain = 0.4;
    data.synth_vol_release = 150.0;
    Preset::with_data("Polyrhythm", data)
}

fn create_triplet_groove() -> Preset {
    let mut data = PresetData::default();
    data.triplet_1_4t = [127.0, 100.0, 80.0, 127.0, 100.0, 80.0];
    data.root_note = 48;
    data.synth_pll_volume = 0.75;
    data.synth_pll_track_speed = 0.5;
    data.synth_vol_attack = 3.0;
    data.synth_vol_decay = 80.0;
    data.synth_vol_sustain = 0.35;
    data.synth_vol_release = 120.0;
    Preset::with_data("Triplet Groove", data)
}

fn create_dotted_bounce() -> Preset {
    let mut data = PresetData::default();
    data.dotted_1_8d = [127.0, 90.0, 127.0, 90.0, 127.0, 90.0];
    data.root_note = 52;
    data.synth_pll_volume = 0.7;
    data.synth_pll_track_speed = 0.55;
    data.synth_vol_attack = 2.0;
    data.synth_vol_decay = 70.0;
    data.synth_vol_sustain = 0.3;
    data.synth_vol_release = 100.0;
    Preset::with_data("Dotted Bounce", data)
}

fn create_sparse_minimal() -> Preset {
    let mut data = PresetData::default();
    data.straight_1_4 = [127.0, 0.0, 60.0, 0.0];
    data.root_note = 36;
    data.synth_pll_volume = 0.9;
    data.synth_pll_track_speed = 0.3;
    data.synth_reverb_mix = 0.3;
    data.synth_vol_attack = 5.0;
    data.synth_vol_decay = 150.0;
    data.synth_vol_sustain = 0.3;
    data.synth_vol_release = 400.0;
    Preset::with_data("Sparse Minimal", data)
}

fn create_busy_16th() -> Preset {
    let mut data = PresetData::default();
    data.straight_1_16 = [
        127.0, 80.0, 100.0, 80.0, 127.0, 80.0, 100.0, 80.0,
        127.0, 80.0, 100.0, 80.0, 127.0, 80.0, 100.0, 80.0
    ];
    data.root_note = 48;
    data.synth_pll_volume = 0.6;
    data.synth_pll_track_speed = 0.6;
    data.synth_vol_attack = 1.0;
    data.synth_vol_decay = 30.0;
    data.synth_vol_sustain = 0.2;
    data.synth_vol_release = 50.0;
    Preset::with_data("Busy 16th", data)
}

fn create_kick_snare() -> Preset {
    let mut data = PresetData::default();
    data.straight_1_4 = [127.0, 0.0, 127.0, 0.0];
    data.straight_1_8 = [0.0, 0.0, 0.0, 127.0, 0.0, 0.0, 0.0, 127.0];
    data.root_note = 36;
    data.notes = vec![
        NotePresetData { midi_note: 38, chance: 127, beat: 100, beat_length: 80 },
    ];
    data.synth_pll_volume = 0.85;
    data.synth_pll_track_speed = 0.7;
    data.synth_vol_attack = 1.0;
    data.synth_vol_decay = 50.0;
    data.synth_vol_sustain = 0.15;
    data.synth_vol_release = 80.0;
    Preset::with_data("Kick Snare", data)
}

fn create_rolling_hats() -> Preset {
    let mut data = PresetData::default();
    data.straight_1_32 = [
        127.0, 40.0, 80.0, 40.0, 100.0, 40.0, 80.0, 40.0,
        127.0, 40.0, 80.0, 40.0, 100.0, 40.0, 80.0, 40.0,
        127.0, 40.0, 80.0, 40.0, 100.0, 40.0, 80.0, 40.0,
        127.0, 40.0, 80.0, 40.0, 100.0, 40.0, 80.0, 40.0
    ];
    data.root_note = 66;
    data.synth_pll_volume = 0.5;
    data.synth_pll_track_speed = 0.8;
    data.synth_vol_attack = 0.5;
    data.synth_vol_decay = 20.0;
    data.synth_vol_sustain = 0.1;
    data.synth_vol_release = 30.0;
    Preset::with_data("Rolling Hats", data)
}

fn create_swing_feel() -> Preset {
    let mut data = PresetData::default();
    data.triplet_1_8t = [127.0, 0.0, 90.0, 127.0, 0.0, 90.0, 127.0, 0.0, 90.0, 127.0, 0.0, 90.0];
    data.root_note = 48;
    data.notes = vec![
        NotePresetData { midi_note: 52, chance: 70, beat: 70, beat_length: 50 },
        NotePresetData { midi_note: 55, chance: 50, beat: 50, beat_length: 70 },
    ];
    data.synth_pll_volume = 0.7;
    data.synth_pll_track_speed = 0.5;
    data.synth_vol_attack = 3.0;
    data.synth_vol_decay = 80.0;
    data.synth_vol_sustain = 0.35;
    data.synth_vol_release = 120.0;
    Preset::with_data("Swing Feel", data)
}

fn create_random_hits() -> Preset {
    let mut data = PresetData::default();
    data.straight_1_16 = [
        80.0, 40.0, 60.0, 30.0, 70.0, 50.0, 40.0, 60.0,
        80.0, 30.0, 50.0, 40.0, 60.0, 50.0, 40.0, 70.0
    ];
    data.root_note = 48;
    data.notes = vec![
        NotePresetData { midi_note: 50, chance: 80, beat: 64, beat_length: 64 },
        NotePresetData { midi_note: 52, chance: 60, beat: 64, beat_length: 64 },
        NotePresetData { midi_note: 55, chance: 70, beat: 64, beat_length: 64 },
        NotePresetData { midi_note: 57, chance: 50, beat: 64, beat_length: 64 },
    ];
    data.synth_pll_volume = 0.65;
    data.synth_pll_track_speed = 0.5;
    data.synth_vol_attack = 2.0;
    data.synth_vol_decay = 60.0;
    data.synth_vol_sustain = 0.3;
    data.synth_vol_release = 100.0;
    Preset::with_data("Random Hits", data)
}

fn create_techno_driving() -> Preset {
    let mut data = PresetData::default();
    data.straight_1_4 = [127.0, 127.0, 127.0, 127.0];
    data.straight_1_16 = [
        40.0, 0.0, 60.0, 0.0, 40.0, 0.0, 60.0, 0.0,
        40.0, 0.0, 80.0, 0.0, 40.0, 0.0, 60.0, 0.0
    ];
    data.root_note = 36;
    data.synth_pll_volume = 0.85;
    data.synth_pll_track_speed = 0.75;
    data.synth_pll_damping = 0.2;
    data.synth_filter_enable = true;
    data.synth_filter_cutoff = 2000.0;
    data.synth_filter_resonance = 0.3;
    data.synth_vol_attack = 1.0;
    data.synth_vol_decay = 40.0;
    data.synth_vol_sustain = 0.2;
    data.synth_vol_release = 60.0;
    Preset::with_data("Techno Driving", data)
}

fn create_house_classic() -> Preset {
    let mut data = PresetData::default();
    data.straight_1_4 = [127.0, 127.0, 127.0, 127.0];
    data.straight_1_8 = [0.0, 100.0, 0.0, 100.0, 0.0, 100.0, 0.0, 100.0];
    data.root_note = 36;
    data.notes = vec![
        NotePresetData { midi_note: 42, chance: 100, beat: 30, beat_length: 100 },
    ];
    data.synth_pll_volume = 0.8;
    data.synth_pll_track_speed = 0.5;
    data.synth_reverb_mix = 0.15;
    data.synth_vol_attack = 2.0;
    data.synth_vol_decay = 60.0;
    data.synth_vol_sustain = 0.25;
    data.synth_vol_release = 100.0;
    Preset::with_data("House Classic", data)
}

fn create_minimal_techno() -> Preset {
    let mut data = PresetData::default();
    data.straight_1_4 = [127.0, 80.0, 127.0, 80.0];
    data.straight_1_16 = [
        0.0, 0.0, 50.0, 0.0, 0.0, 0.0, 50.0, 0.0,
        0.0, 0.0, 50.0, 0.0, 0.0, 0.0, 70.0, 0.0
    ];
    data.root_note = 36;
    data.synth_pll_volume = 0.75;
    data.synth_pll_track_speed = 0.6;
    data.synth_vol_attack = 2.0;
    data.synth_vol_decay = 50.0;
    data.synth_vol_sustain = 0.2;
    data.synth_vol_release = 80.0;
    Preset::with_data("Minimal Techno", data)
}

fn create_acid_line() -> Preset {
    let mut data = PresetData::default();
    data.straight_1_16 = [
        127.0, 60.0, 100.0, 60.0, 127.0, 60.0, 100.0, 60.0,
        127.0, 60.0, 100.0, 60.0, 127.0, 80.0, 100.0, 80.0
    ];
    data.root_note = 36;
    data.notes = vec![
        NotePresetData { midi_note: 38, chance: 80, beat: 40, beat_length: 80 },
        NotePresetData { midi_note: 41, chance: 70, beat: 50, beat_length: 70 },
        NotePresetData { midi_note: 43, chance: 90, beat: 80, beat_length: 60 },
        NotePresetData { midi_note: 48, chance: 60, beat: 100, beat_length: 50 },
    ];
    data.synth_pll_volume = 0.8;
    data.synth_pll_track_speed = 0.65;
    data.synth_filter_enable = true;
    data.synth_filter_cutoff = 800.0;
    data.synth_filter_resonance = 0.6;
    data.synth_filter_env_amount = 3000.0;
    data.synth_filt_attack = 5.0;
    data.synth_filt_decay = 150.0;
    data.synth_filt_sustain = 0.2;
    data.synth_vol_attack = 1.0;
    data.synth_vol_decay = 30.0;
    data.synth_vol_sustain = 0.3;
    data.synth_vol_release = 50.0;
    Preset::with_data("Acid Line", data)
}

fn create_trance_gate() -> Preset {
    let mut data = PresetData::default();
    data.straight_1_16 = [
        127.0, 127.0, 0.0, 0.0, 127.0, 127.0, 0.0, 0.0,
        127.0, 127.0, 0.0, 0.0, 127.0, 127.0, 127.0, 0.0
    ];
    data.root_note = 48;
    data.notes = vec![
        NotePresetData { midi_note: 52, chance: 100, beat: 64, beat_length: 64 },
        NotePresetData { midi_note: 55, chance: 100, beat: 64, beat_length: 64 },
        NotePresetData { midi_note: 60, chance: 80, beat: 64, beat_length: 64 },
    ];
    data.synth_pll_volume = 0.7;
    data.synth_pll_track_speed = 0.4;
    data.synth_reverb_mix = 0.25;
    data.synth_vol_attack = 1.0;
    data.synth_vol_decay = 20.0;
    data.synth_vol_sustain = 0.8;
    data.synth_vol_release = 30.0;
    Preset::with_data("Trance Gate", data)
}

fn create_industrial() -> Preset {
    let mut data = PresetData::default();
    data.straight_1_8 = [127.0, 100.0, 127.0, 100.0, 127.0, 100.0, 127.0, 100.0];
    data.straight_1_16 = [
        0.0, 60.0, 0.0, 60.0, 0.0, 60.0, 0.0, 60.0,
        0.0, 80.0, 0.0, 60.0, 0.0, 80.0, 0.0, 60.0
    ];
    data.root_note = 36;
    data.synth_pll_volume = 0.9;
    data.synth_pll_track_speed = 0.85;
    data.synth_pll_damping = 0.1;
    data.synth_pll_distortion_amount = 0.4;
    data.synth_vol_attack = 0.5;
    data.synth_vol_decay = 30.0;
    data.synth_vol_sustain = 0.3;
    data.synth_vol_release = 40.0;
    Preset::with_data("Industrial", data)
}

fn create_detroit() -> Preset {
    let mut data = PresetData::default();
    data.straight_1_4 = [127.0, 127.0, 127.0, 127.0];
    data.straight_1_16 = [
        0.0, 0.0, 80.0, 0.0, 0.0, 0.0, 80.0, 0.0,
        0.0, 0.0, 80.0, 0.0, 0.0, 0.0, 80.0, 60.0
    ];
    data.root_note = 36;
    data.notes = vec![
        NotePresetData { midi_note: 48, chance: 70, beat: 64, beat_length: 64 },
        NotePresetData { midi_note: 60, chance: 50, beat: 80, beat_length: 50 },
    ];
    data.synth_pll_volume = 0.75;
    data.synth_pll_track_speed = 0.55;
    data.synth_reverb_mix = 0.2;
    data.synth_vol_attack = 3.0;
    data.synth_vol_decay = 80.0;
    data.synth_vol_sustain = 0.3;
    data.synth_vol_release = 150.0;
    Preset::with_data("Detroit", data)
}

fn create_berlin() -> Preset {
    let mut data = PresetData::default();
    data.straight_1_4 = [127.0, 127.0, 127.0, 127.0];
    data.straight_1_8 = [0.0, 90.0, 0.0, 90.0, 0.0, 90.0, 0.0, 90.0];
    data.triplet_1_16t = [
        60.0, 0.0, 0.0, 60.0, 0.0, 0.0, 60.0, 0.0, 0.0, 60.0, 0.0, 0.0,
        60.0, 0.0, 0.0, 60.0, 0.0, 0.0, 60.0, 0.0, 0.0, 80.0, 0.0, 0.0
    ];
    data.root_note = 36;
    data.synth_pll_volume = 0.8;
    data.synth_pll_track_speed = 0.7;
    data.synth_pll_damping = 0.25;
    data.synth_vol_attack = 1.0;
    data.synth_vol_decay = 45.0;
    data.synth_vol_sustain = 0.25;
    data.synth_vol_release = 70.0;
    Preset::with_data("Berlin", data)
}

fn create_dub_techno() -> Preset {
    let mut data = PresetData::default();
    data.straight_1_4 = [127.0, 100.0, 127.0, 100.0];
    data.straight_1_8 = [0.0, 80.0, 0.0, 80.0, 0.0, 80.0, 0.0, 80.0];
    data.root_note = 36;
    data.notes = vec![
        NotePresetData { midi_note: 48, chance: 60, beat: 40, beat_length: 30 },
    ];
    data.synth_pll_volume = 0.7;
    data.synth_pll_track_speed = 0.4;
    data.synth_reverb_mix = 0.4;
    data.synth_reverb_decay = 0.7;
    data.synth_reverb_time_scale = 0.7;
    data.synth_vol_attack = 5.0;
    data.synth_vol_decay = 100.0;
    data.synth_vol_sustain = 0.3;
    data.synth_vol_release = 300.0;
    Preset::with_data("Dub Techno", data)
}

fn create_hard_techno() -> Preset {
    let mut data = PresetData::default();
    data.straight_1_4 = [127.0, 127.0, 127.0, 127.0];
    data.straight_1_8 = [80.0, 80.0, 80.0, 80.0, 80.0, 80.0, 80.0, 80.0];
    data.straight_1_16 = [
        0.0, 50.0, 0.0, 50.0, 0.0, 50.0, 0.0, 50.0,
        0.0, 50.0, 0.0, 50.0, 0.0, 70.0, 0.0, 70.0
    ];
    data.root_note = 36;
    data.synth_pll_volume = 0.95;
    data.synth_pll_track_speed = 0.9;
    data.synth_pll_damping = 0.1;
    data.synth_pll_distortion_amount = 0.3;
    data.synth_vol_attack = 0.5;
    data.synth_vol_decay = 25.0;
    data.synth_vol_sustain = 0.2;
    data.synth_vol_release = 40.0;
    Preset::with_data("Hard Techno", data)
}

fn create_deep_house() -> Preset {
    let mut data = PresetData::default();
    data.straight_1_4 = [127.0, 100.0, 127.0, 100.0];
    data.straight_1_8 = [0.0, 70.0, 0.0, 70.0, 0.0, 70.0, 0.0, 70.0];
    data.root_note = 36;
    data.notes = vec![
        NotePresetData { midi_note: 48, chance: 50, beat: 50, beat_length: 40 },
        NotePresetData { midi_note: 55, chance: 40, beat: 40, beat_length: 50 },
    ];
    data.synth_pll_volume = 0.7;
    data.synth_pll_track_speed = 0.4;
    data.synth_filter_enable = true;
    data.synth_filter_cutoff = 1500.0;
    data.synth_filter_resonance = 0.2;
    data.synth_reverb_mix = 0.2;
    data.synth_vol_attack = 5.0;
    data.synth_vol_decay = 120.0;
    data.synth_vol_sustain = 0.35;
    data.synth_vol_release = 200.0;
    Preset::with_data("Deep House", data)
}

fn create_progressive() -> Preset {
    let mut data = PresetData::default();
    data.straight_1_4 = [127.0, 80.0, 127.0, 80.0];
    data.straight_1_16 = [
        0.0, 0.0, 50.0, 0.0, 0.0, 0.0, 50.0, 0.0,
        0.0, 0.0, 50.0, 0.0, 0.0, 0.0, 70.0, 50.0
    ];
    data.root_note = 48;
    data.notes = vec![
        NotePresetData { midi_note: 52, chance: 70, beat: 64, beat_length: 50 },
        NotePresetData { midi_note: 55, chance: 60, beat: 64, beat_length: 50 },
        NotePresetData { midi_note: 60, chance: 50, beat: 80, beat_length: 40 },
    ];
    data.synth_pll_volume = 0.65;
    data.synth_pll_track_speed = 0.45;
    data.synth_reverb_mix = 0.3;
    data.synth_vol_attack = 10.0;
    data.synth_vol_decay = 150.0;
    data.synth_vol_sustain = 0.4;
    data.synth_vol_release = 250.0;
    Preset::with_data("Progressive", data)
}

fn create_electro() -> Preset {
    let mut data = PresetData::default();
    data.straight_1_8 = [127.0, 80.0, 127.0, 80.0, 127.0, 80.0, 127.0, 80.0];
    data.straight_1_16 = [
        0.0, 50.0, 0.0, 50.0, 0.0, 50.0, 0.0, 50.0,
        0.0, 50.0, 0.0, 50.0, 0.0, 70.0, 0.0, 70.0
    ];
    data.root_note = 36;
    data.synth_pll_volume = 0.85;
    data.synth_pll_track_speed = 0.7;
    data.synth_pll_distortion_amount = 0.2;
    data.synth_vol_attack = 1.0;
    data.synth_vol_decay = 35.0;
    data.synth_vol_sustain = 0.25;
    data.synth_vol_release = 60.0;
    Preset::with_data("Electro", data)
}

fn create_ebm() -> Preset {
    let mut data = PresetData::default();
    data.straight_1_8 = [127.0, 127.0, 127.0, 127.0, 127.0, 127.0, 127.0, 127.0];
    data.root_note = 36;
    data.notes = vec![
        NotePresetData { midi_note: 38, chance: 100, beat: 80, beat_length: 80 },
        NotePresetData { midi_note: 41, chance: 80, beat: 60, beat_length: 70 },
    ];
    data.synth_pll_volume = 0.9;
    data.synth_pll_track_speed = 0.8;
    data.synth_pll_damping = 0.15;
    data.synth_vol_attack = 0.5;
    data.synth_vol_decay = 30.0;
    data.synth_vol_sustain = 0.3;
    data.synth_vol_release = 50.0;
    Preset::with_data("EBM", data)
}

fn create_dark_wave() -> Preset {
    let mut data = PresetData::default();
    data.straight_1_4 = [127.0, 90.0, 127.0, 90.0];
    data.straight_1_8 = [0.0, 60.0, 0.0, 60.0, 0.0, 60.0, 0.0, 60.0];
    data.root_note = 36;
    data.notes = vec![
        NotePresetData { midi_note: 43, chance: 80, beat: 64, beat_length: 40 },
        NotePresetData { midi_note: 48, chance: 60, beat: 50, beat_length: 50 },
    ];
    data.synth_pll_volume = 0.75;
    data.synth_pll_track_speed = 0.5;
    data.synth_reverb_mix = 0.35;
    data.synth_reverb_decay = 0.6;
    data.synth_filter_enable = true;
    data.synth_filter_cutoff = 1200.0;
    data.synth_filter_resonance = 0.3;
    data.synth_vol_attack = 5.0;
    data.synth_vol_decay = 100.0;
    data.synth_vol_sustain = 0.4;
    data.synth_vol_release = 200.0;
    Preset::with_data("Dark Wave", data)
}

fn create_synth_pop() -> Preset {
    let mut data = PresetData::default();
    data.straight_1_4 = [127.0, 100.0, 127.0, 100.0];
    data.straight_1_8 = [60.0, 60.0, 60.0, 60.0, 60.0, 60.0, 60.0, 60.0];
    data.root_note = 48;
    data.notes = vec![
        NotePresetData { midi_note: 52, chance: 90, beat: 64, beat_length: 50 },
        NotePresetData { midi_note: 55, chance: 80, beat: 64, beat_length: 50 },
        NotePresetData { midi_note: 60, chance: 70, beat: 80, beat_length: 40 },
    ];
    data.synth_pll_volume = 0.65;
    data.synth_pll_track_speed = 0.45;
    data.synth_reverb_mix = 0.2;
    data.synth_vol_attack = 5.0;
    data.synth_vol_decay = 80.0;
    data.synth_vol_sustain = 0.5;
    data.synth_vol_release = 150.0;
    Preset::with_data("Synth Pop", data)
}

fn create_glitch_stutter() -> Preset {
    let mut data = PresetData::default();
    data.straight_1_32 = [
        127.0, 127.0, 0.0, 0.0, 100.0, 0.0, 0.0, 0.0,
        127.0, 0.0, 100.0, 100.0, 0.0, 0.0, 80.0, 0.0,
        127.0, 127.0, 127.0, 0.0, 0.0, 0.0, 100.0, 0.0,
        0.0, 80.0, 0.0, 80.0, 127.0, 0.0, 0.0, 60.0
    ];
    data.root_note = 48;
    data.notes = vec![
        NotePresetData { midi_note: 50, chance: 70, beat: 64, beat_length: 100 },
        NotePresetData { midi_note: 53, chance: 50, beat: 64, beat_length: 100 },
    ];
    data.synth_pll_volume = 0.7;
    data.synth_pll_track_speed = 0.6;
    data.synth_vol_attack = 0.5;
    data.synth_vol_decay = 15.0;
    data.synth_vol_sustain = 0.2;
    data.synth_vol_release = 20.0;
    Preset::with_data("Glitch Stutter", data)
}

fn create_idm_complex() -> Preset {
    let mut data = PresetData::default();
    data.straight_1_16 = [
        127.0, 0.0, 60.0, 0.0, 0.0, 80.0, 0.0, 50.0,
        100.0, 0.0, 0.0, 70.0, 0.0, 60.0, 90.0, 0.0
    ];
    data.triplet_1_8t = [80.0, 0.0, 60.0, 80.0, 0.0, 60.0, 80.0, 0.0, 60.0, 80.0, 0.0, 70.0];
    data.root_note = 48;
    data.notes = vec![
        NotePresetData { midi_note: 51, chance: 60, beat: 40, beat_length: 80 },
        NotePresetData { midi_note: 55, chance: 70, beat: 80, beat_length: 60 },
        NotePresetData { midi_note: 58, chance: 50, beat: 50, beat_length: 70 },
    ];
    data.synth_pll_volume = 0.65;
    data.synth_pll_track_speed = 0.55;
    data.synth_pll_damping = 0.4;
    data.synth_vol_attack = 2.0;
    data.synth_vol_decay = 50.0;
    data.synth_vol_sustain = 0.25;
    data.synth_vol_release = 80.0;
    Preset::with_data("IDM Complex", data)
}

fn create_broken_beat() -> Preset {
    let mut data = PresetData::default();
    data.straight_1_16 = [
        127.0, 0.0, 0.0, 80.0, 0.0, 127.0, 0.0, 0.0,
        80.0, 0.0, 0.0, 127.0, 0.0, 80.0, 0.0, 60.0
    ];
    data.root_note = 48;
    data.notes = vec![
        NotePresetData { midi_note: 52, chance: 70, beat: 50, beat_length: 64 },
        NotePresetData { midi_note: 55, chance: 60, beat: 80, beat_length: 64 },
    ];
    data.synth_pll_volume = 0.7;
    data.synth_pll_track_speed = 0.5;
    data.synth_vol_attack = 3.0;
    data.synth_vol_decay = 70.0;
    data.synth_vol_sustain = 0.3;
    data.synth_vol_release = 100.0;
    Preset::with_data("Broken Beat", data)
}

fn create_micro_rhythm() -> Preset {
    let mut data = PresetData::default();
    data.straight_1_32 = [
        100.0, 30.0, 50.0, 30.0, 80.0, 30.0, 50.0, 30.0,
        100.0, 30.0, 50.0, 30.0, 80.0, 30.0, 60.0, 40.0,
        100.0, 30.0, 50.0, 30.0, 80.0, 30.0, 50.0, 30.0,
        100.0, 30.0, 50.0, 40.0, 80.0, 40.0, 70.0, 50.0
    ];
    data.root_note = 60;
    data.synth_pll_volume = 0.55;
    data.synth_pll_track_speed = 0.65;
    data.synth_vol_attack = 0.5;
    data.synth_vol_decay = 20.0;
    data.synth_vol_sustain = 0.15;
    data.synth_vol_release = 30.0;
    Preset::with_data("Micro Rhythm", data)
}

fn create_generative() -> Preset {
    let mut data = PresetData::default();
    data.straight_1_16 = [
        60.0, 30.0, 40.0, 20.0, 50.0, 25.0, 35.0, 15.0,
        55.0, 28.0, 38.0, 18.0, 45.0, 22.0, 32.0, 12.0
    ];
    data.triplet_1_8t = [50.0, 20.0, 30.0, 50.0, 20.0, 30.0, 50.0, 20.0, 30.0, 50.0, 20.0, 40.0];
    data.root_note = 48;
    data.notes = vec![
        NotePresetData { midi_note: 50, chance: 60, beat: 64, beat_length: 64 },
        NotePresetData { midi_note: 52, chance: 50, beat: 64, beat_length: 64 },
        NotePresetData { midi_note: 55, chance: 70, beat: 64, beat_length: 64 },
        NotePresetData { midi_note: 57, chance: 40, beat: 64, beat_length: 64 },
        NotePresetData { midi_note: 60, chance: 55, beat: 64, beat_length: 64 },
    ];
    data.synth_pll_volume = 0.6;
    data.synth_pll_track_speed = 0.5;
    data.synth_reverb_mix = 0.25;
    data.synth_vol_attack = 5.0;
    data.synth_vol_decay = 80.0;
    data.synth_vol_sustain = 0.3;
    data.synth_vol_release = 150.0;
    Preset::with_data("Generative", data)
}

fn create_euclidean_5() -> Preset {
    let mut data = PresetData::default();
    data.straight_1_16 = [
        127.0, 0.0, 0.0, 127.0, 0.0, 0.0, 127.0, 0.0,
        0.0, 127.0, 0.0, 0.0, 127.0, 0.0, 0.0, 0.0
    ];
    data.root_note = 48;
    data.synth_pll_volume = 0.7;
    data.synth_pll_track_speed = 0.5;
    data.synth_vol_attack = 2.0;
    data.synth_vol_decay = 60.0;
    data.synth_vol_sustain = 0.3;
    data.synth_vol_release = 100.0;
    Preset::with_data("Euclidean 5", data)
}

fn create_euclidean_7() -> Preset {
    let mut data = PresetData::default();
    data.straight_1_16 = [
        127.0, 0.0, 127.0, 0.0, 127.0, 0.0, 0.0, 127.0,
        0.0, 127.0, 0.0, 127.0, 0.0, 0.0, 127.0, 0.0
    ];
    data.root_note = 48;
    data.synth_pll_volume = 0.7;
    data.synth_pll_track_speed = 0.5;
    data.synth_vol_attack = 2.0;
    data.synth_vol_decay = 60.0;
    data.synth_vol_sustain = 0.3;
    data.synth_vol_release = 100.0;
    Preset::with_data("Euclidean 7", data)
}

fn create_euclidean_11() -> Preset {
    let mut data = PresetData::default();
    data.straight_1_16 = [
        127.0, 0.0, 127.0, 127.0, 0.0, 127.0, 0.0, 127.0,
        127.0, 0.0, 127.0, 0.0, 127.0, 127.0, 0.0, 127.0
    ];
    data.root_note = 48;
    data.synth_pll_volume = 0.7;
    data.synth_pll_track_speed = 0.5;
    data.synth_vol_attack = 2.0;
    data.synth_vol_decay = 60.0;
    data.synth_vol_sustain = 0.3;
    data.synth_vol_release = 100.0;
    Preset::with_data("Euclidean 11", data)
}

fn create_noise_burst() -> Preset {
    let mut data = PresetData::default();
    data.straight_1_32 = [
        127.0, 100.0, 80.0, 60.0, 40.0, 20.0, 0.0, 0.0,
        0.0, 0.0, 0.0, 0.0, 127.0, 100.0, 80.0, 60.0,
        40.0, 20.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
        127.0, 127.0, 100.0, 80.0, 60.0, 40.0, 20.0, 0.0
    ];
    data.root_note = 72;
    data.synth_pll_volume = 0.6;
    data.synth_pll_track_speed = 0.9;
    data.synth_pll_damping = 0.05;
    data.synth_vol_attack = 0.1;
    data.synth_vol_decay = 10.0;
    data.synth_vol_sustain = 0.1;
    data.synth_vol_release = 15.0;
    Preset::with_data("Noise Burst", data)
}

fn create_granular_feel() -> Preset {
    let mut data = PresetData::default();
    data.straight_1_32 = [
        80.0, 0.0, 60.0, 0.0, 80.0, 0.0, 40.0, 0.0,
        80.0, 0.0, 60.0, 0.0, 80.0, 0.0, 50.0, 0.0,
        80.0, 0.0, 70.0, 0.0, 80.0, 0.0, 45.0, 0.0,
        80.0, 0.0, 55.0, 0.0, 80.0, 0.0, 65.0, 0.0
    ];
    data.root_note = 60;
    data.notes = vec![
        NotePresetData { midi_note: 62, chance: 50, beat: 64, beat_length: 90 },
        NotePresetData { midi_note: 64, chance: 40, beat: 64, beat_length: 90 },
    ];
    data.synth_pll_volume = 0.55;
    data.synth_pll_track_speed = 0.7;
    data.synth_reverb_mix = 0.4;
    data.synth_vol_attack = 0.5;
    data.synth_vol_decay = 25.0;
    data.synth_vol_sustain = 0.1;
    data.synth_vol_release = 40.0;
    Preset::with_data("Granular Feel", data)
}

fn create_polymetric() -> Preset {
    let mut data = PresetData::default();
    data.straight_1_8 = [127.0, 0.0, 0.0, 127.0, 0.0, 0.0, 127.0, 0.0];
    data.triplet_1_4t = [100.0, 0.0, 100.0, 0.0, 100.0, 0.0];
    data.root_note = 48;
    data.notes = vec![
        NotePresetData { midi_note: 55, chance: 80, beat: 30, beat_length: 64 },
    ];
    data.synth_pll_volume = 0.65;
    data.synth_pll_track_speed = 0.5;
    data.synth_vol_attack = 3.0;
    data.synth_vol_decay = 70.0;
    data.synth_vol_sustain = 0.3;
    data.synth_vol_release = 120.0;
    Preset::with_data("Polymetric", data)
}

fn create_chaos_theory() -> Preset {
    let mut data = PresetData::default();
    data.straight_1_16 = [
        90.0, 20.0, 70.0, 10.0, 80.0, 30.0, 60.0, 5.0,
        85.0, 15.0, 75.0, 25.0, 65.0, 35.0, 55.0, 45.0
    ];
    data.triplet_1_8t = [70.0, 10.0, 50.0, 70.0, 10.0, 50.0, 70.0, 10.0, 50.0, 80.0, 20.0, 60.0];
    data.dotted_1_8d = [60.0, 40.0, 60.0, 40.0, 60.0, 40.0];
    data.root_note = 48;
    data.notes = vec![
        NotePresetData { midi_note: 50, chance: 50, beat: 64, beat_length: 64 },
        NotePresetData { midi_note: 53, chance: 60, beat: 64, beat_length: 64 },
        NotePresetData { midi_note: 55, chance: 70, beat: 64, beat_length: 64 },
        NotePresetData { midi_note: 58, chance: 40, beat: 64, beat_length: 64 },
    ];
    data.synth_pll_volume = 0.6;
    data.synth_pll_track_speed = 0.55;
    data.synth_vol_attack = 2.0;
    data.synth_vol_decay = 60.0;
    data.synth_vol_sustain = 0.25;
    data.synth_vol_release = 100.0;
    Preset::with_data("Chaos Theory", data)
}

fn create_stuttered_gate() -> Preset {
    let mut data = PresetData::default();
    data.straight_1_32 = [
        127.0, 127.0, 127.0, 0.0, 0.0, 0.0, 0.0, 0.0,
        127.0, 127.0, 0.0, 0.0, 0.0, 0.0, 127.0, 127.0,
        127.0, 0.0, 0.0, 0.0, 127.0, 127.0, 127.0, 127.0,
        0.0, 0.0, 0.0, 0.0, 127.0, 0.0, 127.0, 0.0
    ];
    data.root_note = 48;
    data.synth_pll_volume = 0.7;
    data.synth_pll_track_speed = 0.5;
    data.synth_vol_attack = 0.5;
    data.synth_vol_decay = 10.0;
    data.synth_vol_sustain = 0.8;
    data.synth_vol_release = 15.0;
    Preset::with_data("Stuttered Gate", data)
}

fn create_bit_crush() -> Preset {
    let mut data = PresetData::default();
    data.straight_1_8 = [127.0, 100.0, 127.0, 100.0, 127.0, 100.0, 127.0, 100.0];
    data.straight_1_16 = [
        0.0, 60.0, 0.0, 60.0, 0.0, 60.0, 0.0, 60.0,
        0.0, 60.0, 0.0, 60.0, 0.0, 80.0, 0.0, 80.0
    ];
    data.root_note = 36;
    data.synth_pll_volume = 0.85;
    data.synth_pll_track_speed = 0.85;
    data.synth_pll_damping = 0.1;
    data.synth_pll_distortion_amount = 0.5;
    data.synth_vol_attack = 0.5;
    data.synth_vol_decay = 25.0;
    data.synth_vol_sustain = 0.2;
    data.synth_vol_release = 40.0;
    Preset::with_data("Bit Crush", data)
}

fn create_circuit_bent() -> Preset {
    let mut data = PresetData::default();
    data.straight_1_16 = [
        127.0, 50.0, 80.0, 30.0, 100.0, 40.0, 70.0, 20.0,
        110.0, 60.0, 90.0, 35.0, 105.0, 45.0, 75.0, 25.0
    ];
    data.root_note = 48;
    data.notes = vec![
        NotePresetData { midi_note: 49, chance: 60, beat: 64, beat_length: 80 },
        NotePresetData { midi_note: 51, chance: 50, beat: 64, beat_length: 80 },
        NotePresetData { midi_note: 54, chance: 70, beat: 64, beat_length: 80 },
    ];
    data.synth_pll_volume = 0.75;
    data.synth_pll_track_speed = 0.75;
    data.synth_pll_damping = 0.2;
    data.synth_pll_distortion_amount = 0.3;
    data.synth_vol_attack = 1.0;
    data.synth_vol_decay = 40.0;
    data.synth_vol_sustain = 0.25;
    data.synth_vol_release = 60.0;
    Preset::with_data("Circuit Bent", data)
}

fn create_data_stream() -> Preset {
    let mut data = PresetData::default();
    data.straight_1_32 = [
        127.0, 60.0, 90.0, 45.0, 110.0, 55.0, 80.0, 40.0,
        120.0, 50.0, 85.0, 42.0, 105.0, 52.0, 75.0, 38.0,
        127.0, 58.0, 88.0, 44.0, 108.0, 54.0, 78.0, 39.0,
        118.0, 48.0, 83.0, 41.0, 102.0, 51.0, 73.0, 36.0
    ];
    data.root_note = 60;
    data.synth_pll_volume = 0.5;
    data.synth_pll_track_speed = 0.6;
    data.synth_vol_attack = 0.5;
    data.synth_vol_decay = 15.0;
    data.synth_vol_sustain = 0.15;
    data.synth_vol_release = 20.0;
    Preset::with_data("Data Stream", data)
}

fn create_ambient_pulse() -> Preset {
    let mut data = PresetData::default();
    data.straight_1_2 = [127.0, 60.0];
    data.root_note = 48;
    data.notes = vec![
        NotePresetData { midi_note: 52, chance: 50, beat: 40, beat_length: 20 },
        NotePresetData { midi_note: 55, chance: 40, beat: 30, beat_length: 20 },
    ];
    data.synth_pll_volume = 0.5;
    data.synth_pll_track_speed = 0.25;
    data.synth_reverb_mix = 0.5;
    data.synth_reverb_decay = 0.8;
    data.synth_reverb_time_scale = 0.8;
    data.synth_vol_attack = 50.0;
    data.synth_vol_decay = 300.0;
    data.synth_vol_sustain = 0.4;
    data.synth_vol_release = 500.0;
    Preset::with_data("Ambient Pulse", data)
}

fn create_drone_evolve() -> Preset {
    let mut data = PresetData::default();
    data.straight_1_1 = [127.0];
    data.root_note = 36;
    data.notes = vec![
        NotePresetData { midi_note: 43, chance: 30, beat: 30, beat_length: 10 },
        NotePresetData { midi_note: 48, chance: 40, beat: 40, beat_length: 10 },
    ];
    data.synth_pll_volume = 0.4;
    data.synth_pll_track_speed = 0.15;
    data.synth_reverb_mix = 0.6;
    data.synth_reverb_decay = 0.9;
    data.synth_reverb_time_scale = 0.9;
    data.synth_vol_attack = 200.0;
    data.synth_vol_decay = 500.0;
    data.synth_vol_sustain = 0.6;
    data.synth_vol_release = 800.0;
    Preset::with_data("Drone Evolve", data)
}

fn create_meditation() -> Preset {
    let mut data = PresetData::default();
    data.straight_1_1 = [80.0];
    data.straight_1_2 = [0.0, 60.0];
    data.root_note = 60;
    data.notes = vec![
        NotePresetData { midi_note: 64, chance: 40, beat: 30, beat_length: 15 },
        NotePresetData { midi_note: 67, chance: 35, beat: 30, beat_length: 15 },
    ];
    data.synth_pll_volume = 0.4;
    data.synth_pll_track_speed = 0.2;
    data.synth_reverb_mix = 0.55;
    data.synth_reverb_decay = 0.85;
    data.synth_vol_attack = 100.0;
    data.synth_vol_decay = 400.0;
    data.synth_vol_sustain = 0.5;
    data.synth_vol_release = 600.0;
    Preset::with_data("Meditation", data)
}

fn create_breath() -> Preset {
    let mut data = PresetData::default();
    data.straight_1_2 = [127.0, 80.0];
    data.root_note = 48;
    data.synth_pll_volume = 0.45;
    data.synth_pll_track_speed = 0.2;
    data.synth_reverb_mix = 0.45;
    data.synth_reverb_decay = 0.75;
    data.synth_vol_attack = 150.0;
    data.synth_vol_decay = 350.0;
    data.synth_vol_sustain = 0.3;
    data.synth_vol_release = 500.0;
    Preset::with_data("Breath", data)
}

fn create_sparse_bells() -> Preset {
    let mut data = PresetData::default();
    data.straight_1_4 = [127.0, 0.0, 50.0, 0.0];
    data.straight_1_8 = [0.0, 0.0, 0.0, 40.0, 0.0, 0.0, 0.0, 0.0];
    data.root_note = 72;
    data.notes = vec![
        NotePresetData { midi_note: 76, chance: 40, beat: 40, beat_length: 30 },
        NotePresetData { midi_note: 79, chance: 30, beat: 30, beat_length: 30 },
    ];
    data.synth_pll_volume = 0.5;
    data.synth_pll_track_speed = 0.3;
    data.synth_reverb_mix = 0.5;
    data.synth_reverb_decay = 0.8;
    data.synth_vol_attack = 5.0;
    data.synth_vol_decay = 200.0;
    data.synth_vol_sustain = 0.2;
    data.synth_vol_release = 400.0;
    Preset::with_data("Sparse Bells", data)
}

fn create_underwater() -> Preset {
    let mut data = PresetData::default();
    data.straight_1_4 = [100.0, 60.0, 80.0, 50.0];
    data.root_note = 36;
    data.notes = vec![
        NotePresetData { midi_note: 43, chance: 50, beat: 40, beat_length: 20 },
        NotePresetData { midi_note: 48, chance: 40, beat: 40, beat_length: 20 },
    ];
    data.synth_pll_volume = 0.45;
    data.synth_pll_track_speed = 0.25;
    data.synth_filter_enable = true;
    data.synth_filter_cutoff = 800.0;
    data.synth_filter_resonance = 0.2;
    data.synth_reverb_mix = 0.6;
    data.synth_reverb_decay = 0.85;
    data.synth_vol_attack = 30.0;
    data.synth_vol_decay = 250.0;
    data.synth_vol_sustain = 0.35;
    data.synth_vol_release = 400.0;
    Preset::with_data("Underwater", data)
}

fn create_space() -> Preset {
    let mut data = PresetData::default();
    data.straight_1_2 = [100.0, 40.0];
    data.straight_1_4 = [0.0, 50.0, 0.0, 30.0];
    data.root_note = 48;
    data.notes = vec![
        NotePresetData { midi_note: 55, chance: 35, beat: 30, beat_length: 15 },
        NotePresetData { midi_note: 60, chance: 30, beat: 30, beat_length: 15 },
        NotePresetData { midi_note: 67, chance: 25, beat: 30, beat_length: 15 },
    ];
    data.synth_pll_volume = 0.4;
    data.synth_pll_track_speed = 0.2;
    data.synth_reverb_mix = 0.65;
    data.synth_reverb_decay = 0.9;
    data.synth_reverb_time_scale = 0.85;
    data.synth_vol_attack = 80.0;
    data.synth_vol_decay = 350.0;
    data.synth_vol_sustain = 0.4;
    data.synth_vol_release = 600.0;
    Preset::with_data("Space", data)
}

fn create_forest() -> Preset {
    let mut data = PresetData::default();
    data.straight_1_8 = [80.0, 0.0, 50.0, 0.0, 60.0, 0.0, 40.0, 30.0];
    data.root_note = 60;
    data.notes = vec![
        NotePresetData { midi_note: 64, chance: 50, beat: 50, beat_length: 40 },
        NotePresetData { midi_note: 67, chance: 40, beat: 40, beat_length: 40 },
        NotePresetData { midi_note: 72, chance: 30, beat: 30, beat_length: 40 },
    ];
    data.synth_pll_volume = 0.5;
    data.synth_pll_track_speed = 0.3;
    data.synth_reverb_mix = 0.45;
    data.synth_reverb_decay = 0.7;
    data.synth_vol_attack = 20.0;
    data.synth_vol_decay = 180.0;
    data.synth_vol_sustain = 0.3;
    data.synth_vol_release = 350.0;
    Preset::with_data("Forest", data)
}

fn create_minimal_piano() -> Preset {
    let mut data = PresetData::default();
    data.straight_1_4 = [127.0, 0.0, 80.0, 0.0];
    data.straight_1_8 = [0.0, 0.0, 0.0, 50.0, 0.0, 0.0, 0.0, 40.0];
    data.root_note = 60;
    data.notes = vec![
        NotePresetData { midi_note: 64, chance: 60, beat: 64, beat_length: 50 },
        NotePresetData { midi_note: 67, chance: 50, beat: 64, beat_length: 50 },
    ];
    data.synth_pll_volume = 0.55;
    data.synth_pll_track_speed = 0.35;
    data.synth_reverb_mix = 0.3;
    data.synth_reverb_decay = 0.6;
    data.synth_vol_attack = 5.0;
    data.synth_vol_decay = 150.0;
    data.synth_vol_sustain = 0.3;
    data.synth_vol_release = 300.0;
    Preset::with_data("Minimal Piano", data)
}

fn create_slow_motion() -> Preset {
    let mut data = PresetData::default();
    data.straight_1_1 = [127.0];
    data.straight_1_2 = [0.0, 80.0];
    data.root_note = 48;
    data.notes = vec![
        NotePresetData { midi_note: 52, chance: 40, beat: 40, beat_length: 20 },
        NotePresetData { midi_note: 55, chance: 35, beat: 40, beat_length: 20 },
    ];
    data.synth_pll_volume = 0.45;
    data.synth_pll_track_speed = 0.2;
    data.synth_pll_glide = 500.0;
    data.synth_reverb_mix = 0.5;
    data.synth_reverb_decay = 0.8;
    data.synth_vol_attack = 100.0;
    data.synth_vol_decay = 400.0;
    data.synth_vol_sustain = 0.5;
    data.synth_vol_release = 600.0;
    Preset::with_data("Slow Motion", data)
}

fn create_dreamscape() -> Preset {
    let mut data = PresetData::default();
    data.straight_1_4 = [100.0, 50.0, 70.0, 40.0];
    data.root_note = 60;
    data.notes = vec![
        NotePresetData { midi_note: 64, chance: 50, beat: 40, beat_length: 20 },
        NotePresetData { midi_note: 67, chance: 45, beat: 35, beat_length: 20 },
        NotePresetData { midi_note: 72, chance: 35, beat: 30, beat_length: 20 },
    ];
    data.synth_pll_volume = 0.4;
    data.synth_pll_track_speed = 0.25;
    data.synth_reverb_mix = 0.55;
    data.synth_reverb_decay = 0.85;
    data.synth_reverb_time_scale = 0.8;
    data.synth_vol_attack = 60.0;
    data.synth_vol_decay = 300.0;
    data.synth_vol_sustain = 0.4;
    data.synth_vol_release = 500.0;
    Preset::with_data("Dreamscape", data)
}

fn create_whisper() -> Preset {
    let mut data = PresetData::default();
    data.straight_1_8 = [60.0, 0.0, 40.0, 0.0, 50.0, 0.0, 30.0, 0.0];
    data.root_note = 72;
    data.notes = vec![
        NotePresetData { midi_note: 76, chance: 40, beat: 40, beat_length: 30 },
        NotePresetData { midi_note: 79, chance: 30, beat: 30, beat_length: 30 },
    ];
    data.synth_pll_volume = 0.35;
    data.synth_pll_track_speed = 0.3;
    data.synth_reverb_mix = 0.5;
    data.synth_reverb_decay = 0.75;
    data.synth_vol_attack = 30.0;
    data.synth_vol_decay = 200.0;
    data.synth_vol_sustain = 0.25;
    data.synth_vol_release = 400.0;
    Preset::with_data("Whisper", data)
}

fn create_heartbeat() -> Preset {
    let mut data = PresetData::default();
    data.straight_1_4 = [127.0, 80.0, 0.0, 0.0];
    data.root_note = 36;
    data.synth_pll_volume = 0.6;
    data.synth_pll_track_speed = 0.35;
    data.synth_reverb_mix = 0.3;
    data.synth_reverb_decay = 0.5;
    data.synth_vol_attack = 10.0;
    data.synth_vol_decay = 100.0;
    data.synth_vol_sustain = 0.2;
    data.synth_vol_release = 200.0;
    Preset::with_data("Heartbeat", data)
}

fn create_time_stretch() -> Preset {
    let mut data = PresetData::default();
    data.straight_1_2 = [127.0, 60.0];
    data.dotted_1_4d = [0.0, 50.0, 40.0];
    data.root_note = 48;
    data.notes = vec![
        NotePresetData { midi_note: 52, chance: 40, beat: 40, beat_length: 20 },
        NotePresetData { midi_note: 55, chance: 35, beat: 35, beat_length: 20 },
    ];
    data.synth_pll_volume = 0.45;
    data.synth_pll_track_speed = 0.25;
    data.synth_pll_glide = 300.0;
    data.synth_reverb_mix = 0.5;
    data.synth_reverb_decay = 0.8;
    data.synth_vol_attack = 80.0;
    data.synth_vol_decay = 350.0;
    data.synth_vol_sustain = 0.4;
    data.synth_vol_release = 550.0;
    Preset::with_data("Time Stretch", data)
}

fn create_frozen() -> Preset {
    let mut data = PresetData::default();
    data.straight_1_1 = [127.0];
    data.root_note = 60;
    data.notes = vec![
        NotePresetData { midi_note: 64, chance: 30, beat: 30, beat_length: 10 },
        NotePresetData { midi_note: 67, chance: 25, beat: 30, beat_length: 10 },
    ];
    data.synth_pll_volume = 0.35;
    data.synth_pll_track_speed = 0.15;
    data.synth_reverb_mix = 0.7;
    data.synth_reverb_decay = 0.95;
    data.synth_reverb_time_scale = 0.9;
    data.synth_vol_attack = 150.0;
    data.synth_vol_decay = 500.0;
    data.synth_vol_sustain = 0.5;
    data.synth_vol_release = 800.0;
    Preset::with_data("Frozen", data)
}

fn create_void() -> Preset {
    let mut data = PresetData::default();
    data.straight_1_2 = [80.0, 0.0];
    data.straight_1_4 = [0.0, 40.0, 0.0, 30.0];
    data.root_note = 36;
    data.notes = vec![
        NotePresetData { midi_note: 43, chance: 30, beat: 30, beat_length: 10 },
        NotePresetData { midi_note: 48, chance: 25, beat: 25, beat_length: 10 },
    ];
    data.synth_pll_volume = 0.3;
    data.synth_pll_track_speed = 0.1;
    data.synth_filter_enable = true;
    data.synth_filter_cutoff = 500.0;
    data.synth_filter_resonance = 0.15;
    data.synth_reverb_mix = 0.75;
    data.synth_reverb_decay = 0.95;
    data.synth_reverb_time_scale = 0.95;
    data.synth_vol_attack = 200.0;
    data.synth_vol_decay = 600.0;
    data.synth_vol_sustain = 0.4;
    data.synth_vol_release = 900.0;
    Preset::with_data("Void", data)
}
