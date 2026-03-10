# Parameter Reference

## PLL Reference

| ID | Name | Range | Default |
|----|------|-------|---------|
| synth_pll_ref_octave | Oct | -3..+3 | 0 |
| synth_pll_ref_tune | Tune | -12..+12 | 0 |
| synth_pll_ref_fine_tune | Fine | -1.0..+1.0 | 0.0 |
| synth_pll_ref_pulse_width | PW | 0.01–0.99 | 0.5 |

## PLL Loop

| ID | Name | Range | Default |
|----|------|-------|---------|
| synth_pll_track_speed | Trk | 0.0–1.0 | 0.5 |
| synth_pll_damping | Dmp | 0.0–1.0 | 0.3 |
| synth_pll_influence | Inf | 0.0–1.0 | 0.5 |
| synth_pll_mult | Mlt | 1,2,4,8,16,32,64 | 1 |
| synth_pll_feedback | FB | 0.0–1.0 | 0.0 |

## PLL Stereo

| ID | Name | Range | Default |
|----|------|-------|---------|
| synth_pll_stereo_damp_offset | StΔ | 0.0–0.5 | 0.0 |
| synth_pll_stereo_track_offset | StTrkΔ | 0.0–0.5 | 0.0 |
| synth_pll_stereo_phase | StPh | 0.0–1.0 | 0.0 |
| synth_pll_cross_feedback | XFB | 0.0–1.0 | 0.0 |

## PLL FM

| ID | Name | Range | Default |
|----|------|-------|---------|
| synth_pll_fm_amount | FM | 0.0–1.0 | 0.0 |
| synth_pll_fm_ratio | Ratio | 1–16 | 1 |
| synth_pll_fm_env_amount | FMEnv | 0.0–1.0 | 0.0 |

## PLL Advanced

| ID | Name | Range | Default |
|----|------|-------|---------|
| synth_pll_retrigger | Retrig | 0.0–1.0 | 0.05 |
| synth_pll_burst_threshold | BstTh | 0.0–1.0 | 0.7 |
| synth_pll_burst_amount | BstAmt | 0.0–10.0 | 3.3 |
| synth_pll_loop_saturation | LoopSat | 1–500 | 100 |
| synth_pll_color_amount | ColorAmt | 0.0–1.0 | 0.25 |
| synth_pll_edge_sensitivity | EdgeSns | 0.001–0.2 | 0.02 |
| synth_pll_range | Rng | 0.0–1.0 | 1.0 |

## PLL Output & Modes

| ID | Name | Range | Default |
|----|------|-------|---------|
| synth_pll_volume | Vol | 0.0–1.0 | 0.0 |
| synth_pll_glide | Glide | 0–500ms | 0 |
| synth_pll_colored | Color | bool | - |
| synth_pll_mode | Mode | false=AnalogPD, true=EdgePFD | false |
| synth_pll_mult_slew | FAST/SLOW | FAST=1/16, SLOW=1/1 | - |
| synth_pll_precision | PREC | bool | true |
| synth_pll_enable | Enable | bool | - |

## VPS

| ID | Name | Range | Default |
|----|------|-------|---------|
| synth_osc_d | D | 0.0–1.0 | 0.5 |
| synth_osc_v | V | 0.0–1.0 | 0.5 |
| synth_osc_stereo_v_offset | VΔ | 0.0–0.3 | 0.0 |
| synth_osc_stereo_d_offset | DΔ | 0.0–0.3 | 0.0 |
| synth_osc_octave | Oct | -3..+3 | 0 |
| synth_osc_tune | Tune | -12..+12 | 0 |
| synth_osc_fine | Fine | -1.0..+1.0 | 0.0 |
| synth_osc_fold | Fold | 0.0–1.0 | 0.0 |
| synth_vps_shape_type | SHP | 0=Soft, 1=Foldback | 0 |
| synth_vps_shape_amount | SHP | 0.0–1.0 | 0.0 |
| synth_vps_phase_mode | Phase | 0=Free, 1=PLL Sync | 0 |
| synth_osc_volume | Vol | 0.0–1.0 | 1.0 |
| synth_vps_enable | Enable | bool | true |

## SAW

| ID | Name | Range | Default |
|----|------|-------|---------|
| synth_saw_enable | Enable | bool | false |
| synth_saw_volume | Vol | 0.0–1.0 | 0.0 |
| synth_saw_octave | Oct | -3..+3 | 0 |
| synth_saw_tune | Tune | -12..+12 | 0 |
| synth_saw_fold | Fold | 0.0–1.0 | 0.0 |
| synth_saw_fold_range | Fold Range | 0=1X, 1=PI | 0 |
| synth_saw_shape_type | Shape | 0–2 | 0 |
| synth_saw_shape_amount | SHP | 0.0–1.0 | 0.0 |

