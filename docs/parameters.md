# Device - Parameter Reference

Complete reference of all parameters and UI controls.

## UI Pages Overview

| Page | Purpose |
|------|---------|
| Beats | Beat probability sliders (91 params) |
| Length | Note duration and modifiers |
| Notes | Piano roll note selection |
| Strength | 96-position beat strength grid |
| Synth | Sound design controls (Tab A/B) |
| Mod | LFO configuration and routing |
| Presets | Save/load presets |

## Synth Page - Tab A

### PLL Reference (Yellow Group)

| ID | Name | Range | Default | Description |
|----|------|-------|---------|-------------|
| synth_pll_ref_octave | Oct | -5 to +5 | 0 | Reference octave |
| synth_pll_ref_tune | Tune | -12 to +12 | 0 | Semitone offset |
| synth_pll_ref_fine_tune | Fine | -1.0 to +1.0 | 0.0 | Fine tune (cents-like) |
| synth_pll_ref_pulse_width | PW | 0.01-0.99 | 0.5 | Reference pulse width |

### PLL Loop (Blue Group)

| ID | Name | Range | Default | Description |
|----|------|-------|---------|-------------|
| synth_pll_track_speed | Trk | 0.0-1.0 | 0.5 | Tracking speed |
| synth_pll_damping | Dmp | 0.0-1.0 | 0.3 | Loop filter damping |
| synth_pll_influence | Inf | 0.0-1.0 | 0.5 | Phase influence |
| synth_pll_mult | Mlt | 1,2,4,8,16,32,64 | 1 | Frequency multiplier |
| synth_pll_feedback | FB | 0.0-1.0 | 0.0 | Output feedback |

### PLL Stereo (Purple Group)

| ID | Name | Range | Default | Description |
|----|------|-------|---------|-------------|
| synth_pll_stereo_damp_offset | StΔ | 0.0-0.5 | 0.0 | Damping L/R offset |
| synth_pll_stereo_track_offset | StTrkΔ | 0.0-0.5 | 0.0 | Track speed L/R offset |
| synth_pll_stereo_phase | StPh | 0.0-1.0 | 0.0 | Phase L/R offset |
| synth_pll_cross_feedback | XFB | 0.0-1.0 | 0.0 | Cross-channel feedback |

### PLL FM

| ID | Name | Range | Default | Description |
|----|------|-------|---------|-------------|
| synth_pll_fm_amount | FM | 0.0-1.0 | 0.0 | FM modulation depth |
| synth_pll_fm_ratio | Ratio | 1-16 | 1 | FM frequency ratio |
| synth_pll_fm_env_amount | FMEnv | 0.0-1.0 | 0.0 | Filter env to FM |

### PLL Advanced

| ID | Name | Range | Default | Description |
|----|------|-------|---------|-------------|
| synth_pll_retrigger | Retrig | 0.0-1.0 | 0.05 | Note retrigger behavior |
| synth_pll_burst_threshold | BstTh | 0.0-1.0 | 0.7 | Overtrack threshold |
| synth_pll_burst_amount | BstAmt | 0.0-10.0 | 3.3 | Overtrack intensity |
| synth_pll_loop_saturation | LoopSat | 1-500 | 100 | Integrator limit |
| synth_pll_color_amount | ColorAmt | 0.0-1.0 | 0.25 | Harmonic saturation |
| synth_pll_edge_sensitivity | EdgeSns | 0.001-0.2 | 0.02 | PFD edge threshold |
| synth_pll_range | Rng | 0.0-1.0 | 1.0 | PLL lock bandwidth (slow→fast) |

### PLL Output

| ID | Name | Range | Default | Description |
|----|------|-------|---------|-------------|
| synth_pll_volume | Vol | 0.0-1.0 | 0.0 | PLL output level |
| synth_pll_glide | Glide | 0-500 ms | 0 | Portamento time |

### PLL Modes (Switches)

| ID | Name | Description |
|----|------|-------------|
| synth_pll_colored | Color | Enable harmonic coloration |
| synth_pll_mode | Mode | false=AnalogLikePD, true=EdgePFD |
| synth_pll_enable | Enable | Bypass PLL oscillator |

