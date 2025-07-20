#!/bin/bash
# Test script for KeyMagic IBus engine in debug mode

set -e

# Check if IBus is running
if ! pgrep -x "ibus-daemon" > /dev/null; then
    echo "IBus daemon is not running. Starting it..."
    ibus-daemon -drx
    sleep 2
fi

# Make sure we're in the right directory
cd "$(dirname "$0")"

# Build if needed
if [ ! -f ibus-engine-keymagic ]; then
    echo "Building engine..."
    make
fi

echo "Starting KeyMagic engine in debug mode..."
echo "This will register the engine as 'keymagic-debug'"
echo ""

# Run in debug mode without --ibus flag
G_MESSAGES_DEBUG=all ./ibus-engine-keymagic --verbose &
ENGINE_PID=$!

echo "Engine started with PID: $ENGINE_PID"
echo ""
echo "You can now test the engine with:"
echo "  ibus engine keymagic-debug"
echo ""
echo "To list available engines:"
echo "  ibus list-engine | grep keymagic"
echo ""
echo "Press Ctrl+C to stop the engine"

# Wait for interrupt
trap "kill $ENGINE_PID 2>/dev/null; exit" INT
wait $ENGINE_PID