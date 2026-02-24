# PLL Improvements Roadmap

Isolated features sorted by priority. Each group can be implemented and tested independently.

## Status Legend

- `[ ]` Pending
- `[~]` In progress
- `[x]` Complete
- `[?]` Needs testing/evaluation

---

## Group A — Range Expansions (Trivial changes, big impact)

### A1. Sub-Octave Multipliers
`[ ]` **Priority: High | Effort: Low**

Add 0.5 and 0.25 to the multiplier list → `[0.25, 0.5, 1, 2, 4, 8, 16, 32, 64]`.
Classic analog PLL technique (frequency dividers). Produces thick sub-bass and octave-down tones.

**Files:** `params.rs` (mult enum range), `voice.rs` (mult step table), `ui/pages/synth.rs` (slider labels)

### A2. Extend FM Amount Range
`[x]` **Priority: High | Effort: Trivial**

Current: `0.0–0.2`. Proposed: `0.0–1.0` (or `0.0–2.0`).
At 0.2 it's limited to subtle detuning. Higher indices unlock metallic, bell-like, and harsh FM timbres.

**Files:** `params.rs` (`synth_pll_fm_amount` range)

### A3. Extend FM Ratio to Non-Integer
`[x]` **Priority: Medium | Effort: Low-Medium**

Current: integer `1–8`. Proposed: float `0.5–16.0` with fine resolution.
Non-integer ratios (1.5, 2.5, golden ratio 1.618, π) produce inharmonic/bell spectra. Keep integer snapping as default behavior with a "free ratio" toggle.

**Files:** `params.rs`, `voice.rs` (fm_ratio handling), `oscillator.rs`

### A4. Extend Loop Saturation Upper Bound
`[ ]` **Priority: Low | Effort: Trivial**

Current: `1.0–500.0`. Proposed: `1.0–1000.0`.
High values = nearly free integrator = wider frequency swings. Extreme experimental territory.

**Files:** `params.rs`, `oscillator.rs` (clamp in `set_experimental_params`)

### A5. Extend Edge Sensitivity
`[ ]` **Priority: Low | Effort: Trivial**

Current: `0.001–0.2`. Proposed: `0.001–0.5`.
Higher sensitivity = PFD triggers on smaller phase changes = more "nervous" tracking.

**Files:** `params.rs`, `oscillator.rs` (clamp in `set_experimental_params`)

---

## Group B — VCO Output Shaping (Low effort, huge tonal variety)

### B1. VCO Waveform Selection
`[x]` **Priority: High | Effort: Low**

Currently: sine or cubic-saturated sine. Add selectable output waveshapes:
- **Sine** (current default)
- **Saw** — `2.0 * phase - 1.0`
- **Square** — `if phase < 0.5 { 1.0 } else { -1.0 }`
- **Triangle** — `4.0 * (phase - (phase + 0.5).floor()).abs() - 1.0`

Phase is already available — this is just a different phase→amplitude mapping. Each shape gives different harmonic character from the same PLL tracking behavior.

**Files:** `oscillator.rs` (new enum + output section), `params.rs` (new param), `voice.rs` (pass through), `ui/pages/synth.rs` (selector)

### B2. Wavefolder on VCO Output
`[ ]` **Priority: Medium | Effort: Low-Medium**

Replace/augment the current cubic saturation (`colored`) with sinusoidal wavefolding. When amplitude exceeds threshold, fold it back. Much richer harmonic spectrum than cubic saturation.

```
folded = sin(signal * fold_amount * PI)
```

Interacts beautifully with PLL's natural amplitude variations. Could be a mode toggle alongside the existing COLOR.

**Files:** `oscillator.rs` (output section), `params.rs` (fold amount param)

### B3. Ring Modulation Mode
`[ ]` **Priority: Low-Medium | Effort: Low**

Multiply PLL VCO output with the reference signal: `output = vco * ref_pulse`.
Sum/difference tones between tracked PLL and its own reference create unique metallic character.

**Files:** `oscillator.rs` or `voice.rs` (post-PLL processing), `params.rs` (ring mod amount)

---

## Group C — Phase Detector Enhancements

