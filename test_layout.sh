#!/bin/bash
set -e  # Exit on any error
# Simple layout testing

cleanup() {
    pkill -f "Xephyr :10" 2>/dev/null || true
    pkill -f "rustile" 2>/dev/null || true
    pkill -f "DISPLAY=:10" 2>/dev/null || true
}

trap cleanup EXIT INT TERM

echo "🔨 Building..."
cargo build --release

echo "📺 Starting Xephyr..."
Xephyr :10 -screen 1200x800 > /dev/null 2>&1 &
sleep 2

echo "🚀 Starting Rustile..."
DISPLAY=:10 ./target/release/rustile &
sleep 1

echo ""
echo "✅ Ready! Opening 5 test windows..."
echo ""

# Open 5 windows with delays
DISPLAY=:10 xclock -digital &
echo "1. xclock"
sleep 1

DISPLAY=:10 xeyes &
echo "2. xeyes"
sleep 1

DISPLAY=:10 xcalc &
echo "3. xcalc"
sleep 1

DISPLAY=:10 xgc &
echo "4. xgc"
sleep 1

DISPLAY=:10 xlogo &
echo "5. xlogo"
sleep 1

echo ""
echo "🎮 Test controls:"
echo "   Alt+J/K     - Focus windows"
echo "   Shift+Alt+M - Swap with master"
echo ""
echo "Current layout: $(grep layout_algorithm ~/.config/rustile/config.toml | sed 's/.*"\(.*\)".*/\1/')"
echo ""
echo "Press Enter to exit..."
read