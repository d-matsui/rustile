# ADR-014: Better Modifier Handling - Left/Right Key Distinction

## Status
**Completed** (2025-10-01)

**History**:
- Proposed: Requirements (2025-10-01)
- Proposed: Design (2025-10-01)
- Proposed: Plan (2025-10-01)
- In Progress (2025-10-01)
- Completed (2025-10-01)

## Context
Currently, rustile does not distinguish between left and right modifier keys. As documented in ADR-008:

```rust
"Alt_L" | "Alt_R" → both become ModMask::M1
// We don't distinguish left/right modifiers
```

This limitation affects users who want to use different shortcuts for left Alt vs right Alt.

In X11, while both Alt_L and Alt_R generate ModMask::M1, they have different keysyms (0xffe9 for Alt_L, 0xffea for Alt_R), making it technically possible to distinguish them.

## Requirements

### Functional Requirements

#### 1. Left/Right Alt Key Distinction
Support distinguishing left and right Alt keys only:
- **Alt_L** (Left Alt, Mod1)
- **Alt_R** (Right Alt, Mod1)

Other modifiers (Ctrl, Shift, Super) do not need left/right distinction in this phase.

#### 2. Configuration Syntax
Allow users to specify left/right modifiers in config.toml:
```toml
[shortcuts]
"Alt_L+t" = "spawn xterm"      # Only left Alt
"Alt_R+t" = "spawn firefox"    # Only right Alt
"Alt+t" = "spawn emacs"        # Either Alt (backward compatibility)
```

#### 3. Backward Compatibility
Existing shortcuts without _L/_R suffix must continue to work:
- `"Alt+t"` matches both Alt_L+t and Alt_R+t
- No breaking changes to existing configurations

When a key event arrives:
- If shortcut specifies `Alt_L`, only match if left Alt is pressed
- If shortcut specifies `Alt_R`, only match if right Alt is pressed
- If shortcut specifies `Alt`, match if either Alt is pressed


### Quality Requirements (rustile specific)
- [ ] Zero-warning requirement maintained
- [ ] Error handling with `anyhow::Result`
- [ ] Test strategy defined (unit/integration/edge cases)
  - Unit: Parse "Alt_L+t", "Alt_R+t", "Alt+t" correctly
  - Unit: Match logic for specific vs generic modifiers
  - Integration: Actual key event matching in Xephyr
  - Edge: Multiple modifiers with left/right (e.g., "Ctrl+Alt_R+t")
- [ ] ADR-005 comment standard compliance
- [ ] No `#[allow()]` attributes


## Design

### Overview

Extend keyboard shortcut system to distinguish between left and right Alt keys while maintaining backward compatibility and X11 protocol constraints.

### Data Structures

#### Current Structure
```rust
pub struct Shortcut {
    pub modifiers: ModMask,  // X11 bit flags (e.g., ModMask::M1)
    pub keycode: u8,
    pub command: String,
}
```

#### New Structure
```rust
pub struct Shortcut {
    pub modifiers: ModMask,
    pub keycode: u8,
    pub command: String,
    pub alt_side: ModifierSide,  // NEW: Alt left/right requirement
    // Future: pub ctrl_side: ModifierSide,
    // Future: pub shift_side: ModifierSide,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ModifierSide {
    Either,     // Default: matches left or right
    LeftOnly,   // Only left modifier key
    RightOnly,  // Only right modifier key
}
```

### Key Design Decisions

#### 1. Separate Field for Side Specification

**Why not extend ModMask?**
- `ModMask` is X11 protocol-defined bit flags
- X11's `KeyPressEvent.state` uses `ModMask::M1` for both Alt_L and Alt_R
- Cannot distinguish left/right at ModMask level

**Solution:**
- Keep `modifiers: ModMask` for X11 compatibility (fast matching)
- Add `alt_side: ModifierSide` for additional left/right requirement
- Clean separation of concerns

#### 2. QueryKeymap for Detection

**Why not track key state?**
- State tracking requires: KeyPress + KeyRelease handling, state fields, synchronization
- Complex, potential race conditions

