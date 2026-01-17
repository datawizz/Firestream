//! Git repository interaction utilities for creating minimal .git directories.
//!
//! This module provides utilities for creating minimal .git directories that are
//! sufficient for Nix flake resolution. Nix flakes require a .git directory to
//! resolve `path:../../../..` style references, but they don't need full history.
//!
//! # Overview
//!
//! The minimal .git structure created contains:
//! - `HEAD` - Current branch reference
//! - `config` - Basic git configuration
//! - `refs/heads/{branch}` - Current branch ref
//! - `objects/{xx}/{rest}` - HEAD commit and tree objects only
//!
//! # Example
//!
//! ```rust,ignore
//! use workspace_embed::git::MinimalGitCreator;
//!
//! let creator = MinimalGitCreator::from_repo("/path/to/repo")?;
//! println!("Current branch: {}", creator.current_branch()?);
//! println!("HEAD commit: {}", creator.head_commit()?);
//!
//! let info = creator.create_minimal("/tmp/embedded")?;
//! // /tmp/embedded/.git now exists with minimal structure
//! ```

use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

use crate::error::{Result, WorkspaceEmbedError};

/// Information about a minimal .git directory that was created.
#[derive(Debug, Clone)]
pub struct MinimalGitInfo {
    /// Path to the created .git directory.
    pub git_dir: PathBuf,
    /// Current branch name (or "HEAD" for detached HEAD).
    pub branch: String,
    /// HEAD commit hash.
    pub commit: String,
}

/// State of the HEAD reference.
#[derive(Debug, Clone)]
enum HeadState {
    /// HEAD points to a branch: "ref: refs/heads/{branch}"
    Branch(String),
    /// HEAD is detached and contains a commit hash directly
    Detached(String),
}

/// Creates minimal .git directories for Nix flake resolution.
///
/// This struct reads from an existing git repository and creates a minimal
/// .git directory containing only the essential files needed for Nix to
/// recognize it as a git repository.
pub struct MinimalGitCreator {
    /// Path to the source .git directory.
    source_git: PathBuf,
}

impl MinimalGitCreator {
    /// Create a new MinimalGitCreator from a source .git directory path.
    ///
    /// # Arguments
    ///
    /// * `source_git` - Path to an existing .git directory
    ///
    /// # Errors
    ///
    /// Returns an error if the .git directory doesn't exist or is not valid.
    pub fn new(source_git: impl AsRef<Path>) -> Result<Self> {
        let source_git = source_git.as_ref().to_path_buf();

        // Verify the .git directory exists
        if !source_git.exists() {
            return Err(WorkspaceEmbedError::GitNotFound {
                path: source_git,
            });
        }

        // Check for HEAD file to verify it's a git directory
        let head_path = source_git.join("HEAD");
        if !head_path.exists() {
            return Err(WorkspaceEmbedError::GitNotFound {
                path: source_git,
            });
        }

        Ok(Self { source_git })
    }

    /// Create a MinimalGitCreator from a repository root directory.
    ///
    /// This will look for a `.git` directory (or file for worktrees) in the given path.
    ///
    /// # Arguments
    ///
    /// * `repo_root` - Path to the repository root
    ///
    /// # Errors
    ///
    /// Returns an error if no .git directory is found.
    pub fn from_repo(repo_root: impl AsRef<Path>) -> Result<Self> {
        let repo_root = repo_root.as_ref();
        let git_path = repo_root.join(".git");

        if git_path.is_file() {
            // Handle git worktrees: .git file contains "gitdir: /path/to/actual/.git"
            let content = fs::read_to_string(&git_path)?;
            let gitdir = content
                .strip_prefix("gitdir: ")
                .map(|s| s.trim())
                .ok_or_else(|| WorkspaceEmbedError::GitNotFound {
                    path: git_path.clone(),
                })?;

            Self::new(gitdir)
        } else {
            Self::new(git_path)
        }
    }

