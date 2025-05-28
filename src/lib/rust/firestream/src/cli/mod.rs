//! CLI module for command-line interface
//!
//! This module provides the command-line argument parsing and execution.

pub mod commands;
pub mod args;

pub use args::Cli;
pub use commands::execute_command;
