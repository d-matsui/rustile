//! Keyboard shortcut management for X11 window manager

use anyhow::Result;
use std::collections::HashMap;
use tracing::{error, info};
use x11rb::connection::Connection;
use x11rb::protocol::xproto::*;

/// Specifies left/right requirement for modifier keys
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ModifierSide {
    Either,    // Matches left or right (default)
    LeftOnly,  // Only left modifier key
    RightOnly, // Only right modifier key
}

/// Shortcut information
#[derive(Debug, Clone)]
pub struct Shortcut {
    pub modifiers: ModMask,     // Bit flags for Ctrl, Alt, etc.
    pub keycode: u8,            // Physical key position
    pub command: String,        // Command to execute
    pub alt_side: ModifierSide, // Alt left/right requirement
}

/// Manages keyboard shortcuts from configuration to X11 event handling
pub struct ShortcutManager {
    /// Map of keynames to keysym values for parsing config
    /// Example: "q" → 0x0071, "Return" → 0xff0d
    keyname_to_keysym: HashMap<String, u32>,
    /// Map of keysym values to keycodes from X11
    /// Example: 0x0071 ('q') → 24, 0x0061 ('a') → 38
    keysym_to_keycode: HashMap<u32, u8>,
    /// Registered shortcuts
    shortcuts: Vec<Shortcut>,
    /// Detected keycode for left Alt (keysym 0xffe9)
    alt_l_keycode: Option<u8>,
    /// Detected keycode for right Alt (keysym 0xffea)
    alt_r_keycode: Option<u8>,
}

impl ShortcutManager {
    /// Creates a new shortcut manager and initializes keysym-to-keycode mapping
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
        // depending on which modifiers (none, Shift, etc.) are pressed
        let keysyms_per_keycode = mapping_reply.keysyms_per_keycode as usize;
        let mut keysym_to_keycode = HashMap::new();

