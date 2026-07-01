//! Container metadata reading and validation.
//!
//! The types and logic in this module now live in the standalone
//! `firestream-metadata` crate so that runtime consumers (`firestream-healthd`,
//! fleet manifest tooling) can depend on the schema without pulling in
//! firestream-vib's build/orchestration dependency tree.
//!
//! This module preserves the public path `firestream_vib::metadata::*` for
//! source compatibility with prior callers — every public name is re-exported
//! from `firestream_metadata` verbatim.
//!
//! # Metadata Files
//!
//! - `metadata.json` - Container info, build info, package list
//! - `sbom-cyclonedx.json` - CycloneDX 1.5 SBOM
//! - `sbom-spdx.json` - SPDX 2.3 SBOM
//! - `closure.json` - Nix closure tree

pub use firestream_metadata::reader;
pub use firestream_metadata::validator;

pub use firestream_metadata::Error;
pub use firestream_metadata::reader::MetadataReader;
pub use firestream_metadata::validator::{Check, MetadataValidator, ValidationResult};
