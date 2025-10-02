# ADR-015: Key Event Replay for Unmatched Shortcuts

## Status
**Current**: Accepted (2025-10-02)

**History**:
- Proposed: Requirements (2025-10-02)
- Proposed: Design (2025-10-02)
- In Progress (2025-10-02)
- Accepted (2025-10-02)

## Context
After implementing ADR-014 (Alt_L/Alt_R distinction), pressing Alt_L+f no longer triggers fullscreen (correct behavior), but the key event does not reach applications like Emacs. In Emacs, Alt_L+f should trigger forward-word (M-f), but nothing happens.

The root cause is that X11's grab_key mechanism intercepts all registered key combinations and sends them to the window manager. When a grabbed key event doesn't match any shortcut (e.g., Alt_L+f when only Alt_R+f is registered), the event is simply dropped instead of being forwarded to the focused application.


## Requirements

### Functional Requirements
1. **Key Event Forwarding**: When a grabbed key event doesn't match any registered shortcut, the event must be replayed to the focused application
   - Example: Alt_L+f should reach Emacs when no Alt_L+f or Alt_f shortcut is registered
   - Example: Alt_R+f should reach Chrome when no Alt_R+f or Alt_f shortcut is registered

2. **Shortcut Matching Preservation**: Existing shortcut matching behavior must remain unchanged
   - Alt+f should still match with either Alt_L or Alt_R
   - Alt_L+f should only match Alt_L+f shortcuts
   - Alt_R+f should only match Alt_R+f shortcuts

3. **Application Behavior**: Applications must receive key events as if the window manager didn't grab them
   - Emacs Alt_L+f → forward-word (M-f)
   - Chrome Alt+Left → browser back
   - No duplicate events when shortcuts do match

### Non-Functional Requirements
1. **Performance**: Event replay must not introduce noticeable latency
2. **X11 Protocol Compliance**: Must use proper X11 event replay mechanisms
3. **Backward Compatibility**: All existing shortcuts must continue to work
4. **Code Quality**: Follow rustile standards (zero warnings, anyhow::Result, tests)


## Design

### Approach
Use X11's synchronous grab mechanism to intercept key events, then either consume or replay them based on shortcut matching.

**Option A: SYNC Grab + AllowEvents (Selected)**
- Change `grab_key` from ASYNC to SYNC mode
- When SYNC grab is active, X11 freezes event processing until we call `AllowEvents`
- In `handle_key_press`:
  - If shortcut matches → `AllowEvents(ASYNC_KEYBOARD)` to process and continue
  - If no match → `AllowEvents(REPLAY_KEYBOARD)` to replay event to focused window
- Pros: Standard X11 protocol mechanism, clean, efficient
- Cons: Requires careful error handling to avoid freezing keyboard

**Option B: UngrabKey + Manual Forwarding**
- Temporarily ungrab the key, manually forward event, re-grab
- Pros: Conceptually simple
- Cons: Race conditions, complex timing issues, not recommended by X11 protocol

**Option C: SendEvent Manual Injection**
- Use `SendEvent` to manually inject key events to focused window
- Pros: No grab mode changes needed
- Cons: Applications can detect synthetic events and may reject them (security), violates X11 best practices

**Decision**: Option A because it's the standard X11 protocol mechanism for this exact use case. Most window managers (i3, dwm, xmonad) use this approach.

### Consequences

**Positive**:
- **Correct X11 behavior**: Applications receive key events they should see
- **User expectations met**: Emacs shortcuts work even when similar WM shortcuts exist
- **Standard protocol**: Uses X11 best practices (same as i3, dwm, xmonad)
- **Fine-grained control**: Alt_L/Alt_R distinction works correctly with application shortcuts

**Negative**:
- **Complexity**: SYNC grab mode requires careful AllowEvents handling
- **Error risk**: If AllowEvents is not called, keyboard freezes (mitigated by proper error handling)
- **Debugging difficulty**: Event replay behavior harder to debug than ASYNC mode

**Neutral**:
- **Performance**: SYNC mode has negligible overhead (microseconds) compared to ASYNC
- **Code size**: Adds ~5-10 lines of code for AllowEvents calls

### Key Changes

**src/keyboard.rs**:
1. `register_shortcut()`: Change `GrabMode::ASYNC` → `GrabMode::SYNC` (both pointer and keyboard)
2. `handle_key_press()`: Return whether shortcut matched (for AllowEvents decision)

**src/window_manager.rs**:
1. `handle_key_press()`: After calling `shortcut_manager.handle_key_press()`:
   - If matched: Call `AllowEvents(ASYNC_KEYBOARD, CurrentTime)` to process shortcut
   - If no match: Call `AllowEvents(REPLAY_KEYBOARD, CurrentTime)` to replay to app

### Testing Strategy

**Unit Tests**:
- Existing shortcut matching tests must pass unchanged
- No new unit tests needed (X11 connection required for AllowEvents)

**Integration Tests** (manual in Xephyr with `./test.sh`):
1. **Alt+f registered, Alt_L+f pressed** → Shortcut triggers (fullscreen), Emacs does NOT see event
2. **Alt+f registered, Alt_R+f pressed** → Shortcut triggers (fullscreen), Emacs does NOT see event
3. **Alt_R+f registered, Alt_L+f pressed** → Emacs receives M-f (forward-word)
4. **No Alt+f shortcuts, Alt_L+f pressed** → Emacs receives M-f (forward-word)
5. **Super+t registered, Super+t pressed** → xterm launches (existing behavior preserved)

**Edge Cases**:
- Keyboard freeze prevention: If AllowEvents fails, should log error but not crash
- Multiple rapid key presses: SYNC mode should handle queuing correctly


## Verification

### Code Quality
- `cargo fmt` - Formatted
- `cargo clippy --all-targets --all-features -- -D warnings` - No warnings
- `cargo test` - 66 tests passing

### Integration Testing (TTY3 with debug build)
Tested with config: `Alt_R+f = "toggle_fullscreen"`

**Test 1: Alt_L+f pressed (no matching shortcut)**
- Expected: Key event replayed to Emacs → M-f (forward-word) works
- Result: PASS - Emacs received M-f and moved cursor forward by word

**Test 2: Alt_R+f pressed (matching shortcut)**
- Expected: Fullscreen toggle executes, Emacs does NOT receive event
- Result: PASS - Window entered fullscreen mode successfully

### Verification Summary
Both tests passed. The SYNC grab + AllowEvents mechanism works correctly:
- Matched shortcuts → Consumed by WM (Alt_R+f triggers fullscreen)
- Unmatched shortcuts → Replayed to focused application (Alt_L+f reaches Emacs as M-f)


## References
- Related: ADR-014 (Better modifier handling - Alt_L/Alt_R distinction)
- X11 Protocol: XGrabKey, XAllowEvents