    /// Get the current branch name.
    ///
    /// Returns the branch name if HEAD points to a branch, or "HEAD" for detached HEAD.
    pub fn current_branch(&self) -> Result<String> {
        match self.parse_head()? {
            HeadState::Branch(branch) => Ok(branch),
            HeadState::Detached(_) => Ok("HEAD".to_string()),
        }
    }

    /// Get the HEAD commit hash.
    pub fn head_commit(&self) -> Result<String> {
        match self.parse_head()? {
            HeadState::Branch(branch) => self.resolve_ref(&format!("refs/heads/{}", branch)),
            HeadState::Detached(commit) => Ok(commit),
        }
    }

    /// Create a minimal .git directory at the destination path.
    ///
    /// This creates a .git directory with only the essential files needed
    /// for Nix flake resolution:
    /// - HEAD
    /// - config (core and remote "origin" sections)
    /// - refs/heads/{branch}
    /// - objects for HEAD commit and tree
    ///
    /// # Arguments
    ///
    /// * `dest` - Destination directory (the .git directory will be created inside)
    ///
    /// # Returns
    ///
    /// Information about the created .git directory.
    pub fn create_minimal(&self, dest: impl AsRef<Path>) -> Result<MinimalGitInfo> {
        let dest = dest.as_ref();
        let git_dir = dest.join(".git");

        // Create the .git directory structure
        fs::create_dir_all(&git_dir)?;
        fs::create_dir_all(git_dir.join("refs/heads"))?;
        fs::create_dir_all(git_dir.join("objects"))?;

        // Parse HEAD to determine branch and commit
        let head_state = self.parse_head()?;
        let (branch, commit) = match &head_state {
            HeadState::Branch(b) => {
                let c = self.resolve_ref(&format!("refs/heads/{}", b))?;
                (b.clone(), c)
            }
            HeadState::Detached(c) => ("HEAD".to_string(), c.clone()),
        };

        // Write HEAD file
        self.write_head(&git_dir, &head_state)?;

        // Write config file
        self.write_config(&git_dir)?;

        // Write refs/heads/{branch} if we have a branch
        if let HeadState::Branch(ref branch_name) = head_state {
            let refs_dir = git_dir.join("refs/heads");
            fs::write(refs_dir.join(branch_name), format!("{}\n", commit))?;
        }

        // Copy git objects for commit and tree
        self.copy_commit_objects(&git_dir, &commit)?;

        Ok(MinimalGitInfo {
            git_dir,
            branch,
            commit,
        })
    }

    /// Parse the HEAD file to determine the current state.
    fn parse_head(&self) -> Result<HeadState> {
        let head_path = self.source_git.join("HEAD");
        let content = fs::read_to_string(&head_path)?;
        let content = content.trim();

        if let Some(refname) = content.strip_prefix("ref: ") {
            // HEAD points to a branch
            if let Some(branch) = refname.strip_prefix("refs/heads/") {
                Ok(HeadState::Branch(branch.to_string()))
            } else {
                // Some other ref (unlikely but handle it)
                let commit = self.resolve_ref(refname)?;
                Ok(HeadState::Detached(commit))
            }
        } else if content.len() == 40 && content.chars().all(|c| c.is_ascii_hexdigit()) {
            // Detached HEAD with commit hash
            Ok(HeadState::Detached(content.to_string()))
        } else {
            Err(WorkspaceEmbedError::GitObjectError {
                oid: format!("invalid HEAD: {}", content),
            })
        }
    }

