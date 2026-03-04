use crate::params::BeatMode;
use crate::sequencer::ml_suggest::{
    flat_index, normalize_beat_constraints, BeatSuggestion, SLOT_COUNT,
};
use rand::Rng;

// --- Groove templates ---
// Each pattern is normalized 0.0-1.0 (scaled to 0-127 later)
// Designed for monophonic synth sequencer: rhythmic motifs with breathing room

const EIGHTH_PATTERNS: [[f32; 8]; 8] = [
    // Pumping: four-on-floor synth pulse
    [1.0, 0.3, 0.8, 0.2, 1.0, 0.3, 0.7, 0.3],
    // Syncopated: offbeat emphasis
    [0.9, 0.0, 0.4, 0.8, 0.0, 0.7, 0.3, 0.0],
    // Driving: relentless with dynamics
    [1.0, 0.5, 0.8, 0.4, 0.9, 0.5, 0.7, 0.4],
    // Tresillo: 3+3+2 Cuban rhythm
    [1.0, 0.0, 0.0, 0.9, 0.0, 0.0, 0.8, 0.0],
    // Offbeat pulse: upbeat-heavy
    [0.3, 0.9, 0.2, 0.8, 0.3, 0.9, 0.2, 0.7],
    // Half-time: sparse, big hits
    [1.0, 0.0, 0.0, 0.0, 0.8, 0.0, 0.0, 0.3],
    // Call-response: active first half, sparse second
    [1.0, 0.6, 0.8, 0.5, 0.0, 0.0, 0.4, 0.0],
    // Reggae/dub: emphasis on 3
    [0.0, 0.0, 1.0, 0.0, 0.0, 0.3, 0.7, 0.0],
];

const SIXTEENTH_PATTERNS: [[f32; 16]; 6] = [
    // Funk: classic 16th groove
    [1.0, 0.0, 0.4, 0.0, 0.8, 0.0, 0.0, 0.5, 1.0, 0.0, 0.3, 0.0, 0.7, 0.0, 0.4, 0.0],
    // Rolling: continuous with dynamics
    [0.9, 0.4, 0.3, 0.5, 0.8, 0.3, 0.3, 0.4, 0.9, 0.4, 0.3, 0.5, 0.8, 0.3, 0.3, 0.4],
    // Stutter: clustered bursts
    [1.0, 0.8, 0.0, 0.0, 1.0, 0.7, 0.0, 0.0, 1.0, 0.8, 0.0, 0.0, 1.0, 0.7, 0.0, 0.0],
    // Broken: irregular clusters
    [1.0, 0.0, 0.6, 0.7, 0.0, 0.0, 0.0, 0.5, 0.0, 0.0, 0.8, 0.0, 0.6, 0.7, 0.0, 0.0],
    // Gallop: dotted-8th feel in 16ths
    [1.0, 0.0, 0.0, 0.7, 0.0, 0.0, 0.9, 0.0, 0.0, 0.6, 0.0, 0.0, 0.8, 0.0, 0.0, 0.5],
    // Sparse 16th: minimal with ghost notes
    [1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.7, 0.0, 0.0, 0.3, 0.0, 0.0, 0.0, 0.0],
];

const TRIPLET_6_PATTERNS: [[f32; 6]; 5] = [
    // Shuffle: classic swing feel
    [1.0, 0.0, 0.5, 0.9, 0.0, 0.4],
    // Waltz flow: strong 1, gentle motion
    [1.0, 0.4, 0.3, 0.7, 0.3, 0.2],
    // Afro: syncopated triplet
    [1.0, 0.0, 0.8, 0.0, 0.7, 0.0],
    // Rolling triplet: all active, varied dynamics
    [0.9, 0.5, 0.6, 0.8, 0.4, 0.5],
    // Sparse triplet: minimal
    [1.0, 0.0, 0.0, 0.0, 0.0, 0.6],
];

const TRIPLET_12_PATTERNS: [[f32; 12]; 4] = [
    // Detailed shuffle
    [1.0, 0.0, 0.4, 0.8, 0.0, 0.3, 0.9, 0.0, 0.5, 0.7, 0.0, 0.3],
    // Triplet funk
    [1.0, 0.3, 0.0, 0.7, 0.0, 0.5, 0.0, 0.4, 0.0, 0.8, 0.0, 0.3],
    // Rolling 12
    [0.8, 0.4, 0.5, 0.7, 0.3, 0.4, 0.8, 0.4, 0.5, 0.7, 0.3, 0.4],
    // Sparse 12
    [1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.7, 0.0, 0.0, 0.0, 0.5, 0.0],
];

