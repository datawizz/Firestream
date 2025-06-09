use crate::app::App;

pub mod app;
pub mod event;
pub mod ui;
pub mod models;
pub mod backend;
pub mod services;
pub mod views;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    // Install color-eyre hooks for better error reporting
    color_eyre::install()?;
    
    // Initialize terminal
    let terminal = ratatui::init();
    
    // Create and run the application
    let app = App::new();
    let result = app.run(terminal).await;
    
    // Restore terminal on exit
    ratatui::restore();
    
    result
}
