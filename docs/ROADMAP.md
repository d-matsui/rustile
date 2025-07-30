# Rustile Development Roadmap

This document outlines the planned features and development direction for Rustile window manager.

## 🎯 Current Status: v0.3.0

Rustile currently supports:
- ✅ BSP (Binary Space Partitioning) layout
- ✅ Configurable gaps and borders
- ✅ Window focus management with visual indication
- ✅ Keyboard navigation and shortcuts
- ✅ TOML-based configuration
- ✅ Automated CI/CD and releases

## 📋 Roadmap (~v1.0.0)

- [ ] **Basic Window Features**
  - [x] Destroy window
  - [x] Switch window
  - [x] Fullscree window
  - [ ] Rotate window
  - [ ] Zoom to parent node

- [ ] **Workspace Features**
  - [ ] Workspace creation/deletion
  - [ ] Workspace switching
  - [ ] Move windows between workspaces

- [ ] **Configuration Enhancements**
  - [ ] Live configuration reload

- [ ] **Floating Windows**
  - [ ] Toggle windows between tiling/floating
  - [ ] Floating window movement/resize
  - [ ] Float rules for specific applications

- [ ] **Refactoring/Misc**
  - [x] Simplified to BSP-only layout (removed master-stack)
  - [x] Eliminated LayoutManager abstraction
  - [x] Separated X11 operations from layout calculations
  - [ ] Comprehensive testing
  - [ ] Comprehensive docs

## 🚀 Future Considerations

- **Multi-Monitor Features**
  - Move windows between monitors

- **Simple and Usefull Key Management**
  - [ ] Distinguish Alt, AltGr (doesnt work only in Xepher?)
  - [ ] Simplify key management (use xmodmap command?)

- **Advanced Features**
  - insert window (yabai-like)
  - Auto-balance window
  - App launcher integration
  - Screen shot
  - Custom status bar
  - Mouse support

- **Wayland Support**
  - Research wlroots integration
  - Maintain X11/Wayland compatibility
  - Wayland-specific features

## 🤝 Contributing

Interested in helping with any of these features? Check out:
- [CONTRIBUTING.md](../CONTRIBUTING.md) (to be created)
- [Technical Deep Dive](TECHNICAL_DEEP_DIVE.md)
- [Development guide](../CLAUDE.md)

## 📅 Version Planning

Versions follow semantic versioning:
- **Patch (0.x.Y)**: Bug fixes and minor improvements
- **Minor (0.X.0)**: New features within a phase
- **Major (X.0.0)**: Significant architectural changes

---

*This roadmap is subject to change based on user feedback and development priorities.*