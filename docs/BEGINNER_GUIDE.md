# 🪟 Rustile Window Manager - Beginner's Guide

## 📚 What is a Window Manager?

A **window manager** is a program that controls how application windows appear and behave on your screen. Instead of windows appearing randomly, a **tiling window manager** automatically arranges them in organized patterns.

### 🖥️ Traditional vs Tiling Window Managers

```
Traditional (Floating) Windows:        Tiling Window Manager:
+----------------------------------+   +----------------------------------+
|                                  |   |                                  |
|  +-------+                       |   | +-----------+ +------------------+|
|  | App1  |   +----------+        |   | |           | |                  ||
|  |       |   |  App2    |        |   | |   App1    | |      App2        ||
|  +-------+   |          |        |   | | (Master)  | |    (Stack)       ||
|               +----------+        |   | |           | |                  ||
|     +---------------+            |   | +-----------+ +------------------+|
|     |    App3      |             |   |               |                  ||
|     |  (hidden)    |             |   |               |      App3        ||
|     +---------------+            |   |               |    (Stack)       ||
|                                  |   |               |                  ||
+----------------------------------+   +-------------------------------+---+
```

**Problems with Floating:**
- Windows overlap and hide each other
- You waste time moving/resizing windows
- Hard to see all applications at once

**Benefits of Tiling:**
- Every window is visible
- No manual positioning needed
- Efficient use of screen space
- Keyboard-driven workflow

## 🏗️ How Rustile Works

### 🎯 Core Concept: X11 Protocol

Rustile communicates with your desktop using **X11**, a protocol that manages graphics on Linux:

```
+-------------------------------------------------------------+
|                    Your Desktop (X11 Server)                |
|                                                             |
|  +-----------+ +-----------+ +-----------+                  |
|  |   xterm   | |  chrome   | |   code    | <- Applications  |
|  | (window)  | | (window)  | | (window)  |                  |
|  +-----------+ +-----------+ +-----------+                  |
|        ^             ^             ^                        |
|        |             |             |                        |
|        +-------------+-------------+                        |
|                      |                                      |
|                      v                                      |
|              +----------------+                             |
|              |    Rustile     |                             |
|              | (Window Manager)| <- Controls positions      |
|              +----------------+    and sizes                |
+-------------------------------------------------------------+
```

**What happens:**
1. Applications create windows
2. X11 tells Rustile "new window appeared!"
3. Rustile calculates where to put it
4. Rustile tells X11 "move window to position (x,y) with size (w,h)"
5. X11 moves the window

### 🔄 Event Loop - The Heart of Rustile

Rustile runs in a continuous loop, waiting for events:

```
Start Rustile
     |
     v
+-------------------------------------------------------------+
|                    MAIN EVENT LOOP                          |
|                                                             |
|  +--- Wait for Event <---------------------------------+    |
|  |                                                     |    |
|  v                                                     |    |
| Event Received                                         |    |
|  |                                                     |    |
|  +--- Key Press? --> Handle Keyboard Shortcut --------+    |
|  |                   (focus_next, swap_master, etc.)  |    |
|  |                                                     |    |
|  +--- New Window? -> Add to window list --------------+    |
|  |                   Calculate layout                  |    |
|  |                   Position window                   |    |
|  |                                                     |    |
|  +--- Window Closed? > Remove from list --------------+    |
|  |                     Recalculate layout             |    |
|  |                                                     |    |
|  +--- Mouse Click? -> Update focus -------------------+    |
|                                                             |
+-------------------------------------------------------------+
```

## 🏗️ Layout Algorithms

Rustile supports two tiling patterns:

### 📐 Master-Stack Layout

The most common tiling pattern:

```
                    Screen (1920x1080)
    +-------------------------------------------------------------+
    | gap                                                     gap |
    | +---------------------------+ gap +---------------------+   |
    | |                           |     |                     |   |
    | |                           |     |      Stack 1        |   |
    | |                           |     |                     |   |
    | |         Master            |     +---------------------+   |
    | |       (50% width)         |     | gap                 |   |
    | |                           |     |      Stack 2        |   |
    | |                           |     |                     |   |
    | |                           |     +---------------------+   |
    | |                           |     | gap                 |   |
    | |                           |     |      Stack 3        |   |
    | |                           |     |                     |   |
    | +---------------------------+     +---------------------+   |
    |                                                         gap |
    +-------------------------------------------------------------+
     ^                           ^
     |                           |
   Master window takes         Stack windows share
   master_ratio (50%)          remaining space equally
   of screen width
```

**How it works:**
- **Master**: First window gets left side (configurable width ratio)
- **Stack**: Additional windows stack vertically on the right
- **Focus**: Red border shows which window receives keyboard input

### 🌳 BSP (Binary Space Partitioning) Layout

More complex but flexible pattern:

```
Step 1: First window          Step 2: Add second window
+-------------------------+   +-------------------------+
|                         |   |           |             |
|                         |   |           |             |
|          App1           |   |   App1    |    App2     |
|         (root)          |   |  (left)   |   (right)   |
|                         |   |           |             |
|                         |   |           |             |
+-------------------------+   +-------------------------+

Step 3: Add third window      Step 4: Add fourth window
+-------------------------+   +-------------------------+
|           |             |   |           |      |      |
|           |    App2     |   |           | App2 | App4 |
|   App1    +-------------+   |   App1    +------+------+
|  (left)   |             |   |  (left)   |      |      |
|           |    App3     |   |           | App3 |      |
|           |             |   |           |      |      |
+-------------------------+   +-------------------------+
```

**How it works:**
- Each new window **splits** an existing window's space
- Creates a **binary tree** structure
- **Alternates** between vertical and horizontal splits
- Very flexible but more complex to understand

## 🧠 Rustile's Brain - The Code Structure

### 📁 File Organization

```
rustile/
├── src/
│   ├── main.rs                 # Program entry point
│   ├── lib.rs                  # Library root
│   │
│   ├── window_manager/         # The main controller
│   │   ├── mod.rs              # Window manager interface
│   │   ├── core.rs             # Initialization & main loop
│   │   ├── events.rs           # Handles X11 events
│   │   ├── focus.rs            # Which window is active
│   │   └── window_ops.rs       # Window operations
│   │
│   ├── layout/                 # How windows are arranged
│   │   ├── mod.rs              # Layout system interface
│   │   ├── manager.rs          # Coordinates layouts
│   │   ├── master_stack.rs     # Master-stack algorithm
│   │   ├── bsp.rs              # BSP algorithm
│   │   ├── types.rs            # Data structures
│   │   ├── traits.rs           # Layout interfaces
│   │   └── constants.rs        # Magic numbers
│   │
│   ├── config/                 # User settings
│   │   ├── mod.rs              # Configuration main
│   │   └── validation.rs       # Config validation
│   │
│   ├── keyboard.rs             # Keyboard shortcuts
│   └── keys.rs                 # Key parsing
└── config.example.toml         # User configuration
```

### 🔧 Data Flow Diagram

