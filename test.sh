#!/bin/bash
# Test rustile in Xephyr window

# Clean up existing instances
pkill -f "Xephyr :5" 2>/dev/null
pkill -f "rustile.*:5" 2>/dev/null

echo "Building rustile..."
cargo build || exit 1

echo "Starting Xephyr..."
Xephyr :5 -screen 1200x800 &
XEPHYR_PID=$!
sleep 2

echo "Starting rustile..."
DISPLAY=:5 RUST_LOG=debug ./target/debug/rustile &
RUSTILE_PID=$!
sleep 1

echo "Opening test windows..."
DISPLAY=:5 xterm &
sleep 0.5
DISPLAY=:5 xlogo &
sleep 0.5
DISPLAY=:5 xcalc &
sleep 0.5
DISPLAY=:5 xeyes &

echo ""
echo "Test environment ready!"
echo ""
echo "Keyboard shortcuts:"
echo "  Alt+j/k         - Focus next/previous"
echo "  Shift+Alt+j/k   - Swap with next/previous"
echo "  Shift+Alt+q     - Close window"
echo "  Alt+f           - Toggle fullscreen"
echo "  Alt+r           - Rotate windows"
echo ""
echo "Close Xephyr window to exit"

trap "kill $XEPHYR_PID $RUSTILE_PID 2>/dev/null" EXIT
wait $XEPHYR_PID