#!/bin/bash
# ProbMachine Kiosk Start Script
# Starts JACK, waits for it, then runs the synth in Cage

export SAMPLE_RATE=48000
export BUFFER_SIZE=256
export XDG_RUNTIME_DIR=/run/user/$(id -u)

# Ensure runtime dir exists
mkdir -p $XDG_RUNTIME_DIR

# Wait for JACK to be ready
for i in {1..30}; do
    if jack_lsp >/dev/null 2>&1; then
        echo "JACK is ready"
        break
    fi
    echo "Waiting for JACK... ($i/30)"
    sleep 1
done

# Run device in Cage (use -- to pass arguments to the app)
exec cage -s -- /home/uchu/probmachine/target/release/device -b jack