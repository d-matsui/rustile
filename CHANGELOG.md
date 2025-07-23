## [0.3.1](https://github.com/d-matsui/rustile/compare/v0.3.0...v0.3.1) (2025-07-23)

### 🐛 Bug Fixes

* use PAT for semantic-release to bypass branch protection ([a333807](https://github.com/d-matsui/rustile/commit/a33380709ed8186b997a25e116faec89766721cf))

### 📖 Documentation

* reorganize documentation structure and simplify README ([d658f52](https://github.com/d-matsui/rustile/commit/d658f52e02ca68bb6e9d91391714f8892c6ce839))

## [0.3.0](https://github.com/d-matsui/rustile/compare/v0.2.0...v0.3.0) (2025-07-23)

### ⚠ BREAKING CHANGES

* Consolidated CI jobs and Claude workflows for better performance

**CI Optimizations:**
- Merge test, format, clippy, docs into single 'test-and-quality' job
- Run security audit in parallel instead of sequentially
- Add conditional release builds (only for main/develop branches)
- Improve Rust caching with versioned keys
- Optimize system dependency installation

**Claude Workflow Consolidation:**
- Merge claude.yml and claude-code-review.yml into single workflow
- Add smart filtering to prevent duplicate runs
- Auto-review only for external contributors to reduce noise
- Implement sticky comments for PR updates

**Performance Improvements:**
- ~40% faster CI execution by eliminating job overhead
- Better cache hit rates with improved cache strategies
- Reduced compute costs through parallel execution
- Eliminated redundant dependency installations

**Documentation:**
- Add comprehensive workflow README with migration notes
- Document optimization results and usage patterns

🤖 Generated with [Claude Code](https://claude.ai/code)

Co-Authored-By: Claude <noreply@anthropic.com>

### 🚀 Features

* add BSP split ratio configuration option ([e397dab](https://github.com/d-matsui/rustile/commit/e397dabe66b9b1ff8268c700231077edcb932ca8))
* enable Claude auto-review for all PRs instead of external contributors only ([75ef5ee](https://github.com/d-matsui/rustile/commit/75ef5ee776230cedbc88ccf5cac41b2e0687ead7))
* implement BSP layout inspired by yabai window manager ([5c84646](https://github.com/d-matsui/rustile/commit/5c84646001978b9151ec5f11b0272c7929f5427c))
* optimize CI workflow for 40% faster execution ([0e91ea8](https://github.com/d-matsui/rustile/commit/0e91ea8e2ed135fc19671357eaad0fcfa25ae4bf))

### 🐛 Bug Fixes

* add comprehensive validation for all config parameters ([f63f87a](https://github.com/d-matsui/rustile/commit/f63f87a636571da51d1e9f4f9e495ec9b2c799d7))
* address final minor nitpick issues from PR review ([62e2cb7](https://github.com/d-matsui/rustile/commit/62e2cb7a0a3dd0c48bb760751ce2c4ca973b0a76))
* address minor nitpick issues from PR review ([fb208d8](https://github.com/d-matsui/rustile/commit/fb208d82d57077fe662329fe8d6835149728a4f0))
* implement configurable BSP split ratio as required by PR review ([6fd2133](https://github.com/d-matsui/rustile/commit/6fd21330121600f541179bdfa9be6fbfb65dcfe0))
* improve cargo-audit caching with direct file check ([72adb34](https://github.com/d-matsui/rustile/commit/72adb340ecc55eec6d876d6c063108da8301c2c3))
* remove unimplemented bsp_split_ratio from config documentation ([92741c7](https://github.com/d-matsui/rustile/commit/92741c7694798ecb88b237cba2506d72b58696e4))
* rename CI job to match required Test Suite status check ([5f73248](https://github.com/d-matsui/rustile/commit/5f73248bfa3599f0415c1ec69bc1a8d0e0056026))
* resolve CI clippy failure with uninlined format args ([4d6ae5e](https://github.com/d-matsui/rustile/commit/4d6ae5ed6fc6554850ae4c9af571130210226b63))
* resolve CI security audit caching issue ([bb30b32](https://github.com/d-matsui/rustile/commit/bb30b324ca4926570220a47f6061a9270527cd7f))

### 📖 Documentation

* add CODE_EXPLANATION.md to .gitignore and update with Japanese content ([edc346e](https://github.com/d-matsui/rustile/commit/edc346e55b20f3e1c59021b7d4e4cedbff0c9a48))
* remove contributor-specific language for personal project ([6b01054](https://github.com/d-matsui/rustile/commit/6b01054308a650464ee929784d2b81d07f6b07c5))
* remove unused develop branch references and clarify GitHub Flow ([8a85cef](https://github.com/d-matsui/rustile/commit/8a85cef17c8e5542164434364b2b6f856b20b581))
* update CLAUDE.md with correct CI-aligned clippy command ([b0bd1cd](https://github.com/d-matsui/rustile/commit/b0bd1cd34559d4f3c97921b154320bdae5685387))
* update CODE_EXPLANATION.md with latest features ([a59b1c2](https://github.com/d-matsui/rustile/commit/a59b1c2c40663ecd47f755c799c24f7ba7ef9cca))

### ♻️ Refactor

* comprehensive codebase cleanup and organization ([00c5fd1](https://github.com/d-matsui/rustile/commit/00c5fd199e51beb693d11145ab5696a3a56c59cd))
* consolidate documentation and remove redundant scripts ([251d208](https://github.com/d-matsui/rustile/commit/251d208cb6b39de1800c8a55fca88754d3a97c82))
* integrate layout switching into dev-tools.sh and remove standalone script ([63d98c3](https://github.com/d-matsui/rustile/commit/63d98c36919fa414054f71725b131b2b2f35445e))

### 🧪 Tests

* add comprehensive BSP layout edge case tests ([374ae1a](https://github.com/d-matsui/rustile/commit/374ae1ae64c8a8f378c850e597d6fd4cf33c5b2b))
* improve layout testing and switching infrastructure ([1e053ef](https://github.com/d-matsui/rustile/commit/1e053efdd64a75dc01abbbc7641e7849d23135ef))

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
