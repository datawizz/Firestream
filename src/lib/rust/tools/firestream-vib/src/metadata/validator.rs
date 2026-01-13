//! Metadata file validator
//!
//! Validates the structure and content of container metadata files.

use super::reader::MetadataReader;
use super::Error;

/// Result of a validation run
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// Whether all checks passed
    pub passed: bool,
    /// Individual check results
    pub checks: Vec<Check>,
}

impl ValidationResult {
    /// Create a new ValidationResult from a list of checks
    pub fn from_checks(checks: Vec<Check>) -> Self {
        let passed = checks.iter().all(|c| c.passed);
        Self { passed, checks }
    }

    /// Get all failed checks
    pub fn failed_checks(&self) -> Vec<&Check> {
        self.checks.iter().filter(|c| !c.passed).collect()
    }

    /// Get all passed checks
    pub fn passed_checks(&self) -> Vec<&Check> {
        self.checks.iter().filter(|c| c.passed).collect()
    }

    /// Get a summary string
    pub fn summary(&self) -> String {
        let passed_count = self.checks.iter().filter(|c| c.passed).count();
        let total = self.checks.len();
        format!(
            "{}/{} checks passed ({})",
            passed_count,
            total,
            if self.passed { "PASS" } else { "FAIL" }
        )
    }
}

/// A single validation check
#[derive(Debug, Clone)]
pub struct Check {
    /// Name of the check
    pub name: String,
    /// Whether the check passed
    pub passed: bool,
    /// Descriptive message about the check result
    pub message: String,
}

impl Check {
    /// Create a passed check
    pub fn pass(name: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            passed: true,
            message: message.into(),
        }
    }

    /// Create a failed check
    pub fn fail(name: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            passed: false,
            message: message.into(),
        }
    }
}

/// Validates container metadata files
pub struct MetadataValidator {
    /// The reader to use for accessing metadata files
    reader: MetadataReader,
}

impl MetadataValidator {
    /// Create a new MetadataValidator with the given reader
    pub fn new(reader: MetadataReader) -> Self {
        Self { reader }
    }

    /// Validate all metadata files exist and are well-formed
    pub fn validate_all(&self) -> Result<ValidationResult, Error> {
        let mut checks = Vec::new();

        // Check metadata.json
        checks.push(self.validate_metadata_json());

        // Check sbom-cyclonedx.json
        checks.push(self.validate_cyclonedx_sbom());

        // Check sbom-spdx.json
        checks.push(self.validate_spdx_sbom());

        // Check closure.json
        checks.push(self.validate_closure());

        Ok(ValidationResult::from_checks(checks))
    }

    /// Validate metadata.json file
    fn validate_metadata_json(&self) -> Check {
        match self.reader.read_metadata() {
            Ok(metadata) => {
                // Validate schema version
                if metadata.schema_version != "1.0.0" {
                    return Check::fail(
                        "metadata.json:schema_version",
                        format!(
                            "Expected schema_version '1.0.0', got '{}'",
                            metadata.schema_version
                        ),
                    );
                }

                // Validate required fields are present and non-empty
                if metadata.container.name.is_empty() {
                    return Check::fail("metadata.json:container.name", "Container name is empty");
                }

                if metadata.container.version.is_empty() {
                    return Check::fail(
                        "metadata.json:container.version",
                        "Container version is empty",
                    );
                }

                if metadata.build.nix_store_path.is_empty() {
                    return Check::fail(
                        "metadata.json:build.nix_store_path",
                        "Nix store path is empty",
                    );
                }

                if metadata.build.nixpkgs_revision.is_empty() {
                    return Check::fail(
                        "metadata.json:build.nixpkgs_revision",
                        "Nixpkgs revision is empty",
                    );
                }

                Check::pass(
                    "metadata.json",
                    format!(
                        "Valid metadata for {} v{}",
                        metadata.container.name, metadata.container.version
                    ),
                )
            }
            Err(Error::FileNotFound(path)) => {
                Check::fail("metadata.json", format!("File not found: {}", path))
            }
            Err(e) => Check::fail("metadata.json", format!("Error reading file: {}", e)),
        }
    }

