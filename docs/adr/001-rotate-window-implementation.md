# ADR-001: Window Rotation by Parent Split Flip

## Status
Accepted

## Context
Users need to reorganize windows in the BSP tiling layout. The challenge is defining what "rotate window" means in a binary tree where each node represents either a window (leaf) or a split direction (horizontal/vertical).

Three potential approaches:
1. Swap window positions (complex swapping rules)
2. Rotate entire subtrees (unpredictable layouts)
3. Flip parent split direction (chosen approach)

## Decision
Implement window rotation by flipping the split direction of the focused window's parent node.

When user presses Alt+r on focused window:
- Find the window's parent split node in BSP tree
- Flip split direction: Horizontal ↔ Vertical
- Reapply layout with modified tree structure

## Consequences

### Positive
- Predictable behavior: each window has one rotation based on tree position
- Minimal tree mutation: only one split direction changes
- Reversible: applying rotate twice returns to original layout
- Simple implementation: single tree modification + layout reapplication

### Negative
- Root window rotation affects all other windows (potentially surprising)
- Users may expect different rotation semantics (e.g., cycling positions)
- Behavior not obvious without understanding BSP tree structure

### Neutral
- Requires visual examples in documentation for user understanding
- Alt+r shortcut provides consistent access across all window arrangements