const DOTTED_6_PATTERNS: [[f32; 6]; 3] = [
    // Dotted groove: uneven spacing creates tension
    [1.0, 0.7, 0.5, 0.9, 0.6, 0.4],
    // Sparse dotted
    [1.0, 0.0, 0.0, 0.8, 0.0, 0.0],
    // Dotted pulse
    [0.9, 0.5, 0.7, 0.4, 0.8, 0.3],
];

// --- Groove families ---

#[derive(Clone, Copy)]
enum GrooveFamily {
    Eighth,
    Sixteenth,
    Triplet,
    SparseLegato,
    Dotted,
    Polyrhythmic,
}

const GROOVE_WEIGHTS: [(GrooveFamily, f32); 6] = [
    (GrooveFamily::Eighth, 0.25),
    (GrooveFamily::Sixteenth, 0.20),
    (GrooveFamily::Triplet, 0.20),
    (GrooveFamily::SparseLegato, 0.10),
    (GrooveFamily::Dotted, 0.10),
    (GrooveFamily::Polyrhythmic, 0.15),
];

fn pick_family(rng: &mut impl Rng) -> GrooveFamily {
    let total: f32 = GROOVE_WEIGHTS.iter().map(|(_, w)| w).sum();
    let mut roll = rng.gen::<f32>() * total;
    for &(family, weight) in &GROOVE_WEIGHTS {
        roll -= weight;
        if roll <= 0.0 {
            return family;
        }
    }
    GrooveFamily::Eighth
}

// --- Pattern application ---

fn apply_pattern(
    beats: &mut [f32; SLOT_COUNT],
    mode: BeatMode,
    count: usize,
    pattern: &[f32],
    variation: f32,
    rng: &mut impl Rng,
) {
    for (i, &base) in pattern.iter().enumerate() {
        if i >= count {
            break;
        }
        let fi = flat_index(mode, count, i);
        if base > 0.0 {
            let varied = (base + rng.gen_range(-variation..variation)).clamp(0.0, 1.0);
            beats[fi] = (varied * 127.0).max(beats[fi]);
        }
    }
}

fn apply_foundation(
    beats: &mut [f32; SLOT_COUNT],
    count: usize,
    prob: f32,
    variation: f32,
    rng: &mut impl Rng,
) {
    for i in 0..count {
        let fi = flat_index(BeatMode::Straight, count, i);
        let p = (prob + rng.gen_range(-variation..variation)).clamp(0.0, 1.0);
        beats[fi] = (p * 127.0).max(beats[fi]);
    }
}

fn add_ghost_notes(
    beats: &mut [f32; SLOT_COUNT],
    mode: BeatMode,
    count: usize,
    chance: f32,
    prob_range: (f32, f32),
    rng: &mut impl Rng,
) {
    for i in 0..count {
        let fi = flat_index(mode, count, i);
        if beats[fi] < 5.0 && rng.gen::<f32>() < chance {
            let p = rng.gen_range(prob_range.0..prob_range.1);
            beats[fi] = p * 127.0;
        }
    }
}

// --- Link generation ---

fn generate_chain_links(
    mode: BeatMode,
    count: usize,
    beats: &[f32; SLOT_COUNT],
    link_chance: f32,
    max_chain: usize,
    rng: &mut impl Rng,
) -> Vec<(u8, u8)> {
    let mut links = Vec::new();
    let mut i = 0;
    while i < count {
        let fi = flat_index(mode, count, i);
        if beats[fi] > 20.0 && rng.gen::<f32>() < link_chance && i + 1 < count {
            let remaining = (count - i - 1).min(max_chain);
            if remaining > 0 {
                let chain_len = rng.gen_range(1..=remaining);
                for j in 0..chain_len {
                    let src_fi = flat_index(mode, count, i + j);
                    let tgt_fi = flat_index(mode, count, i + j + 1);
                    // Only link if target also has some probability
                    if beats[tgt_fi] > 5.0 || j > 0 {
                        links.push((src_fi as u8, tgt_fi as u8));
                    }
                }
                i += chain_len + 1;
                continue;
            }
        }
        i += 1;
    }
    links
}

