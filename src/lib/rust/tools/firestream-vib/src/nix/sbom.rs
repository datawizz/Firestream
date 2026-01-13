//! Software Bill of Materials (SBOM) generation
//!
//! Generates SBOMs from Nix store paths and derivations.

use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::process::Command;
use uuid::Uuid;

/// Software Bill of Materials
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sbom {
    /// SBOM format version
    pub version: String,
    /// List of components/packages
    pub components: Vec<Component>,
    /// Dependencies between components
    pub dependencies: Vec<Dependency>,
    /// Metadata about the SBOM generation
    pub metadata: SbomMetadata,
}

/// SBOM metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SbomMetadata {
    /// Timestamp when SBOM was generated
    pub timestamp: String,
    /// Tool that generated the SBOM
    pub tool: String,
    /// Tool version
    pub tool_version: String,
}

/// A software component/package
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Component {
    /// Component name
    pub name: String,
    /// Component version
    pub version: Option<String>,
    /// Nix store path
    pub store_path: String,
    /// NAR hash of the component
    pub hash: Option<String>,
    /// NAR size in bytes
    pub size: Option<u64>,
    /// Component type (library, application, etc.)
    pub component_type: String,
    /// Package manager (always "nix" for Nix packages)
    pub purl: Option<String>,
}

/// Dependency relationship between components
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    /// Source component (store path)
    pub from: String,
    /// Target component (store path)
    pub to: String,
    /// Dependency type
    pub dep_type: String,
}

