//! TUI module for text-based user interface
//!
//! This module provides the interactive terminal UI using ratatui.

pub mod app;
pub mod ui;
pub mod event;
pub mod splash;
pub mod deploy_tab;

pub use splash::SplashScreen;

use crate::core::Result;

/// Run the TUI application
pub async fn run() -> Result<()> {
    run_with_options(true).await
}

/// Run the TUI application with options
pub async fn run_with_options(show_splash: bool) -> Result<()> {
    let mut app = app::App::new().await?;
    // TODO: Implement splash screen based on show_splash parameter
    if show_splash {
        // Show splash screen logic would go here
    }
    app.run().await
}
