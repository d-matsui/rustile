# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with this tiling window manager codebase.

## Project Overview

Rustile is a tiling window manager written in Rust using x11rb for X11 window management. It implements BSP (Binary Space Partitioning) layout with configurable gaps, focus management, and keyboard shortcuts.

## Quick Reference

### Mandatory Pre-Commit Commands
```bash
source ~/.cargo/env  # Ensure cargo is in PATH
cargo fmt           # Format code
cargo build --all-targets --all-features  # Build all targets to catch warnings
cargo clippy --all-targets --all-features -- -D warnings  # Check for lints (treat warnings as errors)
cargo test          # Run all tests
```

### Development Scripts
```bash
./test.sh    # Interactive testing with Xephyr
./check.sh   # Code quality checks (formatting, clippy, tests, docs)
```

### Rust Version Management
- **Current Version**: Pinned to Rust 1.89 for CI reproducibility and 2024 edition support
- **Update Schedule**: Review quarterly or as needed for new features
- **Rationale**: Prevents unexpected CI failures from Rust version changes while maintaining modern toolchain

## Development Standards

### Code Quality Requirements
- **Zero Warnings**: Builds MUST produce no warnings
- **Formatting**: Use `cargo fmt` before commits
- **Linting**: All clippy warnings MUST be resolved with `--all-targets --all-features -- -D warnings` flags
- **Testing**: All tests MUST pass before commits
- **Error Handling**: Use `anyhow::Result`, never `unwrap()` in production
- **Documentation**: Use `///` for public APIs, `//!` for module-level docs
- **Code Comments**: Follow ADR-005 concise standard - document "what", not "how"

### Forbidden Rust Attributes
**NEVER use warning suppression attributes:**
- `#[allow(dead_code)]` - Remove unused code instead
- `#[allow(unused_variables)]` - Use `_` prefix for intentionally unused vars
- `#[allow(clippy::all)]` - Fix clippy warnings instead
- `#[allow(missing_docs)]` - Document all public items

### Logging Standards
Use simplified 3-level approach with tracing crate:
```rust
use tracing::{info, error, debug};

error!("Failed to become window manager: {:?}", e);  // Critical failures
info!("Mapping window: {:?}", window);               // User-visible operations
#[cfg(debug_assertions)]
debug!("Configure request for window: {:?}", event.window);  // Developer info
```

## Project Structure

```
rustile/
├── src/
│   ├── main.rs              # Entry point and CLI
│   ├── window_manager.rs    # Core window management logic
│   ├── window_renderer.rs   # Window rendering and visual state
│   ├── window_state.rs      # Window state management
│   ├── bsp.rs               # BSP layout algorithm
│   ├── config.rs            # Configuration system with validation
│   └── keyboard.rs          # Keyboard shortcut handling
├── docs/                    # Documentation and ADRs
│   ├── HOW_RUSTILE_WORKS.md # X11 concepts and architecture
│   ├── ROADMAP.md           # Development roadmap
│   └── adr/                 # Architecture Decision Records
├── test.sh                  # Interactive testing script
├── check.sh                 # Code quality checker
└── config.example.toml      # Example configuration
```

## Git Workflow

### Commit Format
Follow [Conventional Commits](https://conventionalcommits.org/):
```
<type>: <description>

🤖 Generated with [Claude Code](https://claude.ai/code)

Co-Authored-By: Claude <noreply@anthropic.com>
```

**Types:** `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore`

### Branch Strategy
- `main` - Production branch with automated releases
- `feature/*` - Feature branches for development
- `fix/*` - Bug fixes
- `docs/*` - Documentation updates
- `refactor/*` - Code refactoring

### Automated Releases
**Fully automated - no manual intervention required:**
- `feat:` commits → MINOR version bump
- `fix:`, `style:`, `refactor:`, `test:` → PATCH version bump
- Automatic `Cargo.toml` version updates, changelog generation, GitHub releases

## Testing

### Test Environment
```bash
./test.sh  # Opens 4 test applications: xterm, xlogo, xcalc, xeyes
```

### Test Categories
- **Unit Tests**: 49 tests covering all major components
- **Integration Tests**: Full window manager behavior in Xephyr
- **Edge Case Tests**: Boundary conditions and error handling
- **Configuration Tests**: Validation and parsing

## Dependencies

### Core Dependencies
- `x11rb` - X11 protocol bindings
- `anyhow` - Error handling
- `tracing` - Logging framework
- `serde` + `toml` - Configuration
- `dirs` - System directory detection

### Development Tools
- Standard Rust toolchain (rustc, cargo, clippy, rustfmt)
- `Xephyr` - Nested X server for testing

## Troubleshooting

### Common Issues
- **Cargo not found**: Run `source ~/.cargo/env`
- **X11 connection failed**: Check DISPLAY variable
- **CI/Local mismatch**: Use `--all-targets --all-features` flags for clippy

### Debug Commands
```bash
RUST_LOG=debug cargo run     # Enable debug logging
RUST_BACKTRACE=1 cargo run   # Show backtraces on panic
```

### Release Issues
- **No release triggered**: Check conventional commit format
- **Build fails**: Verify X11 dependencies and Rust toolchain
- **Permission denied**: Check `GITHUB_TOKEN` has `contents: write`

## Architecture History

### Recent Architectural Evolution
- **Phase 6 (Latest)**: Flattened module structure for simplicity (7 focused files in src/)
- **Phase 5**: Removed master-stack layout, eliminated LayoutManager abstraction (~740 lines removed)
- **Phase 4**: Enhanced documentation with HOW_RUSTILE_WORKS.md
- **Phase 3**: Configuration validation system improvements
- **Phase 2**: Window manager modularization experiment (later simplified)
- **Phase 1**: Layout module refactoring (trait system)

### Architecture Decision Records (ADRs)
See [docs/adr/](docs/adr/) for detailed decisions:
- **ADR-012**: Configuration file handling improvement
- **ADR-011**: BSP screen rect separation
- **ADR-010**: Zoom to parent feature
- **ADR-009**: Unify keyboard modules
- **ADR-008**: X11 modifier system understanding
- **ADR-007**: X11 keyboard mapping understanding
- **ADR-006**: Configure request timeout handling
- **ADR-005**: Code comment standard implementation
- **ADR-004**: X11 event registration strategy
- **ADR-003**: SRP refactoring and three-module architecture
- **ADR-002**: Single source of truth architecture
- **ADR-001**: Rotate window implementation approach

### Current Features
- BSP layout with configurable gaps and borders
- Visual focus management (red=focused, gray=unfocused)
- Keyboard navigation and window operations
- TOML configuration with runtime validation
- Comprehensive test coverage
- Zoom-to-parent functionality for focused windows
- Production-ready TTY setup alongside desktop environments

## Future Development

See [docs/ROADMAP.md](docs/ROADMAP.md) for v1.0.0 and beyond:
- **v1.0.0**: Config file handling improvement (no auto-generation)
- **Future**: Directional insertion, floating windows, workspaces, multi-monitor support