fn generate_phrase_links(
    mode: BeatMode,
    count: usize,
    beats: &mut [f32; SLOT_COUNT],
    phrase_count: usize,
    phrase_len: usize,
    rng: &mut impl Rng,
) -> Vec<(u8, u8)> {
    let mut links = Vec::new();
    let active: Vec<usize> = (0..count)
        .filter(|&i| beats[flat_index(mode, count, i)] > 20.0)
        .collect();

    if active.len() < 2 {
        return links;
    }

    for _ in 0..phrase_count {
        let start_idx = rng.gen_range(0..active.len());
        let start_beat = active[start_idx];
        let len = phrase_len.min(count - start_beat - 1);
        if len == 0 {
            continue;
        }

        for j in 0..len {
            let src = start_beat + j;
            let tgt = start_beat + j + 1;
            if tgt >= count {
                break;
            }
            let tgt_fi = flat_index(mode, count, tgt);
            if beats[tgt_fi] < 5.0 {
                beats[tgt_fi] = rng.gen_range(40.0..80.0);
            }
            let src_fi = flat_index(mode, count, src);
            links.push((src_fi as u8, tgt_fi as u8));
        }
    }
    links
}

// --- Strength grid ---

fn generate_strength_grid(rng: &mut impl Rng) -> [f32; 96] {
    let mut grid = [0.5f32; 96];
    let shape = rng.gen_range(0u8..6);

    match shape {
        0 => {
            // Arc: build up to middle, come back down
            for (i, val) in grid.iter_mut().enumerate() {
                let t = i as f32 / 95.0;
                *val = 0.3 + 0.5 * (t * std::f32::consts::PI).sin();
            }
        }
        1 => {
            // Crescendo
            for (i, val) in grid.iter_mut().enumerate() {
                *val = 0.2 + 0.6 * (i as f32 / 95.0);
            }
        }
        2 => {
            // Accented downbeats (every 24 positions = 1 beat in 96-grid)
            for (i, val) in grid.iter_mut().enumerate() {
                let beat_pos = i % 24;
                *val = if beat_pos < 6 { 0.75 } else { 0.35 };
            }
        }
        3 => {
            // Two-bar feel: first half stronger
            for (i, val) in grid.iter_mut().enumerate() {
                *val = if i < 48 { 0.65 } else { 0.35 };
            }
        }
        4 => {
            // Organic random: smooth noise
            let mut current = 0.5f32;
            for val in grid.iter_mut() {
                current += rng.gen_range(-0.08..0.08);
                current = current.clamp(0.2, 0.8);
                *val = current;
            }
        }
        _ => {
            // Flat with slight variation
            for val in grid.iter_mut() {
                *val = 0.5 + rng.gen_range(-0.1..0.1);
            }
        }
    }

    // Add small random variation to all shapes
    for val in grid.iter_mut() {
        *val = (*val + rng.gen_range(-0.05..0.05)).clamp(0.0, 1.0);
    }

    grid
}

// --- Individual groove generators ---

fn gen_eighth_groove(
    density: f32,
    rng: &mut impl Rng,
) -> (BeatSuggestion, Vec<(u8, u8)>) {
    let mut beats = [0.0f32; SLOT_COUNT];

    let pattern_idx = rng.gen_range(0..EIGHTH_PATTERNS.len());
    let pattern = &EIGHTH_PATTERNS[pattern_idx];
    apply_pattern(&mut beats, BeatMode::Straight, 8, pattern, 0.15, rng);

    // Optional quarter-note foundation (50% chance)
    if rng.gen::<f32>() < 0.5 {
        let foundation_beats: Vec<usize> = (0..4)
            .filter(|_| rng.gen::<f32>() < 0.6)
            .collect();
        for &i in &foundation_beats {
            let fi = flat_index(BeatMode::Straight, 4, i);
            beats[fi] = rng.gen_range(70.0f32..110.0).min(127.0);
        }
    }

    // Ghost notes on 16ths (30% chance per empty slot)
    add_ghost_notes(&mut beats, BeatMode::Straight, 16, 0.15, (0.08, 0.25), rng);

    let mut links = generate_chain_links(BeatMode::Straight, 8, &beats, 0.35, 2, rng);

    // Occasionally add a longer legato phrase
    if rng.gen::<f32>() < 0.4 {
        let phrase_links = generate_phrase_links(
            BeatMode::Straight, 8, &mut beats, 1, rng.gen_range(2..=3), rng,
        );
        links.extend(phrase_links);
    }

    let swing = rng.gen_range(50.0..62.0);
    let strength = generate_strength_grid(rng);

    apply_density_and_normalize(&mut beats, density);
    (BeatSuggestion { beats, swing, strength }, links)
}

