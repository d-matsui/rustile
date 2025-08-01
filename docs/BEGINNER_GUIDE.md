# 🦀 Rustile Guide

Welcome! This guide will teach you how Rustile works while introducing you to X11 window manager concepts. No prior experience with X11, window managers, or Rust required!

## 🎯 What You'll Learn

- **Window Manager Basics** - What they are and how they work
- **X11 Fundamentals** - The graphics system that powers Linux desktops  
- **Rustile Internals** - How a tiling window manager actually works

---

## 1. 🏠 Welcome to Tiling Window Managers

### 🤔 What is a Window Manager?

A **window manager** controls how application windows are displayed and organized on your screen. There are two main types:

**Floating Window Manager** (traditional):
```text
Floating Window Manager:
+----------------------------------+
|  📧 Email                        |
|  +-------+                      |
|  | Inbox |   📝 Text Editor     |
|  |       |   +----------+       |
|  +-------+   | Hello... |       |
|            +--| World!   |       |
|   🌐 Browser | +----------+      |
|   +--------+ |                  |
|   |Google  | |   🎵 Music       |
|   |        | |   +------+       |
|   +--------+ |   |♪ Song|       |
|              |   +------+       |
+----------------------------------+
```

A **tiling window manager** like Rustile automatically arranges windows without overlapping:

```text
Tiling Window Manager (Rustile):
+----------------------------------+
| +----------------+ +-------------+|
| |                | | 📧 Email    ||
| |   🌐 Browser   | +-------------+|
| |                | | 📝 Text     ||
| |                | | Editor      ||
| |                | +-------------+|
| |                | | 🎵 Music    ||
| +----------------+ +-------------+|
+----------------------------------+
```

**Key Differences:**
- **Floating** (traditional): You manually move and resize windows
- **Tiling** (Rustile): Windows automatically arrange themselves
- **No overlapping**: Every window is visible (in tiling mode)
- **Keyboard-driven**: Use shortcuts instead of mouse

### 🧩 Why Use a Tiling Window Manager?

**Benefits:**
- ⚡ **Faster workflow** - No time wasted arranging windows
- 👀 **See everything** - No hidden windows
- ⌨️ **Keyboard efficiency** - Hands stay on keyboard
- 🎯 **Consistent layout** - Same arrangement every time

**Perfect for:**
- Programmers (code + terminal + browser)
- Writers (editor + research + notes)
- Anyone who uses multiple apps simultaneously

---

## 2. 🖥️ Understanding Your Desktop (X11 Basics)

### 🌐 What is X11?

X11 is like the **postal service** for your computer's graphics:

```text
📱 Applications          📮 X11 Server          🖥️ Your Screen
+------------+           +------------+         +------------+
|  Firefox   |  ➤ "I need|            | ➤ Draw |            |
| "I want to |    a window|  X11       |   window|   Screen   |
|  display   |    here"   | (Postal    |   here  |            |
|  a webpage"|           | Service)   |         |            |
+------------+           +------------+         +------------+
                                ⬇️
                         📬 Window Manager (Rustile)
                         "I'll decide WHERE that window goes"
```text

**The Flow:**
1. **Application starts** (Firefox, Terminal, etc.)
2. **Application tells X11**: "I need a window!"
3. **X11 asks Window Manager**: "Where should this window go?"
4. **Rustile decides**: "Put it in the master position"
5. **X11 draws** the window where Rustile said
6. **You see** the arranged windows on screen

### 🎭 Rustile's Role

Rustile is the **traffic controller** for windows:

```text
Without Window Manager:          With Rustile:
+------------------------+       +------------------------+
|  Windows appear        |       |  ┌──────────┬────────┐ |
|  randomly everywhere   |  ➤    |  │          │   App  │ |
|  and overlap each      |       |  │   Main   │    2   │ |
|  other messily         |       |  │   App    ├────────┤ |
|                        |       |  │          │  App 3 │ |
+------------------------+       |  └──────────┴────────┘ |
                                 +------------------------+
```text

---

## 3. 🦀 How Rustile is Built

### 📦 Understanding the Code Structure

Rustile organizes its data in structures. Here's how the main window manager works:

```rust
// Rustile's main "brain"
pub struct WindowManager {
    windows: Vec<Window>,           // 📝 List of all open windows
    focused_window: Option<Window>, // 🎯 Which window gets keyboard input
    config: Config,                 // ⚙️ User settings (gaps, colors, etc.)
}
```

**Think of it like a desk organizer:**
```text
🗃️ WindowManager = {
    📝 Window List: [Firefox, Terminal, VSCode, Music Player]
    🎯 Currently Active: Terminal
    ⚙️ Settings: {
        gap_between_windows: 10 pixels,
        border_color: red,
        shortcuts: Alt+j for next window
    }
}
```

