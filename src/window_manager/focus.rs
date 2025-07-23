//! Window focus management and visual indication

use anyhow::Result;
use tracing::{debug, info};
use x11rb::CURRENT_TIME;
use x11rb::connection::Connection;
use x11rb::protocol::xproto::*;

use super::core::WindowManager;

impl<C: Connection> WindowManager<C> {
    /// Sets focus to a specific window
    pub(super) fn set_focus(&mut self, window: Window) -> Result<()> {
        if !self.windows.contains(&window) {
            return Ok(());
        }

        // Set X11 input focus
        self.conn
            .set_input_focus(InputFocus::POINTER_ROOT, window, CURRENT_TIME)?;

        // Update focus state
        self.focused_window = Some(window);

        // Update window stack (MRU order)
        self.window_stack.retain(|&w| w != window);
        self.window_stack.insert(0, window);

        // Update window borders
        self.update_window_borders()?;

        #[cfg(debug_assertions)]
        debug!("Focus set to window: {:?}", window);
        Ok(())
    }

    /// Updates window borders based on focus state
    fn update_window_borders(&self) -> Result<()> {
        for &window in &self.windows {
            let is_focused = self.focused_window == Some(window);
            let border_color = if is_focused {
                self.config.focused_border_color()
            } else {
                self.config.unfocused_border_color()
            };

            let aux = ChangeWindowAttributesAux::new().border_pixel(border_color);

            self.conn.change_window_attributes(window, &aux)?;

            let config_aux = ConfigureWindowAux::new().border_width(self.config.border_width());

            self.conn.configure_window(window, &config_aux)?;
        }
        Ok(())
    }

    /// Focuses the next window in the stack
    pub fn focus_next(&mut self) -> Result<()> {
        if self.windows.is_empty() {
            return Ok(());
        }

        let next_window = if let Some(current) = self.focused_window {
            // Find current window index and move to next
            if let Some(current_idx) = self.windows.iter().position(|&w| w == current) {
                let next_idx = (current_idx + 1) % self.windows.len();
                self.windows[next_idx]
            } else {
                self.windows[0]
            }
        } else {
            self.windows[0]
        };

        self.set_focus(next_window)?;
        info!("Focused next window: {:?}", next_window);
        Ok(())
    }

    /// Focuses the previous window in the stack
    pub fn focus_prev(&mut self) -> Result<()> {
        if self.windows.is_empty() {
            return Ok(());
        }

        let prev_window = if let Some(current) = self.focused_window {
            // Find current window index and move to previous
            if let Some(current_idx) = self.windows.iter().position(|&w| w == current) {
                let prev_idx = if current_idx == 0 {
                    self.windows.len() - 1
                } else {
                    current_idx - 1
                };
                self.windows[prev_idx]
            } else {
                self.windows[0]
            }
        } else {
            self.windows[0]
        };

        self.set_focus(prev_window)?;
        info!("Focused previous window: {:?}", prev_window);
        Ok(())
    }
}