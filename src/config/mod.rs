//! Configuration system for the window manager

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::info;

pub mod validation;

use validation::{Validate, validators};

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
    /// BSP split ratio (0.0 to 1.0) - applies to BSP layout
    #[serde(default = "default_bsp_split_ratio")]
    pub bsp_split_ratio: f32,
    /// Minimum window width in pixels
    #[serde(default = "default_min_window_width")]
    pub min_window_width: u32,
    /// Minimum window height in pixels
    #[serde(default = "default_min_window_height")]
    pub min_window_height: u32,
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

fn default_bsp_split_ratio() -> f32 {
    0.5
}

fn default_min_window_width() -> u32 {
    100 // Default minimum width - can be customized in config
}

fn default_min_window_height() -> u32 {
    50 // Default minimum height - can be customized in config
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
            bsp_split_ratio: default_bsp_split_ratio(),
            min_window_width: default_min_window_width(),
            min_window_height: default_min_window_height(),
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
            default_display: ":10".to_string(),
        }
    }
}

// Validation trait implementations

impl Validate for LayoutConfig {
    fn validate(&self) -> Result<()> {
        // Validate ratios
        validators::validate_ratio(self.master_ratio, "master_ratio")?;
        validators::validate_ratio(self.bsp_split_ratio, "bsp_split_ratio")?;

        // Validate dimensions
        validators::validate_dimension(self.gap, "gap", 0, 500)?;
        validators::validate_dimension(self.border_width, "border_width", 0, 50)?;
        validators::validate_dimension(self.min_window_width, "min_window_width", 10, 500)?;
        validators::validate_dimension(self.min_window_height, "min_window_height", 10, 500)?;

        // Validate combinations
        validators::validate_combination(
            self.gap,
            "gap",
            self.border_width,
            "border_width",
            600,
            "600px total",
        )?;

        // Validate layout algorithm choice
        validators::validate_choice(
            &self.layout_algorithm,
            "layout_algorithm",
            &["master_stack", "bsp"],
        )?;

        Ok(())
    }
}

impl Validate for GeneralConfig {
    fn validate(&self) -> Result<()> {
        // Validate display format (simple check for X11 display format)
        if !self.default_display.starts_with(':') && !self.default_display.contains('.') {
            return Err(anyhow::anyhow!(
                "default_display should be in X11 format (e.g., ':0', '192.168.1.1:0.0'), got: '{}'",
                self.default_display
            ));
        }
        Ok(())
    }
}

impl Validate for Config {
    fn validate(&self) -> Result<()> {
        // Validate sub-configurations
        self.layout.validate()?;
        self.general.validate()?;

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

    /// Validates the configuration using the Validate trait
    fn validate(&self) -> Result<()> {
        Validate::validate(self)
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

    /// Gets the minimum window width
    pub fn min_window_width(&self) -> u32 {
        self.layout.min_window_width
    }

    /// Gets the minimum window height
    pub fn min_window_height(&self) -> u32 {
        self.layout.min_window_height
    }

    /// Gets the BSP split ratio
    pub fn bsp_split_ratio(&self) -> f32 {
        self.layout.bsp_split_ratio
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
    fn test_minimum_window_size_validation() {
        let mut config = Config::default();

        // Valid minimum sizes should pass
        config.layout.min_window_width = 100;
        config.layout.min_window_height = 50;
        assert!(config.validate().is_ok());

        // Too small width should fail
        config.layout.min_window_width = 5;
        assert!(config.validate().is_err());

        // Reset and test too large width
        config.layout.min_window_width = 600;
        assert!(config.validate().is_err());

        // Reset and test too small height
        config.layout.min_window_width = 100;
        config.layout.min_window_height = 5;
        assert!(config.validate().is_err());

        // Reset and test too large height
        config.layout.min_window_height = 600;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_bsp_split_ratio_validation() {
        let mut config = Config::default();

        // Valid BSP split ratio should pass
        config.layout.bsp_split_ratio = 0.5;
        assert!(config.validate().is_ok());

        // Edge cases - exactly 0.0 should fail, exactly 1.0 should pass
        config.layout.bsp_split_ratio = 0.0;
        assert!(config.validate().is_err());

        config.layout.bsp_split_ratio = 1.0;
        assert!(config.validate().is_ok());

        // Out of range should fail
        config.layout.bsp_split_ratio = 1.5;
        assert!(config.validate().is_err());

        config.layout.bsp_split_ratio = -0.1;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_accessors() {
        let config = Config::default();
        assert_eq!(config.master_ratio(), 0.5);
        assert_eq!(config.default_display(), ":10");
        assert_eq!(config.gap(), 0);
        assert_eq!(config.border_width(), 2);
        assert_eq!(config.focused_border_color(), 0xFF0000);
        assert_eq!(config.unfocused_border_color(), 0x808080);
        assert!(!config.shortcuts().is_empty());
    }
}
