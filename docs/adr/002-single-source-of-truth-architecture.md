# ADR-002: Single Source of Truth Architecture for Window Management

## Status
Accepted

## Context
Rustile's window manager initially used dual storage for managing windows: a `Vec<Window>` for maintaining window lists and a `BspTree` for spatial layout. This dual storage architecture created several problems:

1. **Synchronization Issues**: Two data structures needed to be kept in sync manually
2. **Source of Truth Confusion**: Unclear which data structure was authoritative for different operations
3. **Bug-Prone Operations**: Window operations like rotation and swapping could update one structure but not the other
4. **Code Duplication**: Similar logic needed in multiple places to maintain consistency
5. **Performance Overhead**: Duplicate storage and synchronization costs

### Specific Problems Encountered:
- Window rotations were lost when `apply_layout()` rebuilt the BSP tree from the vector
- Focus management used vector iteration while layout used tree traversal
- Swap operations required careful coordination between both data structures
- Adding/removing windows required updates to both structures

## Decision
We will eliminate the dual storage architecture and use the **BSP tree as the single source of truth** for all window management operations.

### Architecture Changes:

#### Before (Dual Storage):
```rust
pub struct WindowManager<C: Connection> {
    pub(super) windows: Vec<Window>,      // Window list
    pub(super) bsp_tree: BspTree,         // Layout tree
    // ... other fields
}
```

#### After (Single Source of Truth):
```rust
pub struct WindowManager<C: Connection> {
    // Removed: pub(super) windows: Vec<Window>,
    pub(super) bsp_tree: BspTree,         // Single source of truth
    // ... other fields
}
```

### Implementation Strategy:

1. **Phase 2A**: Enhance BSP tree API with comprehensive navigation methods
2. **Phase 2B**: Eliminate dual layout methods that caused tree rebuilding
3. **Phase 2C**: Remove windows vector entirely and create wrapper methods

### API Encapsulation:
Create wrapper methods in `WindowManager` to hide BSP tree implementation:

```rust
// Clean public interface
pub(super) fn add_window_to_layout(&mut self, window: Window)
pub(super) fn remove_window_from_layout(&mut self, window: Window)  
pub(super) fn get_all_windows(&self) -> Vec<Window>
pub(super) fn has_window(&self, window: Window) -> bool
pub(super) fn window_count(&self) -> usize
```

## Rationale

### Advantages:
1. **Architectural Clarity**: One authoritative source eliminates confusion
2. **Bug Prevention**: Eliminates entire class of synchronization bugs
3. **Performance**: No duplicate storage or sync overhead
4. **Code Simplification**: Removes ~60 lines of redundant logic
5. **Maintainability**: Changes only need to be made in one place
6. **Data Integrity**: Impossible for data structures to get out of sync

### BSP Tree as Natural Choice:
- Already contains all window information and relationships
- Provides spatial organization required for tiling
- Supports efficient navigation and modification
- Tree operations preserve user modifications (rotations)

### Considered Alternatives:
1. **Keep Vector as Primary**: Rejected because it lacks spatial relationships
2. **Hybrid Approach**: Rejected because it maintains synchronization complexity
3. **New Data Structure**: Rejected as BSP tree already provides needed functionality

## Consequences

### Positive:
- **Eliminates synchronization bugs**: No more dual storage inconsistency
- **Preserves user modifications**: Rotations and tree changes persist correctly
- **Improved performance**: Single data structure with no sync overhead
- **Cleaner codebase**: Simpler mental model and reduced complexity
- **Better encapsulation**: Implementation details hidden behind clean API
- **Easier testing**: Single source of truth easier to verify and test

### Negative:
- **API indirection**: Some operations now go through wrapper methods
- **Learning curve**: Developers need to understand BSP tree concepts
- **Potential performance considerations**: Tree traversal for linear operations

### Migration Challenges:
- **Complete refactoring required**: Cannot be done incrementally
- **Extensive testing needed**: All window operations must be verified
- **API changes**: Internal interfaces needed updates

## Implementation Details

### Phase 2A: Enhanced BSP Tree API
Added comprehensive navigation methods to `src/layout/bsp.rs`:
```rust
pub fn all_windows(&self) -> Vec<Window>
pub fn window_count(&self) -> usize  
pub fn has_window(&self, window: Window) -> bool
pub fn next_window(&self, current: Window) -> Option<Window>
pub fn prev_window(&self, current: Window) -> Option<Window>
```

### Phase 2B: Unified Layout Application
Eliminated dual layout methods:
- Removed `apply_existing_layout()` and `rebuild_and_apply_layout()`
- Made `apply_layout()` preserve tree structure by default
- Fixed rotation and swap operations to maintain tree integrity

### Phase 2C: Complete Migration
- Removed `windows: Vec<Window>` field from `WindowManager`
- Created wrapper methods for clean API encapsulation
- Updated all window operations to use BSP tree methods
- Fixed clippy warnings and maintained test coverage

### Files Modified:
- `src/window_manager/core.rs`: Removed windows field
- `src/window_manager/events.rs`: Updated to use wrapper methods
- `src/window_manager/focus.rs`: Changed to BSP tree navigation
- `src/window_manager/window_ops.rs`: Added wrapper methods, unified layout

## Success Metrics
- ✅ **All 59 tests pass**: No functionality lost during migration
- ✅ **Zero clippy warnings**: Clean code with strict linting
- ✅ **Clean build**: No warnings with all targets and features
- ✅ **Window operations work correctly**: Focus, swap, rotate, fullscreen all function
- ✅ **Tree structure preserved**: Rotations and modifications persist across operations
- ✅ **Performance maintained**: No observable performance regression

## Risk Mitigation
1. **Comprehensive testing**: All existing tests must pass
2. **Gradual migration**: Implemented in well-defined phases
3. **API preservation**: External behavior maintained despite internal changes
4. **Documentation**: Clear ADR and commit messages for future reference
5. **Reversibility**: Changes are well-documented and could be reverted if needed

## Future Considerations
- **Monitor performance**: Tree traversal vs vector operations for large window counts
- **API evolution**: May need additional BSP tree methods for new features
- **Documentation updates**: Update technical documentation to reflect architecture
- **Onboarding**: New developers need BSP tree architecture explanation

This architecture change establishes a solid foundation for future window management features while eliminating a major source of bugs and complexity.