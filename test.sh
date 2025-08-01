#!/bin/bash
# Test rustile in Xephyr window

# Clean up any existing test instances
pkill -f "Xephyr :10" 2>/dev/null
pkill -f "rustile.*:10" 2>/dev/null

# Setup clean config from example
echo "Setting up config..."
mkdir -p ~/.config/rustile
cp config.example.toml ~/.config/rustile/config.toml

# Build rustile (debug mode for better logging)
echo "Building rustile..."
cargo build || exit 1

# Start test X server
echo "Starting test window..."
Xephyr :10 -screen 1200x800 &
XEPHYR_PID=$!
sleep 2

# Run rustile (debug build)
echo "Starting rustile..."
DISPLAY=:10 ./target/debug/rustile &
RUSTILE_PID=$!
sleep 1

# Open test applications
echo "Opening test windows..."
DISPLAY=:10 xterm &
sleep 0.5
DISPLAY=:10 xlogo &
sleep 0.5
DISPLAY=:10 xcalc &
sleep 0.5
DISPLAY=:10 xeyes &

echo ""
echo "Test environment ready!"
echo ""
echo "Keyboard shortcuts:"
echo "  Alt+j/k         - Focus next/previous"
echo "  Shift+Alt+j/k   - Swap with next/previous"
echo "  Shift+Alt+q     - Close window"
echo "  Alt+f           - Toggle fullscreen"
echo "  Alt+r           - Rotate window (flip parent split direction)"
echo ""
echo "Close Xephyr window to exit"

# Clean up on exit
trap "kill $XEPHYR_PID $RUSTILE_PID 2>/dev/null" EXIT

# Wait for Xephyr to close
wait $XEPHYR_PID