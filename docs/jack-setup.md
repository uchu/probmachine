# JACK Audio Setup Guide

Device uses JACK as the primary audio backend for cross-platform low-latency audio. This guide covers setup for all supported platforms.

## Why JACK?

- **Professional audio** - Standard for pro audio on Linux/Pi
- **Low latency** - Real-time audio with configurable buffer sizes
- **Cross-platform** - Works on Mac, Linux, Windows, Raspberry Pi
- **Flexible routing** - Connect any audio app to any other
- **Configurable buffers** - Set any buffer size you need

## Quick Start

**Environment Variables:**
| Variable | Default | Description |
|----------|---------|-------------|
| `SAMPLE_RATE` | 48000 | Sample rate in Hz (48000, 96000, 192000) |
| `BUFFER_SIZE` | 256 | Buffer size in samples (128, 256, 512, 1024) |
| `BACKEND` | jack | Audio backend (jack, coreaudio, alsa) |

**Using scripts:**
```bash
# Development (builds + runs)
./scripts/dev.sh

# Production (runs pre-built binary)
./scripts/run.sh

# With custom settings
SAMPLE_RATE=96000 BUFFER_SIZE=512 ./scripts/dev.sh
SAMPLE_RATE=192000 ./scripts/run.sh
```

**Manual:**
```bash
# Start JACK server first
jackd -d coreaudio -r 48000 -p 256 &

# Run with cargo (Mac - set library path)
SAMPLE_RATE=48000 DYLD_LIBRARY_PATH=/opt/homebrew/lib cargo run --release -- -b jack

# Or build and run separately
cargo build --release
SAMPLE_RATE=48000 DYLD_LIBRARY_PATH=/opt/homebrew/lib ./target/release/device -b jack
```

**Note:** The `SAMPLE_RATE` environment variable is required due to a bug in the JACK Rust crate that reports incorrect sample rates on macOS.

## Platform Setup

### macOS

**Install JACK:**
```bash
brew install jack
```

**Start JACK server:**
```bash
# With 256 sample buffer (low latency)
jackd -d coreaudio -p 256

# With 512 sample buffer (safer for complex processing)
jackd -d coreaudio -p 512

# With 1024 sample buffer (very stable)
jackd -d coreaudio -p 1024
```

**Run Device:**
```bash
# Set library path for Homebrew JACK
export DYLD_LIBRARY_PATH=/opt/homebrew/lib

# Run with JACK backend
./target/release/device -b jack
```

**Convenience script (macOS):**
```bash
#!/bin/bash
export DYLD_LIBRARY_PATH=/opt/homebrew/lib
./target/release/device -b jack "$@"
```

### Linux (Desktop)

**Install JACK:**
```bash
# Ubuntu/Debian
sudo apt install jackd2

# Fedora
sudo dnf install jack-audio-connection-kit

# Arch
sudo pacman -S jack2
```

**Start JACK server:**
```bash
# With ALSA backend
jackd -d alsa -d hw:0 -p 256 -n 2

# List available devices
cat /proc/asound/cards
```

**Run Device:**
```bash
./target/release/device -b jack
```

### Raspberry Pi

**Install JACK:**
```bash
sudo apt install jackd2
```

**Configure for low latency:**
```bash
# Add user to audio group
sudo usermod -a -G audio $USER

# Set realtime limits (add to /etc/security/limits.d/audio.conf)
@audio - rtprio 99
@audio - memlock unlimited
```

**For HiFiBerry DAC Pro:**
```bash
# Edit /boot/config.txt, add:
dtoverlay=hifiberry-dacpro

# Start JACK with HiFiBerry
jackd -d alsa -d hw:sndrpihifiberry -p 256 -n 2
```

**For USB audio interface:**
```bash
# Find device name
aplay -l

# Start JACK
jackd -d alsa -d hw:1 -p 256 -n 2
```

**Run Device:**
```bash
./target/release/device -b jack
```

### Windows

**Install JACK:**
1. Download JACK2 from https://jackaudio.org/downloads/
2. Run the installer
3. Add JACK bin directory to PATH

**Start JACK:**
```bash
# Using WASAPI (recommended)
jackd -d portaudio

# Or use JACK Control GUI
# Start Menu -> JACK -> JACK Control
```

**Run Device:**
```cmd
device.exe -b jack
```

## Buffer Size Configuration

