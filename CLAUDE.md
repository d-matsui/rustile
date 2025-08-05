# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a tiling window manager written in Rust using x11rb for X11 window management. The project implements BSP (Binary Space Partitioning) layout with configurable gaps, focus management, and keyboard shortcuts.

## Development Rules and Standards

### Code Formatting and Quality

**MANDATORY: Always run these commands before committing:**
```bash
source ~/.cargo/env  # Ensure cargo is in PATH
cargo fmt           # Format code
cargo build --all-targets --all-features  # Build all targets to catch warnings
cargo clippy --all-targets --all-features -- -D warnings  # Check for lints (treat warnings as errors)
cargo test          # Run all tests
```

**Pre-Commit Quality Requirements:**
- **Zero Warnings**: Builds MUST produce no warnings (warnings indicate potential issues)
- **Formatting**: All code MUST be formatted with `cargo fmt` before commits
- **Linting**: All clippy warnings MUST be resolved (use `--all-targets --all-features -- -D warnings` flags to match CI)
  - **CRITICAL**: The `--all-targets --all-features` flags are required to catch issues in test code and all build configurations
  - **CI Alignment**: This exact command must pass locally before commits to prevent CI failures
- **Clean Build**: `cargo build --all-targets --all-features` must complete without warnings
- **Testing**: All tests MUST pass before commits
- **Documentation**: Use `///` for public APIs, `//!` for module-level docs
- **Error Handling**: Use `anyhow::Result` for error propagation, never use `unwrap()` in production code
- **Code Comments**: Follow concise comment standard (see ADR-005 for full details)
  - Single-line function descriptions: document "what", not "how"
  - Remove obvious inline comments that restate the code
  - Keep essential X11 protocol and business logic explanations
  - Eliminate tutorial-style verbosity in code comments

### Forbidden Rust Attributes

**NEVER use these attributes to suppress warnings or errors:**

**Code Quality Suppressors - FORBIDDEN:**
- `#[allow(dead_code)]` - Remove unused code instead
- `#[allow(unused_variables)]` - Use `_` prefix for intentionally unused vars
- `#[allow(unused_imports)]` - Remove unused imports
- `#[allow(unused_mut)]` - Remove unnecessary `mut`
- `#[allow(unreachable_code)]` - Remove unreachable code
- `#[allow(unused_must_use)]` - Handle `Result`/`Option` properly

**Safety/Convention Suppressors - FORBIDDEN:**
- `#[allow(unsafe_code)]` - Avoid unsafe code in this project
- `#[allow(non_snake_case)]` - Follow Rust naming conventions
- `#[allow(clippy::all)]` - Fix clippy warnings instead
- `#[allow(clippy::unwrap_used)]` - Use proper error handling

**Documentation Suppressors - FORBIDDEN:**
- `#[allow(missing_docs)]` - Document all public items
- `#![allow(...)]` - Never use crate-level suppressors

**Correct Approaches:**
- Dead code → Remove it or make it `pub` if part of API
- Unused variables → Use `let _name = value;`
- Clippy warnings → Fix the underlying issue
- Temporary issues → Fix immediately, don't suppress

**Example of what NOT to do:**
```rust
// ❌ WRONG - Never suppress warnings
#[allow(dead_code)]
fn unused_function() { }

// ✅ CORRECT - Remove unused code
// (function deleted)
```

### Commit Conventions

Follow [Conventional Commits](https://conventionalcommits.org/) specification:

```
<type>[optional scope]: <description>

[optional body]

[optional footer(s)]
```

**Required format for this project:**
```
<type>: <description>

🤖 Generated with [Claude Code](https://claude.ai/code)

Co-Authored-By: Claude <noreply@anthropic.com>
```

**Types:**
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting, etc.)
- `refactor`: Code refactoring without feature changes
- `test`: Adding or updating tests
- `chore`: Build process, dependencies, tooling

**Examples:**
```
feat: implement window focus management with visual indication
fix: address PR review feedback for gap system robustness  
docs: update README with installation instructions
```

### Automated Semantic Versioning

**This project uses automated semantic versioning with semantic-release!**

