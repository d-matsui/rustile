//! Keyboard handling and shortcut management

use anyhow::Result;
use std::collections::HashMap;
use tracing::{error, info};
use x11rb::connection::Connection;
use x11rb::protocol::xproto::*;

use crate::keys::KeyParser;

/// Shortcut information
#[derive(Debug, Clone)]
pub struct Shortcut {
    pub modifiers: ModMask,
    pub keycode: u8,
    pub command: String,
}

/// Manages keyboard mappings and shortcuts
pub struct KeyboardManager {
    /// Map of keysyms to keycodes from X11
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
        let mapping_reply = conn
            .get_keyboard_mapping(min_keycode, max_keycode - min_keycode + 1)?
            .reply()?;

        let keysyms_per_keycode = mapping_reply.keysyms_per_keycode as usize;
        let mut keycode_map = HashMap::new();

        // Build keycode map
        for (index, chunk) in mapping_reply
            .keysyms
            .chunks(keysyms_per_keycode)
            .enumerate()
        {
            let keycode = min_keycode + index as u8;

            // Store first keysym for each keycode (unshifted)
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
        let (modifiers, keysym) = self.key_parser.parse_combination(key_combo)?;
        let keycode = self.get_keycode(keysym)?;

        // Grab the key combination
        conn.grab_key(
            true,
            root_window,
            modifiers,
            keycode,
            GrabMode::ASYNC,
            GrabMode::ASYNC,
        )?;

        // Store the shortcut
        self.shortcuts.push(Shortcut {
            modifiers,
            keycode,
            command: command.to_string(),
        });

        Ok(())
    }

    /// Gets the keycode for a given keysym
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

    /// Gets all registered shortcuts
    pub fn shortcuts(&self) -> &[Shortcut] {
        &self.shortcuts
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
