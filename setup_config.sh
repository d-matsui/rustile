#!/bin/bash
set -e  # Exit on any error
# Setup script for Rustile configuration

CONFIG_DIR="$HOME/.config/rustile"
CONFIG_FILE="$CONFIG_DIR/config.toml"

echo "Setting up Rustile configuration..."

# Create config directory if it doesn't exist
if [ ! -d "$CONFIG_DIR" ]; then
    mkdir -p "$CONFIG_DIR"
    echo "Created config directory: $CONFIG_DIR"
fi

# Check if config file already exists
if [ -f "$CONFIG_FILE" ]; then
    echo "Config file already exists at: $CONFIG_FILE"
    echo "Backing up existing config to config.toml.backup"
    cp "$CONFIG_FILE" "$CONFIG_FILE.backup"
fi

# Copy example config
if [ -f "config.example.toml" ]; then
    cp "config.example.toml" "$CONFIG_FILE"
    echo "Copied example config to: $CONFIG_FILE"
else
    echo "Error: config.example.toml not found in current directory"
    echo "Please run this script from the rustile project directory"
    exit 1
fi

echo ""
echo "Configuration setup complete!"
echo "You can now edit the config file at: $CONFIG_FILE"
echo ""
echo "Common customizations:"
echo "  - Change default_display if using different X server"
echo "  - Adjust master_ratio for different window split"
echo "  - Customize border_width, focused_border_color, unfocused_border_color"
echo "  - Add/modify shortcuts in [shortcuts] section"
echo ""
echo "Border color examples:"
echo "  - 0xFF0000 (red), 0x00FF00 (green), 0x0000FF (blue)"
echo "  - 0x808080 (gray), 0xFFFFFF (white), 0x000000 (black)"
echo ""
echo "Start rustile with: cargo run"