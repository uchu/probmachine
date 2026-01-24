#!/usr/bin/env bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
BUILD_DIR="$PROJECT_DIR/builds"

log_info() {
    echo "[INFO] $1"
}

log_success() {
    echo "[SUCCESS] $1"
}

log_error() {
    echo "[ERROR] $1"
}

detect_platform() {
    case "$(uname -s)" in
        Darwin*)    echo "macos";;
        Linux*)     echo "linux";;
        MINGW*|MSYS*|CYGWIN*) echo "windows";;
        *)          echo "unknown";;
    esac
}

setup_build_dir() {
    log_info "Setting up build directory..."
    rm -rf "$BUILD_DIR"
    mkdir -p "$BUILD_DIR"/{macos,windows,linux,raspberry-pi}/{vst3,clap,standalone}
}

build_plugins() {
    local platform=$1
    log_info "Building plugins for $platform..."

    cd "$PROJECT_DIR"
    cargo xtask bundle device --release

    case "$platform" in
        macos)
            if [ -d "target/bundled/device.vst3" ]; then
                cp -r target/bundled/device.vst3 "$BUILD_DIR/macos/vst3/"
                log_success "VST3 plugin built for macOS"
            fi
            if [ -d "target/bundled/device.clap" ]; then
                cp -r target/bundled/device.clap "$BUILD_DIR/macos/clap/"
                log_success "CLAP plugin built for macOS"
            fi
            ;;
        linux)
            if [ -f "target/bundled/device.vst3" ]; then
                cp -r target/bundled/device.vst3 "$BUILD_DIR/linux/vst3/"
                log_success "VST3 plugin built for Linux"
            fi
            if [ -f "target/bundled/device.clap" ]; then
                cp target/bundled/device.clap "$BUILD_DIR/linux/clap/"
                log_success "CLAP plugin built for Linux"
            fi
            ;;
        windows)
            if [ -f "target/bundled/device.vst3" ]; then
                cp -r target/bundled/device.vst3 "$BUILD_DIR/windows/vst3/"
                log_success "VST3 plugin built for Windows"
            fi
            if [ -f "target/bundled/device.clap" ]; then
                cp target/bundled/device.clap "$BUILD_DIR/windows/clap/"
                log_success "CLAP plugin built for Windows"
            fi
            ;;
    esac
}

build_standalone() {
    local platform=$1
    local target=$2

    log_info "Building standalone for $platform (target: $target)..."

    cd "$PROJECT_DIR"

    if [ -n "$target" ]; then
        cargo build --release --bin device --target "$target"
        local binary_path="target/$target/release/device"
    else
        cargo build --release --bin device
        local binary_path="target/release/device"
    fi

    case "$platform" in
        macos)
            if [ -f "${binary_path}" ]; then
                cp "${binary_path}" "$BUILD_DIR/macos/standalone/device"
                chmod +x "$BUILD_DIR/macos/standalone/device"
                log_success "Standalone built for macOS"
            fi
            ;;
        linux)
            if [ -f "${binary_path}" ]; then
                cp "${binary_path}" "$BUILD_DIR/linux/standalone/device"
                chmod +x "$BUILD_DIR/linux/standalone/device"
                log_success "Standalone built for Linux"
            fi
            ;;
        windows)
            if [ -f "${binary_path}.exe" ]; then
                cp "${binary_path}.exe" "$BUILD_DIR/windows/standalone/device.exe"
                log_success "Standalone built for Windows"
            fi
            ;;
        raspberry-pi)
            if [ -f "${binary_path}" ]; then
                cp "${binary_path}" "$BUILD_DIR/raspberry-pi/standalone/device"
                chmod +x "$BUILD_DIR/raspberry-pi/standalone/device"
                log_success "Standalone built for Raspberry Pi"
            fi
            ;;
    esac
}

