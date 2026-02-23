# Device - Synthesis Engine

## Oscillator Types

Device features two distinct oscillator types plus a sub oscillator, each with unique character.

### VPS Oscillator (Variable Phase Shaping)

A waveshaping oscillator using synfx-dsp's VPSOscillator algorithm.

**Parameters:**
| Param | Range | Description |
|-------|-------|-------------|
| D | 0.0-1.0 | Phase distortion amount |
| V | 0.0-1.0 | Shape/timbre control |
| Stereo V Offset | 0.0-1.0 | V difference between L/R for width |
| Octave | -5 to +5 | Octave shift |
| Tune | -12 to +12 | Semitone offset |
| Fine | -1.0 to +1.0 | Fine tune (cents) |
| Fold | 0.0-1.0 | Wavefold amount for harmonic enrichment |
| Volume | 0.0-1.0 | VPS output level |

**Character:**
- D controls phase distortion (soft at 0, harsh at 1)
- V morphs the waveform character
- Stereo V offset creates rich width through different L/R timbres
- Clean, digital precision
- Built-in DC offset prevention via `limit_v()` parameter limiting

**Sound Design Tips:**
- Low D + mid V: Smooth, round tones
- High D + low V: Aggressive, buzzy timbres
- Stereo offset creates chorus-like width without pitch modulation
- Tune/Fine for detuning against PLL oscillator creates thick textures
- Fold adds harmonic richness, especially effective at higher D values

### PLL Oscillator (Phase-Locked Loop)

A novel synthesis method modeling analog PLL circuits. The VCO attempts to track a reference oscillator but can be pushed into unstable, chaotic behavior.

**Reference Section (Yellow):**
| Param | Range | Description |
|-------|-------|-------------|
| Oct | -5 to +5 | Reference octave |
| Tune | -12 to +12 | Semitone offset |
| Fine | -1.0 to +1.0 | Fine tune (cents) |
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
| FM Env | Filter envelope to FM depth |

**Character:**
The PLL oscillator excels at:
- Unstable, analog-like tones at high track speed
- Gliding, resonant behavior at low track speed
- Chaotic, textured sounds with high feedback
- Rich stereo imaging with offset parameters
- FM-like timbres through the FM section
- DC blocking filter on output handles colored mode saturation artifacts
- Click-free operation: VCO phase continues smoothly on retrigger

**Sound Design Tips:**
- Track < 0.3: Slow, gliding portamento character
- Track 0.3-0.7: Stable tracking, predictable pitch
- Track > 0.7: "Overtrack" mode - frequency bursts, instability
- High Damping: Faster settling, cleaner sound
- Low Damping: Ringy, resonant behavior
- Use stereo offsets for massive width without losing mono compatibility
- Range at 0: Very slow lock, creates characteristic analog PLL "hunting" behavior
- Range at 1: Fast lock, tight frequency tracking
- Low Range + any Multiplier: Creates musical portamento-like transitions between pitches
- FAST/SLOW toggle: Tempo-synced multiplier slew (FAST=1/16 note, SLOW=1/1 note duration)
- LFO to PLL Mult: Discrete modulation steps between multiplier values (1,2,4,8,16,32,64) with slew
- LFO to PLL Mult D: Direct continuous modulation of the frequency multiplier for evolving timbres

### Sub Oscillator

A simple sine oscillator one octave below the base frequency.

**Parameters:**
| Param | Range | Description |
|-------|-------|-------------|
| Volume | 0.0-1.0 | Sub level in mix |

**Signal Path:** Added post-reverb to remain clean and punchy, bypassing all effects processing.

## Effects Chain

### Coloration Section

A set of effects applied to the oscillator mix before filtering.

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

### Filter (Moog Ladder)

Classic 4-pole Moog ladder lowpass filter using the Stilson algorithm from synfx-dsp.

**Character:**
| Feature | Description |
|---------|-------------|
| Type | 4-pole ladder (24 dB/oct) |
| Mode | Low-pass only |
| Sound | Fat, warm, squelchy vintage character |
| Saturation | Built-in warm saturation via drive |
| Self-oscillation | Near resonance 1.0 (clamped to 0.98 for stability) |

**Parameters:**
| Param | Range | Description |
|-------|-------|-------------|
| Cutoff | 20 Hz - Nyquist×0.4 | Filter frequency |
| Resonance | 0-0.98 | Q factor (self-oscillates at high values) |
| Drive | 1-15 | Input saturation (tanh soft clipping) |
| Env Amount | -5000 to +5000 | Filter envelope modulation |

**Drive Behavior:**
| Value | Effect |
|-------|--------|
| 1.0 | Clean (no saturation) |
| 2.0-4.0 | Warm saturation |
| 5.0-10.0 | Heavy distortion |
| 10.0-15.0 | Aggressive clipping |

### Reverb (Dattorro)

High-quality plate reverb algorithm from synfx-dsp.

**Input Section:**
| Param | Range | Description |
|-------|-------|-------------|
| Pre-delay | 0-200 ms | Time before reverb onset |
| Input HPF | 20-2000 Hz | High-pass before reverb |
| Input LPF | 1000-20000 Hz | Low-pass before reverb |

**Reverb Character:**
| Param | Description |
|-------|-------------|
| Time Scale | Overall reverb size scaling |
| Diffusion | Early reflection density |
| Diffusion Mix | Early reflections amount |
| Decay | RT60-like tail length |

**Reverb Filters:**
| Param | Description |
|-------|-------------|
| HPF | High-pass in reverb loop |
| LPF | Low-pass in reverb loop |

**Modulation:**
| Param | Description |
|-------|-------------|
| Mod Speed | Internal LFO rate |
| Mod Depth | Pitch modulation amount |
| Mod Shape | LFO waveform |

**Special:**
| Param | Description |
|-------|-------------|
| Mix | Dry/wet balance |
| Ducking | Reduce reverb when input is loud |

**Smoothing:** Mix, ducking, and decay parameters are smoothed over 50ms for click-free transitions.

## Envelopes

Two ADSR envelopes with shape control per segment.

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

### Filter Envelope

Modulates filter cutoff.

**Parameters:** Same as volume envelope, plus:
- **Env Amount**: Bipolar modulation depth to cutoff

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
- VPS: D parameter, V parameter
- Filter: Cutoff, Resonance, Drive
- Coloration: Drift, Tube Drive
- Reverb: Mix, Decay
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
- Use with filter cutoff for rhythmic filter patterns
- Tie adjacent steps for smooth glide transitions (303-style)
- High slew values create smooth, wavering modulation from step patterns
- Combine with LFO modulation for complex rhythmic textures
- Route to PLL parameters for evolving timbral sequences

## Oversampling

Configurable oversampling to reduce aliasing at the cost of CPU.

| Factor | Description | CPU Impact |
|--------|-------------|------------|
| 1x | No oversampling | Lowest |
| 4x | Default, good quality | Moderate |
| 8x | High quality | Higher |
| 16x | Maximum quality | Highest |

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
│           ▼                                                │
│   ┌────────────────┐                                       │
│   │  Moog Filter   │←──Envelope                            │
│   └───────┬────────┘                                       │
│           ▼                                                │
│   ┌────────────────┐                                       │
│   │ Dattorro Reverb│                                       │
│   └───────┬────────┘                                       │
│           │                                                │
│     + ┌───┴───┐                                            │
│       │  Sub  │ (added post-reverb)                        │
│       └───────┘                                            │
│           │                                                │
│           ▼                                                │
│      Master Volume                                         │
└────────────┬───────────────────────────────────────────────┘
             ▼
         Stereo Out
```
