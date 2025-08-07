# Implementation Details

This document explains the crucial implementation details of rustile that are hard to understand without explanation.

## Table of Contents

1. [Window Mapping Process](#window-mapping-process)
2. [BSP Tree Algorithm](#bsp-tree-algorithm)
3. [Focus Management & Borders](#focus-management--borders)
4. [Event Loop & Event Types](#event-loop--event-types)
5. [Keyboard: Keycodes vs Keysyms](#keyboard-keycodes-vs-keysyms)

## Window Mapping Process

When a new window appears (like when you run `xterm`), here's exactly what happens:

### Step 1: X11 Sends MapRequest

```rust
// From src/window_manager.rs
Event::MapRequest(ev) => self.handle_map_request(ev),
```

The X server asks us: "Can I show this window?" We MUST handle this - we're the window manager!

### Step 2: The Complete Mapping Flow

```rust
// From src/window_manager.rs
fn handle_map_request(&mut self, event: MapRequestEvent) -> Result<()> {
    let window = event.window;
    
    // 1. Set border BEFORE mapping (important!)
    self.configure_window_border(window, self.window_state.unfocused_border_color())?;
    
    // 2. Tell X11 to actually show the window
    self.conn.map_window(window)?;
    
    // 3. Add to our BSP tree (determines position)
    self.window_state.add_window_to_layout(window);
    
    // 4. Focus the new window
    self.window_renderer.set_focus(&mut self.conn, &mut self.window_state, window)?;
    
    // 5. Calculate positions and apply
    self.window_renderer.apply_layout(&mut self.conn, &mut self.window_state)?;
}
```

### Visual Flow:

```text
MapRequest arrives
    ↓
Set border (visual feedback)
    ↓
Map window (make visible)
    ↓
Add to BSP tree ──→ ┌─────────┐      ┌─────┬─────┐
                    │ Desktop │  →   │ Old │ New │
                    └─────────┘      └─────┴─────┘
    ↓
Set focus (keyboard input)
    ↓
Apply layout (position windows)
```

### Why This Order Matters:

1. **Border before mapping**: Window appears with correct border immediately
2. **Map before layout**: Window must exist before we can position it
3. **Focus after adding**: Can only focus windows we manage
4. **Layout last**: Positions all windows with new one included

## BSP Tree Algorithm

The Binary Space Partitioning tree is how rustile decides where to place windows.

### The Tree Structure

```rust
// From src/bsp.rs
pub enum BspTree {
    Empty,
    Leaf(Window),
    Node {
        split: SplitDirection,
        left: Box<BspTree>,
        right: Box<BspTree>,
    },
}
```

### How Windows Are Added

```rust
// From src/bsp.rs - simplified
pub fn add_window(&mut self, new_window: Window, focused: Option<Window>, split_ratio: f32) {
    match self {
        BspTree::Empty => {
            // First window - just make it a leaf
            *self = BspTree::Leaf(new_window);
        }
        BspTree::Leaf(existing) => {
            // Second window - create a split
            let split = self.next_split_direction(None);
            *self = BspTree::Node {
                split,
                left: Box::new(BspTree::Leaf(*existing)),
                right: Box::new(BspTree::Leaf(new_window)),
            };
        }
        BspTree::Node { .. } => {
            // Find focused window and split at that location
            // (recursive logic)
        }
    }
}
```

### Visual Examples:

#### Adding First Window:
```text
Empty           Leaf(Window1)
  ○      →         W1
```

#### Adding Second Window:
```text
Leaf(W1)              Node(Horizontal)
                          /        \
  W1      →         Leaf(W1)    Leaf(W2)

Screen:         ┌─────────┬─────────┐
                │   W1    │   W2    │
                └─────────┴─────────┘
```

#### Adding Third Window (focused on W2):
```text
     Node(H)                        Node(H)
    /        \                     /        \
Leaf(W1)   Leaf(W2)    →     Leaf(W1)    Node(V)
                                        /        \
                                   Leaf(W2)   Leaf(W3)

Screen:  ┌─────────┬─────────┐    ┌─────────┬─────────┐
         │   W1    │   W2    │ →  │   W1    │   W2    │
         └─────────┴─────────┘    │         ├─────────┤
                                  │         │   W3    │
                                  └─────────┴─────────┘
```

### Split Direction Alternation

```rust
// From src/bsp.rs
fn next_split_direction(&self, current: Option<SplitDirection>) -> SplitDirection {
    match current {
        Some(SplitDirection::Horizontal) => SplitDirection::Vertical,
        Some(SplitDirection::Vertical) => SplitDirection::Horizontal,
        None => SplitDirection::Horizontal, // Default
    }
}
```

This ensures windows don't get too thin/tall by alternating split directions.

### Geometry Calculation

```rust
// From src/window_state.rs
pub fn calculate_window_geometries(&self, screen_width: u16, screen_height: u16) -> Vec<WindowGeometry> {
    let params = LayoutParams {
        min_window_width: self.config.min_window_width(),
        min_window_height: self.config.min_window_height(),
        gap: self.config.gap(),
    };
    
    calculate_bsp_geometries(&self.bsp_tree, screen_width, screen_height, params)
}
```

The actual calculation recursively divides screen space according to the tree structure.

## Focus Management & Borders

Focus in X11 is complex - we need both X11 input focus AND visual indication.

### Setting Focus

```rust
// From src/window_renderer.rs
pub fn set_focus(&mut self, conn: &mut C, state: &mut WindowState, window: Window) -> Result<()> {
    // 1. Set X11 input focus (keyboard goes here)
    conn.set_input_focus(InputFocus::POINTER_ROOT, window, CURRENT_TIME)?;
    
    // 2. Update our state
    state.set_focused_window(Some(window));
    
    // 3. Update ALL window borders (visual feedback)
    self.update_focus_borders(conn, state)?;
}
```

### Why Both X11 Focus and Visual Borders?

```text
X11 Focus Only:              Our Visual Borders:
┌─────────┐ ┌─────────┐     ┌═════════┐ ┌─────────┐
│ Window1 │ │ Window2 │     ║ Window1 ║ │ Window2 │
└─────────┘ └─────────┘     ╚═════════╝ └─────────┘
    ↑                              ↑
(keyboard input goes here)   (user can SEE this!)
```

### Border Color Logic

```rust
// From src/window_state.rs
pub fn border_color_for_window(&self, window: Window) -> u32 {
    if Some(window) == self.focused_window {
        self.config.focused_border_color()   // Red (0xFF0000)
    } else {
        self.config.unfocused_border_color() // Gray (0x808080)
    }
}
```

### Focus Following Mouse

```rust
// From src/window_manager.rs
Event::EnterNotify(ev) => {
    let window = ev.event;
    // Only focus if it's a window we manage
    if self.window_state.has_window(window) {
        self.window_renderer.set_focus(&mut self.conn, &mut self.window_state, window)?;
    }
}
```

When mouse enters a window, we automatically focus it.

## Event Loop & Event Types

The event loop is the heart of the window manager:

### The Main Loop

```rust
// From src/window_manager.rs
pub fn run(mut self) -> Result<()> {
    loop {
        self.conn.flush()?;                    // Send any pending requests
        let event = self.conn.wait_for_event()?; // BLOCKS until event
        self.handle_event(event)?;              // Process it
    }
}
```

This runs FOREVER until the WM is killed.

### Critical Event Types

#### MapRequest vs MapNotify
- **MapRequest**: "I want to show a window" (we must approve)
- **MapNotify**: "A window was shown" (just information)

We only handle MapRequest because we're the manager!

#### UnmapNotify vs DestroyNotify

```rust
// Window hidden (might come back)
Event::UnmapNotify(ev) => {
    // Remove from layout but window still exists
    self.window_state.remove_window_from_layout(window);
}

// Window destroyed (gone forever)
Event::DestroyNotify(ev) => {
    // Remove from layout AND clean up any references
    self.window_state.remove_window_from_layout(window);
    self.window_state.remove_intentionally_unmapped(window);
}
```

The difference matters for handling fullscreen and window lifecycle!

### Event Flow Diagram

```text
X Server Event Queue
        ↓
wait_for_event() [BLOCKS]
        ↓
Event arrives
        ↓
handle_event() match
        ↓
    ├─ MapRequest ──→ handle_map_request()
    ├─ KeyPress ────→ handle_key_press()
    ├─ UnmapNotify ─→ handle_unmap_notify()
    └─ ... etc
        ↓
Update state/render
        ↓
conn.flush() [send to X]
        ↓
Back to waiting...
```

## Keyboard: Keycodes vs Keysyms

This is one of the most confusing parts of X11!

### The Problem

```text
Physical Key: [A key]
    ↓
Keycode: 38 (hardware scancode)
    ↓
Keysym: 'a' or 'A' (meaning)
```

The same physical key produces different keysyms based on modifiers!

### How Rustile Handles This

```rust
// From src/keyboard.rs
pub struct ShortcutManager {
    keyname_to_keysym: HashMap<String, u32>,
    keysym_to_keycode: HashMap<u32, u8>,
    shortcuts: Vec<Shortcut>,
}

// During initialization
fn init_keycode_map(conn: &impl Connection, setup: &Setup) -> HashMap<Keycode, Vec<Keysym>> {
    let mapping = conn.get_keyboard_mapping(
        setup.min_keycode,
        setup.max_keycode - setup.min_keycode + 1,
    )?;
    
    // Build keycode → keysyms mapping
    // Each keycode can have multiple keysyms (normal, shifted, etc.)
}
```

### Registering Shortcuts

```rust
// From config: "Alt+j" = "focus_next"

// 1. Parse the string
let (modifiers, key) = parse_key_combination("Alt+j")?;

// 2. Convert 'j' to keysym
let keysym = string_to_keysym("j"); // Returns XK_j

// 3. Find which keycode produces this keysym
let keycode = self.keysym_to_keycode(keysym)?;

// 4. Register with X11
conn.grab_key(
    true,
    root,
    modifiers,
    keycode,  // X11 wants keycode, not keysym!
    GrabMode::ASYNC,
    GrabMode::ASYNC,
)?;
```

### Why Both?

- **Keycodes**: What X11 sends us in events (hardware level)
- **Keysyms**: What humans understand ('a', 'Return', 'F1')
- **Config uses keysyms**: `"Alt+j"` not `"Alt+44"`
- **X11 wants keycodes**: For grab_key() calls

The conversion is necessary but confusing!