fn gen_sixteenth_groove(
    density: f32,
    rng: &mut impl Rng,
) -> (BeatSuggestion, Vec<(u8, u8)>) {
    let mut beats = [0.0f32; SLOT_COUNT];

    let pattern_idx = rng.gen_range(0..SIXTEENTH_PATTERNS.len());
    let pattern = &SIXTEENTH_PATTERNS[pattern_idx];
    apply_pattern(&mut beats, BeatMode::Straight, 16, pattern, 0.12, rng);

    // Quarter-note foundation
    apply_foundation(&mut beats, 4, 0.7, 0.2, rng);

    // 16th ghost notes in gaps
    add_ghost_notes(&mut beats, BeatMode::Straight, 16, 0.1, (0.06, 0.2), rng);

    let mut links = generate_chain_links(BeatMode::Straight, 16, &beats, 0.25, 3, rng);

    // Add 1-2 legato phrases across 16ths
    let phrase_count = rng.gen_range(1..=2);
    let phrase_links = generate_phrase_links(
        BeatMode::Straight, 16, &mut beats, phrase_count, rng.gen_range(2..=4), rng,
    );
    links.extend(phrase_links);

    let swing = rng.gen_range(50.0..56.0);
    let strength = generate_strength_grid(rng);

    apply_density_and_normalize(&mut beats, density);
    (BeatSuggestion { beats, swing, strength }, links)
}

fn gen_triplet_groove(
    density: f32,
    rng: &mut impl Rng,
) -> (BeatSuggestion, Vec<(u8, u8)>) {
    let mut beats = [0.0f32; SLOT_COUNT];

    // Choose between triplet 6 and triplet 12
    let use_12 = rng.gen::<f32>() < 0.4;

    if use_12 {
        let pattern_idx = rng.gen_range(0..TRIPLET_12_PATTERNS.len());
        apply_pattern(&mut beats, BeatMode::Triplet, 12, &TRIPLET_12_PATTERNS[pattern_idx], 0.15, rng);
    } else {
        let pattern_idx = rng.gen_range(0..TRIPLET_6_PATTERNS.len());
        apply_pattern(&mut beats, BeatMode::Triplet, 6, &TRIPLET_6_PATTERNS[pattern_idx], 0.15, rng);
    }

    // Optional straight foundation for polyrhythmic hint
    if rng.gen::<f32>() < 0.3 {
        let fi = flat_index(BeatMode::Straight, 2, 0);
        beats[fi] = rng.gen_range(60.0..90.0);
    }

    let div = if use_12 { 12 } else { 6 };
    let mut links = generate_chain_links(BeatMode::Triplet, div, &beats, 0.3, 2, rng);

    if rng.gen::<f32>() < 0.5 {
        let phrase_links = generate_phrase_links(
            BeatMode::Triplet, div, &mut beats, 1, rng.gen_range(2..=3), rng,
        );
        links.extend(phrase_links);
    }

    let swing = 50.0; // triplets provide their own shuffle
    let strength = generate_strength_grid(rng);

    apply_density_and_normalize(&mut beats, density);
    (BeatSuggestion { beats, swing, strength }, links)
}

fn gen_sparse_legato(
    density: f32,
    rng: &mut impl Rng,
) -> (BeatSuggestion, Vec<(u8, u8)>) {
    let mut beats = [0.0f32; SLOT_COUNT];

    // Very few trigger points, heavy linking
    let trigger_count = rng.gen_range(2..=4);
    let div = if rng.gen::<f32>() < 0.6 { 8 } else { 4 };

    let mut positions: Vec<usize> = Vec::new();
    while positions.len() < trigger_count {
        let pos = rng.gen_range(0..div);
        if !positions.contains(&pos) {
            positions.push(pos);
        }
    }
    positions.sort();

    for &pos in &positions {
        let fi = flat_index(BeatMode::Straight, div, pos);
        beats[fi] = rng.gen_range(80.0..127.0);
    }

    // Create long chains from each trigger point
    let mut links = Vec::new();
    for &pos in &positions {
        let chain_len = rng.gen_range(2..=4).min(div - pos - 1);
        for j in 0..chain_len {
            let src = pos + j;
            let tgt = pos + j + 1;
            if tgt >= div {
                break;
            }
            let tgt_fi = flat_index(BeatMode::Straight, div, tgt);
            if beats[tgt_fi] < 5.0 {
                beats[tgt_fi] = rng.gen_range(30.0..60.0);
            }
            let src_fi = flat_index(BeatMode::Straight, div, src);
            links.push((src_fi as u8, tgt_fi as u8));
        }
    }

    let swing = rng.gen_range(50.0..65.0);
    let strength = generate_strength_grid(rng);

    apply_density_and_normalize(&mut beats, density);
    (BeatSuggestion { beats, swing, strength }, links)
}

