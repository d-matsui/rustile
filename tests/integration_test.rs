//! Integration tests for rustile window manager

use rustile::config::Config;
use rustile::layout::{Layout, LayoutManager};

#[test]
fn test_library_exports() {
    // Ensure all modules are properly exported
    let _layout = Layout::MasterStack;
    let _layout_manager = LayoutManager::new();
    
    // Config should be accessible
    let config = Config::default();
    assert_eq!(config.master_ratio(), 0.5);
    assert_eq!(config.default_display(), ":1");
}

#[test]
fn test_layout_manager_creation() {
    // Just test that we can create a layout manager
    let _layout_manager = LayoutManager::default();
    
    // We can't access private fields in integration tests,
    // but we can verify it creates successfully
}