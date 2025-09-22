# ADR-012: Configuration Simplification and Auto-Generation Removal

## Status
**Current**: Accepted (2025-09-22)

**History**:
- Proposed: 2024-09-19
- Accepted: 2025-09-22

## Context
Rustile's current configuration system has two interconnected problems that complicate deployment and testing:

### Problem 1: Automatic File Generation
When Rustile starts without a configuration file (`~/.config/rustile/config.toml`), it automatically generates one with default values. This behavior causes:

1. **Unexpected file creation**: Users may not expect the application to create files without explicit permission
2. **Confusing behavior**: The application modifies the filesystem state as a side effect of reading configuration
3. **Testing complications**: Each test environment requires manual config file management

### Problem 2: Display Configuration Complexity
The current config includes `default_display = ":0"` for specifying which X11 display launched applications should use. This creates several issues:

1. **Environment mismatch**: The hardcoded `:0` is incorrect for:
   - Xephyr testing (typically needs `:10`)
   - TTY sessions (typically needs `:1` or `:2`)
   - SSH forwarding (typically needs `localhost:10.0`)

2. **Redundant configuration**: In X11, child processes naturally inherit the parent's `DISPLAY` environment variable, making explicit configuration unnecessary in most cases

3. **Confusing semantics**: The setting controls where *launched applications* connect, not where Rustile itself connects (which is determined by Rustile's own `DISPLAY` environment variable)

### Current Implementation (src/config.rs)
```rust
pub fn load() -> Result<Self> {
    let config_path = Self::config_path()?;

    if config_path.exists() {
        // Load from file
        let content = std::fs::read_to_string(&config_path)?;
        let config: Config = toml::from_str(&content)?;
        config.validate()?;
        Ok(config)
    } else {
        // Creates default and SAVES TO FILE (problematic)
        info!("Config file not found, creating default config at: {:?}", config_path);
        let default_config = Self::default();
        default_config.save()?;  // <-- This line causes the issue
        Ok(default_config)
    }
}
```

## Decision
Simplify configuration management through two complementary changes:

### Phase 1: Remove Automatic File Generation
1. **Use in-memory defaults**: When no config file exists, use default values without creating a file
2. **Explicit file creation only**: Only create config files when explicitly requested (future CLI command consideration)
3. **Clean separation**: Reading configuration and creating files become separate operations

### Phase 2: Remove Display Configuration
1. **Delete `default_display` setting**: Remove from config structure entirely
2. **Use environment variable inheritance**: Child processes automatically inherit parent's `DISPLAY` environment variable
3. **Rely on X11 natural behavior**: No explicit display configuration needed for launched applications

### Implementation

#### Phase 1: Simplified Config Loading
```rust
// src/config.rs
pub fn load() -> Result<Self> {
    let config_path = Self::config_path()?;

    if config_path.exists() {
        info!("Loading config from: {:?}", config_path);
        let content = std::fs::read_to_string(&config_path)?;
        let config: Config = toml::from_str(&content)?;
        config.validate()?;
        Ok(config)
    } else {
        info!("No config file found, using defaults");
        Ok(Self::default())  // No file creation
    }
}
```

#### Phase 2: Simplified Config Structure
```rust
// Remove default_display from GeneralConfig
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GeneralConfig {
    // default_display: String,  // Removed entirely
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            // No display configuration needed
        }
    }
}
```

#### Application Launching
```rust
// src/window_manager.rs - Child processes inherit DISPLAY automatically
fn spawn_application(&mut self, command: &str) -> Result<()> {
    let mut cmd = Command::new(program);
    // cmd.env("DISPLAY", ...) // No longer needed - automatic inheritance
    cmd.spawn()?;
    Ok(())
}
```

## Consequences

### Positive
- **No unexpected file creation**: File system remains unmodified unless explicitly requested
- **Simplified testing**: No config file management needed for different display environments
- **Natural X11 behavior**: Child processes automatically use the same display as Rustile
- **Reduced configuration surface**: Fewer settings to understand and maintain
- **Better user experience**: Works out-of-the-box in all X11 environments (desktop, TTY, Xephyr, SSH)
- **Cleaner codebase**: Removes display-related configuration logic

### Negative
- **Breaking change**: Existing config files with `default_display` will have unused settings
- **Loss of flexibility**: Cannot force launched applications to use a different display than Rustile (rare use case)
- **Documentation update needed**: README and guides must reflect simplified configuration

### Migration Path
1. ~~Add deprecation notice in v0.9.x about the behavior change~~ (Current version 0.10.0)
2. **Implement new behavior in v1.0.0**:
   - Remove config auto-generation
   - Remove `default_display` from config structure
   - Update documentation to reflect simplified configuration
3. **Backward compatibility**: Existing config files continue to work (unused `default_display` settings are simply ignored)

## References
- Issue: Config file auto-generation causes problems with different display setups
- Related: CLAUDE.md development guidelines on configuration management
