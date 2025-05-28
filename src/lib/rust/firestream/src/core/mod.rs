//! Core module containing shared functionality
//!
//! This module provides core types and utilities used throughout Firestream.

pub mod error;

pub use error::{FirestreamError, Result, error_to_exit_code};