### VPS Oscillator

| ID | Name | Range | Default | Description |
|----|------|-------|---------|-------------|
| synth_osc_d | D | 0.0-1.0 | 0.5 | Phase distortion |
| synth_osc_v | V | 0.0-1.0 | 0.5 | Shape/timbre |
| synth_osc_stereo_v_offset | StV | 0.0-1.0 | 0.0 | V offset L/R |
| synth_osc_octave | Oct | -5 to +5 | 0 | Octave shift |
| synth_osc_volume | Vol | 0.0-1.0 | 1.0 | VPS output level |
| synth_vps_enable | Enable | - | true | Bypass VPS oscillator |

### Sub Oscillator

| ID | Name | Range | Default | Description |
|----|------|-------|---------|-------------|
| synth_sub_volume | Sub | 0.0-1.0 | 0.0 | Sub level (sine, -1 oct) |

### Filter

| ID | Name | Range | Default | Description |
|----|------|-------|---------|-------------|
| synth_filter_cutoff | Cut | 20-20000 Hz | 1000 | Filter frequency |
| synth_filter_resonance | Res | 0.0-0.99 | 0.0 | Resonance/Q |
| synth_filter_drive | Drive | 1.0-15.0 | 1.0 | Input saturation (NOTE: not yet implemented) |
| synth_filter_env_amount | Env | -10000 to +10000 | 0 | Envelope modulation |
| synth_filter_mode | Mode | 0-10 | 3 | Filter type (see below) |
| synth_filter_enable | Enable | - | true | Bypass filter |

**Filter Modes:**
0=LP6, 1=LP12, 2=LP18, 3=LP24, 4=HP6, 5=HP12, 6=HP18, 7=HP24, 8=BP12, 9=BP24, 10=Notch

## Synth Page - Tab B

### Coloration

| ID | Name | Range | Default | Description |
|----|------|-------|---------|-------------|
| synth_ring_mod | Ring | 0.0-1.0 | 0.0 | VPS×PLL ring mod |
| synth_wavefold | Fold | 0.0-1.0 | 0.0 | Wavefolder amount |
| synth_drift_amount | Drift | 0.0-1.0 | 0.0 | Pitch drift depth |
| synth_drift_rate | Rate | 0.0-1.0 | 0.3 | Drift LFO speed |
| synth_noise_amount | Noise | 0.0-1.0 | 0.0 | White noise level |
| synth_tube_drive | Tube | 0.0-1.0 | 0.0 | Tube saturation |
| synth_color_distortion_amount | Dist | 0.0-1.0 | 0.0 | Distortion drive amount |
| synth_color_distortion_threshold | Thr | 0.1-1.0 | 0.7 | Soft clipping threshold |
| synth_coloration_enable | Enable | - | true | Bypass coloration |

### Formant

| ID | Name | Range | Default | Description |
|----|------|-------|---------|-------------|
| synth_formant_mix | Mix | 0.0-1.0 | 0.0 | Dry/formant blend |
| synth_formant_vowel | Vowel | 0.0-1.0 | 0.0 | A→E→I→O→U morph |
| synth_formant_shift | Shift | -2.0 to +2.0 | 0.0 | Frequency shift (oct) |
| synth_formant_enable | Enable | - | true | Bypass formant |

### Volume Envelope

| ID | Name | Range | Default | Description |
|----|------|-------|---------|-------------|
| synth_vol_attack | Atk | 1-5000 ms | 1 | Attack time |
| synth_vol_attack_shape | AtkSh | -5 to +5 | 0 | Attack curve |
| synth_vol_decay | Dec | 1-10000 ms | 20 | Decay time |
| synth_vol_decay_shape | DecSh | -5 to +5 | 0 | Decay curve |
| synth_vol_sustain | Sus | 0.0-1.0 | 1.0 | Sustain level |
| synth_vol_release | Rel | 1-10000 ms | 5 | Release time |
| synth_vol_release_shape | RelSh | -5 to +5 | 0 | Release curve |

### Filter Envelope

