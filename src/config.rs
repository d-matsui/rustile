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
