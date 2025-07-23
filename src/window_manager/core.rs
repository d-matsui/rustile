//! Core window manager functionality: initialization and main loop

use anyhow::Result;
use tracing::{error, info};
use x11rb::connection::Connection;
use x11rb::protocol::xproto::*;

use crate::config::Config;
use crate::keyboard::KeyboardManager;
use crate::layout::LayoutManager;

/// Main window manager structure
pub struct WindowManager<C: Connection> {
    /// X11 connection
    pub(super) conn: C,
    /// Screen information
    pub(super) screen_num: usize,
    /// Currently managed windows
    pub(super) windows: Vec<Window>,
    /// Currently focused window
    pub(super) focused_window: Option<Window>,
    /// Window stack for focus ordering (most recently used first)
    pub(super) window_stack: Vec<Window>,
    /// Layout manager for window arrangement
    pub(super) layout_manager: LayoutManager,
    /// Keyboard manager for shortcuts
    pub(super) keyboard_manager: KeyboardManager,
    /// Configuration
    pub(super) config: Config,
}

impl<C: Connection> WindowManager<C> {
    /// Creates a new window manager instance
    pub fn new(conn: C, screen_num: usize) -> Result<Self> {
        // Load configuration
        let config = Config::load()?;
        info!(
            "Loaded configuration with {} shortcuts",
            config.shortcuts().len()
        );

        let setup = conn.setup();
        let screen = &setup.roots[screen_num];
        let root = screen.root;

        // Initialize keyboard manager
        let mut keyboard_manager = KeyboardManager::new(&conn, setup)?;

        // Register as window manager
        let event_mask = EventMask::SUBSTRUCTURE_REDIRECT | EventMask::SUBSTRUCTURE_NOTIFY;
        let attributes = ChangeWindowAttributesAux::new().event_mask(event_mask);

        if let Err(e) = conn.change_window_attributes(root, &attributes)?.check() {
            error!("Another window manager is already running: {:?}", e);
            return Err(anyhow::anyhow!(
                "Failed to become window manager. Is another WM running?"
            ));
        }

        info!("Successfully became the window manager");

        // Register keyboard shortcuts from config
        keyboard_manager.register_shortcuts(&conn, root, config.shortcuts())?;

        // Create layout manager with configured algorithm
        let mut layout_manager = LayoutManager::new();
        let layout = match config.layout_algorithm() {
            "bsp" => {
                info!("Using BSP layout algorithm");
                crate::layout::Layout::Bsp
            }
            _ => {
                info!("Using Master-Stack layout algorithm (default)");
                crate::layout::Layout::MasterStack
            }
        };
        layout_manager.set_layout(layout);

        Ok(Self {
            conn,
            screen_num,
            windows: Vec::new(),
            focused_window: None,
            window_stack: Vec::new(),
            layout_manager,
            keyboard_manager,
            config,
        })
    }

    /// Runs the main event loop
    pub fn run(mut self) -> Result<()> {
        info!("Starting window manager event loop");

        loop {
            self.conn.flush()?;
            let event = self.conn.wait_for_event()?;
            
            if let Err(e) = self.handle_event(event) {
                error!("Error handling event: {:?}", e);
                // Continue running despite errors
            }
        }
    }
}