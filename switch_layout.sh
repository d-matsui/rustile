#!/bin/bash
set -e  # Exit on any error
# Robust layout switcher

CONFIG="$HOME/.config/rustile/config.toml"

# Check if config exists
if [ ! -f "$CONFIG" ]; then
    echo "❌ Config file not found: $CONFIG"
    echo "   Run rustile once to create the default config"
    exit 1
fi

# Get current layout
current_layout=$(grep "^layout_algorithm" "$CONFIG" | head -1 | cut -d'"' -f2)
echo "Current layout: $current_layout"

# Handle arguments
case "$1" in
    "bsp"|"b")
        new_layout="bsp"
        ;;
    "master"|"m"|"master_stack"|"master-stack"|"ms")
        new_layout="master_stack"
        ;;
    "")
        # Toggle
        if [ "$current_layout" = "bsp" ]; then
            new_layout="master_stack"
        else
            new_layout="bsp"
        fi
        ;;
    *)
        echo "❌ Unknown layout: $1"
        echo "Usage: $0 [bsp|master_stack|master|b|m]"
        echo "   No argument = toggle between layouts"
        exit 1
        ;;
esac

# Apply the change
if sed -i "s/^layout_algorithm = \".*\"/layout_algorithm = \"$new_layout\"/" "$CONFIG"; then
    echo "✅ Switched to $([ "$new_layout" = "bsp" ] && echo "BSP" || echo "Master-Stack") layout"
    
    # Verify the change
    new_value=$(grep "^layout_algorithm" "$CONFIG" | head -1 | cut -d'"' -f2)
    if [ "$new_value" = "$new_layout" ]; then
        echo "✓ Verified: layout_algorithm = \"$new_value\""
    else
        echo "⚠️  Warning: Change may not have applied correctly"
    fi
else
    echo "❌ Failed to update config file"
    exit 1
fi

echo ""
echo "🔧 Restart rustile to apply the new layout"
