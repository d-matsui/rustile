//! Window layout algorithms for the tiling window manager
//!
//! This module provides BSP (Binary Space Partitioning) layout algorithm

// Re-export the main public interface
pub use bsp::{
    BspTree, WindowGeometry, calculate_bsp_geometries, rebuild_bsp_tree, tile_bsp_windows,
};
pub use types::{LayoutParams, LayoutRatios, ScreenParams, SplitDirection, WindowConstraints};

// Public modules for direct access
pub mod bsp;

// Internal modules
mod constants;
mod types;
