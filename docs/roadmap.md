# Roadmap

Pending improvements and ideas. Completed items removed — check git history or code.

## Optimization

### High Impact

**Block-Based LFO Processing** — LFOs computed per-sample but change slowly. Process once per block (64–256 samples), interpolate within block. Est. 20–40% LFO system gain.

**Lock-Free State Sharing** — Replace Mutex on NotePool and strength grid with lock-free structures. Eliminates rare priority inversion.

**Lazy Parameter Updates** — Cache filter coefficients, recalculate only on change. Pre-compute envelope curves on parameter change. Est. 5–10% overall.

## PLL Enhancements

### Range Expansions

| Item | Description | Effort |
|------|-------------|--------|
| Sub-octave multipliers | Add 0.5, 0.25 to mult list | Low |
| Loop saturation upper bound | 1–500 → 1–1000 | Trivial |
| Edge sensitivity upper bound | 0.001–0.2 → 0.001–0.5 | Trivial |

### VCO Output

| Item | Description | Effort |
|------|-------------|--------|
| Wavefolder on VCO | sin(signal × fold × PI), richer than cubic saturation | Low-Med |
| Ring mod mode | PLL VCO × reference signal for metallic character | Low |

### Phase Detector

**PD Mode Morph** — Blend AnalogPD and EdgePFD continuously instead of switching. `error = analog × (1-morph) + edge × morph`. Low effort.

### Sync Modes

| Item | Description | Effort |
|------|-------------|--------|
| Hard sync | Reset VCO phase on reference cycle, classic sync timbres | Low |
| Soft sync | Blend phase toward 0: `phase × (1-amount)`, continuous control | Low |

### Feedback & Chaos

| Item | Description | Effort |
|------|-------------|--------|
| DC offset in PD | Systematic detuning even when "locked" | Trivial |
| Resonant loop filter | Ringing at loop bandwidth frequency | Medium |

### Topology

| Item | Description | Effort |
|------|-------------|--------|
| VPS→PLL reference routing | VPS output feeds PLL reference input (mix knob) | Medium |
| Dual coupled PLL | Two PLLs cross-feeding references | High |
| Cascade PLL | First PLL output → second PLL reference | High |

### Modulation

| Item | Description | Effort |
|------|-------------|--------|
| Envelope→Loop params | Env mod for Track Speed, Damping, Multiplier | Low-Med |
| Noise injection selector | Reference / loop filter / VCO phase | Low-Med |

## Sound Design

### New Oscillator Types

- Additive (per-harmonic control, spectral morphing)
- Wavetable (loadable, position as mod dest)
- Noise generator improvements (pink, brown, crackle, S&H)

### Filters

- Comb filter, phaser allpass chains, waveguide resonators
- Filter FM (audio-rate cutoff mod)
- Serial/parallel filter routing

### Effects

- Tempo-synced delay (ping-pong, tape)
- Bit crusher / sample rate reduction
- Chorus / flanger / phaser
- FFT freeze, spectral smear
- Frequency/pitch shifter

### Modulation

- Envelope followers (self-modulating patches)
- Perlin noise / drunk walk random
- Macro controls (single knob → multiple destinations)

## Sequencer

- Euclidean rhythm generator
- Pattern chaining (multiple bars, MIDI triggers, Markov)
- Probability evolution (see `suggestions/probability-evolution.md`)

## UI

- Oscilloscope / spectrum analyzer
- Level meters with peak hold
- MIDI learn
- Randomize function
- Preset morphing A/B
- XY pad controls

## Code Quality

- Split params.rs (~1200 lines) into parameter groups
- Proper error types instead of panic! in some places
- Unit tests for DSP, integration tests for signal chain