```
User presses key (Alt+j)
        |
        v
+-------------------------------------------------------------+
|                    events.rs                                |
|  +--- Key Press Event -> Parse Shortcut -> Match Command --+|
|  |                                                         ||
|  |    "Alt+j" → "focus_next"                              ||
|  +---------------------------------------------------------+|
+------------------------------+------------------------------+
                               |
                               v
+-------------------------------------------------------------+
|                    focus.rs                                 |
|  +--- focus_next() -> Find next window -> Update focus ----+|
|  |                                                         ||
|  |   current: window_2  →  next: window_3                 ||
|  |   set red border on window_3                           ||
|  +---------------------------------------------------------+|
+------------------------------+------------------------------+
                               |
                               v
+-------------------------------------------------------------+
|                 window_ops.rs                               |
|  +--- apply_layout() -> Call layout manager ---------------+|
|  |                                                         ||
|  |   Trigger visual update of all windows                 ||
|  +---------------------------------------------------------+|
+------------------------------+------------------------------+
                               |
                               v
+-------------------------------------------------------------+
|                layout/manager.rs                            |
|  +--- Choose layout algorithm -> Calculate positions ------+|
|  |                                                         ||
|  |   master_stack OR bsp                                  ||
|  +---------------------------------------------------------+|
+------------------------------+------------------------------+
                               |
                               v
+-------------------------------------------------------------+
|            layout/master_stack.rs OR layout/bsp.rs          |
|  +--- Calculate window positions and sizes ----------------+|
|  |                                                         ||
|  |   window_1: x=0,   y=0,   w=960,  h=1080              ||
|  |   window_2: x=960, y=0,   w=960,  h=540               ||
|  |   window_3: x=960, y=540, w=960,  h=540               ||
|  +---------------------------------------------------------+|
+------------------------------+------------------------------+
                               |
                               v
+-------------------------------------------------------------+
|                        X11                                  |
|  +--- Move windows to calculated positions ----------------+|
|  |                                                         ||
|  |   User sees windows rearrange on screen                ||
|  +---------------------------------------------------------+|
+-------------------------------------------------------------+
```

## 🧩 Key Rust Concepts Used

### 📦 Structs - Data Containers

```rust
// Like a container that holds related data
pub struct WindowManager<C: Connection> {
    conn: C,                    // Connection to X11
    windows: Vec<Window>,       // List of all windows
    focused_window: Option<Window>, // Which window is active
    config: Config,            // User settings
    layout_manager: LayoutManager, // How to arrange windows
}
```

**Think of it like:**
```
WindowManager = {
    📡 X11 connection
    📝 List of windows: [window1, window2, window3]
    🎯 Currently focused: window2
    ⚙️  User settings: gaps=10px, master_ratio=0.5
    📐 Layout calculator
}
```

### 🔄 Enums - Multiple Choices

```rust
// Like a multiple choice question - it can be ONE of these options
pub enum Layout {
    MasterStack,  // Option A: Use master-stack layout
    Bsp,          // Option B: Use BSP layout
}
```

**Visual representation:**
```
Layout = MasterStack  →  +-------------+ +-------+
                         |             | |   2   |
                         |      1      | +-------+
                         |             | |   3   |
                         +-------------+ +-------+

Layout = Bsp         →   +-------+-------+
                         |   1   |   2   |
                         +-------+-------+
                         |   3   |   4   |
                         +-------+-------+
```

### 🏪 Traits - Contracts

```rust
// Like a contract: "Any layout algorithm MUST implement these functions"
pub trait LayoutAlgorithm {
    fn name(&self) -> &'static str;
    fn add_window(&mut self, window: Window, ...);
    fn remove_window(&mut self, window: Window);
    fn apply_layout(&mut self, ...);
}
```

**Why this is useful:**
```
Master-Stack Algorithm implements LayoutAlgorithm
BSP Algorithm implements LayoutAlgorithm
Future Spiral Algorithm implements LayoutAlgorithm

→ All can be used interchangeably!
→ Easy to add new layout types
→ Code stays organized
```

### 🗂️ Modules - Code Organization

```rust
// Like folders for organizing code
mod window_manager {
    mod core;      // Main logic
    mod events;    // Event handling
    mod focus;     // Focus management
}

mod layout {
    mod manager;      // Layout coordination
    mod master_stack; // Master-stack algorithm
    mod bsp;         // BSP algorithm
}
```

## ⚙️ Configuration System

### 📝 TOML Configuration File

