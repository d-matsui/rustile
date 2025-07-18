#!/bin/bash
# Test script for rustile window manager

echo "Starting Rustile test environment..."

# Kill any existing Xephyr
killall Xephyr 2>/dev/null

# Start Xephyr
Xephyr :10 -screen 1280x720 &
XEPHYR_PID=$!
sleep 2

# Run rustile with debug logging
echo "Starting rustile with debug logging..."
DISPLAY=:10 RUST_LOG=debug cargo run &
RUSTILE_PID=$!
sleep 3

# Launch test windows
echo "Launching test windows..."
DISPLAY=:10 xclock -title "Window 1" &
sleep 1
DISPLAY=:10 xclock -title "Window 2" &
sleep 1
DISPLAY=:10 xclock -title "Window 3" &
sleep 1

echo ""
echo "Test environment is running!"
echo "- Press Mod4+T in Xephyr to test keyboard shortcut"
echo "- Close/open windows to test tiling"
echo ""
echo "Press Enter to stop test..."
read

# Cleanup
echo "Cleaning up..."
kill $RUSTILE_PID 2>/dev/null
kill $XEPHYR_PID 2>/dev/null
killall xclock 2>/dev/null


echo "Test complete!"