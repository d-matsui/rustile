//! Key parsing and handling utilities

use x11rb::protocol::xproto::ModMask;
use anyhow::Result;
use std::collections::HashMap;

/// Key combination parser that converts human-readable strings to X11 codes
pub struct KeyParser {
    /// Map of key names to X11 keysyms
    key_names: HashMap<String, u32>,
}

impl KeyParser {
    /// Creates a new key parser with common key mappings
    pub fn new() -> Self {
        let mut key_names = HashMap::new();
        
        // Letters
        for c in 'a'..='z' {
            key_names.insert(c.to_string(), c as u32);
        }
        
        // Numbers
        for c in '0'..='9' {
            key_names.insert(c.to_string(), c as u32);
        }
        
        // Special keys
        key_names.insert("Return".to_string(), 0xff0d);
        key_names.insert("Enter".to_string(), 0xff0d);
        key_names.insert("space".to_string(), 0x0020);
        key_names.insert("Tab".to_string(), 0xff09);
        key_names.insert("Escape".to_string(), 0xff1b);
        key_names.insert("BackSpace".to_string(), 0xff08);
        key_names.insert("Delete".to_string(), 0xffff);
        key_names.insert("Home".to_string(), 0xff50);
        key_names.insert("End".to_string(), 0xff57);
        key_names.insert("Page_Up".to_string(), 0xff55);
        key_names.insert("Page_Down".to_string(), 0xff56);
        key_names.insert("Left".to_string(), 0xff51);
        key_names.insert("Up".to_string(), 0xff52);
        key_names.insert("Right".to_string(), 0xff53);
        key_names.insert("Down".to_string(), 0xff54);
        
        // Function keys
        for i in 1..=12 {
            key_names.insert(format!("F{}", i), 0xffbe + i - 1);
        }
        
        Self { key_names }
    }
    
    /// Parses a key combination string like "Super+t" or "Ctrl+Alt+Return"
    pub fn parse_combination(&self, combo: &str) -> Result<(ModMask, u32)> {
        let parts: Vec<&str> = combo.split('+').collect();
        
        if parts.is_empty() {
            return Err(anyhow::anyhow!("Empty key combination"));
        }
        
        let mut modifiers = ModMask::from(0u16);
        let mut key_name = None;
        
        for part in parts {
            let part = part.trim();
            match part.to_lowercase().as_str() {
                "super" | "mod4" | "win" | "windows" => modifiers |= ModMask::M4,
                "alt" | "mod1" => modifiers |= ModMask::M1,
                "ctrl" | "control" => modifiers |= ModMask::CONTROL,
                "shift" => modifiers |= ModMask::SHIFT,
                _ => {
                    if key_name.is_some() {
                        return Err(anyhow::anyhow!("Multiple keys specified: {}", combo));
                    }
                    key_name = Some(part);
                }
            }
        }
        
        let key_name = key_name.ok_or_else(|| anyhow::anyhow!("No key specified in: {}", combo))?;
        let keysym = self.get_keysym(key_name)?;
        
        Ok((modifiers, keysym))
    }
    
    /// Gets the X11 keysym for a key name
    pub fn get_keysym(&self, key_name: &str) -> Result<u32> {
        // Try exact match first
        if let Some(&keysym) = self.key_names.get(key_name) {
            return Ok(keysym);
        }
        
        // Try lowercase
        if let Some(&keysym) = self.key_names.get(&key_name.to_lowercase()) {
            return Ok(keysym);
        }
        
        Err(anyhow::anyhow!("Unknown key name: {}", key_name))
    }
    
    /// Adds a custom key mapping
    pub fn add_key(&mut self, name: &str, keysym: u32) {
        self.key_names.insert(name.to_string(), keysym);
    }
}

impl Default for KeyParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_key() {
        let parser = KeyParser::new();
        let (modifiers, keysym) = parser.parse_combination("t").unwrap();
        assert_eq!(modifiers, ModMask::from(0u16));
        assert_eq!(keysym, 't' as u32);
    }

    #[test]
    fn test_parse_modified_key() {
        let parser = KeyParser::new();
        let (modifiers, keysym) = parser.parse_combination("Super+t").unwrap();
        assert_eq!(modifiers, ModMask::M4);
        assert_eq!(keysym, 't' as u32);
    }

    #[test]
    fn test_parse_multiple_modifiers() {
        let parser = KeyParser::new();
        let (modifiers, keysym) = parser.parse_combination("Ctrl+Alt+Return").unwrap();
        assert_eq!(modifiers, ModMask::CONTROL | ModMask::M1);
        assert_eq!(keysym, 0xff0d);
    }

    #[test]
    fn test_parse_special_key() {
        let parser = KeyParser::new();
        let (modifiers, keysym) = parser.parse_combination("F1").unwrap();
        assert_eq!(modifiers, ModMask::from(0u16));
        assert_eq!(keysym, 0xffbe);
    }

    #[test]
    fn test_unknown_key() {
        let parser = KeyParser::new();
        assert!(parser.parse_combination("unknown_key").is_err());
    }
}