# ADR-008: X11 Modifier System and Keyboard Shortcuts

## Status
Accepted

## Context
The modifier system in Rustile is complex and not immediately obvious from the code. Developers struggle to understand:

1. **ModMask bit flags** - Why modifiers use binary flags and how they combine
2. **Multiple aliases** - Why "Super", "Win", "Cmd", "Mod4" all mean the same thing
3. **Runtime matching** - How X11 events are matched to configured shortcuts
4. **Lock key filtering** - Why NumLock doesn't break shortcuts

This lack of clarity makes it difficult to debug keyboard issues or add new modifier support.

## Decision
Document the complete modifier system and shortcut matching process to ensure developers understand how keyboard combinations work from config to execution.

### Examples

#### ModMask Bit Flag System
```rust
// X11 uses bit flags for modifiers (simplified):
ModMask::SHIFT   = 0b00000001  // Bit 0
ModMask::LOCK    = 0b00000010  // Bit 1 (CapsLock)
ModMask::CONTROL = 0b00000100  // Bit 2
ModMask::M1      = 0b00001000  // Bit 3 (Alt/Meta)
ModMask::M2      = 0b00010000  // Bit 4 (NumLock)
ModMask::M3      = 0b00100000  // Bit 5 (ScrollLock)
ModMask::M4      = 0b01000000  // Bit 6 (Super/Win)
ModMask::M5      = 0b10000000  // Bit 7 (ISO_Level3_Shift)

// Combining modifiers uses bitwise OR:
Ctrl+Alt = ModMask::CONTROL | ModMask::M1 = 0b00001100
```

#### Complete Shortcut Flow
```
1. Config: "Super+q" = "quit"
   
2. Parse (KeyParser):
   "Super+q" → (ModMask::M4, 0x0071)
   
3. Register (KeyboardManager):
   - Convert: 0x0071 → keycode 24
   - Grab: grab_key(M4, 24)
   - Store: Shortcut { modifiers: M4, keycode: 24, command: "quit" }
   
4. Runtime (User presses Super+Q):
   - X11 sends: KeyPressEvent { keycode: 24, state: 0b01000000 }
   - Match: state has M4 bit? ✓ keycode is 24? ✓
   - Execute: "quit"
```

#### Modifier Aliases
```rust
// All these become ModMask::M4:
"Super" | "Mod4" | "Win" | "Windows" | "Cmd" → ModMask::M4

// All these become ModMask::M1:
"Alt" | "Mod1" | "Meta" → ModMask::M1

// Why? Different platforms use different names for the same key
```

## Consequences

### Positive
- Clear understanding of modifier system prevents bugs
- Developers can debug why shortcuts do/don't trigger
- Easy to add new modifier aliases if needed
- Lock key filtering logic becomes obvious

### Negative
- Additional complexity to understand before modifying keyboard code
- Must understand bit operations for modifier combinations

### Neutral
- No functional changes, only documentation
- Follows established X11 standards

## Implementation Details

### Lock Key Filtering
```rust
// In handle_key_press, we only check relevant modifiers:
let relevant_modifiers = ModMask::SHIFT | ModMask::CONTROL | 
                        ModMask::M1 | ModMask::M4;
let event_modifiers = event.state & relevant_modifiers;

// This filters out lock keys (NumLock, CapsLock, ScrollLock)
// So "Ctrl+A" works whether NumLock is on or off
```

### Why So Many Names?
- **Historical**: Different systems used different names
- **Cross-platform**: Windows key, Command key, Super key
- **X11 generic**: Mod1-Mod5 are position-based names
- **User friendly**: People know "Win" or "Cmd" better than "Mod4"

### Special Cases

#### Hyper Modifier
```rust
"Hyper" → ModMask::M4 | ModMask::M1 | ModMask::CONTROL | ModMask::SHIFT
// All four modifiers at once - rarely used but supported
```

#### Left vs Right
```rust
"Alt_L" | "Alt_R" → both become ModMask::M1
// We don't distinguish left/right modifiers
```

## References
- X11 Protocol: Input Events specification
- `man xmodmap`: Shows modifier mappings
- ModMask values from x11rb protocol definitions