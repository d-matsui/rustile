//! Configuration system for the window manager

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::info;

/// Main configuration structure
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    /// Keyboard shortcuts mapping
    pub shortcuts: HashMap<String, String>,
    /// Layout configuration
    pub layout: LayoutConfig,
    /// General settings
    pub general: GeneralConfig,
}

/// Layout-related configuration
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LayoutConfig {
    /// Layout algorithm to use ("master_stack" or "bsp")
    #[serde(default = "default_layout_algorithm")]
    pub layout_algorithm: String,
    /// Master window ratio (0.0 to 1.0)
    pub master_ratio: f32,
    /// Gap between windows in pixels
    pub gap: u32,
    /// Border width in pixels
    pub border_width: u32,
    /// Focused window border color (hex format, e.g., 0xFF0000 for red)
    pub focused_border_color: u32,
    /// Unfocused window border color (hex format, e.g., 0x808080 for gray)
    pub unfocused_border_color: u32,
}

fn default_layout_algorithm() -> String {
    "master_stack".to_string()
}

/// General application configuration
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GeneralConfig {
    /// Default display for launching applications
    pub default_display: String,
}

impl Default for Config {
    fn default() -> Self {
        let mut shortcuts = HashMap::new();

        // Default keyboard shortcuts
        shortcuts.insert("Super+t".to_string(), "xcalc".to_string());
        shortcuts.insert("Super+Return".to_string(), "xterm".to_string());
        shortcuts.insert("Super+d".to_string(), "dmenu_run".to_string());

        Self {
            shortcuts,
            layout: LayoutConfig::default(),
            general: GeneralConfig::default(),
        }
    }
}

impl Default for LayoutConfig {
    fn default() -> Self {
        Self {
            layout_algorithm: default_layout_algorithm(),
            master_ratio: 0.5,
            gap: 0,
            border_width: 2,
            focused_border_color: 0xFF0000,   // Red
            unfocused_border_color: 0x808080, // Gray
        }
    }
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            default_display: ":1".to_string(),
        }
    }
}

impl Config {
    /// Loads configuration from file, creates default if not found
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()?;

        if config_path.exists() {
            info!("Loading config from: {:?}", config_path);
            let content = std::fs::read_to_string(&config_path)?;
            let config: Config = toml::from_str(&content)?;
            config.validate()?;
            Ok(config)
        } else {
            info!(
                "Config file not found, creating default config at: {:?}",
                config_path
            );
            let default_config = Self::default();
            default_config.save()?;
            Ok(default_config)
        }
    }

    /// Saves current configuration to file
    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path()?;

        // Create config directory if it doesn't exist
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let content = toml::to_string_pretty(self)?;
        std::fs::write(&config_path, content)?;
        info!("Saved config to: {:?}", config_path);
        Ok(())
    }

    /// Gets the config file path
    fn config_path() -> Result<std::path::PathBuf> {
        let config_dir =
            dirs::config_dir().ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?;

        Ok(config_dir.join("rustile").join("config.toml"))
    }

    /// Validates the configuration
    fn validate(&self) -> Result<()> {
        // Validate master ratio
        if self.layout.master_ratio <= 0.0 || self.layout.master_ratio > 1.0 {
            return Err(anyhow::anyhow!(
                "master_ratio must be between 0.0 and 1.0, got: {}",
                self.layout.master_ratio
            ));
        }

        // Validate gap size
        if self.layout.gap > 500 {
            return Err(anyhow::anyhow!(
                "gap is too large (max 500 pixels), got: {}",
                self.layout.gap
            ));
        }

        // Validate border width
        if self.layout.border_width > 50 {
            return Err(anyhow::anyhow!(
                "border_width is too large (max 50 pixels), got: {}",
                self.layout.border_width
            ));
        }

        // Validate gap + border combination
        if self.layout.gap + self.layout.border_width > 600 {
            return Err(anyhow::anyhow!(
                "gap ({}) + border_width ({}) is too large (max combined 600px)",
                self.layout.gap,
                self.layout.border_width
            ));
        }

        // Validate shortcuts
        for (key_combo, command) in &self.shortcuts {
            if key_combo.is_empty() {
                return Err(anyhow::anyhow!("Empty key combination"));
            }
            if command.is_empty() {
                return Err(anyhow::anyhow!("Empty command for key: {}", key_combo));
            }
        }

        Ok(())
    }

    /// Gets the master ratio for layout calculations
    pub fn master_ratio(&self) -> f32 {
        self.layout.master_ratio
    }

    /// Gets the default display for launching applications
    pub fn default_display(&self) -> &str {
        &self.general.default_display
    }

    /// Gets all configured shortcuts
    pub fn shortcuts(&self) -> &HashMap<String, String> {
        &self.shortcuts
    }

    /// Gets the border width for windows
    pub fn border_width(&self) -> u32 {
        self.layout.border_width
    }

    /// Gets the focused window border color
    pub fn focused_border_color(&self) -> u32 {
        self.layout.focused_border_color
    }

    /// Gets the unfocused window border color
    pub fn unfocused_border_color(&self) -> u32 {
        self.layout.unfocused_border_color
    }

    /// Gets the gap between windows
    pub fn gap(&self) -> u32 {
        self.layout.gap
    }

    /// Gets the layout algorithm to use
    pub fn layout_algorithm(&self) -> &str {
        &self.layout.layout_algorithm
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert!(config.layout.master_ratio > 0.0);
        assert!(config.layout.master_ratio <= 1.0);
        assert!(!config.shortcuts.is_empty());
    }

    #[test]
    fn test_config_validation() {
        let mut config = Config::default();

        // Valid config should pass
        assert!(config.validate().is_ok());

        // Invalid master ratio should fail
        config.layout.master_ratio = 1.5;
        assert!(config.validate().is_err());

        // Reset and test empty shortcut
        config.layout.master_ratio = 0.5;
        config.shortcuts.insert("".to_string(), "test".to_string());
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_gap_validation() {
        let mut config = Config::default();

        // Valid gap should pass
        config.layout.gap = 10;
        assert!(config.validate().is_ok());

        // Large gap should fail
        config.layout.gap = 600;
        assert!(config.validate().is_err());

        // Valid border width should pass
        config.layout.gap = 0;
        config.layout.border_width = 5;
        assert!(config.validate().is_ok());

        // Large border width should fail
        config.layout.border_width = 100;
        assert!(config.validate().is_err());

        // Gap + border combination too large should fail
        config.layout.gap = 400;
        config.layout.border_width = 250;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_accessors() {
        let config = Config::default();
        assert_eq!(config.master_ratio(), 0.5);
        assert_eq!(config.default_display(), ":1");
        assert_eq!(config.gap(), 0);
        assert_eq!(config.border_width(), 2);
        assert_eq!(config.focused_border_color(), 0xFF0000);
        assert_eq!(config.unfocused_border_color(), 0x808080);
        assert!(!config.shortcuts().is_empty());
    }
}
