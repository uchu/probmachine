#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

BUFFER_SIZE="${BUFFER_SIZE:-256}"
SAMPLE_RATE="${SAMPLE_RATE:-48000}"
BACKEND="${BACKEND:-jack}"

cd "$PROJECT_DIR"

stop_jack() {
    if pgrep -x jackd &>/dev/null; then
        echo "[INFO] Stopping existing JACK server..."
        pkill -TERM -x jackd 2>/dev/null || true
        sleep 1
        pkill -KILL -x jackd 2>/dev/null || true
        sleep 0.5
    fi
}

start_jack() {
    stop_jack
    echo "[INFO] Starting JACK server at ${SAMPLE_RATE}Hz, buffer ${BUFFER_SIZE}..."
    case "$(uname -s)" in
        Darwin*)
            jackd -d coreaudio -r "$SAMPLE_RATE" -p "$BUFFER_SIZE" &
            ;;
        Linux*)
            jackd -d alsa -d hw:0 -r "$SAMPLE_RATE" -p "$BUFFER_SIZE" -n 2 &
            ;;
    esac
    sleep 3
    # Verify actual rate
    if command -v jack_samplerate &>/dev/null; then
        ACTUAL_RATE=$(jack_samplerate 2>/dev/null || echo "unknown")
        echo "[INFO] JACK actual sample rate: ${ACTUAL_RATE}"
    fi
}

cleanup() {
    echo ""
    echo "[INFO] Shutting down..."

    pkill -TERM -f "target/release/device" 2>/dev/null || true
    sleep 1
    pkill -KILL -f "target/release/device" 2>/dev/null || true
    sleep 0.5

    if [ "$BACKEND" = "jack" ]; then
        pkill -TERM -x jackd 2>/dev/null || true
        sleep 0.5
        pkill -KILL -x jackd 2>/dev/null || true
    fi

    echo "[INFO] Stopped"
}

trap cleanup EXIT INT TERM

case "$(uname -s)" in
    Darwin*)
        export DYLD_LIBRARY_PATH="/opt/homebrew/lib:$DYLD_LIBRARY_PATH"
        ;;
esac

if [ "$BACKEND" = "jack" ]; then
    start_jack
    cargo run --release -- -b jack "$@"
elif [ "$BACKEND" = "coreaudio" ] || [ "$BACKEND" = "core-audio" ]; then
    echo "[INFO] Using CoreAudio at ${SAMPLE_RATE}Hz, buffer ${BUFFER_SIZE}"
    cargo run --release -- -b core-audio -r "$SAMPLE_RATE" -p "$BUFFER_SIZE" "$@"
elif [ "$BACKEND" = "alsa" ]; then
    echo "[INFO] Using ALSA at ${SAMPLE_RATE}Hz, buffer ${BUFFER_SIZE}"
    cargo run --release -- -b alsa -r "$SAMPLE_RATE" -p "$BUFFER_SIZE" "$@"
else
    cargo run --release -- -b "$BACKEND" -r "$SAMPLE_RATE" -p "$BUFFER_SIZE" "$@"
fi
