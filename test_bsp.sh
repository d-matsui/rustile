#!/bin/bash
# Test script for BSP layout in Rustile

cleanup() {
    echo "🧹 Cleaning up..."
    pkill -f "Xephyr :10" 2>/dev/null || true
    pkill -f "rustile" 2>/dev/null || true
    pkill -f "DISPLAY=:10" 2>/dev/null || true
    echo "✅ Test completed!"
}

trap cleanup EXIT INT TERM

echo "🚀 Testing BSP Layout in Rustile"
echo "================================="

echo "🔨 Building Rustile..."
cargo build --release

echo "📺 Starting Xephyr on display :10..."
Xephyr :10 -screen 1280x720 &
sleep 2

echo "🏗️  Starting Rustile window manager with BSP layout..."
RUST_LOG=info DISPLAY=:10 ./target/release/rustile &
sleep 1

echo "🪟 Opening test windows..."
echo "   - Window 1: xclock (should fill full screen)"
DISPLAY=:10 xclock -digital &
sleep 1

echo "   - Window 2: xeyes (should split screen vertically 50/50)"
DISPLAY=:10 xeyes &
sleep 1

echo "   - Window 3: xcalc (should split focused window horizontally)"
DISPLAY=:10 xcalc &
sleep 1

echo ""
echo "✅ BSP Test Environment Ready!"
echo ""
echo "🎮 Test these shortcuts in the Xephyr window:"
echo "   Alt+J      - Focus next window"  
echo "   Alt+K      - Focus previous window"
echo "   Shift+Alt+M - Swap with master"
echo ""
echo "📋 BSP Layout Behavior You Should See:"
echo "   - First window (xclock): Full screen"
echo "   - Second window (xeyes): Screen splits vertically 50/50"
echo "   - Third window (xcalc): Splits focused window horizontally"
echo "   - Pattern: Vertical → Horizontal → Vertical → Horizontal"
echo ""
echo "⚙️  Current Layout: BSP (Binary Space Partitioning)"
echo "   To switch layouts, use: ./switch_layout.sh [bsp|master_stack]"
echo ""
echo "Press Enter to cleanup and exit..."
read