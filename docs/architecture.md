# Architecture

Monophonic synthesizer + probability sequencer. Rust, nih-plug, egui.

## Stack

| Component | Technology |
|-----------|------------|
| Language | Rust nightly (portable SIMD) |
| Plugin | nih-plug (VST3, CLAP, Standalone) |
| DSP | Custom f64 (oversampling, oscillators, distortion) |
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
│              │  PLL ──┼─▶ Mix → Filter → Out         │
│              │  SAW ──┘          │                    │
│              └───────────────────┘                    │
└─────────────────────────────────────────────────────┘
```

## Audio Processing (per sample)

1. **Envelope** → Volume envelope
2. **Parameter slewing** → All continuous params smoothed
3. **Oversampling loop** (1x–128x configurable):
   - VPS oscillator (if enabled)
   - PLL oscillator with FM (if enabled)
   - SAW oscillator (if enabled)
   - Mix oscillators
4. **Downsample** → Anti-aliased to DAW rate
5. **Ladder Filter** (if enabled) → 4/8-pole ladder with 4× oversampling (polyphase FIR upsampling, Butterworth downsampling), dedicated filter envelope (ADSR with shapes), env range 1-8 octaves, drive boost (OFF/+12dB/+24dB/+48dB). Upsampler: 64-tap Kaiser-windowed sinc (16 taps/phase, beta=7.857, -89dB image rejection).
6. **Coloration** → Sub oscillator added
7. **Stereo Control** → Width, mono bass crossover
8. **Box Cut** → Notch at ~400Hz
9. **Master HPF** → Butterworth (Off/35/80/120/220Hz)
10. **Brilliance** → High-shelf exciter
11. **Reverb** (if enabled) → Early reflections + 8-channel FDN late reverb with Hadamard mixing, input diffusion, modulated delay lines, RT60-compensated decay, feedback saturation, ducking, stereo decorrelation
12. **Pitched Looper** → Bar-synced pitched loop capture/playback
13. **Compressor** (if enabled) → Feed-forward VCA with soft-knee, program-dependent release, assignable routing (master/looper/reverb IN/OUT)
14. **Limiter** → Output protection

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
