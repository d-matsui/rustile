//! Keyboard handling, key parsing, and shortcut management

use anyhow::Result;
use std::collections::HashMap;
use tracing::{error, info};
use x11rb::connection::Connection;
use x11rb::protocol::xproto::*;

// ==================== Key Parser ====================

/// Key combination parser that converts human-readable strings to X11 codes
pub struct KeyParser {
    /// Map of key names (strings) to X11 keysym values (u32 numbers)
    key_names: HashMap<String, u32>,
}

impl KeyParser {
    /// Creates a new key parser with common key mappings
    pub fn new() -> Self {
        let mut key_names = HashMap::new();

        // Letters: "a" → 0x0061 (97), "b" → 0x0062 (98), etc.
        for c in 'a'..='z' {
            key_names.insert(c.to_string(), c as u32);
        }

        // Numbers: "0" → 0x0030 (48), "1" → 0x0031 (49), etc.
        for c in '0'..='9' {
            key_names.insert(c.to_string(), c as u32);
        }

        // Special keys with their keysym values
        key_names.insert("Return".to_string(), 0xff0d);    // Return key
        key_names.insert("Enter".to_string(), 0xff0d);     // Same as Return
        key_names.insert("space".to_string(), 0x0020);     // Space bar (32)
        key_names.insert("Tab".to_string(), 0xff09);       // Tab key
        key_names.insert("Escape".to_string(), 0xff1b);    // Escape key
        key_names.insert("BackSpace".to_string(), 0xff08); // Backspace key
        key_names.insert("Delete".to_string(), 0xffff);    // Delete key
        key_names.insert("Home".to_string(), 0xff50);      // Home key
        key_names.insert("End".to_string(), 0xff57);       // End key
        key_names.insert("Page_Up".to_string(), 0xff55);   // Page Up key
        key_names.insert("Page_Down".to_string(), 0xff56); // Page Down key
        key_names.insert("Left".to_string(), 0xff51);      // Left arrow
        key_names.insert("Up".to_string(), 0xff52);        // Up arrow
        key_names.insert("Right".to_string(), 0xff53);     // Right arrow
        key_names.insert("Down".to_string(), 0xff54);

        // Function keys: "F1" → 0xffbe, "F2" → 0xffbf, ..., "F12" → 0xffc9
        for i in 1..=12 {
            key_names.insert(format!("F{i}"), 0xffbe + i - 1);
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
                // Primary modifiers
                "super" | "mod4" | "win" | "windows" | "cmd" => modifiers |= ModMask::M4,
                "alt" | "mod1" | "meta" => modifiers |= ModMask::M1,
                "ctrl" | "control" | "ctl" => modifiers |= ModMask::CONTROL,
                "shift" => modifiers |= ModMask::SHIFT,

                // Less common modifiers
                "mod2" | "numlock" | "num" => modifiers |= ModMask::M2,
                "mod3" | "scrolllock" | "scroll" => modifiers |= ModMask::M3,
                "mod5" | "altgr" | "altgraph" => modifiers |= ModMask::M5,

                // Alternative names for common combinations
                "hyper" => {
                    modifiers |= ModMask::M4 | ModMask::M1 | ModMask::CONTROL | ModMask::SHIFT
                }
                "super_l" | "super_r" => modifiers |= ModMask::M4,
                "alt_l" | "alt_r" => modifiers |= ModMask::M1,
                "ctrl_l" | "ctrl_r" => modifiers |= ModMask::CONTROL,
                "shift_l" | "shift_r" => modifiers |= ModMask::SHIFT,

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
}

impl Default for KeyParser {
    fn default() -> Self {
        Self::new()
    }
}

// ==================== Keyboard Manager ====================

/// Shortcut information
#[derive(Debug, Clone)]
pub struct Shortcut {
    pub modifiers: ModMask,
    pub keycode: u8,
    pub command: String,
}

/// Manages keyboard mappings and shortcuts
pub struct KeyboardManager {
    /// Map of keysym values (u32) to physical keycodes (u8) from X11
    /// Example: 0x0071 ('q') → 24, 0x0061 ('a') → 38
    keycode_map: HashMap<u32, u8>,
    /// Registered shortcuts
    shortcuts: Vec<Shortcut>,
    /// Key parser for human-readable key names
    key_parser: KeyParser,
}

impl KeyboardManager {
    /// Creates a new keyboard manager and initializes keymaps
    pub fn new<C: Connection>(conn: &C, setup: &Setup) -> Result<Self> {
        let min_keycode = setup.min_keycode;
        let max_keycode = setup.max_keycode;

        // Get keyboard mapping from X server
        // X11 returns a flat array of keysym numbers (u32 values)
        // Example: [0x0061, 0x0041, 0x0061, 0x0041, 0x0073, 0x0053, ...]
        //           ('a')   ('A')   ('a')   ('A')   ('s')   ('S')
        //           └─────── keycode 38 ────────┘    └── keycode 39 ──┘
        let mapping_reply = conn
            .get_keyboard_mapping(min_keycode, max_keycode - min_keycode + 1)?
            .reply()?;

        // Each physical key can produce multiple symbols
        // Example: keycode 38 → [0x0061 ('a'), 0x0041 ('A'), 0x00e1 ('á'), 0x00c1 ('Á')]
        // depending on which modifiers (none, Shift, AltGr, Shift+AltGr) are pressed
        let keysyms_per_keycode = mapping_reply.keysyms_per_keycode as usize;
        let mut keycode_map = HashMap::new();

        // Build reverse map: keysym value → physical keycode
        // This allows us to convert config strings to physical keycodes
        // Example flow: "q" → 0x0071 → keycode 24
        for (index, chunk) in mapping_reply
            .keysyms
            .chunks(keysyms_per_keycode)
            .enumerate()
        {
            // Calculate the actual keycode for this chunk
            let keycode = min_keycode + index as u8;

            // Store only the first keysym (unmodified position) for each keycode
            // Example: chunk = [0x0061 ('a'), 0x0041 ('A'), 0x00e1 ('á'), 0x00c1 ('Á')]
            // We store: keycode_map.insert(0x0061, 38)
            // This creates mapping: 0x0061 → keycode 38
            if let Some(&keysym) = chunk.first() {
                if keysym != 0 {
                    keycode_map.insert(keysym, keycode);
                }
            }
        }

        info!(
            "Initialized keyboard manager with {} keycodes",
            keycode_map.len()
        );

        Ok(Self {
            keycode_map,
            shortcuts: Vec::new(),
            key_parser: KeyParser::new(),
        })
    }

    /// Registers shortcuts from configuration
    pub fn register_shortcuts<C: Connection>(
        &mut self,
        conn: &C,
        root_window: Window,
        shortcuts_config: &HashMap<String, String>,
    ) -> Result<()> {
        self.shortcuts.clear();

        for (key_combo, command) in shortcuts_config {
            match self.register_shortcut(conn, root_window, key_combo, command) {
                Ok(()) => {
                    info!("Registered shortcut: {} -> {}", key_combo, command);
                }
                Err(e) => {
                    error!("Failed to register shortcut {}: {}", key_combo, e);
                }
            }
        }

        info!("Registered {} shortcuts", self.shortcuts.len());
        Ok(())
    }

    /// Registers a single shortcut
    fn register_shortcut<C: Connection>(
        &mut self,
        conn: &C,
        root_window: Window,
        key_combo: &str,
        command: &str,
    ) -> Result<()> {
        // Parse human-readable key combo into X11 codes
        // Example: "Super+q" → (ModMask::M4, 0x0071)
        let (modifiers, keysym) = self.key_parser.parse_combination(key_combo)?;

        // Convert keysym value to physical keycode using our mapping
        // Example: 0x0071 ('q') → keycode 24
        let keycode = self.get_keycode(keysym)?;

        // Tell X11 to send us KeyPress events when this combination is pressed
        conn.grab_key(
            true,
            root_window,
            modifiers,
            keycode,
            GrabMode::ASYNC,
            GrabMode::ASYNC,
        )?;

        // Store the shortcut for later lookup when we receive key events
        self.shortcuts.push(Shortcut {
            modifiers,
            keycode,
            command: command.to_string(),
        });

        Ok(())
    }

    /// Gets the physical keycode for a given keysym value
    /// Example: get_keycode(0x0071) → 24 (the 'q' key's physical keycode)
    /// Returns error if keysym not found (e.g., modifier keys like Alt_L aren't in map)
    fn get_keycode(&self, keysym: u32) -> Result<u8> {
        self.keycode_map
            .get(&keysym)
            .copied()
            .ok_or_else(|| anyhow::anyhow!("Could not find keycode for keysym: {:#x}", keysym))
    }

    /// Handles a key press event and returns the command if a shortcut matches
    pub fn handle_key_press(&self, event: &KeyPressEvent) -> Option<&str> {
        // Only consider relevant modifiers, ignoring NumLock, CapsLock, ScrollLock
        let relevant_modifiers = ModMask::SHIFT.bits()
            | ModMask::CONTROL.bits()
            | ModMask::M1.bits()
            | ModMask::M4.bits();
        let event_modifiers_bits = event.state.bits() & relevant_modifiers;

        for shortcut in &self.shortcuts {
            if event_modifiers_bits == shortcut.modifiers.bits() && event.detail == shortcut.keycode
            {
                return Some(&shortcut.command);
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== KeyParser Tests ====================

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

    #[test]
    fn test_mod2_modifier() {
        let parser = KeyParser::new();
        let (modifiers, keysym) = parser.parse_combination("Mod2+t").unwrap();
        assert_eq!(modifiers, ModMask::M2);
        assert_eq!(keysym, 't' as u32);
    }

    #[test]
    fn test_numlock_modifier() {
        let parser = KeyParser::new();
        let (modifiers, keysym) = parser.parse_combination("NumLock+Return").unwrap();
        assert_eq!(modifiers, ModMask::M2);
        assert_eq!(keysym, 0xff0d);
    }

    #[test]
    fn test_altgr_modifier() {
        let parser = KeyParser::new();
        let (modifiers, keysym) = parser.parse_combination("AltGr+e").unwrap();
        assert_eq!(modifiers, ModMask::M5);
        assert_eq!(keysym, 'e' as u32);
    }

    #[test]
    fn test_hyper_modifier() {
        let parser = KeyParser::new();
        let (modifiers, keysym) = parser.parse_combination("Hyper+space").unwrap();
        let expected = ModMask::M4 | ModMask::M1 | ModMask::CONTROL | ModMask::SHIFT;
        assert_eq!(modifiers, expected);
        assert_eq!(keysym, 0x0020);
    }

    #[test]
    fn test_alternative_modifier_names() {
        let parser = KeyParser::new();

        // Test cmd as alias for Super
        let (modifiers1, _) = parser.parse_combination("Cmd+t").unwrap();
        let (modifiers2, _) = parser.parse_combination("Super+t").unwrap();
        assert_eq!(modifiers1, modifiers2);

        // Test meta as alias for Alt
        let (modifiers1, _) = parser.parse_combination("Meta+t").unwrap();
        let (modifiers2, _) = parser.parse_combination("Alt+t").unwrap();
        assert_eq!(modifiers1, modifiers2);

        // Test ctl as alias for Ctrl
        let (modifiers1, _) = parser.parse_combination("Ctl+t").unwrap();
        let (modifiers2, _) = parser.parse_combination("Ctrl+t").unwrap();
        assert_eq!(modifiers1, modifiers2);
    }

    #[test]
    fn test_left_right_modifiers() {
        let parser = KeyParser::new();

        // Left and right should map to same modifier
        let (mod_l, _) = parser.parse_combination("Super_L+t").unwrap();
        let (mod_r, _) = parser.parse_combination("Super_R+t").unwrap();
        let (mod_normal, _) = parser.parse_combination("Super+t").unwrap();

        assert_eq!(mod_l, ModMask::M4);
        assert_eq!(mod_r, ModMask::M4);
        assert_eq!(mod_normal, ModMask::M4);
    }

    #[test]
    fn test_complex_modifier_combinations() {
        let parser = KeyParser::new();

        // Test triple modifier
        let (modifiers, keysym) = parser.parse_combination("Ctrl+Alt+Shift+Delete").unwrap();
        let expected = ModMask::CONTROL | ModMask::M1 | ModMask::SHIFT;
        assert_eq!(modifiers, expected);
        assert_eq!(keysym, 0xffff); // Delete key

        // Test quadruple modifier
        let (modifiers, keysym) = parser
            .parse_combination("Super+Ctrl+Alt+Shift+F12")
            .unwrap();
        let expected = ModMask::M4 | ModMask::CONTROL | ModMask::M1 | ModMask::SHIFT;
        assert_eq!(modifiers, expected);
        assert_eq!(keysym, 0xffc9); // F12 key
    }

    #[test]
    fn test_case_insensitive_modifiers() {
        let parser = KeyParser::new();

        let (mod1, _) = parser.parse_combination("SUPER+t").unwrap();
        let (mod2, _) = parser.parse_combination("super+t").unwrap();
        let (mod3, _) = parser.parse_combination("Super+t").unwrap();
        let (mod4, _) = parser.parse_combination("SuPeR+t").unwrap();

        assert_eq!(mod1, ModMask::M4);
        assert_eq!(mod2, ModMask::M4);
        assert_eq!(mod3, ModMask::M4);
        assert_eq!(mod4, ModMask::M4);
    }

    // ==================== KeyboardManager Tests ====================

    #[test]
    fn test_shortcut_creation() {
        let shortcut = Shortcut {
            modifiers: ModMask::M4,
            keycode: 28,
            command: "xterm".to_string(),
        };

        assert_eq!(shortcut.modifiers, ModMask::M4);
        assert_eq!(shortcut.keycode, 28);
        assert_eq!(shortcut.command, "xterm");
    }

    #[test]
    fn test_key_press_matching() {
        let shortcuts = vec![Shortcut {
            modifiers: ModMask::M4,
            keycode: 28,
            command: "xterm".to_string(),
        }];

        let km = KeyboardManager {
            keycode_map: HashMap::new(),
            shortcuts,
            key_parser: KeyParser::new(),
        };

        // Create a mock key press event
        let event = KeyPressEvent {
            response_type: 0,
            detail: 28,
            sequence: 0,
            time: 0,
            root: 0,
            event: 0,
            child: 0,
            root_x: 0,
            root_y: 0,
            event_x: 0,
            event_y: 0,
            state: KeyButMask::from(ModMask::M4.bits()),
            same_screen: true,
        };

        assert_eq!(km.handle_key_press(&event), Some("xterm"));

        // Test non-matching event
        let event2 = KeyPressEvent {
            detail: 29, // Different key
            state: KeyButMask::from(ModMask::M4.bits()),
            ..event
        };

        assert_eq!(km.handle_key_press(&event2), None);
    }
}
