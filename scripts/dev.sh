#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

BUFFER_SIZE="${BUFFER_SIZE:-256}"
SAMPLE_RATE="${SAMPLE_RATE:-48000}"

cd "$PROJECT_DIR"

is_jack_running() {
    pgrep -x jackd &>/dev/null
}

start_jack_if_needed() {
    if ! is_jack_running; then
        echo "[INFO] Starting JACK server..."
        case "$(uname -s)" in
            Darwin*)
                jackd -d coreaudio -r "$SAMPLE_RATE" -p "$BUFFER_SIZE" &
                ;;
            Linux*)
                jackd -d alsa -d hw:0 -r "$SAMPLE_RATE" -p "$BUFFER_SIZE" -n 2 &
                ;;
        esac
        sleep 2
    fi
}

cleanup() {
    echo ""
    echo "[INFO] Shutting down..."

    # Kill device first, give it time to disconnect from JACK
    pkill -TERM -f "target/release/device" 2>/dev/null || true
    sleep 1

    # Force kill if still running
    pkill -KILL -f "target/release/device" 2>/dev/null || true
    sleep 0.5

    # Then stop JACK
    pkill -TERM -x jackd 2>/dev/null || true
    sleep 0.5
    pkill -KILL -x jackd 2>/dev/null || true

    echo "[INFO] Stopped"
}

trap cleanup EXIT INT TERM

start_jack_if_needed

case "$(uname -s)" in
    Darwin*)
        export DYLD_LIBRARY_PATH="/opt/homebrew/lib:$DYLD_LIBRARY_PATH"
        ;;
esac

cargo run --release -- -b jack "$@"
