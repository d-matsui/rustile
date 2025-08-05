# ADR-006: ConfigureRequest Timeout Handling

## Status
Accepted

## Context

During a refactoring effort to remove "unused" event handlers, the `ConfigureRequest` handler was removed from the window manager. This caused a significant performance regression where applications like xterm would take 5 seconds to launch and appear.

### Problem Analysis

When applications like xterm start up, they follow this sequence:
1. Create window
2. Send `ConfigureRequest` to set preferred geometry
3. Wait for window manager acknowledgment
4. Send `MapRequest` to make window visible

If the window manager doesn't handle `ConfigureRequest` events, applications will wait for acknowledgment until they timeout (typically 5 seconds) before proceeding.

### Log Evidence

Without ConfigureRequest handler:
```
06:17:59.803 - xterm launched successfully  
06:17:59.821 - CreateNotify (xterm created)
06:17:59.839 - ConfigureRequest (UNHANDLED) ← xterm asks to configure
06:18:04.860 - ConfigureRequest (UNHANDLED) ← xterm asks again (5 seconds later!)
06:18:04.885 - MapRequest finally happens ← xterm gives up and maps
```

The 5-second gap between the first ignored ConfigureRequest and the eventual MapRequest demonstrates the application timeout behavior.

## Decision

We will maintain a `ConfigureRequest` event handler that acknowledges all configure requests, even though our tiling window manager will immediately override the requested geometry with its own BSP layout calculations.

### Implementation

```rust
fn handle_configure_request(&mut self, event: ConfigureRequestEvent) -> Result<()> {
    // Forward the request as-is to acknowledge it (X11 protocol compliance)
    let values = ConfigureWindowAux::from_configure_request(&event);
    self.conn.configure_window(event.window, &values)?;
    Ok(())
}
```

## Consequences

### Positive
- **Eliminates launch delays**: Applications appear instantly instead of waiting 5 seconds
- **X11 protocol compliance**: Proper acknowledgment of client requests
- **Better user experience**: Responsive application launching
- **Maintains tiling behavior**: BSP layout still overrides geometry as intended

### Negative
- **Code maintenance**: Must maintain handler that appears "unused" to casual observers
- **Potential confusion**: Future developers might remove this handler thinking it's unnecessary

### Mitigations
- **Comprehensive documentation**: Detailed comments explaining the timeout behavior
- **ADR documentation**: This decision record for future reference
- **Test coverage**: Integration tests should verify fast application launches

## Alternatives Considered

1. **Remove handler entirely**: Rejected due to 5-second launch delays
2. **Handle selectively**: Rejected as complexity isn't worth marginal benefits
3. **Implement timeout detection**: Rejected as unnecessary complexity

## Implementation Notes

The handler performs pure acknowledgment:
- Extracts requested configuration from the event
- Forwards the request to X11 server unchanged
- Our BSP layout system immediately overrides with calculated geometry
- Applications proceed immediately instead of timing out

This pattern is common in tiling window managers that need to balance X11 protocol compliance with their own layout management.

## Related

- Performance regression discovered during event handler cleanup
- Part of overall window manager architecture simplification
- Demonstrates importance of X11 protocol compliance in tiling WMs