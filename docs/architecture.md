# Device - Architecture Overview

A monophonic synthesizer and probability-based sequencer built in Rust with nih-plug and egui.

## Technology Stack

| Component | Technology |
|-----------|------------|
| Language | Rust (nightly for portable SIMD) |
| Plugin Framework | nih-plug |
| DSP Library | synfx-dsp v0.5 |
| GUI Framework | egui + taffy layout |
| Plugin Formats | VST3, CLAP, Standalone |
| Audio Backends | JACK (primary), CoreAudio, ALSA, WASAPI |

## High-Level Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Device Plugin (lib.rs)                   │
├─────────────────────────────────────────────────────────────┤
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
│  │   Sequencer  │  │  Parameters  │  │     GUI      │      │
│  │  (mod.rs)    │  │ (params.rs)  │  │  (ui/mod.rs) │      │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘      │
│         └─────────────────┼──────────────────┘              │
│                           ▼                                  │
│               ┌─────────────────────┐                       │
│               │    SynthEngine      │                       │
│               │   (synth/mod.rs)    │                       │
│               └──────────┬──────────┘                       │
│                          ▼                                   │
│               ┌─────────────────────┐                       │
│               │       Voice         │                       │
│               │  (synth/voice.rs)   │                       │
│               │                     │                       │
│               │  VPS ────┐          │                       │
│               │  PLL ────┼─▶ Mix ─▶ Filter ─▶ Reverb ─▶ Out│
│               │  Sub ────┘          │                       │
│               └─────────────────────┘                       │
└─────────────────────────────────────────────────────────────┘
```

## Module Structure

```
src/
├── lib.rs              # Plugin entry, process callback
├── main.rs             # Standalone binary entry
├── params.rs           # All automatable parameters (~250 params)
├── midi.rs             # Full MIDI I/O processing
├── synth/
│   ├── mod.rs          # SynthEngine coordinator
│   ├── voice.rs        # Main voice with signal routing
│   ├── oscillator.rs   # VPS, PolyBLEP, PLL oscillators
│   ├── filter.rs       # Moog ladder filter (scalar + SIMD stereo)
│   ├── envelope.rs     # ADSR with shape control
│   ├── reverb.rs       # Dattorro reverb wrapper
│   ├── simd.rs         # Portable SIMD helpers (f64x2 stereo)
│   ├── limiter.rs      # Master output limiter
│   └── lfo.rs          # 3 LFOs with mod matrix
├── sequencer/
│   ├── mod.rs          # Probability sequencer logic
│   └── note_utils.rs   # MIDI note handling, NotePool
├── preset/
│   ├── mod.rs          # Preset module exports
│   ├── data.rs         # Preset data structures
│   ├── manager.rs      # Save/load functionality
│   └── defaults.rs     # Factory preset data
└── ui/
    ├── mod.rs          # UI module exports
    ├── page.rs         # Page enum and routing
    ├── navigation.rs   # Tab navigation
    ├── shared_state.rs # UI ↔ Audio thread state
    └── pages/
        ├── beat_probability.rs  # Pattern editor
        ├── length.rs            # Note duration controls
        ├── notes.rs             # Piano roll note selection
        ├── strength.rs          # 96-position strength grid
        ├── synth.rs             # Synthesis controls
        ├── modulation.rs        # LFO routing
        └── presets.rs           # Preset management
