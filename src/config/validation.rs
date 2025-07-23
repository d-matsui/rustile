//! Configuration validation traits and utilities

use anyhow::Result;

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

    /// Validates that a string is one of the allowed values
    pub fn validate_choice(value: &str, field_name: &str, allowed: &[&str]) -> Result<()> {
        if !allowed.contains(&value) {
            return Err(anyhow::anyhow!(
                "{} must be one of {:?}, got: '{}'",
                field_name,
                allowed,
                value
            ));
        }
        Ok(())
    }
}