### C1. Continuous PD Mode Morph
`[ ]` **Priority: Medium | Effort: Low**

Instead of switching between AnalogLikePD and EdgePFD, blend them:
```
error = analog_error * (1.0 - morph) + edge_error * morph
```
At 0% = smooth analog-like. At 100% = aggressive edge detection. In between = hybrid.

**Files:** `oscillator.rs` (compute both, blend), `params.rs` (morph param replaces bool), `voice.rs`, `ui/pages/synth.rs`

### C2. Linear Phase Detector Option
`[x]` **Priority: Low | Effort: Trivial**

Current AnalogLikePD uses `tanh` compression which saturates at large phase errors, limiting acquisition range. Add option for linear (uncompressed) detector for wider capture range at the cost of potential harshness.

**Files:** `oscillator.rs` (conditional tanh)

---

## Group D — Hard Sync & Phase Reset Modes

### D1. Hard Sync Toggle
`[ ]` **Priority: High | Effort: Low**

Reset VCO phase to 0 on each reference oscillator cycle. Classic hard-sync timbres when combined with the multiplier. Reference phase wrap already detected.

```
if ref_phase < prev_ref_phase {
    vco_phase = 0.0;
}
```

Distinct from PLL tracking — hard sync is deterministic and produces sharp harmonic sweeps when multiplier is detuned.

**Files:** `oscillator.rs` (phase reset in `next()`), `params.rs` (bool toggle), `voice.rs`, `ui/pages/synth.rs`

### D2. Soft Sync / Phase Reset Amount
`[ ]` **Priority: Medium | Effort: Low**

Instead of full reset, blend current phase toward 0: `phase = phase * (1.0 - sync_amount)`. Continuous control from free-running (0%) to full hard sync (100%).

**Files:** `oscillator.rs`, `params.rs`

---

## Group E — Feedback & Chaos

### E1. Feedback Path Frequency Divider
`[x]` **Priority: Medium | Effort: Medium**

Classic analog PLL: divide VCO frequency by N before feeding back to phase detector. Divider values: 1, 2, 3, 4, 5, 6, 7, 8.

Different from the existing multiplier — the phase error detection itself operates at the divided rate, producing different tracking/locking behavior.

**Files:** `oscillator.rs` (divide phase in detector input), `params.rs`, `voice.rs`, `ui/pages/synth.rs`

### E2. Chaos / Instability Control
`[x]` **Priority: Medium | Effort: Medium**

Dedicated chaos parameter combining:
- Noise injection into loop filter integrator
- Slow random gain modulation on loop coefficients
- Small delay (1–5 samples) in the feedback path

Each element creates different instability flavors. Even 1–2 samples of delay in phase detection creates complex nonlinear dynamics.

**Files:** `oscillator.rs` (new state variables, noise gen), `params.rs`

### E3. DC Offset / Asymmetry in Phase Detector
`[ ]` **Priority: Low | Effort: Trivial**

Add configurable DC bias to phase detector output. Causes VCO to systematically track above/below reference — creates beating/detuned quality even when "locked."

```
phase_error = phase_error + dc_offset;
```

**Files:** `oscillator.rs`, `params.rs`

---

## Group F — Loop Filter Enhancements

### F1. Second-Order Loop Filter
`[x]` **Priority: Medium | Effort: Medium**

Current: 1-pole lowpass + PI. Add optional 2nd-order filter: steeper rolloff, tighter lock, less jitter, slower acquisition. Toggle between 1st and 2nd order.

**Files:** `oscillator.rs` (additional filter state + coefficients in `prepare_block`)

### F2. Resonant Loop Filter
`[ ]` **Priority: Low-Medium | Effort: Medium**

Add resonance peak to the error signal filter. Creates ringing/oscillation at the loop bandwidth frequency — unusual and could produce unique tonal effects. Resonance amount as a new parameter.

**Files:** `oscillator.rs` (SVF or biquad in loop path), `params.rs`

---

## Group G — Reference Oscillator Enhancements

### G1. Reference Waveform Selection
`[x]` **Priority: Medium | Effort: Medium**

