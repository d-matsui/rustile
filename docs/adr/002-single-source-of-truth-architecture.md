# ADR-002: Single Source of Truth for Window Storage

## Status
Accepted

## Context
The window manager maintained two data structures for windows: `Vec<Window>` for window lists and `BspTree` for spatial layout. This caused synchronization bugs where operations would update one structure but not the other, leading to:

- Lost window rotations when layout rebuilding discarded tree modifications
- Inconsistent focus management (vector vs tree traversal)  
- Complex swap operations requiring dual coordination
- ~60 lines of redundant synchronization logic

## Decision
Remove `Vec<Window>` and use `BspTree` as the single source of truth for all window operations.

```rust
// Before: Dual storage
pub struct WindowManager<C: Connection> {
    windows: Vec<Window>,      // Removed
    bsp_tree: BspTree,         // Single source of truth
}
```

## Consequences

### Positive
- Eliminates synchronization bugs between data structures
- Preserves user modifications (rotations) across all operations
- Reduces codebase complexity and maintenance overhead
- Single API for window operations improves consistency

### Negative  
- Requires wrapper methods for common operations (`get_all_windows()`)
- Tree traversal for linear operations may be slower than vector access
- Developers must understand BSP tree structure for modifications

### Neutral
- All 59 tests continue to pass with no functionality loss
- Migration required three-phase refactoring but maintained API compatibility