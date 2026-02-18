# Device - Building Guide

## Prerequisites

### All Platforms
- Rust nightly toolchain (see `rust-toolchain.toml`)
- Cargo

### macOS
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup default nightly
```

### Linux (Ubuntu/Debian)
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup default nightly

# Install dependencies
sudo apt-get update
sudo apt-get install -y build-essential pkg-config libasound2-dev libjack-jackd2-dev libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev
```

### Raspberry Pi (Raspberry Pi OS Lite)
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup default nightly

# Install dependencies
sudo apt-get update
sudo apt-get install -y build-essential pkg-config libasound2-dev libjack-jackd2-dev libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev
```

### Windows
- Install Rust from https://rustup.rs
- Run `rustup default nightly` in terminal

## Build Commands

### Standalone Application

**Debug build (fast compile, slower runtime):**
```bash
cargo build --bin device
```

**Release build (optimized):**
```bash
cargo build --release --bin device
```

**Run standalone:**
```bash
cargo run --release --bin device
```

### VST3 Plugin

**Build VST3 for current platform:**
```bash
cargo xtask bundle device --release
```

**Output location:**
- macOS: `target/bundled/Device.vst3`
- Linux: `target/bundled/Device.vst3`
- Windows: `target/bundled/Device.vst3`

### CLAP Plugin

**Build CLAP for current platform:**
```bash
cargo xtask bundle device --release
```

**Output location:**
- macOS: `target/bundled/Device.clap`
- Linux: `target/bundled/Device.clap`
- Windows: `target/bundled/Device.clap`

## Platform-Specific Builds

### macOS (Intel)
```bash
rustup target add x86_64-apple-darwin
cargo build --release --target x86_64-apple-darwin --bin device
cargo xtask bundle device --release --target x86_64-apple-darwin
```

### macOS (Apple Silicon)
```bash
rustup target add aarch64-apple-darwin
cargo build --release --target aarch64-apple-darwin --bin device
cargo xtask bundle device --release --target aarch64-apple-darwin
```

### macOS (Universal Binary)
```bash
# Build for both architectures
cargo build --release --target x86_64-apple-darwin --bin device
cargo build --release --target aarch64-apple-darwin --bin device

# Create universal binary manually using lipo (plugins need special handling)
cargo xtask bundle device --release --target x86_64-apple-darwin
cargo xtask bundle device --release --target aarch64-apple-darwin
```

### Linux x86_64
```bash
cargo build --release --bin device
cargo xtask bundle device --release
```

### Raspberry Pi 5 (aarch64)

**On the Pi itself:**
```bash
cargo build --release --bin device
```

**Cross-compile from Linux x86_64:**
```bash
# Install cross-compilation tools
sudo apt-get install gcc-aarch64-linux-gnu

# Add target
rustup target add aarch64-unknown-linux-gnu

# Configure cargo for cross-compilation (add to ~/.cargo/config.toml)
# [target.aarch64-unknown-linux-gnu]
# linker = "aarch64-linux-gnu-gcc"

# Build
cargo build --release --target aarch64-unknown-linux-gnu --bin device
```

### Windows x86_64
```bash
cargo build --release --bin device
cargo xtask bundle device --release
```

## Build Profiles

### Release (default for distribution)
```bash
cargo build --release
```
- LTO: thin (faster linking)
- Symbols: stripped

### Profiling (for performance analysis)
```bash
cargo build --profile profiling
```
- LTO: thin
- Debug symbols: included
- Not stripped

### Debug (for development)
```bash
cargo build
```
- No optimizations
- Full debug symbols

## Installation

### VST3
Copy `target/bundled/Device.vst3` to:
- **macOS**: `~/Library/Audio/Plug-Ins/VST3/` or `/Library/Audio/Plug-Ins/VST3/`
- **Linux**: `~/.vst3/` or `/usr/lib/vst3/`
- **Windows**: `C:\Program Files\Common Files\VST3\`

### CLAP
Copy `target/bundled/Device.clap` to:
- **macOS**: `~/Library/Audio/Plug-Ins/CLAP/` or `/Library/Audio/Plug-Ins/CLAP/`
- **Linux**: `~/.clap/` or `/usr/lib/clap/`
- **Windows**: `C:\Program Files\Common Files\CLAP\`

### Standalone
Place the binary anywhere and run it. On first launch, it will create:
- **macOS**: `~/Library/Application Support/Device/`
- **Linux**: `~/.local/share/Device/`
- **Windows**: `%APPDATA%\Device\`

## Troubleshooting

### Build fails with SIMD errors
Ensure you're using Rust nightly:
```bash
rustup default nightly
rustc --version  # Should show "nightly"
```

### Missing audio backend on Linux
Install JACK development libraries:
```bash
sudo apt-get install libjack-jackd2-dev
```

### GUI fails to display on Linux
Install XCB development libraries:
```bash
sudo apt-get install libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev
```

### Windows standalone has no sound
Run from a command prompt to see backend errors:
```bash
device.exe
```

If you see "falling back to the dummy audio backend", try specifying the backend and sample rate:
```bash
device.exe --backend wasapi
device.exe --sample-rate 44100
device.exe --output-device ""  # Lists available devices
```

### Raspberry Pi performance
For best performance on Raspberry Pi:
1. Use the standalone binary (not VST3)
2. Set oversampling to 1x or 4x
3. Run with JACK audio backend
4. Use `nice -n -20` for real-time priority
