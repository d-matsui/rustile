# ADR-007: X11 Keyboard Mapping Understanding

## Status
Accepted

## Context
The X11 keyboard mapping system causes frequent confusion when implementing keyboard shortcut handling. Developers often misunderstand:

1. **Terminology Confusion**: Keycodes vs keysyms and their numeric representations
2. **Data Structure Mystery**: How X11's flat array of keysyms maps to physical keys
3. **Conversion Flow**: Why we need reverse mapping from keysyms to keycodes

This lack of understanding led to potential bugs in keyboard handling implementation.

## Decision
Document the X11 keyboard mapping system comprehensively to ensure correct implementation and maintenance.

### Key Concepts Clarified

**Important**: Throughout this document, keysym values are shown as both hex numbers and their character representation for clarity:
- `0x0071 ('q')` means the keysym value 0x0071, which represents the character 'q'
- The actual data structures contain only the numbers (u32 values), not characters

#### 1. Three-Level Hierarchy
```
Physical Key → Keycode → Keysym(s)
"Q key"      → 24      → [0x0071 ('q'), 0x0051 ('Q'), 0x0071 ('q'), 0x0051 ('Q')]
```

#### 2. Keysym Representations
Keysyms have two equivalent forms:
- **Symbolic**: `XK_q`, `XK_Return`, `XK_Shift_L` (C header defines)
- **Numeric**: `0x0071`, `0xff0d`, `0xffe1` (what X11 transmits)

Example from X11/keysymdef.h (development header, requires libx11-dev):
```c
#define XK_q  0x0071  /* U+0071 LATIN SMALL LETTER Q */
```

#### 3. X11 Data Structure
```rust
// X11 returns flat array of u32 keysym values
// Example with 4 keysyms per keycode:
[0x0061, 0x0041, 0x0061, 0x0041, 0x0073, 0x0053, 0x0073, 0x0053, ...]
// ('a')   ('A')   ('a')   ('A')   ('s')   ('S')   ('s')   ('S')
// └────── keycode 38 ──────┘      └────── keycode 39 ──────┘

// Position meanings:
// [0]: No modifiers
// [1]: Shift
// [2]: Mode_switch/AltGr (or same as [0] on US keyboards)
// [3]: Shift+Mode_switch (or same as [1] on US keyboards)
```

### Examples

#### Using xmodmap to View Mappings
```bash
$ xmodmap -pke | grep "keycode  24"
keycode  24 = q Q q Q

# This shows the same data our GetKeyboardMappingReply contains
```

#### Using xev to See Runtime Behavior
```bash
# Press 'q' in xev window:
KeyPress event, serial 34, synthetic NO, window 0x2600001,
    state 0x0, keycode 24 (keysym 0x71, q), same_screen YES,
    
# Shows: physical keycode 24 → keysym 0x71 ('q')
```

#### Our Implementation Flow
```rust
// User config: { key = "q", action = "Quit" }
// 
// Conversion process:
"q" → parser.get_keysym("q") → 0x0071 ('q')
0x0071 → keycode_map.get(&0x0071) → 24
conn.grab_key(modifiers, keycode: 24, ...)

// When user presses Q key:
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

### Building the Keycode Map
```rust
// GetKeyboardMappingReply provides flat array of u32 values
let keysyms = [0x0061, 0x0041, 0x0061, 0x0041, ...];  // Numbers, not characters!
let keysyms_per_keycode = 4;

// Chunk and build reverse map
for (index, chunk) in keysyms.chunks(keysyms_per_keycode).enumerate() {
    let keycode = min_keycode + index as u8;
    // Take first keysym (unmodified position)
    if let Some(&keysym) = chunk.first() {
        // Store: keysym value → physical keycode
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
