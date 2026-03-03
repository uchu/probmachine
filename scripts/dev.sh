#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

BUFFER_SIZE="${BUFFER_SIZE:-256}"
BACKEND="${BACKEND:-auto}"

cd "$PROJECT_DIR"

cleanup() {
    echo ""
    echo "[INFO] Shutting down..."
    pkill -TERM -f "target/release/phaseburn" 2>/dev/null || true
    sleep 1
    pkill -KILL -f "target/release/phaseburn" 2>/dev/null || true
    echo "[INFO] Stopped"
}

trap cleanup EXIT INT TERM

case "$(uname -s)" in
    Darwin*)
        if [ "$BACKEND" = "auto" ]; then
            BACKEND="core-audio"
            echo "[INFO] macOS detected, using CoreAudio backend"
        fi
        ;;
    Linux*)
        if [ "$BACKEND" = "auto" ]; then
            BACKEND="alsa"
            echo "[INFO] Linux detected, using ALSA backend"
        fi
        if grep -q "Raspberry Pi" /proc/cpuinfo 2>/dev/null; then
            OUTPUT_DEVICE="${OUTPUT_DEVICE:-snd_rpi_hifiberry_dacplusadc}"
            echo "[INFO] Raspberry Pi detected, output: $OUTPUT_DEVICE"
        fi
        ;;
esac

echo "[INFO] Backend: $BACKEND, Buffer: $BUFFER_SIZE"
if [ -n "$OUTPUT_DEVICE" ]; then
    cargo run --release --bin phaseburn -- -b "$BACKEND" -p "$BUFFER_SIZE" --output-device "$OUTPUT_DEVICE" "$@"
else
    cargo run --release --bin phaseburn -- -b "$BACKEND" -p "$BUFFER_SIZE" "$@"
fi
