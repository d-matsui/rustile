# 🔬 Rustile Technical Deep Dive

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
}

// Example plugins:
struct StatusBarPlugin { /* ... */ }
struct NotificationPlugin { /* ... */ }  
struct WorkspacePlugin { /* ... */ }

impl WindowManagerPlugin for StatusBarPlugin {
    fn on_window_created(&mut self, window: Window, manager: &mut WindowManager) {
        // Update status bar with new window count
        self.update_window_count(manager.windows.len());
    }
}
```

---

This technical deep dive reveals the sophisticated engineering behind Rustile's simple interface. The combination of Rust's memory safety, efficient algorithms, and clean architecture makes it both performant and maintainable.