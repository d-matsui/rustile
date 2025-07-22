# Rustile

A lightweight tiling window manager written in Rust, designed to be simple, efficient, and extensible.

![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)
![X11](https://img.shields.io/badge/X11-Window%20Manager-orange)
![License](https://img.shields.io/github/license/d-matsui/rustile)

## Features

- **Automatic Tiling**: Windows are automatically arranged in a tiling layout
- **Master-Stack Layout**: Primary window on the left, additional windows stack on the right
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

### Configuration

Rustile now uses a TOML configuration file for easy customization. Copy the example configuration to get started:

```bash
mkdir -p ~/.config/rustile
cp config.example.toml ~/.config/rustile/config.toml
```

Edit `~/.config/rustile/config.toml` to customize:
- Keyboard shortcuts with human-readable key combinations
- Master window ratio
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
"Shift+Alt+1" = "gnome-terminal"
"Shift+Alt+2" = "code"
"Shift+Alt+3" = "chrome"
"Super+Return" = "xterm"
"Ctrl+Alt+t" = "gnome-terminal"
```

### Window Management

Rustile automatically manages your windows using a master-stack layout:

- The first window becomes the master window (left half of the screen)
- Additional windows are stacked vertically on the right half
- Windows are automatically resized when added or removed
- Closing a window triggers automatic re-tiling

#### Layout Configuration

Configure the master-stack layout in your `config.toml`:

```toml
[layout]
master_ratio = 0.5  # Master window takes 50% of screen width
gap_size = 0        # Gap between windows in pixels (future feature)
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

### Project Structure

- **Logging**: Uses the `tracing` crate for structured logging
- **Error Handling**: Uses `anyhow` for ergonomic error handling
- **X11 Communication**: Uses `x11rb` for safe X11 protocol implementation

## Troubleshooting

### "Another window manager is already running"

This error occurs when trying to run Rustile while another window manager is active. Make sure to:
- Log out of your current desktop session
- Stop any running window manager
- Use Xephyr for testing alongside your current WM

### Windows not tiling properly

- Check the debug logs: `RUST_LOG=debug rustile`
- Ensure the application supports standard X11 window management
- Some applications may set size hints that affect tiling

### Keyboard shortcuts not working

- Verify the modifier key is correctly mapped: `xmodmap -pm`
- Check if another application has grabbed the same key combination
- Review the debug logs for keyboard event information

## Roadmap

- [x] Configuration file support (TOML)
- [ ] Multiple layout algorithms (horizontal split, grid, fibonacci)
- [ ] Workspace/virtual desktop support
- [ ] Window focus indication and borders
- [ ] More keyboard shortcuts (window navigation, layout switching)
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

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Inspired by tiling window managers like [xpywm](https://github.com/h-ohsaki/xpywm) and [yabai](https://github.com/koekeishiya/yabai)
- Thanks to the Rust community for excellent documentation and tools