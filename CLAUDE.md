# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a tiling window manager written in Rust using x11rb for X11 window management. The project implements a master-stack layout with configurable gaps, focus management, and keyboard shortcuts.

## Development Rules and Standards

### Code Formatting and Quality

**MANDATORY: Always run these commands before committing:**
```bash
source ~/.cargo/env  # Ensure cargo is in PATH
cargo fmt           # Format code
cargo clippy --all-targets --all-features -- -D warnings  # Check for lints (treat warnings as errors)
cargo test          # Run all tests
```

- **Formatting**: All code MUST be formatted with `cargo fmt` before commits
- **Linting**: All clippy warnings MUST be resolved (use `--all-targets --all-features -- -D warnings` flags to match CI)
  - **CRITICAL**: The `--all-targets --all-features` flags are required to catch issues in test code and all build configurations
  - **CI Alignment**: This exact command must pass locally before commits to prevent CI failures
- **Testing**: All tests MUST pass before commits
- **Documentation**: Use `///` for public APIs, `//!` for module-level docs
- **Error Handling**: Use `anyhow::Result` for error propagation, never use `unwrap()` in production code

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
- **PATCH**: Bug fixes (`fix:`, `docs:`, `style:`, `refactor:`, `test:`, `ci:` commits)

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

**Branch naming conventions:**
- `feature/feature-name` - New features
- `fix/bug-description` - Bug fixes
- `docs/documentation-update` - Documentation changes
- `refactor/component-name` - Code refactoring

**Workflow:**
1. Create feature branch from `main`
2. Implement changes with proper commit messages
3. Ensure all checks pass (fmt, clippy, test)
4. Create PR with descriptive title and body
5. Address review feedback
6. Squash merge to `main` after approval

## Development Commands

**Environment Setup:**
```bash
source ~/.cargo/env  # Ensure Rust toolchain is available
```

**Development Workflow:**
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
│   ├── window_manager.rs    # Core window management logic
│   ├── layout.rs            # Tiling layout algorithms
│   ├── config.rs            # Configuration system (TOML)
│   ├── keyboard.rs          # Keyboard shortcut handling
│   └── keys.rs              # Key parsing utilities
├── config.example.toml      # Example configuration
├── test_focus.sh            # Testing script with Xephyr
├── setup_config.sh          # Configuration setup script
└── CLAUDE.md               # This file
```

## Current Features

### Window Management
- **Master-Stack Tiling**: Configurable ratio, automatic window arrangement
- **Focus Management**: Visual borders (red=focused, gray=unfocused)
- **Gap System**: Configurable spacing between windows and screen edges
- **Keyboard Navigation**: Alt+j/k (focus), Shift+Alt+m (swap with master)

### Configuration System
- **TOML Configuration**: `~/.config/rustile/config.toml`
- **Runtime Validation**: Input validation with helpful error messages
- **Flexible Shortcuts**: Support for complex key combinations

### Testing Infrastructure
- **Unit Tests**: Comprehensive test coverage for all components
- **Integration Testing**: Xephyr-based testing environment
- **Edge Case Testing**: Gap calculations, minimum sizes, validation

## Configuration Guidelines

### Validation Rules
- **Gap**: 0-500 pixels (recommended: 0-50)
- **Border Width**: 0-50 pixels (recommended: 1-10)
- **Combined Limits**: gap + border_width ≤ 600 pixels
- **Master Ratio**: 0.0-1.0 (recommended: 0.3-0.7)
- **Minimum Window Sizes**: 100px master, 50px stack windows

### Recommended Ranges
```toml
[layout]
master_ratio = 0.5      # 50% screen width for master
gap = 10               # 10px comfortable spacing
border_width = 5       # 5px visible borders
```

## Testing Strategy

### Test Environment
```bash
# Run test script for manual testing
./test_focus.sh

# Alternative manual testing
Xephyr :10 -screen 1280x720 &
DISPLAY=:10 cargo run
```

### Test Categories
1. **Unit Tests**: Component logic validation
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

### Logging
- Use `tracing` crate for structured logging
- Log levels: `error!`, `warn!`, `info!`, `debug!`, `trace!`
- Include relevant context in log messages

### Documentation
- Document all public APIs with `///` comments
- Include examples in documentation where helpful
- Keep README.md updated with current features
- Maintain CHANGELOG.md for release notes

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
- `fix:`, `docs:`, `style:`, `refactor:`, `test:`, `ci:` → **PATCH** version bump
- `feat!:` or `fix!:` (breaking changes) → **MINOR** bump (pre-1.0, later MAJOR)
- `chore:` commits → **No release** (maintenance only)

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

## Future Development Priorities

1. **Multi-monitor Support**: Extend to multiple screens
2. **Workspace Management**: Virtual desktop functionality  
3. **Floating Windows**: Non-tiled window support
4. **Advanced Layouts**: Grid, spiral, custom layouts
5. **IPC Interface**: Runtime configuration changes
6. **Theme System**: Customizable colors and styling