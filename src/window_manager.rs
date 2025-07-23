//! Core window manager functionality

use anyhow::Result;
use std::process::Command;
use tracing::{debug, error, info};
use x11rb::CURRENT_TIME;
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
    /// Currently focused window
    focused_window: Option<Window>,
    /// Window stack for focus ordering (most recently used first)
    window_stack: Vec<Window>,
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
    fn handle_focus_in(&mut self, event: FocusInEvent) -> Result<()> {
        #[cfg(debug_assertions)]
        debug!("Focus in for window: {:?}", event.event);
        // X11 focus events can be noisy, we mainly rely on our own focus tracking
        Ok(())
    }

    /// Handles focus out events
    fn handle_focus_out(&mut self, event: FocusOutEvent) -> Result<()> {
        #[cfg(debug_assertions)]
        debug!("Focus out for window: {:?}", event.event);
        // X11 focus events can be noisy, we mainly rely on our own focus tracking
        Ok(())
    }

    /// Handles enter notify events (mouse enters window)
    fn handle_enter_notify(&mut self, event: EnterNotifyEvent) -> Result<()> {
        let window = event.event;
        #[cfg(debug_assertions)]
        debug!("Mouse entered window: {:?}", window);

        // Optionally enable focus-follows-mouse
        if self.windows.contains(&window) {
            self.set_focus(window)?;
        }

        Ok(())
    }

    /// Applies the current layout to all managed windows
    fn apply_layout(&mut self) -> Result<()> {
        let setup = self.conn.setup();
        let screen = &setup.roots[self.screen_num];

        self.layout_manager.apply_layout(
            &self.conn,
            &self.windows,
            self.focused_window,
            screen.width_in_pixels,
            screen.height_in_pixels,
            self.config.master_ratio(),
            self.config.layout.bsp_split_ratio,
            self.config.min_window_width(),
            self.config.min_window_height(),
            self.config.gap(),
        )?;

        Ok(())
    }

    /// Sets focus to a specific window
    fn set_focus(&mut self, window: Window) -> Result<()> {
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

    /// Swaps the currently focused window with the master window
    pub fn swap_with_master(&mut self) -> Result<()> {
        if self.windows.len() < 2 {
            return Ok(());
        }

        if let Some(focused) = self.focused_window {
            if let Some(focused_idx) = self.windows.iter().position(|&w| w == focused) {
                if focused_idx != 0 {
                    // Swap with master (index 0)
                    self.windows.swap(0, focused_idx);
                    self.apply_layout()?;
                    info!("Swapped window {:?} with master", focused);
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    // Test helper functions for window management logic
    // These test the core algorithms without requiring X11 connections
    
    /// Helper to test focus_next logic
    fn test_focus_next_logic(windows: &[Window], current: Option<Window>) -> Option<Window> {
        if windows.is_empty() {
            return None;
        }
        
        if let Some(current) = current {
            if let Some(current_idx) = windows.iter().position(|&w| w == current) {
                let next_idx = (current_idx + 1) % windows.len();
                Some(windows[next_idx])
            } else {
                Some(windows[0])
            }
        } else {
            Some(windows[0])
        }
    }
    
    /// Helper to test focus_prev logic
    fn test_focus_prev_logic(windows: &[Window], current: Option<Window>) -> Option<Window> {
        if windows.is_empty() {
            return None;
        }
        
        if let Some(current) = current {
            if let Some(current_idx) = windows.iter().position(|&w| w == current) {
                let prev_idx = if current_idx == 0 {
                    windows.len() - 1
                } else {
                    current_idx - 1
                };
                Some(windows[prev_idx])
            } else {
                Some(windows[0])
            }
        } else {
            Some(windows[0])
        }
    }
    
    /// Helper to test swap_with_master logic
    fn test_swap_with_master_logic(windows: &mut Vec<Window>, focused: Option<Window>) -> bool {
        if windows.len() < 2 {
            return false;
        }
        
        if let Some(focused) = focused {
            if let Some(focused_idx) = windows.iter().position(|&w| w == focused) {
                if focused_idx != 0 {
                    windows.swap(0, focused_idx);
                    return true;
                }
            }
        }
        false
    }
    
    #[test]
    fn test_focus_next_empty_windows() {
        let windows = vec![];
        let result = test_focus_next_logic(&windows, None);
        assert_eq!(result, None);
    }
    
    #[test]
    fn test_focus_next_single_window() {
        let windows = vec![10];
        
        // Should focus the only window when none focused
        let result = test_focus_next_logic(&windows, None);
        assert_eq!(result, Some(10));
        
        // Should stay on same window when already focused
        let result = test_focus_next_logic(&windows, Some(10));
        assert_eq!(result, Some(10));
    }
    
    #[test]
    fn test_focus_next_multiple_windows() {
        let windows = vec![10, 20, 30];
        
        // From no focus, should focus first window
        let result = test_focus_next_logic(&windows, None);
        assert_eq!(result, Some(10));
        
        // From first window, should focus second
        let result = test_focus_next_logic(&windows, Some(10));
        assert_eq!(result, Some(20));
        
        // From second window, should focus third
        let result = test_focus_next_logic(&windows, Some(20));
        assert_eq!(result, Some(30));
        
        // From last window, should wrap to first
        let result = test_focus_next_logic(&windows, Some(30));
        assert_eq!(result, Some(10));
        
        // From non-existent window, should focus first
        let result = test_focus_next_logic(&windows, Some(999));
        assert_eq!(result, Some(10));
    }
    
    #[test]
    fn test_focus_prev_empty_windows() {
        let windows = vec![];
        let result = test_focus_prev_logic(&windows, None);
        assert_eq!(result, None);
    }
    
    #[test]
    fn test_focus_prev_single_window() {
        let windows = vec![10];
        
        // Should focus the only window when none focused
        let result = test_focus_prev_logic(&windows, None);
        assert_eq!(result, Some(10));
        
        // Should stay on same window when already focused
        let result = test_focus_prev_logic(&windows, Some(10));
        assert_eq!(result, Some(10));
    }
    
    #[test]
    fn test_focus_prev_multiple_windows() {
        let windows = vec![10, 20, 30];
        
        // From no focus, should focus first window
        let result = test_focus_prev_logic(&windows, None);
        assert_eq!(result, Some(10));
        
        // From first window, should wrap to last
        let result = test_focus_prev_logic(&windows, Some(10));
        assert_eq!(result, Some(30));
        
        // From second window, should focus first
        let result = test_focus_prev_logic(&windows, Some(20));
        assert_eq!(result, Some(10));
        
        // From last window, should focus second
        let result = test_focus_prev_logic(&windows, Some(30));
        assert_eq!(result, Some(20));
        
        // From non-existent window, should focus first
        let result = test_focus_prev_logic(&windows, Some(999));
        assert_eq!(result, Some(10));
    }
    
    #[test]
    fn test_swap_with_master_empty_windows() {
        let mut windows = vec![];
        let result = test_swap_with_master_logic(&mut windows, None);
        assert!(!result); // No swap should occur
        assert!(windows.is_empty());
    }
    
    #[test]
    fn test_swap_with_master_single_window() {
        let mut windows = vec![10];
        let result = test_swap_with_master_logic(&mut windows, Some(10));
        assert!(!result); // No swap should occur
        assert_eq!(windows, vec![10]);
    }
    
    #[test]
    fn test_swap_with_master_multiple_windows() {
        // Test swapping non-master with master
        let mut windows = vec![10, 20, 30];
        let result = test_swap_with_master_logic(&mut windows, Some(30));
        assert!(result); // Swap should occur
        assert_eq!(windows, vec![30, 20, 10]);
        
        // Test swapping master with master (no-op)
        let mut windows = vec![10, 20, 30];
        let result = test_swap_with_master_logic(&mut windows, Some(10));
        assert!(!result); // No swap should occur
        assert_eq!(windows, vec![10, 20, 30]);
        
        // Test swapping middle window with master
        let mut windows = vec![10, 20, 30];
        let result = test_swap_with_master_logic(&mut windows, Some(20));
        assert!(result); // Swap should occur
        assert_eq!(windows, vec![20, 10, 30]);
        
        // Test non-existent focused window
        let mut windows = vec![10, 20, 30];
        let result = test_swap_with_master_logic(&mut windows, Some(999));
        assert!(!result); // No swap should occur
        assert_eq!(windows, vec![10, 20, 30]);
    }
    
    #[test]
    fn test_focus_cycling_edge_cases() {
        // Test with duplicate windows (should still work)
        let windows = vec![10, 10, 20];
        
        // Should find first occurrence and move correctly
        let result = test_focus_next_logic(&windows, Some(10));
        assert_eq!(result, Some(10)); // Next occurrence of 10
        
        // Test wrapping behavior
        let result = test_focus_next_logic(&windows, Some(20));  
        assert_eq!(result, Some(10)); // Wrap to first
    }
    
    #[test] 
    fn test_window_order_preservation() {
        // Test that window order is preserved correctly during swaps
        let mut windows = vec![1, 2, 3, 4, 5];
        
        // Swap last with master
        test_swap_with_master_logic(&mut windows, Some(5));
        assert_eq!(windows, vec![5, 2, 3, 4, 1]);
        
        // Swap back  
        test_swap_with_master_logic(&mut windows, Some(1));
        assert_eq!(windows, vec![1, 2, 3, 4, 5]);
    }
}
