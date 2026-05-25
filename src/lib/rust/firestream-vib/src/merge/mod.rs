//! SBOM Merge Module
//!
//! Merges multiple CycloneDX/SPDX SBOMs into a unified fleet SBOM.
//!
//! Key responsibilities:
//! - Deduplicate components by purl (not structural equality)
//! - Generate valid bom-refs for all components
//! - Create compositions with proper assembly references
//! - Use CycloneDX properties extension for firestream metadata
//! - Produce valid ISO 8601 timestamps
//!
//! # Example
//!
//! ```ignore
//! let merger = SbomMerger::new("firestream", "1.0.0");
//! merger.add_input(Path::new("/path/to/airflow/metadata"))?;
//! merger.add_input(Path::new("/path/to/spark/metadata"))?;
//! let fleet_sbom = merger.merge()?;
//! ```

use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::fs;
use std::path::Path;
use uuid::Uuid;

/// Output format for merge operation
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OutputFormat {
    /// CycloneDX 1.5 JSON format
    CycloneDx,
    /// SPDX 2.3 JSON format
    Spdx,
    /// Fleet inventory manifest (custom JSON)
    Manifest,
}

impl OutputFormat {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "cyclonedx" | "cdx" => Some(Self::CycloneDx),
            "spdx" => Some(Self::Spdx),
            "manifest" | "inventory" => Some(Self::Manifest),
            _ => None,
        }
    }
}

/// Configuration for SBOM merge operation
#[derive(Debug, Clone)]
pub struct MergeConfig {
    pub fleet_name: String,
    pub fleet_version: String,
    pub format: OutputFormat,
}

/// Source container/artifact information from input SBOM
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SourceInfo {
    name: String,
    version: String,
    #[serde(rename = "type")]
    source_type: String,
}

/// Merged component tracking source containers
#[derive(Debug, Clone)]
struct MergedComponent {
    name: String,
    version: String,
    purl: String,
    component_type: String,
    store_path: Option<String>,
    /// Names of source containers/artifacts that include this component
    sources: BTreeSet<String>,
}

/// SBOM Merger - aggregates multiple SBOMs into a fleet SBOM
pub struct SbomMerger {
    config: MergeConfig,
    /// Components indexed by purl for deduplication
    components: HashMap<String, MergedComponent>,
    /// Dependencies indexed by component purl
    dependencies: HashMap<String, HashSet<String>>,
    /// Source containers/artifacts
    sources: Vec<SourceInfo>,
}

impl SbomMerger {
    /// Create a new SBOM merger
    pub fn new(fleet_name: &str, fleet_version: &str, format: OutputFormat) -> Self {
        Self {
            config: MergeConfig {
                fleet_name: fleet_name.to_string(),
                fleet_version: fleet_version.to_string(),
                format,
            },
            components: HashMap::new(),
            dependencies: HashMap::new(),
            sources: Vec::new(),
        }
    }

