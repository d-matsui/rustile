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

## 📋 Roadmap

### Phase 1: Core Stability (v0.3.x - v0.4.x)
**Focus: Polish existing features and improve reliability**

- [ ] **Bug Fixes & Edge Cases**
  - [ ] Handle window hints (min/max size)
  - [ ] Improve floating window detection
  - [ ] Better error recovery
  
- [ ] **Configuration Enhancements**
  - [ ] Live configuration reload
  - [ ] Config validation tool
  - [ ] Per-application window rules
  
- [ ] **Developer Experience**
  - [ ] Improved debug logging
  - [ ] Better error messages
  - [ ] Development documentation

### Phase 2: Layout System (v0.5.x)
**Focus: Expand layout options and flexibility**

- [ ] **New Layouts**
  - [ ] Grid layout (configurable rows/columns)
  - [ ] Monocle/Fullscreen layout
  - [ ] Fibonacci/Spiral layout
  
- [ ] **Layout Features**
  - [ ] Layout-specific keybindings
  - [ ] Save/restore layout states
  - [ ] Dynamic gap adjustment keybindings
  - [ ] Layout cycling shortcuts

### Phase 3: Multi-Monitor Support (v0.6.x)
**Focus: Proper multi-screen management**

- [ ] **Monitor Detection**
  - [ ] Automatic monitor discovery
  - [ ] Monitor hotplug support
  - [ ] Resolution change handling
  
- [ ] **Multi-Monitor Features**
  - [ ] Independent layouts per monitor
  - [ ] Move windows between monitors
  - [ ] Monitor-aware focus navigation
  - [ ] Primary monitor configuration

### Phase 4: Workspace System (v0.7.x)
**Focus: Virtual workspace implementation**

- [ ] **Core Workspace Features**
  - [ ] Multiple virtual workspaces
  - [ ] Workspace switching keybindings
  - [ ] Per-workspace layouts
  - [ ] Workspace indicators
  
- [ ] **Advanced Workspace Features**
  - [ ] Workspace persistence
  - [ ] Move windows between workspaces
  - [ ] Workspace-specific gaps/settings
  - [ ] Dynamic workspace creation

### Phase 5: Window Management (v0.8.x)
**Focus: Advanced window control**

- [ ] **Floating Windows**
  - [ ] Toggle windows between tiling/floating
  - [ ] Floating window movement/resize
  - [ ] Float rules for specific applications
  
- [ ] **Window Features**
  - [ ] Window tagging system
  - [ ] Scratchpad functionality
  - [ ] Window swallowing
  - [ ] Minimize/restore support

### Phase 6: Integration & Extensibility (v0.9.x)
**Focus: External tool integration**

- [ ] **IPC System**
  - [ ] CLI control interface
  - [ ] Query window/workspace state
  - [ ] Runtime configuration changes
  
- [ ] **Status Bar Support**
  - [ ] Window title export
  - [ ] Workspace state export
  - [ ] Layout indicator export
  - [ ] Example status bar configs

### Phase 7: Polish & Performance (v1.0)
**Focus: Production readiness**

- [ ] **Performance**
  - [ ] Optimize layout calculations
  - [ ] Reduce X11 round trips
  - [ ] Memory usage optimization
  
- [ ] **Visual Polish**
  - [ ] Smooth window animations (optional)
  - [ ] Window transparency support
  - [ ] Theming system
  
- [ ] **Stability**
  - [ ] Comprehensive error handling
  - [ ] Crash recovery
  - [ ] Session management

## 🚀 Future Considerations (Post-1.0)

### Wayland Support
- Research wlroots integration
- Maintain X11/Wayland compatibility
- Wayland-specific features

### Plugin System
- Dynamic library loading
- Plugin API design
- Example plugins

### Advanced Features
- Built-in compositor features
- Advanced window animations
- Gesture support

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

Target release cycle: 1-2 months per minor version

---

*This roadmap is subject to change based on user feedback and development priorities.*