```toml
# ~/.config/rustile/config.toml
[layout]
layout_algorithm = "master_stack"  # Which layout to use
master_ratio = 0.6                # Master window takes 60% of width
gap = 15                          # 15 pixels between windows
border_width = 2                  # 2 pixel window borders

[shortcuts]
"Alt+j" = "focus_next"            # Move focus to next window
"Alt+k" = "focus_prev"            # Move focus to previous window
"Shift+Alt+m" = "swap_with_master" # Swap focused with master
"Shift+Alt+1" = "xterm"           # Launch terminal
```

### 🎨 Visual Settings

```
border_width = 3, gap = 10:

+-------------------------------------------------------------+
| 10px gap from screen edge                                   |
|  +=======================+ 10px +=======================+  |
|  ‖ 3px red border        ‖ gap  ‖ 3px gray border       ‖  |
|  ‖ +-------------------+ ‖      ‖ +-------------------+ ‖  |
|  ‖ |                   | ‖      ‖ |                   | ‖  |
|  ‖ |  Focused Window   | ‖      ‖ | Unfocused Window  | ‖  |
|  ‖ |                   | ‖      ‖ |                   | ‖  |
|  ‖ +-------------------+ ‖      ‖ +-------------------+ ‖  |
|  +=======================+      +=======================+  |
|                                                             |
+-------------------------------------------------------------+
```

## 🔄 Common Operations Explained

### 1️⃣ Adding a New Window

```
Step 1: Application starts (e.g., user runs "xterm")
       +-------------+
       |    xterm    | --> X11: "I need a window!"
       +-------------+

Step 2: X11 notifies Rustile
       +-------------+
       |     X11     | --> Rustile: "New window created: ID 12345"
       +-------------+

Step 3: Rustile adds to its window list
       Before: windows = [101, 102, 103]
       After:  windows = [101, 102, 103, 12345]

Step 4: Recalculate layout
       +-------------------------------------+
       | Master-Stack Layout Calculator      |
       |                                     |
       | 4 windows total:                    |
       | • Master (101): 50% width, full height
       | • Stack (102): 50% width, 1/3 height 
       | • Stack (103): 50% width, 1/3 height
       | • Stack (12345): 50% width, 1/3 height
       +-------------------------------------+

Step 5: Apply new positions
       +-------------------------------------+
       | +-------------+ +-----------------+ |
       | |             | |      102        | |
       | |     101     | +-----------------+ |
       | |  (Master)   | |      103        | |
       | |             | +-----------------+ |
       | |             | |    12345 (new)  | |
       | +-------------+ +-----------------+ |
       +-------------------------------------+
```

### 2️⃣ Focus Navigation (Alt+j)

```
Current state: windows = [101, 102, 103], focused = 102

Step 1: User presses Alt+j
       Keyboard --> Rustile: "focus_next command"

Step 2: Find next window
       Current index: 1 (102 is at position 1)
       Next index: 2 (wrap around if at end)
       Next window: 103

Step 3: Update focus
       Before: focused_window = Some(102)
       After:  focused_window = Some(103)

Step 4: Update visual borders
       +-------------------------------------+
       | +-------------+ +-----------------+ |
       | #     101     # #      102        # | <- Gray borders
       | #             # +=================+ |
       | #             # ‖      103        ‖ | <- Red border (focused)
       | #             # ‖                 ‖ |
       | #             # ‖                 ‖ |
       | +-------------+ +=================+ |
       +-------------------------------------+
```

### 3️⃣ Swap with Master (Shift+Alt+m)

```
Current state: windows = [101, 102, 103], focused = 103

Step 1: User presses Shift+Alt+m
       Keyboard --> Rustile: "swap_with_master command"

Step 2: Find focused window position
       Focused window: 103 (at index 2)
       Master position: index 0

Step 3: Swap in window list
       Before: windows = [101, 102, 103]
       After:  windows = [103, 102, 101]

Step 4: Recalculate and apply layout
       +-------------------------------------+
       | +-------------+ +-----------------+ |
       | ‖     103     ‖ #      102        # | <- 103 now master
       | ‖ (New Master)‖ +-----------------+ |   (with focus)
       | ‖             ‖ #      101        # |
       | ‖             ‖ #                 # |
       | ‖             ‖ #                 # |
       | +============-+ +-----------------+ |
       +-------------------------------------+
```

