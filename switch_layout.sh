#!/bin/bash
# Helper script to switch between layout algorithms

CONFIG_FILE="$HOME/.config/rustile/config.toml"

if [ ! -f "$CONFIG_FILE" ]; then
    echo "❌ Config file not found: $CONFIG_FILE"
    echo "   Run rustile once to create the default config"
    exit 1
fi

# Check current layout
current_layout=$(grep "layout_algorithm" "$CONFIG_FILE" | head -1 | sed 's/.*"\(.*\)".*/\1/')

echo "🔄 Current layout: $current_layout"

case "$1" in
    "bsp")
        echo "🚀 Switching to BSP layout..."
        sed -i 's/layout_algorithm = ".*"/layout_algorithm = "bsp"/' "$CONFIG_FILE"
        echo "✅ Layout set to BSP"
        ;;
    "master_stack"|"master-stack")
        echo "🏗️  Switching to Master-Stack layout..."
        sed -i 's/layout_algorithm = ".*"/layout_algorithm = "master_stack"/' "$CONFIG_FILE"
        echo "✅ Layout set to Master-Stack"
        ;;
    "")
        # No argument - toggle
        if [ "$current_layout" = "bsp" ]; then
            echo "🏗️  Switching to Master-Stack layout..."
            sed -i 's/layout_algorithm = "bsp"/layout_algorithm = "master_stack"/' "$CONFIG_FILE"
            echo "✅ Layout set to Master-Stack"
        else
            echo "🚀 Switching to BSP layout..."
            sed -i 's/layout_algorithm = ".*"/layout_algorithm = "bsp"/' "$CONFIG_FILE"
            echo "✅ Layout set to BSP"
        fi
        ;;
    *)
        echo "❌ Unknown layout: $1"
        echo "Usage: $0 [bsp|master_stack]"
        echo "   $0         - toggle between layouts"
        echo "   $0 bsp     - switch to BSP layout"
        echo "   $0 master_stack - switch to Master-Stack layout"
        exit 1
        ;;
esac

echo ""
echo "🔧 Restart Rustile to apply the new layout!"
echo ""
echo "Current config:"
grep "layout_algorithm" "$CONFIG_FILE" | head -1