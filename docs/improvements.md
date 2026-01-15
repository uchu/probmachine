# Device - Bottlenecks & Improvement Ideas

Analysis of current limitations and potential enhancements for experimental electronic music.

## Current Bottlenecks

### CPU-Intensive Areas

| Component | Impact | Notes |
|-----------|--------|-------|
| Dattorro Reverb | High | 4 allpass chains, modulation at sample rate |
| PLL per-sample math | Medium | Complex PI loop calculations |
| 16x Oversampling | High | 16× more DSP operations |
| Parameter slewing | Low-Medium | Many SlewValue instances per sample |
| Formant 3-band BPF | Medium | Per-sample biquad processing |

### Memory Access Patterns

- **Reverb delay lines**: Large memory footprint, potential cache misses
- **SlewValue state**: Many small objects, scattered memory access
- **Double-buffered sequencer**: Minimal impact but additional allocation

### Thread Contention

- **Mutex on NotePool**: Lock contention when UI updates notes
- **Mutex on strength_values**: Same issue for strength grid
- **Atomic preset_version**: Minimal overhead

## Optimization Opportunities

### High Impact

#### 1. SIMD Optimization for Stereo Processing ✅ COMPLETE
Portable SIMD (`src/synth/simd.rs`) fully integrated using `f64x2` for parallel L/R processing.

Cross-platform: Compiles to SSE/AVX on x86_64, NEON on ARM64 (Raspberry Pi 5)

**Integrated components:**
- `StereoMoogFilter`: Stilson Moog 4-pole ladder filter (f64 precision)
- `stereo_wavefold`: Sinusoidal wavefolding effect
- `stereo_tube_saturate`: Asymmetric tube saturation with soft clipping
- `stereo_distort_bram`: Bram de Jong waveshaper (matching synfx-dsp formula)

**Benefit**: Stereo filter and coloration effects now use parallel SIMD operations

#### 2. Block-Based LFO Processing
LFOs computed per-sample but change slowly. Could:
- Process LFOs once per block (64-256 samples)
- Interpolate values within block
- Maintain smooth modulation with far less computation

**Estimated gain**: 20-40% on LFO system

#### 3. Reverb Optimization
- Pre-compute modulation tables
- Consider simpler reverb algorithms for lower quality settings
- Optionally run reverb at lower internal rate

**Estimated gain**: 10-25% when reverb enabled

### Medium Impact

#### 4. Lock-Free State Sharing
Replace Mutex with lock-free data structures for:
- Note pool updates
- Strength grid values

**Benefit**: Eliminates rare but possible priority inversion

#### 5. Lazy Parameter Updates
Many parameters only need updates on change:
- Cache filter coefficients, recalculate only when cutoff/res change
- Pre-compute envelope curves on parameter change

**Estimated gain**: 5-10% overall

## Sound Design Enhancements

### For Experimental Electronic Music

#### 1. Additional Oscillator Types

**Additive Oscillator**
- User-defined harmonic spectrum
- Per-harmonic amplitude envelopes
- Spectral morphing
- Great for drones, evolving textures

**Noise Generator Improvements**
- Pink noise, brown noise options
- Filtered noise with resonance
- Crackle/dust noise for lo-fi
- Sample-and-hold noise for stepped modulation

**Wavetable Oscillator**
- User-loadable wavetables
- Wavetable position as mod destination
- Interpolation options (linear, cubic)
- FM between wavetable frames

#### 2. PLL Enhancements

**Chaos Mode**
- Add Lorenz attractor modulation
- Logistic map for controlled chaos
- Feedback delay for Karplus-Strong-like timbres

**External Audio Input**
- PLL can track external audio pitch
- Creates vocoder-like effects
- Phase-lock to external beats

**Multiple PLL Stages**
- Cascade PLLs for complex tracking
- Cross-modulate between PLL instances

#### 3. Filter Enhancements

**Additional Filter Types**
- Ladder filter (classic Moog character)
- Comb filter (resonant harmonics)
- Formant banks (multiple vowels simultaneously)
- Phaser-style allpass chains
- Waveguide resonators

**Filter FM**
- Audio-rate cutoff modulation
- Creates metallic, bell-like timbres
- PLL output → filter cutoff

**Serial/Parallel Filter Routing**
- Two filters in series or parallel
- Different types combined (LP + HP = BP with adjustable bandwidth)

