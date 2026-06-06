//! Schema/spec types shared between the firestream metadata generator
//! (build-time, in `firestream-vib`) and runtime consumers (`firestream-healthd`,
//! fleet manifest tooling).
//!
//! This module is pure data — no IO, no Nix invocation, no async. It owns the
//! type definitions for:
//!
//! - The generator input config: [`MetadataConfig`]
//! - The firestream-native `closure.json` document: [`ClosureJson`],
//!   [`ClosureSummary`], [`ClosureNode`], [`PathEntry`]
//! - CycloneDX 1.5 BoM types: [`CycloneDxBom`] and friends
//! - SPDX 2.3 document types: [`SpdxDocument`] and friends
//!
//! The closure-graph *parser* (`ClosureGraph`) and the generator function
//! itself live in `firestream-vib::nix::closure_graph`; this crate is only
//! responsible for the wire schema.

use serde::{Deserialize, Serialize};

// ============================================================================
// Generator input config
// ============================================================================

/// Configuration for metadata generation (passed from Nix to the generator).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataConfig {
    pub container_name: String,
    pub container_version: String,
    pub main_store_path: String,
    #[serde(default)]
    pub exposed_ports: Vec<u16>,
    #[serde(default = "default_user")]
    pub user: String,
    #[serde(default)]
    pub workdir: String,
    #[serde(default)]
    pub nixpkgs_revision: String,
    #[serde(default)]
    pub flake_uri: String,
    #[serde(default)]
    pub flake_revision: String,
}

fn default_user() -> String {
    "1001:1001".to_string()
}

// ============================================================================
// closure.json structure
// ============================================================================

/// Top-level `closure.json` document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClosureJson {
    pub schema_version: String,
    pub generated_at: String,
    pub root_path: String,
    pub summary: ClosureSummary,
    pub paths: Vec<PathEntry>,
    pub tree: ClosureNode,
}

/// High-level counts for a closure document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClosureSummary {
    pub total_store_paths: usize,
    pub total_references: usize,
    pub unique_packages: usize,
}

/// One Nix store path and its direct references.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathEntry {
    pub path: String,
    pub references: Vec<String>,
    pub reference_count: usize,
}

/// Node in the closure tree (a Nix store path + recursive references).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClosureNode {
    pub path: String,
    pub name: String,
    pub version: Option<String>,
    pub purl: String,
    pub references: Vec<ClosureNode>,
}

// ============================================================================
// CycloneDX 1.5 structures
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CycloneDxBom {
    #[serde(rename = "$schema")]
    pub schema: String,
    #[serde(rename = "bomFormat")]
    pub bom_format: String,
    #[serde(rename = "specVersion")]
    pub spec_version: String,
    #[serde(rename = "serialNumber")]
    pub serial_number: String,
    pub version: u32,
    pub metadata: CycloneDxMetadata,
    pub components: Vec<CycloneDxComponent>,
    pub dependencies: Vec<CycloneDxDependency>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CycloneDxMetadata {
    pub timestamp: String,
    pub tools: CycloneDxTools,
    pub component: CycloneDxMainComponent,
    pub manufacture: CycloneDxManufacture,
    pub licenses: Vec<CycloneDxLicenseWrapper>,
    pub properties: Vec<CycloneDxProperty>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CycloneDxTools {
    pub components: Vec<CycloneDxToolComponent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CycloneDxToolComponent {
    #[serde(rename = "type")]
    pub component_type: String,
    pub name: String,
    pub version: String,
    pub publisher: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CycloneDxMainComponent {
    #[serde(rename = "type")]
    pub component_type: String,
    #[serde(rename = "bom-ref")]
    pub bom_ref: String,
    pub name: String,
    pub version: String,
    pub purl: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CycloneDxManufacture {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CycloneDxLicenseWrapper {
    pub license: CycloneDxLicense,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CycloneDxLicense {
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CycloneDxProperty {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CycloneDxComponent {
    #[serde(rename = "type")]
    pub component_type: String,
    #[serde(rename = "bom-ref")]
    pub bom_ref: String,
    pub name: String,
    pub version: String,
    pub purl: String,
    pub properties: Vec<CycloneDxProperty>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CycloneDxDependency {
    #[serde(rename = "ref")]
    pub dep_ref: String,
    #[serde(rename = "dependsOn")]
    pub depends_on: Vec<String>,
}

// ============================================================================
// SPDX 2.3 structures
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpdxDocument {
    #[serde(rename = "spdxVersion")]
    pub spdx_version: String,
    #[serde(rename = "dataLicense")]
    pub data_license: String,
    #[serde(rename = "SPDXID")]
    pub spdx_id: String,
    pub name: String,
    #[serde(rename = "documentNamespace")]
    pub document_namespace: String,
    #[serde(rename = "creationInfo")]
    pub creation_info: SpdxCreationInfo,
    pub packages: Vec<SpdxPackage>,
    pub relationships: Vec<SpdxRelationship>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpdxCreationInfo {
    pub created: String,
    pub creators: Vec<String>,
    #[serde(rename = "licenseListVersion")]
    pub license_list_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpdxPackage {
    #[serde(rename = "SPDXID")]
    pub spdx_id: String,
    pub name: String,
    #[serde(rename = "versionInfo")]
    pub version_info: String,
    #[serde(rename = "downloadLocation")]
    pub download_location: String,
    #[serde(rename = "filesAnalyzed")]
    pub files_analyzed: bool,
    #[serde(rename = "licenseConcluded")]
    pub license_concluded: String,
    #[serde(rename = "licenseDeclared")]
    pub license_declared: String,
    #[serde(rename = "copyrightText")]
    pub copyright_text: String,
    #[serde(rename = "externalRefs")]
    pub external_refs: Vec<SpdxExternalRef>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpdxExternalRef {
    #[serde(rename = "referenceCategory")]
    pub reference_category: String,
    #[serde(rename = "referenceType")]
    pub reference_type: String,
    #[serde(rename = "referenceLocator")]
    pub reference_locator: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpdxRelationship {
    #[serde(rename = "spdxElementId")]
    pub spdx_element_id: String,
    #[serde(rename = "relationshipType")]
    pub relationship_type: String,
    #[serde(rename = "relatedSpdxElement")]
    pub related_spdx_element: String,
}
