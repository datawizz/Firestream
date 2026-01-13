//! Nix closure graph parsing and metadata generation
//!
//! Parses `exportReferencesGraph` output and generates container metadata files:
//! - metadata.json: Container configuration and build provenance
//! - sbom-cyclonedx.json: CycloneDX 1.5 SBOM
//! - sbom-spdx.json: SPDX 2.3 SBOM
//! - closure.json: Full Nix closure dependency tree

use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;
use uuid::Uuid;

/// Configuration for metadata generation (passed from Nix)
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

/// Parsed Nix store path components
#[derive(Debug, Clone)]
pub struct StorePath {
    pub full_path: String,
    pub hash: String,
    pub name: String,
    pub version: Option<String>,
}

impl StorePath {
    /// Parse a Nix store path into components
    /// Format: /nix/store/<hash>-<name>-<version>
    pub fn parse(path: &str) -> Self {
        let basename = path.rsplit('/').next().unwrap_or(path);

        // Extract hash (first 32 chars before first dash)
        let parts: Vec<&str> = basename.splitn(2, '-').collect();
        if parts.len() < 2 || parts[0].len() != 32 {
            return StorePath {
                full_path: path.to_string(),
                hash: String::new(),
                name: basename.to_string(),
                version: None,
            };
        }

        let hash = parts[0].to_string();
        let name_version = parts[1];

        // Try to extract version (typically starts with a digit after the last relevant dash)
        // Common patterns: name-1.2.3, name-1.2.3-p1, name-1.2.3_beta
        let version_regex = regex::Regex::new(r"^(.+?)-(\d+[\d.\-_a-zA-Z]*)$").unwrap();
        if let Some(caps) = version_regex.captures(name_version) {
            StorePath {
                full_path: path.to_string(),
                hash,
                name: caps.get(1).map(|m| m.as_str().to_string()).unwrap_or_default(),
                version: caps.get(2).map(|m| m.as_str().to_string()),
            }
        } else {
            StorePath {
                full_path: path.to_string(),
                hash,
                name: name_version.to_string(),
                version: None,
            }
        }
    }

    /// Generate Package URL (purl) for this store path
    pub fn purl(&self) -> String {
        let safe_name = self.name.replace('/', "%2F").replace('@', "%40");
        match &self.version {
            Some(v) => format!("pkg:nix/{}@{}", safe_name, v),
            None => format!("pkg:nix/{}", safe_name),
        }
    }
}

/// Closure graph parsed from exportReferencesGraph output
#[derive(Debug, Clone)]
pub struct ClosureGraph {
    /// Map of store path -> list of references
    pub graph: HashMap<String, Vec<String>>,
}

impl ClosureGraph {
    /// Parse exportReferencesGraph output file
    ///
    /// The format alternates between:
    /// - Store path line
    /// - Count of references line
    /// - Reference lines (count of them)
    pub fn parse_file(path: &Path) -> Result<Self, super::Error> {
        let content = fs::read_to_string(path).map_err(|e| {
            super::Error::Io(std::io::Error::new(
                e.kind(),
                format!("Failed to read closure graph file '{}': {}", path.display(), e),
            ))
        })?;

        Self::parse(&content)
    }

    /// Parse exportReferencesGraph content
    pub fn parse(content: &str) -> Result<Self, super::Error> {
        let mut graph: HashMap<String, Vec<String>> = HashMap::new();
        let lines: Vec<&str> = content
            .lines()
            .map(|l| l.trim())
            .filter(|l| !l.is_empty())
            .collect();

        let mut i = 0;
        while i < lines.len() {
            let line = lines[i];

            if line.starts_with("/nix/store/") {
                let current_path = line.to_string();
                let mut refs = Vec::new();

                // Next line should be reference count
                i += 1;
                if i < lines.len() {
                    if let Ok(ref_count) = lines[i].parse::<usize>() {
                        // Read the references
                        for _ in 0..ref_count {
                            i += 1;
                            if i < lines.len() && lines[i].starts_with("/nix/store/") {
                                refs.push(lines[i].to_string());
                            }
                        }
                    }
                }

                graph.insert(current_path, refs);
            }
            i += 1;
        }

        Ok(ClosureGraph { graph })
    }

