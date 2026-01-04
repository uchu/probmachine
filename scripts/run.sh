#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
BINARY="$PROJECT_DIR/target/release/device"

BUFFER_SIZE="${BUFFER_SIZE:-256}"
SAMPLE_RATE="${SAMPLE_RATE:-48000}"

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
    if command -v jackd &>/dev/null; then
        return 0
    fi
    return 1
}

is_jack_running() {
    if pgrep -x jackd &>/dev/null; then
        return 0
    fi
    return 1
}

start_jack() {
    local platform=$1
    log_info "Starting JACK server (buffer: $BUFFER_SIZE, rate: $SAMPLE_RATE)..."

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

    sleep 2

    if is_jack_running; then
        log_info "JACK server started successfully"
    else
        log_error "Failed to start JACK server"
        exit 1
    fi
}

stop_jack() {
    if is_jack_running; then
        log_info "Stopping JACK server..."
        pkill -x jackd 2>/dev/null || true
        sleep 1
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

run_device() {
    local platform=$1
    log_info "Starting Device..."

    case "$platform" in
        macos)
            export DYLD_LIBRARY_PATH="/opt/homebrew/lib:$DYLD_LIBRARY_PATH"
            ;;
    esac

    exec "$BINARY" -b jack "$@"
}

cleanup() {
    echo ""
    log_info "Shutting down..."

    # Kill device first, give it time to disconnect from JACK
    pkill -TERM -f "device" 2>/dev/null || true
    sleep 1
    pkill -KILL -f "device" 2>/dev/null || true
    sleep 0.5

    # Then stop JACK
    stop_jack
}

main() {
    local platform=$(detect_platform)
    log_info "Detected platform: $platform"

    if ! check_jack_installed; then
        install_instructions "$platform"
        exit 1
    fi

    build_if_needed

    trap cleanup EXIT INT TERM

    if ! is_jack_running; then
        start_jack "$platform"
    else
        log_info "JACK server already running"
    fi

    run_device "$platform" "$@"
}

main "$@"
