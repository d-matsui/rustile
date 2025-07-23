# Rustile

A lightweight tiling window manager written in Rust, designed to be simple, efficient, and extensible.

![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)
![X11](https://img.shields.io/badge/X11-Window%20Manager-orange)
![License](https://img.shields.io/github/license/d-matsui/rustile)

## Features

- **Automatic Tiling**: Windows are automatically arranged without overlapping
- **Multiple Layouts**: Master-Stack and BSP (Binary Space Partitioning) layouts
- **Configurable Gaps**: Customizable spacing between windows and screen edges
- **Visual Focus Management**: Colored borders and keyboard navigation
- **TOML Configuration**: Easy-to-edit configuration with validation
- **Keyboard-Driven**: Control windows without touching the mouse
- **Lightweight**: Minimal resource usage and fast performance

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
git clone https://github.com/d-matsui/rustile.git
cd rustile
cargo build --release
cp target/release/rustile ~/.local/bin/  # or /usr/local/bin/
```

### Pre-built Binaries

Download from [GitHub Releases](https://github.com/d-matsui/rustile/releases)

## Quick Start

### 1. Setup Configuration
```bash
mkdir -p ~/.config/rustile
cp config.example.toml ~/.config/rustile/config.toml
```

### 2. Test with Xephyr (Recommended)
```bash
# Use the development script
./scripts/dev-tools.sh layout
```

Or manually:
```bash
Xephyr :10 -screen 1280x720 &
DISPLAY=:10 rustile &
DISPLAY=:10 xterm &  # Open test applications
```

### 3. Start Rustile

**Option A: Display Manager**
Create `/usr/share/xsessions/rustile.desktop`:
```ini
[Desktop Entry]
Name=Rustile
Comment=Tiling window manager written in Rust
Exec=rustile
Type=Application
```

**Option B: Using xinit**
Add to `~/.xinitrc`:
```bash
exec rustile
```

## Configuration

Edit `~/.config/rustile/config.toml`:

```toml
[layout]
layout_algorithm = "master_stack"  # or "bsp"
master_ratio = 0.5                 # Master window width ratio
gap = 10                           # Pixels between windows
border_width = 5                   # Window border thickness
focused_border_color = 0xFF0000    # Red for focused window
unfocused_border_color = 0x808080  # Gray for unfocused

[shortcuts]
# Application shortcuts
"Shift+Alt+1" = "gnome-terminal"
"Super+Return" = "xterm"

# Window management
"Alt+j" = "focus_next"             # Focus next window
"Alt+k" = "focus_prev"             # Focus previous window  
"Shift+Alt+m" = "swap_with_master" # Swap with master window
```

## Basic Usage

### Window Management
- **First window** becomes the master (left side in master-stack)
- **Additional windows** stack on the right or split recursively (BSP)
- **Closing a window** triggers automatic re-tiling

### Keyboard Navigation
- `Alt+j/k` - Cycle through windows
- `Shift+Alt+m` - Swap focused window with master
- Custom shortcuts for launching applications

### Layout Switching
```bash
# Switch between layouts
./scripts/dev-tools.sh switch bsp         # Switch to BSP
./scripts/dev-tools.sh switch master      # Switch to Master-Stack  
./scripts/dev-tools.sh switch             # Toggle between layouts
```

## Troubleshooting

### "Another window manager is already running"
- Log out of current desktop session
- Stop any running window manager
- Use Xephyr for testing: `./scripts/dev-tools.sh layout`

### Configuration Issues
- Check syntax: `RUST_LOG=info rustile`
- Reset to defaults: `rm ~/.config/rustile/config.toml`

### Debug Mode
```bash
RUST_LOG=debug rustile
```

## Documentation

- **[Architecture Guide](docs/ARCHITECTURE.md)** - Technical details and code structure
- **[Development Roadmap](docs/ROADMAP.md)** - Planned features and timeline
- **[Development Guide](CLAUDE.md)** - Contributing and development workflow

## Contributing

Contributions welcome! Please:
1. Use conventional commit messages (`feat:`, `fix:`, etc.)
2. Run `cargo fmt` and `cargo clippy` before committing
3. Add tests for new functionality
4. Check existing issues and PRs

## License

MIT License - see [LICENSE](LICENSE) file for details.

## Acknowledgments

Inspired by [yabai](https://github.com/koekeishiya/yabai) and other tiling window managers.