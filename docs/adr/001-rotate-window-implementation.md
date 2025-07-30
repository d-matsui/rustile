# ADR-001: Rotate Window Implementation in BSP Layout

## Status
Proposed

## Context
Rustile uses a Binary Space Partitioning (BSP) layout algorithm to arrange windows in a tree structure. Users need a way to rotate/reorganize windows to change their spatial arrangement while maintaining the BSP tree integrity.

The challenge is defining what "rotate window" means in the context of a BSP tree, where each node represents either a window (leaf) or a split direction (internal node with Horizontal/Vertical orientation).

## Decision
We will implement window rotation by **flipping the split direction of the focused window's parent node**.

### Rotation Algorithm:
1. Identify the currently focused window
2. Find the parent split node of the focused window in the BSP tree
3. Flip the parent's split direction (Horizontal ↔ Vertical)
4. Reapply the layout with the modified tree structure
5. Maintain focus on the same window

### Behavior Examples:

#### Example 1: 3 Windows, A Focused
**Before:**
```
       Root(V)              Layout:
      /      \               ┌─────┬─────────┐
     A        Split(H)       │  A  │    B    │
             /        \      │     ├─────────┤  
            B          C     │     │    C    │
                             └─────┴─────────┘
```

**After (A's parent Root(V) → Root(H)):**
```
       Root(H)              Layout:
      /      \               ┌───────────────┐
     A        Split(H)       │       A       │
             /        \      ├───────────────┤
            B          C     │       B       │
                             ├───────────────┤
                             │       C       │
                             └───────────────┘
```

#### Example 2: Same Tree, B Focused
**Before:**
```
       Root(V)              Layout:
      /      \               ┌─────┬─────────┐
     A        Split(H)       │  A  │    B    │ ← B focused
             /        \      │     ├─────────┤  
            B          C     │     │    C    │
                             └─────┴─────────┘
```

**After (B's parent Split(H) → Split(V)):**
```
       Root(V)              Layout:
      /      \               ┌─────┬───┬───┐
     A        Split(V)       │  A  │ B │ C │
             /        \      │     │   │   │
            B          C     └─────┴───┴───┘
```

## Rationale

### Advantages:
1. **Predictable Behavior**: Each window has exactly one rotation behavior based on its position in the tree
2. **Minimal Tree Mutation**: Only one split direction changes, preserving most of the tree structure
3. **Reversible**: Applying rotate twice returns to the original layout
4. **Focus Preservation**: The focused window remains focused after rotation
5. **Universal Application**: Works consistently regardless of tree depth or complexity

### Considered Alternatives:
1. **Rotate Window Positions**: Swap window positions while keeping tree structure → Rejected because it's complex to define consistent swapping rules
2. **Rotate Entire Subtrees**: Rotate tree branches → Rejected because it can create unpredictable layouts
3. **Rotate Focus Order**: Just change focus traversal → Rejected because it doesn't change visual layout

## Consequences

### Positive:
- Simple mental model for users: "rotation changes how my window relates to its siblings"
- Easy to implement: single tree mutation + layout reapplication
- Consistent behavior across different window arrangements
- Low computational complexity

### Negative:
- Some users might expect different rotation semantics (e.g., cycling window positions)
- Root window rotation affects all other windows, which might be surprising
- Not immediately obvious which direction will result from rotation without understanding BSP trees

### Risk Mitigation:
- Clear documentation with visual examples
- Consistent keyboard shortcut (Alt+r) 
- Informative log messages during rotation
- Comprehensive testing with various window arrangements

## Implementation Notes
- Implement in `src/window_manager/window_ops.rs`
- Add `rotate_windows` command to event dispatcher
- Modify BSP tree in-place by flipping split direction
- Reuse existing layout application logic
- Add comprehensive tests for various tree structures

## Success Metrics
- Users can predictably reorganize windows using rotation
- No performance degradation with rotation operations
- Rotation works consistently across different numbers of windows and tree configurations