    /// Add an input directory containing SBOM files
    ///
    /// Expected structure:
    /// - metadata.json: Container metadata
    /// - sbom-cyclonedx.json: CycloneDX SBOM
    pub fn add_input(&mut self, path: &Path) -> Result<(), MergeError> {
        // Read metadata.json to get source info
        let metadata_path = path.join("metadata.json");
        if !metadata_path.exists() {
            return Err(MergeError::MissingFile(format!(
                "metadata.json not found in {}",
                path.display()
            )));
        }

        let metadata_content = fs::read_to_string(&metadata_path)
            .map_err(|e| MergeError::Io(format!("Failed to read {}: {}", metadata_path.display(), e)))?;
        let metadata: serde_json::Value = serde_json::from_str(&metadata_content)
            .map_err(|e| MergeError::Parse(format!("Failed to parse metadata.json: {}", e)))?;

        let container_name = metadata["container"]["name"]
            .as_str()
            .unwrap_or("unknown")
            .to_string();
        let container_version = metadata["container"]["version"]
            .as_str()
            .unwrap_or("unknown")
            .to_string();

        self.sources.push(SourceInfo {
            name: container_name.clone(),
            version: container_version,
            source_type: "container".to_string(),
        });

        // Read sbom-cyclonedx.json
        let sbom_path = path.join("sbom-cyclonedx.json");
        if !sbom_path.exists() {
            return Err(MergeError::MissingFile(format!(
                "sbom-cyclonedx.json not found in {}",
                path.display()
            )));
        }

        let sbom_content = fs::read_to_string(&sbom_path)
            .map_err(|e| MergeError::Io(format!("Failed to read {}: {}", sbom_path.display(), e)))?;
        let sbom: serde_json::Value = serde_json::from_str(&sbom_content)
            .map_err(|e| MergeError::Parse(format!("Failed to parse sbom-cyclonedx.json: {}", e)))?;

        // Extract components
        if let Some(components) = sbom["components"].as_array() {
            for comp in components {
                let purl = comp["purl"].as_str().unwrap_or("").to_string();
                if purl.is_empty() {
                    continue;
                }

                let name = comp["name"].as_str().unwrap_or("unknown").to_string();
                let version = comp["version"].as_str().unwrap_or("unknown").to_string();
                let component_type = comp["type"].as_str().unwrap_or("library").to_string();

                // Extract store path from properties if present
                let store_path = comp["properties"]
                    .as_array()
                    .and_then(|props| {
                        props.iter().find_map(|p| {
                            if p["name"].as_str() == Some("nix:store_path") {
                                p["value"].as_str().map(String::from)
                            } else {
                                None
                            }
                        })
                    });

                // Merge with existing or insert new
                if let Some(existing) = self.components.get_mut(&purl) {
                    existing.sources.insert(container_name.clone());
                } else {
                    let mut sources = BTreeSet::new();
                    sources.insert(container_name.clone());
                    self.components.insert(purl.clone(), MergedComponent {
                        name,
                        version,
                        purl: purl.clone(),
                        component_type,
                        store_path,
                        sources,
                    });
                }
            }
        }

        // Extract dependencies
        if let Some(deps) = sbom["dependencies"].as_array() {
            for dep in deps {
                let ref_purl = dep["ref"].as_str().unwrap_or("").to_string();
                if ref_purl.is_empty() {
                    continue;
                }

                let depends_on: HashSet<String> = dep["dependsOn"]
                    .as_array()
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str().map(String::from))
                            .collect()
                    })
                    .unwrap_or_default();

