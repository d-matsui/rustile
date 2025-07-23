//! Window layout algorithms for the tiling window manager
//!
//! This module provides different tiling layout algorithms:
//! - Master-Stack: Traditional master window with stack
//! - BSP (Binary Space Partitioning): Recursive window splitting

// Re-export the main public interface
pub use manager::LayoutManager;
pub use types::{Layout, SplitDirection};

// Internal modules
mod bsp;
mod manager;
mod master_stack;
mod types;