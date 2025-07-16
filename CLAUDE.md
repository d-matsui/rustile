# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a tiling window manager written in Rust. The project is in its initial stages with only a basic README and .gitignore file currently present.

## Development Commands

Since this is a Rust project, standard Cargo commands will be used:

- `cargo build` - Build the project
- `cargo run` - Build and run the project
- `cargo test` - Run tests
- `cargo check` - Check code without building
- `cargo fmt` - Format code
- `cargo clippy` - Run linter
- `cargo doc` - Generate documentation

## Project Structure

The project is currently in its initial state with no source code yet. When development begins, it will likely follow standard Rust project structure:

- `src/` - Source code directory
- `src/main.rs` - Main entry point
- `Cargo.toml` - Project configuration and dependencies
- `target/` - Build artifacts (ignored by git)

## Development Notes

- The project uses a standard Rust .gitignore that excludes build artifacts, backup files, and IDE-specific files
- Mutation testing support is configured (mutants.out directories are ignored)
- The project is set up for standard Rust development workflow

## Window Manager Context

As a tiling window manager, this project will likely involve:
- X11 or Wayland protocol interaction
- Window management and layout algorithms
- Configuration parsing and management
- Event handling for window creation/destruction
- Keyboard and mouse input handling

When implementing features, consider the typical architecture of window managers and follow Rust best practices for systems programming.