```

## Audio Processing Flow

### Per-Sample Processing (voice.rs)

1. **Envelope Generation** - Volume and filter envelopes
2. **Parameter Slewing** - All continuous parameters smoothed
3. **Oversampling Loop** (1x/4x/8x/16x configurable):
   - VPS oscillator processing (if enabled)
   - PLL oscillator with FM (if enabled)
   - Mix oscillators
   - Coloration effects (ring mod, wavefold, drift, noise, tube)
   - SVF filter with envelope modulation
   - Dattorro reverb processing
   - Sub oscillator added post-reverb
4. **Downsampling** - Anti-aliased reduction to DAW rate
5. **Master Volume** - Final output level

### Signal Precision

| Stage | Precision |
|-------|-----------|
| Phase accumulators | f64 |
| Oscillator DSP | f64 |
| Envelopes | f64 |
| Reverb (Dattorro) | f64 |
| Filter (Moog Ladder) | f64 (SIMD stereo) |
| Coloration | f64 (SIMD stereo) |
| Oversampling buffers | f32 |
| Plugin output | f32 |

### SIMD Stereo Processing (Integrated)

SIMD module (`simd.rs`) provides cross-platform stereo processing using Rust's portable SIMD (`std::simd`, nightly):
- **Type**: `f64x2` for parallel L/R processing
- **x86_64**: Compiles to SSE/AVX instructions
- **ARM64**: Compiles to NEON instructions (Raspberry Pi 5)
- **Fallback**: Scalar operations on unsupported platforms

Integrated SIMD components:
- `StereoMoogFilter`: 4-pole Stilson Moog ladder filter (f64 precision)
- `stereo_wavefold`: Sinusoidal wavefolding effect
- `stereo_tube_saturate`: Asymmetric tube saturation
- `stereo_distort_bram`: Bram de Jong waveshaper distortion

Utility processors available:
- `StereoSlewValue`, `StereoDCBlocker`, `StereoOnePoleLPF`

## Thread Safety

- **Audio Thread**: Zero-allocation processing, pre-allocated buffers
- **GUI Thread**: egui immediate mode rendering
- **State Sharing**:
  - `Arc<DeviceParams>` for parameter access
  - `Arc<SharedUiState>` with `Mutex` for note pool and strength grid
  - `AtomicU32/AtomicU64` for CPU load and preset version

## Key Design Decisions

### Monophonic Architecture
Single voice simplifies DSP while enabling CPU-intensive algorithms (PLL, 16x oversampling).

### Probability-Based Sequencer
Instead of fixed patterns, each beat position has a probability value. Multiple divisions (straight, triplet, dotted) compete for the same time slot.

### PLL Synthesis
Novel approach using Phase-Locked Loop for chaotic, analog-like behavior. The VCO tracks a reference but can be pushed into unstable states.

### Selective Oversampling
All oscillators, filter, and reverb run at the oversampled rate to prevent aliasing, but can be reduced for CPU savings.

### Parameter Organization
~250 parameters organized into logical groups:
- Beat probabilities (152 params: 63 straight + 45 triplet + 44 dotted)
- Synthesis (~50 params)
- LFOs (30 params - 3 LFOs × 10 params each)
- Modifiers (18 params for length/decay/position)

## Performance Characteristics

| Metric | Typical Value |
|--------|---------------|
| CPU (4x OS) | 8-15% single core |
| CPU (16x OS) | 25-40% single core |
| Memory | ~50MB |
| Latency | < 1ms (excluding host) |
| Binary Size | ~5MB stripped |

## Build Profiles

```toml
[profile.release]
lto = "thin"
strip = "symbols"

[profile.profiling]
inherits = "release"
debug = true
strip = "none"
```

## MIDI Support

### MIDI Input
- **Note Events**: NoteOn/NoteOff from external MIDI controllers (any channel)
- **Control Changes**: CC messages stored in state, accessible for parameter modulation
- **14-bit CC**: Support for high-resolution control (CC 0-31 + CC 32-63)
- **NRPN**: Non-Registered Parameter Number tracking

### MIDI Output
- **Sequencer Notes**: Generated notes output as MIDI NoteOn/NoteOff
- **Velocity**: Per-note velocity passed through to MIDI output
- **Sample-accurate timing**: All MIDI events aligned to buffer positions
- **Host tempo sync**: Reads tempo from DAW transport

### MIDI Processing Architecture
```
MIDI Input → MidiProcessor.input → MidiState
                                   ├── CC tracking
                                   └── External note events

Sequencer → SynthEngine.process_block() → midi_events_buffer
                                              ↓
MidiProcessor.output ← note_on/note_off_from_sequencer()
       ↓
context.send_event() → MIDI Output
```

## Version History

- **v1.7.0**: Full MIDI I/O - note input/output, CC handling, transport sync
- **v1.6.0**: SIMD stereo DSP - Moog filter, wavefold, tube saturation, distortion
- **v1.5.0**: Portable SIMD infrastructure for future stereo DSP optimization
- **v1.4.0**: JACK as primary backend, multi-platform support
- **v1.3.0**: Musical note selection system, strength grid
- **v1.2.0**: Oversampling refactor, sub oscillator fix
- **v1.1.0**: f64 precision throughout DSP chain
