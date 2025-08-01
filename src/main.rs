//! Rustile - A tiling window manager written in Rust
//!
//! Entry point for the window manager. Initializes logging and starts the window manager.

use anyhow::Result;
use tracing::info;

// Module declarations
mod bsp;
mod config;
mod keyboard;
mod window_manager;

use window_manager::WindowManager;

fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    info!("Starting Rustile window manager");

    // Connect to X11 server
    let (conn, screen_num) = x11rb::connect(None)?;
    info!("Connected to X11 display on screen {}", screen_num);

    // Create and run window manager
    let wm = WindowManager::new(conn, screen_num)?;
    wm.run()
}
