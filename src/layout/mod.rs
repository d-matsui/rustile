//! Window layout algorithms for the tiling window manager
//!
//! This module provides BSP (Binary Space Partitioning) layout algorithm

// Re-export the main public interface
pub use manager::LayoutManager;
pub use types::{LayoutParams, LayoutRatios, ScreenParams, SplitDirection, WindowConstraints};

// Internal modules
mod bsp;
mod constants;
mod manager;
mod types;
