# Rustile Development Roadmap

This document outlines the planned features and development direction for Rustile window manager.

## 📊 Current Status

### Window Operations

- ✅ **BSP tiling layout** - Binary space partitioning window management
- ✅ **Window operations** - Focus, destroy, swap, rotate, fullscreen, zoom-to-parent
- ✅ **Visual focus management** - Red borders for focused, gray for unfocused windows

### Configuration & System

- ✅ **TOML configuration** - Runtime validation, gaps, borders, split ratios
- ✅ **Comprehensive testing** - 66 unit tests covering core functionality
- ✅ **CI/CD automation** - Semantic releases, security audits, code quality checks
- ✅ **Standardized logging** - 3-level logging with tracing framework
- ✅ **Modular architecture** - Clean separation of concerns (7 focused modules)
- ✅ **Zero-warning builds** - Strict clippy rules, automated formatting
- ✅ **Single source of truth** - Eliminated duplicate state management

### Workspace Management

- ✅ **Single workspace** - Current implementation supports one workspace

### Input & Shortcuts

- ✅ **Keyboard shortcuts** - Comprehensive Alt+key bindings for all operations

### Platform & Integration

- ✅ **X11 support** - Full X11 window management integration
- ✅ **Comprehensive documentation** - ADRs, implementation guides, user documentation

## 🎯 v1.0.0 - Stable Release

### Window Operations

### Configuration & System

- [x] **Config file handling improvement** - Use in-memory defaults instead of auto-generating files (see [ADR-012](adr/012-config-file-handling-improvement.md))
- [x] **Production installation** - Installation guide

### Workspace Management

### Input & Shortcuts

### Platform & Integration

## 🚀 Feature Expansion

### Window Operations

- [ ] **Directional insertion** - Insert new windows in specific directions (left/right/up/down)
- [ ] **Float toggle** - Switch windows between tiling and floating modes
- [ ] **Float movement** - Keyboard shortcuts for moving/resizing floating windows

### Configuration & System

- [ ] **Auto-balance BSP tree** - Automatically balance BSP tree ratios for optimal space usage
- [ ] **Enhanced error messages** - User-friendly configuration validation errors
- [ ] **Live config reload** - Update settings without restarting rustile
- [ ] **Application rules** - Automatically float specific applications (dialogs, popups)

### Workspace Management

- [ ] **Workspace creation/deletion** - Create and manage multiple workspaces
- [ ] **Workspace switching** - Navigate between workspaces with keyboard shortcuts
- [ ] **Window-workspace movement** - Move windows between different workspaces

### Input & Shortcuts

- [ ] **Better modifier handling** - Distinguish between left and right Alt keys
- [ ] **Shortcut conflicts detection** - Warn about conflicting keybindings
- [ ] **Mouse support** - Optional mouse interactions for window management

### Platform & Integration

- [ ] **Multi-monitor support** - Automatically detect and configure multiple monitors
- [ ] **Application launcher** - Built-in or integration with dmenu/rofi
- [ ] **Status bar support** - Integration with external status bars
- [ ] **Screenshot utility** - Quick screenshot functionality
- [ ] **Wayland compatibility** - Research wlroots integration, maintain X11 compatibility

## 🐛 Known Issues & Bug Fixes

### Window Management Bugs

- [ ] **Black root window issue** - After closing all windows with Shift+Alt+q, a black root window remains when opening new applications (reproduced with Emacs)
- [ ] **Chrome Xephyr compatibility** - Investigate why Google Chrome doesn't launch in Xephyr test environment
