# PhaseBurn - Synthesis Engine

## Oscillator Types

PhaseBurn features two distinct oscillator types plus a sub oscillator, each with unique character.

### VPS Oscillator (Variable Phase Shaping)

A waveshaping oscillator using synfx-dsp's VPSOscillator algorithm.

**Parameters:**
| Param | Range | Description |
|-------|-------|-------------|
| D | 0.0-1.0 | Phase distortion amount |
| V | 0.0-1.0 | Shape/timbre control |
| Stereo V Offset (VΔ) | 0.0-0.3 | V difference between L/R for width |
| Stereo D Offset (DΔ) | 0.0-0.3 | D difference between L/R for stereo phase distortion |
| Octave | -3 to +3 | Octave shift |
| Tune | -12 to +12 | Semitone offset |
| Fine | -1.0 to +1.0 | Fine tune (cents) |
| Fold | 0.0-1.0 | Wavefold amount for harmonic enrichment |
| Shape Type | SOFT/FOLD | Waveshaper type (Soft Clip via tanh, Foldback distortion) |
| Shape Amount (SHP) | 0.0-1.0 | Waveshaper intensity |
| Phase Mode | FREE/SYNC | Phase behavior (toggle) |
| Volume | 0.0-1.0 | VPS output level |

**Phase Modes:**
- **FREE**: Randomized phase on note trigger (default, avoids DC clicks)
- **SYNC**: VPS phase resets when PLL reference oscillator completes a cycle (hard sync character, creates harmonically rich timbres that track the PLL reference frequency)

**Shape Types (toggle, active when SHP > 0):**
- **SOFT**: Tanh soft clipping — smooth saturation that compresses peaks
- **FOLD**: Foldback distortion — aggressive harmonic generation, distinct from the sine-based Fold parameter

**Character:**
- D controls phase distortion (soft at 0, harsh at 1)
- V morphs the waveform character
- Stereo V offset creates rich width through different L/R timbres
- Stereo D offset creates width through different L/R phase distortion
- Clean, digital precision
- Built-in DC offset prevention via `limit_v()` parameter limiting

**Sound Design Tips:**
- Low D + mid V: Smooth, round tones
- High D + low V: Aggressive, buzzy timbres
- Stereo V+D offsets together create complex, wide stereo fields
- Tune/Fine for detuning against PLL oscillator creates thick textures
- Fold adds harmonic richness, especially effective at higher D values
- Shape FOLD + low amount creates different harmonics than Fold parameter
- PLL SYNC mode with different PLL reference octave creates classic hard sync sweeps

### PLL Oscillator (Phase-Locked Loop)

A novel synthesis method modeling analog PLL circuits. The VCO attempts to track a reference oscillator but can be pushed into unstable, chaotic behavior.

**Reference Section (Yellow):**
| Param | Range | Description |
|-------|-------|-------------|
| Oct | -3 to +3 | Reference octave |
| Tune | -12 to +12 | Semitone offset |
| PW | 0.01-0.99 | Reference pulse width |

**PLL Loop (Blue):**
| Param | Range | Description |
|-------|-------|-------------|
| Trk | 0.0-1.0 | Tracking speed (low=glide, high=overtrack) |
| Dmp | 0.0-1.0 | Loop filter damping |
| Inf | 0.0-1.0 | Phase influence on VCO |
| Mlt | 1-64 | Frequency multiplier |
| FB | 0.0-1.0 | Output feedback to reference |

**PLL Modes:**
- **AnalogLikePD**: XOR-style phase detector, smooth character
- **EdgePFD**: Edge-triggered phase-frequency detector, more aggressive

**Precision Toggle (PREC):**
- **ON (default)**: New PLL-theory loop — cubic speed curve, ωn/ζ-based Kp/Ki, sub-sample edge detection, clamped phase error. More predictable, musically tuned response.
- **OFF**: Legacy loop — linear speed, ad-hoc damp_factor Kp/Ki, sample-level edges, tanh phase error. Looser, more chaotic character.

**Advanced PLL Parameters:**
| Param | Description |
|-------|-------------|
| Rst (Reset) | Loop state reset intensity on note trigger |
| Burst Threshold | Track speed level where overtrack engages |
| Burst Amount | Intensity of frequency bursts |
| Loop Saturation | Limits integrator runaway |
| Color Amount | Harmonic saturation coefficient |
| Edge Sensitivity | Phase detector edge threshold |
| Range | PLL lock bandwidth (0=very slow sync, 1=fast sync) |

