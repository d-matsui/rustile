# ADR-003: SRP Refactoring to Three-Module Architecture

## Status
**Current**: Deprecated (2024-09-10)

**History**:
- Proposed: 2024-08-12
- Accepted: 2024-08-12
- Deprecated: 2024-09-10

## Context
After consolidating window state into `WindowOperations` (ADR-002), the module grew to 567 lines and violated Single Responsibility Principle by mixing state management, X11 operations, and coordination logic. This caused:

- Testing difficulty: X11 operations couldn't be tested independently
- Maintenance burden: State changes required understanding X11 protocols
- Critical bug: Windows stacked at top-left instead of proper BSP arrangement

## Decision
Split `WindowOperations` into three modules following Single Responsibility Principle:

**WindowState** - Pure state management (no X11 dependencies)
```rust
pub struct WindowState {
    focused_window: Option<Window>,
    bsp_tree: BspTree,
    fullscreen_window: Option<Window>,
    intentionally_unmapped: HashSet<Window>,
    config: Config,
    screen_num: usize,
}
```

**WindowRenderer** - X11 operations with dependency injection
```rust
pub struct WindowRenderer {
    // Stateless - operates on injected WindowState
}
```

**WindowManager** - Event coordination between state and rendering
```rust
pub struct WindowManager<C: Connection> {
    conn: C,
    shortcut_manager: ShortcutManager,
    window_state: WindowState,
    window_renderer: WindowRenderer,
}
```

### Examples
```rust
// Before: Coupled state and rendering
self.window_operations.apply_layout(&mut self.conn)

// After: Separated with dependency injection
self.window_renderer.apply_layout(&mut self.conn, &mut self.window_state)
```

## Consequences

### Positive
- Fixed critical window positioning bug (proper BSP layout restored)
- Pure state functions enable comprehensive unit testing
- Clear separation: Events → State Update → Render
- Reduced codebase by 579 lines while preserving functionality
- Each module has single, focused responsibility

### Negative
- More function parameters due to dependency injection
- Slightly more complex call sites (must pass state to renderer)
- Developers must understand data flow between three modules

### Neutral
- All 66 tests pass with no functionality loss
- Interactive testing confirms proper window arrangement
- Follows "Functional Core, Imperative Shell" architecture pattern

## Deprecation Reason
This ADR described a 3-module architecture that was later superseded by the current 7-module flat structure (main.rs, window_manager.rs, window_state.rs, window_renderer.rs, bsp.rs, config.rs, keyboard.rs). The flat structure proved simpler and more maintainable than the hierarchical approach described here.

## References
- Related: ADR-002 (Single source of truth)
- Current implementation: src/ directory with 7-module flat structure