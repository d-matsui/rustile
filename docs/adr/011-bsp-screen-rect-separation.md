# ADR-011: Separation of BSP Tree Logic from Screen Geometry Calculations

## Status
**Current**: Accepted (2024-09-10)

**History**:
- Proposed: 2024-09-10
- Accepted: 2024-09-10

## Context
The current BSP module (`src/bsp.rs`) violates the Single Responsibility Principle by mixing pure tree operations with screen geometry calculations and rendering concerns.

### Current Problems

1. **Duplicated Code**: Screen rectangle calculation logic is duplicated in 3 locations:
   - `bsp.rs:582` in `calculate_bsp_geometries()`
   - `window_renderer.rs:141` in `apply_state()` for zoom handling
   - `window_renderer.rs:327` in `zoom_parent()`

2. **Mixed Responsibilities**: The BSP module contains:
   - Pure tree data structure operations (add, remove, rotate, swap)
   - Screen geometry calculations (`BspRect`, `WindowGeometry`)
   - Layout parameter management (`LayoutParams`)
   - Direct X11 window type dependencies

3. **Tight Coupling**: BSP tree operations are tightly coupled with rendering calculations, making it difficult to:
   - Test BSP tree logic independently
   - Reuse BSP tree for different rendering strategies
   - Understand the separation of concerns

### Example of Current Duplication
```rust
// Pattern appears 3 times in codebase
let screen_rect = crate::bsp::BspRect {
    x: state.layout_params().gap as i32,
    y: state.layout_params().gap as i32,
    width: (screen.width_in_pixels as i32 - 2 * state.layout_params().gap as i32)
        .max(state.layout_params().min_window_width as i32),
    height: (screen.height_in_pixels as i32 - 2 * state.layout_params().gap as i32)
        .max(state.layout_params().min_window_height as i32),
};
```

## Decision

Refactor the BSP module to separate pure tree operations from screen geometry calculations following these principles:

### 1. Module Responsibility Separation
- **BSP Module (`bsp.rs`)**: Focus exclusively on tree operations
  - Tree manipulation: add, remove, rotate, swap
  - Tree navigation: next, previous, find parent
  - Tree queries: window count, contains window
  
- **Window Renderer (`window_renderer.rs`)**: Handle all geometry calculations
  - Screen rectangle calculations
  - Window geometry mapping
  - Layout parameter application

### 2. Type Reorganization
Move rendering-specific types from `bsp.rs` to `window_renderer.rs`:
- `BspRect` → Rendering concern
- `WindowGeometry` → Rendering concern  
- `LayoutParams` → Rendering concern

Keep in `bsp.rs`:
- `BspNode` → Pure tree structure
- `BspTree` → Tree container
- `SplitDirection` → Tree property

### 3. Function Relocation
Move `calculate_bsp_geometries()` and `calculate_bsp_recursive()` from `bsp.rs` to `window_renderer.rs` as these functions:
- Calculate screen coordinates
- Apply gap and border calculations
- Transform tree structure to render geometries

### 4. Centralized Screen Rectangle Helper
Create a single helper function in `window_state.rs` to eliminate duplication:
```rust
impl WindowState {
    pub fn calculate_screen_rect(&self, screen_width: u16, screen_height: u16) -> BspRect {
        let params = self.layout_params();
        BspRect {
            x: params.gap as i32,
            y: params.gap as i32,
            width: (screen_width as i32 - 2 * params.gap as i32)
                .max(params.min_window_width as i32),
            height: (screen_height as i32 - 2 * params.gap as i32)
                .max(params.min_window_height as i32),
        }
    }
}
```

## Implementation Steps

1. **Phase 1: Extract Helper**
   - Create `calculate_screen_rect()` in `window_state.rs`
   - Update all 3 duplicate locations to use the helper

2. **Phase 2: Move Types**
   - Move `BspRect`, `WindowGeometry`, `LayoutParams` to `window_renderer.rs`
   - Update imports across codebase

3. **Phase 3: Relocate Functions**
   - Move `calculate_bsp_geometries()` to `window_renderer.rs`
   - Move `calculate_bsp_recursive()` to `window_renderer.rs`
   - Update `find_parent_bounds()` to return tree-relative bounds

4. **Phase 4: Clean Dependencies**
   - Remove `use x11rb::protocol::xproto::*` from `bsp.rs`
   - Use generic window ID type alias if needed

## Consequences

### Positive
- **Clear separation of concerns**: BSP handles tree logic, renderer handles geometry
- **Eliminated code duplication**: Single source of truth for screen rectangle calculation
- **Improved testability**: BSP tree can be tested without X11 dependencies
- **Better maintainability**: Changes to rendering don't affect tree logic
- **Follows SOLID principles**: Single Responsibility and Dependency Inversion

### Negative
- **More inter-module communication**: Renderer needs to query BSP tree then calculate geometries
- **Potential performance impact**: Additional function calls (negligible in practice)

### Neutral
- **File size changes**: `bsp.rs` becomes smaller, `window_renderer.rs` grows slightly
- **Import adjustments**: Various files need import updates

## Alternatives Considered

1. **Create separate geometry module**: Rejected as it would add unnecessary complexity for ~200 lines of code
2. **Keep everything in BSP**: Rejected as it violates SRP and maintains tight coupling
3. **Move everything to renderer**: Rejected as BSP tree is a valid standalone data structure

## References
- ADR-003: SRP refactoring principle established for this codebase
- ROADMAP.md: Lists "Screen rect calculation cleanup" as architectural goal