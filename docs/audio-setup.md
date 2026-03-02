# Audio Setup Guide

PhaseBurn uses native platform audio backends by default. No additional software installation is required.

## Quick Start

```bash
# Development (builds + runs)
./scripts/dev.sh

# Production (runs pre-built binary)
./scripts/run.sh

# Direct binary
./target/release/phaseburn        # Mac/Linux
phaseburn.exe                     # Windows
```

The app automatically detects the platform, selects the native audio backend, and uses the device's native sample rate.

## How It Works

The standalone binary uses nih-plug's backend system:
- **macOS**: CoreAudio (native, zero-config)
- **Windows**: WASAPI (native, zero-config)
- **Linux / Raspberry Pi**: ALSA (native, zero-config)

The app queries the default output device's native sample rate and uses it. This prevents internal sample rate conversion which causes buffer size instability on CoreAudio and WASAPI.

Multichannel audio interfaces (e.g. UAD Apollo 8x8, Focusrite 18i20) are supported. The app writes stereo to the first two output channels and silences the rest.

## Platform Details

### macOS

Works out of the box with any audio device (built-in speakers, headphones, USB interfaces, Thunderbolt interfaces).

```bash
./scripts/dev.sh
```

The script sets `DYLD_LIBRARY_PATH` for Homebrew compatibility. The binary uses CoreAudio directly.

**Pro interfaces (UAD Apollo, etc.):** The multichannel patch accepts devices with more than 2 channels. Output goes to channels 1-2 (typically main L/R).

### Windows

Works out of the box with WASAPI.

```cmd
phaseburn.exe
```

Or with custom buffer size:
```cmd
phaseburn.exe -p 256
```

No JACK or ASIO installation required. WASAPI shared mode is used, which works alongside other audio applications.

### Linux Desktop

The scripts detect Linux and use ALSA directly (no JACK needed).

```bash
./scripts/dev.sh
```

On PipeWire-based systems (Ubuntu 23+, Fedora 34+), ALSA is transparently routed through PipeWire. No configuration needed.

**Build dependencies:**
```bash
sudo apt install build-essential pkg-config libasound2-dev \
  libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev
```

### Raspberry Pi 5

The scripts detect Raspberry Pi and configure ALSA with the HiFiBerry DAC as the default output device.

```bash
./scripts/run.sh
```

This runs: `phaseburn -b alsa -p 256 --output-device snd_rpi_hifiberry_dacplusadc`

**HiFiBerry setup:**
```bash
# /boot/config.txt
dtoverlay=hifiberry-dacplusadc
```

**Low-latency configuration:**
```bash
# Add user to audio group
sudo usermod -a -G audio $USER

# /etc/security/limits.d/audio.conf
@audio - rtprio 99
@audio - memlock unlimited
```

**Different audio device:** Override the output device:
```bash
# Check available devices
aplay -l

# Use a different device
OUTPUT_DEVICE="hw:1" ./scripts/run.sh
```

### VST3 / CLAP (all platforms)

When running as a plugin inside a DAW, the host provides audio I/O. None of the standalone audio backend code is involved. Sample rate and buffer size are controlled by the DAW.

## CLI Options

```
phaseburn [OPTIONS]

Options:
  -b, --backend <BACKEND>           Audio backend [default: auto]
                                    Values: auto, jack, alsa, core-audio, wasapi, dummy
  -p, --period-size <PERIOD_SIZE>   Buffer size in samples [default: 512]
  -r, --sample-rate <SAMPLE_RATE>   Sample rate in Hz [default: device native]
  --output-device <NAME>            Output device name
  --input-device <NAME>             Input device name
  --midi-input <NAME>               MIDI input device name
  --midi-output <NAME>              MIDI output device name
```

## Script Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `BACKEND` | `auto` | Audio backend (`auto`, `alsa`, `core-audio`, `wasapi`) |
| `BUFFER_SIZE` | `256` | Buffer size in samples |
| `OUTPUT_DEVICE` | (auto) | Output device name (set automatically on Raspberry Pi) |

```bash
BUFFER_SIZE=512 ./scripts/dev.sh
BACKEND=core-audio ./scripts/dev.sh
OUTPUT_DEVICE="hw:1" ./scripts/run.sh
```

## Buffer Size & Latency

| Buffer Size | Latency @ 48kHz | Latency @ 44.1kHz | Use Case |
|-------------|-----------------|-------------------|----------|
| 64 | ~1.3ms | ~1.5ms | Ultra low-latency |
| 128 | ~2.7ms | ~2.9ms | Low-latency |
| 256 | ~5.3ms | ~5.8ms | Balanced (recommended) |
| 512 | ~10.7ms | ~11.6ms | Stable |
| 1024 | ~21.3ms | ~23.2ms | Very stable |

## Sample Rate

The app uses the output device's native sample rate by default. Common values:

| Rate | Typical Devices |
|------|-----------------|
| 44100 Hz | Built-in speakers, consumer USB audio |
| 48000 Hz | Professional interfaces, HDMI output |
| 96000 Hz | Pro interfaces in high-res mode |

If the device reports a different native rate than the CLI default (48000), the app logs a message and switches automatically. PhaseBurn uses internal oversampling (configurable 1x-16x) regardless of host sample rate.

## Running as a Service (Raspberry Pi)

For headless operation:

```ini
# /etc/systemd/system/phaseburn.service
[Unit]
Description=PhaseBurn Audio Synth
After=sound.target

[Service]
Type=simple
User=pi
Environment=DISPLAY=:0
ExecStart=/home/pi/phaseburn -b alsa -p 256 --output-device snd_rpi_hifiberry_dacplusadc
Restart=always

[Install]
WantedBy=multi-user.target
```

```bash
sudo systemctl enable phaseburn
sudo systemctl start phaseburn
```

## Using JACK (Optional)

JACK is no longer required but still supported. Use it if you need inter-application audio routing.

```bash
BACKEND=jack ./scripts/dev.sh
```

When using JACK, sample rate and buffer size are controlled by the JACK server, not the app's CLI flags.

## Troubleshooting

### Mac: No audio output
- Check System Settings → Sound → Output device is correct
- Try: `phaseburn -b core-audio --output-device ""` to list available devices

### Windows: No audio output
- Check Windows Sound settings → default output device
- Try: `phaseburn.exe -b wasapi --output-device ""` to list available devices

### Linux: "Cannot open audio device"
- Check `aplay -l` for available devices
- Ensure `libasound2-dev` is installed
- Try: `phaseburn -b alsa --output-device ""` to list available devices

### Raspberry Pi: Wrong output device
- Run `aplay -l` to find the correct device name
- Override: `OUTPUT_DEVICE="your_device" ./scripts/run.sh`

### Audio glitches / crackling
- Increase buffer size: `BUFFER_SIZE=512 ./scripts/dev.sh`
- On Linux/Pi: configure realtime limits (see Raspberry Pi section)
- Ensure no other application is exclusively using the audio device
