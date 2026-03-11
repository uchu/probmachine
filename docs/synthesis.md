# Synthesis Engine

## Oscillators

### VPS (Variable Phase Shaping)

Waveshaping oscillator based on Vector Phase Shaping (Kleimola et al.). **D** controls phase distortion, **V** morphs timbre. Stereo V/D offsets create width through different L/R parameters. Custom f64 implementation for full double-precision throughout the signal path.

**Phase modes:** FREE (random phase on trigger) or SYNC (resets on PLL reference cycle → hard sync character).

**Shape types:** SOFT (tanh clipping) or FOLD (foldback distortion). Independent from the Fold parameter which uses sine-based wavefolding.

**Stereo drift:** When stereo V/D offsets are active, L/R oscillators run identical frequencies but with different phase shaping parameters. The different `phi_vps` trajectories cause gradual phase divergence between channels, creating an evolving stereo image — this is intentional and produces natural width. Oversampling recommended at 2× or higher for cleanest results, especially with extreme D/V settings.

**Formant mode (FMT toggle):** Extends V range from [0,1] to [1,10], enabling formant synthesis per the original Kleimola paper. When FMT is ON, the V slider maps to `1 + V×9`, placing formant peaks at harmonics `(2v-1)×f0`. V=0 → fundamental, V≈0.17 → 3rd harmonic, V=0.5 → 10th harmonic, V=1.0 → 19th harmonic. Formants are strongest at exact integer harmonics (v multiples of 0.5). Higher oversampling recommended for high V values to manage aliasing from the increased harmonic content.

**Tips:** Low D + mid V = smooth. High D + low V = aggressive. Stereo offsets together = complex width. PLL SYNC + different reference octave = classic hard sync sweeps. FMT ON + sweep V = vocal/resonant formant character.

### PLL (Phase-Locked Loop)

Novel synthesis modeling analog PLL circuits. VCO tracks a reference oscillator but can be pushed into chaos.

**Core parameters:** Track speed (VCO lock speed), Damping (loop filter), Influence (phase coupling), Multiplier (1–64 freq ratio), Feedback (output→reference).

**Two detector modes:** AnalogLikePD (XOR-style, smooth) and EdgePFD (edge-triggered, aggressive).

**Precision toggle (PREC):**
- ON: PLL-theory loop — cubic speed curve, ωn/ζ-based coefficients, sub-sample edges
- OFF: Legacy loop — linear speed, ad-hoc coefficients, looser character

**Advanced:** Burst (overtrack intensity), Loop Saturation (integrator limit), Color (harmonic saturation), Edge Sensitivity, Range (lock bandwidth — 0=slow hunting, 1=fast lock).

**Stereo PLL:** Fully independent L/R signal paths — each channel has its own reference oscillator, damping offset, track offset, and phase offset. EdgePFD mode correctly tracks per-channel reference frequency for accurate stereo phase detection.

**FM:** Amount, Ratio (integer), Envelope-to-FM.

**Internal DSP:** Range-reduced 9th-order Taylor sine (~-108dB THD), robust phase wrapping (handles multi-cycle excursions), sample-rate-compensated feedback filter (consistent ~1479Hz cutoff across all oversampling ratios).

**Tips:** Track < 0.3 = gliding. 0.3–0.7 = stable. > 0.7 = overtrack bursts. Low damping = ringy/resonant. High = smooth. Low Range = analog hunting. FAST/SLOW toggle = tempo-synced multiplier slew.

### SAW

PolyBLEP sawtooth with DC blocking, waveshaping (3 types), wavefold (1X/PI modes), fine tuning (±1 semitone). DC blocker at 10Hz provides fast transient settling while preserving bass. Oversampling recommended for best anti-aliasing at high pitches — PolyBLEP provides baseline alias suppression at 1× but benefits from 2× or higher.

### Sub

Pure sine, one octave below base frequency. Two routing toggles:
- **Filter**: OUT (default) bypasses the ladder filter, IN routes sub through the filter with all other oscillators.
- **HPF**: OUT (default) bypasses Master HPF, IN passes through.

## Ladder Filter