**Stereo PLL:**
| Param | Description |
|-------|-------------|
| St Damp Δ | Damping difference between L/R channels |
| St Track Δ | Track speed offset between channels |
| St Phase | Phase offset for stereo width |
| Cross FB | L→R and R→L cross-feedback amount |

**FM Section:**
| Param | Description |
|-------|-------------|
| FM Amount | FM modulation depth |
| FM Ratio | Integer ratio for FM frequency |
| FM Env | Envelope to FM depth (legacy parameter) |

**Character:**
The PLL oscillator excels at:
- Unstable, analog-like tones at high track speed
- Gliding, resonant behavior at low track speed
- Chaotic, textured sounds with high feedback
- Rich stereo imaging with offset parameters
- FM-like timbres through the FM section
- Click-free operation: VCO phase continues smoothly on retrigger
- Precision toggle for switching between tight PLL-theory loop and looser legacy behavior

**Loop Coefficient Theory (Precision ON):**
- Track speed is cubed before the error filter sigmoid, giving much more resolution at low values
- Kp (proportional) and Ki (integral) derived from ωn (natural frequency) and ζ (damping ratio)
- Bandwidth maps [3..150 Hz] from the curved track speed
- Damping maps to ζ: low damping → underdamped/ringing, high damping → overdamped/smooth
- AnalogLikePD uses linear phase detector for wider capture range
- EdgePFD uses sub-sample interpolation for cleaner high-frequency tracking

**Loop Coefficient Theory (Precision OFF / Legacy):**
- Linear track speed fed through sigmoid
- Ad-hoc Kp/Ki derived from damp_factor (1 - damping * 0.95)
- Sample-level edge detection with integer counters
- Phase error normalized via tanh for softer saturation

**Sound Design Tips:**
- Track < 0.3: Slow, gliding portamento character (cubic curve gives fine control here)
- Track 0.3-0.7: Stable tracking, predictable pitch
- Track > 0.7: "Overtrack" mode - frequency bursts, instability
- High Damping: ζ≈1.5, overdamped — fast settling, clean sound, proportional-dominant
- Low Damping: ζ≈0.15, underdamped — ringy, resonant, integral-dominant
- Use stereo offsets for massive width without losing mono compatibility
- Range at 0: Very slow lock, creates characteristic analog PLL "hunting" behavior
- Range at 1: Fast lock, tight frequency tracking
- Low Range + any Multiplier: Creates musical portamento-like transitions between pitches
- FAST/SLOW toggle: Tempo-synced multiplier slew (FAST=1/16 note, SLOW=1/1 note duration)
- LFO to PLL Mult: Discrete modulation steps between multiplier values (1,2,4,8,16,32,64) with slew
- LFO to PLL Mult D: Direct continuous modulation of the frequency multiplier for evolving timbres

### SAW Oscillator

A polyBLEP sawtooth oscillator with DC blocking, waveshaping, and wavefold.

**Parameters:**
| Param | Range | Description |
|-------|-------|-------------|
| Volume | 0.0-1.0 | SAW output level |
| Octave | -3 to +3 | Octave shift |
| Tune | -12 to +12 | Semitone offset |
| Fold | 0.0-1.0 | Wavefold amount |
| Fold Range | 1X/PI | Fold function: 1X=sin(x), PI=sin(x*PI) for more aggressive folding |
| Shape Type | 0-2 | Waveshaper type |
| Shape Amount | 0.0-1.0 | Waveshaper intensity |

**DC Block Filter:**
The SAW oscillator uses a sample-rate-aware DC block filter with coefficient `r = exp(-2π × 3.5 / sample_rate)`. This keeps the HPF cutoff at ~3.5 Hz regardless of oversampling factor, preserving bass content at high oversampling ratios (32x, 64x, 128x).

### Sub Oscillator

A pure sine wave one octave below the base frequency for clean bass reinforcement.

**Parameters:**
| Param | Range | Description |
|-------|-------|-------------|
| Volume | 0.0-1.0 | Sub level in mix |

**Signal Path:** Routed separately from main oscillator mix. By default bypasses the Master HPF (OUT mode) to preserve clean sub bass. Can be routed through the HPF (IN mode) via the SUB toggle in the Master HPF section.

## Effects Chain

### Coloration Section

A set of effects applied to the oscillator mix.