Currently fixed to PolyBLEP pulse. Phase detector responds differently to different reference shapes:
- **Pulse** (current)
- **Saw** — asymmetric phase detection, different lock character
- **Sine** — smoothest tracking, most "analog VCO" character
- **Noise-modulated pulse** — jittery, lo-fi character

**Files:** `voice.rs` (reference oscillator section), `params.rs`, `ui/pages/synth.rs`

### G2. VPS→PLL Reference Routing
`[ ]` **Priority: Low-Medium | Effort: Medium**

Let VPS oscillator output feed the PLL reference input. Complex interaction between the two oscillator systems. Could be a mix knob: 0% = normal reference, 100% = VPS feeds PLL.

**Files:** `voice.rs` (routing), `params.rs`

---

## Group H — Topology Modes (Advanced)

### H1. Injection-Locked Mode
`[x]` **Priority: Medium | Effort: Medium**

Instead of full feedback loop, inject reference signal directly into VCO (additive). VCO "pulls" toward reference frequency. Different character — smoother, less aggressive than PLL tracking.

```
vco_input = corrected_freq + injection_amount * ref_signal * base_freq
```

**Files:** `oscillator.rs` (new mode in processing), `params.rs`

### H2. Dual/Coupled PLL
`[ ]` **Priority: Low | Effort: High**

Two PLLs where each output feeds the other's reference. Complex coupled oscillator behavior — can produce synchronized chaos, phase patterns, and emergent harmonics.

**Files:** `voice.rs` (second PLL instance + cross-routing), potentially `oscillator.rs`

### H3. Cascade PLL
`[ ]` **Priority: Low | Effort: High**

First PLL output feeds second PLL as reference. Each stage adds tracking lag and harmonic complexity. Computationally expensive (2× PLL cost).

**Files:** `voice.rs` (second PLL instance + serial routing)

---

## Group I — Stereo & Modulation Enhancements

### I1. Envelope→Loop Parameters
`[ ]` **Priority: Medium | Effort: Low-Medium**

Extend envelope modulation beyond FM to:
- **Track Speed** — loose attack → tight sustain (PLL "catches" the note)
- **Damping** — ringing attack → stable sustain
- **Multiplier** (continuous) — harmonic sweep over note duration

**Files:** `params.rs` (env amount params), `voice.rs` (envelope application), `ui/pages/synth.rs`

### I2. Noise Injection Point Selector
`[ ]` **Priority: Low-Medium | Effort: Low-Medium**

Selectable noise injection: reference, loop filter, or VCO phase. Each creates different instability flavor:
- Reference noise: jittery pitch
- Loop filter noise: wandering tracking
- VCO phase noise: gritty output

**Files:** `oscillator.rs` or `voice.rs`, `params.rs`

---

## Group J — Quality & Precision

### J1. Anti-Aliasing for High Multipliers
`[x]` **Priority: High | Effort: Medium**

At 44.1kHz with 440Hz × 64 = 28.16kHz — right at Nyquist. Options:
- **2× oversampling** specifically in PLL processing loop
- **Bandlimit VCO output** with 1-pole filter tuned to Nyquist/2
- **PolyBLEP on VCO output** for non-sine waveforms (relevant if B1 implemented)

**Files:** `oscillator.rs` (oversampling wrapper or output filter)

### J2. Sub-Sample Edge Interpolation
`[x]` **Priority: Low-Medium | Effort: Medium**

EdgePFD measures timing at sample resolution. At high frequencies, quantization of zero-crossing detection creates jitter. Linear interpolation between samples for edge detection:
```
// Instead of sample_counter, interpolate exact crossing point
crossing_time = sample_n + (threshold - prev) / (cur - prev)
```

**Files:** `oscillator.rs` (`next_pfd` method)

### J3. Track Speed Curve Improvement
`[x]` **Priority: Low | Effort: Trivial**

Current mapping uses sigmoid: `exp(speed * 6 - 3) / (1 + exp(...))`. Verify this gives good resolution at the low end where "barely tracking" behaviors live. A `speed²` or `speed³` mapping might give better perceptual control.



**Files:** `oscillator.rs` (`prepare_block`)