| ID | Name | Range | Default | Description |
|----|------|-------|---------|-------------|
| synth_filt_attack | Atk | 1-5000 ms | 1 | Attack time |
| synth_filt_attack_shape | AtkSh | -5 to +5 | 0 | Attack curve |
| synth_filt_decay | Dec | 1-10000 ms | 20 | Decay time |
| synth_filt_decay_shape | DecSh | -5 to +5 | 0 | Decay curve |
| synth_filt_sustain | Sus | 0.0-1.0 | 1.0 | Sustain level |
| synth_filt_release | Rel | 1-10000 ms | 5 | Release time |
| synth_filt_release_shape | RelSh | -5 to +5 | 0 | Release curve |

### Reverb

| ID | Name | Range | Default | Description |
|----|------|-------|---------|-------------|
| synth_reverb_mix | Mix | 0.0-1.0 | 0.0 | Dry/wet balance |
| synth_reverb_pre_delay | PreD | 0-200 ms | 50 | Pre-delay time |
| synth_reverb_time_scale | Time | 0.0-2.0 | 0.85 | Size scaling |
| synth_reverb_input_hpf | InHP | 20-2000 Hz | 20 | Input high-pass |
| synth_reverb_input_lpf | InLP | 1k-20k Hz | 18000 | Input low-pass |
| synth_reverb_hpf | HP | 20-2000 Hz | 100 | Reverb high-pass |
| synth_reverb_lpf | LP | 1k-20k Hz | 14000 | Reverb low-pass |
| synth_reverb_mod_speed | ModSpd | 0.0-2.0 | 0.3 | Modulation rate |
| synth_reverb_mod_depth | ModDep | 0.0-1.0 | 0.4 | Modulation depth |
| synth_reverb_mod_shape | ModShp | 0.0-1.0 | 0.5 | Mod waveform |
| synth_reverb_diffusion_mix | DifMix | 0.0-1.0 | 0.85 | Early reflections |
| synth_reverb_diffusion | Dif | 0.0-1.0 | 0.75 | Diffusion density |
| synth_reverb_decay | Decay | 0.0-1.0 | 0.8 | Tail length |
| synth_reverb_ducking | Duck | 0.0-1.0 | 0.0 | Sidechain ducking |
| synth_reverb_enable | Enable | - | true | Bypass reverb |

### Master

| ID | Name | Range | Default | Description |
|----|------|-------|---------|-------------|
| synth_volume | Vol | 0.0-1.0 | 0.8 | Master output |

### Quality Settings

| ID | Name | Options | Default | Description |
|----|------|---------|---------|-------------|
| synth_oversampling_factor | OS | 0=1x, 1=2x, 2=4x, 3=8x, 4=16x | 0 | Oversampling factor |

**How it works:**
- Base rate = DAW/system sample rate (automatic)
- Processing rate = Base rate × Oversampling factor

**Examples:**
- DAW at 48k, OS=1x → processes at 48k
- DAW at 48k, OS=4x → processes at 192k, 4x downsampling
- DAW at 192k, OS=1x → processes at 192k (recommended for high sample rates)
- DAW at 192k, OS=2x → processes at 384k (very CPU intensive)

## Modulation Page

### LFO 1/2/3 (identical structure)

| ID | Name | Range | Default | Description |
|----|------|-------|---------|-------------|
| lfo[N]_rate | Rate | 0.01-50 Hz | 1.0 | Free-run frequency |
| lfo[N]_waveform | Wave | 0-4 | 0 | Shape (see below) |
| lfo[N]_tempo_sync | Sync | bool | false | Lock to tempo |
| lfo[N]_sync_division | Div | 0-13 | 2 | Sync division |
| lfo[N]_sync_source | Src | -1 to 2 | -1 | Cross-mod source |
| lfo[N]_phase_mod | PhMod | 0.0-1.0 | 0.0 | Phase mod amount |
| lfo[N]_dest1 | Dst1 | 0-26 | 0 | First destination |
| lfo[N]_amount1 | Amt1 | -1.0 to +1.0 | 0.0 | First mod amount |
| lfo[N]_dest2 | Dst2 | 0-26 | 0 | Second destination |
| lfo[N]_amount2 | Amt2 | -1.0 to +1.0 | 0.0 | Second mod amount |

