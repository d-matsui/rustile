# Rustile Development Roadmap

This document outlines the planned features and development direction for Rustile window manager.

## 📊 Current Status

### Window Operations

- [x] **BSP tiling layout** - Binary space partitioning window management
- [x] **Window operations** - Focus, destroy, swap, rotate, fullscreen, zoom-to-parent
- [x] **Visual focus management** - Red borders for focused, gray for unfocused windows

### Configuration & System

- [x] **TOML configuration** - Runtime validation, gaps, borders, split ratios
- [x] **Comprehensive testing** - 66 unit tests covering core functionality
- [x] **CI/CD automation** - Semantic releases, security audits, code quality checks
- [x] **Standardized logging** - 3-level logging with tracing framework
- [x] **Modular architecture** - Clean separation of concerns (7 focused modules)
- [x] **Zero-warning builds** - Strict clippy rules, automated formatting
- [x] **Single source of truth** - Eliminated duplicate state management

### Workspace Management

- [x] **Single workspace** - Current implementation supports one workspace

### Input & Shortcuts

- [x] **Keyboard shortcuts** - Comprehensive Alt+key bindings for all operations

### Platform & Integration

- [x] **X11 support** - Full X11 window management integration
- [x] **Comprehensive documentation** - ADRs, implementation guides, user documentation
- [x] **Clipboard setup guide** - Documentation for X11 clipboard manager setup (xclip) and middle-button paste configuration

## 🚀 Feature Expansion

### Window Operations

- [ ] **Float toggle** - Switch windows between tiling and floating modes
- [ ] **Float movement** - Keyboard shortcuts for moving/resizing floating windows

### Configuration & System

- [x] **Manual balance BSP tree** - Manual command to balance BSP tree ratios for optimal space usage (Shift+Alt+0)
- [ ] **Enhanced error messages** - User-friendly configuration validation errors
- [ ] **Live config reload** - Update settings without restarting rustile
- [x] **Restart shortcut** - Keyboard shortcut to restart rustile (pkill + startx)
- [ ] **Application rules** - Automatically float specific applications (dialogs, popups)
- [ ] **Enhanced debug messages** - User-friendly debug logging (human-readable logs)
- [ ] **Automated integration testing** - Command-line driven tests in Xephyr with log verification (eliminate manual ./test.sh verification)

### Workspace Management

- [x] **Workspace creation/deletion** - Create and manage multiple workspaces
- [x] **Workspace switching** - Navigate between workspaces with keyboard shortcuts
- 🔥 [ ] **Window-workspace movement** - Move windows between different workspaces

### Input & Shortcuts

- [x] **Better modifier handling** - Distinguish between left and right Alt keys
- [ ] **Shortcut conflicts detection** - Warn about conflicting keybindings
- [ ] **Mouse support** - Optional mouse interactions for window management

### Platform & Integration

- 🔥 [ ] **Multi-monitor support** - Automatically detect and configure multiple monitors
- 🔥 [ ] **Status bar support** - Integration with external status bars
- 🔥 [ ] **Screenshot utility** - Quick screenshot functionality
- [ ] **Application launcher** - Built-in or integration with dmenu/rofi
- [ ] **Wayland compatibility** - Research wlroots integration, maintain X11 compatibility

## 🐛 Known Issues & Bug Fixes

### Window Management Bugs

- [x] **Emacs double MapRequest bug** - Fixed: Emacs sent duplicate MapRequest events causing invisible windows and layout issues (fixed in v1.0.1)

## 🔧 CI/CD & Infrastructure

### Build & Release Process

- [ ] **CI/CD documentation** - Document current workflows and troubleshooting guide
- [ ] **Workflow simplification** - Reduce complexity in GitHub Actions workflows