**Solution: Use X11's QueryKeymap**
- `QueryKeymap` returns bitmap of all currently pressed keys (256 keycodes)
- Check specific keycode bits for Alt_L and Alt_R
- X11 is single source of truth (No state management needed)
- Called only when Alt side check is required

#### 3. Runtime Keycode Detection

**Why runtime?**
- X11 keycodes are environment-dependent (not hardcoded)
- Alt_L might be keycode 64 on one system, 133 on another

**Solution:**
- Detect Alt_L keycode (keysym 0xffe9) at startup from existing keysym_to_keycode map
- Detect Alt_R keycode (keysym 0xffea) at startup
- Consistent with existing keycode detection approach

#### 4. Backward Compatibility

**Default behavior:**
- `alt_side: ModifierSide::Either` (default)
- Existing "Alt+t" shortcuts automatically work with both Alt keys
- No config migration needed

#### 5. Future Extension Path

**Adding Ctrl left/right:**
```rust
pub struct Shortcut {
    pub modifiers: ModMask,
    pub keycode: u8,
    pub command: String,
    pub alt_side: ModifierSide,
    pub ctrl_side: ModifierSide,  // Just add new field
}
```

Supports complex shortcuts like "Alt_R+Ctrl_L+j" naturally.

### Constraints & Impact

**Module Structure:**
- Primary changes: `src/keyboard.rs` only
- Minor change: `src/window_manager.rs` (pass Connection to handle_key_press)
- No impact on: bsp, config, window_state, window_renderer

**X11 Event Handling:**
- No new event subscriptions
- Enhanced existing KeyPress handling
- QueryKeymap call added (only when needed)
- No KeyRelease handling required

**Logging:**
- Startup: Alt_L/Alt_R keycode detection
- Debug builds: QueryKeymap check results

**BSP Layout:**
- No interaction with layout system

### Configuration Examples

```toml
[shortcuts]
# Backward compatible - matches both Alt keys
"Alt+t" = "spawn xterm"

# Left Alt only
"Alt_L+f" = "spawn firefox"

# Right Alt only
"Alt_R+c" = "spawn chrome"

# Complex: right Alt + left Ctrl (future)
"Ctrl+Alt_R+j" = "special_action"
```

### Matching Flow (Conceptual)

```
User presses: Left Alt + t

1. X11 sends: KeyPressEvent { detail: 28, state: ModMask::M1 }

2. Check "Alt+t" (alt_side: Either)
   - modifiers match: M1 ✓
   - keycode match: 28 ✓
   - alt_side: Either → no check needed ✓
   → EXECUTE

3. Check "Alt_L+t" (alt_side: LeftOnly)
   - modifiers match: M1 ✓
   - keycode match: 28 ✓
   - alt_side: LeftOnly → QueryKeymap
     → Check Alt_L keycode in bitmap ✓
   → EXECUTE (if not already executed)

4. Check "Alt_R+t" (alt_side: RightOnly)
   - modifiers match: M1 ✓
   - keycode match: 28 ✓
   - alt_side: RightOnly → QueryKeymap
     → Check Alt_R keycode in bitmap ✗
   → SKIP
```


## Implementation Plan

### phase 1: Add ModifierSide Enum and Update Shortcut Structure

**Files:** `src/keyboard.rs`

**Changes:**
```rust
// Add new enum
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ModifierSide {
    Either,
    LeftOnly,
    RightOnly,
}

// Update Shortcut struct
pub struct Shortcut {
    pub modifiers: ModMask,
    pub keycode: u8,
    pub command: String,
    pub alt_side: ModifierSide,  // NEW
}
```

**Testing:**
- Unit test: Verify enum derives work correctly
- Unit test: Verify Shortcut creation with alt_side field

### Phase 2: Detect Alt_L/Alt_R Keycodes at Startup

**Files:** `src/keyboard.rs`

**Changes:**
```rust
// Add fields to ShortcutManager
pub struct ShortcutManager {
    // ... existing fields
    alt_l_keycode: Option<u8>,  // NEW
    alt_r_keycode: Option<u8>,  // NEW
}

// In ShortcutManager::new(), after building keysym_to_keycode:
let alt_l_keycode = keysym_to_keycode.get(&0xffe9).copied();
let alt_r_keycode = keysym_to_keycode.get(&0xffea).copied();
info!("Detected Alt_L keycode: {:?}", alt_l_keycode);
info!("Detected Alt_R keycode: {:?}", alt_r_keycode);
```

