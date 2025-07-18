//! Configuration constants and settings for the window manager

use x11rb::protocol::xproto::ModMask;

/// Master window ratio (0.0 to 1.0)
pub const MASTER_RATIO: f32 = 0.5;

/// Default modifier key for shortcuts (Super/Windows key)
pub const MOD_KEY: ModMask = ModMask::M4;

/// Default display for launching applications
pub const DEFAULT_DISPLAY: &str = ":10";

/// X11 keysym for 'T' key
pub const XK_T: u32 = 0x0074;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_master_ratio_valid() {
        // Master ratio should be between 0.0 and 1.0
        assert!(MASTER_RATIO > 0.0);
        assert!(MASTER_RATIO <= 1.0);
    }

    #[test]
    fn test_display_format() {
        // Display should start with :
        assert!(DEFAULT_DISPLAY.starts_with(':'));
        
        // Should be parseable as a number after :
        let display_num = DEFAULT_DISPLAY.trim_start_matches(':');
        assert!(display_num.parse::<u32>().is_ok());
    }

    #[test]
    fn test_keysym_values() {
        // XK_T should match X11 standard keysym
        assert_eq!(XK_T, 0x0074);
    }
}
