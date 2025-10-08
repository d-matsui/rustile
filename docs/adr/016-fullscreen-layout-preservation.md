# ADR-016: Fullscreen Layout Preservation

## Status
**Current**: Accepted (2025-10-08)

**History**:
- Proposed: Requirements (2025-10-08)
- Proposed: Design (2025-10-08)
- In Progress (2025-10-08)
- Accepted (2025-10-08)

## Context
Fullscreen mode was being unexpectedly exited when using IME (Input Method Editor) like SKK in applications like Emacs. The user would enter fullscreen with Alt_R+f, switch to Japanese input with Ctrl+j (SKK), and upon typing, fullscreen would automatically exit, returning to tiled layout.

**Root Cause**: `WorkspaceRenderer::apply_workspace()` unconditionally applied normal tiling layout even when in fullscreen mode, overwriting the fullscreen layout maintained by `WindowManager::apply_fullscreen_layout()`.

Various X11 events (ConfigureNotify, MapNotify, etc.) triggered `apply_workspace()` calls, which would destroy the fullscreen layout.


## Requirements

### Functional Requirements
1. **Fullscreen Preservation**: Once entered, fullscreen mode must remain active until explicitly exited by user action (toggle fullscreen, focus different window, window swap operations)
   - IME activity (SKK input switching) must not exit fullscreen
   - Background X11 events must not exit fullscreen

### Non-Functional Requirements
1. **State Consistency**: Workspace state (fullscreen_window) and visual layout must always be synchronized
2. **Performance**: Fullscreen checks must not introduce noticeable latency
3. **Code Quality**: Follow rustile standards (zero warnings, tests pass)


## Design

### Approach
Skip layout application in `apply_workspace` when workspace is in fullscreen mode.

**Solution**: Check fullscreen state before applying layout
- In `apply_workspace()`, check if `workspace.fullscreen_window().is_some()`
- If in fullscreen mode, only update input focus and skip all layout calculations
- Fullscreen layout is managed exclusively by `WindowManager::apply_fullscreen_layout()`

**Alternative Considered: Flag-Based Approach**
- Add boolean flag `in_fullscreen_mode` to WorkspaceRenderer
- Update flag when entering/exiting fullscreen
- **Rejected**: Duplicates state already in Workspace, violates single source of truth principle


### Consequences

**Positive**:
- Fullscreen mode is now stable and reliable
- IME (SKK) works correctly in fullscreen applications
- Maintains architectural principle: Workspace is single source of truth
- Minimal code change (one if-statement)

**Negative**:
- `apply_workspace` now has two code paths (fullscreen vs normal)
- Need to ensure all fullscreen exit paths call `apply_workspace` to restore normal layout


### Key Changes

**Modified Files**:

**`src/workspace_renderer.rs`**
- `apply_workspace()`: Early return when `fullscreen_window.is_some()`
- Only updates input focus in fullscreen mode, skips all layout calculations

**Code Example**:
```rust
pub fn apply_workspace<C: Connection>(
    &mut self,
    conn: &mut C,
    workspace: &Workspace,
) -> Result<()> {
    if workspace.get_all_windows().is_empty() {
        return Ok(());
    }

    // Don't apply normal layout if in fullscreen mode
    // Fullscreen layout is managed by WindowManager
    if workspace.fullscreen_window().is_some() {
        #[cfg(debug_assertions)]
        debug!("Skipping layout application (in fullscreen mode)");
        // Still set focus if needed
        if let Some(focused) = workspace.focused_window() {
            conn.set_input_focus(InputFocus::POINTER_ROOT, focused, CURRENT_TIME)?;
        }
        conn.flush()?;
        return Ok(());
    }

    // Apply normal layout...
}
```


### Testing Strategy

**Manual Testing**:
- Emacs fullscreen + SKK input switching (Ctrl+j) → verify fullscreen maintained
- Emacs fullscreen + Japanese character input → verify fullscreen maintained

**Unit Tests**:
- Existing fullscreen tests continue to pass (72/72 tests pass)
- No new unit tests needed - behavior is integration-level


## Verification

### Quality Gates
- [x] Code builds without errors/warnings
- [x] All tests pass (72/72)
- [x] Documentation updated (this ADR)

### Test Results

**Unit Tests**:
```bash
running 72 tests
test result: ok. 72 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

**Build & Linting**:
```bash
cargo fmt          # ✓ No formatting issues
cargo build        # ✓ Finished in 1.78s
cargo clippy       # ✓ No warnings
```

**Integration Tests**:

**Scenario: SKK Input Switching in Fullscreen Emacs**
- Setup: Open Emacs, enter fullscreen (Alt_R+f)
- Action: Press Ctrl+j (SKK Japanese mode), type characters, press l (ASCII mode)
- Expected: Fullscreen maintained throughout
- Actual: ✓ Fullscreen preserved


## References
- Related: ADR-002 (Single Source of Truth Architecture)
- Issue: Fullscreen exits unexpectedly with SKK/IME input