**Testing:**
- Integration test: Verify keycodes detected correctly in Xephyr
- Edge case: Test when Alt_R is not present on keyboard

### Phase 3: Update Config Parsing

**Files:** `src/keyboard.rs`

**Changes:**
```rust
// Update parse_key_combination signature
fn parse_key_combination(&self, key_combo: &str)
    -> Result<(ModMask, u32, ModifierSide)> {  // Add ModifierSide return

    let mut alt_side = ModifierSide::Either;  // Default

    // In match statement for modifier parsing:
    match part.to_lowercase().as_str() {
        "alt_l" => {
            modifiers |= ModMask::M1;
            alt_side = ModifierSide::LeftOnly;
        }
        "alt_r" => {
            modifiers |= ModMask::M1;
            alt_side = ModifierSide::RightOnly;
        }
        "alt" | "mod1" | "meta" => {
            modifiers |= ModMask::M1;
            alt_side = ModifierSide::Either;
        }
        // ... rest unchanged
    }

    Ok((modifiers, keysym, alt_side))
}

// Update register_shortcut to use alt_side
pub fn register_shortcut(&mut self, key_combo: &str, command: String) -> Result<()> {
    let (modifiers, keysym, alt_side) = self.parse_key_combination(key_combo)?;
    // ... convert keysym to keycode ...
    self.shortcuts.push(Shortcut {
        modifiers,
        keycode,
        command,
        alt_side,  // NEW
    });
    Ok(())
}
```

**Testing:**
- Unit test: Parse "Alt+t" → alt_side: Either
- Unit test: Parse "Alt_L+t" → alt_side: LeftOnly
- Unit test: Parse "Alt_R+t" → alt_side: RightOnly
- Unit test: Parse "Ctrl+c" → alt_side: Either (default)
- Unit test: Parse "Alt_L+Alt_R+t" → error or last wins (define behavior)

### Phase 4: Implement QueryKeymap Check

**Files:** `src/keyboard.rs`

**Changes:**
```rust
// Add helper method
fn query_alt_side_match<C: Connection>(&self, conn: &C, required: ModifierSide) -> Result<bool> {
    match required {
        ModifierSide::Either => Ok(true),  // Always match
        ModifierSide::LeftOnly | ModifierSide::RightOnly => {
            let reply = conn.query_keymap()?.reply()?;
            let keys = reply.keys;

            let alt_l_pressed = self.alt_l_keycode
                .map(|kc| {
                    let byte_idx = (kc / 8) as usize;
                    let bit_idx = kc % 8;
                    keys[byte_idx] & (1 << bit_idx) != 0
                })
                .unwrap_or(false);

            let alt_r_pressed = self.alt_r_keycode
                .map(|kc| {
                    let byte_idx = (kc / 8) as usize;
                    let bit_idx = kc % 8;
                    keys[byte_idx] & (1 << bit_idx) != 0
                })
                .unwrap_or(false);

            #[cfg(debug_assertions)]
            debug!("Alt state: L={}, R={}, required={:?}", alt_l_pressed, alt_r_pressed, required);

            Ok(match required {
                ModifierSide::LeftOnly => alt_l_pressed,
                ModifierSide::RightOnly => alt_r_pressed,
                ModifierSide::Either => unreachable!(),
            })
        }
    }
}
```

**Testing:**
- Unit test: query_alt_side_match with Either always returns true
- Integration test: Verify QueryKeymap detection in Xephyr
- Edge case: Alt_L/Alt_R keycode is None

### Phase 5: Update Event Matching

**Files:** `src/keyboard.rs`, `src/window_manager.rs`

