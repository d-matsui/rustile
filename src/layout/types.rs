//! Core types for the layout system

/// Represents different tiling layouts
#[derive(Debug, Clone, Copy)]
pub enum Layout {
    /// Master-stack layout: one master window on the left, stack on the right
    MasterStack,
    /// Binary Space Partitioning layout: recursive splitting of space
    Bsp,
}

/// Represents a split direction in BSP layout
#[derive(Debug, Clone, Copy)]
pub enum SplitDirection {
    Horizontal,
    Vertical,
}

/// Rectangle for BSP layout calculations
#[derive(Debug, Clone, Copy)]
pub(crate) struct BspRect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}