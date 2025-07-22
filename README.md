# Rustile

A lightweight tiling window manager written in Rust, designed to be simple, efficient, and extensible.

![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)
![X11](https://img.shields.io/badge/X11-Window%20Manager-orange)
![License](https://img.shields.io/github/license/d-matsui/rustile)

## Features

- **Automatic Tiling**: Windows are automatically arranged in a tiling layout
- **Master-Stack Layout**: Primary window on the left, additional windows stack on the right
- **Configurable Gaps**: Customizable spacing between windows and screen edges
- **Window Focus Management**: Visual focus indication with colored borders and keyboard navigation
- **Robust Configuration**: TOML-based config with validation and helpful error messages
- **Keyboard-Driven**: Control your windows without touching the mouse
- **Lightweight**: Minimal resource usage and fast performance
- **Extensible**: Modular architecture makes it easy to add new features
- **Modern Codebase**: Written in Rust with safety and performance in mind

## Installation

### Prerequisites

- Rust 1.70 or later
- X11 development libraries

#### On Debian/Ubuntu:
```bash
sudo apt-get install build-essential libx11-dev libxcb1-dev
```

### Building from Source

```bash
# Clone the repository
git clone https://github.com/d-matsui/rustile.git
cd rustile

# Build in release mode
cargo build --release

# The binary will be at target/release/rustile
```

### Installation

```bash
# Install to ~/.local/bin
mkdir -p ~/.local/bin
cp target/release/rustile ~/.local/bin/

# Or install system-wide
sudo cp target/release/rustile /usr/local/bin/
```

## Usage

### Starting Rustile

#### Option 1: Using a Display Manager

Create a desktop entry file at `/usr/share/xsessions/rustile.desktop`:

```ini
[Desktop Entry]
Name=Rustile
Comment=Tiling window manager written in Rust
Exec=rustile
Type=Application
```

Then select Rustile from your display manager's session menu.

#### Option 2: Using xinit/startx

Add to your `~/.xinitrc`:

```bash
exec rustile
```

Then run `startx` from the console.

#### Option 3: Testing with Xephyr

For testing without replacing your current window manager:

```bash
# Start a nested X server
Xephyr :10 -screen 1280x720

# In another terminal, run rustile
DISPLAY=:10 rustile

# Launch applications in the nested server
DISPLAY=:10 xterm
DISPLAY=:10 firefox
```

**Quick Testing**: Use the included test script:
```bash
./test_focus.sh
```

### Configuration

Rustile now uses a TOML configuration file for easy customization. Copy the example configuration to get started:

```bash
mkdir -p ~/.config/rustile
cp config.example.toml ~/.config/rustile/config.toml
```

Edit `~/.config/rustile/config.toml` to customize:
- Keyboard shortcuts with human-readable key combinations
- Master window ratio and gap spacing
- Window border appearance (width and colors)
- Default display for launching applications

#### Keyboard Shortcuts

Define shortcuts in the `[shortcuts]` section using this format:
```toml
"Modifier+Key" = "command"
```

Supported modifiers:
- **Primary**: `Super` (Win/Cmd), `Alt` (Meta), `Ctrl`, `Shift`
- **Less common**: `NumLock`, `ScrollLock`, `AltGr`
- **Special**: `Hyper` (Super+Alt+Ctrl+Shift)

Example shortcuts:
```toml
[shortcuts]
# Application shortcuts
"Shift+Alt+1" = "gnome-terminal"
"Shift+Alt+2" = "code"
"Shift+Alt+3" = "chrome"

# Window focus and navigation shortcuts  
"Alt+j" = "focus_next"        # Focus next window
"Alt+k" = "focus_prev"        # Focus previous window
"Shift+Alt+m" = "swap_with_master"  # Swap focused window with master
```

### Window Management

Rustile automatically manages your windows using a master-stack layout:

- The first window becomes the master window (left half of the screen)
- Additional windows are stacked vertically on the right half
- Windows are automatically resized when added or removed
- Closing a window triggers automatic re-tiling

#### Focus Management

- **Visual Indication**: Focused window has a red border, unfocused windows have gray borders
- **Keyboard Navigation**: Use `Alt+j/k` to cycle through windows
- **Window Swapping**: `Shift+Alt+m` swaps the focused window with master
- **Auto Focus**: New windows automatically receive focus

#### Layout Configuration

Configure the master-stack layout and visual appearance in your `config.toml`:

```toml
[layout]
# Master window takes 50% of screen width (recommended: 0.3-0.7)
master_ratio = 0.5

# Gap between windows and screen edges in pixels (recommended: 0-50)
gap = 10

# Window border settings
border_width = 5               # Border width in pixels (recommended: 1-10)
focused_border_color = 0xFF0000   # Red color for focused window
unfocused_border_color = 0x808080 # Gray color for unfocused windows
```

**Configuration Validation:**
- Gap: 0-500 pixels (max combined with border: 600px)
- Border width: 0-50 pixels
- Master ratio: 0.0-1.0
- Minimum window sizes enforced: 100px master, 50px stack windows

## Testing

### Automated Testing

Use the included test script for quick testing:

```bash
./test_focus.sh
```

This script will:
1. Build Rustile in release mode
2. Start Xephyr nested X server
3. Launch Rustile in the test environment
4. Open multiple test applications
5. Display testing instructions

### Manual Testing

```bash
# Build the project
cargo build --release

# Run tests
cargo test

# Test with Xephyr
Xephyr :10 -screen 1280x720 &
DISPLAY=:10 ./target/release/rustile &
DISPLAY=:10 xterm &
```

## Architecture

Rustile follows a modular architecture for maintainability and extensibility:

```
src/
├── main.rs           # Entry point
├── lib.rs            # Library root
├── config.rs         # Configuration loading and management
├── window_manager.rs # Core window manager logic
├── layout.rs         # Window layout algorithms
├── keyboard.rs       # Keyboard handling
└── keys.rs           # Key combination parser
```

### Key Components

- **WindowManager**: Main struct that manages X11 connection and window state
- **LayoutManager**: Handles window arrangement algorithms
- **KeyboardManager**: Manages keyboard shortcuts and key mappings
- **Focus System**: Tracks focused windows with visual indication

## Development

### Building and Testing

```bash
# Build the project
cargo build

# Run tests
cargo test

# Run with debug logging
RUST_LOG=debug cargo run

# Check code quality
cargo clippy
cargo fmt --check
```

### Adding New Features

#### Adding a New Layout

1. Add a new variant to the `Layout` enum in `src/layout.rs`
2. Implement the layout algorithm in `LayoutManager`
3. Add a keyboard shortcut to switch layouts

#### Adding Keyboard Shortcuts

1. Add the shortcut to your `~/.config/rustile/config.toml`:
   ```toml
   "Super+b" = "firefox"
   ```
2. Reload rustile to apply the new configuration

#### Adding Window Management Commands

1. Add the command handler in `WindowManager::handle_key_press()`
2. Implement the command method (e.g., `focus_next()`)
3. Update the example configuration

### Project Structure

- **Logging**: Uses the `tracing` crate for structured logging
- **Error Handling**: Uses `anyhow` for ergonomic error handling
- **X11 Communication**: Uses `x11rb` for safe X11 protocol implementation

## Releases and Versioning

Rustile uses **automated semantic versioning** with continuous releases. Every PR merged to main automatically creates a new release based on the commit messages.

### Semantic Versioning (v0.x.x Development Phase)

We follow [Semantic Versioning](https://semver.org/) with special rules for the development phase:

- **v0.x.y**: Development releases (current phase)
- **v1.0.0**: First stable release (future)

#### Version Bumping Rules

| Commit Type | Version Change | Example |
|-------------|---------------|---------|
| `feat:` | Minor bump | 0.1.0 → 0.2.0 |
| `fix:` | Patch bump | 0.1.0 → 0.1.1 |
| `perf:` | Patch bump | 0.1.0 → 0.1.2 |
| `docs:` | Patch bump | 0.1.0 → 0.1.3 |
| `feat!:` | Minor bump* | 0.1.0 → 0.2.0 |

*Breaking changes stay in v0.x during development phase

### Commit Message Format

Use conventional commits for automatic versioning:

```bash
# New features (minor version bump)
feat: add multi-monitor support
feat: implement floating windows

# Bug fixes (patch version bump)  
fix: resolve keyboard mapping issue
fix: prevent memory leak in layout manager

# Breaking changes (minor bump during v0.x)
feat!: change configuration file format
fix!: update keyboard shortcut API

# Other types (patch version bump)
perf: optimize window tiling algorithm
docs: update installation instructions
style: improve code formatting
refactor: restructure layout module
test: add integration tests
ci: update GitHub Actions workflow

# No release
chore: update dependencies
chore: clean up code comments
```

### Automatic Release Process

1. **Develop**: Create feature branch with conventional commits
2. **PR**: Create pull request to main branch
3. **Review**: Code review and CI checks pass
4. **Merge**: PR merged to main
5. **Auto-Release**: 
   - Analyzes commit messages since last release
   - Determines appropriate version bump
   - Updates `Cargo.toml` automatically
   - Builds multi-platform binaries
   - Creates GitHub release with changelog
   - Uploads binaries as release assets

### Manual Release Control

Skip automatic release by adding `[skip ci]` to commit message:

```bash
git commit -m "chore: update README [skip ci]"
```

### Download Releases

Pre-built binaries are available for every release:

- **Linux x86_64 (glibc)**: For most Linux distributions
- **Linux x86_64 (musl)**: For Alpine Linux and static linking

Download from: [GitHub Releases](https://github.com/d-matsui/rustile/releases)

### Development Roadmap

- **v0.1.x**: Core tiling functionality, configuration, focus management, and gap system ✅
- **v0.2.x**: Multi-monitor support and advanced window navigation
- **v0.3.x**: Floating windows and multiple layout algorithms
- **v0.4.x**: Workspace management and window rules
- **v0.5.x**: Status bar integration and themes
- **v1.0.0**: Stable API and production-ready release

## Troubleshooting

### "Another window manager is already running"

This error occurs when trying to run Rustile while another window manager is active. Make sure to:
- Log out of your current desktop session
- Stop any running window manager  
- Use Xephyr for testing alongside your current WM

### Configuration Issues

If you encounter configuration problems:
- Check the log output for validation errors: `RUST_LOG=info rustile`
- Verify your TOML syntax is correct
- Ensure all values are within the documented limits (see Configuration section)
- Reset to defaults by removing `~/.config/rustile/config.toml`

### Windows not tiling properly

- Check the debug logs: `RUST_LOG=debug rustile`
- Ensure the application supports standard X11 window management
- Some applications may set size hints that affect tiling

### Keyboard shortcuts not working

- Verify the modifier key is correctly mapped: `xmodmap -pm`
- Check if another application has grabbed the same key combination
- Review the debug logs for keyboard event information

### Focus indicators not visible

- Ensure windows support border modifications
- Check if the application overrides window decorations
- Try with simple applications like `xterm` first

## Roadmap

- [x] Configuration file support (TOML)
- [x] Window focus management with visual indication
- [x] Keyboard-driven window navigation
- [x] Configurable gaps between windows
- [x] Configurable window borders and colors
- [x] Robust input validation and error handling
- [x] Comprehensive test coverage
- [ ] Multiple layout algorithms (horizontal split, grid, fibonacci)
- [ ] Workspace/virtual desktop support
- [ ] More keyboard shortcuts (layout switching)
- [ ] Floating window support
- [ ] Multi-monitor support
- [ ] Status bar integration
- [ ] Window rules and application-specific behavior

## Contributing

Contributions are welcome! Please feel free to submit issues and pull requests.

### Development Guidelines

1. Follow Rust naming conventions and idioms
2. Add tests for new functionality
3. Update documentation as needed
4. Ensure `cargo clippy` and `cargo fmt` pass
5. Keep commits focused and descriptive
6. Use conventional commit messages for automatic versioning

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Inspired by tiling window managers like [xpywm](https://github.com/h-ohsaki/xpywm) and [yabai](https://github.com/koekeishiya/yabai)
- Thanks to the Rust community for excellent documentation and tools