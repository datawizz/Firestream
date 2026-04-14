//! Source Archive Module
//!
//! Archives source code from Nix packages for license compliance and reproducibility.
//!
//! Key responsibilities:
//! - Parse source map JSON (generated at Nix eval time)
//! - Archive sources to tarball files with content-based deduplication
//! - Generate source_index.json with coverage statistics
//!
//! # Architecture
//!
//! The source map is generated at Nix evaluation time (not build time) by
//! introspecting `.src` attributes on packages. This avoids sandbox issues
//! with `nix-store --query`.
//!
//! Sources are deduplicated by content hash (SHA256), not filename, to handle
//! cases where different packages may have identically-named but different sources.
//!
//! # Example
//!
//! ```ignore
//! let source_map = SourceMap::from_file("source-map.json")?;
//! let mut archiver = SourceArchiver::new(Path::new("output/sources"));
//! let index = archiver.archive_all(&source_map)?;
//! ```

use chrono::Utc;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Source type classification for packages
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum SourceType {
    /// Package has accessible source code
    HasSource,
    /// Binary-only package (no source available)
    Binary,
    /// Bootstrap package (part of Nix bootstrap)
    Bootstrap,
    /// Unknown classification
    Unknown,
}

impl Default for SourceType {
    fn default() -> Self {
        Self::Unknown
    }
}

/// Entry in the source map (generated at Nix eval time)
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceMapEntry {
    /// Path to source in Nix store (if available)
    pub src_path: Option<SourcePath>,
    /// Classification of this package's source availability
    #[serde(rename = "type", default)]
    pub source_type: SourceType,
    /// SPDX license identifier
    pub license: Option<String>,
    /// Package name
    pub pname: Option<String>,
    /// Package version
    pub version: Option<String>,
}

/// Source path can be a single path or multiple paths (for packages with srcs)
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum SourcePath {
    Single(String),
    Multiple(Vec<String>),
}

impl SourcePath {
    /// Get all source paths as a vector
    pub fn paths(&self) -> Vec<&str> {
        match self {
            SourcePath::Single(p) => vec![p.as_str()],
            SourcePath::Multiple(ps) => ps.iter().map(|s| s.as_str()).collect(),
        }
    }
}

/// Source map parsed from JSON
pub type SourceMap = HashMap<String, SourceMapEntry>;

/// Parse source map from JSON file
pub fn parse_source_map(path: &Path) -> Result<SourceMap, Error> {
    let content = fs::read_to_string(path).map_err(|e| {
        Error::Io(format!(
            "Failed to read source map '{}': {}",
            path.display(),
            e
        ))
    })?;

    serde_json::from_str(&content).map_err(|e| {
        Error::Parse(format!(
            "Failed to parse source map '{}': {}",
            path.display(),
            e
        ))
    })
}

/// Entry in the source index output file
#[derive(Debug, Clone, Serialize)]
pub struct SourceIndexEntry {
    /// Package URL
    pub purl: String,
    /// Package name
    pub name: String,
    /// Package version
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// Source type classification
    pub source_type: SourceType,
    /// Original source store path
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_store_path: Option<String>,
    /// Archive filename (if archived)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub archive_name: Option<String>,
    /// Archive size in bytes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size_bytes: Option<u64>,
    /// SHA256 hash of archive content
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sha256: Option<String>,
    /// SPDX license identifier
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,
}

/// Coverage statistics for source archiving
#[derive(Debug, Clone, Serialize, Default)]
pub struct SourceCoverage {
    /// Total packages in closure
    pub total_packages: usize,
    /// Packages with accessible source
    pub has_source: usize,
    /// Binary-only packages
    pub binary_only: usize,
    /// Bootstrap packages
    pub bootstrap: usize,
    /// Unknown classification
    pub unknown: usize,
    /// Number of archives created
    pub archived_count: usize,
    /// Total bytes of all archives
    pub archived_bytes: u64,
}

/// Source index output file structure
#[derive(Debug, Clone, Serialize)]
pub struct SourceIndex {
    /// Schema version
    pub schema_version: String,
    /// Generation timestamp (ISO 8601)
    pub generated_at: String,
    /// Coverage statistics
    pub coverage: SourceCoverage,
    /// Per-package source entries
    pub entries: Vec<SourceIndexEntry>,
}

