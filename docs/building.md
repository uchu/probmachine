# Building

## Prerequisites

All platforms: Rust nightly (see `rust-toolchain.toml`).

**Linux/Pi additional:**
```bash
sudo apt-get install build-essential pkg-config libasound2-dev libjack-jackd2-dev \
  libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev
```

## Build Commands

```bash
# Standalone
cargo build --release --bin device
cargo run --release --bin device

# VST3 + CLAP
cargo xtask bundle device --release
```

Output: `target/bundled/PhaseBurn.vst3` and `target/bundled/PhaseBurn.clap`

## Cross-Platform Targets

```bash
# macOS Intel
cargo build --release --target x86_64-apple-darwin --bin device

# macOS Apple Silicon
cargo build --release --target aarch64-apple-darwin --bin device

# Raspberry Pi (cross from x86_64)
rustup target add aarch64-unknown-linux-gnu
cargo build --release --target aarch64-unknown-linux-gnu --bin device
# Requires: sudo apt-get install gcc-aarch64-linux-gnu
# Add to ~/.cargo/config.toml: [target.aarch64-unknown-linux-gnu] linker = "aarch64-linux-gnu-gcc"
```

## Build Profiles

| Profile | LTO | Symbols | Use |
|---------|-----|---------|-----|
| debug | no | full | Development |
| release | thin | stripped | Distribution |
| profiling | thin | included | Performance analysis |

## Plugin Installation

| Format | macOS | Linux | Windows |
|--------|-------|-------|---------|
| VST3 | `~/Library/Audio/Plug-Ins/VST3/` | `~/.vst3/` | `C:\Program Files\Common Files\VST3\` |
| CLAP | `~/Library/Audio/Plug-Ins/CLAP/` | `~/.clap/` | `C:\Program Files\Common Files\CLAP\` |

## Troubleshooting

- **SIMD errors**: Ensure `rustup default nightly`
- **Missing audio backend**: Install `libjack-jackd2-dev`
- **GUI fails (Linux)**: Install `libxcb-*-dev` packages