#### 4. Modulation Enhancements

**Envelope Followers**
- Track amplitude of oscillator outputs
- Create self-modulating patches
- Ducking and dynamic textures

**Random Modulation**
- Perlin noise generator
- Drunk walk (smooth random)
- Probability gates for triggered random

**Macro Controls**
- Single knob controlling multiple destinations
- Preset interpolation via macro
- Performance-friendly interface

**Step Sequencer for Modulation**
- Pattern-based parameter automation
- Tied to main sequencer or independent
- Per-step glide and probability

#### 5. Effects Enhancements

**Delay Effects**
- Tempo-synced stereo delay
- Ping-pong with feedback filtering
- Granular delay with pitch shifting
- Tape delay with wow/flutter

**Distortion Variety**
- Bit crusher / sample rate reduction
- Digital clipping modes
- Asymmetric soft clipping curves
- Multiband distortion

**Modulation Effects**
- Chorus/flanger/phaser
- Ring modulator with internal carrier
- Frequency shifter
- Pitch shifter with formant preservation

**Spectral Effects**
- FFT-based freeze
- Spectral smear/blur
- Pitch tracking for harmonizer

### Sequencer Enhancements

#### 1. Tempo & Time Signature

- Host tempo sync
- Variable time signatures (5/4, 7/8, etc.)
- Polymetric patterns (different lengths per division)

#### 2. Pattern Chaining

- Multiple bars with different probability settings
- Pattern triggers from MIDI
- Markov chain between patterns

#### 3. Euclidean Rhythms

- Generate probability patterns from Euclidean algorithms
- Rotation control
- Hybrid probability/euclidean mode

#### 4. Probability Evolution

- Gradual probability drift over time
- Call-and-response patterns
- Learning from user input

## Code Quality Improvements

### 1. Unused Code Warnings
Current warnings to address:
- `presets.rs:182` - unused assignment to `current_location`
- `data.rs:430` - `with_data` function never used

### 2. Parameter Organization
`params.rs` is ~1200 lines. Consider:
- Split into parameter groups (beat_params.rs, synth_params.rs, etc.)
- Use macros for repetitive parameter definitions
- Parameter struct composition instead of flat struct

### 3. Error Handling
- Add proper error types instead of panic! in some places
- Result types for file operations
- Graceful degradation on failure

### 4. Testing
- Unit tests for DSP algorithms
- Integration tests for signal chain
- Property-based testing for sequencer probability math

## UI Improvements

### 1. Visual Feedback
- Oscilloscope display showing waveform
- Spectrum analyzer
- Level meters with peak hold
- LFO phase visualization

### 2. Interaction
- MIDI learn for parameters
- Randomize function (partial or full)
- Preset morphing between A/B slots
- Undo/redo for parameter changes

### 3. Touch Screen Optimization
- Larger touch targets
- XY pad controls
- Gesture support (pinch zoom, swipe)

## Platform-Specific

### Raspberry Pi Optimization
- ✅ NEON SIMD via portable SIMD (filter, wavefold, tube sat, distortion)
- Test and profile on ARM hardware
- Memory pool for allocation-free audio path
- Lower default oversampling for Pi

### Real-Time Safety
- Audit for any blocking operations in process()
- Remove any remaining allocations in audio path
- Profile for worst-case latency

## Priority Matrix

| Enhancement | Effort | Impact | Priority | Status |
|-------------|--------|--------|----------|--------|
| SIMD stereo | Medium | High | High | ✅ Complete |
| Block LFO | Low | Medium | High | Pending |
| Lock-free state | Medium | Low | Medium | Pending |
| Additional filters | Medium | High | High | Pending |
| Delay effect | Medium | High | High | Pending |
| Wavetable OSC | High | High | Medium | Pending |
| Pattern chaining | Medium | Medium | Medium | Pending |
| Host tempo sync | Low | High | High | Pending |
| Visual feedback | Medium | Medium | Medium | Pending |
| Spectral effects | High | Medium | Low | Pending |

## Quick Wins

1. **Fix compiler warnings** - Clean up unused code
2. **Implement filter drive** - Parameter exists but is not applied in filter.rs
3. **Host tempo sync** - Read from transport context
4. **MIDI CC mapping** - Infrastructure exists, just needs wiring
5. **Block-based LFO** - Straightforward refactor
6. **Reverb quality setting** - Lower internal rate option
