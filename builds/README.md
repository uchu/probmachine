# Device - Build Artifacts

Cross-platform builds for Device synthesizer with factory presets.

## Platforms

### macOS (Intel/Apple Silicon)
- **Standalone**: `macos/standalone/device`
- **VST3**: `macos/vst3/device.vst3`
- **CLAP**: `macos/clap/device.clap`

### Windows (x86_64)
- **Standalone**: `windows/standalone/device.exe`
- **VST3**: `windows/vst3/device.vst3`
- **CLAP**: `windows/clap/device.clap`

### Linux (x86_64)
- **Standalone**: `linux/standalone/device`
- **VST3**: `linux/vst3/device.vst3`
- **CLAP**: `linux/clap/device.clap`

### Raspberry Pi (ARM64)
- **Standalone**: `raspberry-pi/standalone/device`

## Installation

Extract the archive for your platform and copy files to the appropriate locations.

**macOS**: `~/Library/Audio/Plug-Ins/{VST3,CLAP}/`
**Windows**: `C:\Program Files\Common Files\{VST3,CLAP}\`
**Linux**: `~/.vst3/` or `~/.clap/`

## Audio Backends

- **macOS**: JACK (recommended), CoreAudio
- **Windows**: WASAPI (default), JACK
- **Linux**: JACK, ALSA
- **Raspberry Pi**: JACK

## Notes

- Windows build includes 16MB stack size fix
- macOS: Run with `./scripts/dev.sh` or `device -b core-audio`
- All builds include factory presets bank A (slots 1-12)

Build date: $(date +%Y-%m-%d)
