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
     - Example: `"Alt+j" = "focus_next"`
     - Example: `"Super+Return" = "xterm"`
   - **layout**: Window layout settings
     - `master_ratio`: 0.0-1.0 (default 0.5) - master window width ratio
     - `gap`: Pixels between windows (default 0, max 500)
     - `border_width`: Border thickness in pixels (default 2, max 50)
     - `focused_border_color`: Color for focused window (default 0xFF0000 - red)
     - `unfocused_border_color`: Color for unfocused windows (default 0x808080 - gray)
   - **general**: General settings
     - `default_display`: X11 display for launching apps

3. **Configuration Validation**:
   - **master_ratio**: Must be between 0.0 and 1.0
   - **gap**: Maximum 500 pixels to prevent unusable layouts
   - **border_width**: Maximum 50 pixels to prevent excessive borders
   - **Combined limits**: gap + border_width maximum 600 pixels total
   - **Shortcuts**: Key combinations and commands must be non-empty

4. **Example Config File**:
```toml
[general]
default_display = ":1"

[layout]
master_ratio = 0.5              # Master window takes 50% width
gap = 10                        # 10 pixel gaps between windows
border_width = 5                # 5 pixel window borders
focused_border_color = 0xFF0000   # Red border for focused window
unfocused_border_color = 0x808080 # Gray border for unfocused windows

[shortcuts]
# Application shortcuts
"Shift+Alt+1" = "gnome-terminal"
"Shift+Alt+2" = "code"
"Super+Return" = "xterm"

# Window management shortcuts
"Alt+j" = "focus_next"           # Cycle to next window
"Alt+k" = "focus_prev"           # Cycle to previous window
"Shift+Alt+m" = "swap_with_master" # Promote window to master
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
    /// Currently focused window
    focused_window: Option<Window>,
    /// Window stack for focus ordering (most recently used first)
    window_stack: Vec<Window>,
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
- `focused_window`: Currently focused window (if any)
- `window_stack`: Stack of windows in most-recently-used order for focus cycling
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
        focused_window: None,
        window_stack: Vec::new(),
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
        
        // Handle window management commands
        match command {
            "focus_next" => return self.focus_next(),
            "focus_prev" => return self.focus_prev(),
            "swap_with_master" => return self.swap_with_master(),
            _ => {
                // Handle regular application commands
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
        }
    }
    Ok(())
}
```

**What happens:**
1. Check for keyboard shortcuts (both window management and application shortcuts)
2. If it's a window management command (focus_next, focus_prev, swap_with_master), handle it directly
3. Otherwise, launch the associated application with proper DISPLAY environment

#### Map Request Handler (New Window)
```rust
fn handle_map_request(&mut self, event: MapRequestEvent) -> Result<()> {
    let window = event.window;
    info!("Mapping window: {:?}", window);
    
    // Set initial border attributes before mapping
    let border_aux = ChangeWindowAttributesAux::new()
        .border_pixel(self.config.unfocused_border_color());
    self.conn.change_window_attributes(window, &border_aux)?;
    
    let config_aux = ConfigureWindowAux::new()
        .border_width(self.config.border_width());
    self.conn.configure_window(window, &config_aux)?;
    
    // Map the window (make it visible)
    self.conn.map_window(window)?;
    
    // Add to managed windows
    self.windows.push(window);
    
    // Set focus to new window
    self.set_focus(window)?;
    
    // Apply layout
    self.apply_layout()?;
    
    Ok(())
}
```

**What happens:**
1. A new window wants to appear
2. Set up border appearance (width and unfocused color)
3. Make the window visible
4. Add it to our list of managed windows
5. Set focus to the new window (including border color change)
6. Rearrange all windows using the layout algorithm

#### Unmap Notify Handler (Window Closed)
```rust
fn handle_unmap_notify(&mut self, event: UnmapNotifyEvent) -> Result<()> {
    let window = event.window;
    info!("Unmapping window: {:?}", window);
    
    // Remove from managed windows and stack
    self.windows.retain(|&w| w != window);
    self.window_stack.retain(|&w| w != window);
    
    // Update focus if focused window was unmapped
    if self.focused_window == Some(window) {
        self.focused_window = self.window_stack.first().copied();
        if let Some(next_focus) = self.focused_window {
            self.set_focus(next_focus)?;
        }
    }
    
    // Reapply layout
    self.apply_layout()?;
    
    Ok(())
}
```

**What happens:**
1. A window has been closed
2. Remove it from our window list and focus stack
3. If the closed window was focused, focus the next window in the MRU stack
4. Rearrange remaining windows to fill the space

### Focus Management and Window Navigation

The window manager now includes sophisticated focus tracking and keyboard navigation.

#### Focus Tracking Data Structures