### J4. Decouple Burst from Damping
`[x]` **Priority: Low | Effort: Trivial**

Currently `burst * (1.0 - damping)` — high damping kills burst entirely. Add option for independent burst that ignores damping, or a partial coupling control.

**Files:** `oscillator.rs` (burst calculation at line ~377)

---

## Implementation Order Suggestion

**Phase 1 — Quick wins, test immediately:**
A1, A2, B1, D1

**Phase 2 — Shape the sound further:**
B2, C1, A3, I1

**Phase 3 — Quality & depth:**
J1, E1, E2, G1

**Phase 4 — Experimental territory:**
F1, F2, H1, H2, H3

Each group is self-contained. Within a group, items are independent unless noted.

## Loop Coefficient Calculation — IMPLEMENTED

Coefficients now use proper PLL theory:
- `track_speed` is cubed before the sigmoid, giving ~4× more resolution at the low end
- Bandwidth mapped [3..150 Hz] from curved speed
- `ωn = 2π × bandwidth` derived from track_speed
- `ζ = 0.15 + damping × 1.35` maps damping to PLL damping ratio
- `Kp = 2ζωn × influence × modifiers` (proportional)
- `Ki = ωn² × influence × modifiers` (integral)
- Kp/Ki ratio now varies with ζ and ωn (was fixed 1:100)
- AnalogLikePD uses linear clamp instead of tanh for wider capture range
- EdgePFD uses sub-sample interpolation for cleaner high-frequency tracking

## Selective PLL Oversampling — IMPLEMENTED

Restructured `voice.rs::process()` so only PLL (+ reference/FM oscillators) runs at the oversampled rate. VPS, sub, filter, reverb, and coloration all run at DAW rate. This significantly reduces CPU usage since filter/reverb/VPS don't benefit from oversampling.

- PLL loop runs `effective_oversample_ratio` iterations, writes raw PLL output to oversampling buffers, then downsamples to a single sample
- VPS sync detection (`ref_phase_wrapped`) is tracked during PLL loop and applied once before VPS processing
- All mixing (ring mod, wavefold, tube) happens at DAW rate on the downsampled PLL + DAW-rate VPS
- Filter uses `process_sample()` directly instead of `process_buffers()` for single-sample processing
- Reverb and sub oscillator process single samples at DAW rate

## CPU Optimization Pass — IMPLEMENTED

Systematic optimization of PLLOscillator hot path (~40-50% CPU reduction):

### sin() elimination
- VCO output uses `fast_sin_unit()` — refined parabolic approximation (<0.06% error, inaudible)
- PFD edge detection uses triangle wave instead of sin — same zero crossings, exact subsample interpolation
- Colored output path uses `fast_sin_unit()`, cubic saturation uses `x*x*x` instead of `powi(3)`
- Eliminates 1-4 `sin()` calls per sample per PLL oscillator (×2 for stereo)

### Rate-limited coefficient computation
- `prepare_block()` split: cheap smoothing (mult, color_x) runs every call, expensive coefficients (exp, sqrt, omega_n) run every 32 calls
- AA filter coefficients moved to `set_sample_rate()` since they only depend on sample rate

### Sample-rate compensated parameters
- Integrator decay: preserves original time constants (~227ms at damping=0, ~2.3ms at damping=1) regardless of sample rate
- PFD smoothing alpha: preserves ~0.13ms time constant regardless of sample rate
- Both previously had fixed per-sample coefficients that behaved differently at 96kHz/192kHz vs 44.1kHz

### Cached computed values
- `cached_israte` (1/sample_rate), `cached_nyquist` (0.48*sr) avoid per-sample recomputation
- `cached_integrator_decay`, `cached_pfd_alpha` precomputed in coefficient update

### Other optimizations
- `wrap_pi()` simplified from floor-division to conditional branches (input always in (-TAU, TAU))
- Overtrack burst path merged — single phase advancement instead of duplicated code
- AnalogLikePD phase diff uses `FRAC_1_PI` multiply instead of PI divide
- PFD sample counter resets at 2^30 to prevent f64 precision loss in long sessions