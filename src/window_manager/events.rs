//! X11 event handling for the window manager

use anyhow::Result;
use std::process::Command;
#[cfg(debug_assertions)]
use tracing::debug;
use tracing::{error, info};
use x11rb::connection::Connection;
use x11rb::protocol::Event;
use x11rb::protocol::xproto::*;

use super::core::WindowManager;

impl<C: Connection> WindowManager<C> {
    /// Main event dispatcher
    pub(super) fn handle_event(&mut self, event: Event) -> Result<()> {
        match event {
            Event::KeyPress(ev) => self.handle_key_press(ev),
            Event::MapRequest(ev) => self.handle_map_request(ev),
            Event::UnmapNotify(ev) => self.handle_unmap_notify(ev),
            Event::ConfigureRequest(ev) => self.handle_configure_request(ev),
            Event::DestroyNotify(ev) => self.handle_destroy_notify(ev),
            Event::FocusIn(ev) => self.handle_focus_in(ev),
            Event::FocusOut(ev) => self.handle_focus_out(ev),
            Event::EnterNotify(ev) => self.handle_enter_notify(ev),
            _ => {
                #[cfg(debug_assertions)]
                debug!("Unhandled event: {:?}", event);
                Ok(())
            }
        }
    }

    /// Handles key press events
    fn handle_key_press(&mut self, event: KeyPressEvent) -> Result<()> {
        if let Some(command) = self.keyboard_manager.handle_key_press(&event) {
            info!("Shortcut pressed, executing: {}", command);

            // Handle window management commands
            match command {
                "focus_next" => return self.focus_next(),
                "focus_prev" => return self.focus_prev(),
                "swap_with_master" => return self.swap_with_master(),
                "destroy_window" => return self.destroy_focused_window(),
                _ => {
                    // Handle regular application commands
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
            }
        }
        Ok(())
    }

    /// Handles window map requests
    fn handle_map_request(&mut self, event: MapRequestEvent) -> Result<()> {
        let window = event.window;
        info!("Mapping window: {:?}", window);

        // Set initial border attributes before mapping
        let border_aux =
            ChangeWindowAttributesAux::new().border_pixel(self.config.unfocused_border_color());

        self.conn.change_window_attributes(window, &border_aux)?;

        let config_aux = ConfigureWindowAux::new().border_width(self.config.border_width());

        self.conn.configure_window(window, &config_aux)?;

        // Map the window
        self.conn.map_window(window)?;

        // Add to managed windows
        self.windows.push(window);

        // Set focus to new window
        self.set_focus(window)?;

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
        self.window_stack.retain(|&w| w != window);

        // Update focus if focused window was unmapped
        if self.focused_window == Some(window) {
            self.focused_window = self.window_stack.first().copied();
            if let Some(next_focus) = self.focused_window {
                self.set_focus(next_focus)?;
            }
        }

        // Reapply layout
        self.apply_layout()?;

        Ok(())
    }

    /// Handles window configure requests
    fn handle_configure_request(&mut self, event: ConfigureRequestEvent) -> Result<()> {
        #[cfg(debug_assertions)]
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
        self.window_stack.retain(|&w| w != window);

        // Update focus if focused window was destroyed
        if self.focused_window == Some(window) {
            self.focused_window = self.window_stack.first().copied();
            if let Some(next_focus) = self.focused_window {
                self.set_focus(next_focus)?;
            }
        }

        // Reapply layout
        self.apply_layout()?;

        Ok(())
    }

    /// Handles focus in events
    fn handle_focus_in(&mut self, _event: FocusInEvent) -> Result<()> {
        #[cfg(debug_assertions)]
        debug!("Focus in event for window: {:?}", _event.event);
        Ok(())
    }

    /// Handles focus out events
    fn handle_focus_out(&mut self, _event: FocusOutEvent) -> Result<()> {
        #[cfg(debug_assertions)]
        debug!("Focus out event for window: {:?}", _event.event);
        Ok(())
    }

    /// Handles mouse enter events
    fn handle_enter_notify(&mut self, event: EnterNotifyEvent) -> Result<()> {
        let window = event.event;
        #[cfg(debug_assertions)]
        debug!("Mouse entered window: {:?}", window);

        // Only focus if it's a managed window
        if self.windows.contains(&window) {
            self.set_focus(window)?;
        }
        Ok(())
    }
}
