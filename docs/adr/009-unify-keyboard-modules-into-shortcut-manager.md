# ADR-009: Unify Keyboard Modules into ShortcutManager

## Status
**Current**: Accepted (2024-08-12)

**History**:
- Proposed: 2024-08-12
- Accepted: 2024-08-12

## Context
The keyboard module contained two separate structs: `KeyParser` and `KeyboardManager`. Analysis revealed that:

1. **KeyParser was never used independently** - It was only created and owned by KeyboardManager
2. **Artificial separation** - The split didn't represent a real architectural boundary
3. **Naming confusion** - "Manager" suffix was vague and didn't express the actual responsibility
4. **Unnecessary complexity** - Two structs for what is conceptually one responsibility: managing keyboard shortcuts

## Decision
Combine `KeyParser` and `KeyboardManager` into a single `ShortcutManager` struct that handles the complete lifecycle of keyboard shortcuts from configuration parsing to X11 event handling.

### Before: Two Structs
```rust
// Separate parser for config strings
pub struct KeyParser {
    keyname_to_keysym: HashMap<String, u32>,
}

// Manager that owns parser and handles X11
pub struct KeyboardManager {
    keycode_map: HashMap<u32, u8>,
    shortcuts: Vec<Shortcut>,
    key_parser: KeyParser,  // Owned dependency
}
```

### After: One Unified Struct
```rust
// Single struct managing shortcuts end-to-end
pub struct ShortcutManager {
    keyname_to_keysym: HashMap<String, u32>,   // From KeyParser
    keysym_to_keycode: HashMap<u32, u8>,       // From KeyboardManager
    shortcuts: Vec<Shortcut>,                  // From KeyboardManager
}
```

### Method Organization
```rust
impl ShortcutManager {
    // Public API
    pub fn new() -> Result<Self>
    pub fn register_shortcuts() -> Result<()>
    pub fn handle_key_press() -> Option<&str>
    
    // Private helpers (formerly KeyParser methods)
    fn parse_key_combination() -> Result<(ModMask, u32)>
    fn get_keysym() -> Result<u32>
    fn get_keycode() -> Result<u8>
}
```

## Consequences

### Positive
- **Simpler architecture** - One cohesive unit instead of artificial separation
- **Clearer naming** - `ShortcutManager` immediately conveys its purpose
- **Reduced complexity** - No ownership dependencies between structs
- **Better encapsulation** - Parsing logic is properly private
- **Easier testing** - Single struct to test without inter-dependencies

### Negative
- **Larger struct** - Combined struct has more responsibilities
- **API change** - Breaking change for any external users (none currently)

### Neutral
- **Line count unchanged** - Same logic, just reorganized
- **Performance unchanged** - No runtime impact

## Implementation Details

### Migration Steps
1. Move all `KeyParser` methods into `ShortcutManager` as private methods
2. Merge data fields from both structs
3. Update `WindowManager` to use `ShortcutManager`
4. Update all documentation references
5. Preserve all existing tests with minimal changes

### Test Strategy
All existing tests were preserved by:
- Creating a helper function `create_test_manager()` for test setup
- Replacing `KeyParser::new()` with the helper function
- Keeping test logic identical to ensure no regression

## References
- Original separation was premature abstraction
- Follows principle: "Make it work, make it right, make it fast"
- Aligns with Rust's preference for composition when ownership is clear