**Changes in keyboard.rs:**
```rust
// Update handle_key_press signature to accept Connection
pub fn handle_key_press<C: Connection>(
    &self,
    conn: &C,
    event: &KeyPressEvent
) -> Result<Option<&str>> {  // Changed return to Result
    // ... existing matching logic ...

    for shortcut in &self.shortcuts {
        // Existing checks
        if event_modifiers_bits != shortcut.modifiers.bits() { continue; }
        if event.detail != shortcut.keycode { continue; }

        // NEW: Check alt_side if Alt is in modifiers
        if shortcut.modifiers.contains(ModMask::M1) {
            if !self.query_alt_side_match(conn, shortcut.alt_side)? {
                continue;
            }
        }

        return Ok(Some(&shortcut.command));
    }
    Ok(None)
}
```

**Changes in window_manager.rs:**
```rust
// Update call site to pass connection and handle Result
if let Some(command) = self.shortcut_manager.handle_key_press(&self.conn, &event)? {
    self.execute_command(command)?;
}
```

**Testing:**
- Integration test: Press Alt_L+t, verify "Alt_L+t" matches but "Alt_R+t" doesn't
- Integration test: Press Alt_R+t, verify "Alt_R+t" matches but "Alt_L+t" doesn't
- Integration test: Press either Alt+t, verify "Alt+t" matches
- Integration test: "Ctrl+c" works without Alt check

### Phase 6: Update Tests

**Files:** `src/keyboard.rs`, `src/window_manager.rs`

**Changes:**
- Update all existing tests that create Shortcut to include alt_side field
- Add new test cases for left/right Alt distinction
- Add edge case tests (keyboard without Alt_R, etc.)

**Test checklist:**
- [ ] Existing unit tests still pass
- [ ] New parse tests for Alt_L/Alt_R
- [ ] New integration tests for key matching
- [ ] Edge case: keyboard without Alt_R

### Quality Gates

- [x] `cargo fmt` passes
- [x] `cargo clippy --all-targets --all-features -- -D warnings` passes
- [x] `cargo test` all tests pass (66 tests)
- [x] `./test.sh` manual verification with Xephyr
- [x] Documentation: Updated config.example.toml with Alt_L/Alt_R examples
- [x] Logging: Verified Alt_L/Alt_R keycode detection messages


## Verification Results

Verified on 2025-10-01 using Xephyr :5

### Test Configuration
```toml
[shortcuts]
"Alt_L+1" = "xterm"   # Left Alt only
"Alt_R+2" = "xcalc"   # Right Alt only
"Alt+3" = "xlogo"     # Either Alt
```

### Results
**Keycode Detection:**
- Alt_L keycode: 64 ✓
- Alt_R keycode: 108 ✓

**Shortcut Matching:**
- Alt_L+1 → xterm spawned (left Alt only) ✓
- Alt_R+2 → xcalc spawned (right Alt only) ✓
- Alt+3 → xlogo spawned (either Alt) ✓
- Mismatch prevention: Left Alt does not trigger Alt_R shortcuts ✓
- Mismatch prevention: Right Alt does not trigger Alt_L shortcuts ✓

**Quality Checks:**
- `cargo test`: 66 tests passed ✓
- `cargo clippy --all-targets --all-features -- -D warnings`: no warnings ✓
- `cargo fmt`: formatted ✓


## Consequences

### Positive
- **User flexibility**: Users can now assign different actions to left and right Alt keys
- **Backward compatibility**: Existing configurations continue to work without modification
- **Clean implementation**: Separate `alt_side` field keeps concerns separated from X11's ModMask
- **No state management**: QueryKeymap approach avoids complex KeyRelease tracking
- **Extensible design**: Easy to add Ctrl_L/Ctrl_R support in the future by adding `ctrl_side` field

### Negative
- **Slight performance overhead**: QueryKeymap call added for Alt-containing shortcuts (negligible in practice)
- **X11 dependency**: QueryKeymap is X11-specific (acceptable for X11 window manager)

### Neutral
- **Test complexity**: Unit tests can't mock X11 Connection, so full testing requires Xephyr integration tests
- **Alt-only support**: Only Alt key has left/right distinction in this phase (by design)


## Deprecation Reason
N/A

## References
- Related: ADR-008 (X11 Modifier System and Keyboard Shortcuts)
- Related: ADR-007 (X11 Keyboard Mapping Understanding)
- ROADMAP: Input & Shortcuts > Better modifier handling