**Ring Modulation:**
- Multiplies VPS × PLL outputs
- Creates metallic, bell-like timbres
- Amount: 0-1 controls dry/ring mix

**Wavefolding:**
- Soft wavefolder using sine function
- Progressive harmonic enrichment
- Amount: 0-1 controls fold intensity

**Drift:**
- Slow random frequency modulation (PLL reference only)
- Amount: 0-1 controls pitch drift depth
- Rate: 0-1 controls drift LFO speed
- Separate L/R phases for stereo movement

**Noise:**
- White noise injection
- Follows volume envelope
- Adds texture and breath to sounds

**Tube Saturation:**
- Asymmetric soft clipping
- Harder on positive, softer on negative half-cycles
- Adds warmth and compression

**Distortion:**
- Amount: 0-1 controls drive intensity (up to 50x gain)
- Threshold: 0.1-1.0 controls soft clipping threshold
- Aggressive volume compensation for consistent perceived loudness
- Applied after tube saturation for maximum harmonic content

### Master Highpass Filter

A 2nd-order Butterworth SVF highpass filter on the master output, placed before volume and limiter.

**Frequency Modes:**
| Mode | Cutoff | Description |
|------|--------|-------------|
| OFF | — | No filtering (default) |
| 35 | 35 Hz | Removes sub-bass rumble |
| 80 | 80 Hz | Tightens low end |
| 120 | 120 Hz | Cuts mud frequencies |
| 220 | 220 Hz | Aggressive bass cut |

**Boost Modes:**
| Mode | Q | Description |
|------|---|-------------|
| FLAT | 0.707 | Butterworth — maximally flat passband (default) |
| MED | 2.0 | Resonant peak just above cutoff (same as SAW tight filter) |
| HIGH | 4.0 | Aggressive resonant peak for bass emphasis |

**Sub Routing:**
| Mode | Description |
|------|-------------|
| OUT | Sub oscillator bypasses HPF — preserves clean sub bass (default) |
| IN | Sub oscillator passes through HPF — filters sub along with main signal |

**Implementation:** State Variable Filter (SVF) topology, f64 precision. Processes stereo independently.

### Brilliance Filter

A high-shelf exciter placed after the Master HPF in the signal chain. Extracts high-frequency content via an SVF highpass at 4.5kHz, optionally saturates it, and mixes it back into the signal.

**Parameters:**
| Param | Range | Description |
|-------|-------|-------------|
| Amount (AMT) | 0.0-1.0 | High-shelf boost level — how much processed highs are added back (0 = off) |
| Drive (DRV) | 0.0-1.0 | Saturation intensity on extracted highs (0 = clean shelf boost, 1 = heavy harmonic exciter) |

**How it works:**
1. SVF highpass (Q=0.5, 4.5kHz) extracts high-frequency content
2. Drive applies tanh saturation to the extracted highs, generating new harmonics above the cutoff
3. The processed highs are mixed back: `output = input + amount × saturated_highs`

**Character:**
- Amount alone (Drive=0): Clean high-shelf boost — adds air and presence
- Amount + Drive: Harmonic exciter — creates new high-frequency harmonics that weren't in the original signal, adding sparkle and brilliance
- Especially effective on PLL, FM, and wavefolder timbres where there is already rich harmonic content to excite

**Implementation:** SVF topology, f64 precision, same as Master HPF. Processes stereo independently.

## Envelopes

### Volume Envelope

Controls amplitude of all oscillators.

**Parameters:**
| Stage | Range | Shape |
|-------|-------|-------|
| Attack | 1-400 ms | -5 to +5 (log→lin→exp) |
| Decay | 1-1000 ms | -5 to +5 |
| Sustain | 0-1 | - |
| Release | 1-1000 ms | -5 to +5 |

**Anti-Click Behavior:**
- Minimum attack time: 1ms (2ms on retrigger for smoother transitions)
- Minimum release time: 1ms
- Velocity changes are smoothed over 5ms to prevent amplitude discontinuities
- Master volume changes are smoothed over 20ms
- Oscillator phases are randomized on note trigger to avoid consistent click artifacts
- VPS uses `limit_v()` for DC offset prevention, PLL uses DC blocking filter for colored mode

**Special:** Decay time can be modified per-note by the sequencer's decay modifier system.

## LFO System

Three independent LFOs with flexible routing.