## Sub

| ID | Name | Range | Default |
|----|------|-------|---------|
| synth_sub_volume | Sub | 0.0–1.0 | 0.0 |
| synth_sub_filter_route | Filter Route | bool (false=OUT, true=IN) | false |

## Ladder Filter

| ID | Name | Range | Default |
|----|------|-------|---------|
| synth_filter_enable | Enable | bool | false |
| synth_filter_cutoff | Cutoff | 20–20000 Hz | 20000 |
| synth_filter_resonance | Res | 0.0–1.05 | 0.0 |
| synth_filter_drive | Drive | 0.0–1.0 | 0.0 |
| synth_filter_mode | Mode | 0=LP24, 1=LP12, 2=BP12, 3=HP24 | 0 |
| synth_filter_key_track | Key Track | 0.0–1.0 | 0.0 |
| synth_filter_env_amount | Env Amount | -1.0–1.0 | 0.0 |
| synth_filter_stereo_sep | Stereo Sep | 0.0–0.50 | 0.0 |
| synth_filter_drive_boost | Drive Boost | 0=OFF, 1=+12dB, 2=+24dB, 3=+48dB | 0 |
| synth_filter_sat_type | Sat Type | 0=Transistor, 1=Diode, 2=Tube | 0 |
| synth_filter_morph | Morph | 0.0–1.0 | 0.0 |
| synth_filter_fm | FM | 0.0–1.0 | 0.0 |
| synth_filter_feedback | Feedback | -1.0–1.0 | 0.0 |
| synth_filter_bass_lock | Bass Lock | 0.0–1.0 | 0.0 |
| synth_filter_pole_spread | Pole Spread | 0.0–1.0 | 0.0 |
| synth_filter_res_character | Res Char | 0.0–1.0 | 0.0 |
| synth_filter_res_tilt | Res Tilt | -1.0–1.0 | 0.0 |
| synth_filter_cutoff_slew | Cut Slew | 0.0–1.0 | 0.0 |
| synth_filter_poles | Poles | 0=4-pole(24dB), 1=8-pole(48dB) | 0 |

## Filter Envelope

| ID | Name | Range | Default |
|----|------|-------|---------|
| synth_filter_env_attack | Atk | 0.5–5000ms | 10.0 |
| synth_filter_env_attack_shape | AtkSh | -1.0..+1.0 | 0.0 |
| synth_filter_env_decay | Dec | 0.5–10000ms | 100.0 |
| synth_filter_env_decay_shape | DecSh | -1.0..+1.0 | 0.0 |
| synth_filter_env_sustain | Sus | 0.0–1.0 | 0.7 |
| synth_filter_env_release | Rel | 0.5–10000ms | 200.0 |
| synth_filter_env_release_shape | RelSh | -1.0..+1.0 | 0.0 |
| synth_filter_env_dip | Dip | 0.0–1.0 | 0.0 |
| synth_filter_env_range | Range | 1.0–8.0 octaves | 4.0 |

## Coloration

| ID | Name | Range | Default |
|----|------|-------|---------|
| synth_ring_mod | Ring | 0.0–1.0 | 0.0 |
| synth_wavefold | Fold | 0.0–1.0 | 0.0 |
| synth_drift_amount | Drift | 0.0–1.0 | 0.0 |
| synth_drift_rate | Rate | 0.0–1.0 | 0.3 |
| synth_noise_amount | Noise | 0.0–1.0 | 0.0 |
| synth_tube_drive | Tube | 0.0–1.0 | 0.0 |
| synth_color_distortion_amount | Dist | 0.0–1.0 | 0.0 |
| synth_color_distortion_threshold | Thr | 0.1–1.0 | 0.7 |
| synth_coloration_enable | Enable | bool | true |

## Formant

| ID | Name | Range | Default |
|----|------|-------|---------|
| synth_formant_mix | Mix | 0.0–1.0 | 0.0 |
| synth_formant_vowel | Vowel | 0.0–1.0 | 0.0 |
| synth_formant_shift | Shift | -2.0..+2.0 | 0.0 |
| synth_formant_enable | Enable | bool | true |

## Volume Envelope