                self.dependencies
                    .entry(ref_purl)
                    .or_default()
                    .extend(depends_on);
            }
        }

        Ok(())
    }

    /// Merge all inputs and generate output SBOM
    pub fn merge(&self) -> Result<String, MergeError> {
        match self.config.format {
            OutputFormat::CycloneDx => self.generate_cyclonedx(),
            OutputFormat::Spdx => self.generate_spdx(),
            OutputFormat::Manifest => self.generate_manifest(),
        }
    }

    /// Generate CycloneDX 1.5 fleet SBOM
    fn generate_cyclonedx(&self) -> Result<String, MergeError> {
        let timestamp = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
        let serial_uuid = generate_uuid(&format!(
            "{}-{}-{}",
            self.config.fleet_name, self.config.fleet_version, timestamp
        ));

        // Build components list
        let components: Vec<serde_json::Value> = self
            .components
            .values()
            .map(|comp| {
                let mut properties = vec![
                    serde_json::json!({
                        "name": "firestream:source",
                        "value": comp.sources.iter().cloned().collect::<Vec<_>>().join(",")
                    })
                ];

                if let Some(ref store_path) = comp.store_path {
                    properties.push(serde_json::json!({
                        "name": "nix:store_path",
                        "value": store_path
                    }));
                }

                serde_json::json!({
                    "type": comp.component_type,
                    "bom-ref": comp.purl,
                    "name": comp.name,
                    "version": comp.version,
                    "purl": comp.purl,
                    "properties": properties
                })
            })
            .collect();

        // Build dependencies list
        let dependencies: Vec<serde_json::Value> = self
            .dependencies
            .iter()
            .map(|(ref_purl, depends_on)| {
                serde_json::json!({
                    "ref": ref_purl,
                    "dependsOn": depends_on.iter().cloned().collect::<Vec<_>>()
                })
            })
            .collect();

        // Build compositions (assemblies)
        let assemblies: Vec<String> = self.components.keys().cloned().collect();

        let bom = serde_json::json!({
            "$schema": "http://cyclonedx.org/schema/bom-1.5.schema.json",
            "bomFormat": "CycloneDX",
            "specVersion": "1.5",
            "serialNumber": format!("urn:uuid:{}", serial_uuid),
            "version": 1,
            "metadata": {
                "timestamp": timestamp,
                "tools": {
                    "components": [{
                        "type": "application",
                        "name": "firestream-vib",
                        "version": env!("CARGO_PKG_VERSION"),
                        "publisher": "Firestream"
                    }]
                },
                "component": {
                    "type": "application",
                    "bom-ref": format!("{}-fleet", self.config.fleet_name),
                    "name": self.config.fleet_name,
                    "version": self.config.fleet_version,
                    "purl": format!("pkg:firestream/{}@{}", self.config.fleet_name, self.config.fleet_version)
                },
                "manufacture": {
                    "name": "Firestream"
                },
                "licenses": [{
                    "license": {
                        "id": "Apache-2.0"
                    }
                }],
                "properties": self.sources.iter().map(|s| {
                    serde_json::json!({
                        "name": format!("firestream:source:{}", s.name),
                        "value": s.version
                    })
                }).collect::<Vec<_>>()
            },
            "components": components,
            "dependencies": dependencies,
            "compositions": [{
                "aggregate": "complete",
                "assemblies": assemblies
            }]
        });

        serde_json::to_string_pretty(&bom)
            .map_err(|e| MergeError::Serialization(e.to_string()))
    }

    /// Generate SPDX 2.3 fleet SBOM
    fn generate_spdx(&self) -> Result<String, MergeError> {
        let timestamp = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
        let doc_uuid = generate_uuid(&format!(
            "{}-{}-spdx",
            self.config.fleet_name, self.config.fleet_version
        ));

        // Helper to generate SPDX-safe IDs
        let spdx_id = |name: &str, version: &str| -> String {
            let safe_name: String = name
                .chars()
                .map(|c| if c.is_alphanumeric() || c == '.' || c == '-' || c == '_' { c } else { '-' })
                .collect();
            let safe_version: String = version
                .chars()
                .map(|c| if c.is_alphanumeric() || c == '.' || c == '-' || c == '_' { c } else { '-' })
                .collect();
            format!("SPDXRef-Package-{}-{}", safe_name, safe_version)
        };

        // Root package for the fleet
        let root_spdx_id = format!("SPDXRef-Package-{}", self.config.fleet_name);
        let mut packages = vec![serde_json::json!({
            "SPDXID": root_spdx_id,
            "name": self.config.fleet_name,
            "versionInfo": self.config.fleet_version,
            "downloadLocation": "NOASSERTION",
            "filesAnalyzed": false,
            "licenseConcluded": "Apache-2.0",
            "licenseDeclared": "Apache-2.0",
            "copyrightText": "NOASSERTION",
            "externalRefs": [{
                "referenceCategory": "PACKAGE-MANAGER",
                "referenceType": "purl",
                "referenceLocator": format!("pkg:firestream/{}@{}", self.config.fleet_name, self.config.fleet_version)
            }],
            "comment": format!("Fleet SBOM aggregating {} sources: {}",
                self.sources.len(),
                self.sources.iter().map(|s| s.name.as_str()).collect::<Vec<_>>().join(", ")
            )
        })];

        // Map purls to SPDX IDs
        let mut purl_to_spdx_id: HashMap<String, String> = HashMap::new();

        // Add component packages
        for comp in self.components.values() {
            let comp_spdx_id = spdx_id(&comp.name, &comp.version);
            purl_to_spdx_id.insert(comp.purl.clone(), comp_spdx_id.clone());

            packages.push(serde_json::json!({
                "SPDXID": comp_spdx_id,
                "name": comp.name,
                "versionInfo": comp.version,
                "downloadLocation": "NOASSERTION",
                "filesAnalyzed": false,
                "licenseConcluded": "NOASSERTION",
                "licenseDeclared": "NOASSERTION",
                "copyrightText": "NOASSERTION",
                "externalRefs": [{
                    "referenceCategory": "PACKAGE-MANAGER",
                    "referenceType": "purl",
                    "referenceLocator": comp.purl
                }],
                "comment": format!("Sources: {}", comp.sources.iter().cloned().collect::<Vec<_>>().join(", "))
            }));
        }

        // Build relationships
        let mut relationships = vec![
            serde_json::json!({
                "spdxElementId": "SPDXRef-DOCUMENT",
                "relationshipType": "DESCRIBES",
                "relatedSpdxElement": root_spdx_id
            })
        ];

        // Containment relationships (fleet contains components)
        for comp in self.components.values() {
            if let Some(comp_spdx_id) = purl_to_spdx_id.get(&comp.purl) {
                relationships.push(serde_json::json!({
                    "spdxElementId": &root_spdx_id,
                    "relationshipType": "CONTAINS",
                    "relatedSpdxElement": comp_spdx_id
                }));
            }
        }

        // Dependency relationships
        for (from_purl, to_purls) in &self.dependencies {
            if let Some(from_spdx_id) = purl_to_spdx_id.get(from_purl) {
                for to_purl in to_purls {
                    if let Some(to_spdx_id) = purl_to_spdx_id.get(to_purl) {
                        relationships.push(serde_json::json!({
                            "spdxElementId": from_spdx_id,
                            "relationshipType": "DEPENDS_ON",
                            "relatedSpdxElement": to_spdx_id
                        }));
                    }
                }
            }
        }

        let spdx = serde_json::json!({
            "spdxVersion": "SPDX-2.3",
            "dataLicense": "CC0-1.0",
            "SPDXID": "SPDXRef-DOCUMENT",
            "name": format!("{}-{}-fleet", self.config.fleet_name, self.config.fleet_version),
            "documentNamespace": format!("https://firestream.dev/spdx/fleet/{}/{}",
                self.config.fleet_name, doc_uuid),
            "creationInfo": {
                "created": timestamp,
                "creators": [
                    format!("Tool: firestream-vib-{}", env!("CARGO_PKG_VERSION")),
                    "Organization: Firestream"
                ],
                "licenseListVersion": "3.21"
            },
            "packages": packages,
            "relationships": relationships
        });

        serde_json::to_string_pretty(&spdx)
            .map_err(|e| MergeError::Serialization(e.to_string()))
    }

    /// Generate fleet inventory manifest
    fn generate_manifest(&self) -> Result<String, MergeError> {
        let timestamp = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();

        // Group components by source
        let mut components_by_source: BTreeMap<String, Vec<&MergedComponent>> = BTreeMap::new();
        for comp in self.components.values() {
            for source in &comp.sources {
                components_by_source
                    .entry(source.clone())
                    .or_default()
                    .push(comp);
            }
        }

        // Count unique vs shared components
        let unique_count = self.components.values().filter(|c| c.sources.len() == 1).count();
        let shared_count = self.components.len() - unique_count;

        let manifest = serde_json::json!({
            "schemaVersion": "1.0.0",
            "generatedAt": timestamp,
            "fleet": {
                "name": self.config.fleet_name,
                "version": self.config.fleet_version
            },
            "summary": {
                "sourceCount": self.sources.len(),
                "totalComponents": self.components.len(),
                "uniqueComponents": unique_count,
                "sharedComponents": shared_count,
                "totalDependencies": self.dependencies.values().map(|v| v.len()).sum::<usize>()
            },
            "sources": self.sources.iter().map(|s| {
                let source_comps = components_by_source.get(&s.name).map(|c| c.len()).unwrap_or(0);
                serde_json::json!({
                    "name": s.name,
                    "version": s.version,
                    "type": s.source_type,
                    "componentCount": source_comps
                })
            }).collect::<Vec<_>>(),
            "components": self.components.values().map(|c| {
                serde_json::json!({
                    "name": c.name,
                    "version": c.version,
                    "purl": c.purl,
                    "type": c.component_type,
                    "sources": c.sources.iter().cloned().collect::<Vec<_>>(),
                    "shared": c.sources.len() > 1
                })
            }).collect::<Vec<_>>()
        });

        serde_json::to_string_pretty(&manifest)
            .map_err(|e| MergeError::Serialization(e.to_string()))
    }
}

