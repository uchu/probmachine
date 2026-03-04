# Audio Setup

## Quick Start

```bash
./scripts/dev.sh    # Build + run
./scripts/run.sh    # Run pre-built binary
```

Auto-detects platform, selects native backend, uses device's native sample rate.

## Backends

| Platform | Backend | Config |
|----------|---------|--------|
| macOS | CoreAudio | Zero-config |
| Windows | WASAPI | Zero-config |
| Linux/Pi | ALSA | Zero-config (PipeWire transparent) |

Multichannel interfaces supported — output goes to channels 1-2.

## Raspberry Pi

Scripts auto-detect Pi and configure ALSA with HiFiBerry DAC.

```bash
# /boot/config.txt
dtoverlay=hifiberry-dacplusadc

# Low-latency setup
sudo usermod -a -G audio $USER
# /etc/security/limits.d/audio.conf
@audio - rtprio 99
@audio - memlock unlimited
```

Override device: `OUTPUT_DEVICE="hw:1" ./scripts/run.sh`

## CLI Options

```
-b, --backend <BACKEND>         auto|jack|alsa|core-audio|wasapi|dummy
-p, --period-size <SIZE>        Buffer size (default: 512)
-r, --sample-rate <RATE>        Sample rate (default: device native)
--output-device <NAME>          Output device
--input-device <NAME>           Input device
--midi-input <NAME>             MIDI input device
--midi-output <NAME>            MIDI output device
```

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `BACKEND` | `auto` | Audio backend |
| `BUFFER_SIZE` | `256` | Buffer size in samples |
| `OUTPUT_DEVICE` | (auto) | Output device name |

## Buffer Size & Latency

| Buffer | Latency @48kHz |
|--------|----------------|
| 64 | ~1.3ms |
| 128 | ~2.7ms |
| 256 | ~5.3ms (recommended) |
| 512 | ~10.7ms |

## Running as Service (Pi)

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

## VST3/CLAP

When running as DAW plugin, host provides audio I/O. Standalone audio code not involved.

## Troubleshooting

- **No audio**: Check system sound output. Use `--output-device ""` to list devices.
- **Linux "Cannot open device"**: Install `libasound2-dev`, check `aplay -l`.
- **Glitches**: Increase buffer size. On Linux/Pi: configure realtime limits.
