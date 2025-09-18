# ADR-012: Configuration File Handling Improvement

## Status
Proposed

## Context
Currently, when Rustile starts without a configuration file (`~/.config/rustile/config.toml`), it automatically generates one with default values. This behavior has several issues:

1. **Unexpected file creation**: Users may not expect the application to create files without explicit permission
2. **Display number inflexibility**: The auto-generated config uses `default_display = ":0"`, which is incorrect for:
   - Xephyr testing (typically needs `:10`)
   - TTY sessions (typically needs `:1` or `:2`)
   - Multiple X server setups

3. **Confusing behavior**: The application modifies the filesystem state as a side effect of reading configuration

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
Modify the configuration loading behavior to:

1. **Use in-memory defaults**: When no config file exists, use default values without creating a file
2. **Explicit file creation only**: Only create config files when explicitly requested (e.g., via a command-line flag or separate initialization command)
3. **Runtime display detection**: Consider using the `DISPLAY` environment variable as a fallback when no config exists

### Proposed Implementation
```rust
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
        let mut default_config = Self::default();

        // Optionally use DISPLAY environment variable
        if let Ok(display) = std::env::var("DISPLAY") {
            if !display.is_empty() {
                default_config.general.default_display = display;
            }
        }

        Ok(default_config)
    }
}

// Separate method for explicit config creation
pub fn init_config() -> Result<()> {
    let config_path = Self::config_path()?;
    if !config_path.exists() {
        let default_config = Self::default();
        default_config.save()?;
        info!("Created config file at: {:?}", config_path);
    } else {
        info!("Config file already exists at: {:?}", config_path);
    }
    Ok(())
}
```

## Consequences

### Positive
- **No unexpected file creation**: File system remains unmodified unless explicitly requested
- **Flexible display configuration**: Works correctly with different display numbers without manual config editing
- **Cleaner separation of concerns**: Reading configuration and creating files are separate operations
- **Better user experience**: Users can test Rustile without any configuration

### Negative
- **Breaking change**: Users who rely on auto-generation need to explicitly create config
- **Documentation update needed**: README and guides must explain the new behavior

### Migration Path
1. Add deprecation notice in v0.9.x about the behavior change
2. Implement new behavior in v1.0.0
3. Provide clear documentation on creating initial configuration
4. Consider adding `rustile --init-config` command for easy setup

## References
- Issue: Config file auto-generation causes problems with different display setups
- Related: CLAUDE.md development guidelines on configuration management
