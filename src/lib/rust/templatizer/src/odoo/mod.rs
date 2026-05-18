//! Odoo addons project template generator
//!
//! This module provides functionality for generating self-contained Odoo
//! addons projects that consume Firestream containers via a Nix flake.
//!
//! # Example
//!
//! ```rust,ignore
//! use templatizer::odoo::{OdooConfig, OdooConfigBuilder, OdooGenerator};
//!
//! let config = OdooConfigBuilder::new("my-odoo-addons")
//!     .odoo_version(18)
//!     .organization("Acme Corp")
//!     .build()?;
//!
//! let generator = OdooGenerator::new()?;
//! generator.generate(&config, "/output/path".as_ref())?;
//! ```

pub mod config;
pub mod generator;

pub use config::{OdooConfig, OdooConfigBuilder, get_example};
pub use generator::OdooGenerator;