| ID | Name | Range | Default |
|----|------|-------|---------|
| synth_vol_attack | Atk | 0.5–5000ms | 10.0 |
| synth_vol_attack_shape | AtkSh | -1.0..+1.0 | 0.0 |
| synth_vol_decay | Dec | 0.5–10000ms | 100.0 |
| synth_vol_decay_shape | DecSh | -1.0..+1.0 | 0.0 |
| synth_vol_sustain | Sus | 0.0–1.0 | 0.7 |
| synth_vol_release | Rel | 0.5–10000ms | 200.0 |
| synth_vol_release_shape | RelSh | -1.0..+1.0 | 0.0 |
| synth_retrigger_dip | Dip | 0.0–1.0 | 0.0 |
| synth_phase_reset | PhRst | bool | true |
| synth_pll_tail | Tail | bool | false |
| synth_pll_tail_time | TailT | 50–5000ms | 500.0 |
| synth_pll_tail_amount | TailA | 0.0–1.0 | 0.3 |

## Master

| ID | Name | Range | Default |
|----|------|-------|---------|
| master_hpf | HPF | 0=Off, 1=35Hz, 2=80Hz, 3=120Hz, 4=220Hz | 0 |
| master_hpf_boost | Boost | 0=Flat, 1=Med Q2, 2=High Q4 | 0 |
| master_hpf_sub | Sub | 0=OUT, 1=IN | 0 |
| brilliance_amount | AMT | 0.0–1.0 | 0.0 |
| brilliance_drive | DRV | 0.0–1.0 | 0.0 |
| synth_volume | Vol | 0.0–1.0 | 0.8 |
| synth_oversampling_factor | OS | 0=1x..7=128x | 0 |

## LFO 1/2/3

| ID | Name | Range | Default |
|----|------|-------|---------|
| lfo[N]_rate | Rate | 0.01–50Hz | 1.0 |
| lfo[N]_waveform | Wave | 0=Sin,1=Tri,2=Saw,3=Sq,4=S&H | 0 |
| lfo[N]_tempo_sync | Sync | bool | false |
| lfo[N]_sync_division | Div | 0–13 | 2 |
| lfo[N]_sync_source | Src | -1..2 | -1 |
| lfo[N]_phase_mod | PhMod | 0.0–1.0 | 0.0 |
| lfo[N]_dest1/dest2 | Dst | 0–30 | 0 |
| lfo[N]_amount1/amount2 | Amt | -1.0..+1.0 | 0.0 |

**Sync divisions:** 0=1/1, 1=1/2, 2=1/4, 3=1/8, 4=1/16, 5=1/32, 6=1/2D, 7=1/4D, 8=1/8D, 9=1/16D, 10=1/2T, 11=1/4T, 12=1/8T, 13=1/16T

**Mod destinations:** 0=None | PLL: 1=Damp, 2=Infl, 3=Track, 4=FM, 5=XFB, 6=OT, 7=Rng, 17=Vol, 20=Mult, 21=MultD | Sub: 19=Vol | VPS: 8=D, 9=V, 25=VΔ, 23=DΔ, 24=Fold, 22=SHP, 18=Vol | SAW: 28=Fold, 29=SHP, 30=Vol | Color: 13=Drift, 14=Tube | Filter: 42=Cutoff, 43=Res, 44=Drive, 45=EnvAmt, 46=Morph, 47=FM, 48=Feedback, 49=BassLock, 50=PoleSpread, 51=ResChar, 52=ResTilt

## Step Modulator

| ID | Name | Range | Default |
|----|------|-------|---------|
| mseq_step_[1-16] | Step N | -1.0..+1.0 | 0.0 |
| mseq_ties | Ties | 0–65535 bitmask | 0 |
| mseq_division | Rate | 0–13 | 3 |
| mseq_slew | Slew | 0–200ms | 5.0 |
| mseq_dest1/dest2 | Dst | 0–30 | 0 |
| mseq_amount1/amount2 | Amt | -1.0..+1.0 | 0.0 |

## Sequencer

| ID | Name | Range | Default |
|----|------|-------|---------|
| swing_amount | Swing | 50–75% | 50 |
| note_length_percent | Length | 1–200% | 100 |

Beat probabilities: 152 params (div[X]_beat[Y]), range 0–127, default 0. See `sequencer.md` for slot layout.

**Length/Decay/Position modifiers:** 2 slots each with target (-100..+100), amount, prob (0–127).

**Velocity modifiers:** Strength-based and length-based, each with target, amount (-99..+27), prob.

## UI State (not plugin params)

| Setting | Location |
|---------|----------|
| note_pool | SharedUiState |
| root_note | SharedUiState |
| strength_values [96] | SharedUiState |
