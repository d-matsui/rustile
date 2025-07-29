#!/bin/bash
# Test fullscreen feature

echo "Starting fullscreen test..."
echo "Instructions:"
echo "1. Press Alt+f to toggle fullscreen on focused window"
echo "2. Press Alt+j/k to switch focus"
echo "3. Press Alt+f again on different window"
echo "4. Press Shift+Alt+q to close windows"
echo ""

# Run test script with multiple windows
./test.sh &
TEST_PID=$!
sleep 3

# Open multiple test windows
DISPLAY=:10 xterm -title "Window 1" &
sleep 1
DISPLAY=:10 xterm -title "Window 2" &
sleep 1
DISPLAY=:10 xterm -title "Window 3" &

echo "Test windows opened. Try fullscreen with Alt+f!"

# Wait for test script
wait $TEST_PID