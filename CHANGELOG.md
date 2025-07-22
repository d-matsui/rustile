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