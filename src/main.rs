//! Rustile - X11 tiling window manager entry point

use anyhow::Result;
use tracing::info;

// Module declarations
mod bsp;
mod config;
mod keyboard;
mod window_manager;
mod window_renderer;
mod window_state;
mod workspace;

use window_manager::WindowManager;

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    info!("Starting Rustile window manager");

    let (conn, screen_num) = x11rb::connect(None)?;
    info!("Connected to X11 display on screen {}", screen_num);

    let wm = WindowManager::new(conn, screen_num)?;
    wm.run()
}