JACK controls the buffer size, not the application. Change buffer size by restarting JACK:

| Buffer Size | Latency @ 48kHz | Use Case |
|-------------|-----------------|----------|
| 64 | ~1.3ms | Ultra low-latency (requires fast system) |
| 128 | ~2.7ms | Low-latency performance |
| 256 | ~5.3ms | Balanced (recommended) |
| 512 | ~10.7ms | Complex processing |
| 1024 | ~21.3ms | Very stable |

**Example:**
```bash
# Stop current JACK
pkill jackd

# Start with new buffer size
jackd -d coreaudio -p 128  # Mac
jackd -d alsa -d hw:0 -p 128 -n 2  # Linux
```

## CLI Options

Device supports various command-line options for audio configuration:

```bash
# Use JACK backend explicitly
device -b jack

# Use CoreAudio (Mac) - note: may have buffer size issues
device -b core-audio

# Use auto-detect (tries JACK first, then platform default)
device -b auto

# Set sample rate (ignored when using JACK)
device -b core-audio -r 48000

# Set buffer size (ignored when using JACK)
device -b core-audio -p 256
```

## Troubleshooting

### Mac: "libjack.0.dylib not found"
```bash
export DYLD_LIBRARY_PATH=/opt/homebrew/lib
```

### Linux: "Cannot lock memory"
```bash
# Add to /etc/security/limits.d/audio.conf
@audio - memlock unlimited
```

### Linux: "Cannot use realtime scheduling"
```bash
# Add user to audio group
sudo usermod -a -G audio $USER
# Log out and back in
```

### JACK: "Cannot open device"
- Check device is not in use by another application
- Verify correct device name with `aplay -l` (Linux) or Audio MIDI Setup (Mac)

### JACK: High CPU usage / Xruns
- Increase buffer size
- Stop other audio applications
- Use `cpufreq-set -g performance` on Linux

## Sample Rates

Common sample rates and their uses:

| Rate | Use Case |
|------|----------|
| 44100 Hz | CD quality, general use |
| 48000 Hz | Professional audio (recommended) |
| 88200 Hz | High quality, 2x oversampling |
| 96000 Hz | Professional high-resolution |
| 192000 Hz | Ultra high-resolution |

Note: Higher sample rates require more CPU. Device uses internal oversampling (configurable 1x-16x) regardless of host sample rate.

## Using CoreAudio Directly (UAD, Apollo, etc.)

For professional interfaces like UAD Apollo that have their own sample rate management, you may want to use CoreAudio directly instead of JACK:

```bash
# Run at your interface's native rate (e.g., 192kHz)
./target/release/device -b core-audio -r 192000 -p 256

# Or use the dev.sh script:
BACKEND=coreaudio SAMPLE_RATE=192000 ./scripts/dev.sh
```

**Important:** When using CoreAudio directly:
- The app will request the specified sample rate from your interface
- If the rate doesn't match, the driver may force a change
- Set `-r` to match your interface's current rate to avoid conflicts

**JACK vs CoreAudio:**

| Aspect | JACK | CoreAudio Direct |
|--------|------|------------------|
| Sample rate control | JACK server | App's `-r` flag |
| Dynamic rate change | Requires restart | May crash - restart app |
| Routing flexibility | Full | Limited |
| Latency | Configurable | OS-controlled |
| Best for | Development, routing | Simple playback, UAD |

## Connecting Audio

With JACK, you can route audio flexibly:

**Mac/Linux with QjackCtl:**
```bash
# Install QjackCtl for GUI routing
brew install qjackctl  # Mac
sudo apt install qjackctl  # Linux
```

**Command-line routing:**
```bash
# Connect Device outputs to system playback
jack_connect Device:output_1 system:playback_1
jack_connect Device:output_2 system:playback_2
```

## Running as a Service (Linux/Pi)

For headless operation:

```bash
# Create systemd service /etc/systemd/system/device.service
[Unit]
Description=Device Audio Synth
After=sound.target

[Service]
Type=simple
User=pi
Environment=DISPLAY=:0
ExecStartPre=/usr/bin/jackd -d alsa -d hw:0 -p 256 -n 2 &
ExecStart=/home/pi/device -b jack
Restart=always

[Install]
WantedBy=multi-user.target
```

Enable and start:
```bash
sudo systemctl enable device
sudo systemctl start device
```