build_cross_platform_windows() {
    log_info "Cross-compiling for Windows..."

    if ! command -v cargo-xwin &> /dev/null; then
        log_error "cargo-xwin not found. Install with: cargo install cargo-xwin"
        return 1
    fi

    cd "$PROJECT_DIR"

    log_info "Building Windows standalone binary..."
    cargo xwin build --release --bin device --target x86_64-pc-windows-msvc

    if [ -f "target/x86_64-pc-windows-msvc/release/device.exe" ]; then
        cp "target/x86_64-pc-windows-msvc/release/device.exe" "$BUILD_DIR/windows/standalone/"
        log_success "Windows standalone cross-compiled"
    fi

    log_info "Building Windows plugin library..."
    cargo xwin build --release --lib --target x86_64-pc-windows-msvc

    if [ -f "target/x86_64-pc-windows-msvc/release/device.dll" ]; then
        mkdir -p "$BUILD_DIR/windows/vst3/device.vst3/Contents/x86_64-win"
        cp "target/x86_64-pc-windows-msvc/release/device.dll" "$BUILD_DIR/windows/vst3/device.vst3/Contents/x86_64-win/device.vst3"
        log_success "Windows VST3 plugin built"

        cp "target/x86_64-pc-windows-msvc/release/device.dll" "$BUILD_DIR/windows/clap/device.clap"
        log_success "Windows CLAP plugin built"
    fi
}

build_cross_platform_linux() {
    log_info "Cross-compiling for Linux..."

    if ! command -v cross &> /dev/null; then
        log_error "cross not found. Install with: cargo install cross"
        return 1
    fi

    cross build --release --bin device --target x86_64-unknown-linux-gnu

    if [ -f "target/x86_64-unknown-linux-gnu/release/device" ]; then
        cp "target/x86_64-unknown-linux-gnu/release/device" "$BUILD_DIR/linux/standalone/"
        chmod +x "$BUILD_DIR/linux/standalone/device"
        log_success "Linux standalone cross-compiled"
    fi
}

build_raspberry_pi() {
    log_info "Building for Raspberry Pi..."

    if ! command -v cross &> /dev/null; then
        log_error "cross not found. Install with: cargo install cross"
        log_error "Skipping Raspberry Pi build"
        return 1
    fi

    cross build --release --bin device --target aarch64-unknown-linux-gnu

    if [ -f "target/aarch64-unknown-linux-gnu/release/device" ]; then
        cp "target/aarch64-unknown-linux-gnu/release/device" "$BUILD_DIR/raspberry-pi/standalone/"
        chmod +x "$BUILD_DIR/raspberry-pi/standalone/device"
        log_success "Raspberry Pi standalone built"
    fi
}

create_readme() {
    log_info "Creating README..."

    cat > "$BUILD_DIR/README.md" << 'EOF'
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
EOF

    log_success "README created"
}

create_archives() {
    log_info "Creating archives..."

    cd "$BUILD_DIR"

    for platform_dir in */; do
        platform="${platform_dir%/}"
        if [ -d "$platform" ] && [ "$(ls -A $platform 2>/dev/null)" ]; then
            tar -czf "${platform}.tar.gz" "$platform"
            log_success "Created ${platform}.tar.gz ($(du -sh ${platform}.tar.gz | cut -f1))"
        fi
    done
}

print_summary() {
    log_info "Build Summary"
    echo "======================================"

    for platform in macos windows linux raspberry-pi; do
        echo ""
        echo "[$platform]"
        for format in vst3 clap standalone; do
            dir="$BUILD_DIR/$platform/$format"
            if [ -d "$dir" ] && [ "$(ls -A $dir 2>/dev/null)" ]; then
                echo "  ✓ $format: $(ls -1 $dir | head -1)"
            else
                echo "  ✗ $format: not built"
            fi
        done
    done

    echo ""
    echo "======================================"
    log_success "All builds complete! Output in: $BUILD_DIR"
}

main() {
    local current_platform=$(detect_platform)
    log_info "Detected platform: $current_platform"

    setup_build_dir

    case "$current_platform" in
        macos)
            log_info "Building native macOS binaries..."
            build_plugins "macos"
            build_standalone "macos" ""

            if [ "${CROSS_COMPILE:-yes}" = "yes" ]; then
                log_info "Cross-compiling for other platforms..."
                build_cross_platform_windows || true
                build_cross_platform_linux || true
                build_raspberry_pi || true
            fi
            ;;

        linux)
            log_info "Building native Linux binaries..."
            build_plugins "linux"
            build_standalone "linux" ""

            if [ "${CROSS_COMPILE:-yes}" = "yes" ]; then
                build_raspberry_pi || true
            fi
            ;;

        windows)
            log_info "Building native Windows binaries..."
            build_plugins "windows"
            build_standalone "windows" ""
            ;;

        *)
            log_error "Unsupported platform: $current_platform"
            exit 1
            ;;
    esac

    create_readme
    create_archives
    print_summary
}

main "$@"