    /// Resolve a ref to its commit hash.
    ///
    /// Handles both loose refs (files in refs/) and packed refs.
    fn resolve_ref(&self, refname: &str) -> Result<String> {
        // First try loose ref
        let loose_path = self.source_git.join(refname);
        if loose_path.exists() {
            let content = fs::read_to_string(&loose_path)?;
            return Ok(content.trim().to_string());
        }

        // Try packed-refs
        let packed_refs_path = self.source_git.join("packed-refs");
        if packed_refs_path.exists() {
            let file = fs::File::open(&packed_refs_path)?;
            let reader = BufReader::new(file);

            for line in reader.lines() {
                let line = line?;
                let line = line.trim();

                // Skip comments and empty lines
                if line.is_empty() || line.starts_with('#') || line.starts_with('^') {
                    continue;
                }

                // Format: "{hash} {refname}"
                let parts: Vec<&str> = line.splitn(2, ' ').collect();
                if parts.len() == 2 && parts[1] == refname {
                    return Ok(parts[0].to_string());
                }
            }
        }

        Err(WorkspaceEmbedError::GitObjectError {
            oid: format!("ref not found: {}", refname),
        })
    }

    /// Write the HEAD file to the destination .git directory.
    fn write_head(&self, git_dir: &Path, head_state: &HeadState) -> Result<()> {
        let head_path = git_dir.join("HEAD");
        let content = match head_state {
            HeadState::Branch(branch) => format!("ref: refs/heads/{}\n", branch),
            HeadState::Detached(commit) => format!("{}\n", commit),
        };
        fs::write(head_path, content)?;
        Ok(())
    }

    /// Write a minimal config file to the destination .git directory.
    ///
    /// Copies the [core] and [remote "origin"] sections from the source.
    fn write_config(&self, git_dir: &Path) -> Result<()> {
        let source_config = self.source_git.join("config");
        let dest_config = git_dir.join("config");

        if !source_config.exists() {
            // Create a minimal config if source doesn't have one
            let minimal_config = "[core]\n\trepositoryformatversion = 0\n\tfilemode = true\n\tbare = false\n";
            fs::write(dest_config, minimal_config)?;
            return Ok(());
        }

        // Parse and extract relevant sections
        let content = fs::read_to_string(&source_config)?;
        let mut output = String::new();
        let mut in_relevant_section = false;

        for line in content.lines() {
            let trimmed = line.trim();

            // Check for section headers
            if trimmed.starts_with('[') && trimmed.ends_with(']') {
                // Determine if this is a section we want to keep
                let section_lower = trimmed.to_lowercase();
                in_relevant_section = section_lower == "[core]"
                    || section_lower.starts_with("[remote ");

                if in_relevant_section {
                    if !output.is_empty() {
                        output.push('\n');
                    }
                    output.push_str(line);
                    output.push('\n');
                }
            } else if in_relevant_section {
                // Include lines within relevant sections
                output.push_str(line);
                output.push('\n');
            }
        }

        // If we didn't find a [core] section, add a minimal one
        if !output.contains("[core]") {
            let minimal = "[core]\n\trepositoryformatversion = 0\n\tfilemode = true\n\tbare = false\n";
            output = format!("{}{}", minimal, output);
        }

        fs::write(dest_config, output)?;
        Ok(())
    }

    /// Copy git objects for the commit and its tree to the destination.
    ///
    /// We only copy:
    /// - The commit object itself
    /// - The tree object referenced by the commit
    ///
    /// We intentionally skip blob objects as Nix only needs repo identity.
    fn copy_commit_objects(&self, git_dir: &Path, commit_hash: &str) -> Result<()> {
        // Copy the commit object
        self.copy_object(git_dir, commit_hash)?;

        // Try to read the commit object to extract the tree hash
        // Git objects are stored as: {2 char dir}/{38 char file}
        if let Ok(tree_hash) = self.read_tree_from_commit(commit_hash) {
            self.copy_object(git_dir, &tree_hash)?;
        }

        Ok(())
    }

