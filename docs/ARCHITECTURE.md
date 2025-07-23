# Rustile Architecture

This document provides a comprehensive overview of Rustile's architecture, from project structure to implementation details.

## Table of Contents

1. [Project Overview](#project-overview)
2. [Project Structure](#project-structure)
3. [Core Components](#core-components)
4. [Event Flow](#event-flow)
5. [Configuration System](#configuration-system)
6. [Layout Algorithms](#layout-algorithms)
7. [Development Workflow](#development-workflow)

## Project Overview

Rustile is a tiling window manager for X11 written in Rust. It automatically arranges windows without overlapping, providing keyboard-driven window management with configurable layouts.

### Key Features
- Master-Stack and BSP (Binary Space Partitioning) layouts
- Configurable gaps and window borders
- Window focus management with visual indicators
- TOML-based configuration
- Keyboard shortcuts for all operations

### Architecture Diagram
```
┌─────────────┐    ┌──────────────┐    ┌─────────────┐
│   main.rs   │───▶│WindowManager │───▶│ X11 Server  │
└─────────────┘    └──────┬───────┘    └─────────────┘
                          │
                          ▼
              ┌─────────────────────────┐
              │     Core Components     │
              ├─────────────────────────┤
              │ • LayoutManager         │
              │ • KeyboardManager       │
              │ • Config                │
              └─────────────────────────┘
```

## Project Structure

```
rustile/
├── src/                    # Core source code
│   ├── main.rs            # Entry point
│   ├── lib.rs             # Library interface
│   ├── window_manager.rs  # Core window management
│   ├── layout.rs          # Tiling algorithms
│   ├── config.rs          # Configuration system
│   ├── keyboard.rs        # Keyboard shortcuts
│   └── keys.rs            # Key parsing utilities
├── scripts/               # Development tools
│   └── dev-tools.sh       # Unified dev utility
├── docs/                  # Documentation
│   ├── ARCHITECTURE.md    # This file
│   └── ROADMAP.md        # Future plans
├── .github/workflows/     # CI/CD pipelines
├── config.example.toml    # Example configuration
└── CLAUDE.md             # Development guidelines
```

## Core Components

### 1. Main Entry Point (`main.rs`)

The entry point initializes logging, connects to X11, and starts the window manager:

```rust
fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let (conn, screen_num) = x11rb::connect(None)?;
    let wm = WindowManager::new(conn, screen_num)?;
    wm.run()
}
```

### 2. Window Manager (`window_manager.rs`)

The heart of the system that coordinates all components:

**Key Responsibilities:**
- X11 event handling
- Window lifecycle management
- Focus state tracking
- Command dispatch

**Main Data Structure:**
```rust
pub struct WindowManager<C: Connection> {
    conn: C,                           // X11 connection
    windows: Vec<Window>,              // Managed windows
    focused_window: Option<Window>,    // Current focus
    window_stack: Vec<Window>,         // MRU order
    layout_manager: LayoutManager,     // Layout algorithms
    keyboard_manager: KeyboardManager, // Shortcuts
    config: Config,                    // User settings
}
```

**Key Methods:**
- `new()` - Initialize and register as WM
- `run()` - Main event loop
- `handle_event()` - Event dispatcher
- `set_focus()` - Focus management
- `apply_layout()` - Trigger layout update

### 3. Layout System (`layout.rs`)

Implements window arrangement algorithms:

**Supported Layouts:**
```rust
pub enum Layout {
    MasterStack,  // Traditional master + stack
    Bsp,          // Binary space partitioning
}
```

**Master-Stack Layout:**
- One master window on the left (configurable ratio)
- Remaining windows stacked vertically on the right
- Configurable gaps between windows

**BSP Layout:**
- Recursive binary splits of screen space
- Alternating horizontal/vertical splits
- Tree-based window organization

### 4. Configuration (`config.rs`)

TOML-based configuration with validation:

```toml
[layout]
layout_algorithm = "master_stack"  # or "bsp"
master_ratio = 0.5
gap = 10
border_width = 2
focused_border_color = 0xFF0000
unfocused_border_color = 0x808080

[shortcuts]
"Alt+j" = "focus_next"
"Alt+k" = "focus_prev"
"Shift+Alt+m" = "swap_with_master"
"Super+Return" = "xterm"
```

**Validation Rules:**
- Gap: 0-500 pixels
- Border width: 0-50 pixels
- Master ratio: 0.0-1.0
- Combined gap + border ≤ 600 pixels

### 5. Keyboard Management (`keyboard.rs` & `keys.rs`)

Handles keyboard shortcuts and key parsing:

**Key Features:**
- Human-readable key combinations
- Support for all X11 modifiers
- Dynamic shortcut registration
- Command dispatch system

**Example Key Parsing:**
```rust
"Super+Return" → (ModMask::M4, 0xff0d)
"Ctrl+Alt+Delete" → (ModMask::CONTROL | ModMask::M1, 0xffff)
```

## Event Flow

### Window Creation
```
Application starts
    ↓
X11 sends MapRequest
    ↓
WindowManager.handle_map_request()
    ├── Set border attributes
    ├── Make window visible
    ├── Add to window list
    ├── Set focus (update borders)
    └── Apply layout algorithm
        ↓
Windows arranged on screen
```

### Keyboard Input
```
User presses key combination
    ↓
X11 sends KeyPress event
    ↓
KeyboardManager matches shortcut
    ↓
Execute command:
    ├── Window commands (focus_next, swap_with_master)
    └── Launch applications
```

### Window Closing
```
Window closes
    ↓
X11 sends UnmapNotify
    ↓
Remove from window list
    ↓
Update focus if needed
    ↓
Re-apply layout
```

## Configuration System

### Configuration Loading
1. Check `~/.config/rustile/config.toml`
2. Fall back to defaults if not found
3. Validate all values
4. Apply settings to window manager

### Runtime Configuration
- Configuration loaded at startup
- No hot-reload currently (planned feature)
- Validation ensures safe values
- Clear error messages for invalid configs

## Layout Algorithms

### Master-Stack Algorithm
```
1. Calculate available space (screen - gaps)
2. Position master window:
   - X: gap
   - Y: gap  
   - Width: master_ratio * available_width
   - Height: screen_height - 2*gap
3. Stack remaining windows:
   - X: gap + master_width + gap
   - Y: gap + index * (stack_height + gap)
   - Equal height distribution
```

### BSP Algorithm
```
1. Start with root node (full screen)
2. For each window:
   - Find target node (leaf)
   - Split node (alternating H/V)
   - Create two children
   - Place old window in one child
   - Place new window in other child
3. Apply recursive layout to tree
```

## Development Workflow

### Building and Testing
```bash
# Setup development environment
./scripts/dev-tools.sh setup

# Run comprehensive tests
./scripts/dev-tools.sh test

# Check code quality
./scripts/dev-tools.sh check

# Test layouts interactively
./scripts/dev-tools.sh layout
```

### Code Quality Standards
- **Formatting**: `cargo fmt` (required)
- **Linting**: `cargo clippy -- -D warnings`
- **Testing**: Unit and integration tests
- **Error Handling**: Always use `Result<T>`
- **Documentation**: Public APIs documented

### Adding New Features

1. **New Layout Algorithm:**
   - Add variant to `Layout` enum
   - Implement algorithm in `LayoutManager`
   - Add configuration options
   - Update example config

2. **New Keyboard Command:**
   - Add command to shortcuts config
   - Implement handler in `WindowManager`
   - Update documentation

3. **New Configuration Option:**
   - Add field to config structs
   - Implement validation
   - Update defaults and example
   - Document in README

## Error Handling

- Use `anyhow::Result<T>` for error propagation
- Never use `unwrap()` in production code
- Log errors with context using `tracing`
- Graceful degradation where possible

## Performance Considerations

- Minimize X11 round trips
- Cache expensive calculations
- Use efficient data structures
- Profile with many windows

## Testing Strategy

### Unit Tests
- Test each component in isolation
- Cover edge cases
- Validate error handling

### Integration Tests
- Test with Xephyr
- Verify layout algorithms
- Test focus management

### Manual Testing
```bash
# Start test environment
./scripts/dev-tools.sh layout

# In another terminal
./scripts/dev-tools.sh switch bsp
```

---

*For development guidelines and contribution rules, see [CLAUDE.md](../CLAUDE.md)*  
*For future development plans, see [ROADMAP.md](ROADMAP.md)*