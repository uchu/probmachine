# Device - Audio Synthesizer

A professional-grade monophonic synthesizer built in Rust featuring PLL synthesis, advanced DSP, and a polyrhythmic sequencer.

## Quick Start

### Prerequisites

**macOS:**
```bash
brew install jack
```

**Linux/Raspberry Pi:**
```bash
sudo apt install jackd2
```

### Run

```bash
# Development (auto-starts JACK)
./scripts/dev.sh

# Or with custom buffer size
BUFFER_SIZE=128 ./scripts/dev.sh
```

### Manual Setup

```bash
# 1. Start JACK server
jackd -d coreaudio -p 256 &  # Mac
jackd -d alsa -d hw:0 -p 256 -n 2 &  # Linux

# 2. Build and run
cargo build --release

# Mac (set library path)
DYLD_LIBRARY_PATH=/opt/homebrew/lib ./target/release/device -b jack

# Linux
./target/release/device -b jack
```

## Buffer Size Configuration

Control latency by changing JACK's buffer size:

| Buffer | Latency @48kHz | Use Case |
|--------|----------------|----------|
| 64     | ~1.3ms         | Ultra low-latency |
| 128    | ~2.7ms         | Low-latency |
| 256    | ~5.3ms         | Balanced (default) |
| 512    | ~10.7ms        | Complex processing |

```bash
# Restart JACK with different buffer
pkill jackd
jackd -d coreaudio -p 128 &
```

## CLI Options

```
-b, --backend       Audio backend: jack, core-audio, alsa, wasapi, auto
-p, --period-size   Buffer size (ignored for JACK)
-r, --sample-rate   Sample rate (ignored for JACK)
--tempo             Transport tempo (default: 120)
```

## Platforms

| Platform | Backend | Status |
|----------|---------|--------|
| macOS    | JACK    | Supported |
| Linux    | JACK    | Supported |
| Raspberry Pi | JACK | Supported (HiFiBerry DAC Pro) |
| Windows  | JACK    | Supported |

## Documentation

- [JACK Setup Guide](docs/jack-setup.md) - Detailed platform setup
- [Architecture](docs/architecture.md) - Technical documentation

## Building

```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release

# Run tests
cargo test

# Check for issues
cargo clippy
```

## License

See LICENSE file.