```rust
/// Currently focused window
focused_window: Option<Window>,
/// Window stack for focus ordering (most recently used first)
window_stack: Vec<Window>,
```

**Focus State Management:**
- `focused_window`: Tracks which window currently has focus (keyboard input)
- `window_stack`: Maintains Most-Recently-Used (MRU) order for intelligent focus cycling
- Visual feedback through colored borders (red for focused, gray for unfocused)

#### Set Focus Function

```rust
fn set_focus(&mut self, window: Window) -> Result<()> {
    if !self.windows.contains(&window) {
        return Ok(());
    }

    // Set X11 input focus
    self.conn.set_input_focus(InputFocus::POINTER_ROOT, window, CURRENT_TIME)?;

    // Update focus state
    self.focused_window = Some(window);

    // Update window stack (MRU order)
    self.window_stack.retain(|&w| w != window);
    self.window_stack.insert(0, window);

    // Update window borders
    self.update_window_borders()?;

    debug!("Focus set to window: {:?}", window);
    Ok(())
}
```

**Focus Setting Steps:**
1. **Validation**: Ensure window is still managed
2. **X11 Focus**: Tell X11 server to direct keyboard input to the window
3. **Internal Tracking**: Update our focus state
4. **MRU Stack**: Move focused window to front of stack (most recent)
5. **Visual Feedback**: Update border colors to show focus state

#### Border Management

```rust
fn update_window_borders(&self) -> Result<()> {
    for &window in &self.windows {
        let is_focused = self.focused_window == Some(window);
        let border_color = if is_focused {
            self.config.focused_border_color()    // Red (0xFF0000)
        } else {
            self.config.unfocused_border_color()  // Gray (0x808080)
        };

        let aux = ChangeWindowAttributesAux::new().border_pixel(border_color);
        self.conn.change_window_attributes(window, &aux)?;

        let config_aux = ConfigureWindowAux::new()
            .border_width(self.config.border_width());
        self.conn.configure_window(window, &config_aux)?;
    }
    Ok(())
}
```

**Border Updates:**
- **Focused windows**: Red border (configurable color)
- **Unfocused windows**: Gray border (configurable color)
- **Border width**: Configurable (default 2px, max 50px)
- **Real-time updates**: Borders change immediately when focus changes

#### Keyboard Navigation Commands

##### Focus Next Window (Alt+j)

```rust
pub fn focus_next(&mut self) -> Result<()> {
    if self.windows.is_empty() {
        return Ok(());
    }

    let next_window = if let Some(current) = self.focused_window {
        // Find current window index and move to next
        if let Some(current_idx) = self.windows.iter().position(|&w| w == current) {
            let next_idx = (current_idx + 1) % self.windows.len();
            self.windows[next_idx]
        } else {
            self.windows[0]
        }
    } else {
        self.windows[0]
    };

    self.set_focus(next_window)?;
    info!("Focused next window: {:?}", next_window);
    Ok(())
}
```

##### Focus Previous Window (Alt+k)

```rust
pub fn focus_prev(&mut self) -> Result<()> {
    if self.windows.is_empty() {
        return Ok(());
    }

    let prev_window = if let Some(current) = self.focused_window {
        // Find current window index and move to previous
        if let Some(current_idx) = self.windows.iter().position(|&w| w == current) {
            let prev_idx = if current_idx == 0 {
                self.windows.len() - 1
            } else {
                current_idx - 1
            };
            self.windows[prev_idx]
        } else {
            self.windows[0]
        }
    } else {
        self.windows[0]
    };

    self.set_focus(prev_window)?;
    info!("Focused previous window: {:?}", prev_window);
    Ok(())
}
```

##### Swap with Master Window (Shift+Alt+m)

```rust
pub fn swap_with_master(&mut self) -> Result<()> {
    if self.windows.len() < 2 {
        return Ok(());
    }

    if let Some(focused) = self.focused_window {
        if let Some(focused_idx) = self.windows.iter().position(|&w| w == focused) {
            if focused_idx != 0 {
                // Swap with master (index 0)
                self.windows.swap(0, focused_idx);
                self.apply_layout()?;
                info!("Swapped window {:?} with master", focused);
            }
        }
    }
    Ok(())
}
```

**Navigation Features:**
- **Cyclic navigation**: Alt+j/k cycles through windows in order
- **Master swap**: Shift+Alt+m promotes focused window to master position
- **Visual feedback**: Immediate border color changes show focus
- **MRU tracking**: Focus history maintained for intelligent window switching

---

## Layout System (layout.rs)

The layout system determines where windows are positioned and how big they are. It now includes a configurable gap system with proper validation and minimum size enforcement.

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

### Master-Stack Algorithm with Gap System

This is the core algorithm that positions windows with configurable gaps and robust minimum size handling:

```rust
fn tile_master_stack(
    &self,
    conn: &impl Connection,
    screen: &Screen,
    windows: &[Window],
    master_ratio: f32,
    gap: u32,
) -> Result<()> {
    if windows.is_empty() {
        return Ok(());
    }

    let screen_width = screen.width_in_pixels as i16;   // e.g., 1280
    let screen_height = screen.height_in_pixels as i16; // e.g., 720
    let num_windows = windows.len() as i16;
    let gap_i16 = gap as i16;

    // Configure master window (first window)
    let master_window = windows[0];
    let master_width = if num_windows > 1 {
        // Multiple windows: master takes ratio of available space, ensure minimum 100px
        let available_width = screen_width - 3 * gap_i16;  // left + center + right gaps
        if available_width > 150 {
            // Need at least 150px total (100px master + 50px stack)
            ((available_width as f32 * master_ratio) as i16).max(100)
        } else {
            // Fallback: reduce gaps to fit windows
            (screen_width / 2).max(100)
        }
    } else {
        // Single window: full width minus gaps, minimum 100px
        (screen_width - 2 * gap_i16).max(100)
    };

    let master_config = ConfigureWindowAux::new()
        .x(gap_i16 as i32)                              // Start after left gap
        .y(gap_i16 as i32)                              // Start after top gap
        .width(master_width.max(100) as u32)            // Minimum 100px width
        .height((screen_height - 2 * gap_i16).max(100) as u32); // Minimum 100px height

    conn.configure_window(master_window, &master_config)?;

    // Configure stack windows if any
    if num_windows > 1 {
        let stack_windows = &windows[1..];
        let num_stack = stack_windows.len() as i16;
        let stack_x = gap_i16 + master_width + gap_i16; // Add gap between master and stack
        let stack_width = (screen_width - stack_x - gap_i16).max(50); // Minimum usable width

        // Ensure we have enough space for stack windows with minimum height
        let min_total_height = num_stack * 50 + (num_stack - 1) * gap_i16; // 50px min per window
        let available_height = screen_height - 2 * gap_i16;

        let total_stack_height = if available_height >= min_total_height {
            available_height - (num_stack - 1) * gap_i16
        } else {
            // Fallback: reduce gaps if necessary to fit windows
            (available_height - num_stack * 50).max(num_stack * 50)
        };

        let stack_height = (total_stack_height / num_stack).max(50); // Minimum 50px height

        for (index, &window) in stack_windows.iter().enumerate() {
            let stack_y = gap_i16 + (index as i16) * (stack_height + gap_i16);

            let stack_config = ConfigureWindowAux::new()
                .x(stack_x as i32)
                .y(stack_y as i32)
                .width(stack_width.max(1) as u32)
                .height(stack_height.max(1) as u32);

            conn.configure_window(window, &stack_config)?;
        }
    }

    Ok(())
}
```

**Visual Examples:**

```
1 Window (with gaps and borders):
┌─────────────────────────────────┐
│ ╔═════════════════════════════╗ │
│ ║                             ║ │
│ ║           W1                ║ │
│ ║       (focused, red)        ║ │
│ ║                             ║ │
│ ╚═════════════════════════════╝ │
└─────────────────────────────────┘
     Gaps around, border colored

2 Windows (master-stack with gaps):
┌─────────────────────────────────┐
│ ╔═══════════════╗ ╔═══════════╗ │
│ ║               ║ ║           ║ │
│ ║      W1       ║ ║    W2     ║ │
│ ║   (focused,   ║ ║ (unfocused║ │
│ ║     red)      ║ ║   gray)   ║ │
│ ║               ║ ║           ║ │
│ ╚═══════════════╝ ╚═══════════╝ │
└─────────────────────────────────┘
  Master ~50%      Stack with gap

3 Windows (with vertical stack gaps):
┌─────────────────────────────────┐
│ ╔═══════════════╗ ╔═══════════╗ │
│ ║               ║ ║    W2     ║ │
│ ║      W1       ║ ║ (unfocused║ │
│ ║   (focused,   ║ ║   gray)   ║ │
│ ║     red)      ║ ╚═══════════╝ │
│ ║               ║               │
│ ║               ║ ╔═══════════╗ │
│ ║               ║ ║    W3     ║ │
│ ╚═══════════════╝ ║(unfocused)║ │
│                   ╚═══════════╝ │
└─────────────────────────────────┘
  Master window       Stacked with gaps
```

**Algorithm Steps:**

1. **Gap Management**:
   - Calculate available space after subtracting gaps
   - Three gaps for dual-pane: left, center (between master/stack), right
   - Additional gaps between stacked windows vertically
   - Fallback logic when gaps are too large for screen size