### 🎛️ Layout Options

Rustile supports different layout algorithms:

```rust
// Rustile can arrange windows in different patterns
pub enum Layout {
    MasterStack,  // One big window + smaller stack
    Bsp,          // Binary space partitioning (advanced)
}
```

**Visual representation:**
```text
Layout::MasterStack          Layout::Bsp
┌─────────────┬─────┐       ┌───────┬───────┐
│             │  2  │       │   1   │   2   │
│      1      ├─────┤       ├───────┼───────┤
│             │  3  │       │   3   │   4   │
└─────────────┴─────┘       └───────┴───────┘
```text

### 🔄 Event Handling

Rustile responds to different events from X11:

```rust
// When something happens (an "event"), Rustile decides what to do
match event {
    KeyPress { key: Alt + J } => {
        // User pressed Alt+J, so focus next window
        self.focus_next_window();
    },
    NewWindow { window_id } => {
        // A new app opened, so add it to our layout
        self.add_window_to_layout(window_id);
    },
    WindowClosed { window_id } => {
        // An app closed, so remove it and rearrange
        self.remove_window_and_reflow(window_id);
    },
}
```text

**Like a receptionist at a busy office:**
```text
🔔 "Someone's at the door"     ➤ "Please come in and sit here"
🔔 "Phone is ringing"          ➤ "Hello, how can I help you?"
🔔 "Someone's leaving"         ➤ "Have a nice day, close the door"
```text

### 🛡️ Reliability

Rustile is designed to be crash-free:

```rust
// ❌ This would crash in C/C++:
// window_id = 12345;
// delete_window(window_id);
// use_window(window_id);  // CRASH! Window was already deleted

// ✅ Rust prevents this:
let window_id = Some(12345);
if let Some(id) = window_id {
    delete_window(id);
    // window_id is now None, can't accidentally use deleted window
}
```text

**Benefits for window managers:**
- 🚫 **No crashes** from accessing deleted windows
- 🚫 **No memory leaks** from forgotten cleanup
- 🚫 **No race conditions** between threads
- ✅ **Reliable** window management

---

## 4. 🧩 How Rustile Works (Visual Step-by-Step)

### 🔄 The Main Event Loop

Rustile runs in a continuous loop, like a waiter in a restaurant:

```text
    🍽️ Rustile Event Loop
         ⏰ 1. Wait for something to happen
              ⬇️
🔔 2. Event happens! (key press, new window, etc.)
              ⬇️
🤔 3. "What should I do about this?"
              ⬇️
⚡ 4. Take action (move windows, change focus, etc.)
              ⬇️
♻️ 5. Go back to waiting
              ⬆️
              ⬅️─────────────────────┘
```text

**In Rust code:**
```rust
// Simplified version of Rustile's main loop
loop {
    // 1. Wait for something to happen
    let event = wait_for_event();
    
    // 2. Decide what to do
    match event {
        KeyPress => handle_keyboard(),
        NewWindow => arrange_windows(),
        WindowClosed => cleanup_and_rearrange(),
    }
    
    // 3. Update the display
    refresh_screen();
    
    // 4. Loop forever
}
```text

### 🪟 What Happens When You Open an App

Let's trace what happens when you open Firefox:

```text
Step 1: You run "firefox" in terminal
    👤 User ──"firefox"──➤ 💻 Terminal

Step 2: Firefox starts and asks X11 for a window
    🦊 Firefox ──"I need a window!"──➤ 🖥️ X11

Step 3: X11 asks Rustile where to put it
    🖥️ X11 ──"Where should Firefox go?"──➤ 🦀 Rustile

Step 4: Rustile calculates the best position
    🦀 Rustile thinks:
    "I have 2 windows already: [Terminal, VSCode]
     Firefox should go in the stack area
     Position: x=960, y=0, width=960, height=540"

Step 5: X11 draws Firefox in that position
    🖥️ X11 ──draws──➤ 📺 Screen

Step 6: You see the new layout
    Before:                    After:
    ┌─────────────┬─────┐     ┌─────────────┬─────┐
    │             │VSCode│     │             │VSCode│
    │   Terminal  │     │     │   Terminal  ├─────┤
    │             │     │     │             │Fire-│
    │             │     │     │             │fox  │
    └─────────────┴─────┘     └─────────────┴─────┘
```text

### 🎯 Focus Management (Red Borders)

Focus determines which window receives your keyboard input:

```rust
// Rustile tracks which window is "active"
pub struct WindowManager {
    focused_window: Option<Window>,  // Currently focused window
    // ... other fields
}

// When focus changes:
fn set_focus(&mut self, new_window: Window) {
    // Remove red border from old window
    if let Some(old_focused) = self.focused_window {
        self.set_border_color(old_focused, GRAY);
    }
    
    // Add red border to new window
    self.set_border_color(new_window, RED);
    self.focused_window = Some(new_window);
}
```text

