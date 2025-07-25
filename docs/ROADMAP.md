# Rustile Development Roadmap

This document outlines the planned features and development direction for Rustile window manager.

## 🎯 Current Status: v0.3.0

Rustile currently supports:
- ✅ Master-Stack and BSP layouts
- ✅ Configurable gaps and borders
- ✅ Window focus management with visual indication
- ✅ Keyboard navigation and shortcuts
- ✅ TOML-based configuration
- ✅ Automated CI/CD and releases

## 📋 Roadmap (~v1.0.0)

- [ ] **Basic Window Features**
  - [x] Destroy window
  - [x] Switch window
  - [ ] Rotate window
  - [ ] Auto-balance window
  - [ ] Resize Window (full width/height, full screen)

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
  - [ ] Change default layout to bsp (master-stack is just for early development)
  - [ ] Remove unnecessary features (swap master, X, etc.)
  - [ ] Simplify key management (use xmodmap command?)
  - [ ] Comprehensive testing
  - [ ] Comprehensive docs

## 🚀 Future Considerations

- **Multi-Monitor Features**
  - Move windows between monitors

- **Advanced Features**
  - Custom status bar
  - Mouse support

- **Wayland Support**
  - Research wlroots integration
  - Maintain X11/Wayland compatibility
  - Wayland-specific features

## 🤝 Contributing

Interested in helping with any of these features? Check out:
- [CONTRIBUTING.md](../CONTRIBUTING.md) (to be created)
- [Architecture documentation](ARCHITECTURE.md)
- [Development guide](../CLAUDE.md)

## 📅 Version Planning

Versions follow semantic versioning:
- **Patch (0.x.Y)**: Bug fixes and minor improvements
- **Minor (0.X.0)**: New features within a phase
- **Major (X.0.0)**: Significant architectural changes

---

*This roadmap is subject to change based on user feedback and development priorities.*