2. **Master Window**:
   - Always the first window in the list
   - Takes left portion with configurable ratio (default 50%)
   - Width = master_ratio * available_width (after gaps), minimum 100px
   - Height = full screen height minus top/bottom gaps, minimum 100px
   - Positioned with gap offset: x=gap, y=gap

3. **Stack Windows**:
   - All remaining windows in vertical stack
   - Share the right side of screen after master and gaps
   - Each gets equal height with gaps between: minimum 50px per window
   - Width = remaining available width, minimum 50px
   - Positioned: x=gap+master_width+gap, y=gap+index*(height+gap)

4. **Robustness Features**:
   - **Minimum sizes**: 100px master width, 50px stack width/height
   - **Gap validation**: Maximum 500px, combined gap+border max 600px
   - **Overflow protection**: Fallback to reduced gaps when space is tight
   - **Integer safety**: All calculations checked for positive values

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
│   ├── Window management commands: focus_next, focus_prev, swap_with_master
│   └── Application commands: launch programs
├── MapRequest → handle_map_request()
│   ├── Set border attributes (unfocused color, width)
│   ├── Make window visible
│   ├── Add to window list and focus stack
│   ├── Set focus (including border color update)
│   └── Apply layout algorithm
├── UnmapNotify → handle_unmap_notify()
│   ├── Remove from window list and focus stack
│   ├── Update focus if needed (focus next in MRU order)
│   └── Re-apply layout algorithm
├── EnterNotify → handle_enter_notify()
│   └── Optional focus-follows-mouse behavior
└── Focus events (FocusIn/FocusOut)
    └── Handled by internal focus tracking
```

### Layout Application Flow

```
apply_layout()
├── Get screen dimensions from X11
├── Call LayoutManager.apply_layout()
│   └── tile_master_stack(windows, master_ratio, gap)
│       ├── Calculate available space after gaps
│       ├── Ensure minimum window sizes (100px master, 50px stack)
│       ├── Calculate master window size/position with gaps
│       ├── Calculate stack window sizes/positions with gaps
│       ├── Apply overflow protection and fallback sizing
│       └── Send configure commands to X11
├── X11 moves/resizes all windows with gaps and borders
└── Border colors remain based on focus state
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
    ├── Set border attributes (gray border, configurable width)
    ├── conn.map_window() - make it visible
    ├── windows.push() - add to our list
    ├── set_focus() - focus new window and update borders
    │   ├── Update focused_window and window_stack (MRU order)
    │   ├── Set X11 input focus
    │   └── update_window_borders() - red for focused, gray for others
    └── apply_layout() - rearrange everything with gaps
        ↓
 LayoutManager.tile_master_stack(windows, master_ratio, gap)
    ├── Calculate positions with gaps and minimum sizes
    └── conn.configure_window() for each window
        ↓
X11 moves/resizes windows with gaps and colored borders
    ↓
User sees tiled windows with visual focus indication
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

### Using Window Navigation (Alt+j)

```
User presses Alt+j (focus_next)
    ↓
X11 sends KeyPress event to window manager
    ↓
WindowManager.handle_key_press()
    ├── KeyboardManager finds "focus_next" command
    └── WindowManager executes: self.focus_next()
        ↓
focus_next() method
    ├── Find current focused window index in windows list
    ├── Calculate next window index (wrapping around)
    ├── Call set_focus() with next window
    │   ├── Update focused_window and window_stack (MRU order)
    │   ├── Set X11 input focus to new window
    │   └── update_window_borders()
    │       ├── Set focused window border to red
    │       └── Set all other window borders to gray
    └── Layout remains unchanged (only focus changes)
        ↓
User sees border colors change instantly
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
- ✅ Window focus management with visual borders
- ✅ Keyboard navigation (Alt+j/k for cycling, Shift+Alt+m for swapping)
- ✅ Window stack (MRU order) and focus tracking
- ✅ Configurable gap system with validation and bounds checking
- ✅ Enhanced border configuration (width, focused/unfocused colors)
- ✅ Robustness improvements (minimum sizes, overflow protection)

Current capabilities:
- **Focus Management**: Visual feedback with colored borders (red/gray)
- **Keyboard Navigation**: Cycle through windows with Alt+j/k
- **Window Promotion**: Swap focused window to master with Shift+Alt+m
- **Gap System**: Configurable spacing between windows (0-500px)
- **Border System**: Configurable borders with focus-aware colors
- **Robust Layout**: Minimum sizes and overflow protection
- **Configuration Validation**: Prevents invalid settings

Future features to add:
- Multiple layouts (grid, spiral, floating)
- Multi-monitor support
- Window decorations and titles
- Status bars and workspace indicators
- Dynamic layout switching

The window manager shows how a relatively small amount of well-structured code can create a functional desktop environment.