    /// Validate sbom-cyclonedx.json file
    fn validate_cyclonedx_sbom(&self) -> Check {
        match self.reader.read_cyclonedx_sbom() {
            Ok(sbom) => {
                // Validate bomFormat
                let bom_format = sbom.get("bomFormat").and_then(|v| v.as_str()).unwrap_or("");

                if bom_format != "CycloneDX" {
                    return Check::fail(
                        "sbom-cyclonedx.json:bomFormat",
                        format!("Expected bomFormat 'CycloneDX', got '{}'", bom_format),
                    );
                }

                // Validate specVersion
                let spec_version = sbom
                    .get("specVersion")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                if spec_version != "1.5" {
                    return Check::fail(
                        "sbom-cyclonedx.json:specVersion",
                        format!("Expected specVersion '1.5', got '{}'", spec_version),
                    );
                }

                // Count components if present
                let component_count = sbom
                    .get("components")
                    .and_then(|v| v.as_array())
                    .map(|a| a.len())
                    .unwrap_or(0);

                Check::pass(
                    "sbom-cyclonedx.json",
                    format!(
                        "Valid CycloneDX 1.5 SBOM with {} components",
                        component_count
                    ),
                )
            }
            Err(Error::FileNotFound(path)) => {
                Check::fail("sbom-cyclonedx.json", format!("File not found: {}", path))
            }
            Err(e) => Check::fail("sbom-cyclonedx.json", format!("Error reading file: {}", e)),
        }
    }

    /// Validate sbom-spdx.json file
    fn validate_spdx_sbom(&self) -> Check {
        match self.reader.read_spdx_sbom() {
            Ok(sbom) => {
                // Validate spdxVersion
                let spdx_version = sbom
                    .get("spdxVersion")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                if spdx_version != "SPDX-2.3" {
                    return Check::fail(
                        "sbom-spdx.json:spdxVersion",
                        format!("Expected spdxVersion 'SPDX-2.3', got '{}'", spdx_version),
                    );
                }

                // Count packages if present
                let package_count = sbom
                    .get("packages")
                    .and_then(|v| v.as_array())
                    .map(|a| a.len())
                    .unwrap_or(0);

                Check::pass(
                    "sbom-spdx.json",
                    format!("Valid SPDX 2.3 SBOM with {} packages", package_count),
                )
            }
            Err(Error::FileNotFound(path)) => {
                Check::fail("sbom-spdx.json", format!("File not found: {}", path))
            }
            Err(e) => Check::fail("sbom-spdx.json", format!("Error reading file: {}", e)),
        }
    }

    /// Validate closure.json file
    fn validate_closure(&self) -> Check {
        match self.reader.read_closure() {
            Ok(closure) => {
                // Check for summary field
                let has_summary = closure.get("summary").is_some();
                if !has_summary {
                    return Check::fail("closure.json:summary", "Missing required 'summary' field");
                }

                // Check for paths field
                let has_paths = closure.get("paths").is_some();
                if !has_paths {
                    return Check::fail("closure.json:paths", "Missing required 'paths' field");
                }

                // Get path count
                let path_count = closure
                    .get("paths")
                    .and_then(|v| v.as_array())
                    .map(|a| a.len())
                    .unwrap_or(0);

                Check::pass(
                    "closure.json",
                    format!("Valid closure file with {} paths", path_count),
                )
            }
            Err(Error::FileNotFound(path)) => {
                Check::fail("closure.json", format!("File not found: {}", path))
            }
            Err(e) => Check::fail("closure.json", format!("Error reading file: {}", e)),
        }
    }

    /// Validate that expected packages are present in the metadata
    ///
    /// Returns a list of package names that were expected but not found.
    pub fn validate_packages(&self, expected: &[&str]) -> Result<Vec<String>, Error> {
        let metadata = self.reader.read_metadata()?;

        let present_packages: std::collections::HashSet<&str> = metadata
            .packages
            .items
            .iter()
            .map(|p| p.name.as_str())
            .collect();

        let missing: Vec<String> = expected
            .iter()
            .filter(|pkg| !present_packages.contains(*pkg))
            .map(|s| s.to_string())
            .collect();

        Ok(missing)
    }

