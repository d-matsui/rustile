# Rustile

An X11 tiling window manager written in Rust, inspired by [yabai](https://github.com/koekeishiya/yabai) and [xpywm](https://github.com/h-ohsaki/xpywm).

![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)
![X11](https://img.shields.io/badge/X11-Window%20Manager-orange)
![License](https://img.shields.io/github/license/d-matsui/rustile)

![Example](<Screenshot from 2025-07-23 15-14-27.png>)

## Key Features

- **Automatic Window Tiling** - No manual window arrangement needed
- **Keyboard-Driven Workflow** - Efficient control without mouse dependency
- **Easy TOML Configuration** - Simple, readable config files
- **Lightweight Performance** - Minimal resource usage, fast startup
- **Extensible & Customizable** - Adapt to your workflow needs

## Installation

### Build from Source or Download from [GitHub Releases](https://github.com/d-matsui/rustile/releases)

**Build from source:**
```bash
# Install build dependencies (Debian/Ubuntu)
sudo apt-get install build-essential libx11-dev libxcb1-dev

# Clone and build
git clone https://github.com/d-matsui/rustile.git
cd rustile
cargo build --release
```

### Install the Binary
```bash
# Copy to system PATH
sudo cp target/release/rustile /usr/local/bin/  # or from downloaded binary
chmod +x /usr/local/bin/rustile
```

## Setup

### Stop Existing Window Manager
**On Debian/Ubuntu with GNOME:**
```bash
# Check current session type
echo $XDG_SESSION_TYPE  # "x11" = OK, "wayland" = Not compatible

# If running Wayland:
# - Log out and choose "GNOME on Xorg" at login screen
# - XWayland won't work - rustile needs real X11

# For X11 sessions, stop GNOME temporarily for testing:
sudo systemctl stop gdm3  # Warning: This stops your entire desktop!
```

### Make Rustile Your Default Window Manager

**Option A: Choose at Login (Recommended)**
Create `/usr/share/xsessions/rustile.desktop`:
```ini
[Desktop Entry]
Name=Rustile
Comment=Tiling window manager written in Rust
Exec=rustile
Type=Application
```
Then log out and select "Rustile" from the login screen.

**Option B: Using startx**
Add to `~/.xinitrc`:
```bash
exec rustile
```
Then use `startx` to launch.

## Basic Usage

```bash
# Window Navigation
Alt+j/k              Focus next/previous window
Shift+Alt+j/k        Swap window positions
Shift+Alt+m          Swap with master window

# Window Control  
Shift+Alt+q          Close window
Shift+Alt+1          Open terminal
```

## Configuration (Optional)

Rustile works out of the box, but you can customize it:

**Create config file:**
```bash
mkdir -p ~/.config/rustile
cp config.example.toml ~/.config/rustile/config.toml
```

**Example customizations:**
```toml
[layout]
gap = 15                           # Spacing between windows
border_width = 3                   # Window border thickness
focused_border_color = 0x00FF00    # Green borders for focused window

[shortcuts]
"Super+Return" = "xterm"           # Terminal with Super+Enter
"Super+d" = "rofi -show run"       # Application launcher
```

See [config.example.toml](config.example.toml) for all available options.

## Troubleshooting

### Configuration Issues

```bash
# Check for errors (logs to stderr):
RUST_LOG=debug rustile 2>&1 | tee rustile.log

# Reset to defaults:
rm ~/.config/rustile/config.toml
```

## Documentation

- **[How Rustile Works](docs/HOW_RUSTILE_WORKS.md)** - X11 concepts, architecture, and event flow
- **[Implementation Details](docs/IMPLEMENTATION_DETAILS.md)** - Technical details with code examples
- **[Roadmap](docs/ROADMAP.md)** - Planned features and development timeline

## License

MIT License - see [LICENSE](LICENSE) file for details.