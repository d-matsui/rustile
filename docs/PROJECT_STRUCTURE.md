# Project Structure

This document outlines the organization of the Rustile codebase for maintainers and contributors.

## Directory Layout

```
rustile/
├── src/                    # Core source code
│   ├── main.rs            # Application entry point
│   ├── lib.rs             # Library interface
│   ├── window_manager.rs  # Core window management
│   ├── layout.rs          # Tiling layout algorithms
│   ├── config.rs          # Configuration system
│   ├── keyboard.rs        # Keyboard shortcut handling  
│   └── keys.rs            # Key parsing utilities
├── scripts/               # Development utilities
│   └── dev-tools.sh       # Consolidated development script
├── docs/                  # Project documentation
│   └── PROJECT_STRUCTURE.md  # This file
├── .github/               # GitHub workflows and templates
│   └── workflows/         # CI/CD pipelines
├── config.example.toml    # Example configuration
├── setup_config.sh        # Configuration setup script
├── switch_layout.sh       # Layout switching utility
├── test_layout.sh         # Interactive testing script
├── CLAUDE.md              # Claude Code development guide
├── README.md              # User documentation
└── CHANGELOG.md           # Release history
```

## Code Organization

### Core Modules (`src/`)

#### `main.rs` - Application Entry Point
- Initializes logging and error handling
- Sets up X11 connection
- Starts the window manager event loop

#### `window_manager.rs` - Core Management Logic
- **Primary Responsibility**: X11 event handling and window lifecycle
- **Key Components**: Event loop, window focus, keyboard shortcuts
- **Dependencies**: Config, Layout, Keyboard managers

#### `layout.rs` - Tiling Algorithms
- **Master-Stack Layout**: Traditional tiling with master + stack
- **BSP Layout**: Binary space partitioning (yabai-inspired)
- **Extensible**: Easy to add new layout algorithms

#### `config.rs` - Configuration System
- **TOML-based**: User-friendly configuration format
- **Validation**: Comprehensive input validation with helpful errors
- **Runtime Loading**: Hot-reload support for development

#### `keyboard.rs` - Shortcut Management
- **X11 Integration**: Grabs global keyboard shortcuts
- **Flexible Mapping**: Support for complex key combinations
- **Command Dispatch**: Routes shortcuts to window manager actions

#### `keys.rs` - Key Parsing Utilities
- **String Parsing**: Convert "Ctrl+Alt+T" to X11 key codes
- **Modifier Support**: All standard and extended modifiers
- **Error Handling**: Clear feedback for invalid key combinations

### Development Tools (`scripts/`)

#### `dev-tools.sh` - Consolidated Development Script
**Unified interface for all development tasks:**

```bash
./scripts/dev-tools.sh setup     # Development environment setup
./scripts/dev-tools.sh test      # Run comprehensive tests
./scripts/dev-tools.sh layout    # Interactive layout testing
./scripts/dev-tools.sh check     # Quality checks (fmt, clippy, test)
./scripts/dev-tools.sh clean     # Clean build artifacts
./scripts/dev-tools.sh release   # Build release binary
```

### Legacy Scripts (Root Directory)
These remain for backward compatibility but are superseded by `dev-tools.sh`:

- `setup_config.sh` - Use `dev-tools.sh setup`
- `test_layout.sh` - Use `dev-tools.sh layout`  
- `switch_layout.sh` - Standalone utility (kept)

## Architecture Patterns

### Dependency Flow
```
main.rs
  └── WindowManager
      ├── Config (configuration loading)
      ├── LayoutManager (window arrangement)
      └── KeyboardManager (shortcut handling)
          └── Keys (key parsing utilities)
```

### Event Flow
```
X11 Events → WindowManager → [Layout|Keyboard|Config] → X11 Commands
```

### Error Handling
- **Consistent**: All functions return `anyhow::Result<T>`
- **Contextual**: Errors include helpful context about what failed
- **Graceful**: Non-fatal errors are logged, fatal errors exit cleanly

## Development Workflow

### Adding New Features
1. **Plan**: Update this structure doc if architecture changes
2. **Implement**: Follow existing patterns and error handling
3. **Test**: Add unit tests and integration tests
4. **Document**: Update relevant docs and CLAUDE.md
5. **Validate**: Run `./scripts/dev-tools.sh check`

### Modifying Layouts
1. **Extend**: Add new variants to `Layout` enum in `layout.rs`
2. **Implement**: Add algorithm in `LayoutManager`
3. **Configure**: Add config options if needed
4. **Test**: Use `./scripts/dev-tools.sh layout` for interactive testing

### Configuration Changes
1. **Schema**: Update structs in `config.rs`
2. **Validation**: Add validation rules
3. **Defaults**: Update default values and example config
4. **Migration**: Consider backward compatibility

## Build Artifacts

### Development
- `target/debug/rustile` - Debug build with logging
- `target/debug/deps/` - Test and dependency artifacts

### Production  
- `target/release/rustile` - Optimized binary
- Generated via `cargo build --release` or `dev-tools.sh release`

## Documentation Strategy

### User-Facing
- **README.md**: Installation, usage, basic configuration
- **config.example.toml**: Comprehensive configuration guide

### Developer-Facing  
- **CLAUDE.md**: Development standards and CI requirements
- **PROJECT_STRUCTURE.md**: This architecture guide
- **Code Comments**: Inline documentation for complex logic

### Generated
- **CHANGELOG.md**: Automated release notes
- **Cargo docs**: `cargo doc` for API documentation

## Quality Standards

### Code Quality
- **Formatting**: `cargo fmt` (enforced in CI)
- **Linting**: `cargo clippy -- -D warnings` (zero warnings policy)  
- **Testing**: Comprehensive unit and integration tests
- **Documentation**: Public APIs documented with examples

### Performance
- **Debug Logging**: Conditional compilation for production builds
- **Memory**: No unnecessary allocations in hot paths
- **X11 Efficiency**: Minimal protocol roundtrips

### Security
- **Input Validation**: All user inputs validated
- **No Unwrap**: Never use `.unwrap()` in production code
- **Error Context**: Always provide meaningful error messages

This structure supports the project's goals of simplicity, performance, and maintainability while providing clear extension points for future development.