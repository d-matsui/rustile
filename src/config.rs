//! Configuration system for the window manager

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::info;

// === Validation Traits and Utilities ===

/// Trait for validating configuration values
pub trait Validate {
    /// Validates the configuration and returns detailed error information
    fn validate(&self) -> Result<()>;
}

/// Validation utilities for common configuration patterns
pub mod validators {
    use anyhow::Result;

    /// Validates that a ratio is within valid bounds (0.0, 1.0]
    pub fn validate_ratio(value: f32, field_name: &str) -> Result<()> {
        if value <= 0.0 || value > 1.0 {
            return Err(anyhow::anyhow!(
                "{} must be between 0.0 and 1.0, got: {}",
                field_name,
                value
            ));
        }
        Ok(())
    }

    /// Validates that a dimension is within reasonable bounds
    pub fn validate_dimension(value: u32, field_name: &str, min: u32, max: u32) -> Result<()> {
        if value < min || value > max {
            return Err(anyhow::anyhow!(
                "{} must be between {} and {}, got: {}",
                field_name,
                min,
                max,
                value
            ));
        }
        Ok(())
    }

    /// Validates that a combination of values doesn't exceed limits
    pub fn validate_combination<T>(
        value1: T,
        name1: &str,
        value2: T,
        name2: &str,
        max_combined: T,
        description: &str,
    ) -> Result<()>
    where
        T: std::ops::Add<Output = T> + PartialOrd + std::fmt::Display + Copy,
    {
        let combined = value1 + value2;
        if combined > max_combined {
            return Err(anyhow::anyhow!(
                "{} ({}) + {} ({}) is too large (max {}): combined = {}",
                name1,
                value1,
                name2,
                value2,
                description,
                combined
            ));
        }
        Ok(())
    }
}

// === Configuration Structures ===

/// Main configuration structure
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    /// Keyboard shortcuts mapping
    pub shortcuts: HashMap<String, String>,
    /// Layout configuration
    pub layout: LayoutConfig,
}

/// Layout-related configuration
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LayoutConfig {
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

fn default_bsp_split_ratio() -> f32 {
    0.5
}

fn default_min_window_width() -> u32 {
    100 // Default minimum width - can be customized in config
}

fn default_min_window_height() -> u32 {
    50 // Default minimum height - can be customized in config
}


impl Default for Config {
    fn default() -> Self {
        let mut shortcuts = HashMap::new();

        // Default application shortcuts
        shortcuts.insert("Shift+Alt+1".to_string(), "xterm".to_string());
        shortcuts.insert("Shift+Alt+2".to_string(), "emacs".to_string());
        shortcuts.insert("Shift+Alt+3".to_string(), "google-chrome".to_string());

        // Default window management shortcuts
        shortcuts.insert("Alt+j".to_string(), "focus_next".to_string());
        shortcuts.insert("Alt+k".to_string(), "focus_prev".to_string());
        shortcuts.insert("Shift+Alt+j".to_string(), "swap_window_next".to_string());
        shortcuts.insert("Shift+Alt+k".to_string(), "swap_window_prev".to_string());
        shortcuts.insert("Shift+Alt+q".to_string(), "destroy_window".to_string());
        shortcuts.insert("Alt+f".to_string(), "toggle_fullscreen".to_string());
        shortcuts.insert("Alt+r".to_string(), "rotate_windows".to_string());
        shortcuts.insert("Alt+d".to_string(), "toggle_zoom".to_string());

        Self {
            shortcuts,
            layout: LayoutConfig::default(),
        }
    }
}

impl Default for LayoutConfig {
    fn default() -> Self {
        Self {
            bsp_split_ratio: default_bsp_split_ratio(),
            min_window_width: default_min_window_width(),
            min_window_height: default_min_window_height(),
            gap: 10,                          // 10px gap for comfortable spacing
            border_width: 5,                   // 5px for visible borders
            focused_border_color: 0xFF0000,   // Red
            unfocused_border_color: 0x808080, // Gray
        }
    }
}


// === Validation Implementations ===

impl Validate for LayoutConfig {
    fn validate(&self) -> Result<()> {
        // Validate ratios
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

        Ok(())
    }
}


impl Validate for Config {
    fn validate(&self) -> Result<()> {
        // Validate sub-configurations
        self.layout.validate()?;

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
            info!("No config file found, using defaults");
            Ok(Self::default())
        }
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
        assert!(config.layout.bsp_split_ratio > 0.0);
        assert!(config.layout.bsp_split_ratio <= 1.0);
        assert!(!config.shortcuts.is_empty());
    }

    #[test]
    fn test_config_validation() {
        let mut config = Config::default();

        // Valid config should pass
        assert!(config.validate().is_ok());

        // Invalid bsp split ratio should fail
        config.layout.bsp_split_ratio = 1.5;
        assert!(config.validate().is_err());

        // Reset and test empty shortcut
        config.layout.bsp_split_ratio = 0.5;
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
        assert_eq!(config.gap(), 10);
        assert_eq!(config.border_width(), 5);
        assert_eq!(config.focused_border_color(), 0xFF0000);
        assert_eq!(config.unfocused_border_color(), 0x808080);
        assert!(!config.shortcuts().is_empty());
    }
}