        // Build reverse map: keysym → keycode
        // This allows us to convert keynames to keycodes
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
            // We store: keysym_to_keycode.insert(0x0061, 38)
            // This creates mapping: 0x0061 → keycode 38
            if let Some(&keysym) = chunk.first()
                && keysym != 0
            {
                keysym_to_keycode.insert(keysym, keycode);
            }
        }

        info!(
            "Initialized shortcut manager with {} keycodes",
            keysym_to_keycode.len()
        );

        // Detect Alt_L and Alt_R keycodes for left/right distinction
        let alt_l_keycode = keysym_to_keycode.get(&0xffe9).copied(); // Alt_L keysym
        let alt_r_keycode = keysym_to_keycode.get(&0xffea).copied(); // Alt_R keysym
        info!("Detected Alt_L keycode: {:?}", alt_l_keycode);
        info!("Detected Alt_R keycode: {:?}", alt_r_keycode);

        Ok(Self {
            keyname_to_keysym: Self::build_keysym_table(),
            keysym_to_keycode,
            shortcuts: Vec::new(),
            alt_l_keycode,
            alt_r_keycode,
        })
    }

    /// Builds the keyname-to-keysym mapping table
    fn build_keysym_table() -> HashMap<String, u32> {
        let mut keyname_to_keysym = HashMap::new();

        // Letters: "a" → 0x0061 (97), "b" → 0x0062 (98), etc.
        for c in 'a'..='z' {
            keyname_to_keysym.insert(c.to_string(), c as u32);
        }

        // Numbers: "0" → 0x0030 (48), "1" → 0x0031 (49), etc.
        for c in '0'..='9' {
            keyname_to_keysym.insert(c.to_string(), c as u32);
        }

        // Special keys with their keysym values
        keyname_to_keysym.insert("Return".to_string(), 0xff0d); // Return key
        keyname_to_keysym.insert("Enter".to_string(), 0xff0d); // Same as Return
        keyname_to_keysym.insert("space".to_string(), 0x0020); // Space bar (32)
        keyname_to_keysym.insert("Tab".to_string(), 0xff09); // Tab key
        keyname_to_keysym.insert("Escape".to_string(), 0xff1b); // Escape key
        keyname_to_keysym.insert("BackSpace".to_string(), 0xff08); // Backspace key
        keyname_to_keysym.insert("Delete".to_string(), 0xffff); // Delete key
        keyname_to_keysym.insert("Home".to_string(), 0xff50); // Home key
        keyname_to_keysym.insert("End".to_string(), 0xff57); // End key
        keyname_to_keysym.insert("Page_Up".to_string(), 0xff55); // Page Up key
        keyname_to_keysym.insert("Page_Down".to_string(), 0xff56); // Page Down key
        keyname_to_keysym.insert("Left".to_string(), 0xff51); // Left arrow
        keyname_to_keysym.insert("Up".to_string(), 0xff52); // Up arrow
        keyname_to_keysym.insert("Right".to_string(), 0xff53); // Right arrow
        keyname_to_keysym.insert("Down".to_string(), 0xff54);

        // Function keys: "F1" → 0xffbe, "F2" → 0xffbf, ..., "F12" → 0xffc9
        for i in 1..=12 {
            keyname_to_keysym.insert(format!("F{i}"), 0xffbe + i - 1);
        }

        keyname_to_keysym
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
        // Parse key combination string into modifiers, keysym, and Alt side requirement
        // Example: "Super+q" → (ModMask::M4, 0x0071, ModifierSide::Either)
        // Example: "Alt_L+t" → (ModMask::M1, 0x0074, ModifierSide::LeftOnly)
        let (modifiers, keysym, alt_side) = self.parse_key_combination(key_combo)?;

        // Convert keysym to keycode using our mapping
        // Example: 0x0071 ('q') → keycode 24
        let keycode = self.get_keycode(keysym)?;

        // Tell X11 to send us KeyPress events when this combination is pressed
        // Use SYNC mode to enable event replay for unmatched shortcuts (see ADR-015)
        conn.grab_key(
            true,
            root_window,
            modifiers,
            keycode,
            GrabMode::SYNC,
            GrabMode::SYNC,
        )?;

        // Store the shortcut for later lookup when we receive key events
        self.shortcuts.push(Shortcut {
            modifiers,
            keycode,
            command: command.to_string(),
            alt_side,
        });

        Ok(())
    }

    /// Handles a key press event and returns the command if a shortcut matches
    /// Returns (matched_command, did_match) tuple for AllowEvents decision
    pub fn handle_key_press<C: Connection>(
        &self,
        conn: &C,
        event: &KeyPressEvent,
    ) -> Result<(Option<&str>, bool)> {
        // Filter out lock keys (NumLock, CapsLock, ScrollLock) so they don't break shortcuts
        let relevant_modifiers = ModMask::SHIFT.bits()
            | ModMask::CONTROL.bits()
            | ModMask::M1.bits()
            | ModMask::M4.bits();
        let event_modifiers_bits = event.state.bits() & relevant_modifiers;

        // Match event against stored shortcuts (both modifiers and keycode must match)
        for shortcut in &self.shortcuts {
            if event_modifiers_bits != shortcut.modifiers.bits() {
                continue;
            }
            if event.detail != shortcut.keycode {
                continue;
            }

            // If Alt is in modifiers, check left/right requirement
            if shortcut.modifiers.contains(ModMask::M1)
                && !self.query_alt_side_match(conn, shortcut.alt_side)?
            {
                continue;
            }

            return Ok((Some(&shortcut.command), true));
        }
        Ok((None, false))
    }

    /// Checks if the Alt key side requirement matches current state
    /// Uses X11 QueryKeymap to detect which Alt key is actually pressed
    fn query_alt_side_match<C: Connection>(
        &self,
        conn: &C,
        required: ModifierSide,
    ) -> Result<bool> {
        match required {
            ModifierSide::Either => Ok(true), // Always match
            ModifierSide::LeftOnly | ModifierSide::RightOnly => {
                // Query X11 for all currently pressed keys (256 bits = 32 bytes)
                let reply = conn.query_keymap()?.reply()?;
                let keys = reply.keys;

                // Check if Alt_L is pressed
                let alt_l_pressed = self
                    .alt_l_keycode
                    .map(|kc| {
                        let byte_idx = (kc / 8) as usize;
                        let bit_idx = kc % 8;
                        keys[byte_idx] & (1 << bit_idx) != 0
                    })
                    .unwrap_or(false);

                // Check if Alt_R is pressed
                let alt_r_pressed = self
                    .alt_r_keycode
                    .map(|kc| {
                        let byte_idx = (kc / 8) as usize;
                        let bit_idx = kc % 8;
                        keys[byte_idx] & (1 << bit_idx) != 0
                    })
                    .unwrap_or(false);

                #[cfg(debug_assertions)]
                {
                    use tracing::debug;
                    debug!(
                        "Alt state: L={}, R={}, required={:?}",
                        alt_l_pressed, alt_r_pressed, required
                    );
                }

                Ok(match required {
                    ModifierSide::LeftOnly => alt_l_pressed,
                    ModifierSide::RightOnly => alt_r_pressed,
                    ModifierSide::Either => unreachable!(),
                })
            }
        }
    }

    /// Parses a key combination string like "Super+t" or "Ctrl+Alt+Return"
    /// Returns modifiers, keysym for the key, and Alt left/right requirement
    fn parse_key_combination(&self, combo: &str) -> Result<(ModMask, u32, ModifierSide)> {
        let parts: Vec<&str> = combo.split('+').collect();

        if parts.is_empty() {
            return Err(anyhow::anyhow!("Empty key combination"));
        }

        // Build modifier bit flags by OR-ing each modifier together
        let mut modifiers = ModMask::from(0u16);
        let mut keyname = None;
        let mut alt_side = ModifierSide::Either;

        for part in parts {
            let part = part.trim();
            match part.to_lowercase().as_str() {
                // Primary modifiers (see ADR-008 for why so many aliases)
                "super" | "mod4" | "win" | "windows" | "cmd" => modifiers |= ModMask::M4,
                "alt" | "mod1" | "meta" => {
                    modifiers |= ModMask::M1;
                    alt_side = ModifierSide::Either;
                }
                "alt_l" => {
                    modifiers |= ModMask::M1;
                    alt_side = ModifierSide::LeftOnly;
                }
                "alt_r" => {
                    modifiers |= ModMask::M1;
                    alt_side = ModifierSide::RightOnly;
                }
                "ctrl" | "control" | "ctl" => modifiers |= ModMask::CONTROL,
                "shift" => modifiers |= ModMask::SHIFT,

                // Less common modifiers
                "mod2" | "numlock" | "num" => modifiers |= ModMask::M2,
                "mod3" | "scrolllock" | "scroll" => modifiers |= ModMask::M3,
                "mod5" => modifiers |= ModMask::M5,

                // Special combination: all four main modifiers at once
                "hyper" => {
                    modifiers |= ModMask::M4 | ModMask::M1 | ModMask::CONTROL | ModMask::SHIFT
                }
                "super_l" | "super_r" => modifiers |= ModMask::M4,
                "ctrl_l" | "ctrl_r" => modifiers |= ModMask::CONTROL,
                "shift_l" | "shift_r" => modifiers |= ModMask::SHIFT,

                _ => {
                    if keyname.is_some() {
                        return Err(anyhow::anyhow!("Multiple keys specified: {}", combo));
                    }
                    keyname = Some(part);
                }
            }
        }

        let keyname = keyname.ok_or_else(|| anyhow::anyhow!("No key specified in: {}", combo))?;
        let keysym = self.get_keysym(keyname)?;

        Ok((modifiers, keysym, alt_side))
    }

    /// Gets the keysym for a given keyname
    fn get_keysym(&self, keyname: &str) -> Result<u32> {
        // Try exact match first
        if let Some(&keysym) = self.keyname_to_keysym.get(keyname) {
            return Ok(keysym);
        }

        // Try lowercase
        if let Some(&keysym) = self.keyname_to_keysym.get(&keyname.to_lowercase()) {
            return Ok(keysym);
        }

        Err(anyhow::anyhow!("Unknown keyname: {}", keyname))
    }

    /// Gets the keycode for a given keysym
    /// Example: get_keycode(0x0071) → 24 (the 'q' key's keycode)
    /// Returns error if keysym not found (e.g., modifier keysyms aren't stored)
    fn get_keycode(&self, keysym: u32) -> Result<u8> {
        self.keysym_to_keycode
            .get(&keysym)
            .copied()
            .ok_or_else(|| anyhow::anyhow!("Could not find keycode for keysym: {:#x}", keysym))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper function to create a ShortcutManager for testing
    fn create_test_manager() -> ShortcutManager {
        ShortcutManager {
            keyname_to_keysym: ShortcutManager::build_keysym_table(),
            keysym_to_keycode: HashMap::new(),
            shortcuts: Vec::new(),
            alt_l_keycode: None,
            alt_r_keycode: None,
        }
    }

    // ==================== Key Parsing Tests ====================

    #[test]
    fn test_parse_simple_key() {
        let manager = create_test_manager();
        let (modifiers, keysym, _) = manager.parse_key_combination("t").unwrap();
        assert_eq!(modifiers, ModMask::from(0u16));
        assert_eq!(keysym, 't' as u32);
    }

    #[test]
    fn test_parse_modified_key() {
        let manager = create_test_manager();
        let (modifiers, keysym, _) = manager.parse_key_combination("Super+t").unwrap();
        assert_eq!(modifiers, ModMask::M4);
        assert_eq!(keysym, 't' as u32);
    }

    #[test]
    fn test_parse_multiple_modifiers() {
        let manager = create_test_manager();
        let (modifiers, keysym, _) = manager.parse_key_combination("Ctrl+Alt+Return").unwrap();
        assert_eq!(modifiers, ModMask::CONTROL | ModMask::M1);
        assert_eq!(keysym, 0xff0d);
    }

    #[test]
    fn test_parse_special_key() {
        let manager = create_test_manager();
        let (modifiers, keysym, _) = manager.parse_key_combination("F1").unwrap();
        assert_eq!(modifiers, ModMask::from(0u16));
        assert_eq!(keysym, 0xffbe);
    }

    #[test]
    fn test_unknown_key() {
        let manager = create_test_manager();
        assert!(manager.parse_key_combination("unknown_key").is_err());
    }

    #[test]
    fn test_mod2_modifier() {
        let manager = create_test_manager();
        let (modifiers, keysym, _) = manager.parse_key_combination("Mod2+t").unwrap();
        assert_eq!(modifiers, ModMask::M2);
        assert_eq!(keysym, 't' as u32);
    }

    #[test]
    fn test_numlock_modifier() {
        let manager = create_test_manager();
        let (modifiers, keysym, _) = manager.parse_key_combination("NumLock+Return").unwrap();
        assert_eq!(modifiers, ModMask::M2);
        assert_eq!(keysym, 0xff0d);
    }

    #[test]
    fn test_hyper_modifier() {
        let manager = create_test_manager();
        let (modifiers, keysym, _) = manager.parse_key_combination("Hyper+space").unwrap();
        let expected = ModMask::M4 | ModMask::M1 | ModMask::CONTROL | ModMask::SHIFT;
        assert_eq!(modifiers, expected);
        assert_eq!(keysym, 0x0020);
    }

    #[test]
    fn test_alternative_modifier_names() {
        let manager = create_test_manager();

        // Test cmd as alias for Super
        let (modifiers1, _, _) = manager.parse_key_combination("Cmd+t").unwrap();
        let (modifiers2, _, _) = manager.parse_key_combination("Super+t").unwrap();
        assert_eq!(modifiers1, modifiers2);

        // Test meta as alias for Alt
        let (modifiers1, _, _) = manager.parse_key_combination("Meta+t").unwrap();
        let (modifiers2, _, _) = manager.parse_key_combination("Alt+t").unwrap();
        assert_eq!(modifiers1, modifiers2);

        // Test ctl as alias for Ctrl
        let (modifiers1, _, _) = manager.parse_key_combination("Ctl+t").unwrap();
        let (modifiers2, _, _) = manager.parse_key_combination("Ctrl+t").unwrap();
        assert_eq!(modifiers1, modifiers2);
    }

    #[test]
    fn test_left_right_modifiers() {
        let manager = create_test_manager();

        // Left and right should map to same modifier
        let (mod_l, _, _) = manager.parse_key_combination("Super_L+t").unwrap();
        let (mod_r, _, _) = manager.parse_key_combination("Super_R+t").unwrap();
        let (mod_normal, _, _) = manager.parse_key_combination("Super+t").unwrap();

        assert_eq!(mod_l, ModMask::M4);
        assert_eq!(mod_r, ModMask::M4);
        assert_eq!(mod_normal, ModMask::M4);
    }

    #[test]
    fn test_alt_left_right_side_detection() {
        let manager = create_test_manager();

        // Alt_L should return LeftOnly
        let (modifiers, keysym, alt_side) = manager.parse_key_combination("Alt_L+t").unwrap();
        assert_eq!(modifiers, ModMask::M1);
        assert_eq!(keysym, 't' as u32);
        assert_eq!(alt_side, ModifierSide::LeftOnly);

        // Alt_R should return RightOnly
        let (modifiers, keysym, alt_side) = manager.parse_key_combination("Alt_R+t").unwrap();
        assert_eq!(modifiers, ModMask::M1);
        assert_eq!(keysym, 't' as u32);
        assert_eq!(alt_side, ModifierSide::RightOnly);

        // Alt should return Either
        let (modifiers, keysym, alt_side) = manager.parse_key_combination("Alt+t").unwrap();
        assert_eq!(modifiers, ModMask::M1);
        assert_eq!(keysym, 't' as u32);
        assert_eq!(alt_side, ModifierSide::Either);

        // Non-Alt modifiers should default to Either
        let (modifiers, keysym, alt_side) = manager.parse_key_combination("Ctrl+c").unwrap();
        assert_eq!(modifiers, ModMask::CONTROL);
        assert_eq!(keysym, 'c' as u32);
        assert_eq!(alt_side, ModifierSide::Either);
    }

    #[test]
    fn test_complex_modifier_combinations() {
        let manager = create_test_manager();

        // Test triple modifier
        let (modifiers, keysym, _) = manager
            .parse_key_combination("Ctrl+Alt+Shift+Delete")
            .unwrap();
        let expected = ModMask::CONTROL | ModMask::M1 | ModMask::SHIFT;
        assert_eq!(modifiers, expected);
        assert_eq!(keysym, 0xffff); // Delete key

        // Test quadruple modifier
        let (modifiers, keysym, _) = manager
            .parse_key_combination("Super+Ctrl+Alt+Shift+F12")
            .unwrap();
        let expected = ModMask::M4 | ModMask::CONTROL | ModMask::M1 | ModMask::SHIFT;
        assert_eq!(modifiers, expected);
        assert_eq!(keysym, 0xffc9); // F12 key
    }

    #[test]
    fn test_case_insensitive_modifiers() {
        let manager = create_test_manager();

        let (mod1, _, _) = manager.parse_key_combination("SUPER+t").unwrap();
        let (mod2, _, _) = manager.parse_key_combination("super+t").unwrap();
        let (mod3, _, _) = manager.parse_key_combination("Super+t").unwrap();
        let (mod4, _, _) = manager.parse_key_combination("SuPeR+t").unwrap();

        assert_eq!(mod1, ModMask::M4);
        assert_eq!(mod2, ModMask::M4);
        assert_eq!(mod3, ModMask::M4);
        assert_eq!(mod4, ModMask::M4);
    }

    // ==================== Shortcut Management Tests ====================

    #[test]
    fn test_shortcut_creation() {
        let shortcut = Shortcut {
            modifiers: ModMask::M4,
            keycode: 28,
            command: "xterm".to_string(),
            alt_side: ModifierSide::Either,
        };

        assert_eq!(shortcut.modifiers, ModMask::M4);
        assert_eq!(shortcut.keycode, 28);
        assert_eq!(shortcut.command, "xterm");
    }

    // Note: handle_key_press now requires X11 Connection, so integration testing
    // via Xephyr (./test.sh) is needed for full event matching verification
}
