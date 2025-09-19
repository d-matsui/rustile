# ADR-007: X11 Keyboard Mapping Understanding

## Status
**Current**: Accepted (2024-08-12)
<!-- Investigation record - understanding document, not implementation decision -->

**History**:
- Documented: 2024-08-12

## Context
The X11 keyboard system causes frequent confusion when implementing keyboard shortcut handling. Developers often misunderstand:

1. **Terminology Confusion**: Keycodes vs keysyms and their numeric representations
2. **Data Structure Mystery**: How X11's flat array of keysyms maps to keycodes
3. **Conversion Flow**: Why we need reverse mapping from keysyms to keycodes

This lack of understanding led to potential bugs in keyboard handling implementation.

## Decision
Document the X11 keyboard system comprehensively to ensure correct implementation and maintenance.

### Key Concepts Clarified

**Important**: This document uses only 3 core terms for clarity:
- **Keynames**: What users type in config ("q", "Return", "Super+q")
- **Keysyms**: Standard numbers for keys (0x0071 for 'q', 0xff0d for Return)
- **Keycodes**: Physical positions on keyboard (24, 36, etc. - varies by layout)

#### 1. The Complete Flow
```
Keyname  → Keysym → Keycode
"q"      → 0x0071 → 24 (varies by keyboard)
"Return" → 0xff0d → 36 (varies by keyboard)
```

#### 2. Keysyms Are Standard Numbers
Keysyms are universal constants (like ASCII codes):
- 0x0071 always means 'q' on every system
- 0xff0d always means Return on every system
- 0xffbe always means F1 on every system

These numbers are defined in X11 standards:
```c
// From X11/keysymdef.h (if installed):
#define XK_q  0x0071  /* Always 'q' everywhere */
```

#### 3. What X11 Provides
```rust
// X11 tells us which keysyms are at which keycodes:
[0x0061, 0x0041, 0x0061, 0x0041, 0x0073, 0x0053, 0x0073, 0x0053, ...]
// ('a')   ('A')   ('a')   ('A')   ('s')   ('S')   ('s')   ('S')
// └────── keycode 38 ──────┘      └────── keycode 39 ──────┘

// Position meanings:
// [0]: No modifiers
// [1]: Shift
// [2]: Mode_switch/ISO_Level3_Shift (or same as [0] on US keyboards)
// [3]: Shift+Mode_switch (or same as [1] on US keyboards)
```

### Examples

#### Using xmodmap to View Keysym-to-Keycode Mapping
```bash
$ xmodmap -pke | grep "keycode  24"
keycode  24 = q Q q Q

# This shows: keycode 24 contains keysyms [0x0071, 0x0051, 0x0071, 0x0051]
```

#### Using xev to See Runtime Behavior
```bash
# Press 'q' in xev window:
KeyPress event, serial 34, synthetic NO, window 0x2600001,
    state 0x0, keycode 24 (keysym 0x71, q), same_screen YES,
    
# Shows: keycode 24 → keysym 0x71 ('q')
```

#### Our Implementation Flow
```rust
// User config: { key = "q", action = "Quit" }
// 
// Step 1: Convert keyname to keysym
"q" → parser.get_keysym("q") → 0x0071

// Step 2: Convert keysym to keycode using X11 mapping
0x0071 → keycode_map.get(&0x0071) → 24

// Step 3: Register keycode with X11
conn.grab_key(modifiers, keycode: 24, ...)

// Runtime: When user presses Q key
X11 sends KeyPressEvent { keycode: 24, ... }
We match: keycode 24 → execute "Quit" action
```

## Alternatives Considered

### Alternative 1: Direct Keycode Configuration
- **Rejected**: Users would need to write `key = 24` instead of `key = "q"`
- **Rationale**: Platform-specific, not portable, poor user experience

### Alternative 2: Runtime Keysym Lookup
- **Rejected**: Looking up keysym for each key event
- **Rationale**: Inefficient, requires reverse lookup on every key press

### Alternative 3: Include All Keysym Positions
- **Rejected**: Store all 4 keysyms per keycode
- **Rationale**: Config uses unmodified keys; positions 2-4 add complexity without benefit

## Implementation Details

### Building the Keysym-to-Keycode Map
```rust
// X11 provides flat array of keysym values
let keysyms = [0x0061, 0x0041, 0x0061, 0x0041, ...];  // Numbers only!
let keysyms_per_keycode = 4;

// Chunk and build reverse map
for (index, chunk) in keysyms.chunks(keysyms_per_keycode).enumerate() {
    let keycode = min_keycode + index as u8;
    // Take first keysym (unmodified position)
    if let Some(&keysym) = chunk.first() {
        // Store: keysym → keycode
        keycode_map.insert(keysym, keycode);  // e.g., 0x0071 → 24
    }
}
```

## Consequences

### Positive
- Clear understanding prevents keyboard handling bugs
- Developers can debug using xmodmap/xev effectively
- Future keyboard features built on solid foundation

### Negative
- Additional complexity to understand before modifying keyboard code
- Requires basic X11 protocol knowledge

### Neutral
- No functional changes, only documentation and comment improvements
- Keyboard handling implementation remains unchanged

## References
- X11 Protocol Specification: Keyboard encoding
- `man xmodmap`: Keyboard modifier and keymap utility
- `man xev`: X event viewer utility