impl SourceIndex {
    /// Create a new empty source index
    pub fn new() -> Self {
        Self {
            schema_version: "1.0".to_string(),
            generated_at: Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
            coverage: SourceCoverage::default(),
            entries: Vec::new(),
        }
    }

    /// Update coverage statistics from entries
    pub fn update_coverage(&mut self) {
        self.coverage.total_packages = self.entries.len();
        self.coverage.has_source = self
            .entries
            .iter()
            .filter(|e| e.source_type == SourceType::HasSource)
            .count();
        self.coverage.binary_only = self
            .entries
            .iter()
            .filter(|e| e.source_type == SourceType::Binary)
            .count();
        self.coverage.bootstrap = self
            .entries
            .iter()
            .filter(|e| e.source_type == SourceType::Bootstrap)
            .count();
        self.coverage.unknown = self
            .entries
            .iter()
            .filter(|e| e.source_type == SourceType::Unknown)
            .count();
        self.coverage.archived_count = self
            .entries
            .iter()
            .filter(|e| e.archive_name.is_some())
            .count();
        self.coverage.archived_bytes = self
            .entries
            .iter()
            .filter_map(|e| e.size_bytes)
            .sum();
    }

    /// Write index to file
    pub fn write_to_file(&self, path: &Path) -> Result<(), Error> {
        let json = serde_json::to_string_pretty(self).map_err(|e| {
            Error::Serialization(format!("Failed to serialize source index: {}", e))
        })?;

        fs::write(path, json).map_err(|e| {
            Error::Io(format!(
                "Failed to write source index to '{}': {}",
                path.display(),
                e
            ))
        })
    }
}

impl Default for SourceIndex {
    fn default() -> Self {
        Self::new()
    }
}

/// Source archiver - handles tarball creation with deduplication
pub struct SourceArchiver {
    /// Output directory for archives
    output_dir: PathBuf,
    /// Content hash -> archive name (for deduplication)
    seen_hashes: HashMap<String, String>,
    /// Stats tracking
    total_archived: usize,
    total_bytes: u64,
}

impl SourceArchiver {
    /// Create a new source archiver
    pub fn new(output_dir: &Path) -> Result<Self, Error> {
        fs::create_dir_all(output_dir).map_err(|e| {
            Error::Io(format!(
                "Failed to create sources directory '{}': {}",
                output_dir.display(),
                e
            ))
        })?;

        Ok(Self {
            output_dir: output_dir.to_path_buf(),
            seen_hashes: HashMap::new(),
            total_archived: 0,
            total_bytes: 0,
        })
    }

    /// Archive a source path and return (archive_name, size, hash)
    ///
    /// Returns None if the path doesn't exist or can't be archived
    pub fn archive_source(
        &mut self,
        source_path: &Path,
        name: &str,
        version: Option<&str>,
    ) -> Result<Option<(String, u64, String)>, Error> {
        if !source_path.exists() {
            eprintln!(
                "Warning: Source path does not exist: {}",
                source_path.display()
            );
            return Ok(None);
        }

        // Calculate content hash
        let content_hash = self.hash_path(source_path)?;
        let short_hash = &content_hash[..8];

        // Check for deduplication
        if let Some(existing_archive) = self.seen_hashes.get(&content_hash) {
            eprintln!(
                "Dedup: {} already archived as {}",
                source_path.display(),
                existing_archive
            );
            return Ok(Some((existing_archive.clone(), 0, content_hash)));
        }

        // Create archive filename
        let safe_name = sanitize_filename(name);
        let archive_name = match version {
            Some(v) => format!("{}-{}-{}.tar.gz", safe_name, sanitize_filename(v), short_hash),
            None => format!("{}-{}.tar.gz", safe_name, short_hash),
        };

        let archive_path = self.output_dir.join(&archive_name);

        // Create tarball
        self.create_tarball(source_path, &archive_path)?;

        // Get size
        let size = fs::metadata(&archive_path)
            .map_err(|e| Error::Io(format!("Failed to stat archive: {}", e)))?
            .len();

        // Track for deduplication
        self.seen_hashes
            .insert(content_hash.clone(), archive_name.clone());
        self.total_archived += 1;
        self.total_bytes += size;

        eprintln!(
            "Archived: {} -> {} ({} bytes)",
            source_path.display(),
            archive_name,
            size
        );

        Ok(Some((archive_name, size, content_hash)))
    }

