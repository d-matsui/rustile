# Rustile Development Roadmap

This document outlines the planned features and development direction for Rustile window manager.

## 🎯 Current Status (v0.7.x)

Rustile currently provides:

- **✅ Core Window Management**
  - BSP (Binary Space Partitioning) tiling layout
  - Window focus with visual borders (red=focused, gray=unfocused)
  - Basic window operations (destroy, rotate, fullscreen, swap)
  - Keyboard-driven workflow with customizable shortcuts

- **✅ Configuration & Quality**
  - TOML-based configuration with validation
  - Configurable gaps, borders, and split ratios
  - Comprehensive test suite (66 tests)
  - Automated CI/CD with semantic releases
  - Standardized logging with debug support

- ⭐⭐⭐ **Window Operations**
  - [x] **Zoom to parent** - Focus and expand window to its parent container size *(v0.8.0)*
  - [ ] **Auto-balance** - Automatically balance BSP tree ratios for optimal space usage
  - [ ] **Directional insertion** - Insert new windows in specific directions (left/right/up/down)

- **Configuration**
  - [ ] **Live config reload** - Update settings without restarting rustile
  - ⭐ [ ] **Enhanced logging** - Show application names ("xterm") instead of window IDs
  - [ ] **Better error messages** - User-friendly configuration validation errors

- **Floating Windows**
  - ⭐ [ ] **Float toggle** - Switch windows between tiling and floating modes  
  - [ ] **Float movement** - Keyboard shortcuts for moving/resizing floating windows
  - [ ] **Application rules** - Automatically float specific applications (dialogs, popups)

- **Keyboard Improvements**
  - [ ] ⭐ **Better modifier handling** - Distinguish between left and right Alt keys
  - [ ] **Key management simplification** - Consider using xmodmap for cleaner key handling
  - [ ] **Shortcut conflicts detection** - Warn about conflicting keybindings

- ⭐⭐ **Multi-Workspace Support**
  - [ ] **Workspace creation/deletion** - Create and manage multiple workspaces
  - [ ] **Workspace switching** - Navigate between workspaces with keyboard shortcuts
  - [ ] **Window-workspace movement** - Move windows between different workspaces

- ⭐ **Multi-Monitor**
  - [ ] **Monitor detection** - Automatically detect and configure multiple monitors
  - [ ] **Window movement** - Move windows between different monitors
  - [ ] **Per-monitor workspaces** - Independent workspace management per monitor

- **Integrations**
  - [ ] **Application launcher** - Built-in or integration with dmenu/rofi
  - [ ] **Screenshot utility** - Quick screenshot functionality
  - [ ] **Status bar support** - Integration with external status bars
  - [ ] **Mouse support** - Optional mouse interactions for window management

- **Architecture & Refactoring**
  - [ ] **Screen rect calculation cleanup** - Move rendering calculations out of BSP tree module
  - [ ] **Responsibility separation** - Ensure BSP tree focuses purely on tree operations
  - [ ] **Code duplication elimination** - Remove duplicated screen rectangle calculations

- **Wayland Support**
  - [ ] **Research wlroots** - Investigate Wayland compositor integration
  - [ ] **Protocol compatibility** - Maintain X11 compatibility alongside Wayland
  - [ ] **Wayland-specific features** - Leverage Wayland-only capabilities