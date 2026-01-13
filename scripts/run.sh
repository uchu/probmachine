#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
BINARY="$PROJECT_DIR/target/release/device"

BUFFER_SIZE="${BUFFER_SIZE:-256}"
SAMPLE_RATE="${SAMPLE_RATE:-48000}"
BACKEND="${BACKEND:-jack}"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

log_info() { echo -e "${GREEN}[INFO]${NC} $1"; }
log_warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
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

check_jack_installed() {
    command -v jackd &>/dev/null
}

stop_jack() {
    if pgrep -x jackd &>/dev/null; then
        log_info "Stopping existing JACK server..."
        pkill -TERM -x jackd 2>/dev/null || true
        sleep 1
        pkill -KILL -x jackd 2>/dev/null || true
        sleep 1
    fi
}

start_jack() {
    local platform=$1
    stop_jack
    log_info "Starting JACK server at ${SAMPLE_RATE}Hz, buffer ${BUFFER_SIZE}..."

    case "$platform" in
        macos)
            jackd -d coreaudio -r "$SAMPLE_RATE" -p "$BUFFER_SIZE" &
            ;;
        linux)
            jackd -d alsa -d hw:0 -r "$SAMPLE_RATE" -p "$BUFFER_SIZE" -n 2 &
            ;;
        raspberrypi)
            if aplay -l | grep -q "hifiberry"; then
                jackd -d alsa -d hw:sndrpihifiberry -r "$SAMPLE_RATE" -p "$BUFFER_SIZE" -n 2 &
            else
                jackd -d alsa -d hw:0 -r "$SAMPLE_RATE" -p "$BUFFER_SIZE" -n 2 &
            fi
            ;;
        *)
            log_error "Unknown platform for JACK startup"
            exit 1
            ;;
    esac

    sleep 3

    if pgrep -x jackd &>/dev/null; then
        log_info "JACK server started successfully"
    else
        log_error "Failed to start JACK server"
        exit 1
    fi
}

install_instructions() {
    local platform=$1
    log_error "JACK is not installed."
    echo ""
    case "$platform" in
        macos)
            echo "Install with Homebrew:"
            echo "  brew install jack"
            ;;
        linux|raspberrypi)
            echo "Install with apt:"
            echo "  sudo apt install jackd2"
            echo ""
            echo "Or with dnf (Fedora):"
            echo "  sudo dnf install jack-audio-connection-kit"
            ;;
        windows)
            echo "Download JACK from:"
            echo "  https://jackaudio.org/downloads/"
            ;;
    esac
    echo ""
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

    pkill -TERM -f "target/release/device" 2>/dev/null || true
    sleep 1
    pkill -KILL -f "target/release/device" 2>/dev/null || true
    sleep 1

    if [ "$BACKEND" = "jack" ]; then
        stop_jack
    fi

    log_info "Stopped"
}

main() {
    local platform=$(detect_platform)
    log_info "Detected platform: $platform"
    log_info "Backend: $BACKEND, Sample rate: $SAMPLE_RATE, Buffer: $BUFFER_SIZE"

    build_if_needed

    trap cleanup EXIT INT TERM

    case "$platform" in
        macos)
            export DYLD_LIBRARY_PATH="/opt/homebrew/lib:$DYLD_LIBRARY_PATH"
            ;;
    esac

    if [ "$BACKEND" = "jack" ]; then
        if ! check_jack_installed; then
            install_instructions "$platform"
            exit 1
        fi
        start_jack "$platform"
        exec "$BINARY" -b jack "$@"
    elif [ "$BACKEND" = "coreaudio" ] || [ "$BACKEND" = "core-audio" ]; then
        log_info "Using CoreAudio at ${SAMPLE_RATE}Hz, buffer ${BUFFER_SIZE}"
        exec "$BINARY" -b core-audio -r "$SAMPLE_RATE" -p "$BUFFER_SIZE" "$@"
    elif [ "$BACKEND" = "alsa" ]; then
        log_info "Using ALSA at ${SAMPLE_RATE}Hz, buffer ${BUFFER_SIZE}"
        exec "$BINARY" -b alsa -r "$SAMPLE_RATE" -p "$BUFFER_SIZE" "$@"
    else
        exec "$BINARY" -b "$BACKEND" -r "$SAMPLE_RATE" -p "$BUFFER_SIZE" "$@"
    fi
}

main "$@"