fn gen_dotted_groove(
    density: f32,
    rng: &mut impl Rng,
) -> (BeatSuggestion, Vec<(u8, u8)>) {
    let mut beats = [0.0f32; SLOT_COUNT];

    let pattern_idx = rng.gen_range(0..DOTTED_6_PATTERNS.len());
    apply_pattern(&mut beats, BeatMode::Dotted, 6, &DOTTED_6_PATTERNS[pattern_idx], 0.15, rng);

    // Optional dotted-11 ghost layer
    if rng.gen::<f32>() < 0.3 {
        add_ghost_notes(&mut beats, BeatMode::Dotted, 11, 0.2, (0.1, 0.3), rng);
    }

    // Straight anchor on beat 1
    let fi = flat_index(BeatMode::Straight, 4, 0);
    beats[fi] = rng.gen_range(70.0f32..100.0).min(127.0);

    let mut links = generate_chain_links(BeatMode::Dotted, 6, &beats, 0.3, 2, rng);

    if rng.gen::<f32>() < 0.4 {
        let phrase_links = generate_phrase_links(
            BeatMode::Dotted, 6, &mut beats, 1, 2, rng,
        );
        links.extend(phrase_links);
    }

    let swing = 50.0;
    let strength = generate_strength_grid(rng);

    apply_density_and_normalize(&mut beats, density);
    (BeatSuggestion { beats, swing, strength }, links)
}

fn gen_polyrhythmic(
    density: f32,
    rng: &mut impl Rng,
) -> (BeatSuggestion, Vec<(u8, u8)>) {
    let mut beats = [0.0f32; SLOT_COUNT];

    // Layer 1: Straight 8
    let eighth_idx = rng.gen_range(0..EIGHTH_PATTERNS.len());
    let mut eighth_pattern = EIGHTH_PATTERNS[eighth_idx];
    // Reduce probabilities for layering (avoid excessive total)
    for p in &mut eighth_pattern {
        *p *= 0.7;
    }
    apply_pattern(&mut beats, BeatMode::Straight, 8, &eighth_pattern, 0.1, rng);

    // Layer 2: Triplet 6
    let triplet_idx = rng.gen_range(0..TRIPLET_6_PATTERNS.len());
    let mut triplet_pattern = TRIPLET_6_PATTERNS[triplet_idx];
    for p in &mut triplet_pattern {
        *p *= 0.6;
    }
    apply_pattern(&mut beats, BeatMode::Triplet, 6, &triplet_pattern, 0.1, rng);

    // Links within each layer
    let mut links = generate_chain_links(BeatMode::Straight, 8, &beats, 0.2, 2, rng);
    let triplet_links = generate_chain_links(BeatMode::Triplet, 6, &beats, 0.25, 2, rng);
    links.extend(triplet_links);

    let swing = rng.gen_range(50.0..55.0);
    let strength = generate_strength_grid(rng);

    apply_density_and_normalize(&mut beats, density);
    (BeatSuggestion { beats, swing, strength }, links)
}

// --- Density and normalization ---

fn apply_density_and_normalize(beats: &mut [f32; SLOT_COUNT], density: f32) {
    for b in beats.iter_mut() {
        *b = (*b * density).clamp(0.0, 127.0);
        if *b < 5.0 {
            *b = 0.0;
        }
    }
    normalize_beat_constraints(beats);
}

// --- Public API ---

pub fn generate_groove(
    density: f32,
    rng: &mut impl Rng,
) -> (BeatSuggestion, Vec<(u8, u8)>) {
    let family = pick_family(rng);
    match family {
        GrooveFamily::Eighth => gen_eighth_groove(density, rng),
        GrooveFamily::Sixteenth => gen_sixteenth_groove(density, rng),
        GrooveFamily::Triplet => gen_triplet_groove(density, rng),
        GrooveFamily::SparseLegato => gen_sparse_legato(density, rng),
        GrooveFamily::Dotted => gen_dotted_groove(density, rng),
        GrooveFamily::Polyrhythmic => gen_polyrhythmic(density, rng),
    }
}