    /// Calculate SHA256 hash of a path (file or directory)
    fn hash_path(&self, path: &Path) -> Result<String, Error> {
        if path.is_file() {
            self.hash_file(path)
        } else if path.is_dir() {
            self.hash_directory(path)
        } else {
            Err(Error::Io(format!(
                "Path is neither file nor directory: {}",
                path.display()
            )))
        }
    }

    /// Hash a single file
    fn hash_file(&self, path: &Path) -> Result<String, Error> {
        let mut file = File::open(path).map_err(|e| {
            Error::Io(format!("Failed to open file '{}': {}", path.display(), e))
        })?;

        let mut hasher = Sha256::new();
        let mut buffer = [0u8; 8192];

        loop {
            let n = file.read(&mut buffer).map_err(|e| {
                Error::Io(format!("Failed to read file '{}': {}", path.display(), e))
            })?;
            if n == 0 {
                break;
            }
            hasher.update(&buffer[..n]);
        }

        Ok(format!("{:x}", hasher.finalize()))
    }

    /// Hash a directory (by listing all files and hashing their names and contents)
    fn hash_directory(&self, path: &Path) -> Result<String, Error> {
        let mut hasher = Sha256::new();

        // Collect and sort entries for deterministic hashing
        let mut entries: Vec<_> = walkdir::WalkDir::new(path)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
            .collect();

        entries.sort_by(|a, b| a.path().cmp(b.path()));

        for entry in entries {
            let rel_path = entry
                .path()
                .strip_prefix(path)
                .unwrap_or(entry.path())
                .to_string_lossy();

            // Hash the relative path
            hasher.update(rel_path.as_bytes());

            // Hash file contents (not directories)
            if entry.file_type().is_file() {
                if let Ok(content) = fs::read(entry.path()) {
                    hasher.update(&content);
                }
            }
        }

        Ok(format!("{:x}", hasher.finalize()))
    }

    /// Create a tarball from source path
    fn create_tarball(&self, source: &Path, dest: &Path) -> Result<(), Error> {
        // Use tar command for efficiency and to handle all edge cases
        let status = Command::new("tar")
            .args([
                "-czf",
                &dest.to_string_lossy(),
                "-C",
                &source.parent().unwrap_or(source).to_string_lossy(),
                &source
                    .file_name()
                    .unwrap_or(source.as_os_str())
                    .to_string_lossy(),
            ])
            .status()
            .map_err(|e| Error::Io(format!("Failed to execute tar: {}", e)))?;

        if !status.success() {
            return Err(Error::Archive(format!(
                "tar failed with status: {}",
                status
            )));
        }

        Ok(())
    }

    /// Get total archived count
    pub fn total_archived(&self) -> usize {
        self.total_archived
    }

    /// Get total bytes archived
    pub fn total_bytes(&self) -> u64 {
        self.total_bytes
    }
}

/// Archive all sources from a source map and generate index
pub fn archive_sources(
    source_map: &SourceMap,
    output_dir: &Path,
) -> Result<SourceIndex, Error> {
    let sources_dir = output_dir.join("sources");
    let mut archiver = SourceArchiver::new(&sources_dir)?;
    let mut index = SourceIndex::new();

    for (store_path, entry) in source_map {
        // Parse store path for purl
        let parsed = crate::nix::closure_graph::StorePath::parse(store_path);
        let purl = parsed.purl();
        let name = entry.pname.as_deref().unwrap_or(&parsed.name);
        let version = entry.version.as_deref().or(parsed.version.as_deref());

        let index_entry = if let Some(ref src_path) = entry.src_path {
            // Try to archive each source path
            let paths = src_path.paths();
            let mut archive_name = None;
            let mut size_bytes = None;
            let mut sha256 = None;
            let mut source_store_path = None;

            for path_str in paths {
                let path = Path::new(path_str);
                if let Some((arch_name, size, hash)) =
                    archiver.archive_source(path, name, version)?
                {
                    archive_name = Some(arch_name);
                    size_bytes = Some(size);
                    sha256 = Some(hash);
                    source_store_path = Some(path_str.to_string());
                    break; // Use first successful archive
                }
            }

            SourceIndexEntry {
                purl,
                name: name.to_string(),
                version: version.map(String::from),
                source_type: if archive_name.is_some() {
                    SourceType::HasSource
                } else {
                    entry.source_type.clone()
                },
                source_store_path,
                archive_name,
                size_bytes,
                sha256,
                license: entry.license.clone(),
            }
        } else {
            // No source path available
            SourceIndexEntry {
                purl,
                name: name.to_string(),
                version: version.map(String::from),
                source_type: entry.source_type.clone(),
                source_store_path: None,
                archive_name: None,
                size_bytes: None,
                sha256: None,
                license: entry.license.clone(),
            }
        };

        index.entries.push(index_entry);
    }

    // Sort entries by name for consistent output
    index.entries.sort_by(|a, b| a.name.cmp(&b.name));

    // Update coverage stats
    index.update_coverage();

    Ok(index)
}

