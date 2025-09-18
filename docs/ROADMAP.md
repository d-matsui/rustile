# Rustile Development Roadmap

This document outlines the planned features and development direction for Rustile window manager.

Current version: v0.8.1 - Beta quality tiling window manager

## 📊 Current Status (v0.8.1)

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

- [ ] **Directional insertion** - Insert new windows in specific directions (left/right/up/down)

### Configuration & System

- [ ] **Config file handling improvement** - Use in-memory defaults instead of auto-generating files (see [ADR-012](adr/012-config-file-handling-improvement.md))
- [ ] **Production installation** - Installation guide

### Workspace Management

- *No additional features planned for v1.0.0*

### Input & Shortcuts

- *No additional features planned for v1.0.0*

### Platform & Integration

- *No additional features planned for v1.0.0*

## 🚀 Feature Expansion

### Window Operations

- [ ] **Float toggle** - Switch windows between tiling and floating modes
- [ ] **Float movement** - Keyboard shortcuts for moving/resizing floating windows
- [ ] **Window-workspace movement** - Move windows between different workspaces

### Configuration & System

- [ ] **Auto-balance BSP tree** - Automatically balance BSP tree ratios for optimal space usage
- [ ] **Enhanced error messages** - User-friendly configuration validation errors
- [ ] **Live config reload** - Update settings without restarting rustile
- [ ] **Application rules** - Automatically float specific applications (dialogs, popups)

### Workspace Management

- [ ] **Workspace creation/deletion** - Create and manage multiple workspaces
- [ ] **Workspace switching** - Navigate between workspaces with keyboard shortcuts

### Input & Shortcuts

- [ ] **Better modifier handling** - Distinguish between left and right Alt keys
- [ ] **Shortcut conflicts detection** - Warn about conflicting keybindings
- [ ] **Mouse support** - Optional mouse interactions for window management

### Platform & Integration

- [ ] **Multi-monitor support** - Automatically detect and configure multiple monitors
- [ ] **Wayland compatibility** - Research wlroots integration, maintain X11 compatibility
- [ ] **Application launcher** - Built-in or integration with dmenu/rofi
- [ ] **Screenshot utility** - Quick screenshot functionality
- [ ] **Status bar support** - Integration with external status bars
