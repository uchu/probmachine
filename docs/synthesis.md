# Synthesis Engine

## Oscillators

### VPS (Variable Phase Shaping)

Waveshaping oscillator using synfx-dsp's VPSOscillator. **D** controls phase distortion, **V** morphs timbre. Stereo V/D offsets create width through different L/R parameters.

**Phase modes:** FREE (random phase on trigger) or SYNC (resets on PLL reference cycle → hard sync character).

**Shape types:** SOFT (tanh clipping) or FOLD (foldback distortion). Independent from the Fold parameter which uses sine-based wavefolding.

**Tips:** Low D + mid V = smooth. High D + low V = aggressive. Stereo offsets together = complex width. PLL SYNC + different reference octave = classic hard sync sweeps.

### PLL (Phase-Locked Loop)

Novel synthesis modeling analog PLL circuits. VCO tracks a reference oscillator but can be pushed into chaos.

**Core parameters:** Track speed (VCO lock speed), Damping (loop filter), Influence (phase coupling), Multiplier (1–64 freq ratio), Feedback (output→reference).

**Two detector modes:** AnalogLikePD (XOR-style, smooth) and EdgePFD (edge-triggered, aggressive).

**Precision toggle (PREC):**
- ON: PLL-theory loop — cubic speed curve, ωn/ζ-based coefficients, sub-sample edges
- OFF: Legacy loop — linear speed, ad-hoc coefficients, looser character

**Advanced:** Burst (overtrack intensity), Loop Saturation (integrator limit), Color (harmonic saturation), Edge Sensitivity, Range (lock bandwidth — 0=slow hunting, 1=fast lock).

**Stereo PLL:** Independent L/R via damping offset, track offset, phase offset, cross-feedback.

**FM:** Amount, Ratio (integer), Envelope-to-FM.

**Tips:** Track < 0.3 = gliding. 0.3–0.7 = stable. > 0.7 = overtrack bursts. Low damping = ringy/resonant. High = smooth. Low Range = analog hunting. FAST/SLOW toggle = tempo-synced multiplier slew.

### SAW

PolyBLEP sawtooth with DC blocking, waveshaping (3 types), wavefold (1X/PI modes). Sample-rate-aware DC block preserves bass at high oversampling.

### Sub

Pure sine, one octave below base frequency. Routed separately — bypasses Master HPF by default (OUT mode) to preserve clean sub bass.

## Effects Chain

### Coloration

Applied to oscillator mix (all at DAW rate, not oversampled):
- **Ring Mod**: VPS × PLL multiplication
- **Wavefold**: Sine-based progressive folding
- **Drift**: Slow random pitch modulation (PLL reference only, separate L/R phases)
- **Noise**: White noise following volume envelope
- **Tube**: Asymmetric soft clipping (harder positive, softer negative)
- **Distortion**: Up to 50× gain with soft threshold clipping + volume compensation

### Master HPF

2nd-order Butterworth SVF highpass. Modes: Off/35/80/120/220Hz. Boost modes: Flat (Q=0.707), Medium (Q=2.0), High (Q=4.0). Sub routing: OUT bypasses HPF, IN passes through.

### Brilliance

High-shelf exciter at 4.5kHz. Amount = boost level, Drive = tanh saturation on extracted highs generating new harmonics. Clean shelf at Drive=0, harmonic exciter at high Drive.

## Envelopes

Volume ADSR: Attack 1–5000ms, Decay 1–10000ms, Sustain 0–1, Release 1–10000ms. Each stage has shape control (-5 to +5, log→exp). Anti-click: 1ms min attack, 2ms on retrigger, 5ms velocity smoothing, 20ms volume smoothing.

## LFO System

3 independent LFOs, each with 2 mod destination slots.

**Waveforms:** Sine, Triangle, Saw, Square, Sample&Hold. Free-run (0.01–50Hz) or tempo-synced (1/1 to 1/32 including dotted/triplet).

**Cross-modulation:** Each LFO can use another as phase modulation source.

**Destinations:** PLL (Damp, Infl, Track, FM, XFB, Burst, Range, Vol, Mult discrete, Mult continuous), VPS (D, V, VΔ, DΔ, Fold, Shape, Vol), SAW (Fold, Shape, Vol), Sub (Vol), Coloration (Drift, Tube).

## Modulation Step Sequencer

16-step bipolar (-1 to +1) sequencer with 303-style ties and slew. Tempo-synced (same divisions as LFOs). 2 destination slots.

Tied steps: linear interpolation between values (glide). Non-tied: smoothed by slew parameter (0–200ms).

## Signal Flow

```
Sequencer → note/gate
     ↓
VPS + PLL (+ FM) → Mix × Envelope → Coloration → Sub added
     ↓
Main ──→ Master HPF ←── Sub [IN mode]
     │                     │
     ├─────────────────────┘ Sub [OUT mode]
     ↓
Brilliance → Limiter → Stereo Out
```
