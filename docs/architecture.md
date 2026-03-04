# Architecture

Monophonic synthesizer + probability sequencer. Rust, nih-plug, egui.

## Stack

| Component | Technology |
|-----------|------------|
| Language | Rust nightly (portable SIMD) |
| Plugin | nih-plug (VST3, CLAP, Standalone) |
| DSP | synfx-dsp v0.5 |
| GUI | egui + taffy layout |
| Audio | JACK, CoreAudio, ALSA, WASAPI |

## System Diagram

```
┌─────────────────────────────────────────────────────┐
│                   PhaseBurn Plugin                    │
│  ┌────────────┐  ┌────────────┐  ┌────────────┐     │
│  │  Sequencer │  │ Parameters │  │    GUI     │     │
│  └─────┬──────┘  └─────┬──────┘  └─────┬──────┘     │
│        └────────────────┼───────────────┘             │
│                         ▼                             │
│              ┌───────────────────┐                    │
│              │    SynthEngine    │                    │
│              └────────┬──────────┘                    │
│                       ▼                               │
│              ┌───────────────────┐                    │
│              │      Voice        │                    │
│              │  VPS ──┐          │                    │
│              │  PLL ──┼─▶ Mix → Coloration → Out     │
│              │  Sub ──┘          │                    │
│              └───────────────────┘                    │
└─────────────────────────────────────────────────────┘
```

## Audio Processing (per sample)

1. **Envelope** → Volume envelope
2. **Parameter slewing** → All continuous params smoothed
3. **Oversampling loop** (1x–128x configurable):
   - VPS oscillator (if enabled)
   - PLL oscillator with FM (if enabled)
   - Mix oscillators
   - Coloration (ring mod, wavefold, drift, noise, tube, distortion)
   - Sub oscillator added post-coloration
4. **Downsample** → Anti-aliased to DAW rate
5. **Master HPF** → Butterworth (Off/35/80/120/220Hz)
6. **Brilliance** → High-shelf exciter
7. **Master Volume**
8. **Limiter** → Output protection

PLL runs at oversampled rate; VPS, sub, coloration run at DAW rate.

### Precision

Phase accumulators, oscillators, envelopes, filters: all f64. Oversampling buffers and output: f32.

### SIMD

`f64x2` stereo processing via `std::simd` (nightly). Compiles to SSE/AVX (x86) or NEON (ARM/Pi 5). Used in wavefold, tube saturation, distortion.

## Thread Model

- **Audio thread**: Zero-allocation, pre-allocated buffers
- **GUI thread**: egui immediate mode
- **Sharing**: `Arc<DeviceParams>` for params, `Arc<SharedUiState>` with `Mutex` for NotePool/strength, atomics for CPU load/preset version

## MIDI Architecture

```
DAW input → context.next_event() ──┐
                                    ├──→ MidiProcessor → MidiState
Direct device (midir) → queue ─────┘    ├── CC tracking
                                        └── External notes
                                             ↓
                                        MidiModeProcessor
                                        ├── Passthrough → voice
                                        ├── Chord Follow → NotePool
                                        └── Accompaniment → harmonic analysis → NotePool

Sequencer → midi_events_buffer → MidiProcessor.output
  → context.send_event() (DAW)
  → MidiDeviceManager.flush_output() (direct)
```

### MIDI Input Modes

| Mode | External MIDI | Sequencer | Voice driven by |
|------|--------------|-----------|-----------------|
| Passthrough | Plays voice directly | Also plays | Both (mono, last wins) |
| Chord Follow | Updates NotePool from held chord | Plays from pool | Sequencer only |
| Accompaniment | Feeds harmonic analysis | Plays from analysis pool | Sequencer only |

**Chord Follow:** Held notes → NotePool where each note becomes a selection with velocity-mapped chance.

**Accompaniment:** Notes accumulated per bar, analyzed at bar boundaries (12 roots × 9 scales with hysteresis). NotePool generated from detected key. Harmonic memory persists across rewinds.

### Direct MIDI (Standalone)

Uses `midir` for device enumeration. Input callback → lock-free queue → audio thread drain via `try_lock()`. Settings persist to `Device/settings.json`.

## Key Design Decisions

- **Monophonic**: Single voice enables CPU-intensive algorithms (PLL, high oversampling)
- **Probability sequencer**: Each beat has a probability value; multiple divisions compete for the same time
- **PLL synthesis**: Phase-Locked Loop VCO tracks reference but can be pushed into instability
- **Selective oversampling**: Only PLL runs at oversampled rate for CPU savings
- **~250 parameters**: Organized as beat probabilities (152), synthesis (~50), LFOs (30), modifiers (18)
