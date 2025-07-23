#!/bin/bash
set -e
# Rustile Development Tools - Consolidated utility script

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

show_help() {
    cat << EOF
Rustile Development Tools

USAGE:
    $0 <command> [options]

COMMANDS:
    setup           Setup development environment
    test            Run comprehensive tests  
    layout          Test layout switching
    clean           Clean build artifacts and caches
    check           Run all quality checks (fmt, clippy, test)
    release         Build release binary

EXAMPLES:
    $0 setup        # Initial development setup
    $0 test         # Run test suite with Xephyr
    $0 layout       # Interactive layout testing
    $0 check        # Pre-commit quality checks
    $0 clean        # Clean all build artifacts

EOF
}

setup_dev() {
    echo "🔧 Setting up Rustile development environment..."
    
    # Check dependencies
    if ! command -v Xephyr &> /dev/null; then
        echo "⚠️  Warning: Xephyr not found. Install with: sudo apt-get install xserver-xephyr"
    fi
    
    # Setup config inline
    if [ -f "$PROJECT_ROOT/config.example.toml" ]; then
        echo "📋 Setting up configuration..."
        
        CONFIG_DIR="$HOME/.config/rustile"
        CONFIG_FILE="$CONFIG_DIR/config.toml"
        
        # Create config directory if it doesn't exist
        if [ ! -d "$CONFIG_DIR" ]; then
            mkdir -p "$CONFIG_DIR"
            echo "Created config directory: $CONFIG_DIR"
        fi
        
        # Check if config file already exists
        if [ -f "$CONFIG_FILE" ]; then
            echo "Config file already exists at: $CONFIG_FILE"
        else
            echo "Copying example config to: $CONFIG_FILE"
            cp "$PROJECT_ROOT/config.example.toml" "$CONFIG_FILE"
            echo "✅ Configuration setup complete!"
            echo "💡 Edit $CONFIG_FILE to customize settings"
        fi
    fi
    
    # Build project
    echo "🔨 Building project..."
    cd "$PROJECT_ROOT"
    cargo build
    
    echo "✅ Development environment ready!"
    echo "💡 Run '$0 test' to start testing"
}

run_tests() {
    echo "🧪 Running Rustile test suite..."
    cd "$PROJECT_ROOT"
    
    # Unit tests
    echo "📋 Running unit tests..."
    cargo test
    
    # Integration tests with Xephyr
    if command -v Xephyr &> /dev/null; then
        echo "🖥️  Starting interactive test environment..."
        run_xephyr_test
    else
        echo "⚠️  Skipping Xephyr tests (not installed)"
    fi
}

run_xephyr_test() {
    echo "🔨 Building..."
    cargo build --release
    
    cleanup_test() {
        pkill -f "Xephyr :10" 2>/dev/null || true
        pkill -f "rustile" 2>/dev/null || true
        pkill -f "DISPLAY=:10" 2>/dev/null || true
    }
    
    trap cleanup_test EXIT INT TERM
    
    echo "📺 Starting Xephyr..."
    Xephyr :10 -screen 1200x800 > /dev/null 2>&1 &
    sleep 2
    
    echo "🚀 Starting Rustile..."
    DISPLAY=:10 RUST_LOG=debug "$PROJECT_ROOT/target/release/rustile" &
    sleep 1
    
    echo "✨ Opening test windows..."
    DISPLAY=:10 xterm -geometry 40x15 -title "Test Window 1" &
    sleep 1
    DISPLAY=:10 xterm -geometry 40x15 -title "Test Window 2" &
    sleep 1
    DISPLAY=:10 xterm -geometry 40x15 -title "Test Window 3" &
    
    echo ""
    echo "🎮 Interactive Test Environment Ready!"
    echo "📋 Try these shortcuts:"
    echo "   Alt+j/k    - Focus next/previous window"
    echo "   Shift+Alt+m - Swap with master"
    echo "   Shift+Alt+1 - Launch terminal"
    echo ""
    echo "💡 Use switch_layout.sh in another terminal to test layout switching"
    echo "📺 Close Xephyr window to exit"
    echo ""
    
    wait
}

test_layout() {
    echo "🎮 Starting layout testing environment..."
    cd "$PROJECT_ROOT"
    
    if ! command -v Xephyr &> /dev/null; then
        echo "❌ Xephyr required for layout testing"
        echo "Install with: sudo apt-get install xserver-xephyr"
        exit 1
    fi
    
    echo "📋 Current layout: $(grep layout_algorithm ~/.config/rustile/config.toml 2>/dev/null | sed 's/.*"\(.*\)".*/\1/' || echo 'master_stack')"
    echo "🔄 Use '$PROJECT_ROOT/switch_layout.sh' to change layouts"
    echo ""
    
    run_xephyr_test
}

clean_all() {
    echo "🧹 Cleaning Rustile build artifacts..."
    cd "$PROJECT_ROOT"
    
    # Clean Rust artifacts
    cargo clean
    
    # Clean any test processes
    pkill -f "Xephyr.*:10" 2>/dev/null || true
    pkill -f "rustile" 2>/dev/null || true
    
    echo "✅ Cleanup complete!"
}

quality_check() {
    echo "🔍 Running quality checks..."
    cd "$PROJECT_ROOT"
    
    echo "📝 Checking formatting..."
    cargo fmt --all -- --check
    
    echo "🔧 Running clippy..."
    cargo clippy --all-targets --all-features -- -D warnings
    
    echo "🧪 Running tests..."
    cargo test
    
    echo "📚 Checking documentation..."
    cargo doc --no-deps --document-private-items --quiet
    
    echo "✅ All quality checks passed!"
}

build_release() {
    echo "🚀 Building release binary..."
    cd "$PROJECT_ROOT"
    
    cargo build --release
    
    echo "✅ Release binary built: target/release/rustile"
    echo "📦 Size: $(du -h target/release/rustile | cut -f1)"
}

# Main command dispatcher
case "${1:-help}" in
    setup)
        setup_dev
        ;;
    test)
        run_tests
        ;;
    layout)
        test_layout
        ;;
    clean)
        clean_all
        ;;
    check)
        quality_check
        ;;
    release)
        build_release
        ;;
    help|--help|-h)
        show_help
        ;;
    *)
        echo "❌ Unknown command: $1"
        echo ""
        show_help
        exit 1
        ;;
esac