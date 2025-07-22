//! Core window manager functionality

use anyhow::Result;
use std::process::Command;
use tracing::{debug, error, info};
use x11rb::connection::Connection;
use x11rb::protocol::Event;
use x11rb::protocol::xproto::*;

use crate::config::Config;
use crate::keyboard::KeyboardManager;
use crate::layout::LayoutManager;

/// Main window manager structure
pub struct WindowManager<C: Connection> {
    /// X11 connection
    conn: C,
    /// Screen information
    screen_num: usize,
    /// Currently managed windows
    windows: Vec<Window>,
    /// Layout manager for window arrangement
    layout_manager: LayoutManager,
    /// Keyboard manager for shortcuts
    keyboard_manager: KeyboardManager,
    /// Configuration
    config: Config,
}

impl<C: Connection> WindowManager<C> {
    /// Creates a new window manager instance
    pub fn new(conn: C, screen_num: usize) -> Result<Self> {
        // Load configuration
        let config = Config::load()?;
        info!("Loaded configuration with {} shortcuts", config.shortcuts().len());

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

        Ok(Self {
            conn,
            screen_num,
            windows: Vec::new(),
            layout_manager: LayoutManager::new(),
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
            }
        }
    }

    /// Handles a single X11 event
    fn handle_event(&mut self, event: Event) -> Result<()> {
        match event {
            Event::KeyPress(ev) => self.handle_key_press(ev),
            Event::MapRequest(ev) => self.handle_map_request(ev),
            Event::UnmapNotify(ev) => self.handle_unmap_notify(ev),
            Event::ConfigureRequest(ev) => self.handle_configure_request(ev),
            Event::DestroyNotify(ev) => self.handle_destroy_notify(ev),
            _ => {
                debug!("Unhandled event: {:?}", event);
                Ok(())
            }
        }
    }

    /// Handles key press events
    fn handle_key_press(&mut self, event: KeyPressEvent) -> Result<()> {
        if let Some(command) = self.keyboard_manager.handle_key_press(&event) {
            info!("Shortcut pressed, executing: {}", command);
            
            // Parse command (simple implementation, could be improved)
            let parts: Vec<&str> = command.split_whitespace().collect();
            if let Some(program) = parts.first() {
                let mut cmd = Command::new(program);
                
                // Add arguments if any
                if parts.len() > 1 {
                    cmd.args(&parts[1..]);
                }
                
                // Set display environment
                cmd.env("DISPLAY", self.config.default_display());
                
                match cmd.spawn() {
                    Ok(_) => info!("Successfully launched: {}", command),
                    Err(e) => error!("Failed to launch {}: {}", command, e),
                }
            }
        }
        Ok(())
    }

    /// Handles window map requests
    fn handle_map_request(&mut self, event: MapRequestEvent) -> Result<()> {
        let window = event.window;
        info!("Mapping window: {:?}", window);

        // Map the window
        self.conn.map_window(window)?;

        // Add to managed windows
        self.windows.push(window);

        // Apply layout
        self.apply_layout()?;

        Ok(())
    }

    /// Handles window unmap notifications
    fn handle_unmap_notify(&mut self, event: UnmapNotifyEvent) -> Result<()> {
        let window = event.window;
        info!("Unmapping window: {:?}", window);

        // Remove from managed windows
        self.windows.retain(|&w| w != window);

        // Reapply layout
        self.apply_layout()?;

        Ok(())
    }

    /// Handles window configure requests
    fn handle_configure_request(&mut self, event: ConfigureRequestEvent) -> Result<()> {
        debug!("Configure request for window: {:?}", event.window);

        // For now, just honor the request
        // In the future, we might want to be more selective
        let values = ConfigureWindowAux::from_configure_request(&event);
        self.conn.configure_window(event.window, &values)?;

        Ok(())
    }

    /// Handles window destroy notifications
    fn handle_destroy_notify(&mut self, event: DestroyNotifyEvent) -> Result<()> {
        let window = event.window;
        info!("Window destroyed: {:?}", window);

        // Remove from managed windows
        self.windows.retain(|&w| w != window);

        // Reapply layout
        self.apply_layout()?;

        Ok(())
    }

    /// Applies the current layout to all managed windows
    fn apply_layout(&mut self) -> Result<()> {
        let setup = self.conn.setup();
        let screen = &setup.roots[self.screen_num];

        self.layout_manager.apply_layout(
            &self.conn,
            screen,
            &self.windows,
            self.config.master_ratio(),
        )?;

        Ok(())
    }
}
