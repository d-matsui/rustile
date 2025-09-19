# ADR-004: X11 Event Registration Strategy

## Status
**Current**: Accepted (2024-08-12)

**History**:
- Proposed: 2024-08-12
- Accepted: 2024-08-12

## Context
Window managers in X11 need to receive various types of events to function properly. However, different events have different registration mechanisms and performance implications. The question is: which events should be registered how, and when?

Key considerations:
1. **Window Manager Privileges**: Only certain events make an application a "window manager"
2. **Performance**: Registering for too many events creates unnecessary overhead
3. **Security**: Some events require special privileges that should be isolated
4. **Modularity**: Event registration should be organized by responsibility

## Decision
Use a **layered event registration strategy** where different event types are registered through different mechanisms:

### Layer 1: Window Manager Core Events (Root Window)
Register on the root window with minimal event mask:
```rust
let event_mask = EventMask::SUBSTRUCTURE_REDIRECT | EventMask::SUBSTRUCTURE_NOTIFY;
conn.change_window_attributes(root, &ChangeWindowAttributesAux::new().event_mask(event_mask))?;
```

### Layer 2: Keyboard Events (Per-Shortcut Registration)
Register specific key combinations through ShortcutManager:
```rust
shortcut_manager.register_shortcuts(&conn, root, config.shortcuts())?;
```

### Layer 3: Focus Events (Automatic via Window Operations)
Receive focus events automatically when managing windows - no explicit registration needed.

### Events by Registration Layer

| Event Type | Registration Method | Rationale |
|------------|-------------------|-----------|
| `MapRequest` | SUBSTRUCTURE_REDIRECT | Core WM privilege - intercept window creation |
| `UnmapNotify` | SUBSTRUCTURE_NOTIFY | Core WM privilege - track window changes |
| `DestroyNotify` | SUBSTRUCTURE_NOTIFY | Core WM privilege - track window lifecycle |
| `ConfigureRequest` | SUBSTRUCTURE_REDIRECT | Core WM privilege - intercept resize requests |
| `KeyPress` | Per-shortcut registration | Performance - only specific combinations |
| `FocusIn/Out` | Automatic with focus operations | No explicit registration needed |
| `EnterNotify` | Automatic with managed windows | No explicit registration needed |

## Consequences

### Positive
- **Minimal Root Registration**: Only essential WM events registered on root window
- **Performance Optimized**: Keyboard events only for configured shortcuts, not all keys
- **Clear Separation**: Window management vs keyboard handling vs focus management
- **Security Isolated**: WM privileges separate from other event handling
- **Modular Design**: Each subsystem handles its own event registration

### Negative
- **Complex Registration Logic**: Multiple registration points instead of single location
- **Documentation Overhead**: Requires explanation of why events come from different sources
- **Debugging Complexity**: Event flow harder to trace across multiple registration points

### Neutral
- **X11 Protocol Compliance**: Follows standard X11 window manager patterns
- **Code Organization**: Events handled in logical modules rather than centralized dispatch
- **Future Extensibility**: Easy to add new event types through appropriate layer

## References
- Implementation: src/window_manager.rs event handling
- Implementation: src/keyboard.rs shortcut registration
- Related: X11 protocol documentation