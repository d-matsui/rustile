# Rustile Code Explanation

This document explains how every part of the Rustile window manager works, from the entry point to the window tiling algorithms.

## Table of Contents

1. [Project Overview](#project-overview)
2. [Main Entry Point (main.rs)](#main-entry-point-mainrs)
3. [Configuration (config.rs)](#configuration-configrs)
4. [Window Manager Core (window_manager.rs)](#window-manager-core-window_managerrs)
5. [Layout System (layout.rs)](#layout-system-layoutrs)
6. [Keyboard Management (keyboard.rs)](#keyboard-management-keyboardrs)
7. [Key Parser (keys.rs)](#key-parser-keysrs)
8. [Library Structure (lib.rs)](#library-structure-librs)
9. [How Components Interact](#how-components-interact)
9. [Event Flow](#event-flow)
10. [Testing](#testing)

---

## Project Overview

Rustile is a tiling window manager written in Rust that automatically arranges windows without overlapping. It uses the X11 protocol to communicate with the display server and manage windows.

**Key Concepts:**
- **Window Manager**: A program that controls how windows are displayed
- **Tiling**: Automatically arranging windows to fill the screen without overlap
- **X11**: The display server protocol used on Linux systems
- **Event-driven**: The program responds to events (window opens, key presses, etc.)

**Architecture:**
```
┌─────────────┐    ┌──────────────┐    ┌─────────────┐
│   main.rs   │───▶│WindowManager │───▶│   X11       │
└─────────────┘    └──────┬───────┘    │   Server    │
                          │            └─────────────┘
                          ▼
              ┌─────────────────────────┐
              │     Components:         │
              │ ┌─────────────────────┐ │
              │ │   LayoutManager     │ │
              │ │   (window tiling)   │ │
              │ └─────────────────────┘ │
              │ ┌─────────────────────┐ │
              │ │  KeyboardManager    │ │
              │ │  (shortcuts)        │ │
              │ └─────────────────────┘ │
              └─────────────────────────┘
```

---

## Main Entry Point (main.rs)

```rust
//! Entry point for the window manager. Initializes logging and starts the window manager.

use anyhow::Result;
use rustile::window_manager::WindowManager;
use tracing::info;

fn main() -> Result<()> {
    // Initialize logging system to see debug messages
    tracing_subscriber::fmt::init();
    
    info!("Starting Rustile window manager");
    
    // Connect to X11 server (display server)
    // Returns connection and screen number
    let (conn, screen_num) = x11rb::connect(None)?;
    info!("Connected to X11 display on screen {}", screen_num);
    
    // Create and run window manager
    let wm = WindowManager::new(conn, screen_num)?;
    wm.run()
}
```

**What happens here:**

1. **Logging Setup**: `tracing_subscriber::fmt::init()` sets up logging so you can see what the window manager is doing
2. **X11 Connection**: `x11rb::connect(None)` connects to the X11 display server
   - `conn`: The connection object for sending commands to X11
   - `screen_num`: Which monitor/screen to use (usually 0 for primary display)
3. **Window Manager Creation**: `WindowManager::new()` creates our window manager instance
4. **Event Loop**: `wm.run()` starts the infinite loop that handles events

**Error Handling**: The `Result<()>` return type means the function can fail with an error, and the `?` operator propagates any errors up the call stack.

---

## Configuration (config.rs)

Rustile now uses a dynamic configuration system that loads settings from TOML files.

```rust
//! Configuration loading and management for the window manager

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    pub shortcuts: HashMap<String, String>,
    pub layout: LayoutConfig,
    pub general: GeneralConfig,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LayoutConfig {
    pub master_ratio: f32,
    pub gap_size: u32,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GeneralConfig {
    pub default_display: String,
}
```

**Configuration System:**

1. **Loading Order**:
   - First tries: `~/.config/rustile/config.toml`
   - Falls back to default values if not found

2. **Configuration Structure**:
   - **shortcuts**: Maps key combinations to commands
     - Example: `"Super+t" = "xterm"`
   - **layout**: Window layout settings
     - `master_ratio`: 0.0-1.0 (default 0.5)
     - `gap_size`: Pixels between windows (future feature)
   - **general**: General settings
     - `default_display`: X11 display for launching apps

3. **Example Config File**:
```toml
[general]
default_display = ":1"

[layout]
master_ratio = 0.5
gap_size = 0

[shortcuts]
"Shift+Alt+1" = "gnome-terminal"
"Shift+Alt+2" = "code"
"Super+Return" = "xterm"
```

**Benefits of TOML Configuration:**
- User-friendly format
- No recompilation needed for changes
- Supports complex key combinations
- Easy to share configurations

---

## Window Manager Core (window_manager.rs)

This is the heart of the window manager that coordinates everything.

### Data Structure

```rust
/// Main window manager structure
pub struct WindowManager<C: Connection> {
    /// X11 connection
    conn: C,
    /// Screen information
    screen_num: usize,
    /// Currently managed windows
    windows: Vec<Window>,
    /// Layout manager for window arrangement
    layout_manager: LayoutManager,
    /// Keyboard manager for shortcuts
    keyboard_manager: KeyboardManager,
    /// Configuration
    config: Config,
}
```

**Fields Explained:**
- `conn`: The connection to X11 server for sending commands
- `screen_num`: Which monitor we're managing
- `windows`: List of all windows we're currently managing
- `layout_manager`: Handles the positioning and sizing of windows
- `keyboard_manager`: Handles keyboard shortcuts

### Initialization (`new()`)

```rust
pub fn new(conn: C, screen_num: usize) -> Result<Self> {
    // Load configuration
    let config = Config::load()?;
    info!("Loaded configuration with {} shortcuts", config.shortcuts().len());

    let setup = conn.setup();
    let screen = &setup.roots[screen_num];
    let root = screen.root;

    // Initialize keyboard manager
    let mut keyboard_manager = KeyboardManager::new(&conn, setup)?;

    // Register as window manager
    let event_mask = EventMask::SUBSTRUCTURE_REDIRECT | EventMask::SUBSTRUCTURE_NOTIFY;
    let attributes = ChangeWindowAttributesAux::new().event_mask(event_mask);
    
    if let Err(e) = conn.change_window_attributes(root, &attributes)?.check() {
        error!("Another window manager is already running: {:?}", e);
        return Err(anyhow::anyhow!("Failed to become window manager. Is another WM running?"));
    }
    
    info!("Successfully became the window manager");

    // Register keyboard shortcuts from config
    keyboard_manager.register_shortcuts(&conn, root, config.shortcuts())?;

    Ok(Self {
        conn,
        screen_num,
        windows: Vec::new(),
        layout_manager: LayoutManager::new(),
        keyboard_manager,
        config,
    })
}
```

**Initialization Steps:**

1. **Get Screen Info**: Extract information about the monitor from X11
2. **Create Keyboard Manager**: Set up keyboard handling
3. **Register as Window Manager**: Tell X11 that we want to control window placement
   - `SUBSTRUCTURE_REDIRECT`: We control where windows go
   - `SUBSTRUCTURE_NOTIFY`: We get notified when windows are created/destroyed
4. **Error Check**: If another window manager is running, this will fail
5. **Register Shortcuts**: Tell X11 to send us Super+T key events
6. **Create Instance**: Initialize all the components

### Main Event Loop (`run()`)

```rust
pub fn run(mut self) -> Result<()> {
    info!("Starting window manager event loop");
    
    loop {
        self.conn.flush()?;
        let event = self.conn.wait_for_event()?;
        
        if let Err(e) = self.handle_event(event) {
            error!("Error handling event: {:?}", e);
        }
    }
}
```

**Event Loop Steps:**
1. **Flush**: Send any pending commands to X11
2. **Wait**: Block until we receive an event from X11
3. **Handle**: Process the event (delegate to specific handlers)
4. **Error Handling**: Log errors but don't crash
5. **Repeat**: Go back to step 1

### Event Handlers

#### Key Press Handler
```rust
fn handle_key_press(&mut self, event: KeyPressEvent) -> Result<()> {
    if let Some(command) = self.keyboard_manager.handle_key_press(&event) {
        info!("Shortcut pressed, executing: {}", command);
        
        // Parse command (simple implementation, could be improved)
        let parts: Vec<&str> = command.split_whitespace().collect();
        if let Some(program) = parts.first() {
            let mut cmd = Command::new(program);
            
            // Add arguments if any
            if parts.len() > 1 {
                cmd.args(&parts[1..]);
            }
            
            // Set display environment
            cmd.env("DISPLAY", self.config.default_display());
            
            match cmd.spawn() {
                Ok(_) => info!("Successfully launched: {}", command),
                Err(e) => error!("Failed to launch {}: {}", command, e),
            }
        }
    }
    Ok(())
}
```

**What happens:**
1. Check if Super key is held down AND T key is pressed
2. If yes, launch xcalc calculator application
3. Set the DISPLAY environment variable so it appears on the right screen

#### Map Request Handler (New Window)
```rust
fn handle_map_request(&mut self, event: MapRequestEvent) -> Result<()> {
    let window = event.window;
    info!("Mapping window: {:?}", window);
    
    // Map the window (make it visible)
    self.conn.map_window(window)?;
    
    // Add to managed windows
    self.windows.push(window);
    
    // Apply layout with configured master ratio
    self.apply_layout()?;
    
    Ok(())
}
```

**What happens:**
1. A new window wants to appear
2. Tell X11 to make it visible
3. Add it to our list of managed windows
4. Rearrange all windows using the layout algorithm

#### Unmap Notify Handler (Window Closed)
```rust
fn handle_unmap_notify(&mut self, event: UnmapNotifyEvent) -> Result<()> {
    let window = event.window;
    info!("Unmapping window: {:?}", window);
    
    // Remove from managed windows
    self.windows.retain(|&w| w != window);
    
    // Reapply layout
    self.apply_layout()?;
    
    Ok(())
}
```

**What happens:**
1. A window has been closed
2. Remove it from our list
3. Rearrange remaining windows to fill the space

---

## Layout System (layout.rs)

The layout system determines where windows are positioned and how big they are.

### Layout Types

```rust
/// Represents different tiling layouts
#[derive(Debug, Clone, Copy)]
pub enum Layout {
    /// Master-stack layout: one master window on the left, stack on the right
    MasterStack,
}

/// Window layout manager
pub struct LayoutManager {
    current_layout: Layout,
}
```

Currently, we only have one layout (MasterStack), but this design makes it easy to add more layouts like:
- Horizontal split
- Grid layout
- Fibonacci spiral
- Floating windows

### Master-Stack Algorithm

This is the core algorithm that positions windows:

```rust
fn tile_master_stack(&self, conn: &impl Connection, screen: &Screen, windows: &[Window]) -> Result<()> {
    // Handle empty case
    if windows.is_empty() {
        return Ok(());
    }

    let screen_width = screen.width_in_pixels as i16;   // e.g., 1280
    let screen_height = screen.height_in_pixels as i16; // e.g., 720
    let num_windows = windows.len() as i16;

    // Configure master window (first window)
    let master_window = windows[0];
    let master_width = if num_windows > 1 {
        (screen_width as f32 * MASTER_RATIO) as i16  // 50% = 640 pixels
    } else {
        screen_width  // Full width if only one window
    };

    let master_config = ConfigureWindowAux::new()
        .x(0)                           // Left edge of screen
        .y(0)                           // Top edge of screen
        .width(master_width as u32)     // 640 pixels wide
        .height(screen_height as u32);  // Full height

    conn.configure_window(master_window, &master_config)?;

    // Configure stack windows (remaining windows)
    if num_windows > 1 {
        let stack_windows = &windows[1..];  // All except first
        let stack_x = master_width;         // Start where master ends
        let stack_width = screen_width - master_width;  // Remaining width
        let stack_height = screen_height / (num_windows - 1);  // Divide height

        for (index, &window) in stack_windows.iter().enumerate() {
            let stack_y = (index as i16) * stack_height;  // Stack vertically

            let stack_config = ConfigureWindowAux::new()
                .x(stack_x as i32)          // Right half of screen
                .y(stack_y as i32)          // Stacked position
                .width(stack_width as u32)  // Right half width
                .height(stack_height as u32); // Divided height

            conn.configure_window(window, &stack_config)?;
        }
    }

    Ok(())
}
```

**Visual Examples:**

```
1 Window:                2 Windows:               3 Windows:
┌─────────────────┐      ┌────────┬────────┐      ┌────────┬────────┐
│                 │      │        │        │      │        │   W2   │
│       W1        │      │   W1   │   W2   │      │   W1   ├────────┤
│                 │      │        │        │      │        │   W3   │
│                 │      │        │        │      │        │        │
└─────────────────┘      └────────┴────────┘      └────────┴────────┘
     Full screen            50% | 50%              50% | 50% split
```

**Algorithm Steps:**

1. **Master Window**:
   - Always the first window in the list
   - Takes left side of screen
   - Width = MASTER_RATIO * screen_width (default 50%)
   - Height = full screen height

2. **Stack Windows**:
   - All other windows
   - Share the right side of screen
   - Each gets equal height: screen_height / number_of_stack_windows
   - All have same width: remaining screen width

3. **Positioning**:
   - Master: x=0, y=0
   - Stack: x=master_width, y=index*stack_height

---

## Keyboard Management (keyboard.rs)

The keyboard system handles mapping keys and processing shortcuts.

### Key Concepts

- **Keysym**: A universal key identifier (e.g., 0x0074 for 'T')
- **Keycode**: The physical key number on your specific keyboard
- **Modifier**: Keys like Shift, Ctrl, Alt, Super that modify other keys

### Data Structure

```rust
/// Manages keyboard mappings and shortcuts
pub struct KeyboardManager {
    /// Map of keysyms to keycodes
    keycode_map: HashMap<u32, u8>,
    /// Registered shortcuts
    shortcuts: Vec<Shortcut>,
}

/// Represents a keyboard shortcut
#[derive(Debug, Clone)]
pub struct Shortcut {
    pub modifiers: KeyButMask,
    pub keycode: u8,
    pub command: String,
}
```

The keyboard manager now stores both the keycode mapping and a list of registered shortcuts with their associated commands.

### Initialization

```rust
pub fn new<C: Connection>(conn: &C, setup: &Setup) -> Result<Self> {
    let min_keycode = setup.min_keycode;
    let max_keycode = setup.max_keycode;
    
    // Get keyboard mapping from X server
    let mapping_reply = conn
        .get_keyboard_mapping(min_keycode, max_keycode - min_keycode + 1)?
        .reply()?;
    
    let keysyms_per_keycode = mapping_reply.keysyms_per_keycode as usize;
    let mut keycode_map = HashMap::new();
    
    // Build keycode map
    for (index, chunk) in mapping_reply.keysyms.chunks(keysyms_per_keycode).enumerate() {
        let keycode = min_keycode + index as u8;
        
        // Store first keysym for each keycode (unshifted)
        if let Some(&keysym) = chunk.first() {
            if keysym != 0 {
                keycode_map.insert(keysym, keycode);
            }
        }
    }
    
    info!("Initialized keyboard manager with {} keycodes", keycode_map.len());
    
    Ok(Self { 
        keycode_map,
        shortcuts: Vec::new(),
    })
}
```

**What this does:**
1. Ask X11 for the keyboard mapping table
2. For each physical key, get what symbol it represents
3. Build a map: keysym → keycode
4. Example: 0x0074 ('T') → keycode 28

### Registering Shortcuts from Config

```rust
pub fn register_shortcuts<C: Connection>(
    &mut self,
    conn: &C,
    root: Window,
    shortcuts: &HashMap<String, String>,
) -> Result<()> {
    for (key_combo, command) in shortcuts {
        // Parse the key combination (e.g., "Super+t")
        let (modifiers, keysym) = parse_key_combination(key_combo)?;
        
        // Get the physical keycode
        let keycode = self.get_keycode(keysym);
        if keycode == 0 {
            warn!("Could not find keycode for key '{}', skipping", key_combo);
            continue;
        }
        
        // Convert ModMask to KeyButMask for X11
        let key_but_mask = KeyButMask::from(modifiers.bits());
        
        // Grab the key combination
        conn.grab_key(
            true,
            root,
            key_but_mask,
            keycode,
            GrabMode::ASYNC,
            GrabMode::ASYNC,
        )?;
        
        // Store the shortcut
        self.shortcuts.push(Shortcut {
            modifiers: key_but_mask,
            keycode,
            command: command.clone(),
        });
    }
    
    Ok(())
}
```

**What this does:**
1. Iterate through all configured shortcuts
2. Parse human-readable key combinations (handled by keys.rs)
3. Convert to physical keycodes
4. Register each combination with X11
5. Store shortcuts for later matching

---

## Key Parser (keys.rs)

The key parser module handles converting human-readable key combinations into X11 keysyms and modifiers. This is what makes the configuration system user-friendly by allowing keys like "Super+t" instead of raw hex values.

### Core Function

```rust
/// Parse a key combination string like "Super+t" or "Ctrl+Alt+Delete"
pub fn parse_key_combination(combo: &str) -> Result<(ModMask, u32)> {
    let parts: Vec<&str> = combo.split('+').collect();
    
    if parts.is_empty() {
        return Err(anyhow::anyhow!("Empty key combination"));
    }
    
    let mut modifiers = ModMask::from(0u16);
    let key_part;
    
    // Parse modifiers and key
    if parts.len() == 1 {
        // Single key without modifiers
        key_part = parts[0];
    } else {
        // Multiple parts - all but last are modifiers
        for modifier in &parts[..parts.len() - 1] {
            match modifier.to_lowercase().as_str() {
                "super" | "mod4" | "win" | "windows" | "cmd" => modifiers |= ModMask::M4,
                "alt" | "mod1" | "meta" => modifiers |= ModMask::M1,
                "ctrl" | "control" | "ctl" => modifiers |= ModMask::CONTROL,
                "shift" => modifiers |= ModMask::SHIFT,
                "mod2" | "numlock" | "num" => modifiers |= ModMask::M2,
                "mod3" | "scrolllock" | "scroll" => modifiers |= ModMask::M3,
                "mod5" | "altgr" | "altgraph" => modifiers |= ModMask::M5,
                "hyper" => {
                    // Hyper = Super+Alt+Ctrl+Shift
                    modifiers |= ModMask::M4 | ModMask::M1 | 
                                ModMask::CONTROL | ModMask::SHIFT;
                }
                _ => return Err(anyhow::anyhow!("Unknown modifier: {}", modifier)),
            }
        }
        key_part = parts.last().unwrap();
    }
    
    // Convert key name to keysym
    let keysym = get_keysym_from_name(key_part)?;
    
    Ok((modifiers, keysym))
}
```

### Modifier Support

The key parser supports comprehensive modifier keys with alternative names for cross-platform familiarity:

**Primary Modifiers:**
- `Super`, `Mod4`, `Win`, `Windows`, `Cmd` → Super key (Windows/Cmd key)
- `Alt`, `Mod1`, `Meta` → Alt key
- `Ctrl`, `Control`, `Ctl` → Control key
- `Shift` → Shift key

**Less Common Modifiers:**
- `Mod2`, `NumLock`, `Num` → Num Lock
- `Mod3`, `ScrollLock`, `Scroll` → Scroll Lock
- `Mod5`, `AltGr`, `AltGraph` → AltGr (right Alt on international keyboards)

**Special Combinations:**
- `Hyper` → All four main modifiers combined (Super+Alt+Ctrl+Shift)

### Key Name Mapping

```rust
fn get_keysym_from_name(name: &str) -> Result<u32> {
    let normalized = name.to_lowercase();
    
    match normalized.as_str() {
        // Letters (a-z)
        c if c.len() == 1 && c.chars().next().unwrap().is_ascii_lowercase() => {
            Ok(c.chars().next().unwrap() as u32)
        }
        
        // Numbers (0-9)
        c if c.len() == 1 && c.chars().next().unwrap().is_ascii_digit() => {
            Ok(c.chars().next().unwrap() as u32)
        }
        
        // Special keys
        "space" => Ok(0x0020),
        "return" | "enter" => Ok(0xff0d),
        "tab" => Ok(0xff09),
        "escape" | "esc" => Ok(0xff1b),
        "backspace" => Ok(0xff08),
        "delete" | "del" => Ok(0xffff),
        
        // Function keys
        "f1" => Ok(0xffbe),
        "f2" => Ok(0xffbf),
        "f3" => Ok(0xffc0),
        "f4" => Ok(0xffc1),
        "f5" => Ok(0xffc2),
        "f6" => Ok(0xffc3),
        "f7" => Ok(0xffc4),
        "f8" => Ok(0xffc5),
        "f9" => Ok(0xffc6),
        "f10" => Ok(0xffc7),
        "f11" => Ok(0xffc8),
        "f12" => Ok(0xffc9),
        
        // Arrow keys
        "up" => Ok(0xff52),
        "down" => Ok(0xff54),
        "left" => Ok(0xff51),
        "right" => Ok(0xff53),
        
        _ => Err(anyhow::anyhow!("Unknown key name: {}", name)),
    }
}
```

### Example Usage

```rust
// Simple key
parse_key_combination("t") → (ModMask::empty(), 0x0074)

// Single modifier
parse_key_combination("Super+t") → (ModMask::M4, 0x0074)

// Multiple modifiers  
parse_key_combination("Ctrl+Alt+Delete") → (ModMask::CONTROL | ModMask::M1, 0xffff)

// Alternative names
parse_key_combination("Cmd+space") → (ModMask::M4, 0x0020)  // Same as Super+space
parse_key_combination("Win+Return") → (ModMask::M4, 0xff0d)  // Same as Super+Return

// Complex combination
parse_key_combination("Hyper+F12") → (ModMask::M4 | ModMask::M1 | ModMask::CONTROL | ModMask::SHIFT, 0xffc9)
```

### Case Insensitivity

All parsing is case-insensitive for user convenience:

```rust
"SUPER+T" == "super+t" == "Super+T" == "SuPeR+t"
```

### Error Handling

The parser provides helpful error messages:

```rust
parse_key_combination("") → Error: "Empty key combination"
parse_key_combination("Unknown+t") → Error: "Unknown modifier: unknown"  
parse_key_combination("Super+xyz") → Error: "Unknown key name: xyz"
```

### Integration with Keyboard Manager

The key parser is used by the keyboard manager during shortcut registration:

```rust
pub fn register_shortcuts<C: Connection>(
    &mut self,
    conn: &C,
    root: Window,
    shortcuts: &HashMap<String, String>,
) -> Result<()> {
    for (key_combo, command) in shortcuts {
        // Parse the human-readable key combination
        match parse_key_combination(key_combo) {
            Ok((modifiers, keysym)) => {
                // Convert to keycode and register with X11
                let keycode = self.get_keycode(keysym);
                if keycode != 0 {
                    self.register_shortcut(conn, root, modifiers, keycode, command)?;
                }
            }
            Err(e) => warn!("Failed to parse key combination '{}': {}", key_combo, e),
        }
    }
    Ok(())
}
```

This allows users to write configuration like:

```toml
[shortcuts]
"Super+Return" = "xterm"
"Ctrl+Alt+t" = "gnome-terminal" 
"Shift+Alt+1" = "firefox"
"Win+space" = "dmenu_run"  # Alternative name for Super
```

### Benefits

1. **User-Friendly**: Natural key combinations instead of hex codes
2. **Cross-Platform**: Alternative modifier names (Cmd, Win, Meta)
3. **Flexible**: Case-insensitive, multiple naming options
4. **Robust**: Comprehensive error handling and validation
5. **Extensible**: Easy to add new key names and modifiers

---

## Library Structure (lib.rs)

```rust
//! Rustile - A tiling window manager written in Rust
//! 
//! This window manager provides automatic window tiling with a master-stack layout.
//! It's designed to be simple, efficient, and extensible.

pub mod config;
pub mod keyboard;
pub mod keys;
pub mod layout;
pub mod window_manager;
```

This file defines what parts of the library are public. It allows other code (like main.rs) to use our modules.

---

## How Components Interact

Here's how all the pieces work together:

### Startup Sequence

```
1. main.rs
   ├── Initialize logging
   ├── Connect to X11
   └── Create WindowManager
       ├── Load configuration from TOML
       │   └── Try ~/.config/rustile/config.toml
       ├── Create KeyboardManager
       │   └── Load keyboard mappings from X11
       ├── Create LayoutManager
       │   └── Set default layout (MasterStack)
       ├── Register as window manager
       │   └── Tell X11 we control window placement
       └── Register shortcuts from config
           └── Parse and grab all configured key combinations

2. Start event loop
   └── Wait for X11 events forever
```

### Event Processing Flow

```
X11 Event → WindowManager.handle_event()
├── KeyPress → handle_key_press()
│   ├── KeyboardManager checks against registered shortcuts
│   ├── Find matching shortcut by keycode and modifiers
│   └── Execute associated command if match
├── MapRequest → handle_map_request()
│   ├── Make window visible
│   ├── Add to window list
│   └── Apply layout algorithm
└── UnmapNotify → handle_unmap_notify()
    ├── Remove from window list
    └── Re-apply layout algorithm
```

### Layout Application Flow

```
apply_layout()
├── Get screen dimensions from X11
├── Call LayoutManager.apply_layout()
│   └── tile_master_stack()
│       ├── Calculate master window size/position
│       ├── Calculate stack window sizes/positions
│       └── Send configure commands to X11
└── X11 moves/resizes all windows
```

---

## Event Flow

Here's what happens when you use the window manager:

### Opening a Window

```
User runs: xclock
    ↓
X11 creates window but doesn't show it
    ↓
X11 sends MapRequest to window manager
    ↓
WindowManager.handle_map_request()
    ├── conn.map_window() - make it visible
    ├── windows.push() - add to our list
    └── apply_layout() - rearrange everything
        ↓
LayoutManager.tile_master_stack()
    ├── Calculate new positions for all windows
    └── conn.configure_window() for each window
        ↓
X11 moves/resizes windows
    ↓
User sees tiled windows
```

### Closing a Window

```
User closes window (X button or Alt+F4)
    ↓
X11 destroys window
    ↓
X11 sends UnmapNotify to window manager
    ↓
WindowManager.handle_unmap_notify()
    ├── windows.retain() - remove from our list
    └── apply_layout() - rearrange remaining windows
        ↓
LayoutManager.tile_master_stack()
    ├── Calculate new positions for remaining windows
    └── conn.configure_window() for each window
        ↓
X11 moves/resizes remaining windows
    ↓
User sees remaining windows fill the space
```

### Pressing a Configured Shortcut

```
User presses Shift+Alt+1 (configured for gnome-terminal)
    ↓
X11 sends KeyPress event to window manager
    ↓
WindowManager.handle_key_press()
    ↓
KeyboardManager.handle_key_press()
    ├── Find shortcut matching keycode + modifiers
    ├── Return command: "gnome-terminal"
    └── WindowManager executes: Command::new("gnome-terminal").spawn()
        ↓
New gnome-terminal process starts
    ↓
gnome-terminal creates window → MapRequest event
    ↓
(Follow "Opening a Window" flow above)
```

---

## Testing

The project includes comprehensive tests to ensure reliability:

### Unit Tests

Located in each module file (`#[cfg(test)]` sections):

**Config Tests**:
- Test configuration loading from TOML
- Validate default values
- Test accessor methods

**Layout Tests**:
- Test layout manager creation
- Handle empty window lists
- Validate dimension calculations

**Keyboard Tests**:
- Test keycode lookup
- Test shortcut matching
- Handle missing keys

**Keys Tests** (22 tests total):
- Parse simple keys and modifiers
- Test alternative modifier names (Win, Cmd, Meta)
- Case-insensitive parsing
- Complex modifier combinations
- Special keys (Return, space, F-keys)
- Error handling for unknown keys

### Manual Testing

Use `test_rustile.sh` for interactive testing:
1. Starts Xephyr (nested X server)
2. Runs rustile with debug logging
3. Opens test windows
4. Allows manual interaction

### Running Tests

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_master_window_dimensions

# Manual testing
./test_rustile.sh
```

---

## Summary

Rustile is a simple but complete tiling window manager that demonstrates:

1. **X11 Protocol**: How to communicate with the display server
2. **Event-Driven Programming**: Responding to user actions
3. **Modular Design**: Each component has a clear responsibility
4. **Rust Safety**: Memory safety and error handling
5. **Testing**: Unit and integration tests for reliability

The code is designed to be:
- **Readable**: Clear names and documentation
- **Maintainable**: Modular structure
- **Extensible**: Easy to add new features
- **Safe**: Rust's type system prevents common bugs

Recent improvements:
- ✅ Configuration file support (TOML)
- ✅ Human-readable key combinations
- ✅ Support for all X11 modifiers
- ✅ Cross-platform modifier naming

Future features to add:
- Multiple layouts
- Window navigation shortcuts
- Multi-monitor support
- Window decorations
- Status bars

The window manager shows how a relatively small amount of well-structured code can create a functional desktop environment.