/// CycloneDX BOM structure
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CycloneDxBom {
    #[serde(rename = "bomFormat")]
    bom_format: String,
    #[serde(rename = "specVersion")]
    spec_version: String,
    version: u32,
    metadata: CycloneDxMetadata,
    components: Vec<CycloneDxComponent>,
    dependencies: Vec<CycloneDxDependency>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CycloneDxMetadata {
    timestamp: String,
    tools: Vec<CycloneDxTool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CycloneDxTool {
    vendor: String,
    name: String,
    version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CycloneDxComponent {
    #[serde(rename = "type")]
    component_type: String,
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    purl: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    hashes: Option<Vec<CycloneDxHash>>,
    properties: Vec<CycloneDxProperty>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CycloneDxHash {
    alg: String,
    content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CycloneDxProperty {
    name: String,
    value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CycloneDxDependency {
    #[serde(rename = "ref")]
    dep_ref: String,
    #[serde(rename = "dependsOn")]
    depends_on: Vec<String>,
}

impl Sbom {
    /// Generate SBOM from a Nix store path
    pub async fn from_store_path(path: &str) -> Result<Self, super::Error> {
        generate_cyclonedx(path).await
    }

    /// Export SBOM to CycloneDX JSON format
    pub fn to_cyclonedx(&self) -> Result<String, super::Error> {
        let bom = CycloneDxBom {
            bom_format: "CycloneDX".to_string(),
            spec_version: "1.5".to_string(),
            version: 1,
            metadata: CycloneDxMetadata {
                timestamp: self.metadata.timestamp.clone(),
                tools: vec![CycloneDxTool {
                    vendor: "Firestream".to_string(),
                    name: self.metadata.tool.clone(),
                    version: self.metadata.tool_version.clone(),
                }],
            },
            components: self
                .components
                .iter()
                .map(|c| {
                    let mut properties = vec![CycloneDxProperty {
                        name: "nix:storePath".to_string(),
                        value: c.store_path.clone(),
                    }];

                    if let Some(size) = c.size {
                        properties.push(CycloneDxProperty {
                            name: "nix:narSize".to_string(),
                            value: size.to_string(),
                        });
                    }

                    CycloneDxComponent {
                        component_type: c.component_type.clone(),
                        name: c.name.clone(),
                        version: c.version.clone(),
                        purl: c.purl.clone(),
                        hashes: c.hash.as_ref().map(|h| {
                            vec![CycloneDxHash {
                                alg: "SHA-256".to_string(),
                                content: h.clone(),
                            }]
                        }),
                        properties,
                    }
                })
                .collect(),
            dependencies: self
                .dependencies
                .iter()
                .map(|d| CycloneDxDependency {
                    dep_ref: d.from.clone(),
                    depends_on: vec![d.to.clone()],
                })
                .collect(),
        };

        serde_json::to_string_pretty(&bom).map_err(|e| {
            super::Error::ParseError(format!("Failed to serialize CycloneDX: {}", e))
        })
    }

    /// Export SBOM to SPDX JSON format
    pub fn to_spdx(&self) -> Result<String, super::Error> {
        let packages: Vec<serde_json::Value> = self
            .components
            .iter()
            .map(|c| {
                let mut pkg = serde_json::json!({
                    "SPDXID": format!("SPDXRef-{}", c.name),
                    "name": c.name,
                    "versionInfo": c.version,
                    "downloadLocation": "NOASSERTION",
                    "filesAnalyzed": false,
                });

                if let Some(hash) = &c.hash {
                    pkg["externalRefs"] = serde_json::json!([{
                        "referenceCategory": "PACKAGE-MANAGER",
                        "referenceType": "purl",
                        "referenceLocator": format!("pkg:nix/{}@{}", c.name, hash),
                    }]);
                }

                pkg
            })
            .collect();

        let relationships: Vec<serde_json::Value> = self
            .dependencies
            .iter()
            .map(|d| {
                serde_json::json!({
                    "spdxElementId": d.from,
                    "relationshipType": "DEPENDS_ON",
                    "relatedSpdxElement": d.to,
                })
            })
            .collect();

        let spdx = serde_json::json!({
            "spdxVersion": "SPDX-2.3",
            "dataLicense": "CC0-1.0",
            "SPDXID": "SPDXRef-DOCUMENT",
            "name": "Nix Package SBOM",
            "documentNamespace": format!("https://firestream.io/sbom/{}", Uuid::new_v4()),
            "creationInfo": {
                "created": self.metadata.timestamp,
                "creators": [
                    format!("Tool: {}", self.metadata.tool),
                ],
            },
            "packages": packages,
            "relationships": relationships,
        });

        serde_json::to_string_pretty(&spdx).map_err(|e| {
            super::Error::ParseError(format!("Failed to serialize SPDX: {}", e))
        })
    }

    /// Write SBOM to a file in the specified format
    pub async fn write_to_file(&self, path: &str, format: SbomFormat) -> Result<(), super::Error> {
        let content = match format {
            SbomFormat::CycloneDx => self.to_cyclonedx()?,
            SbomFormat::Spdx => self.to_spdx()?,
        };

        tokio::fs::write(path, content)
            .await
            .map_err(|e| super::Error::Io(e))
    }
}

/// SBOM export formats
#[derive(Debug, Clone, Copy)]
pub enum SbomFormat {
    CycloneDx,
    Spdx,
}

/// Generate CycloneDX SBOM from a Nix store path
pub async fn generate_cyclonedx(store_path: &str) -> Result<Sbom, super::Error> {
    // Get recursive closure information
    let output = Command::new("nix")
        .args(["path-info", "--json", "--recursive", store_path])
        .output()
        .await
        .map_err(|e| super::Error::Io(e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(super::Error::CommandFailed(format!(
            "nix path-info failed: {}",
            stderr
        )));
    }

    let json_str = String::from_utf8_lossy(&output.stdout);
    let path_infos: Vec<serde_json::Value> = serde_json::from_str(&json_str).map_err(|e| {
        super::Error::ParseError(format!("Failed to parse path-info JSON: {}", e))
    })?;

    // Build components from path info
    let mut components = Vec::new();
    let mut path_to_name: HashMap<String, String> = HashMap::new();

    for info in &path_infos {
        let path = info["path"]
            .as_str()
            .ok_or_else(|| super::Error::ParseError("Missing path field".to_string()))?;

        // Extract package name from store path
        // Nix store paths are like: /nix/store/hash-name-version
        let name = path
            .split('/')
            .last()
            .and_then(|s| s.split('-').skip(1).next())
            .unwrap_or("unknown")
            .to_string();

        let nar_hash = info["narHash"].as_str().map(String::from);
        let nar_size = info["narSize"].as_u64();

        // Generate PURL (Package URL) for Nix packages
        let purl = if let Some(hash) = &nar_hash {
            Some(format!("pkg:nix/{}@{}", name, hash))
        } else {
            None
        };

        path_to_name.insert(path.to_string(), name.clone());

        components.push(Component {
            name: name.clone(),
            version: None, // Version extraction would require parsing the derivation
            store_path: path.to_string(),
            hash: nar_hash,
            size: nar_size,
            component_type: "library".to_string(),
            purl,
        });
    }

    // Build dependency graph
    let mut dependencies = Vec::new();

    for info in &path_infos {
        let from_path = info["path"]
            .as_str()
            .ok_or_else(|| super::Error::ParseError("Missing path field".to_string()))?;

        if let Some(references) = info["references"].as_array() {
            for ref_val in references {
                if let Some(to_path) = ref_val.as_str() {
                    if from_path != to_path {
                        dependencies.push(Dependency {
                            from: from_path.to_string(),
                            to: to_path.to_string(),
                            dep_type: "runtime".to_string(),
                        });
                    }
                }
            }
        }
    }

    Ok(Sbom {
        version: "1.0.0".to_string(),
        components,
        dependencies,
        metadata: SbomMetadata {
            timestamp: Utc::now().to_rfc3339(),
            tool: "firestream-vib".to_string(),
            tool_version: env!("CARGO_PKG_VERSION").to_string(),
        },
    })
}

/// Export SBOM to SPDX format and write to file
pub fn export_spdx(sbom: &Sbom, path: &str) -> Result<(), super::Error> {
    let spdx_json = sbom.to_spdx()?;
    std::fs::write(path, spdx_json).map_err(|e| super::Error::Io(e))
}
