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

## Parameter & Modulation Smoothing

All parameter transitions are per-sample to prevent zipper noise and artifacts:

- **`SlewValue`** (linear rate limiter): Most UI parameters. Rate = max change per sample for a given ms time.
- **`OnePoleSlewValue`** (true exponential one-pole): VPS/SAW shape and fold params. Uses `1 - exp(-1/samples)` coefficient for natural exponential settling.
- **`StereoSlewValue`** (stereo exponential one-pole): Stereo signal-path smoothing. Same exponential coefficient as OnePoleSlewValue.

### Modulation slew architecture

Three-tier design eliminates double-slewing while maintaining click protection:

1. **Source-level slew** — Only where needed:
   - LFO S&H: user-configurable slew (default 5ms) smooths random step jumps
   - LFO continuous waveforms (Sine/Tri/Saw/Square): no output slew (waveform shape preserved)
   - ModSequencer: user-configurable slew for non-tied steps; smoothstep (S-curve) interpolation for tied steps; per-step probability; variable length (1–16); note-on retrigger; 4 routing slots
2. **Voice mod_slew** (0.5ms) — Minimal anti-click protection on all mod destinations. Prevents clicks from routing changes without reducing modulation depth or rounding waveform shapes.
3. **Parameter slew** (20-60ms) — Smooths UI control changes. Only affects the base target value, not modulation.

### Filter cutoff modulation

LFO/ModSeq modulation uses **octave-based (logarithmic) scaling** (±5 octaves): `cutoff × 2^(mod × 5)`. This gives perceptually consistent modulation depth across the entire frequency range. Key tracking and filter envelope modulation use semitone-based scaling internally in the ladder filter.

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
11. **Pitched Looper** → Bar-synced pitched loop capture/playback. Input routed from: individual oscs (VPS/PLL/SAW), post-filter (FLTR), or pre-master (PRE, post-HPF/BoxCut/Brilliance). Priority: PRE > FLTR > individual oscs.
12. **Reverb** (if enabled) → Early reflections + 8-channel FDN late reverb with Hadamard mixing, input diffusion, modulated delay lines, RT60-compensated decay, feedback saturation, ducking, stereo decorrelation. Send from: individual oscs (VPS/PLL/SAW/SUB), post-filter (FLTR, exclusive with oscs), looper contribution only.
13. **Compressor** (if enabled) → Feed-forward VCA with adjustable soft-knee, true-peak detection, program-dependent release, stereo link control, auto makeup gain, parameter smoothing, lookahead with host PDC reporting and crossfaded delay transitions (64-sample smoothstep), assignable routing (master/looper/reverb IN/OUT)
14. **Limiter** → Output protection

PLL runs at oversampled rate; VPS, SAW, sub, coloration run at DAW rate with configurable oversampling (1×–128×). Oversampling anti-alias filter: 8th-order cascaded Butterworth at 0.86× Nyquist for strong alias rejection. SAW uses PolyBLEP as baseline anti-aliasing; VPS relies on oversampling alone (cosine readout provides inherent bandwidth limiting). Both benefit from 2× or higher oversampling for cleanest results at high pitches.

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