## 🐛 Debugging and Troubleshooting

### 🔍 Log Messages

Rustile produces helpful log messages:

```bash
# Run with debug logging
RUST_LOG=debug cargo run

# Example output:
INFO  rustile::window_manager::events - New window mapped: 16777225
DEBUG rustile::layout::manager - Applied layout to 3 windows
INFO  rustile::window_manager::focus - Focused next window: Some(16777226)
DEBUG rustile::layout::bsp - BSP: Adding window 16777227 targeting Some(16777226)
```

### 🛠️ Test Environment

```bash
# Start test environment
./scripts/dev-tools.sh layout

# This creates:
+-------------------------------------------------------------+
|  Xephyr :10 (Nested X Server)                              |
|  +---------------------------------------------------------+|
|  |              Test Desktop (:10)                         ||
|  |                                                         ||
|  |  +-------------+ +-------------------+                  ||
|  |  |             | |                   |                  ||
|  |  |   xterm     | |     xlogo        | <- Test windows  ||
|  |  |             | |                   |                  ||
|  |  |             | |                   |                  ||
|  |  +-------------+ +-------------------+                  ||
|  +---------------------------------------------------------+|
+-------------------------------------------------------------+
```

## 🎯 Next Steps for Learning

### 🔧 Try These Modifications

1. **Change Master Ratio:**
   ```toml
   # In ~/.config/rustile/config.toml
   master_ratio = 0.7  # Master takes 70% instead of 50%
   ```

2. **Add Custom Shortcut:**
   ```toml
   [shortcuts]
   "Super+t" = "xterm"  # Windows key + t opens terminal
   ```

3. **Experiment with Gaps:**
   ```toml
   gap = 20           # Larger gaps
   border_width = 1   # Thinner borders
   ```

### 📚 Code Reading Path

1. **Start here:** `src/main.rs` - See how the program starts
2. **Then:** `src/window_manager/core.rs` - Understand the main loop
3. **Next:** `src/window_manager/events.rs` - See how events are handled
4. **Finally:** `src/layout/master_stack.rs` - Understand layout math

### 🧪 Experiment Ideas

1. **Add a new layout algorithm**
2. **Create custom keyboard shortcuts**
3. **Implement window decorations**
4. **Add multi-monitor support**

## 📖 Glossary

| Term | Definition | Visual Example |
|------|------------|----------------|
| **Window** | A rectangular area where an application displays its content | `+-----+`<br>`| App |`<br>`+-----+` |
| **Focus** | Which window receives keyboard input (shown with red border) | `+=====+` <- Focused<br>`‖ App ‖`<br>`+=====+` |
| **Master** | The main window (usually largest) in master-stack layout | `+-------+ +---+`<br>`|Master | |Stk|`<br>`+-------+ +---+` |
| **Stack** | Secondary windows arranged vertically | `+---+ +---+`<br>`|Mst| |St1|`<br>`+---+ +---+`<br>`      |St2|`<br>`      +---+` |
| **Layout** | The algorithm used to arrange windows | Master-Stack vs BSP |
| **BSP** | Binary Space Partitioning - recursive window splitting | `+---+---+`<br>`| 1 | 2 |`<br>`+---+---+`<br>`| 3 | 4 |`<br>`+---+---+` |
| **Event** | A message from X11 (key press, new window, etc.) | User presses key → Event → Action |
| **X11** | The graphics system on Linux that manages windows | The "messenger" between apps and window manager |

---

🎉 **Congratulations!** You now understand how Rustile works from the ground up. The combination of Rust's safety, X11's flexibility, and tiling algorithms creates an efficient window management system.