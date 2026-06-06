//! Runtime reader over a Firestream charts symlink farm.
//!
//! Given a `charts_dir` containing:
//!
//! ```text
//! charts_dir/
//!   index.json
//!   airflow/chart-manifest.json
//!   postgresql/chart-manifest.json
//!   ...
//! ```
//!
//! `Charts::open(charts_dir)` parses `index.json` eagerly. Per-chart
//! manifests are parsed lazily on first access and cached.
//!
//! The cache is wrapped in `RwLock` because the top-level `firestream`
//! crate is async / multi-threaded and may resolve charts concurrently.
//! Manifests are cheap to clone, so `get`/`stack` return owned
//! `ChartManifest` values rather than references — this keeps the lock
//! scope tight and the API simple.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::RwLock;

use crate::spec::{ChartManifest, Index};
use crate::Error;

/// Name of the aggregate index file at the root of the charts farm.
pub const INDEX_FILE: &str = "index.json";

/// Reader over a charts symlink farm.
pub struct Charts {
    /// Directory containing `index.json` and one subdir per chart.
    charts_dir: PathBuf,

    /// Eagerly-loaded index.
    index: Index,

    /// Lazy cache of per-chart manifests, keyed by Firestream chart name.
    cache: RwLock<HashMap<String, ChartManifest>>,
}

impl Charts {
    /// Open a charts symlink farm. Reads and parses `index.json`
    /// immediately; per-chart manifests are loaded on demand.
    pub fn open(charts_dir: impl AsRef<Path>) -> Result<Self, Error> {
        let charts_dir = charts_dir.as_ref().to_path_buf();
        let index_path = charts_dir.join(INDEX_FILE);

        let content = match std::fs::read_to_string(&index_path) {
            Ok(c) => c,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                return Err(Error::IndexNotFound(index_path.display().to_string()));
            }
            Err(e) => {
                return Err(Error::Io {
                    file: index_path.display().to_string(),
                    source: e,
                });
            }
        };

        let index: Index = serde_json::from_str(&content).map_err(|e| Error::ParseError {
            file: index_path.display().to_string(),
            source: e,
        })?;

        Ok(Self {
            charts_dir,
            index,
            cache: RwLock::new(HashMap::new()),
        })
    }

    /// Returns the parsed index. Useful for inspecting schema version and
    /// stack definitions without loading every manifest.
    pub fn index(&self) -> &Index {
        &self.index
    }

    /// Path to the charts root directory.
    pub fn charts_dir(&self) -> &Path {
        &self.charts_dir
    }

    /// List the names of all charts registered in the index. Order is
    /// determined by `BTreeMap` iteration (lexicographic).
    pub fn list(&self) -> Vec<&str> {
        self.index.charts.keys().map(String::as_str).collect()
    }

    /// List the names of all stacks registered in the index.
    pub fn list_stacks(&self) -> Vec<&str> {
        self.index.stacks.keys().map(String::as_str).collect()
    }

    /// Load (or read from cache) the manifest for the named chart.
    ///
    /// Returns an owned [`ChartManifest`] (cheap clone). Errors:
    ///
    /// - [`Error::ChartNotInIndex`] — name isn't in `index.charts`.
    /// - [`Error::ManifestNotFound`] — file referenced by the index is
    ///   missing on disk.
    /// - [`Error::ParseError`] — file exists but doesn't match the v1
    ///   schema.
    pub fn get(&self, name: &str) -> Result<ChartManifest, Error> {
        // Fast path: already cached.
        {
            let cache = self.cache.read().map_err(|_| Error::LockPoisoned)?;
            if let Some(m) = cache.get(name) {
                return Ok(m.clone());
            }
        }

        // Look up in the index.
        let entry = self
            .index
            .charts
            .get(name)
            .ok_or_else(|| Error::ChartNotInIndex(name.to_string()))?;

        // Resolve relative path against the charts directory.
        let manifest_path = self.charts_dir.join(&entry.manifest_path);

        let content = match std::fs::read_to_string(&manifest_path) {
            Ok(c) => c,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                return Err(Error::ManifestNotFound(manifest_path.display().to_string()));
            }
            Err(e) => {
                return Err(Error::Io {
                    file: manifest_path.display().to_string(),
                    source: e,
                });
            }
        };

        let manifest: ChartManifest =
            serde_json::from_str(&content).map_err(|e| Error::ParseError {
                file: manifest_path.display().to_string(),
                source: e,
            })?;

        // Insert into cache and return a clone.
        let mut cache = self.cache.write().map_err(|_| Error::LockPoisoned)?;
        cache.insert(name.to_string(), manifest.clone());
        Ok(manifest)
    }

    /// Resolve the named stack to an ordered list of manifests.
    ///
    /// Stack entries that aren't registered in `index.charts` are SKIPPED
    /// (with a `tracing::warn`) rather than erroring — per Agent C's
    /// note, stacks are allowed to reference charts that Wave-3 hasn't
    /// wired up yet.
    ///
    /// Errors only if the *stack itself* isn't in the index, or if a chart
    /// that IS in the index has an unreadable / malformed manifest.
    pub fn stack(&self, name: &str) -> Result<Vec<ChartManifest>, Error> {
        let entries = self
            .index
            .stacks
            .get(name)
            .ok_or_else(|| Error::StackNotInIndex(name.to_string()))?;

        let mut out = Vec::with_capacity(entries.len());
        for chart_name in entries {
            match self.get(chart_name) {
                Ok(m) => out.push(m),
                Err(Error::ChartNotInIndex(_)) => {
                    tracing::warn!(
                        stack = %name,
                        chart = %chart_name,
                        "stack references chart not yet registered in index; skipping"
                    );
                }
                Err(e) => return Err(e),
            }
        }
        Ok(out)
    }
}

impl std::fmt::Debug for Charts {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Charts")
            .field("charts_dir", &self.charts_dir)
            .field("schema_version", &self.index.schema_version)
            .field("charts", &self.index.charts.keys().collect::<Vec<_>>())
            .field("stacks", &self.index.stacks.keys().collect::<Vec<_>>())
            .finish()
    }
}
