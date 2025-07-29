## [0.6.1](https://github.com/d-matsui/rustile/compare/v0.6.0...v0.6.1) (2025-07-29)

### 📖 Documentation

* add comprehensive architecture explanation to technical guide ([48def8c](https://github.com/d-matsui/rustile/commit/48def8cd15ff89cba97001fcdee860f1093fc85f))
* update documentation to reflect BSP-only architecture ([8800c4b](https://github.com/d-matsui/rustile/commit/8800c4bba1678b07f34d277f73d5365209353397))

### ♻️ Refactor

* remove LayoutManager abstraction ([3dad536](https://github.com/d-matsui/rustile/commit/3dad536bcef42a129dd7d60b7ce8e525c60c0807))
* separate X11 operations from layout calculations ([cd914d4](https://github.com/d-matsui/rustile/commit/cd914d4efbb52fc6829f9d5ba644bf2d69d20773))
* simplify to BSP-only layout algorithm ([006e430](https://github.com/d-matsui/rustile/commit/006e430fbb3013aa679c52e7f58b4a600aa142f4))

## [0.6.0](https://github.com/d-matsui/rustile/compare/v0.5.1...v0.6.0) (2025-07-29)

### 🚀 Features

* change fullscreen shortcut to AltGr+f (Right Alt) ([b007c87](https://github.com/d-matsui/rustile/commit/b007c87f5e853bbb846ad04f0496b085a54b1038))
* implement complete fullscreen toggle functionality ([f3369e3](https://github.com/d-matsui/rustile/commit/f3369e34b501c8e52ce1fe6abb6474fc054cc0de))
* implement fullscreen toggle feature ([4d9b1b6](https://github.com/d-matsui/rustile/commit/4d9b1b63baf87c063e92f394cf7cede473beb635))

### 🐛 Bug Fixes

* fullscreen mode now properly handles focus changes ([50ecc00](https://github.com/d-matsui/rustile/commit/50ecc00e2b552436905c72fef211ef774754f54e))

### 🧪 Tests

* add comprehensive fullscreen state transition tests ([348a8c5](https://github.com/d-matsui/rustile/commit/348a8c585758a6aac9e0c5bebb6db3fab22e687f))

## [0.5.1](https://github.com/d-matsui/rustile/compare/v0.5.0...v0.5.1) (2025-07-28)

### 🐛 Bug Fixes

* resolve PR review issues with broken references and commands ([8e739af](https://github.com/d-matsui/rustile/commit/8e739af8f48df96876c2029c25c017de1b1ff083))

### 📖 Documentation

* comprehensive README update with user-focused content ([9da0ab4](https://github.com/d-matsui/rustile/commit/9da0ab44bddb6142c1a5263d64af4559963ceb2b))
* fix FIXME comments and improve figure formatting for GitHub ([bc3e823](https://github.com/d-matsui/rustile/commit/bc3e823e5e15fc108da359d79d0714bf09c5d89b))
* remove FIXME comment from beginner guide ([92b814f](https://github.com/d-matsui/rustile/commit/92b814f0378c28441e2796b2811abe415747e109))
* restructure documentation with simplified beginner and technical guides ([4fa6e1a](https://github.com/d-matsui/rustile/commit/4fa6e1a4a77fcd16877ac92cc4cd7544f219a9e6))

### ♻️ Refactor

* simplify test scripts following KISS principle ([04ee3f3](https://github.com/d-matsui/rustile/commit/04ee3f342b9ec1f770cdf34bc0f22aac7d95563d))

## [0.5.0](https://github.com/d-matsui/rustile/compare/v0.4.0...v0.5.0) (2025-07-24)

### 🚀 Features

* implement window swapping functionality with Shift+Alt+j/k shortcuts ([115774b](https://github.com/d-matsui/rustile/commit/115774bd018ecf57bd1c59abdf1ba48648863995))

## [0.4.0](https://github.com/d-matsui/rustile/compare/v0.3.4...v0.4.0) (2025-07-24)

### 🚀 Features

* implement destroy/close window functionality ([0dee404](https://github.com/d-matsui/rustile/commit/0dee40428c9ef384294d1a4a1a7b38c1a5ab2b0d))

### 📖 Documentation

* comprehensive documentation update with GitHub-friendly ASCII art ([b7cbfa4](https://github.com/d-matsui/rustile/commit/b7cbfa42d1fc100f93c508a4e1ff2aac46577918))

## [0.3.4](https://github.com/d-matsui/rustile/compare/v0.3.3...v0.3.4) (2025-07-23)

### 🐛 Bug Fixes

* resolve cargo fmt formatting issues for CI compliance ([72bb8c2](https://github.com/d-matsui/rustile/commit/72bb8c2f7f1f79e828d1cb856a1a71520e777d89))
* resolve compiler warnings for conditional debug imports ([ddd4dc2](https://github.com/d-matsui/rustile/commit/ddd4dc220b670048b191dd295571669416319406))
* swap_with_master now reapplies layout to update window positions ([d9f088e](https://github.com/d-matsui/rustile/commit/d9f088e148101742a8853ad34b05c927cdf3a892))
* update default display from :1 to :10 for test environment compatibility ([7c9df1f](https://github.com/d-matsui/rustile/commit/7c9df1faca598f66193c83e1df683b2452f2d645))

### 📖 Documentation

* add comprehensive educational documentation with visual diagrams ([19b785e](https://github.com/d-matsui/rustile/commit/19b785ead55d4e412192ddd4fa42a4efbc6c0ae4))
* add destroy window to roadmap basic features ([af33c7d](https://github.com/d-matsui/rustile/commit/af33c7ded51d38f6599c5d71c25eeaaf26e1a9b0))
* enforce zero-warning builds and fix code formatting ([b8a2aff](https://github.com/d-matsui/rustile/commit/b8a2aff117c99b36fb6e9519d341055e968a8466))
* restore yabai reference in README acknowledgments ([942226c](https://github.com/d-matsui/rustile/commit/942226ccfecebe80b4f3816c731f1bbd4ecd8ec5))
* update ROADMAP.md with refined development priorities ([db3b4a1](https://github.com/d-matsui/rustile/commit/db3b4a13d4ed411b2e67882d8cf8ec4116574bec))

### ♻️ Refactor

* complete layout.rs modularization with improved code organization ([c854661](https://github.com/d-matsui/rustile/commit/c85466139b04c467b7eb4e130f44a052533998ce))
* extract constants and introduce parameter structs for better maintainability ([40b3b2b](https://github.com/d-matsui/rustile/commit/40b3b2b6aff3ff19bf213399819fbbe1184307cc))
* implement layout trait and improve config validation system ([f3f6964](https://github.com/d-matsui/rustile/commit/f3f6964635a48212d714e2f482d780245908a627))
* split layout.rs into modules and remove yabai references ([565b6d7](https://github.com/d-matsui/rustile/commit/565b6d7b76787a0e6b69264320036f16962a762f))
* split window_manager.rs into focused modules for better organization ([e9a253a](https://github.com/d-matsui/rustile/commit/e9a253adbd5195b57a013fc4289c14a83be9f226))

### 🧪 Tests

* add comprehensive window manager tests for core business logic ([158915b](https://github.com/d-matsui/rustile/commit/158915b6b73cec3e6b37aca16aa3be6d55051c0e))
* enhance test script to open 4 diverse X11 applications ([4826d4f](https://github.com/d-matsui/rustile/commit/4826d4f80da4c6a5015162b1b4aea26860f585e4))

## [0.3.3](https://github.com/d-matsui/rustile/compare/v0.3.2...v0.3.3) (2025-07-23)

### 🐛 Bug Fixes

* prevent docs commits from triggering releases ([c88ab8c](https://github.com/d-matsui/rustile/commit/c88ab8c08c2601887d9263982b2e231d463a5bed))

## [0.3.2](https://github.com/d-matsui/rustile/compare/v0.3.1...v0.3.2) (2025-07-23)

### 📖 Documentation

* improve roadmap structure and acknowledgments ([5bd3022](https://github.com/d-matsui/rustile/commit/5bd30226c549afc34f06603e8f3b9a0564cdefe0))

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
