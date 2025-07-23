//! Window layout algorithms for the tiling window manager
//!
//! This module provides different tiling layout algorithms:
//! - Master-Stack: Traditional master window with stack
//! - BSP (Binary Space Partitioning): Recursive window splitting

// Re-export the main public interface
pub use manager::LayoutManager;
pub use types::{Layout, SplitDirection, LayoutParams, ScreenParams, WindowConstraints, LayoutRatios};
pub use traits::LayoutAlgorithm;

// Internal modules
mod algorithms;
mod bsp;
mod constants;
mod manager;
mod master_stack;
mod traits;
mod types;
