# ADR-010: Zoom to Parent Feature Implementation

## Status
**Current**: Accepted (2024-09-10)

**History**:
- Proposed: 2024-09-10
- Accepted: 2024-09-10

## Context
Users need to temporarily focus on a specific window without losing the underlying BSP layout structure. The "zoom to parent" feature allows expanding a window to fill its parent container's space, similar to features in other tiling window managers (i3's "fullscreen mode 2", dwm's "zoom", bspwm's "monocle" mode).

Key design questions:
1. Allow multiple simultaneous zoomed windows or single zoom only?
2. How should zoom interact with other operations (focus, rotate, window add/remove)?
3. Rendering approach: hide siblings or overlay the zoomed window?
4. User interface: toggle or separate zoom/unzoom commands?

## Decision
Implement a simple, predictable zoom feature with these characteristics:
- **Single zoom only**: One window zoomed at a time for simplicity
- **Toggle interface**: Same key (Alt+d) to zoom/unzoom
- **Overlay rendering**: Use X11 stacking order (z-order) to display zoomed window on top
- **Non-destructive**: BSP tree structure remains unchanged

### Behavior Rules

#### Zoom Toggle Logic
- No zoom active + focus on window → Zoom focused window to parent bounds
- Zoom active on focused window → Unzoom
- Zoom active on different window → Unzoom old, zoom new

#### Events that Clear Zoom
- `rotate` command execution (tree structure changes)
- New window added (user needs to see new window)
- Zoomed window removed

#### Events that Prevent Zoom
- Fullscreen mode active (zoom command ignored while in fullscreen)

#### Events that Preserve Zoom
- `focus_next`/`focus_prev` commands
- Mouse focus changes
- Other windows removed (not the zoomed one)

### Examples
```
Initial 4-window layout:
┌─────┬─────────┐
│     │    B    │
│  A  ├────┬────┤
│     │ C  │ D  │
└─────┴────┴────┘

Zoom C → Fills bottom-right quarter (C+D's parent area):
┌─────┬─────────┐
│     │    B    │
│  A  ├─────────┤
│     │    C    │
└─────┴─────────┘

Zoom B → Fills right half (B+CD's parent area):
┌─────┬─────────┐
│     │         │
│  A  │    B    │
│     │         │
└─────┴─────────┘
```

## Implementation Approach

### State Management
```rust
// In WindowState
zoomed_window: Option<Window>

// In WindowRenderer or WindowManager
fn toggle_zoom(&mut self) -> Result<()>
fn clear_zoom(&mut self) -> Result<()>
```

### BSP Tree Enhancement
```rust
// Find parent split bounds for a window
fn find_parent_bounds(&self, window: Window, screen_rect: BspRect) -> Option<BspRect>
```

### Rendering Logic
1. Calculate normal geometries for all windows
2. If zoom active, override zoomed window's geometry with parent bounds
3. Configure zoomed window with higher stack order (ConfigureWindowAux::new().stack_mode(StackMode::Above))

## Consequences

### Positive
- Simple mental model (one zoom at a time)
- Predictable toggle behavior
- Non-destructive to layout structure
- Minimal state management complexity
- Natural interaction with focus commands

### Negative
- Cannot zoom multiple windows simultaneously
- Rotate command breaks zoom (intentional for v1.0 simplicity)
- Single window (root) cannot zoom (no parent exists)

### Neutral
- Zoom state is transient (lost on WM restart)
- Requires BSP tree traversal to find parent bounds
- Different from fullscreen (window-level vs container-level)

## References
- Implementation: src/window_state.rs zoom functionality
- Implementation: src/window_renderer.rs zoom rendering
- Related: ADR-011 (BSP tree operations)