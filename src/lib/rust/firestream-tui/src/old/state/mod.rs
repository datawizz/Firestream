//! State management module
//!
//! Provides plan/apply workflow with filesystem-based state storage

pub mod schema;
pub mod manager;
pub mod lock;
pub mod diff;

pub use schema::*;
pub use manager::StateManager;
pub use lock::StateLock;
pub use diff::StateDiff;
