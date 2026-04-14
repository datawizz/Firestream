// Public library exports
pub mod app;
pub mod backend;
pub mod event;
pub mod models;
pub mod services;
pub mod views;
pub mod ui;

// Re-export commonly used types
pub use app::App;
pub use backend::{FirestreamBackend, ApiClient, LocalBackend};
