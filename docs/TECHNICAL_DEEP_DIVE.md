# 🔬 Rustile Technical Deep Dive

This document provides comprehensive technical documentation for developers, contributors, and advanced users who want to understand or modify Rustile's internals. For beginner-friendly introduction, see [BEGINNER_GUIDE.md](BEGINNER_GUIDE.md).

## 📋 Table of Contents

1. [Project Overview](#project-overview)
2. [Project Structure](#project-structure)
3. [Core Components](#core-components)
4. [Event Flow](#event-flow)
5. [Memory Layout and Data Structures](#memory-layout-and-data-structures)
6. [X11 Protocol Deep Dive](#x11-protocol-deep-dive)
7. [Layout Algorithm Mathematics](#layout-algorithm-mathematics)
8. [Configuration System](#configuration-system)
9. [Keyboard Handling](#keyboard-handling)
10. [Window Operations](#window-operations)
11. [Performance Analysis](#performance-analysis)
12. [Rust Safety and Error Handling](#rust-safety-and-error-handling)
13. [Testing Architecture](#testing-architecture)
14. [Development Workflow](#development-workflow)
15. [Future Architecture Considerations](#future-architecture-considerations)

## 🏗️ Project Overview

Rustile is a tiling window manager for X11 written in Rust. It automatically arranges windows without overlapping, providing keyboard-driven window management with configurable layouts.

### 🔑 Key Features
- Master-Stack and BSP (Binary Space Partitioning) layouts
- Configurable gaps and window borders with robust validation
- Window focus management with visual indicators
- Window swapping operations (next/prev/master)
- TOML-based configuration with runtime validation
- Keyboard shortcuts with exact modifier matching
- Graceful window destruction with WM_DELETE_WINDOW protocol

### 🏛️ Architecture Diagram
```
┌─────────────┐    ┌──────────────┐    ┌─────────────┐
│   main.rs   │───▶│WindowManager │───▶│ X11 Server  │
└─────────────┘    └──────┬───────┘    └─────────────┘
                          │
                          ▼
              ┌─────────────────────────┐
              │     Core Components     │
              ├─────────────────────────┤
              │ • LayoutManager         │
              │ • KeyboardManager       │
              │ • Config                │
              │ • WindowOps             │
              └─────────────────────────┘
```

## 📁 Project Structure

```
rustile/
├── src/                           # Core source code
│   ├── main.rs                    # Entry point and CLI
│   ├── lib.rs                     # Library interface
│   │
│   ├── window_manager/            # Core window management
│   │   ├── mod.rs                 # Public interface
│   │   ├── core.rs                # Initialization & main loop
│   │   ├── events.rs              # X11 event handling
│   │   ├── focus.rs               # Focus state management
│   │   └── window_ops.rs          # Window operations & layout
│   │
│   ├── layout/                    # Tiling layout algorithms
│   │   ├── mod.rs                 # Layout system interface
│   │   ├── manager.rs             # Layout coordination
│   │   ├── master_stack.rs        # Master-stack algorithm
│   │   ├── bsp.rs                 # BSP algorithm
│   │   ├── types.rs               # Data structures
│   │   ├── traits.rs              # Layout interfaces
│   │   └── constants.rs           # Configuration constants
│   │
│   ├── config/                    # Configuration system
│   │   ├── mod.rs                 # Configuration main
│   │   └── validation.rs          # Input validation
│   │
│   ├── keyboard.rs                # Keyboard shortcut handling
│   └── keys.rs                    # Key parsing utilities
│
├── test.sh                        # Simple test script with Xephyr
├── check.sh                       # Code quality checker
├── docs/                          # Documentation
│   ├── TECHNICAL_DEEP_DIVE.md    # This file
│   ├── BEGINNER_GUIDE.md         # User-friendly guide
│   └── ROADMAP.md               # Future plans
├── .github/workflows/             # CI/CD pipelines
├── config.example.toml            # Example configuration
└── CLAUDE.md                     # Development guidelines
```

## 🔧 Core Components

### 1. Main Entry Point (`main.rs`)

The entry point initializes logging, connects to X11, and starts the window manager:

```rust
fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let (conn, screen_num) = x11rb::connect(None)?;
    let wm = WindowManager::new(conn, screen_num)?;
    wm.run()
}
```

### 2. Window Manager (`window_manager/core.rs`)

The heart of the system that coordinates all components:

**Key Responsibilities:**
- X11 event handling and connection management
- Window lifecycle management (map/unmap/destroy)
- Focus state tracking with visual indicators
- Command dispatch from keyboard shortcuts
- Layout triggering and coordination

**Main Data Structure:**
```rust
pub struct WindowManager<C: Connection> {
    conn: C,                           // X11 connection
    windows: Vec<Window>,              // Managed windows (ordered)
    focused_window: Option<Window>,    // Current focus
    window_stack: Vec<Window>,         // MRU (Most Recently Used) order
    layout_manager: LayoutManager,     // Layout algorithms
    keyboard_manager: KeyboardManager, // Shortcut handling
    config: Config,                    // User settings
    screen_num: usize,                // Active screen number
}
```

**Key Methods:**
- `new()` - Initialize, register as WM, setup event handlers
- `run()` - Main event loop with X11 event processing
- `handle_event()` - Event dispatcher (keyboard, window events)
- `set_focus()` - Focus management with border updates
- `apply_layout()` - Trigger layout recalculation

### 3. Event Handler (`window_manager/events.rs`)

Handles all X11 events with specialized processing:

**Event Types:**
- `MapRequest` - New window creation
- `UnmapNotify` - Window closing/hiding
- `KeyPress` - Keyboard shortcuts
- `ButtonPress` - Mouse focus changes
- `ConfigureRequest` - Window resize requests

### 4. Layout System (`layout/`)

Implements window arrangement algorithms with modular design:

**Supported Layouts:**
```rust
pub enum Layout {
    MasterStack,  // Traditional master + stack
    Bsp,          // Binary space partitioning
}
```

**Layout Manager Coordination:**
- Chooses active layout algorithm
- Handles window additions/removals
- Coordinates with WindowManager for positioning
- Manages layout-specific state

### 5. Window Operations (`window_manager/window_ops.rs`)

Implements window manipulation operations:

**Core Operations:**
- `swap_with_master()` - Swap focused window with master position
- `swap_window_next()` - Swap with next window in sequence
- `swap_window_prev()` - Swap with previous window in sequence
- `destroy_focused_window()` - Graceful window termination
- `apply_layout()` - Coordinate layout recalculation

**Window Destruction Protocol:**
1. Try graceful close with `WM_DELETE_WINDOW` message
2. Fall back to forceful `XKillClient` if unsupported
3. Remove from window list and update focus
4. Trigger layout recalculation

## 🔄 Event Flow

### Window Creation Sequence
```
Application starts (e.g., xterm)
    ↓
X11 sends MapRequest event
    ↓
WindowManager.handle_map_request()
    ├── Validate window properties
    ├── Set border attributes (width, color)
    ├── Make window visible (map_window)
    ├── Add to window list
    ├── Set focus (update borders: red=focused, gray=unfocused)
    └── Apply layout algorithm
        ├── Calculate positions for all windows
        ├── Send ConfigureWindow requests to X11
        └── Flush changes to display
    ↓
Windows arranged on screen
```

### Keyboard Input Processing
```
User presses key combination (e.g., Alt+j)
    ↓
X11 sends KeyPress event
    ↓
KeyboardManager.handle_key_press()
    ├── Extract modifiers and keycode
    ├── Apply modifier mask (ignore NumLock, CapsLock)
    ├── Match against configured shortcuts (exact matching)
    └── Return command string or None
    ↓
Execute command:
    ├── Window commands (focus_next, swap_with_master, destroy_window)
    ├── Layout commands (switch_layout)
    └── Application launches (xterm, etc.)
```

### Window Closing Sequence
```
Window closes (user closes app or destroy_window command)
    ↓
X11 sends UnmapNotify event
    ↓
WindowManager.handle_unmap_notify()
    ├── Remove from window list
    ├── Update focus if closed window was focused
    │   └── Focus next available window
    └── Re-apply layout to remaining windows
    ↓
Screen updated with new arrangement
```

## ⚙️ Configuration System

### 📝 TOML Configuration Structure

Rustile uses TOML for human-readable configuration with comprehensive validation:

```toml
# ~/.config/rustile/config.toml
[layout]
layout_algorithm = "master_stack"    # "master_stack" or "bsp"
master_ratio = 0.6                   # Master window width ratio (0.0-1.0)
bsp_split_ratio = 0.5                # BSP split ratio (0.0-1.0)
gap = 15                             # Pixels between windows (0-500)
border_width = 2                     # Window border thickness (0-50)
focused_border_color = 0xFF0000      # Red border for focused window
unfocused_border_color = 0x808080    # Gray border for unfocused windows

[shortcuts]
"Alt+j" = "focus_next"               # Move focus to next window
"Alt+k" = "focus_prev"               # Move focus to previous window
"Shift+Alt+j" = "swap_window_next"   # Swap with next window
"Shift+Alt+k" = "swap_window_prev"   # Swap with previous window
"Shift+Alt+m" = "swap_with_master"   # Swap focused with master
"Shift+Alt+q" = "destroy_window"     # Close focused window
"Super+Return" = "xterm"             # Launch terminal
```

### 🛡️ Configuration Validation Rules

```rust
// Validation constraints for robustness
const MIN_GAP: u32 = 0;
const MAX_GAP: u32 = 500;
const MIN_BORDER_WIDTH: u32 = 0;
const MAX_BORDER_WIDTH: u32 = 50;
const MAX_COMBINED_GAP_BORDER: u32 = 600;
const MIN_MASTER_RATIO: f32 = 0.0;
const MAX_MASTER_RATIO: f32 = 1.0;

// Combined validation
if gap + border_width > MAX_COMBINED_GAP_BORDER {
    return Err("Gap + border width cannot exceed 600 pixels");
}
```

**Validation Examples:**
- ✅ `gap = 10, border_width = 5` → Valid
- ❌ `gap = 400, border_width = 300` → Exceeds combined limit
- ✅ `master_ratio = 0.7` → Valid
- ❌ `master_ratio = 1.5` → Outside valid range

### 🔄 Configuration Loading Process

```
1. Startup → Check ~/.config/rustile/config.toml
         ↓
2. File exists? → Parse TOML → Validate values → Apply settings
         ↓                ↓               ↓
3. File missing → Use defaults → Skip validation → Apply defaults
         ↓
4. Parse error → Log error → Use defaults → Continue startup
         ↓
5. Invalid values → Log specific errors → Use defaults → Continue startup
```

**Error Handling:**
```rust
// Clear, actionable error messages
"Gap value 600 exceeds maximum of 500 pixels"
"Master ratio 1.2 must be between 0.0 and 1.0"
"Invalid key combination 'Alt+Invalid' in shortcuts"
```

### 🔧 Runtime Configuration Behavior

- **Startup Only**: Configuration loaded once at startup
- **No Hot-reload**: Changes require restart (planned feature)
- **Graceful Fallback**: Invalid configs use safe defaults
- **User Feedback**: Clear error messages for debugging

## ⌨️ Keyboard Handling

### 🐛 Critical Bug Fix: Exact Modifier Matching

**Problem:** Original implementation used subset matching, causing conflicts:
```rust
// OLD (buggy) - subset matching
if event.state.contains(shortcut.modifiers) {
    // Alt+j matched when Shift+Alt+j was pressed!
    return Some(&shortcut.command);
}
```

**Solution:** Implemented exact modifier matching with masking:
```rust
// NEW (fixed) - exact matching
pub fn handle_key_press(&self, event: &KeyPressEvent) -> Option<&str> {
    // Mask out lock keys (NumLock, CapsLock, ScrollLock)
    let relevant_modifiers = ModMask::SHIFT.bits() 
                           | ModMask::CONTROL.bits() 
                           | ModMask::M1.bits()      // Alt
                           | ModMask::M4.bits();     // Super
    
    let event_modifiers_bits = event.state.bits() & relevant_modifiers;
    
    for shortcut in &self.shortcuts {
        // Exact bit comparison instead of subset matching
        if event_modifiers_bits == shortcut.modifiers.bits() 
           && event.detail == shortcut.keycode {
            return Some(&shortcut.command);
        }
    }
    None
}
```

**Impact:**
- ✅ `Alt+j` only matches `Alt+j`, not `Shift+Alt+j`
- ✅ `Shift+Alt+j` works independently for window swapping
- ✅ Lock keys (NumLock, CapsLock) are properly ignored
- ✅ All modifier combinations work as expected

### 🔤 Key Parsing System

```rust
// Human-readable → X11 representation
"Super+Return" → (ModMask::M4, 0xff0d)
"Ctrl+Alt+Delete" → (ModMask::CONTROL | ModMask::M1, 0xffff)
"Shift+Alt+j" → (ModMask::SHIFT | ModMask::M1, 0x006a)
```

**Modifier Mapping:**
- `Shift` → `ModMask::SHIFT`
- `Ctrl` → `ModMask::CONTROL`
- `Alt` → `ModMask::M1`
- `Super` (Windows key) → `ModMask::M4`

## 🪟 Window Operations

### 🔄 Window Swapping Implementation

Recent addition: Comprehensive window swapping system with three operations:

```rust
// Swap directions for code reuse
#[derive(Debug, Clone, Copy)]
enum SwapDirection {
    Next,     // Swap with next window in list
    Previous, // Swap with previous window in list
}

// Public interface methods
pub fn swap_window_next(&mut self) -> Result<()> {
    self.swap_window_direction(SwapDirection::Next)
}

pub fn swap_window_prev(&mut self) -> Result<()> {
    self.swap_window_direction(SwapDirection::Previous)
}

pub fn swap_with_master(&mut self) -> Result<()> {
    // Direct swap with master (index 0)
    if let Some(focused_idx) = self.find_focused_index() {
        if focused_idx != 0 {
            self.windows.swap(0, focused_idx);
            self.apply_layout()?;
        }
    }
    Ok(())
}
```

**Swapping Logic:**
```rust
fn swap_window_direction(&mut self, direction: SwapDirection) -> Result<()> {
    if self.windows.len() < 2 { return Ok(()); }
    
    if let Some(focused_idx) = self.find_focused_index() {
        let target_idx = match direction {
            SwapDirection::Next => (focused_idx + 1) % self.windows.len(),
            SwapDirection::Previous => {
                if focused_idx == 0 {
                    self.windows.len() - 1  // Wrap to end
                } else {
                    focused_idx - 1
                }
            }
        };
        
        self.windows.swap(focused_idx, target_idx);
        self.apply_layout()?;  // Trigger visual update
    }
    Ok(())
}
```

**Example Swapping Sequence:**
```
Initial: [window_A, window_B, window_C], focused = window_B

swap_window_next():
  Before: [A, B*, C]  (* = focused)
  After:  [A, C, B*]  (B swapped with C)
  
swap_window_prev() from focused = C:
  Before: [A, C*, B]
  After:  [C*, A, B]  (C swapped with A, wrapped around)
  
swap_with_master() from focused = A:
  Before: [C, A*, B]
  After:  [A*, C, B]  (A swapped with master position)
```

### 🗑️ Window Destruction Protocol

Implements graceful window closing with fallback:

```rust
pub fn destroy_focused_window(&mut self) -> Result<()> {
    if let Some(focused) = self.focused_window {
        // 1. Try graceful close first
        self.close_window_gracefully(focused)
            .or_else(|_| {
                // 2. Fall back to forceful termination
                self.kill_window_forcefully(focused)
            })?;
    }
    Ok(())
}
```

**Graceful Close Process:**
1. Query window for `WM_PROTOCOLS` property
2. Check if `WM_DELETE_WINDOW` is supported
3. Send `ClientMessage` with `WM_DELETE_WINDOW`
4. Let application handle cleanup and close itself

**Forceful Termination:**
1. Use `XKillClient` to immediately terminate
2. X11 cleans up resources
3. Application may lose unsaved data

## 🧬 Memory Layout and Data Structures

### 🏗️ WindowManager Structure Breakdown

```rust
pub struct WindowManager<C: Connection> {
    conn: C,                           // X11 connection (heap allocated)
    windows: Vec<Window>,              // Dynamic array of window IDs
    window_stack: Vec<Window>,         // Window stacking order
    focused_window: Option<Window>,    // Currently focused window ID
    config: Config,                    // Configuration struct
    layout_manager: LayoutManager,     // Layout algorithm coordinator
    screen_num: usize,                // Active screen number
}
```

**Memory visualization:**
```
Stack Memory:                    Heap Memory:
┌─────────────────────┐         ┌─────────────────────────┐
│ WindowManager       │         │ Vec<Window> capacity: 8 │
│ ├─ conn: *ptr    ───┼────────►│ [101, 102, 103, _, _, _] │
│ ├─ windows: Vec  ───┼────────►│ length: 3               │
│ ├─ window_stack  ───┼────────►│ ┌─────────────────────┐ │
│ ├─ focused_window   │         │ │ Another Vec<Window> │ │
│ ├─ config          │         │ └─────────────────────┘ │
│ ├─ layout_manager   │         └─────────────────────────┘
│ └─ screen_num       │
└─────────────────────┘
```

### 🌳 BSP Tree Structure

```rust
pub enum BspNode {
    Split {
        direction: SplitDirection,  // Vertical or Horizontal
        ratio: f32,                // Split ratio (0.0-1.0)
        left: Box<BspNode>,        // Left/top child (heap allocated)
        right: Box<BspNode>,       // Right/bottom child (heap allocated)
    },
    Leaf(Window),                  // Terminal node with window ID
}
```

**Tree visualization with memory layout:**
```
BSP Tree for 4 windows:               Memory Layout:

         Split(V, 0.5)                Stack:     Heap:
        ┌─────┴─────┐                 ┌─────┐    ┌─────────────┐
        │           │                 │ BSP │───►│ Split Node  │
     Leaf(1)   Split(H, 0.5)         │Tree │    │ ├─ V, 0.5   │
                   ┌─────┴─────┐      └─────┘    │ ├─ left: *──┼─►[Leaf(1)]
                   │           │                 │ └─ right: *─┼─►┌─────────┐
                Leaf(2)     Split(V, 0.5)                      │  │Split H  │
                           ┌─────┴─────┐                       │  │├─ H,0.5 │
                           │           │                       │  │├─left:*─┼─►[Leaf(2)]
                        Leaf(3)     Leaf(4)                    │  │└─right*─┼─►┌──────┐
                                                              │  └─────────┘ │Split V│
Result on screen:                                             │              │├─V,0.5│
┌─────────┬─────────┐                                         │              │├─left*┼─►[Leaf(3)]
│    1    │    2    │                                         │              │└─rght*┼─►[Leaf(4)]
├─────────┼─────────┤                                         │              └──────┘
│    3    │    4    │                                         └─────────────┘
└─────────┴─────────┘
```

## ⚡ Performance Analysis

### 🔄 Event Loop Performance

```
X11 Event Loop Complexity Analysis:

┌─ wait_for_event() ───────────────── O(1) - blocking syscall
│
├─ Pattern matching ───────────────── O(1) - compile-time optimization
│
├─ Window operations:
│  ├─ Add window ──────────────────── O(n) - Vec::push + layout recalc
│  ├─ Remove window ───────────────── O(n) - Vec::retain + layout recalc
│  ├─ Focus next/prev ─────────────── O(n) - linear search in Vec
│  └─ Swap with master ───────────── O(1) - Vec::swap + layout recalc
│
└─ Layout algorithms:
   ├─ Master-stack ──────────────────── O(n) - linear iteration
   └─ BSP ────────────────────────────── O(n log n) - tree operations
```

### 📊 Memory Usage Patterns

```rust
// Typical memory usage for 10 windows:
struct MemoryFootprint {
    window_ids: Vec<u32>,        // 10 * 4 bytes = 40 bytes
    bsp_nodes: Vec<BspNode>,     // ~10 * 32 bytes = 320 bytes (tree)
    config: Config,              // ~200 bytes (small structs)
    x11_buffers: Vec<u8>,        // Variable (protocol buffers)
    // Total: < 1KB for typical usage
}
```

## 🔧 X11 Protocol Deep Dive

### 📨 Message Flow

```
Application Lifecycle:

1. Application starts:
   App ──► X11 Server: CreateWindow(width, height, class)
   X11 Server ──► Rustile: MapRequestEvent { window: 12345 }

2. Rustile processes:
   ┌─────────────────────────────────────────────────────────┐
   │ handle_map_request(MapRequestEvent)                     │
   │ ├─ conn.map_window(12345)           // Make visible     │
   │ ├─ windows.push(12345)              // Track window     │
   │ ├─ layout_manager.add_window(12345) // Add to layout    │
   │ └─ apply_layout()                   // Recalculate      │
   └─────────────────────────────────────────────────────────┘

3. Layout positioning:
   Rustile ──► X11 Server: ConfigureWindow {
       window: 12345,
       x: 960, y: 0,
       width: 960, height: 540
   }

4. Visual update:
   X11 Server ──► Hardware: Update display buffer
   User sees: Window appears in calculated position
```

### 🎯 Focus Management Protocol

```
Focus Change Sequence:

User Input: Alt+j keypress
     │
     ▼
┌─────────────────────────────────────────────────────────────┐
│ X11 KeyPress Event                                          │
│ ┌─ key_code: 44 (j)                                         │
│ ├─ modifiers: Mod1Mask (Alt)                               │
│ └─ window: current_focused                                  │
└─────────────────────┬───────────────────────────────────────┘
                      │
                      ▼
┌─────────────────────────────────────────────────────────────┐
│ Rustile Event Handler                                       │
│ ┌─ Parse key combination: "Alt+j"                           │
│ ├─ Lookup command: "focus_next"                             │
│ └─ Execute: focus_next()                                    │
└─────────────────────┬───────────────────────────────────────┘
                      │
                      ▼
┌─────────────────────────────────────────────────────────────┐
│ Focus Management                                            │
│ ┌─ current: Some(window_A)                                  │
│ ├─ calculate next: window_B                                 │
│ ├─ conn.set_input_focus(window_B, CURRENT_TIME)           │
│ └─ update borders: set_window_border_color()               │
└─────────────────────┬───────────────────────────────────────┘
                      │
                      ▼
┌─────────────────────────────────────────────────────────────┐
│ X11 Protocol Messages                                       │
│ ┌─ SetInputFocus { focus: window_B, time: CURRENT_TIME }   │
│ ├─ ChangeWindowAttributes {                                 │
│ │    window: window_A,                                      │
│ │    border_pixel: 0x808080  // Gray                       │
│ │  }                                                        │
│ └─ ChangeWindowAttributes {                                 │
│      window: window_B,                                      │
│      border_pixel: 0xFF0000  // Red                        │
│    }                                                        │
└─────────────────────────────────────────────────────────────┘
```

## 🧮 Layout Algorithm Mathematics

### 🔄 Algorithm Selection and Coordination

The layout system uses a manager pattern for algorithm coordination:

```rust
pub struct LayoutManager {
    current_layout: Layout,
    master_stack: MasterStackLayout,
    bsp: BspLayout,
}

impl LayoutManager {
    pub fn apply_layout(
        &mut self,
        conn: &impl Connection,
        windows: &[Window],
        focused_window: Option<Window>,
        screen_width: u16,
        screen_height: u16,
        master_ratio: f32,
        bsp_split_ratio: f32,
        min_width: u32,
        min_height: u32,
        gap: u32,
    ) -> Result<()> {
        match self.current_layout {
            Layout::MasterStack => {
                self.master_stack.apply_layout(
                    conn, windows, focused_window,
                    screen_width, screen_height,
                    master_ratio, min_width, min_height, gap
                )
            },
            Layout::Bsp => {
                self.bsp.apply_layout(
                    conn, windows, focused_window,
                    screen_width, screen_height,
                    bsp_split_ratio, min_width, min_height, gap
                )
            },
        }
    }
}
```

### 📐 Master-Stack Calculations

```rust
// Mathematical model for master-stack layout:
fn calculate_master_stack_geometry(
    screen_width: u16,      // S_w
    screen_height: u16,     // S_h  
    num_windows: usize,     // n
    master_ratio: f32,      // r
    gap: u32,              // g
) -> Vec<WindowGeometry> {
    
    // Master window:
    // x = g
    // y = g  
    // width = (S_w - 3g) * r     // 3 gaps: left, center, right
    // height = S_h - 2g          // 2 gaps: top, bottom
    
    let master_x = gap;
    let master_y = gap;
    let master_width = ((screen_width as f32 - 3.0 * gap as f32) * master_ratio) as u32;
    let master_height = screen_height - 2 * gap;
    
    // Stack windows (if n > 1):
    // x = g + master_width + g = g + (S_w - 3g) * r + g
    // y = g + i * (stack_height + g)    where i = window index
    // width = S_w - x - g = S_w - g - (S_w - 3g) * r - g - g = (S_w - 3g) * (1 - r)
    // height = (S_h - 2g - (n-1) * g) / (n-1) = (S_h - (n+1) * g) / (n-1)
    
    if num_windows == 1 {
        return vec![WindowGeometry { 
            x: master_x, y: master_y, 
            width: screen_width - 2 * gap,  // Full width minus side gaps
            height: master_height 
        }];
    }
    
    let stack_x = master_x + master_width + gap;
    let stack_width = ((screen_width as f32 - 3.0 * gap as f32) * (1.0 - master_ratio)) as u32;
    let available_stack_height = screen_height - 2 * gap - (num_windows - 1) as u32 * gap;
    let stack_height = available_stack_height / (num_windows - 1) as u32;
    
    let mut geometries = vec![WindowGeometry {
        x: master_x, y: master_y,
        width: master_width, height: master_height,
    }];
    
    for i in 1..num_windows {
        geometries.push(WindowGeometry {
            x: stack_x,
            y: master_y + (i - 1) as u32 * (stack_height + gap),
            width: stack_width,
            height: stack_height,
        });
    }
    
    geometries
}
```

**Visual proof with numbers:**
```
Screen: 1920x1080, master_ratio: 0.6, gap: 10, windows: 3

// Enhanced gap system with border integration
const EFFECTIVE_GAPS = gap + border_width;  // Total spacing
const SCREEN_AVAILABLE_WIDTH = 1920 - (2 * gap);  // Account for screen edges
const SCREEN_AVAILABLE_HEIGHT = 1080 - (2 * gap);

// Window border visual integration
for window in windows {
    conn.change_window_attributes(window, &ChangeWindowAttributesAux::new()
        .border_width(config.border_width())
        .border_pixel(if focused { 
            config.focused_border_color() 
        } else { 
            config.unfocused_border_color() 
        })
    )?;
}

Master calculation:
├─ x = 10
├─ y = 10
├─ width = (1920 - 30) * 0.6 = 1890 * 0.6 = 1134
└─ height = 1080 - 20 = 1060

Stack calculations (2 windows):
├─ x = 10 + 1134 + 10 = 1154
├─ width = (1920 - 30) * 0.4 = 756
├─ available_height = 1080 - 20 - 10 = 1050
├─ stack_height = 1050 / 2 = 525
├─ stack1_y = 10 + 0 * (525 + 10) = 10
└─ stack2_y = 10 + 1 * (525 + 10) = 545

Result:
┌────────────────────────────────────────────────────────────┐
│10                                                          │
│ ┌──────────────────────────────────┐10┌─────────────────┐ │
│ │                                  │  │     Stack1      │ │
│ │            Master                │  │   756x525       │ │
│ │           1134x1060              │  │                 │ │
│ │                                  │  └─────────────────┘ │
│ │                                  │10┌─────────────────┐ │
│ │                                  │  │     Stack2      │ │
│ │                                  │  │   756x525       │ │
│ │                                  │  │                 │ │
│ └──────────────────────────────────┘  └─────────────────┘ │
└────────────────────────────────────────────────────────────┘
```

### 🌳 BSP Tree Traversal Algorithm

```rust
// Recursive BSP layout calculation:
fn apply_bsp_recursive(
    node: &BspNode,
    rect: Rectangle,     // Available space
    min_width: u32,
    min_height: u32,
    gap: u32,
) -> Vec<WindowGeometry> {
    match node {
        BspNode::Leaf(window) => {
            // Terminal case: position single window
            vec![WindowGeometry {
                x: rect.x,
                y: rect.y,
                width: rect.width.max(min_width),
                height: rect.height.max(min_height),
            }]
        },
        BspNode::Split { direction, ratio, left, right } => {
            let gap_i32 = gap as i32;
            
            // Calculate split rectangles
            let (left_rect, right_rect) = match direction {
                SplitDirection::Vertical => {
                    // Split left/right
                    let split_x = (rect.width as f32 * ratio) as i32;
                    let left_rect = Rectangle {
                        x: rect.x,
                        y: rect.y,
                        width: split_x.max(min_width as i32),
                        height: rect.height,
                    };
                    let right_rect = Rectangle {
                        x: rect.x + split_x + gap_i32,
                        y: rect.y,
                        width: (rect.width - split_x - gap_i32).max(min_width as i32),
                        height: rect.height,
                    };
                    (left_rect, right_rect)
                },
                SplitDirection::Horizontal => {
                    // Split top/bottom  
                    let split_y = (rect.height as f32 * ratio) as i32;
                    let left_rect = Rectangle {
                        x: rect.x,
                        y: rect.y,
                        width: rect.width,
                        height: split_y.max(min_height as i32),
                    };
                    let right_rect = Rectangle {
                        x: rect.x,
                        y: rect.y + split_y + gap_i32,
                        width: rect.width,
                        height: (rect.height - split_y - gap_i32).max(min_height as i32),
                    };
                    (left_rect, right_rect)
                },
            };
            
            // Recursively process children
            let mut result = apply_bsp_recursive(left, left_rect, min_width, min_height, gap);
            result.extend(apply_bsp_recursive(right, right_rect, min_width, min_height, gap));
            result
        }
    }
}
```

**BSP Split Decision Tree:**
```
Decision Process for Window Placement:

Split Count = 0 (even) ──► Vertical Split
│
├─ Available: 1920x1080
├─ Ratio: 0.5  
├─ Left:  960x1080 (window A)
└─ Right: 960x1080 (available for next split)

Split Count = 1 (odd) ──► Horizontal Split  
│
├─ Available: 960x1080 (right side from above)
├─ Ratio: 0.5
├─ Top:    960x540 (window B)
└─ Bottom: 960x540 (available for next split)

Split Count = 2 (even) ──► Vertical Split
│
├─ Available: 960x540 (bottom-right from above)  
├─ Ratio: 0.5
├─ Left:  480x540 (window C)
└─ Right: 480x540 (window D)

Final Layout:
┌─────────────┬─────────────┐
│             │      B      │
│      A      ├─────┬───────┤
│             │  C  │   D   │
└─────────────┴─────┴───────┘
```

## 🔒 Rust Safety and Error Handling

### 🛡️ Memory Safety Guarantees

```rust
// Rust prevents these common C/C++ window manager bugs:

// 1. Dangling window pointers:
let window_id: Window = 12345;
windows.retain(|&w| w != window_id);  // Window removed from list
// Later access to window_id is safe - just an integer, not a pointer

// 2. Buffer overflows in window lists:
let mut windows: Vec<Window> = Vec::with_capacity(10);
windows.push(new_window);  // Automatically grows, no overflow possible

// 3. Use-after-free of X11 resources:
impl Drop for WindowManager<C> {
    fn drop(&mut self) {
        // Rust automatically cleans up X11 connection
        // No manual resource management required
    }
}

// 4. Race conditions with window state:
struct WindowManager<C: Connection> {
    // All fields owned by single thread - no data races
    // X11 connection is !Send + !Sync - prevents accidental sharing
}
```

### ⚠️ Error Propagation Pattern

```rust
// Rustile's error handling strategy:
use anyhow::{Result, Context};

fn complex_window_operation(&mut self) -> Result<()> {
    // Each fallible operation uses ? operator for early return
    let window = self.create_window()
        .context("Failed to create window")?;
    
    self.configure_window(window)
        .context("Failed to configure window")?;
    
    self.map_window(window)
        .context("Failed to map window")?;
    
    self.apply_layout()
        .context("Failed to apply layout after window creation")?;
    
    Ok(())  // Success case
}

// Error chain example:
// Error: Failed to apply layout after window creation
// Caused by: Failed to calculate BSP layout  
// Caused by: X11 connection lost
// Caused by: Broken pipe (os error 32)
```

## 🚀 Performance Optimizations

### ⚡ Layout Calculation Caching

```rust
// Future optimization opportunity:
pub struct LayoutCache {
    last_window_count: usize,
    last_screen_size: (u16, u16),
    last_config_hash: u64,
    cached_geometries: Vec<WindowGeometry>,
}

impl LayoutCache {
    fn is_valid(&self, current_state: &WindowManagerState) -> bool {
        self.last_window_count == current_state.windows.len()
            && self.last_screen_size == current_state.screen_size
            && self.last_config_hash == current_state.config_hash()
    }
    
    // Only recalculate if something changed
    fn get_or_calculate(&mut self, state: &WindowManagerState) -> &[WindowGeometry] {
        if !self.is_valid(state) {
            self.cached_geometries = calculate_layout(state);
            self.update_cache_keys(state);
        }
        &self.cached_geometries
    }
}
```

### 🔄 Event Loop Optimizations

```rust
// Batch X11 operations for better performance:
impl WindowManager<C> {
    fn apply_layout_batch(&mut self) -> Result<()> {
        let geometries = self.layout_manager.calculate_all_positions(&self.windows)?;
        
        // Batch configure requests instead of individual calls
        let configure_requests: Vec<_> = self.windows
            .iter()
            .zip(geometries.iter())
            .map(|(&window, geometry)| {
                ConfigureWindowRequest {
                    window,
                    value_list: ConfigureWindowAux::new()
                        .x(geometry.x as i32)
                        .y(geometry.y as i32)
                        .width(geometry.width)
                        .height(geometry.height),
                }
            })
            .collect();
        
        // Send all requests in one batch
        for request in configure_requests {
            self.conn.configure_window(request.window, &request.value_list)?;
        }
        
        // Single flush instead of per-window flushes
        self.conn.flush()?;
        Ok(())
    }
}
```

## 🧪 Testing Architecture

### 🎯 Testing Strategy Overview

Rustile employs a multi-layered testing approach:

**1. Unit Tests** - Component isolation
**2. Integration Tests** - Full system behavior
**3. Manual Testing** - Interactive validation
**4. Configuration Tests** - Validation robustness

### 🔧 Development Workflow Integration

```bash
# Comprehensive testing script
./test.sh
```

**Test Execution Flow:**
```
1. Unit Tests
   ├── Window operation logic
   ├── Layout calculations
   ├── Configuration validation
   └── Key parsing utilities

2. Integration Tests
   ├── Event handling
   ├── Focus management
   ├── Layout application
   └── Window lifecycle

3. Code Quality Checks
   ├── cargo fmt (formatting)
   ├── cargo clippy (linting)
   └── cargo doc (documentation)

4. Manual Test Environment
   └── Xephyr-based interactive testing
```

### 🎯 Unit Test Strategy

```rust
// Window manager business logic tests (no X11 required):
mod tests {
    use super::*;
    
    // Test pure functions without X11 dependencies
    #[test]
    fn test_focus_navigation_logic() {
        let windows = vec![10, 20, 30];
        
        // Test wrapping behavior
        assert_eq!(find_next_window(&windows, Some(30)), Some(10));
        assert_eq!(find_next_window(&windows, Some(10)), Some(20));
        
        // Test edge cases
        assert_eq!(find_next_window(&[], None), None);
        assert_eq!(find_next_window(&windows, Some(999)), Some(10));
    }
    
    #[test]  
    fn test_master_stack_geometry_calculation() {
        let geometries = calculate_master_stack_layout(
            1920, 1080,  // screen size
            3,           // window count  
            0.6,         // master ratio
            10,          // gap
        );
        
        assert_eq!(geometries.len(), 3);
        
        // Master window
        assert_eq!(geometries[0].width, 1134);  // (1920-30) * 0.6
        assert_eq!(geometries[0].height, 1060); // 1080 - 20
        
        // Stack windows
        assert_eq!(geometries[1].width, 756);   // (1920-30) * 0.4
        assert_eq!(geometries[1].height, 525);  // (1060-10) / 2
    }
}
```

### 🖥️ Integration Test Environment

```bash
# Xephyr-based testing setup:
#!/bin/bash
# test_focus.sh

# 1. Start nested X server
Xephyr :10 -screen 1280x720 &
XEPHYR_PID=$!

# 2. Set up test environment
export DISPLAY=:10
export RUST_LOG=debug

# 3. Start window manager
cargo run &
WM_PID=$!

# 4. Launch test applications
sleep 1
xterm -e "sleep 30" &
xterm -e "sleep 30" &  
xterm -e "sleep 30" &

# 5. Interactive testing
echo "Test environment ready!"
echo "Press Ctrl+C to cleanup"

# 6. Cleanup on exit
trap "kill $XEPHYR_PID $WM_PID; exit" INT
wait
```

## 🛠️ Development Workflow

### 🚀 Quick Development Commands

```bash
# Setup development environment
# Setup: Ensure cargo is available\nsource ~/.cargo/env

# Run comprehensive tests
./test.sh

# Interactive layout testing
./test.sh

# Quality checks (fmt, clippy, test, docs)
./check.sh

# Clean build artifacts
cargo clean

# Build release binary
cargo build --release
```

### ✅ Code Quality Standards

**Required Before Commits:**
```bash
source ~/.cargo/env  # Ensure cargo is in PATH
cargo fmt           # Format code
cargo clippy -- -D warnings  # Check for lints (treat warnings as errors)
cargo test          # Run all tests
```

**Code Standards:**
- **Formatting**: All code MUST be formatted with `cargo fmt`
- **Linting**: All clippy warnings MUST be resolved (use `-D warnings` flag)
- **Testing**: All tests MUST pass before commits
- **Documentation**: Use `///` for public APIs, `//!` for module-level docs
- **Error Handling**: Use `anyhow::Result` for error propagation, never use `unwrap()` in production

### 🔧 Adding New Features

**1. New Layout Algorithm:**
- Add variant to `Layout` enum in `layout/types.rs`
- Implement `LayoutAlgorithm` trait
- Add to `LayoutManager` coordination
- Update configuration options
- Add comprehensive tests
- Update example config

**2. New Keyboard Command:**
- Add command to shortcuts config validation
- Implement handler in appropriate module
- Add to event dispatcher
- Test with interactive environment
- Update documentation

**3. New Configuration Option:**
- Add field to config structs
- Implement validation rules
- Update defaults and example
- Add tests for edge cases
- Document in user guides

### 🧪 Testing Workflow

**Unit Testing:**
```bash
# Test specific modules
cargo test window_manager::tests
cargo test layout::master_stack::tests
cargo test config::validation::tests

# Test with output
cargo test -- --nocapture
```

**Integration Testing:**
```bash
# Start test environment
./test.sh

# In another terminal, test features:
DISPLAY=:10 xterm &  # Test window creation
DISPLAY=:10 xlogo &  # Test multiple windows

# Test keyboard shortcuts:
# Alt+j/k - focus navigation
# Shift+Alt+j/k - window swapping
# Shift+Alt+m - swap with master
# Shift+Alt+q - destroy window
```

**Manual Testing Scripts:**
```bash
# Test rustile in isolated environment
./test.sh

# Check code quality  
./check.sh
```

### 📋 Commit Guidelines

Follow [Conventional Commits](https://conventionalcommits.org/) with automated versioning:

```
<type>: <description>

🤖 Generated with [Claude Code](https://claude.ai/code)

Co-Authored-By: Claude <noreply@anthropic.com>
```

**Types:**
- `feat`: New feature (triggers MINOR version bump)
- `fix`: Bug fix (triggers PATCH version bump)
- `docs`: Documentation changes
- `style`: Code style changes (formatting, etc.)
- `refactor`: Code refactoring without feature changes
- `test`: Adding or updating tests
- `chore`: Build process, dependencies, tooling

**Examples:**
```bash
git commit -m "feat: implement window swapping with Shift+Alt+j/k shortcuts

🤖 Generated with [Claude Code](https://claude.ai/code)

Co-Authored-By: Claude <noreply@anthropic.com>"

git commit -m "fix: resolve keyboard shortcut matching bug for modifier combinations

🤖 Generated with [Claude Code](https://claude.ai/code)

Co-Authored-By: Claude <noreply@anthropic.com>"
```

### 🔄 Automated Release Process

**This project uses fully automated semantic versioning:**

1. **Push to main** → GitHub Actions analyzes commits
2. **Version determination** → Based on conventional commit types
3. **Automated updates** → `Cargo.toml`, `CHANGELOG.md`, git tags
4. **Release creation** → GitHub release with binaries
5. **Commit back** → Updated files committed with `[skip ci]`

**IMPORTANT:** Never manually update versions - it's automated!

## 🔮 Future Architecture Considerations

### 🌐 Multi-Monitor Support

```rust
// Planned architecture for multi-monitor:
pub struct MultiMonitorManager {
    monitors: Vec<Monitor>,
    window_assignments: HashMap<Window, MonitorId>,
    layout_managers: HashMap<MonitorId, LayoutManager>,
}

impl MultiMonitorManager {
    fn handle_window_movement(&mut self, window: Window, target_monitor: MonitorId) {
        // 1. Remove from current monitor
        if let Some(current_monitor) = self.window_assignments.get(&window) {
            self.layout_managers.get_mut(current_monitor)
                .unwrap()
                .remove_window(window);
        }
        
        // 2. Add to target monitor  
        self.layout_managers.get_mut(&target_monitor)
            .unwrap()
            .add_window(window);
            
        // 3. Update assignment
        self.window_assignments.insert(window, target_monitor);
        
        // 4. Refresh both monitor layouts
        self.apply_layout_to_monitor(current_monitor);
        self.apply_layout_to_monitor(target_monitor);
    }
}
```

### 🔌 Plugin Architecture

```rust
// Extensible plugin system design:
pub trait WindowManagerPlugin {
    fn name(&self) -> &str;
    fn on_window_created(&mut self, window: Window, manager: &mut WindowManager);
    fn on_window_destroyed(&mut self, window: Window, manager: &mut WindowManager);
    fn on_layout_changed(&mut self, layout: Layout, manager: &mut WindowManager);
    fn on_window_swapped(&mut self, from: Window, to: Window, manager: &mut WindowManager);
}

// Example plugins:
struct StatusBarPlugin { /* ... */ }
struct NotificationPlugin { /* ... */ }  
struct WorkspacePlugin { /* ... */ }
struct WindowHistoryPlugin { /* ... */ }  // Track window operations

impl WindowManagerPlugin for StatusBarPlugin {
    fn on_window_created(&mut self, window: Window, manager: &mut WindowManager) {
        // Update status bar with new window count
        self.update_window_count(manager.windows.len());
    }
    
    fn on_window_swapped(&mut self, from: Window, to: Window, manager: &mut WindowManager) {
        // Show temporary notification of swap operation
        self.show_swap_notification(from, to);
    }
}
```

### 🚀 Enhanced Window Operations

```rust
// Future window operation extensions:
pub enum WindowOperation {
    Swap { from: Window, to: Window },
    Move { window: Window, to_workspace: WorkspaceId },
    Resize { window: Window, direction: ResizeDirection },
    Float { window: Window, toggle: bool },
    Minimize { window: Window },
    Maximize { window: Window, toggle: bool },
}

pub struct WindowOperationHistory {
    operations: VecDeque<WindowOperation>,
    max_history: usize,
}

impl WindowOperationHistory {
    pub fn undo_last_operation(&mut self, manager: &mut WindowManager) -> Result<()> {
        if let Some(operation) = self.operations.pop_back() {
            match operation {
                WindowOperation::Swap { from, to } => {
                    // Reverse the swap
                    manager.swap_specific_windows(to, from)?;
                },
                // Handle other operation reversals...
            }
        }
        Ok(())
    }
}
```

### 🌐 Advanced Layout Algorithms

```rust
// Future layout implementations:
pub enum Layout {
    MasterStack,
    Bsp,
    Grid,           // Regular grid arrangement
    Spiral,         // Fibonacci spiral layout
    ThreeColumn,    // Master + two stacks
    Floating,       // Traditional floating windows
    Custom(Box<dyn LayoutAlgorithm>),  // User-defined layouts
}

// Grid layout example:
pub struct GridLayout {
    columns: usize,
    rows: usize,
    auto_adjust: bool,  // Automatically adjust grid size
}

impl LayoutAlgorithm for GridLayout {
    fn apply_layout(&mut self, windows: &[Window], screen: Rectangle) -> Vec<WindowGeometry> {
        let (cols, rows) = if self.auto_adjust {
            self.calculate_optimal_grid(windows.len())
        } else {
            (self.columns, self.rows)
        };
        
        // Calculate cell size
        let cell_width = screen.width / cols as u32;
        let cell_height = screen.height / rows as u32;
        
        windows.iter().enumerate().map(|(i, _)| {
            let col = i % cols;
            let row = i / cols;
            
            WindowGeometry {
                x: screen.x + (col as u32 * cell_width),
                y: screen.y + (row as u32 * cell_height),
                width: cell_width,
                height: cell_height,
            }
        }).collect()
    }
}
```

### 🔗 IPC (Inter-Process Communication) Interface

```rust
// Future runtime configuration changes:
pub struct IpcServer {
    socket_path: PathBuf,
    listener: UnixListener,
}

#[derive(Serialize, Deserialize)]
pub enum IpcCommand {
    SetMasterRatio(f32),
    SetGap(u32),
    SetLayout(Layout),
    SwapWindows { from: Window, to: Window },
    GetWindowList,
    GetCurrentLayout,
    ReloadConfig,
}

#[derive(Serialize, Deserialize)]
pub enum IpcResponse {
    Success,
    Error(String),
    WindowList(Vec<WindowInfo>),
    CurrentLayout(Layout),
}

impl IpcServer {
    pub fn handle_command(&self, cmd: IpcCommand, wm: &mut WindowManager) -> IpcResponse {
        match cmd {
            IpcCommand::SetMasterRatio(ratio) => {
                if ratio >= 0.0 && ratio <= 1.0 {
                    wm.config.set_master_ratio(ratio);
                    wm.apply_layout().map(|_| IpcResponse::Success)
                        .unwrap_or_else(|e| IpcResponse::Error(e.to_string()))
                } else {
                    IpcResponse::Error("Master ratio must be between 0.0 and 1.0".to_string())
                }
            },
            // Handle other commands...
        }
    }
}
```

---

## 📚 Conclusion

This technical deep dive reveals the sophisticated engineering behind Rustile's simple interface. The combination of:

- **Rust's Memory Safety** - Eliminates entire classes of window manager bugs
- **Efficient Algorithms** - O(n) layout calculations with caching opportunities
- **Clean Architecture** - Modular design enabling easy feature additions
- **Robust Error Handling** - Graceful degradation and clear error messages
- **Comprehensive Testing** - Unit, integration, and manual testing strategies
- **Automated Quality** - CI/CD pipeline ensuring code quality and releases

Makes Rustile both **performant and maintainable**, providing a solid foundation for future window management innovations.

**Key Technical Achievements:**
1. ✅ **Critical Bug Fix** - Exact keyboard modifier matching
2. ✅ **Window Swapping** - Comprehensive positional exchange operations
3. ✅ **Graceful Termination** - WM_DELETE_WINDOW protocol with forceful fallback
4. ✅ **Robust Configuration** - Validation with helpful error messages
5. ✅ **Modular Architecture** - Easy to extend with new layouts and features

*For user-friendly documentation, see [BEGINNER_GUIDE.md](BEGINNER_GUIDE.md)*  
*For development guidelines, see [CLAUDE.md](../CLAUDE.md)*  
*For future plans, see [ROADMAP.md](ROADMAP.md)*