**Per-LFO Parameters:**
| Param | Description |
|-------|-------------|
| Rate | 0.01-50 Hz (free running) |
| Waveform | Sine, Triangle, Saw, Square, S&H |
| Tempo Sync | Lock to BPM divisions |
| Sync Division | 1/1 to 1/32, dotted, triplet |
| Sync Source | Cross-modulation from other LFO |
| Phase Mod | Amount of sync influence |

**Modulation Slots:** Each LFO has 2 destination slots.

**Available Destinations:**
- PLL: Damping, Influence, Track Speed, FM Amount, Cross Feedback, Overtone (Burst), Range, Multiplier (discrete), Multiplier Direct (continuous)
- VPS: D parameter, V parameter, Stereo V Offset, Stereo D Offset, Fold, Shape Amount
- Coloration: Drift, Tube Drive
- Volumes: PLL, VPS, Sub

## Modulation Step Sequencer

A tempo-synced 16-step modulation sequencer with 303-style tie/glide and controllable slew. Provides rhythmic modulation alongside the LFOs.

**Parameters:**
| Param | Range | Default | Description |
|-------|-------|---------|-------------|
| Steps 1-16 | -1.0 to +1.0 | 0.0 | Bipolar step values |
| Ties | bitmask (u16) | 0 | Tie flags per step |
| Division | 1/1 to 1/16T | 1/8 | Step rate (same divisions as LFOs) |
| Slew | 0-200 ms | 5.0 | Transition smoothing for non-tied steps |

**Modulation Slots:** 2 destination slots (same as LFOs), each with destination and bipolar amount.

**Processing:**
1. Step frequency derived from BPM and selected division
2. Phase advances 0.0 to 16.0, wrapping at 16
3. Current step index = floor(phase) % 16
4. If current step has tie flag: linearly interpolate to next step value based on fractional phase
5. If no tie: target is current step value, smoothed by slew
6. Output (-1.0 to 1.0) feeds into ModulationValues, accumulated with LFO modulation

**Sound Design Tips:**
- Tie adjacent steps for smooth glide transitions (303-style)
- High slew values create smooth, wavering modulation from step patterns
- Combine with LFO modulation for complex rhythmic textures
- Route to PLL parameters for evolving timbral sequences

## Oversampling

Configurable oversampling to reduce aliasing at the cost of CPU.

| Factor | Description | CPU Impact |
|--------|-------------|------------|
| 1x | No oversampling | Lowest |
| 4x | Good quality | Moderate |
| 8x | High quality | Higher |
| 16x | Very high quality | High |
| 32x | Excellent quality | Very high |
| 64x | Near-perfect quality | Extreme |
| 128x | Maximum quality | Highest |

**Base Rate Option:** Force minimum 88.2kHz internal rate for extra quality at lower host sample rates.

## Signal Flow Summary

```
                    ┌───────────────────┐
                    │    Sequencer      │
                    │   (note/gate)     │
                    └─────────┬─────────┘
                              │
┌─────────────────────────────▼──────────────────────────────┐
│                       VOICE                                 │
│  ┌─────────┐  ┌─────────┐                                  │
│  │   VPS   │  │   PLL   │←──FM──┐                          │
│  │ Osc L/R │  │ Osc L/R │       │                          │
│  └────┬────┘  └────┬────┘   ┌───┴───┐                      │
│       │             │        │FM Osc │                      │
│       └──────┬──────┘        └───────┘                      │
│              ▼                                               │
│        ┌──────────┐                                          │
│        │ Mix + Vol│                                          │
│        │ Envelope │                                          │
│        └────┬─────┘                                          │
│           ▼                                                │
│   ┌────────────────┐                                       │
│   │  Coloration    │                                       │
│   │ Ring,Fold,Drift│                                       │
│   │ Noise,Tube,Dist│                                       │
│   └───────┬────────┘                                       │
│           │                                                │
│           ▼                                                │
│      Master Volume                                         │
└────────────┬───────────────────────────────────────────────┘
             │
     Main ───┤                ┌───────┐
             │           ┌────│  Sub  │────┐
             ▼           │    └───────┘    │
      ┌──────────────┐   │  [IN]     [OUT] │
      │ Master HPF   │◄──┘                 │
      └──────┬───────┘                     │
             ├─────────────────────────────┘
             ▼
      ┌──────────────┐
      │ Brilliance   │ (Amount + Drive)
      └──────┬───────┘
             ▼
      ┌──────────────┐
      │   Limiter    │
      └──────┬───────┘
             ▼
         Stereo Out
```