/// Sanitize a string for use in a filename
fn sanitize_filename(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '.' || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

/// Error type for source archive operations
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(String),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Archive error: {0}")]
    Archive(String),

    #[error("Serialization error: {0}")]
    Serialization(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_source_type_serialization() {
        let json = serde_json::to_string(&SourceType::HasSource).unwrap();
        assert_eq!(json, "\"has-source\"");

        let json = serde_json::to_string(&SourceType::Bootstrap).unwrap();
        assert_eq!(json, "\"bootstrap\"");
    }

    #[test]
    fn test_source_map_entry_parsing() {
        let json = r#"{
            "srcPath": "/nix/store/xxx-src",
            "type": "has-source",
            "license": "MIT",
            "pname": "test-pkg",
            "version": "1.0.0"
        }"#;

        let entry: SourceMapEntry = serde_json::from_str(json).unwrap();
        assert_eq!(entry.pname, Some("test-pkg".to_string()));
        assert_eq!(entry.source_type, SourceType::HasSource);
    }

    #[test]
    fn test_source_map_entry_multiple_sources() {
        let json = r#"{
            "srcPath": ["/nix/store/xxx-src1", "/nix/store/xxx-src2"],
            "type": "has-source",
            "pname": "test-pkg"
        }"#;

        let entry: SourceMapEntry = serde_json::from_str(json).unwrap();
        if let Some(SourcePath::Multiple(paths)) = entry.src_path {
            assert_eq!(paths.len(), 2);
        } else {
            panic!("Expected multiple paths");
        }
    }

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(sanitize_filename("hello-world"), "hello-world");
        assert_eq!(sanitize_filename("hello/world"), "hello_world");
        assert_eq!(sanitize_filename("pkg@1.0"), "pkg_1.0");
    }

    #[test]
    fn test_source_index_coverage() {
        let mut index = SourceIndex::new();
        index.entries.push(SourceIndexEntry {
            purl: "pkg:nix/test@1.0".to_string(),
            name: "test".to_string(),
            version: Some("1.0".to_string()),
            source_type: SourceType::HasSource,
            source_store_path: Some("/nix/store/xxx".to_string()),
            archive_name: Some("test-1.0-abc.tar.gz".to_string()),
            size_bytes: Some(1024),
            sha256: Some("abc123".to_string()),
            license: Some("MIT".to_string()),
        });
        index.entries.push(SourceIndexEntry {
            purl: "pkg:nix/bootstrap".to_string(),
            name: "bootstrap".to_string(),
            version: None,
            source_type: SourceType::Bootstrap,
            source_store_path: None,
            archive_name: None,
            size_bytes: None,
            sha256: None,
            license: None,
        });

        index.update_coverage();

        assert_eq!(index.coverage.total_packages, 2);
        assert_eq!(index.coverage.has_source, 1);
        assert_eq!(index.coverage.bootstrap, 1);
        assert_eq!(index.coverage.archived_count, 1);
        assert_eq!(index.coverage.archived_bytes, 1024);
    }

    #[test]
    fn test_archiver_hash_file() {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("test.txt");
        let mut file = File::create(&file_path).unwrap();
        write!(file, "hello world").unwrap();

        let archiver = SourceArchiver::new(dir.path()).unwrap();
        let hash = archiver.hash_file(&file_path).unwrap();

        // SHA256 of "hello world"
        assert_eq!(
            hash,
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
        );
    }
}
