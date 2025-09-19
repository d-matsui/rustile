# How Rustile Works

This guide explains how rustile functions as an X11 window manager and its architecture.

## Table of Contents

1. [X11 and Window Managers](#x11-and-window-managers)
2. [How Rustile Becomes the Window Manager](#how-rustile-becomes-the-window-manager)
3. [Architecture Overview](#architecture-overview)
4. [Event Flow](#event-flow)

## X11 and Window Managers

### What is X11?

X11 is a protocol that separates applications from display hardware:

```text
┌─────────────┐         ┌─────────────┐         ┌─────────────┐
│   X Client  │ ←────→  │  X Server   │ ←────→  │   Display   │
│   (xterm)   │ network │  (manages   │         │  (monitor)  │
└─────────────┘ protocol│  graphics)  │         └─────────────┘
                        └─────────────┘
```

- **X Clients**: Applications like xterm, firefox, etc.
- **X Server**: Manages the display, handles drawing commands
- **Network Protocol**: Clients and server can be on different machines!

### Where Does a Window Manager Fit?

The X server handles graphics but NOT window policies (position, size, decorations). That's the window manager's job:

```text
                        ┌─────────────┐
                        │  X Server   │
                        └─────────────┘
                               ↑
                               │ manages windows for
                        ┌─────────────┐
                        │Window Manager│ ← "I decide where windows go"
                        │  (rustile)   │
                        └─────────────┘
                               ↑
                    ┌──────────┴──────────┐
                    ↓                     ↓
             ┌─────────────┐       ┌─────────────┐
             │   xterm     │       │   firefox   │
             └─────────────┘       └─────────────┘
```

The window manager:

- Receives events about new windows (MapRequest)
- Decides where to place windows
- Handles user input (Alt+Tab, etc.)
- Manages focus (which window receives keyboard input)

## How Rustile Becomes the Window Manager

When rustile starts (`src/main.rs`):

```rust
// 1. Connect to X11 server
let (conn, screen_num) = x11rb::connect(None)?;

// 2. Create WindowManager
let wm = WindowManager::new(conn, screen_num)?;
```

Inside `WindowManager::new()` (`src/window_manager.rs`):

```rust
// 3. Register as window manager by requesting SubstructureRedirect
let event_mask = EventMask::SUBSTRUCTURE_REDIRECT | EventMask::SUBSTRUCTURE_NOTIFY;
conn.change_window_attributes(root, &attributes)?;
```

This is THE critical step! By requesting `SUBSTRUCTURE_REDIRECT`:

- X server will send us MapRequest events (new windows)
- Only ONE client can have this (that's why only one WM can run)
- If another WM is running, this fails!

```text
Before: X Server → directly maps windows
After:  X Server → asks rustile → rustile decides → tells X Server
```

## Architecture Overview

Rustile uses a 3-module architecture following Single Responsibility Principle:

```text
┌─────────────────────────────────────────────────────────┐
│                   WindowManager                         │
│  - Receives X11 events                                  │
│  - Coordinates state updates and rendering              │
└─────────────────────┬───────────────────────────────────┘
                      │ owns
        ┌─────────────┴───────────┐
        ↓                         ↓
┌──────────────┐          ┌──────────────┐
│ WindowState  │          │WindowRenderer│
│              │ <------  │              │
│ - focus      │ injected │ - X11 ops    │
│ - BSP tree   │   into   │ - drawing    │
│ - fullscreen │          │ - borders    │
└──────────────┘          └──────────────┘
```

### Why This Architecture?

**Separation of Concerns:**

- **WindowState**: Pure data, no X11 calls (can test without X server!)
- **WindowRenderer**: All X11 operations (side effects isolated here)
- **WindowManager**: Event handling and coordination

**Dependency Injection Pattern:**

```rust
// WindowRenderer doesn't own state, receives it as parameter
pub fn apply_layout(&mut self, conn: &mut C, state: &mut WindowState)
//                                           ^^^^^^^^^^^^^^^^^^^^ injected!
```

This means:

- Renderer can't accidentally store stale state
- Easy to test with mock state
- Clear data flow

## Event Flow

Let's trace what happens when you open a new xterm window:

```text
1. User runs: $ xterm
       ↓
2. xterm connects to X server: "I want a window"
       ↓
3. X server sends MapRequest event to rustile
       ↓
4. Rustile's event loop receives it:

   Event Flow Through Rustile:
   ┌─────────────────────────────────────────────┐
   │ WindowManager::handle_event(MapRequest)     │
   │                                             │
   │  1. handle_map_request()                    │
   │     ├─ set border color                     │
   │     ├─ conn.map_window()                    │
   │     ├─ state.add_window_to_layout() ────────┼───→ Updates BSP tree
   │     ├─ renderer.set_focus() ────────────────┼───→ X11 focus
   │     └─ renderer.apply_layout() ─────────────┼───→ Position windows
   └─────────────────────────────────────────────┘
```

Visual representation of BSP tree update:

```text
Before:                    After:
┌─────────────┐           ┌─────┬─────┐
│             │           │     │     │
│   Desktop   │    →      │ Win1│ Win2│
│   (empty)   │           │     │     │
└─────────────┘           └─────┴─────┘
```

### Event Types Handled

Rustile handles these X11 events:

- **MapRequest**: New window wants to be shown
- **UnmapNotify**: Window is being hidden
- **DestroyNotify**: Window is being closed
- **ConfigureRequest**: Window wants to change size/position
- **KeyPress**: Keyboard shortcuts (Alt+j, Alt+k, etc.)
- **EnterNotify**: Mouse enters a window (focus follows mouse)
- **FocusIn/FocusOut**: Focus change notifications

Each event type has a corresponding `handle_*` method in WindowManager.