Generalized N-pole transistor ladder filter (D'Angelo & Valimaki topology). Switchable between 4-pole (24dB/oct) and 8-pole (48dB/oct) modes. Placed after oscillator mixing, before master volume scaling.

### Architecture
- 4 or 8 cascaded one-pole sections with nonlinear feedback
- TPT (topology-preserving transform) integrators using `tan(π·fc/fs)` frequency warping
- Per-stage nonlinear TPT integrators (D'Angelo & Välimäki) — saturation on both input and state with proper `g/(1+g)` scaling
- 4× oversampling: 64-tap polyphase FIR upsampler (Kaiser β=7.86, -89dB image rejection) + 8th-order Butterworth anti-alias downsampler (cutoff 0.9×Nyquist)
- Bounded Padé-approximant fast tanh (input-clamped to ±3 for C1-continuous saturation ceiling)
- Gain-normalized saturation — character parameter shapes the curve without shifting cutoff frequency
- Empirical resonance compensation (1 + res×1.5) for loudness-matched passband across resonance range
- Precomputed cutoff slew coefficient (exp() moved out of per-sample path)
- Batched parameter update via `FilterParams` struct (single call per sample instead of 17 setters)
- Sample-rate-adaptive DC blocker (~10Hz corner at any sample rate)
- f64 precision throughout (matches voice pipeline)
- Independent L/R state for true stereo processing
- Single feedback path from last active stage — NOT cascaded 4-pole filters

### Pole Modes

**4-pole (24dB/oct):** Classic Moog ladder topology. Resonance peak at cutoff frequency. Self-oscillation threshold k=4 (Barkhausen: k=1/cos⁴(π/4)).

**8-pole (48dB/oct):** True 8-stage ladder inspired by the Doepfer A-108. Resonance peak displaced to ~41% of cutoff (unique sonic character). Self-oscillation threshold k≈1.884 (Barkhausen: k=1/cos⁸(π/8), per D'Angelo & Välimäki 2014). Almost no software synth offers a true nonlinear 8-pole ladder with pole-mixing.

### Modes (derived from ladder taps)

**4-pole modes:**
| Mode | Slope | Output |
|------|-------|--------|
| LP24 | -24dB/oct | stage[3] — classic fat ladder lowpass |
| LP12 | -12dB/oct | stage[1] — gentler, more open |
| BP12 | Band | stage[1] - stage[3] — resonant peak |
| NTCH | Notch | input - 2·stage[1] + 2·stage[2] |
| HP24 | -24dB/oct | input - 4·s[0] + 6·s[1] - 4·s[2] + s[3] |

**8-pole modes:**
| Mode | Slope | Output |
|------|-------|--------|
| LP48 | -48dB/oct | stage[7] — ultra-steep lowpass |
| LP24 | -24dB/oct | stage[3] — 4-pole tap from 8-stage chain |
| BP24 | Band | stage[3] - 4·s[4] + 6·s[5] - 4·s[6] + s[7] |
| NTCH | Notch | input - 4·s[0] + 6·s[1] - 4·s[2] + 2·s[3] |
| HP48 | -48dB/oct | binomial-expansion highpass from all 8 taps |

### Parameters
- **Poles**: 4-POLE (24dB/oct) or 8-POLE (48dB/oct)
- **Cutoff** (20Hz–20kHz): Exponential scaling, modulation range ±5 octaves (logarithmic, perceptually consistent)
- **Resonance** (0–1.05): >1.0 enables self-oscillation, bass compensation applied
- **Drive** (0–1): Pre-filter saturation amount (1 + drive × 3 gain into tanh)
- **Drive Boost**: OFF (1×), +12dB, +24dB, +48dB — extra gain multiplier on drive for aggressive saturation
- **Key Track** (0–100%): Cutoff follows played note frequency (semitone-based, relative to A440)
- **Env Amount** (-1 to +1): Bipolar depth control for filter envelope modulation (env_amount × filter_env × range octaves)
- **Stereo Separation** (0–50%): Shifts L cutoff down and R cutoff up by the amount in octaves
- **Mode**: Pole-morph across 5 modes (labels change with pole count)

### Advanced Controls
- **Saturation Type**: Transistor (default, classic ladder — Padé tanh), Diode (harder knee, asymmetric), Tube (soft asymmetric, even harmonics). Selects the nonlinearity applied per stage.
- **Pole Morph** (0–1): Continuous blending between filter modes. Interpolates tap coefficients for smooth timbral transitions. Mode labels update dynamically based on pole count.
- **Filter FM** (0–1): Audio-rate frequency modulation of the cutoff. Modulates cutoff up to ±2 octaves at full depth. Creates metallic, bell-like, and chaotic timbres at high depths.
- **Feedback** (-1 to +1): Routes filter output back to input through soft-clip protection. Positive = standard feedback reinforcement, negative = phase-inverted feedback for hollow/cancellation tones. Feedback state is captured pre-resonance-compensation for stability.
- **Bass Lock** (0–1): Compensates low-frequency energy loss from resonance. Uses quadratic resonance scaling — inactive at zero resonance, engages progressively from res ~0.2, reaching full compensation at res ~0.67. Essential for bass-heavy patches.
- **Pole Spread** (0–1): Offsets the cutoff frequency of each cascaded stage. At 0 all poles are coincident (standard ladder). In 8-pole mode, spread offsets are distributed across all 8 stages for wider effect.
- **Resonance Character** (0–1): Controls saturation curve sharpness across all filter stages. Multiplies the input to each saturator by a hardness factor (1× to 4×), pushing signals deeper into the nonlinear region. Gain-normalized (divides by full hardness factor) so cutoff frequency remains stable regardless of character setting. Affects every stage equally — audible at any resonance level, not just high resonance.
- **Resonance Tilt** (-1 to +1): Offsets resonance between L/R channels. Negative = more resonance on L, positive = more on R. Creates stereo movement in the resonant peak without affecting cutoff.
- **Cutoff Slew** (0–1): Smooths cutoff changes with a one-pole lowpass. At 0, cutoff responds instantly. Higher values add portamento-like lag to all cutoff modulation — envelope, LFO, and direct control.

### Filter Envelope
Dedicated DAHDSR envelope for cutoff modulation, independent from the volume envelope. Same 64-bit precision Envelope implementation with shaped curves.

- **DAHDSR**: Attack (0.1–5000ms), Hold (0–5000ms), Decay (0.1–10000ms), Sustain (0–1), Release (0.1–10000ms, 3ms internal floor), each stage with shape control
- **Hold**: Delays decay by holding at peak value after attack completes. Skipped if < 0.1ms. Scales with key tracking.
- **Loop Mode**: OneShot (default) or LoopAHD — loops Attack→Hold→Decay while note held, creating repeating envelope shapes
- **Dip**: Retrigger dip (0–1) for percussive filter plucks on re-articulation
- **Range**: Maximum modulation depth in octaves (1–8). Combined with Env Amount: `env_mod = env_amount × envelope_output × range`
- **Copy from Vol Env**: UI button copies all volume envelope ADSR + dip params to filter envelope

### Signal Position
```
[VPS + PLL + SAW mix] → LADDER FILTER (× filter_env modulation) → × vol_envelope → × master_vol → output
```

### Modulation Targets
Cutoff, Resonance, Drive, and Env Amount are available as LFO/Step Mod destinations (indices 42–45). Advanced filter controls are also modulatable: Pole Morph (46), Filter FM (47), Feedback (48), Bass Lock (49), Pole Spread (50), Resonance Character (51), Resonance Tilt (52). Envelope extended targets: Env Hold (53), Env Key Track (54), Env Depth (55), Vel→Attack (56), Vel→Decay (57), Vel→Sustain (58), Filter Env Hold (59).

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

Custom 64-bit precision DAHDSR envelope (Attack-Hold-Decay-Sustain-Release). Attack 0.1–5000ms, Hold 0–5000ms, Decay 0.1–10000ms, Sustain 0–1, Release 0.1–10000ms. Each stage has shape control (-1.0 to +1.0: negative=logarithmic, 0=linear, positive=exponential) using RC-circuit-like curves. Optional S-curve mode per stage (attack, decay, release) splits the exponential curve into two symmetric halves for ease-in-out behavior — the shape slider then controls S-curve intensity. Uses `exp_m1` for numerical stability, C1-continuous at the midpoint.

- **Hold Stage**: Holds at peak (1.0) after attack completes, before decay begins. Skipped if < 0.1ms. Useful for pads and sustained transients.
- **Loop Mode**: OneShot (default DAHDSR) or LoopAHD — loops Attack→Hold→Decay while note is held, creating rhythmic or evolving shapes. Loop starts from sustain level when decay completes.
- **Depth** (volume env only): Controls how much the envelope affects output. At 1.0 = normal envelope behavior. At 0.0 = constant output (envelope bypassed). Formula: `output = depth × raw_env + (1.0 - depth)`.
- **Tempo Sync**: When enabled, attack, hold, decay, and release times are derived from musical divisions (1/1 through 1/128, dotted, triplets, 2/1, 4/1) synced to host/MIDI clock BPM. Formula: `time_ms = division_beats / BPM × 60000`. Each stage has an independent division selector.
- **Key Tracking**: Scales envelope times by `2^(-key_track × (note-60)/12)`. Higher notes get shorter envelopes, lower notes get longer. Applied to attack, hold, decay, and release.
- **Velocity Routing**: Three independent velocity→envelope routes:
  - **Vel→Attack**: Velocity modulates attack time via `2^(-amount × (vel-0.5) × 4)`. Positive = high velocity → shorter attack.
  - **Vel→Decay**: Same formula for decay time.
  - **Vel→Sustain**: Linear offset `amount × (vel - 0.5)` added to sustain level.

Retrigger from any stage starts attack from current value (C0-continuous). Retrigger Dip (0–1) creates percussive re-articulation by dipping amplitude before re-attack. Phase Reset toggle controls whether oscillator phases reset on retrigger. Sustain changes smoothly via 3ms one-pole filter.

### Anti-Click Protection

- **Release floor:** User-facing release can go as low as 0.1ms, but the internal minimum is clamped to 3ms to prevent discontinuities when cutting complex waveforms. This preserves snappy release feel while eliminating clicks on low-frequency content.
- **Quick fade on force-off:** When the envelope is forcibly stopped (e.g., preset change), a 2ms linear fade-out replaces the previous hard cut to zero. If a new note triggers during the fade, the attack starts from the current fading value for seamless continuity.
- **Attack minimum:** 0.1ms (~4–5 samples at 44.1kHz) allows near-instant transients for percussive sounds. Clicks on the attack side are perceptually acceptable and often desirable.

PLL Tail: When enabled, PLL resonance rings out after note release with configurable decay time (50–5000ms) and amount (0–1).

## LFO System

3 independent LFOs, each with 2 mod destination slots.

**Waveforms:** Sine, Triangle, Saw, Square, Sample&Hold. Free-run (0.01–50Hz) or tempo-synced (1/1 to 1/32 including dotted/triplet). Output slew (user-configurable, default 5ms) applies only to S&H waveform to smooth random step transitions; continuous waveforms pass through unslewed for full waveform fidelity.

**Cross-modulation:** Each LFO can use another as phase modulation source.

**Destinations:** PLL (Damp, Infl, Track, FM, XFB, Burst, Range, Vol, Mult discrete, Mult continuous), VPS (D, V, VΔ, DΔ, Fold, Shape, Vol), SAW (Fold, Shape, Vol), Sub (Vol), Coloration (Drift, Tube).

## Modulation Step Sequencer

Variable-length (1–16 steps) bipolar or unipolar step sequencer with tempo sync (same divisions as LFOs) and 4 modulation routing slots.

**Step values:** -1.0 to +1.0 per step. In bipolar mode (default), output spans the full range. In unipolar mode, output is 0.0 to +1.0.

**Variable length:** 1–16 steps via mseq_length. The sequence loops at the configured length.

**Ties:** 303-style ties use smoothstep (S-curve) interpolation between adjacent step values for smooth glides. Non-tied steps use the slew parameter (0–200ms) for transition smoothing.

**Per-step probability:** Each step has an independent probability (0–100%). When a step fires, its probability is evaluated — on failure, the previous value is held.

**Retrigger:** When enabled, the sequence resets to step 1 on each note-on event. When disabled (default), the sequence free-runs with the tempo.

**Routing:** 4 destination slots (dest1–dest4), each with bipolar amount (-1.0 to +1.0).

**Utility tools:** Random (randomize step values), Clear (zero all steps), Invert (negate all values), Mirror (reverse step order).

## Pitched Looper

Bar-synced pitched looper that captures internal signal and replays it as a pitch-shifted, rhythmic texture layer. Auto-record is always active — recording triggers automatically at bar boundaries based on interval settings.

**Auto-Record**: Automatically records N beats every M bars. Record length and playback length use the same 16-division system as LFOs (straight, dotted, triplet: 1/1 to 4/1). Interval: every 1, 2, 4, 8 bars. Example: record 1/2 dotted every 4 bars. Recorded buffers have a smooth crossfade applied at start and end to prevent clicks.

**Input Source**: Per-source toggle buttons (OUT/IN) control what signal feeds the looper's recording buffer. Sources are exclusive by priority — higher-priority sources override lower ones:
- **PRE** (highest priority): Pre-master signal (post-HPF/BoxCut/Brilliance, before looper). Full processed master bus.
- **FLTR**: Post-ladder-filter signal (with vol envelope + master vol). Overrides individual osc selections.
- **VPS**: VPS oscillator output (scaled by vol envelope + master vol)
- **PLL**: PLL oscillator output (scaled by vol envelope + master vol)
- **SAW**: SAW oscillator output (scaled by vol envelope + master vol)
Default: VPS=OUT, PLL=OUT, SAW=OUT, FLTR=IN, PRE=OUT. Individual oscs can be combined, but are overridden when FLTR or PRE is active.

**Parameters:**
- **Pitch** (-24 to +24 st): Semitone transposition via 16-point Kaiser-windowed sinc interpolation (β=7.857)
- **Doppler** (-24 to +24 st): Pitch overshoot on each playback trigger. Positive = starts high, swoops down. Negative = starts low, swoops up. Exponential decay (~300ms) back to target pitch.
- **Decay** (0–100%): Amplitude multiplier per repeat (1.0 = infinite loop)
- **Start** (0–100%): Playback offset within buffer
- **Volume** (0–300%): Loop level added to signal (>100% for boost)
- **Length** (1/32 to 4/1, dotted, triplet): How many beats the playback lasts per bar trigger
- **Direction**: FWD / REV / PING (ping-pong)
- **Stutter** (OFF/2/4/8/16): Chops buffer into N slices, retriggers each sequentially
- **Key Track**: Transposes loop relative to recorded note
- **Freeze**: Locks decay at 100%, loop persists indefinitely

**Signal position:** Input captured from individual oscillator sources (VPS, PLL, SAW), post-filter (FLTR), or pre-master (PRE) signal. Sources are exclusive by priority: PRE > FLTR > individual oscs. Playback mixed before Stereo Control.

## Lush Reverb

Dual-stage algorithmic reverb: early reflections feeding into an 8-channel FDN late reverb tank. All f64 internal precision.

### Architecture

**Early Reflections**
- 12 stereo tapped delay lines modeling initial room reflections
- Per-tap one-pole HF rolloff simulating distance-based air absorption (progressively darker at longer delays)
- Cross-channel feed (15%) between L/R taps for spatial coherence
- Per-channel one-pole LPF on summed ER output
- Tap delays scale with time_scale parameter (0.3 + 0.7 × time_scale) for consistent room-size feel
- ER output feeds into late reverb tank (35% blend) for natural ER→late transition

**Late Reverb (FDN Tank)**
- 8-channel Feedback Delay Network with Hadamard mixing matrix (energy-preserving, unitary)
- 4-stage allpass input diffusers per channel (L/R with different delay lengths for decorrelation)
- Diffusion mix control: crossfade between raw and diffused input to the tank
- Per-channel modulated delay reads: Hermite-interpolated fractional delay (min 1 sample guard) with hybrid LFO (sine + random walk, controllable shape)
- Modulation depth scaled by sample rate ratio (`depth × sr/44100`)
- RT60-compensated per-channel decay: `gain = 10^(-3 × delay_time / RT60)`
- Frequency-dependent damping: independent LPF + HPF per FDN channel in feedback path
- Controllable saturation in feedback: crossfade between clean and cubic soft-clip (`x - x³/3`), off by default
- Smoothed FDN delay lengths: ~50ms one-pole slew prevents clicks during automation
- Alternating-sign denormal guard per FDN channel
- Input injection with alternating polarity for stereo decorrelation
- Normalized stereo output taps (sum-of-squares = 1.0)
- Mid/side stereo width control on wet output
- Soft clipper on final output for protection against hot signals
- Sample-rate-adaptive DC blockers (5Hz cutoff) on stereo output

**Signal Flow**
```
Send Input → Pre-delay (Hermite) → ┬→ Early Reflections (12 taps × per-tap HF rolloff × cross-feed)
                                     │                           │
                                     │                    ER output (×0.5 to mix)
                                     │                           │ (×0.35 feed to tank)
                                     └→ Input HPF/LPF → Diffusers ──→ + ER feed → FDN Tank
                                                                                      ↓
                          ER × 0.5 + Late → Width → DC Block → Soft Clip → Ducking → Mix
```

### Mix Behavior

Send/return topology: `output = dry + mix × wet`

- At mix=0: pure dry signal
- At mix=1: dry + full wet reverb
- The mix knob acts as a wet level control, dry signal always passes through unchanged

### Parameters
- **Mix** (0–1): Wet signal level added to dry
- **Pre-delay** (0–500ms): Delay before reverb onset, tempo-syncable. Hermite-interpolated.
- **Time Scale** (0–1): Room size — scales FDN delay lengths (0.1–1.0×) and ER tap times (0.3–1.0×)
- **Decay** (0–1): RT60 from 0.3s to 30s (quadratic scaling)
- **Diffusion** (0–1): Allpass diffuser coefficient (smear density)
- **Diffusion Mix** (0–1): Blend between raw and diffused tank input
- **Stereo Width** (0–1): Mid/side width of wet signal (0=mono, 1=full stereo)
- **Saturation** (0–1): Tank feedback warmth — crossfade between clean and cubic soft-clip
- **Input HPF / Input LPF**: Bandpass filtering on reverb input
- **Tank HPF / Tank LPF**: Frequency-dependent decay — damping filters in FDN feedback
- **Mod Speed** (0–1): Chorus-like modulation rate in the tank (quadratic, 0.1–4.0 Hz)
- **Mod Depth** (0–1): Modulation excursion (quadratic, up to 12 samples at 44.1kHz)
- **Mod Shape** (0–1): Blend between sine LFO (0) and random walk (1)

### Ducking
- **Ducking Amount** (0–1): RMS-like power-tracking envelope follower (1ms attack)
- **Duck Release**: Release time, tempo-syncable via division selector
- **Rhythm Duck Depth** (0–1): Tempo-synced volume pumping on wet signal
- **Rhythm Duck Division**: Beat subdivision for rhythmic ducking
- **Rhythm Duck Smooth** (ms): Raised-cosine smoothing on rhythmic ducking envelope

### Design Notes
- Denormal protection: alternating-sign ±1e-18 per FDN channel
- Sample-rate independent: all delay lengths, filter coefficients, modulation depth, smoothing, DC blocker, and envelope times scale with sample rate
- Parameter smoothing: ~20ms one-pole on mix, ~50ms on FDN delay lengths
- Hermite interpolation on pre-delay and FDN delay reads with minimum 1-sample guard
- ER feeds into late tank for seamless early→late transition (no parallel-sum discontinuity)
- Per-tap ER HF rolloff models distance-based air absorption
- Output soft clipper prevents clipping at high decay / hot input without audible coloring
- 8 LFOs with mutually-prime base rates prevent periodic correlation
- Decay gains initialized to 0.85 — reverb produces output immediately after construction

## Compressor

Feed-forward VCA compressor with true-peak detection, program-dependent release, adjustable stereo linking, and optional lookahead with host PDC reporting. Inserted after reverb, before stereo control. All f64 internal precision.

### Architecture
- Feed-forward topology: predictable, clean, CPU-efficient
- 2nd-order Butterworth sidechain HPF using Transposed Direct Form II biquad (numerically stable at high sample rates)
- Adjustable soft-knee gain computation (0–12dB knee width, 0 = hard knee) for smooth transition
- True-peak detection: 4x Catmull-Rom inter-sample peak detection on sidechain signal catches peaks between samples that standard peak detection misses
- Hybrid true-peak/RMS level detection (70% true-peak, 30% RMS, 10ms RMS window) for musical response
- Per-sample parameter smoothing (~5ms time constant) on threshold, ratio, makeup, mix, and knee prevents zipper noise during automation
- dB-domain gain smoothing: symmetric attack/release behavior on logarithmic scale
- Program-dependent release: dual time constants (50ms attack / 500ms release envelope) with properly normalized blend across 0–20dB GR range — adapts smoothly to transient density
- Adjustable stereo linking (0–100%): 0% = fully independent per-channel compression, 100% = max(L,R) linked — per-channel gain smoothing ensures correct stereo behavior at all link values
- Auto makeup gain: `-threshold × (1 - 1/ratio) × 0.5 dB` compensates for ~50% of gain reduction at threshold
- Optional lookahead (stereo delay line, max 256 samples) with host latency reporting via `context.set_latency_samples()` for proper Plugin Delay Compensation — delay line reset on mode change prevents stale-sample artifacts
- Gain reduction metering: peak GR per block exposed for UI display
- Optimized math: `exp`/`ln` instead of `powf`/`log10` in the per-sample loop
- Zero latency when compressor is disabled or lookahead is off

### Parameters
- **Enable**: On/Off bypass
- **Threshold** (-40 to 0 dB): Where compression starts
- **Ratio** (1:1 to 20:1): Compression amount
- **Attack** (0.1 – 100 ms): Transient shaping
- **Release** (5 – 2000 ms): Recovery speed
- **Knee** (0 – 12 dB): Soft-knee width (0 = hard knee)
- **Makeup** (0 – 24 dB): Output gain compensation
- **Auto Makeup**: Automatic gain compensation based on threshold/ratio
- **Mix** (0 – 100%): Dry/wet for parallel compression
- **Stereo Link** (0 – 100%): 0% = dual mono, 100% = stereo linked
- **SC HPF** (Off / 80 / 150 / 250 Hz): Sidechain highpass filter
- **Lookahead** (Off / 1ms / 2.5ms / 5ms): Audio delay for transient-transparent gain reduction (reports latency to host for PDC)

### Signal Routing (IN/OUT)
Each signal component can be routed through or around the compressor:
- **MSTR** (default IN): Main synth signal (post-brilliance)
- **LOOP** (default OUT): Looper playback contribution
- **VERB** (default OUT): Reverb wet signal

When a source is IN, it passes through the compressor. When OUT, it bypasses and is summed back after compression. When all sources are IN, the compressor operates as a simple insert on the full mix (fast path, no decomposition overhead).

## Signal Flow

```
Sequencer → note/gate
     ↓
VPS ──┬── [VPS IN] ──→ LOOPER (record)    VPS ──[VPS IN]──→ REVERB (send)
PLL ──┤── [PLL IN] ──→        ↑            PLL ──[PLL IN]──→    ↑
SAW ──┤── [SAW IN] ──→        ↑            SAW ──[SAW IN]──→    ↑
      ↓                       ↑            SUB ──[SUB IN]──→    ↑
 Mix → Ladder Filter ─[FLTR IN]           FLTR ─[FLTR IN]──→    ↑
      ↓                                   LOOP ─[LOOP IN]──→    ↑
× Envelope → Coloration → Sub added              (looper-only)
      ↓
HPF → Box Cut → Brilliance ─[PRE IN]──→ LOOPER (record)
      ↓
LOOPER (playback mix) → REVERB → COMPRESSOR → Stereo Control
      │                                            │
      ├── Sub [IN mode] → HPF → Box Cut ──────────┘
      ↓
Global Volume → Limiter → Stereo Out

LOOPER input priority: PRE > FLTR > individual oscs (exclusive)
REVERB send: FLTR exclusive with VPS/PLL/SAW/SUB; LOOP sends looper contribution only
COMPRESSOR route: MSTR/LOOP/VERB segments (independent, additive)
```