    /// Get all unique packages from the closure
    pub fn packages(&self) -> Vec<Package> {
        let mut seen_purls: HashSet<String> = HashSet::new();
        let mut packages = Vec::new();

        for store_path in self.graph.keys() {
            let parsed = StorePath::parse(store_path);
            let purl = parsed.purl();

            if !seen_purls.contains(&purl) {
                seen_purls.insert(purl.clone());
                packages.push(Package {
                    name: parsed.name.clone(),
                    version: parsed.version.clone(),
                    purl,
                    store_path: store_path.clone(),
                });
            }
        }

        packages.sort_by(|a, b| a.name.cmp(&b.name));
        packages
    }

    /// Build a tree structure from the closure starting at root
    pub fn build_tree(&self, root_path: &str, max_depth: usize) -> ClosureNode {
        let mut visited = HashSet::new();
        self.build_node(root_path, &mut visited, 0, max_depth)
    }

    fn build_node(
        &self,
        path: &str,
        visited: &mut HashSet<String>,
        depth: usize,
        max_depth: usize,
    ) -> ClosureNode {
        let parsed = StorePath::parse(path);
        let purl = parsed.purl();

        if visited.contains(path) || depth > max_depth {
            return ClosureNode {
                path: path.to_string(),
                name: parsed.name,
                version: parsed.version,
                purl,
                references: Vec::new(),
            };
        }

        visited.insert(path.to_string());

        let references = self
            .graph
            .get(path)
            .map(|refs| {
                refs.iter()
                    .filter(|r| *r != path)
                    .map(|r| self.build_node(r, visited, depth + 1, max_depth))
                    .collect()
            })
            .unwrap_or_default();

        ClosureNode {
            path: path.to_string(),
            name: parsed.name,
            version: parsed.version,
            purl,
            references,
        }
    }
}

/// Package information extracted from closure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Package {
    pub name: String,
    pub version: Option<String>,
    pub purl: String,
    pub store_path: String,
}

/// Node in the closure tree
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClosureNode {
    pub path: String,
    pub name: String,
    pub version: Option<String>,
    pub purl: String,
    pub references: Vec<ClosureNode>,
}

/// Generate deterministic UUID v5 from string
fn generate_uuid(input: &str) -> String {
    let namespace = Uuid::parse_str("6ba7b810-9dad-11d1-80b4-00c04fd430c8").unwrap();
    Uuid::new_v5(&namespace, input.as_bytes()).to_string()
}

// ============================================================================
// Output JSON structures
// ============================================================================

