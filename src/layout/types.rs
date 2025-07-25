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

/// Screen dimensions and constraints for layout calculations
#[derive(Debug, Clone, Copy)]
pub struct ScreenParams {
    pub width: u16,
    pub height: u16,
    pub gap: u32,
}

/// Window size constraints for layout calculations
#[derive(Debug, Clone, Copy)]
pub struct WindowConstraints {
    pub min_width: u32,
    pub min_height: u32,
}

/// Layout ratios and split configuration
#[derive(Debug, Clone, Copy)]
pub struct LayoutRatios {
    pub master_ratio: f32,
    pub bsp_split_ratio: f32,
}

/// Combined parameters for layout operations to reduce function signatures
#[derive(Debug, Clone, Copy)]
pub struct LayoutParams {
    pub screen: ScreenParams,
    pub constraints: WindowConstraints,
    pub ratios: LayoutRatios,
}