    /// Get a reference to the underlying reader
    pub fn reader(&self) -> &MetadataReader {
        &self.reader
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_valid_metadata() -> serde_json::Value {
        serde_json::json!({
            "schema_version": "1.0.0",
            "generated_at": "2024-01-15T10:30:00Z",
            "container": {
                "name": "test-container",
                "version": "1.0.0",
                "exposed_ports": [8080],
                "user": "app",
                "workdir": "/app"
            },
            "build": {
                "nix_store_path": "/nix/store/abc123-test-container",
                "nixpkgs_revision": "abc123def456",
                "build_timestamp": "2024-01-15T10:00:00Z"
            },
            "packages": {
                "total": 2,
                "items": [
                    {
                        "name": "openssl",
                        "version": "3.0.0",
                        "purl": "pkg:nix/openssl@3.0.0",
                        "store_path": "/nix/store/xyz-openssl-3.0.0"
                    },
                    {
                        "name": "zlib",
                        "version": "1.2.13",
                        "purl": "pkg:nix/zlib@1.2.13",
                        "store_path": "/nix/store/xyz-zlib-1.2.13"
                    }
                ]
            }
        })
    }

    fn create_valid_cyclonedx() -> serde_json::Value {
        serde_json::json!({
            "bomFormat": "CycloneDX",
            "specVersion": "1.5",
            "version": 1,
            "components": [
                {"name": "openssl", "version": "3.0.0"},
                {"name": "zlib", "version": "1.2.13"}
            ]
        })
    }

    fn create_valid_spdx() -> serde_json::Value {
        serde_json::json!({
            "spdxVersion": "SPDX-2.3",
            "SPDXID": "SPDXRef-DOCUMENT",
            "packages": [
                {"SPDXID": "SPDXRef-Package-openssl"},
                {"SPDXID": "SPDXRef-Package-zlib"}
            ]
        })
    }

    fn create_valid_closure() -> serde_json::Value {
        serde_json::json!({
            "summary": {
                "total_paths": 10,
                "total_size": 1000000
            },
            "paths": [
                {"path": "/nix/store/abc123-openssl"},
                {"path": "/nix/store/def456-zlib"}
            ]
        })
    }

    fn setup_valid_metadata_dir(temp_dir: &TempDir) {
        fs::write(
            temp_dir.path().join("metadata.json"),
            serde_json::to_string_pretty(&create_valid_metadata()).unwrap(),
        )
        .unwrap();
        fs::write(
            temp_dir.path().join("sbom-cyclonedx.json"),
            serde_json::to_string_pretty(&create_valid_cyclonedx()).unwrap(),
        )
        .unwrap();
        fs::write(
            temp_dir.path().join("sbom-spdx.json"),
            serde_json::to_string_pretty(&create_valid_spdx()).unwrap(),
        )
        .unwrap();
        fs::write(
            temp_dir.path().join("closure.json"),
            serde_json::to_string_pretty(&create_valid_closure()).unwrap(),
        )
        .unwrap();
    }

    #[test]
    fn test_validate_all_pass() {
        let temp_dir = TempDir::new().unwrap();
        setup_valid_metadata_dir(&temp_dir);

        let reader = MetadataReader::new(temp_dir.path());
        let validator = MetadataValidator::new(reader);

        let result = validator.validate_all().unwrap();
        assert!(result.passed);
        assert_eq!(result.checks.len(), 4);
    }

    #[test]
    fn test_validate_missing_file() {
        let temp_dir = TempDir::new().unwrap();

        let reader = MetadataReader::new(temp_dir.path());
        let validator = MetadataValidator::new(reader);

        let result = validator.validate_all().unwrap();
        assert!(!result.passed);
        assert!(result.checks.iter().all(|c| !c.passed));
    }

    #[test]
    fn test_validate_wrong_schema_version() {
        let temp_dir = TempDir::new().unwrap();

        let mut metadata = create_valid_metadata();
        metadata["schema_version"] = serde_json::json!("2.0.0");

        fs::write(
            temp_dir.path().join("metadata.json"),
            serde_json::to_string_pretty(&metadata).unwrap(),
        )
        .unwrap();

        let reader = MetadataReader::new(temp_dir.path());
        let validator = MetadataValidator::new(reader);

        let result = validator.validate_all().unwrap();
        let metadata_check = result
            .checks
            .iter()
            .find(|c| c.name.starts_with("metadata.json"))
            .unwrap();
        assert!(!metadata_check.passed);
        assert!(metadata_check.message.contains("schema_version"));
    }

    #[test]
    fn test_validate_packages() {
        let temp_dir = TempDir::new().unwrap();
        setup_valid_metadata_dir(&temp_dir);

        let reader = MetadataReader::new(temp_dir.path());
        let validator = MetadataValidator::new(reader);

        // Test with packages that exist
        let missing = validator.validate_packages(&["openssl", "zlib"]).unwrap();
        assert!(missing.is_empty());

        // Test with packages that don't exist
        let missing = validator
            .validate_packages(&["openssl", "curl", "nonexistent"])
            .unwrap();
        assert_eq!(missing.len(), 2);
        assert!(missing.contains(&"curl".to_string()));
        assert!(missing.contains(&"nonexistent".to_string()));
    }

    #[test]
    fn test_validation_result_summary() {
        let checks = vec![
            Check::pass("test1", "passed"),
            Check::fail("test2", "failed"),
            Check::pass("test3", "passed"),
        ];

        let result = ValidationResult::from_checks(checks);
        assert!(!result.passed);
        assert_eq!(result.passed_checks().len(), 2);
        assert_eq!(result.failed_checks().len(), 1);
        assert!(result.summary().contains("2/3"));
    }
}
