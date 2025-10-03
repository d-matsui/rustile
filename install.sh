#!/bin/bash
# install.sh - Build and install rustile to /usr/local/bin

set -e

MODE=${1:-release}

if [ "$MODE" = "debug" ]; then
    echo "Building rustile in debug mode..."
    RUSTFLAGS="-D warnings" cargo build
    echo "Installing to /usr/local/bin/rustile..."
    sudo cp target/debug/rustile /usr/local/bin/rustile
    echo "✓ Installed rustile (debug) to /usr/local/bin/rustile"
elif [ "$MODE" = "release" ]; then
    echo "Building rustile in release mode..."
    RUSTFLAGS="-D warnings" cargo build --release
    echo "Installing to /usr/local/bin/rustile..."
    sudo cp target/release/rustile /usr/local/bin/rustile
    echo "✓ Installed rustile (release) to /usr/local/bin/rustile"
else
    echo "Usage: $0 [release|debug]"
    echo "  Default: release"
    exit 1
fi