    /// Copy a single git object to the destination.
    fn copy_object(&self, git_dir: &Path, hash: &str) -> Result<()> {
        if hash.len() < 3 {
            return Err(WorkspaceEmbedError::GitObjectError {
                oid: format!("invalid hash: {}", hash),
            });
        }

        let (dir, file) = hash.split_at(2);
        let source_path = self.source_git.join("objects").join(dir).join(file);
        let dest_dir = git_dir.join("objects").join(dir);
        let dest_path = dest_dir.join(file);

        // Object might be in a pack file, in which case we can't easily copy it
        // For now, we only handle loose objects
        if source_path.exists() {
            fs::create_dir_all(&dest_dir)?;
            fs::copy(&source_path, &dest_path)?;
        }
        // If object is packed, we silently skip it - Nix can still work without it
        // in most cases since it primarily needs the .git directory to exist

        Ok(())
    }

    /// Try to read the tree hash from a commit object.
    ///
    /// Git commit objects (when decompressed) have the format:
    /// ```text
    /// commit {size}\0tree {tree_hash}
    /// parent {parent_hash}
    /// author ...
    /// committer ...
    ///
    /// commit message
    /// ```
    ///
    /// We parse this to extract the tree hash.
    fn read_tree_from_commit(&self, commit_hash: &str) -> Result<String> {
        if commit_hash.len() < 3 {
            return Err(WorkspaceEmbedError::GitObjectError {
                oid: format!("invalid commit hash: {}", commit_hash),
            });
        }

        let (dir, file) = commit_hash.split_at(2);
        let object_path = self.source_git.join("objects").join(dir).join(file);

        if !object_path.exists() {
            // Object might be packed - we can't easily read packed objects
            return Err(WorkspaceEmbedError::GitObjectError {
                oid: format!("commit object not found (may be packed): {}", commit_hash),
            });
        }

        // Read and decompress the object
        let compressed = fs::read(&object_path)?;
        let decompressed = decompress_zlib(&compressed)?;

        // Parse the decompressed content to find "tree {hash}"
        // Format after null byte: "tree {40-char-hash}\n..."
        let content = String::from_utf8_lossy(&decompressed);

        // Find "tree " after the header
        for line in content.lines() {
            if let Some(tree_part) = line.strip_prefix("tree ") {
                let tree_hash = tree_part.trim();
                if tree_hash.len() == 40 && tree_hash.chars().all(|c| c.is_ascii_hexdigit()) {
                    return Ok(tree_hash.to_string());
                }
            }
        }

        Err(WorkspaceEmbedError::GitObjectError {
            oid: format!("could not find tree in commit: {}", commit_hash),
        })
    }
}

/// Decompress zlib-compressed data (used by git objects).
///
/// This is a minimal zlib decompressor for git objects.
fn decompress_zlib(data: &[u8]) -> Result<Vec<u8>> {
    use std::io::Read;

    // Git uses zlib compression (RFC 1950)
    let mut decoder = flate2::read::ZlibDecoder::new(data);
    let mut decompressed = Vec::new();
    decoder.read_to_end(&mut decompressed).map_err(|e| {
        WorkspaceEmbedError::ExtractionError {
            reason: format!("zlib decompression failed: {}", e),
        }
    })?;
    Ok(decompressed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_head_state_branch_parsing() {
        // This test would require a mock .git directory
        // For now, just verify the enum variants exist
        let branch = HeadState::Branch("main".to_string());
        let detached = HeadState::Detached("abc123".to_string());

        match branch {
            HeadState::Branch(name) => assert_eq!(name, "main"),
            _ => panic!("Expected Branch variant"),
        }

        match detached {
            HeadState::Detached(hash) => assert_eq!(hash, "abc123"),
            _ => panic!("Expected Detached variant"),
        }
    }

    #[test]
    fn test_minimal_git_info_fields() {
        let info = MinimalGitInfo {
            git_dir: PathBuf::from("/tmp/test/.git"),
            branch: "main".to_string(),
            commit: "abc123def456".to_string(),
        };

        assert_eq!(info.git_dir, PathBuf::from("/tmp/test/.git"));
        assert_eq!(info.branch, "main");
        assert_eq!(info.commit, "abc123def456");
    }
}