/// metadata.json structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataJson {
    pub schema_version: String,
    pub generated_at: String,
    pub container: ContainerInfo,
    pub build: BuildInfo,
    pub packages: PackagesInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerInfo {
    pub name: String,
    pub version: String,
    pub exposed_ports: Vec<u16>,
    pub user: String,
    pub workdir: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildInfo {
    pub nix_store_path: String,
    pub nixpkgs_revision: String,
    pub flake_uri: String,
    pub flake_revision: String,
    pub build_timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackagesInfo {
    pub total: usize,
    pub items: Vec<Package>,
}

/// closure.json structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClosureJson {
    pub schema_version: String,
    pub generated_at: String,
    pub root_path: String,
    pub summary: ClosureSummary,
    pub paths: Vec<PathEntry>,
    pub tree: ClosureNode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClosureSummary {
    pub total_store_paths: usize,
    pub total_references: usize,
    pub unique_packages: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathEntry {
    pub path: String,
    pub references: Vec<String>,
    pub reference_count: usize,
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

// ============================================================================
// Metadata generator
// ============================================================================

/// Generate all metadata files from closure graph and config
pub fn generate_metadata(
    closure_graph: &ClosureGraph,
    config: &MetadataConfig,
    output_dir: &Path,
) -> Result<(), super::Error> {
    let timestamp = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    let packages = closure_graph.packages();

    // Create output directory
    fs::create_dir_all(output_dir).map_err(|e| {
        super::Error::Io(std::io::Error::new(
            e.kind(),
            format!("Failed to create output directory '{}': {}", output_dir.display(), e),
        ))
    })?;

    // Generate metadata.json
    eprintln!("Generating metadata.json...");
    let metadata = generate_metadata_json(config, &packages, &timestamp);
    let metadata_path = output_dir.join("metadata.json");
    fs::write(&metadata_path, serde_json::to_string_pretty(&metadata).unwrap())?;

    // Generate sbom-cyclonedx.json
    eprintln!("Generating sbom-cyclonedx.json...");
    let cyclonedx = generate_cyclonedx(config, &packages, closure_graph, &timestamp);
    let cyclonedx_path = output_dir.join("sbom-cyclonedx.json");
    fs::write(&cyclonedx_path, serde_json::to_string_pretty(&cyclonedx).unwrap())?;

    // Generate sbom-spdx.json
    eprintln!("Generating sbom-spdx.json...");
    let spdx = generate_spdx(config, &packages, closure_graph, &timestamp);
    let spdx_path = output_dir.join("sbom-spdx.json");
    fs::write(&spdx_path, serde_json::to_string_pretty(&spdx).unwrap())?;

    // Generate closure.json
    eprintln!("Generating closure.json...");
    let closure = generate_closure_json(closure_graph, config, &packages, &timestamp);
    let closure_path = output_dir.join("closure.json");
    fs::write(&closure_path, serde_json::to_string_pretty(&closure).unwrap())?;

    eprintln!("Generated all metadata files in {}", output_dir.display());
    Ok(())
}

fn generate_metadata_json(
    config: &MetadataConfig,
    packages: &[Package],
    timestamp: &str,
) -> MetadataJson {
    MetadataJson {
        schema_version: "1.0.0".to_string(),
        generated_at: timestamp.to_string(),
        container: ContainerInfo {
            name: config.container_name.clone(),
            version: config.container_version.clone(),
            exposed_ports: config.exposed_ports.clone(),
            user: config.user.clone(),
            workdir: config.workdir.clone(),
        },
        build: BuildInfo {
            nix_store_path: config.main_store_path.clone(),
            nixpkgs_revision: config.nixpkgs_revision.clone(),
            flake_uri: config.flake_uri.clone(),
            flake_revision: config.flake_revision.clone(),
            build_timestamp: timestamp.to_string(),
        },
        packages: PackagesInfo {
            total: packages.len(),
            items: packages.to_vec(),
        },
    }
}

fn generate_cyclonedx(
    config: &MetadataConfig,
    packages: &[Package],
    closure_graph: &ClosureGraph,
    timestamp: &str,
) -> CycloneDxBom {
    let serial_uuid = generate_uuid(&format!(
        "{}-{}-{}",
        config.container_name, config.container_version, timestamp
    ));

    let components: Vec<CycloneDxComponent> = packages
        .iter()
        .map(|pkg| CycloneDxComponent {
            component_type: "library".to_string(),
            bom_ref: pkg.purl.clone(),
            name: pkg.name.clone(),
            version: pkg.version.clone().unwrap_or_else(|| "unknown".to_string()),
            purl: pkg.purl.clone(),
            properties: vec![CycloneDxProperty {
                name: "nix:store_path".to_string(),
                value: pkg.store_path.clone(),
            }],
        })
        .collect();

    let dependencies: Vec<CycloneDxDependency> = closure_graph
        .graph
        .iter()
        .map(|(store_path, refs)| {
            let parsed = StorePath::parse(store_path);
            let purl = parsed.purl();

            let depends_on: Vec<String> = refs
                .iter()
                .filter(|r| *r != store_path)
                .map(|r| StorePath::parse(r).purl())
                .collect();

            CycloneDxDependency {
                dep_ref: purl,
                depends_on,
            }
        })
        .collect();

    CycloneDxBom {
        schema: "http://cyclonedx.org/schema/bom-1.5.schema.json".to_string(),
        bom_format: "CycloneDX".to_string(),
        spec_version: "1.5".to_string(),
        serial_number: format!("urn:uuid:{}", serial_uuid),
        version: 1,
        metadata: CycloneDxMetadata {
            timestamp: timestamp.to_string(),
            tools: CycloneDxTools {
                components: vec![CycloneDxToolComponent {
                    component_type: "application".to_string(),
                    name: "firestream-vib".to_string(),
                    version: env!("CARGO_PKG_VERSION").to_string(),
                    publisher: "Firestream".to_string(),
                }],
            },
            component: CycloneDxMainComponent {
                component_type: "container".to_string(),
                bom_ref: format!(
                    "pkg:docker/firestream-{}@{}",
                    config.container_name, config.container_version
                ),
                name: format!("firestream-{}", config.container_name),
                version: config.container_version.clone(),
                purl: format!(
                    "pkg:docker/firestream-{}@{}",
                    config.container_name, config.container_version
                ),
            },
            manufacture: CycloneDxManufacture {
                name: "Firestream".to_string(),
            },
            licenses: vec![CycloneDxLicenseWrapper {
                license: CycloneDxLicense {
                    id: "Apache-2.0".to_string(),
                },
            }],
            properties: vec![CycloneDxProperty {
                name: "firestream:flake_uri".to_string(),
                value: config.flake_uri.clone(),
            }],
        },
        components,
        dependencies,
    }
}

fn generate_spdx(
    config: &MetadataConfig,
    packages: &[Package],
    closure_graph: &ClosureGraph,
    timestamp: &str,
) -> SpdxDocument {
    let doc_namespace = format!(
        "https://firestream.dev/spdx/{}/{}",
        config.container_name, config.container_version
    );

    // Helper to generate SPDX ID
    let spdx_id = |name: &str, version: Option<&str>| -> String {
        let safe_name: String = name.chars().map(|c| if c.is_alphanumeric() || c == '.' || c == '-' || c == '_' { c } else { '-' }).collect();
        let safe_version: String = version
            .unwrap_or("unknown")
            .chars()
            .map(|c| if c.is_alphanumeric() || c == '.' || c == '-' || c == '_' { c } else { '-' })
            .collect();
        format!("SPDXRef-Package-{}-{}", safe_name, safe_version)
    };

    // Root package
    let root_spdx_id = format!("SPDXRef-Package-firestream-{}", config.container_name);
    let mut spdx_packages = vec![SpdxPackage {
        spdx_id: root_spdx_id.clone(),
        name: format!("firestream-{}", config.container_name),
        version_info: config.container_version.clone(),
        download_location: if config.flake_uri.is_empty() {
            "NOASSERTION".to_string()
        } else {
            config.flake_uri.clone()
        },
        files_analyzed: false,
        license_concluded: "Apache-2.0".to_string(),
        license_declared: "Apache-2.0".to_string(),
        copyright_text: "NOASSERTION".to_string(),
        external_refs: vec![SpdxExternalRef {
            reference_category: "PACKAGE-MANAGER".to_string(),
            reference_type: "purl".to_string(),
            reference_locator: format!(
                "pkg:docker/firestream-{}@{}",
                config.container_name, config.container_version
            ),
        }],
        comment: None,
    }];

    // Map store paths to SPDX IDs
    let mut path_to_spdx_id: HashMap<String, String> = HashMap::new();

    for pkg in packages {
        let pkg_spdx_id = spdx_id(&pkg.name, pkg.version.as_deref());
        path_to_spdx_id.insert(pkg.store_path.clone(), pkg_spdx_id.clone());

        spdx_packages.push(SpdxPackage {
            spdx_id: pkg_spdx_id,
            name: pkg.name.clone(),
            version_info: pkg.version.clone().unwrap_or_else(|| "unknown".to_string()),
            download_location: "NOASSERTION".to_string(),
            files_analyzed: false,
            license_concluded: "NOASSERTION".to_string(),
            license_declared: "NOASSERTION".to_string(),
            copyright_text: "NOASSERTION".to_string(),
            external_refs: vec![SpdxExternalRef {
                reference_category: "PACKAGE-MANAGER".to_string(),
                reference_type: "purl".to_string(),
                reference_locator: pkg.purl.clone(),
            }],
            comment: Some(format!("Nix store path: {}", pkg.store_path)),
        });
    }

    // Build relationships
    let mut relationships = vec![SpdxRelationship {
        spdx_element_id: "SPDXRef-DOCUMENT".to_string(),
        relationship_type: "DESCRIBES".to_string(),
        related_spdx_element: root_spdx_id.clone(),
    }];

    // Dependency relationships
    for (store_path, refs) in &closure_graph.graph {
        if let Some(from_id) = path_to_spdx_id.get(store_path) {
            for ref_path in refs {
                if ref_path != store_path {
                    if let Some(to_id) = path_to_spdx_id.get(ref_path) {
                        relationships.push(SpdxRelationship {
                            spdx_element_id: from_id.clone(),
                            relationship_type: "DEPENDS_ON".to_string(),
                            related_spdx_element: to_id.clone(),
                        });
                    }
                }
            }
        }
    }

    // Containment relationships
    for pkg in packages {
        if let Some(pkg_spdx_id) = path_to_spdx_id.get(&pkg.store_path) {
            relationships.push(SpdxRelationship {
                spdx_element_id: root_spdx_id.clone(),
                relationship_type: "CONTAINS".to_string(),
                related_spdx_element: pkg_spdx_id.clone(),
            });
        }
    }

    SpdxDocument {
        spdx_version: "SPDX-2.3".to_string(),
        data_license: "CC0-1.0".to_string(),
        spdx_id: "SPDXRef-DOCUMENT".to_string(),
        name: format!(
            "firestream-{}-{}",
            config.container_name, config.container_version
        ),
        document_namespace: doc_namespace,
        creation_info: SpdxCreationInfo {
            created: timestamp.to_string(),
            creators: vec![
                format!("Tool: firestream-vib-{}", env!("CARGO_PKG_VERSION")),
                "Organization: Firestream".to_string(),
            ],
            license_list_version: "3.21".to_string(),
        },
        packages: spdx_packages,
        relationships,
    }
}

fn generate_closure_json(
    closure_graph: &ClosureGraph,
    config: &MetadataConfig,
    packages: &[Package],
    timestamp: &str,
) -> ClosureJson {
    let total_refs: usize = closure_graph.graph.values().map(|v| v.len()).sum();
    let tree = closure_graph.build_tree(&config.main_store_path, 50);

    let paths: Vec<PathEntry> = closure_graph
        .graph
        .iter()
        .map(|(path, refs)| PathEntry {
            path: path.clone(),
            references: refs.clone(),
            reference_count: refs.len(),
        })
        .collect();

    ClosureJson {
        schema_version: "1.0.0".to_string(),
        generated_at: timestamp.to_string(),
        root_path: config.main_store_path.clone(),
        summary: ClosureSummary {
            total_store_paths: closure_graph.graph.len(),
            total_references: total_refs,
            unique_packages: packages.len(),
        },
        paths,
        tree,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_store_path_parsing() {
        let path = "/nix/store/a1b2c3d4e5f6g7h8i9j0k1l2m3n4o5p6-python3-3.11.0";
        let parsed = StorePath::parse(path);
        assert_eq!(parsed.hash, "a1b2c3d4e5f6g7h8i9j0k1l2m3n4o5p6");
        assert_eq!(parsed.name, "python3");
        assert_eq!(parsed.version, Some("3.11.0".to_string()));
    }

    #[test]
    fn test_store_path_no_version() {
        let path = "/nix/store/a1b2c3d4e5f6g7h8i9j0k1l2m3n4o5p6-glibc";
        let parsed = StorePath::parse(path);
        assert_eq!(parsed.name, "glibc");
        assert_eq!(parsed.version, None);
    }

    #[test]
    fn test_purl_generation() {
        let path = StorePath {
            full_path: "/nix/store/xxx-python3-3.11.0".to_string(),
            hash: "xxx".to_string(),
            name: "python3".to_string(),
            version: Some("3.11.0".to_string()),
        };
        assert_eq!(path.purl(), "pkg:nix/python3@3.11.0");
    }

    #[test]
    fn test_closure_graph_parsing() {
        let content = r#"
/nix/store/a1b2c3d4e5f6g7h8i9j0k1l2m3n4o5p6-python3-3.11.0
2
/nix/store/b2c3d4e5f6g7h8i9j0k1l2m3n4o5p6q7-glibc-2.38
/nix/store/c3d4e5f6g7h8i9j0k1l2m3n4o5p6q7r8-zlib-1.3
/nix/store/b2c3d4e5f6g7h8i9j0k1l2m3n4o5p6q7-glibc-2.38
0
"#;
        let graph = ClosureGraph::parse(content).unwrap();
        assert_eq!(graph.graph.len(), 2);
        assert_eq!(
            graph
                .graph
                .get("/nix/store/a1b2c3d4e5f6g7h8i9j0k1l2m3n4o5p6-python3-3.11.0")
                .unwrap()
                .len(),
            2
        );
    }
}