/// Generate deterministic UUID v5 from string
fn generate_uuid(input: &str) -> String {
    let namespace = Uuid::parse_str("6ba7b810-9dad-11d1-80b4-00c04fd430c8").unwrap();
    Uuid::new_v5(&namespace, input.as_bytes()).to_string()
}

/// Error type for merge operations
#[derive(Debug, thiserror::Error)]
pub enum MergeError {
    #[error("Missing required file: {0}")]
    MissingFile(String),

    #[error("IO error: {0}")]
    Io(String),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Serialization error: {0}")]
    Serialization(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    fn create_test_sbom_dir(name: &str, version: &str) -> TempDir {
        let dir = TempDir::new().unwrap();

        // Create metadata.json
        let metadata = serde_json::json!({
            "container": {
                "name": name,
                "version": version
            }
        });
        let mut metadata_file = fs::File::create(dir.path().join("metadata.json")).unwrap();
        write!(metadata_file, "{}", serde_json::to_string_pretty(&metadata).unwrap()).unwrap();

        // Create sbom-cyclonedx.json
        let sbom = serde_json::json!({
            "bomFormat": "CycloneDX",
            "specVersion": "1.5",
            "components": [
                {
                    "type": "library",
                    "name": "openssl",
                    "version": "3.0.12",
                    "purl": "pkg:nix/openssl@3.0.12",
                    "properties": [
                        { "name": "nix:store_path", "value": "/nix/store/xxx-openssl-3.0.12" }
                    ]
                },
                {
                    "type": "library",
                    "name": format!("{}-lib", name),
                    "version": version,
                    "purl": format!("pkg:nix/{}-lib@{}", name, version),
                    "properties": []
                }
            ],
            "dependencies": [
                {
                    "ref": format!("pkg:nix/{}-lib@{}", name, version),
                    "dependsOn": ["pkg:nix/openssl@3.0.12"]
                }
            ]
        });
        let mut sbom_file = fs::File::create(dir.path().join("sbom-cyclonedx.json")).unwrap();
        write!(sbom_file, "{}", serde_json::to_string_pretty(&sbom).unwrap()).unwrap();

        dir
    }

    #[test]
    fn test_merge_two_sboms() {
        let dir1 = create_test_sbom_dir("airflow", "2.8.0");
        let dir2 = create_test_sbom_dir("spark", "4.0.0");

        let mut merger = SbomMerger::new("firestream", "1.0.0", OutputFormat::CycloneDx);
        merger.add_input(dir1.path()).unwrap();
        merger.add_input(dir2.path()).unwrap();

        let result = merger.merge().unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        // Check that openssl is deduplicated but has both sources
        let components = parsed["components"].as_array().unwrap();
        let openssl = components.iter().find(|c| c["name"] == "openssl").unwrap();
        let sources = openssl["properties"]
            .as_array()
            .unwrap()
            .iter()
            .find(|p| p["name"] == "firestream:source")
            .unwrap();
        assert!(sources["value"].as_str().unwrap().contains("airflow"));
        assert!(sources["value"].as_str().unwrap().contains("spark"));

        // Check compositions exist
        assert!(parsed["compositions"].as_array().is_some());
    }

    #[test]
    fn test_manifest_format() {
        let dir1 = create_test_sbom_dir("airflow", "2.8.0");

        let mut merger = SbomMerger::new("firestream", "1.0.0", OutputFormat::Manifest);
        merger.add_input(dir1.path()).unwrap();

        let result = merger.merge().unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        assert_eq!(parsed["fleet"]["name"], "firestream");
        assert!(parsed["summary"]["totalComponents"].as_u64().unwrap() >= 2);
    }
}
