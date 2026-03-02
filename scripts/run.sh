#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
BINARY="$PROJECT_DIR/target/release/phaseburn"

BUFFER_SIZE="${BUFFER_SIZE:-256}"
BACKEND="${BACKEND:-auto}"

RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'

log_info() { echo -e "${GREEN}[INFO]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }

detect_platform() {
    case "$(uname -s)" in
        Darwin*) echo "macos" ;;
        Linux*)
            if grep -q "Raspberry Pi" /proc/cpuinfo 2>/dev/null; then
                echo "raspberrypi"
            else
                echo "linux"
            fi
            ;;
        MINGW*|MSYS*|CYGWIN*) echo "windows" ;;
        *) echo "unknown" ;;
    esac
}

build_if_needed() {
    if [ ! -f "$BINARY" ]; then
        log_info "Building release binary..."
        cd "$PROJECT_DIR"
        cargo build --release
    fi
}

cleanup() {
    echo ""
    log_info "Shutting down..."
    pkill -TERM -f "target/release/phaseburn" 2>/dev/null || true
    sleep 1
    pkill -KILL -f "target/release/phaseburn" 2>/dev/null || true
    log_info "Stopped"
}

main() {
    local platform=$(detect_platform)
    log_info "Platform: $platform, Backend: $BACKEND, Buffer: $BUFFER_SIZE"

    build_if_needed

    trap cleanup EXIT INT TERM

    case "$platform" in
        macos)
            export DYLD_LIBRARY_PATH="/opt/homebrew/lib:$DYLD_LIBRARY_PATH"
            ;;
        raspberrypi)
            if [ "$BACKEND" = "auto" ]; then
                BACKEND="alsa"
                log_info "Raspberry Pi detected, using ALSA backend with HiFiBerry"
            fi
            OUTPUT_DEVICE="${OUTPUT_DEVICE:-snd_rpi_hifiberry_dacplusadc}"
            ;;
        linux)
            if [ "$BACKEND" = "auto" ]; then
                BACKEND="alsa"
                log_info "Linux detected, using ALSA backend"
            fi
            ;;
    esac

    if [ -n "$OUTPUT_DEVICE" ]; then
        exec "$BINARY" -b "$BACKEND" -p "$BUFFER_SIZE" --output-device "$OUTPUT_DEVICE" "$@"
    else
        exec "$BINARY" -b "$BACKEND" -p "$BUFFER_SIZE" "$@"
    fi
}

main "$@"