**Waveforms:** 0=Sine, 1=Triangle, 2=Saw, 3=Square, 4=Sample&Hold

**Sync Divisions:**
0=1/1, 1=1/2, 2=1/4, 3=1/8, 4=1/16, 5=1/32,
6=1/2D, 7=1/4D, 8=1/8D, 9=1/16D,
10=1/2T, 11=1/4T, 12=1/8T, 13=1/16T

**Mod Destinations:**
0=None, 1=PLL Damp, 2=PLL Infl, 3=PLL Track, 4=PLL FB, 5=PLL FM,
6=PLL PW, 7=PLL StPh, 8=PLL XFB, 9=PLL FMEnv, 10=PLL OT, 11=PLL Rng,
12=VPS D, 13=VPS V, 14=Filt Cut, 15=Filt Res, 16=Filt Drv,
17=Ring, 18=Fold, 19=Drift, 20=Noise, 21=Tube, 22=Rev Mix, 23=Rev Decay,
24=PLL Vol, 25=VPS Vol, 26=Sub Vol

## Length Page

### Base Length

| ID | Name | Range | Default | Description |
|----|------|-------|---------|-------------|
| note_length_percent | Length | 1-200% | 100 | Base note duration |

### Length Modifiers (×2)

| ID | Name | Range | Default | Description |
|----|------|-------|---------|-------------|
| len_mod_[N]_target | Target | -100 to +100 | 0 | Beat strength target |
| len_mod_[N]_amount | Amount | -100 to +100 | 0 | Duration modifier (-100=shortest, +100=2×) |
| len_mod_[N]_prob | Prob | 0-127 | 0 | Chance of applying |

### Decay Modifiers (×2)

| ID | Name | Range | Default | Description |
|----|------|-------|---------|-------------|
| decay_mod_[N]_target | Target | -100 to +100 | 0 | Beat strength target |
| decay_mod_[N]_amount | Amount | -100 to +100 | 0 | Decay modifier (-100=shortest, +100=2×) |
| decay_mod_[N]_prob | Prob | 0-127 | 0 | Chance of applying |

### Position Modifiers (×2)

| ID | Name | Range | Default | Description |
|----|------|-------|---------|-------------|
| pos_mod_[N]_target | Target | -100 to +100 | 0 | Beat strength target |
| pos_mod_[N]_shift | Shift | -50% to +50% | 0 | Time shift |
| pos_mod_[N]_prob | Prob | 0-127 | 0 | Chance of applying |

## Beats Page

### Swing

| ID | Name | Range | Default | Description |
|----|------|-------|---------|-------------|
| swing_amount | Swing | 50-75% | 50 | Eighth-note swing |

### Beat Probabilities (152 total)

### Straight (63 params)
div1_beat1 (1), div2_beat[1-2] (2), div4_beat[1-4] (4), div8_beat[1-8] (8), div16_beat[1-16] (16), div32_beat[1-32] (32)

### Triplet (45 params)
div3t_beat[1-3] (3), div6t_beat[1-6] (6), div12t_beat[1-12] (12), div24t_beat[1-24] (24)

### Dotted (44 params)
div2d_beat[1-2] (2), div3d_beat[1-3] (3), div6d_beat[1-6] (6), div11d_beat[1-11] (11), div22d_beat[1-22] (22)

All beat params: Range 0-127, Default 0

## Notes Page (UI State)

Settings stored in SharedUiState, not plugin params:

| Setting | Description |
|---------|-------------|
| note_pool | List of NoteSelection with chance and bias |
| root_note | MIDI note number of root |

## Strength Page (UI State)

| Setting | Description |
|---------|-------------|
| strength_values | 96-position array (0.0-1.0) |

## MIDI CC Support

MIDI CCs are received but currently not mapped to parameters. The infrastructure exists in `midi.rs` for:
- Standard 7-bit CCs (0-127)
- 14-bit CCs via MSB/LSB pairs
- NRPN addressing
