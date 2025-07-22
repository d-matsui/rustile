## [0.2.0](https://github.com/d-matsui/rustile/compare/v0.1.0...v0.2.0) (2025-07-22)

### 🚀 Features

* implement automated semantic release system ([5b58963](https://github.com/d-matsui/rustile/commit/5b58963616770466a870ce4c4349be14873077c1))
* implement configurable gap system for window spacing ([fbe6ac3](https://github.com/d-matsui/rustile/commit/fbe6ac3b9d604c5f5e070ce3ac4149ea1406d5de))
* implement window focus management with visual indication ([5430fe3](https://github.com/d-matsui/rustile/commit/5430fe343c4a1b29e88edcca4ddf279294268a2f))

### 🐛 Bug Fixes

* address PR review feedback for gap system robustness ([c0c32ef](https://github.com/d-matsui/rustile/commit/c0c32eff32d3e3d21b8d8ae212eb0d9b2582f33b))
* improve gap system robustness and validation ([8968dd8](https://github.com/d-matsui/rustile/commit/8968dd8fc5c4c5e87fa7621e21f631a4d2ceebaa))
* resolve semantic-release timestamp parsing error ([e0dfc43](https://github.com/d-matsui/rustile/commit/e0dfc4323fbc04bf13ad851e39ca5124e7f47739))
* use latest semantic-release plugin versions without explicit version pins ([758cf94](https://github.com/d-matsui/rustile/commit/758cf94cd82e142d2ce2d43e979da3bbba98fff7))

### 📖 Documentation

* add comprehensive development rules and conventions to CLAUDE.md ([5159138](https://github.com/d-matsui/rustile/commit/51591380706d60251c7c8fff05621431bc3f898d))
* add configuration troubleshooting guide ([a67e539](https://github.com/d-matsui/rustile/commit/a67e539d4731c1d14ca67689ab10740c88ae209d))
* update README with gap system and configuration improvements ([7235afd](https://github.com/d-matsui/rustile/commit/7235afd6d9fd0ec6e77c44ecbe20bdf868fdf6df))

### 🔧 CI/CD

* fix automated version and changelog generation ([47acc73](https://github.com/d-matsui/rustile/commit/47acc73103aa2f966602e013c367786849b63360))
* fix semantic-release cargo set-version command failure ([f531be8](https://github.com/d-matsui/rustile/commit/f531be85fc72b37c310dc972ef340118f3d1f43a))

# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Comprehensive GitHub Actions CI/CD pipeline
- Support for Ubuntu 24.04 in workflows
- Automated security auditing with cargo-audit
- Dependabot configuration for dependency updates

### Changed
- Updated workflows to use latest action versions (v4)
- Improved code formatting and clippy compliance

## [0.1.0] - 2025-07-22

### Added
- TOML-based configuration system (`~/.config/rustile/config.toml`)
- Human-readable keyboard shortcuts (e.g., `"Super+Return" = "xterm"`)
- Comprehensive modifier key support (Super, Alt, Ctrl, Shift, NumLock, AltGr, Hyper)
- Cross-platform modifier naming (Win, Cmd, Meta for compatibility)
- Case-insensitive key parsing
- Key combination parser with extensive error handling
- Master-stack tiling window layout algorithm
- X11-based window management
- Automatic window tiling and resizing
- Dynamic keyboard shortcut registration
- Configuration validation and default creation
- Comprehensive test suite (22 unit tests)
- Modular architecture with separate concerns
- Logging and error handling throughout

### Technical Details
- Written in Rust with safety and performance focus
- Uses x11rb for X11 communication
- Supports all X11 modifier keys with alternative names
- TOML configuration with serde for serialization
- Comprehensive documentation and code examples

### Infrastructure
- GitHub Actions CI/CD pipeline
- Automated testing, linting, and security auditing
- Release automation for tagged versions
- Multi-platform binary builds (Linux x86_64 glibc/musl)
- Dependabot for automated dependency updates
- Branch protection rules integration

[Unreleased]: https://github.com/d-matsui/rustile/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/d-matsui/rustile/releases/tag/v0.1.0
