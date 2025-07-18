//! Keyboard handling and shortcut management

use anyhow::Result;
use std::collections::HashMap;
use tracing::info;
use x11rb::connection::Connection;
use x11rb::protocol::xproto::*;

/// Manages keyboard mappings and shortcuts
pub struct KeyboardManager {
    /// Map of keysyms to keycodes
    keycode_map: HashMap<u32, u8>,
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

        Ok(Self { keycode_map })
    }

    /// Gets the keycode for a given keysym
    pub fn get_keycode(&self, keysym: u32) -> u8 {
        self.keycode_map.get(&keysym).copied().unwrap_or(0)
    }

    /// Grabs a key combination on the given window
    pub fn grab_key<C: Connection>(
        &self,
        conn: &C,
        window: Window,
        modifiers: ModMask,
        keysym: u32,
    ) -> Result<()> {
        let keycode = self.get_keycode(keysym);

        if keycode == 0 {
            return Err(anyhow::anyhow!(
                "Could not find keycode for keysym: {:#x}",
                keysym
            ));
        }

        conn.grab_key(
            true,
            window,
            modifiers,
            keycode,
            GrabMode::ASYNC,
            GrabMode::ASYNC,
        )?;

        info!(
            "Grabbed key: modifiers={:?}, keysym={:#x}, keycode={}",
            modifiers, keysym, keycode
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keycode_not_found() {
        let keyboard_manager = KeyboardManager {
            keycode_map: HashMap::new(),
        };
        
        // Non-existent keysym should return 0
        assert_eq!(keyboard_manager.get_keycode(0x9999), 0);
    }

    #[test]
    fn test_keycode_lookup() {
        let mut keycode_map = HashMap::new();
        keycode_map.insert(0x0074, 28); // XK_T -> keycode 28 (example)
        
        let keyboard_manager = KeyboardManager { keycode_map };
        
        assert_eq!(keyboard_manager.get_keycode(0x0074), 28);
    }
}
