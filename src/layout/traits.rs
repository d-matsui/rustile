//! Layout traits for extensible tiling algorithms

use anyhow::Result;
use x11rb::connection::Connection;
use x11rb::protocol::xproto::Window;

use super::types::LayoutParams;

/// Trait for window layout algorithms
/// 
/// This trait enables extensible layout algorithms with a common interface.
/// Each layout algorithm can maintain its own state and implement specific
/// tiling behavior while providing consistent integration with the window manager.
pub trait LayoutAlgorithm {
    /// Returns the name/identifier of this layout algorithm
    fn name(&self) -> &'static str;
    
    /// Called when a window is added to the layout
    /// 
    /// # Arguments
    /// * `window` - The new window to add
    /// * `focused_window` - Currently focused window (for split targeting in BSP)
    /// * `params` - Layout parameters (ratios, constraints, etc.)
    fn add_window(&mut self, window: Window, focused_window: Option<Window>, params: &LayoutParams);
    
    /// Called when a window is removed from the layout
    /// 
    /// # Arguments  
    /// * `window` - The window to remove
    fn remove_window(&mut self, window: Window);
    
    /// Applies the layout algorithm to arrange windows
    /// 
    /// # Arguments
    /// * `conn` - X11 connection for window operations
    /// * `windows` - List of managed windows in order
    /// * `focused_window` - Currently focused window
    /// * `params` - Layout parameters (screen size, ratios, constraints, etc.)
    fn apply_layout<C: Connection>(
        &mut self,
        conn: &C,
        windows: &[Window],
        focused_window: Option<Window>,
        params: &LayoutParams,
    ) -> Result<()>;
    
    /// Called when the layout is switched away from this algorithm
    /// 
    /// Allows the layout to clean up any internal state or prepare for deactivation.
    fn on_deactivate(&mut self) {
        // Default implementation does nothing
    }
    
    /// Called when the layout is switched to this algorithm
    /// 
    /// Allows the layout to initialize or reset state when becoming active.
    fn on_activate(&mut self) {
        // Default implementation does nothing  
    }
}