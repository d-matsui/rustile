#!/bin/bash

# Simple Rustile focus test script

cleanup() {
    echo "🧹 Cleaning up..."
    pkill -f "Xephyr :10" 2>/dev/null || true
    pkill -f "rustile" 2>/dev/null || true
    pkill -f "DISPLAY=:10" 2>/dev/null || true
    echo "✅ Done"
}

trap cleanup EXIT INT TERM

echo "🔨 Building Rustile..."
cargo build --release

echo "🖥️  Starting Xephyr..."
Xephyr :10 -screen 1280x720 &
sleep 2

echo "🚀 Starting Rustile..."
RUST_LOG=info DISPLAY=:10 ./target/release/rustile &
sleep 1

echo "📱 Launching test windows..."
DISPLAY=:10 xclock &
DISPLAY=:10 xcalc &
DISPLAY=:10 xeyes &

echo ""
echo "✅ Test environment ready!"
echo ""
echo "Test these shortcuts in the Xephyr window:"
echo "  Alt+j - Focus next window"
echo "  Alt+k - Focus previous window" 
echo "  Shift+Alt+m - Swap with master"
echo ""
echo "Press Enter to cleanup..."
read