Follow [SemVer](https://semver.org/) (MAJOR.MINOR.PATCH):
- **MAJOR**: Breaking changes to public API (currently bumps MINOR due to pre-1.0)
- **MINOR**: New features (`feat:` commits)
- **PATCH**: Bug fixes (`fix:`, `style:`, `refactor:`, `test:`, `ci:` commits)
- **NO RELEASE**: Documentation and maintenance (`docs:`, `chore:` commits)

**Automated Version Management:**
- ✅ **Version Updates**: `Cargo.toml` version automatically updated on release
- ✅ **Changelog Generation**: `CHANGELOG.md` automatically updated with release notes
- ✅ **Git Tags**: Release tags (`v0.1.0`, `v0.2.0`, etc.) automatically created
- ✅ **GitHub Releases**: Automated releases with binaries and release notes
- ✅ **Commit Analysis**: Conventional commits analyzed to determine version bump

**How it works:**
1. Push commits to `main` branch with conventional commit messages
2. GitHub Actions runs semantic-release on every push to main
3. If releasable commits found: version bumped, changelog updated, release created
4. Binaries built and attached to GitHub release
5. `Cargo.toml` and `CHANGELOG.md` committed back to main with `[skip ci]`

**IMPORTANT: Never manually update versions or changelog - it's automated!**

### Branch Management

**Branching Strategy: GitHub Flow (Simplified)**
- ✅ **`main`** - Production branch with automated releases
- ✅ **`feature/*`** - Feature branches for all development
- ❌ **No `develop` branch** - Direct main workflow for simplicity

**Branch naming conventions:**
- `feature/feature-name` - New features
- `fix/bug-description` - Bug fixes
- `docs/documentation-update` - Documentation changes
- `refactor/component-name` - Code refactoring

**Workflow:**
1. Create feature branch from `main`: `git checkout -b feature/my-feature`
2. Implement changes with proper commit messages
3. Ensure all checks pass (fmt, clippy, test)
4. Push branch: `git push -u origin feature/my-feature`
5. Create PR with descriptive title and body
6. Address review feedback
7. Squash merge to `main` after approval
8. Automatic release triggered by semantic-release on main

## Development Commands

**Environment Setup:**
```bash
source ~/.cargo/env  # Ensure Rust toolchain is available
```

**Using the Simple Test Scripts:**
```bash
# Interactive testing with Xephyr
./test.sh

# Code quality checks (formatting, clippy, tests, docs)
./check.sh
```

**Manual Development Commands:**
```bash
cargo check          # Quick syntax check
cargo build          # Full build
cargo run            # Build and run
cargo test           # Run all tests
cargo fmt            # Format code (REQUIRED before commits)
cargo clippy --all-targets --all-features -- -D warnings  # Lint code (REQUIRED before commits)
cargo doc --open     # Generate and open documentation
```

**Testing Commands:**
```bash
cargo test                    # Run all tests
cargo test --lib            # Run library tests only
cargo test --bin            # Run binary tests only
cargo test test_name         # Run specific test
```

## Project Structure and Architecture

```
rustile/
├── src/
│   ├── main.rs              # Entry point and CLI
│   ├── lib.rs               # Library root
│   │
│   ├── window_manager/      # Core window management
│   │   ├── mod.rs           # Module interface & tests
│   │   ├── core.rs          # WindowManager struct & main loop
│   │   ├── events.rs        # X11 event handling
│   │   ├── focus.rs         # Focus management
│   │   └── window_ops.rs    # Window operations
│   │
│   ├── layout/              # BSP tiling algorithm
│   │   ├── mod.rs           # Layout module interface
│   │   ├── bsp.rs           # BSP tree algorithm & geometry calculation
│   │   ├── types.rs         # Data structures
│   │   └── constants.rs     # Layout constants
│   │
│   ├── config/              # Configuration system
│   │   ├── mod.rs           # Config structures
│   │   └── validation.rs    # Validation trait & validators
│   │
│   ├── keyboard.rs          # Keyboard shortcut handling
│   └── keys.rs              # Key parsing utilities
│
├── docs/
│   ├── BEGINNER_GUIDE.md    # Guide for first-time users
│   ├── TECHNICAL_DEEP_DIVE.md # Advanced implementation details (merged from ARCHITECTURE.md)
│   └── ROADMAP.md           # Development roadmap
│
├── test.sh                  # Simple interactive testing script
├── check.sh                 # Simple code quality checker  
├── config.example.toml      # Example configuration
├── CLAUDE.md                # This file
└── README.md                # User documentation
```

## Current Features

### Window Management
- **BSP Layout**: Binary Space Partitioning for flexible window arrangement
- **Focus Management**: Visual borders (red=focused, gray=unfocused)
- **Gap System**: Configurable spacing between windows and screen edges
- **Keyboard Navigation**: Alt+j/k (focus), Shift+Alt+m (swap with master)
- **Window Operations**: Automatic tiling on add/remove

### Configuration System
- **TOML Configuration**: `~/.config/rustile/config.toml`
- **Runtime Validation**: Input validation with helpful error messages
- **Flexible Shortcuts**: Support for complex key combinations
- **Layout Options**: bsp_split_ratio, minimum window sizes

### Testing Infrastructure
- **Unit Tests**: 49 tests covering all major components
- **Integration Testing**: Xephyr-based testing environment with 4 test applications
- **Edge Case Testing**: Gap calculations, minimum sizes, validation
- **Zero-Warning CI**: Strict quality enforcement

## Configuration Guidelines

### Validation Rules
- **Gap**: 0-500 pixels (recommended: 0-50)
- **Border Width**: 0-50 pixels (recommended: 1-10)
- **Combined Limits**: gap + border_width ≤ 600 pixels
- **Master Ratio**: 0.0-1.0 (recommended: 0.3-0.7)
- **BSP Split Ratio**: 0.0-1.0 (recommended: 0.5)
- **Minimum Window Sizes**: 100px master, 50px stack windows

### Recommended Configuration
```toml
[general]
default_display = ":10"  # For Xephyr testing

[layout]
bsp_split_ratio = 0.5   # Equal splits for BSP
gap = 10                # 10px comfortable spacing
border_width = 5        # 5px visible borders
min_window_width = 100  # Minimum window width
min_window_height = 50  # Minimum window height
focused_border_color = 0xFF0000    # Red
unfocused_border_color = 0x808080  # Gray

[shortcuts]
"Shift+Alt+1" = "xterm"
"Alt+j" = "focus_next"
"Alt+k" = "focus_prev"
"Shift+Alt+m" = "swap_with_master"
```

## Testing Strategy

### Test Environment Setup
```bash
# Use the simple test script (recommended)
./test.sh

# This opens 4 test applications:
# 1. xterm (running top)
# 2. xlogo (X11 logo)
# 3. xcalc (calculator)
# 4. xeyes (graphics demo)
```

### Test Categories
1. **Unit Tests**: Component logic validation (49 tests)
2. **Integration Tests**: Full window manager behavior
3. **Edge Case Tests**: Boundary conditions and error handling
4. **Configuration Tests**: Validation and parsing

### Required Test Coverage
- All public APIs must have unit tests
- Configuration validation must be thoroughly tested
- Layout calculations must handle edge cases
- Error conditions must be properly tested

## Code Quality Standards

### Error Handling
- Use `anyhow::Result<T>` for fallible operations
- Provide descriptive error messages with context
- Never use `panic!`, `unwrap()`, or `expect()` in production code
- Log errors appropriately with `tracing` crate

### Logging Standards
**Rustile uses a simplified 3-level logging approach with the tracing crate:**

**Import Style:**
```rust
use tracing::{info, error, debug};
```

**Log Levels:**
- `error!()` - Breaking functionality, critical failures that affect user experience
- `info!()` - User-visible events and important state changes (window operations, WM lifecycle)
- `debug!()` - Developer debugging information (wrapped in `#[cfg(debug_assertions)]`)

**Usage Guidelines:**
- **error!**: X11 connection failures, config load errors, window manager registration failures
- **info!**: Window mapping/unmapping, focus changes, shortcut execution, layout applications
- **debug!**: Event details, internal state changes, detailed flow information

**Examples:**
```rust
// Error level - critical failures
error!("Failed to become window manager: {:?}", e);
error!("X11 connection lost: {:?}", e);

// Info level - user-visible operations  
info!("Successfully became the window manager");
info!("Mapping window: {:?}", window);
info!("Shortcut pressed, executing: {}", command);

// Debug level - developer information
#[cfg(debug_assertions)]
debug!("Configure request for window: {:?}", event.window);
```

**Standards:**
- Use consistent import style across all modules
- Wrap debug! calls in `#[cfg(debug_assertions)]` for performance
- Include relevant context (window IDs, commands, error details)
- Keep messages concise but informative

### Documentation
- Document all public APIs with `///` comments
- Include examples in documentation where helpful
- Keep README.md updated with current features
- Educational docs: BEGINNER_GUIDE.md, TECHNICAL_DEEP_DIVE.md

## Dependencies Management

### Core Dependencies
- `x11rb`: X11 protocol bindings
- `anyhow`: Error handling
- `tracing`: Logging framework
- `serde`: Configuration serialization
- `toml`: Configuration format
- `dirs`: System directory detection

### Development Dependencies
- Standard Rust toolchain (rustc, cargo)
- `clippy`: Linting
- `rustfmt`: Code formatting
- `Xephyr`: Nested X server for testing

## Security Considerations

- Never log sensitive information
- Validate all user inputs (configuration, commands)
- Handle X11 protocol errors gracefully
- Avoid buffer overflows in layout calculations

## Performance Guidelines

- Minimize X11 roundtrips in event handling
- Cache expensive calculations where appropriate
- Profile layout algorithms for large window counts
- Use appropriate data structures for window tracking

## Automated Release Process

**This project uses fully automated releases - no manual intervention required!**

### How Releases Work
1. **Development**: Work on feature branches, create PRs to `main`
2. **Merge to Main**: Once PR is merged, GitHub Actions analyzes commits
3. **Automatic Release**: If releasable commits found, semantic-release:
   - Determines version bump based on commit types
   - Updates `Cargo.toml` version automatically
   - Generates `CHANGELOG.md` entries from commits
   - Creates git tag and GitHub release
   - Builds and uploads release binaries
   - Commits updated files back to main

### Manual Release (Emergency Only)
If automated release fails, manual steps:
1. Ensure all tests pass: `cargo test`
2. Ensure code is formatted: `cargo fmt`
3. Ensure no clippy warnings: `cargo clippy --all-targets --all-features -- -D warnings`
4. Push to main - automation will handle the rest

### Release Artifacts
Each release automatically includes:
- **Source Code**: Automatic GitHub tarball/zip
- **Linux x86_64 (glibc)**: `rustile-linux-x86_64` 
- **Linux x86_64 (musl)**: `rustile-linux-x86_64-musl`
- **Release Notes**: Auto-generated from conventional commits

### Version Bump Rules
- `feat:` commits → **MINOR** version bump (new features)
- `fix:`, `style:`, `refactor:`, `test:`, `ci:` → **PATCH** version bump
- `feat!:` or `fix!:` (breaking changes) → **MINOR** bump (pre-1.0, later MAJOR)  
- `docs:`, `chore:` commits → **No release** (documentation and maintenance)

## Troubleshooting

### Common Issues
- **Cargo not found**: Run `source ~/.cargo/env`
- **X11 connection failed**: Check DISPLAY variable
- **Permission denied**: Ensure user has X11 access
- **Another WM running**: Kill existing window manager first

### Debug Environment
```bash
RUST_LOG=debug cargo run  # Enable debug logging
RUST_BACKTRACE=1 cargo run  # Show backtraces on panic
```

### CI/Local Alignment Issues
If CI fails but local tests pass, ensure you're running the exact same commands as CI:

**Common Issue**: Running `cargo clippy -- -D warnings` locally but CI runs `cargo clippy --all-targets --all-features -- -D warnings`
- **Local**: Only checks main library code
- **CI**: Checks all targets including tests, benches, examples, and all feature combinations
- **Solution**: Always use the full command from the mandatory checklist above

**Example**: The `--all-targets` flag catches clippy issues in test code that may not appear in basic clippy runs.

### Release Automation Issues
- **Release not triggered**: Check conventional commit format in commit messages
- **Version not updated**: Ensure `cargo-edit` is installed in CI environment
- **Changelog not generated**: Check that `@semantic-release/changelog` plugin is installed
- **Binary build fails**: Check X11 dependencies and Rust toolchain setup
- **Permission denied**: Ensure `GITHUB_TOKEN` has `contents: write` permission

## Recent Architectural Changes

### Phase 1: Layout Module Refactoring
- Split 1039-line `layout.rs` into focused modules
- Created trait system for extensible layout algorithms
- Extracted constants to improve maintainability

### Phase 2: Window Manager Modularization  
- Split 643-line `window_manager.rs` into 5 focused modules
- Added comprehensive test coverage (11 new tests)
- Improved separation of concerns

### Phase 3: Configuration Improvements
- Created validation trait system
- Modularized config structure
- Added reusable validators

### Phase 4: Documentation Enhancement
- Added HOW_RUSTILE_WORKS.md explaining X11 concepts and architecture
- Created IMPLEMENTATION_DETAILS.md with technical details and code examples
- Focused on essential concepts with visual ASCII diagrams
- Removed outdated beginner and deep dive documentation

### Phase 5: Architecture Simplification (Latest)
- **Removed master-stack layout**: Simplified to BSP-only (~564 lines removed)
- **Eliminated LayoutManager abstraction**: Direct BSP tree usage (~176 lines removed)
- **Separated X11 from layout calculations**: Pure geometry calculation functions
- **Total reduction**: ~740 lines while maintaining all functionality

## Future Development Priorities

See [docs/ROADMAP.md](docs/ROADMAP.md) for detailed planning:

1. **Basic Window Features**: Destroy, switch, rotate, auto-balance
2. **BSP Enhancements**: Directional focus, targeted insertion
3. **Configuration**: Live reload support
4. **Floating Windows**: Toggle tiling/floating mode
5. **Refactoring**: Docker tests, simplified key management
6. **Multi-Monitor**: Window movement between screens