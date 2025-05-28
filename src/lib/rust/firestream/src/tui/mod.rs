//! TUI module for text-based user interface
//!
//! This module provides the interactive terminal UI using ratatui.

pub mod app;
pub mod ui;
pub mod event;

use crate::core::Result;

/// Run the TUI application
pub async fn run() -> Result<()> {
    let mut app = app::App::new().await?;
    app.run().await
}