**Visual representation:**
```text
Before Alt+J:                  After Alt+J:
┌─────────────┬─────┐         ┌─────────────┬─────┐
│             │     │         │             │═════│ ← Red border
│   Terminal  │VSCode│         │   Terminal  ║VSCode║   (focused)
│   (focused) │     │         │             ║     ║
│═════════════│     │         │             ║     ║
└─────────────┴─────┘         └─────────────┴═════┘
  ↑ Red border moved                        ↑ Focus moved here
```text

---

## 5. 🎹 Basic Operations (Hands-On)

### ⌨️ Essential Keyboard Shortcuts

These are the core shortcuts you need to know:

```text
🎯 FOCUS (Which window gets your typing):
Alt + J     ➤  Focus next window (clockwise)
Alt + K     ➤  Focus previous window (counter-clockwise)

🔄 SWAP (Move windows around):
Shift + Alt + J  ➤  Swap focused window with next window
Shift + Alt + K  ➤  Swap focused window with previous window  
Shift + Alt + M  ➤  Swap focused window with master (main) window

🗑️ MANAGE:
Shift + Alt + Q  ➤  Close focused window

🚀 LAUNCH:
Super + Return   ➤  Open terminal
```text

### 🎮 Try It Yourself

**Exercise 1: Moving Focus**
1. Open 3 applications (terminal, browser, text editor)
2. Press `Alt + J` repeatedly
3. Watch the red border move between windows
4. Try `Alt + K` to go backwards

**Exercise 2: Rearranging Windows**  
1. Focus the middle window
2. Press `Shift + Alt + J` (swap with next)
3. Notice how the windows exchange positions
4. Try `Shift + Alt + M` (swap with master)

**Exercise 3: Master Window**
```text
Initial Layout:           After Shift+Alt+M:
┌─────────────┬─────┐    ┌─────────────┬─────┐
│             │  B  │    │             │  A  │
│      A      ├─────┤ ➤  │      B      ├─────┤
│ (Master)    │  C* │    │ (New Master)│  C* │
│             │     │    │             │     │
└─────────────┴─────┘    └─────────────┴─────┘
                           * = focused window
```text

---

## 6. 🔧 Configuration Basics

### 📝 What is TOML?

TOML is a simple format for configuration files, like a recipe:

```toml
# ~/.config/rustile/config.toml

[layout]
bsp_split_ratio = 0.5        # Equal 50/50 splits for new windows
gap = 10                     # 10 pixels between windows
border_width = 2             # 2 pixel thick borders

[shortcuts]
"Alt+j" = "focus_next"       # Define what Alt+J does
"Alt+k" = "focus_prev"       # Define what Alt+K does
"Super+Return" = "xterm"     # Super+Enter opens terminal
```text

### 🎨 Visual Settings Explained

```text
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
```text

---

## 8. 📚 Key Concepts & Glossary

### 🪟 Window Manager Concepts

| Term | Definition | Visual Example |
|------|------------|----------------|
| **Window** | Rectangular area where an app displays content | `+-----+`<br>`| App |`<br>`+-----+` |
| **Focus** | Which window receives keyboard input (red border) | `+=====+` ← Focused<br>`‖ App ‖`<br>`+=====+` |
| **Master** | First window in BSP tree (top-left position) | `+-------+ +---+`<br>`|  1st  | |2nd|`<br>`+-------+ +---+` |
| **BSP** | Binary Space Partitioning - divides screen into halves | `+---+---+`<br>`| A | B |` → vertical split<br>`+---+---+` |
| **Layout** | Algorithm for arranging windows | BSP (Binary Space Partitioning) |
| **Tiling** | Automatic window arrangement (no overlapping) | All windows visible, organized |
| **Event** | Message from X11 (key press, new window, etc.) | User presses key → Event → Action |
| **X11** | Graphics system on Linux that manages windows | "Postal service" between apps and window manager |

## 🎓 What's Next?

### 🌟 You Now Understand:
- ✅ How window managers work (automatic vs manual arrangement)
- ✅ Basic Rust programming concepts (structs, enums, pattern matching)
- ✅ X11 graphics system (how apps talk to your desktop)
- ✅ Rustile's event-driven architecture
- ✅ Essential keyboard shortcuts and workflows

### 🚀 Ready for More?

**Dive Deeper into Rustile:**
- [TECHNICAL_DEEP_DIVE.md](TECHNICAL_DEEP_DIVE.md) - Advanced implementation details
- Try customizing layouts and adding new shortcuts
- Contribute to the Rustile project on GitHub

---

*Happy tiling!* 🪟✨
