# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a tiling window manager written in Rust using x11rb for X11 window management. The project implements basic tiling functionality with a master-stack layout.

## Development Commands

Since this is a Rust project, standard Cargo commands will be used:

- `cargo build` - Build the project
- `cargo run` - Build and run the project
- `cargo test` - Run tests
- `cargo check` - Check code without building
- `cargo fmt` - Format code
- `cargo clippy` - Run linter
- `cargo doc` - Generate documentation

## Current Features

- **Basic Window Management**: Handles MapRequest and UnmapNotify events
- **Master-Stack Tiling**: First window takes half the screen (master), subsequent windows stack vertically on the right
- **Keyboard Shortcuts**: Mod4+T launches xcalc as a test application
- **X11 Integration**: Uses x11rb for low-level X11 protocol handling

## Project Structure

- `src/main.rs` - Main entry point with window manager logic
- `Cargo.toml` - Project configuration with dependencies:
  - `x11rb` - X11 Rust bindings
  - `anyhow` - Error handling
  - `tracing` - Logging infrastructure
  - `xkeysym` - X11 keysym definitions
- `target/` - Build artifacts (ignored by git)

## Development Notes

- The project uses a standard Rust .gitignore that excludes build artifacts, backup files, and IDE-specific files
- Mutation testing support is configured (mutants.out directories are ignored)
- The project is set up for standard Rust development workflow

## Key Components

### Main Loop (src/main.rs)
- Establishes connection to X11 server
- Registers as window manager by setting SUBSTRUCTURE_REDIRECT
- Main event loop handles:
  - `KeyPress`: Keyboard shortcuts (currently Mod4+T)
  - `MapRequest`: New window creation
  - `UnmapNotify`: Window destruction

### Tiling Algorithm
The `tile()` function implements a master-stack layout:
- Master window: Takes left half of screen
- Stack windows: Split right half vertically
- Single window: Full screen

## Testing

Run in Xephyr as shown in README:
```bash
Xephyr :1 -screen 1280x720
DISPLAY=:10 cargo run
```

## Next Steps for Development

- Add more tiling layouts (horizontal split, grid, etc.)
- Implement workspace/desktop management
- Add configuration file support
- Handle window focus and borders
- Implement more keyboard shortcuts
- Add window floating support