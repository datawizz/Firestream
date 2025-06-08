use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use similar::{ChangeTag, TextDiff};
use std::fs::{self, Metadata};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::process::Command;
use tokio::fs as async_fs;
use walkdir::WalkDir;

/// Configuration for the filesystem tool
#[derive(Debug, Clone)]
pub struct FilesystemConfig {
    allowed_directories: Vec<PathBuf>,
}

impl FilesystemConfig {
    /// Create a new filesystem configuration with the given allowed directories
    pub fn new(allowed_directories: Vec<PathBuf>) -> Result<Self> {
        let normalized_dirs: Result<Vec<_>> = allowed_directories
            .into_iter()
            .map(|dir| {
                let expanded = expand_home(&dir)?;
                let canonical = fs::canonicalize(&expanded)
                    .with_context(|| format!("Failed to canonicalize directory: {:?}", expanded))?;
                Ok(canonical)
            })
            .collect();
        
        Ok(Self {
            allowed_directories: normalized_dirs?,
        })
    }
    
    /// Validate that a path is within the allowed directories
    pub fn validate_path(&self, path: &Path) -> Result<PathBuf> {
        let expanded = expand_home(path)?;
        let absolute = if expanded.is_absolute() {
            expanded
        } else {
            std::env::current_dir()?.join(expanded)
        };
        
        // Try to get the canonical path if it exists
        let canonical = match fs::canonicalize(&absolute) {
            Ok(path) => path,
            Err(_) => {
                // For non-existent files, find the first existing parent
                let mut current = absolute.clone();
                let mut non_existent_parts = Vec::new();
                
                // First, collect all non-existent parts
                loop {
                    if current.exists() {
                        // Found an existing directory, canonicalize it
                        let canonical_base = fs::canonicalize(&current)
                            .with_context(|| format!("Failed to canonicalize: {:?}", current))?;
                        
                        // Rebuild the path with the canonical base
                        let mut result = canonical_base;
                        for part in non_existent_parts.iter().rev() {
                            result = result.join(part);
                        }
                        break result;
                    } else {
                        // This part doesn't exist, store it and continue up
                        if let Some(file_name) = current.file_name() {
                            non_existent_parts.push(file_name.to_os_string());
                        }
                        
                        if let Some(parent) = current.parent() {
                            current = parent.to_path_buf();
                        } else {
                            return Err(anyhow!("Could not find any existing parent directory for: {:?}", absolute));
                        }
                    }
                }
            }
        };
        
        // Check if path is within allowed directories
        let is_allowed = self.allowed_directories.iter().any(|allowed| {
            canonical.starts_with(allowed)
        });
        
        if !is_allowed {
            return Err(anyhow!(
                "Access denied - path outside allowed directories: {:?}",
                canonical
            ));
        }
        
        Ok(canonical)
    }
}

/// Expand ~ to home directory
fn expand_home(path: &Path) -> Result<PathBuf> {
    let path_str = path.to_string_lossy();
    if path_str.starts_with("~/") {
        let home_dir = home::home_dir()
            .ok_or_else(|| anyhow!("Could not determine home directory"))?;
        Ok(home_dir.join(&path_str[2..]))
    } else if path_str == "~" {
        home::home_dir()
            .ok_or_else(|| anyhow!("Could not determine home directory"))
    } else {
        Ok(path.to_path_buf())
    }
}

/// Check if ripgrep is available in the system PATH
fn is_ripgrep_available() -> bool {
    Command::new("rg")
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// File information structure
#[derive(Debug, Serialize, Deserialize)]
pub struct FileInfo {
    pub size: u64,
    pub created: Option<DateTime<Utc>>,
    pub modified: DateTime<Utc>,
    pub accessed: DateTime<Utc>,
    pub is_directory: bool,
    pub is_file: bool,
    pub permissions: String,
}

impl FileInfo {
    fn from_metadata(metadata: &Metadata) -> Result<Self> {
        let created = metadata.created().ok().map(|t| DateTime::from(t));
        let modified = DateTime::from(metadata.modified()?);
        let accessed = DateTime::from(metadata.accessed()?);
        
        let permissions = format!("{:o}", metadata.permissions().mode() & 0o777);
        
        Ok(FileInfo {
            size: metadata.len(),
            created,
            modified,
            accessed,
            is_directory: metadata.is_dir(),
            is_file: metadata.file_type().is_file(),
            permissions,
        })
    }
}

/// Edit operation for modifying files
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditOperation {
    pub old_text: String,
    pub new_text: String,
}

/// Tree entry for directory tree representation
#[derive(Debug, Serialize, Deserialize)]
pub struct TreeEntry {
    pub name: String,
    #[serde(rename = "type")]
    pub entry_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<TreeEntry>>,
}

/// Main filesystem operations struct
pub struct FilesystemOps {
    config: FilesystemConfig,
}

impl FilesystemOps {
    /// Create a new filesystem operations instance
    pub fn new(config: FilesystemConfig) -> Self {
        Self { config }
    }
    
    /// Read a single file
    pub async fn read_file(&self, path: &Path) -> Result<String> {
        let valid_path = self.config.validate_path(path)?;
        let content = async_fs::read_to_string(&valid_path)
            .await
            .with_context(|| format!("Failed to read file: {:?}", valid_path))?;
        Ok(content)
    }
    
    /// Read multiple files
    pub async fn read_multiple_files(&self, paths: &[PathBuf]) -> Vec<Result<(PathBuf, String)>> {
        let mut results = Vec::new();
        
        for path in paths {
            let result = match self.read_file(path).await {
                Ok(content) => Ok((path.clone(), content)),
                Err(e) => Err(e),
            };
            results.push(result);
        }
        
        results
    }
    
    /// Write a file
    pub async fn write_file(&self, path: &Path, content: &str) -> Result<()> {
        let valid_path = self.config.validate_path(path)?;
        
        // Create parent directory if it doesn't exist
        if let Some(parent) = valid_path.parent() {
            async_fs::create_dir_all(parent).await?;
        }
        
        async_fs::write(&valid_path, content)
            .await
            .with_context(|| format!("Failed to write file: {:?}", valid_path))?;
        Ok(())
    }
    
    /// Edit a file with the given operations
    pub async fn edit_file(
        &self,
        path: &Path,
        edits: &[EditOperation],
        dry_run: bool,
    ) -> Result<String> {
        let valid_path = self.config.validate_path(path)?;
        let content = self.read_file(&valid_path).await?;
        
        let mut modified_content = content.clone();
        
        for edit in edits {
            modified_content = apply_edit(&modified_content, edit)?;
        }
        
        // Generate diff
        let diff = create_unified_diff(&content, &modified_content, path);
        
        if !dry_run {
            self.write_file(&valid_path, &modified_content).await?;
        }
        
        Ok(diff)
    }
    
    /// Create a directory
    pub async fn create_directory(&self, path: &Path) -> Result<()> {
        let valid_path = self.config.validate_path(path)?;
        async_fs::create_dir_all(&valid_path)
            .await
            .with_context(|| format!("Failed to create directory: {:?}", valid_path))?;
        Ok(())
    }
    
    /// List directory contents
    pub async fn list_directory(&self, path: &Path) -> Result<Vec<(String, bool)>> {
        let valid_path = self.config.validate_path(path)?;
        let mut entries = Vec::new();
        
        let mut dir = async_fs::read_dir(&valid_path).await?;
        while let Some(entry) = dir.next_entry().await? {
            let name = entry.file_name().to_string_lossy().to_string();
            let is_dir = entry.file_type().await?.is_dir();
            entries.push((name, is_dir));
        }
        
        entries.sort_by(|a, b| a.0.cmp(&b.0));
        Ok(entries)
    }
    
    /// Get directory tree
    pub async fn directory_tree(&self, path: &Path) -> Result<Vec<TreeEntry>> {
        let valid_path = self.config.validate_path(path)?;
        self.build_tree(valid_path).await
    }
    
    fn build_tree(&self, path: PathBuf) -> Pin<Box<dyn std::future::Future<Output = Result<Vec<TreeEntry>>> + Send + '_>> {
        Box::pin(async move {
            let mut entries = Vec::new();
            let mut dir = async_fs::read_dir(&path).await?;
            
            while let Some(entry) = dir.next_entry().await? {
                let name = entry.file_name().to_string_lossy().to_string();
                let file_type = entry.file_type().await?;
                
                let mut tree_entry = TreeEntry {
                    name,
                    entry_type: if file_type.is_dir() {
                        "directory".to_string()
                    } else {
                        "file".to_string()
                    },
                    children: None,
                };
                
                if file_type.is_dir() {
                    let child_path = entry.path();
                    if let Ok(children) = self.build_tree(child_path).await {
                        tree_entry.children = Some(children);
                    }
                }
                
                entries.push(tree_entry);
            }
            
            entries.sort_by(|a, b| a.name.cmp(&b.name));
            Ok(entries)
        })
    }
    
    /// Move or rename a file
    pub async fn move_file(&self, source: &Path, destination: &Path) -> Result<()> {
        let valid_source = self.config.validate_path(source)?;
        let valid_dest = self.config.validate_path(destination)?;
        
        // Check if destination exists
        if valid_dest.exists() {
            return Err(anyhow!("Destination already exists: {:?}", valid_dest));
        }
        
        // Create parent directory if needed
        if let Some(parent) = valid_dest.parent() {
            async_fs::create_dir_all(parent).await?;
        }
        
        async_fs::rename(&valid_source, &valid_dest)
            .await
            .with_context(|| {
                format!("Failed to move {:?} to {:?}", valid_source, valid_dest)
            })?;
        Ok(())
    }
    
    /// Search for files matching a pattern
    /// 
    /// This function automatically uses ripgrep (`rg`) if available on the system PATH
    /// for significantly faster searches (10-100x improvement on large directories).
    /// Falls back to a built-in walkdir implementation if ripgrep is not available.
    pub fn search_files(
        &self,
        path: &Path,
        pattern: &str,
        exclude_patterns: &[String],
    ) -> Result<Vec<PathBuf>> {
        let valid_path = self.config.validate_path(path)?;
        
        // Try to use ripgrep if available
        if let Ok(results) = self.search_with_ripgrep(&valid_path, pattern, exclude_patterns) {
            return Ok(results);
        }
        
        // Fall back to walkdir implementation
        self.search_with_walkdir(&valid_path, pattern, exclude_patterns)
    }
    
    /// Search using ripgrep (fast external tool)
    fn search_with_ripgrep(
        &self,
        path: &Path,
        pattern: &str,
        exclude_patterns: &[String],
    ) -> Result<Vec<PathBuf>> {
        // Check if ripgrep is available
        if !is_ripgrep_available() {
            return Err(anyhow!("ripgrep not available"));
        }
        
        let mut cmd = Command::new("rg");
        cmd.arg("--files")
            .arg("--hidden")
            .arg("--no-ignore-vcs")
            .arg("--no-messages")
            .arg(path);
        
        // Add exclude patterns
        for exclude in exclude_patterns {
            // If the pattern looks like a directory name (no glob chars), 
            // convert it to exclude the directory and its contents
            if !exclude.contains('*') && !exclude.contains('?') && !exclude.contains('[') {
                // Exclude both the directory itself and its contents
                cmd.arg("--glob").arg(format!("!{}", exclude));
                cmd.arg("--glob").arg(format!("!{}/**", exclude));
            } else {
                cmd.arg("--glob").arg(format!("!{}", exclude));
            }
        }
        
        let output = cmd.output()
            .context("Failed to execute ripgrep")?;
        
        if !output.status.success() {
            return Err(anyhow!("ripgrep command failed"));
        }
        
        let pattern_lower = pattern.to_lowercase();
        let mut results = Vec::new();
        
        // Parse output and filter by pattern
        let output_str = String::from_utf8_lossy(&output.stdout);
        for line in output_str.lines() {
            let path_buf = PathBuf::from(line);
            
            // Validate path is within allowed directories
            if self.config.validate_path(&path_buf).is_err() {
                continue;
            }
            
            // Check if filename matches pattern
            if let Some(file_name) = path_buf.file_name() {
                if file_name.to_string_lossy().to_lowercase().contains(&pattern_lower) {
                    results.push(path_buf);
                }
            }
        }
        
        Ok(results)
    }
    
    /// Search using walkdir (fallback implementation)
    fn search_with_walkdir(
        &self,
        path: &Path,
        pattern: &str,
        exclude_patterns: &[String],
    ) -> Result<Vec<PathBuf>> {
        let pattern_lower = pattern.to_lowercase();
        let mut results = Vec::new();
        
        for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
            let entry_path = entry.path();
            
            // Validate each path
            if self.config.validate_path(entry_path).is_err() {
                continue;
            }
            
            // Check exclude patterns
            let relative_path = entry_path.strip_prefix(path).unwrap_or(entry_path);
            let should_exclude = exclude_patterns.iter().any(|pattern| {
                // If the pattern looks like a directory name (no glob chars),
                // check if the path contains that directory
                if !pattern.contains('*') && !pattern.contains('?') && !pattern.contains('[') {
                    // Check if any component of the path matches the pattern
                    relative_path.components().any(|component| {
                        if let std::path::Component::Normal(name) = component {
                            name.to_string_lossy() == *pattern
                        } else {
                            false
                        }
                    })
                } else {
                    // Use glob pattern matching for patterns with wildcards
                    glob::Pattern::new(pattern)
                        .ok()
                        .map(|p| p.matches_path(relative_path))
                        .unwrap_or(false)
                }
            });
            
            if should_exclude {
                continue;
            }
            
            // Check if filename matches pattern
            if let Some(file_name) = entry_path.file_name() {
                if file_name.to_string_lossy().to_lowercase().contains(&pattern_lower) {
                    results.push(entry_path.to_path_buf());
                }
            }
        }
        
        Ok(results)
    }
    
    /// Get file information
    pub async fn get_file_info(&self, path: &Path) -> Result<FileInfo> {
        let valid_path = self.config.validate_path(path)?;
        let metadata = async_fs::metadata(&valid_path)
            .await
            .with_context(|| format!("Failed to get metadata for: {:?}", valid_path))?;
        FileInfo::from_metadata(&metadata)
    }
    
    /// List allowed directories
    pub fn list_allowed_directories(&self) -> Vec<&Path> {
        self.config.allowed_directories.iter().map(|p| p.as_path()).collect()
    }
}

/// Apply a single edit operation to content
fn apply_edit(content: &str, edit: &EditOperation) -> Result<String> {
    let normalized_content = normalize_line_endings(content);
    let normalized_old = normalize_line_endings(&edit.old_text);
    let normalized_new = normalize_line_endings(&edit.new_text);
    
    // Try direct replacement first
    if normalized_content.contains(&normalized_old) {
        return Ok(normalized_content.replace(&normalized_old, &normalized_new));
    }
    
    // Try line-by-line matching with whitespace flexibility
    let old_lines: Vec<&str> = normalized_old.lines().collect();
    let mut content_lines: Vec<String> = normalized_content.lines().map(|s| s.to_string()).collect();
    
    'outer: for i in 0..=content_lines.len().saturating_sub(old_lines.len()) {
        let potential_match = &content_lines[i..i + old_lines.len()];
        
        // Check if lines match (ignoring whitespace differences)
        let is_match = old_lines.iter().zip(potential_match).all(|(old_line, content_line)| {
            old_line.trim() == content_line.trim()
        });
        
        if is_match {
            // Preserve indentation from the original content
            let original_indent = content_lines[i].chars().take_while(|c| c.is_whitespace()).collect::<String>();
            let new_lines: Vec<String> = normalized_new.lines().enumerate().map(|(j, line)| {
                if j == 0 {
                    format!("{}{}", original_indent, line.trim_start())
                } else {
                    // Try to preserve relative indentation
                    line.to_string()
                }
            }).collect();
            
            content_lines.splice(i..i + old_lines.len(), new_lines);
            break 'outer;
        }
    }
    
    Ok(content_lines.join("\n"))
}

/// Normalize line endings to Unix style
fn normalize_line_endings(text: &str) -> String {
    text.replace("\r\n", "\n")
}

/// Create a unified diff between two texts
fn create_unified_diff(original: &str, modified: &str, path: &Path) -> String {
    let normalized_original = normalize_line_endings(original);
    let normalized_modified = normalize_line_endings(modified);
    
    let diff = TextDiff::from_lines(&normalized_original, &normalized_modified);
    let path_str = path.to_string_lossy();
    
    let mut output = String::new();
    output.push_str(&format!("--- {}\n", path_str));
    output.push_str(&format!("+++ {} (modified)\n", path_str));
    
    let mut has_changes = false;
    
    for (idx, group) in diff.grouped_ops(3).iter().enumerate() {
        if idx > 0 {
            output.push_str("\n");
        }
        
        for op in group {
            for change in diff.iter_changes(op) {
                let (sign, _style) = match change.tag() {
                    ChangeTag::Delete => {
                        has_changes = true;
                        ("-", "delete")
                    },
                    ChangeTag::Insert => {
                        has_changes = true;
                        ("+", "insert")
                    },
                    ChangeTag::Equal => (" ", "equal"),
                };
                
                output.push_str(&format!("{}{}", sign, change.value()));
                if !change.value().ends_with('\n') {
                    output.push('\n');
                }
            }
        }
    }
    
    if !has_changes {
        // If no changes, still show the diff header
        output.push_str(" No changes\n");
    }
    
    format!("```diff\n{}```", output)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::{TempDir, NamedTempFile};
    use std::io::Write;
    
    #[tokio::test]
    async fn test_read_write_file() {
        let temp_dir = TempDir::new().unwrap();
        let config = FilesystemConfig::new(vec![temp_dir.path().to_path_buf()]).unwrap();
        let ops = FilesystemOps::new(config);
        
        let test_path = temp_dir.path().join("test.txt");
        let content = "Hello, World!";
        
        ops.write_file(&test_path, content).await.unwrap();
        let read_content = ops.read_file(&test_path).await.unwrap();
        
        assert_eq!(content, read_content);
    }
    
    #[tokio::test]
    async fn test_read_multiple_files() {
        let temp_dir = TempDir::new().unwrap();
        let config = FilesystemConfig::new(vec![temp_dir.path().to_path_buf()]).unwrap();
        let ops = FilesystemOps::new(config);
        
        // Create test files
        let file1 = temp_dir.path().join("file1.txt");
        let file2 = temp_dir.path().join("file2.txt");
        let file3 = temp_dir.path().join("nonexistent.txt");
        
        ops.write_file(&file1, "Content 1").await.unwrap();
        ops.write_file(&file2, "Content 2").await.unwrap();
        
        let paths = vec![file1.clone(), file2.clone(), file3.clone()];
        let results = ops.read_multiple_files(&paths).await;
        
        assert_eq!(results.len(), 3);
        assert!(results[0].is_ok());
        assert_eq!(results[0].as_ref().unwrap().1, "Content 1");
        assert!(results[1].is_ok());
        assert_eq!(results[1].as_ref().unwrap().1, "Content 2");
        assert!(results[2].is_err());
    }
    
    #[tokio::test]
    async fn test_edit_file_basic() {
        let temp_dir = TempDir::new().unwrap();
        let config = FilesystemConfig::new(vec![temp_dir.path().to_path_buf()]).unwrap();
        let ops = FilesystemOps::new(config);
        
        // Create a temporary file
        let test_path = temp_dir.path().join("edit_test.txt");
        let original_content = "Line 1\nLine 2\nLine 3\n";
        ops.write_file(&test_path, original_content).await.unwrap();
        
        // Test basic edit
        let edits = vec![EditOperation {
            old_text: "Line 2".to_string(),
            new_text: "Modified Line 2".to_string(),
        }];
        
        let diff = ops.edit_file(&test_path, &edits, false).await.unwrap();
        
        // Verify the file was edited
        let new_content = ops.read_file(&test_path).await.unwrap();
        assert!(new_content.contains("Modified Line 2"));
        assert!(new_content.contains("Line 1"));
        assert!(new_content.contains("Line 3"));
        
        // Verify diff format
        assert!(diff.contains("```diff"));
        assert!(diff.contains("-Line 2"));
        assert!(diff.contains("+Modified Line 2"));
    }
    
    #[tokio::test]
    async fn test_edit_file_with_tempfile() {
        let temp_dir = TempDir::new().unwrap();
        let config = FilesystemConfig::new(vec![temp_dir.path().to_path_buf()]).unwrap();
        let ops = FilesystemOps::new(config);
        
        // Create a named temporary file
        let mut temp_file = NamedTempFile::new_in(temp_dir.path()).unwrap();
        writeln!(temp_file, "Hello World").unwrap();
        writeln!(temp_file, "This is a test").unwrap();
        writeln!(temp_file, "Goodbye World").unwrap();
        temp_file.flush().unwrap();
        
        let test_path = temp_file.path();
        
        // Test multiple edits
        let edits = vec![
            EditOperation {
                old_text: "Hello World".to_string(),
                new_text: "Greetings Earth".to_string(),
            },
            EditOperation {
                old_text: "Goodbye World".to_string(),
                new_text: "Farewell Earth".to_string(),
            },
        ];
        
        let _diff = ops.edit_file(test_path, &edits, false).await.unwrap();
        
        // Verify the edits
        let new_content = ops.read_file(test_path).await.unwrap();
        assert!(new_content.contains("Greetings Earth"));
        assert!(new_content.contains("This is a test"));
        assert!(new_content.contains("Farewell Earth"));
        assert!(!new_content.contains("Hello World"));
        assert!(!new_content.contains("Goodbye World"));
    }
    
    #[tokio::test]
    async fn test_edit_file_dry_run() {
        let temp_dir = TempDir::new().unwrap();
        let config = FilesystemConfig::new(vec![temp_dir.path().to_path_buf()]).unwrap();
        let ops = FilesystemOps::new(config);
        
        let test_path = temp_dir.path().join("dry_run_test.txt");
        let original_content = "Original content";
        ops.write_file(&test_path, original_content).await.unwrap();
        
        let edits = vec![EditOperation {
            old_text: "Original".to_string(),
            new_text: "Modified".to_string(),
        }];
        
        // Perform dry run
        let diff = ops.edit_file(&test_path, &edits, true).await.unwrap();
        
        // Verify file wasn't modified
        let content = ops.read_file(&test_path).await.unwrap();
        assert_eq!(content, original_content);
        
        // But diff should still be generated
        assert!(diff.contains("-Original"));
        assert!(diff.contains("+Modified"));
    }
    
    #[tokio::test]
    async fn test_edit_file_multiline() {
        let temp_dir = TempDir::new().unwrap();
        let config = FilesystemConfig::new(vec![temp_dir.path().to_path_buf()]).unwrap();
        let ops = FilesystemOps::new(config);
        
        let test_path = temp_dir.path().join("multiline_test.txt");
        let original_content = "function test() {\n    return 42;\n}\n";
        ops.write_file(&test_path, original_content).await.unwrap();
        
        let edits = vec![EditOperation {
            old_text: "function test() {\n    return 42;\n}".to_string(),
            new_text: "function test() {\n    console.log('Testing');\n    return 42;\n}".to_string(),
        }];
        
        ops.edit_file(&test_path, &edits, false).await.unwrap();
        
        let new_content = ops.read_file(&test_path).await.unwrap();
        assert!(new_content.contains("console.log('Testing')"));
    }
    
    #[tokio::test]
    async fn test_edit_file_with_indentation() {
        let temp_dir = TempDir::new().unwrap();
        let config = FilesystemConfig::new(vec![temp_dir.path().to_path_buf()]).unwrap();
        let ops = FilesystemOps::new(config);
        
        let test_path = temp_dir.path().join("indent_test.txt");
        let original_content = "class Test {\n    method() {\n        // TODO: implement\n    }\n}";
        ops.write_file(&test_path, original_content).await.unwrap();
        
        let edits = vec![EditOperation {
            old_text: "        // TODO: implement".to_string(),
            new_text: "        console.log('Implemented!');".to_string(),
        }];
        
        ops.edit_file(&test_path, &edits, false).await.unwrap();
        
        let new_content = ops.read_file(&test_path).await.unwrap();
        assert!(new_content.contains("        console.log('Implemented!')"));
        // Verify indentation is preserved
        assert!(!new_content.contains("console.log('Implemented!')\n    }"));
    }
    
    #[tokio::test]
    async fn test_path_validation() {
        let temp_dir = TempDir::new().unwrap();
        let config = FilesystemConfig::new(vec![temp_dir.path().to_path_buf()]).unwrap();
        let ops = FilesystemOps::new(config);
        
        // Should succeed - within allowed directory
        let valid_path = temp_dir.path().join("allowed.txt");
        assert!(ops.write_file(&valid_path, "test").await.is_ok());
        
        // Should fail - outside allowed directory
        let invalid_path = PathBuf::from("/tmp/not_allowed.txt");
        assert!(ops.write_file(&invalid_path, "test").await.is_err());
    }
    
    #[tokio::test]
    async fn test_create_directory() {
        let temp_dir = TempDir::new().unwrap();
        let config = FilesystemConfig::new(vec![temp_dir.path().to_path_buf()]).unwrap();
        let ops = FilesystemOps::new(config);
        
        let dir_path = temp_dir.path().join("test_dir/nested/deeply");
        ops.create_directory(&dir_path).await.unwrap();
        
        assert!(dir_path.exists());
        assert!(dir_path.is_dir());
        
        // Test creating existing directory (should succeed)
        assert!(ops.create_directory(&dir_path).await.is_ok());
    }
    
    #[tokio::test]
    async fn test_list_directory() {
        let temp_dir = TempDir::new().unwrap();
        let config = FilesystemConfig::new(vec![temp_dir.path().to_path_buf()]).unwrap();
        let ops = FilesystemOps::new(config);
        
        // Create test structure
        ops.write_file(&temp_dir.path().join("file1.txt"), "content").await.unwrap();
        ops.write_file(&temp_dir.path().join("file2.rs"), "content").await.unwrap();
        ops.create_directory(&temp_dir.path().join("subdir")).await.unwrap();
        
        let entries = ops.list_directory(temp_dir.path()).await.unwrap();
        
        assert_eq!(entries.len(), 3);
        // Check sorting
        assert_eq!(entries[0].0, "file1.txt");
        assert_eq!(entries[1].0, "file2.rs");
        assert_eq!(entries[2].0, "subdir");
        // Check types
        assert!(!entries[0].1); // file1.txt is not a directory
        assert!(!entries[1].1); // file2.rs is not a directory
        assert!(entries[2].1);  // subdir is a directory
    }
    
    #[tokio::test]
    async fn test_directory_tree() {
        let temp_dir = TempDir::new().unwrap();
        let config = FilesystemConfig::new(vec![temp_dir.path().to_path_buf()]).unwrap();
        let ops = FilesystemOps::new(config);
        
        // Create test structure
        ops.write_file(&temp_dir.path().join("root.txt"), "content").await.unwrap();
        ops.create_directory(&temp_dir.path().join("dir1")).await.unwrap();
        ops.write_file(&temp_dir.path().join("dir1/file1.txt"), "content").await.unwrap();
        ops.create_directory(&temp_dir.path().join("dir1/subdir")).await.unwrap();
        ops.write_file(&temp_dir.path().join("dir1/subdir/deep.txt"), "content").await.unwrap();
        
        let tree = ops.directory_tree(temp_dir.path()).await.unwrap();
        
        // Verify tree structure
        assert_eq!(tree.len(), 2); // root.txt and dir1
        
        let root_txt = tree.iter().find(|e| e.name == "root.txt").unwrap();
        assert_eq!(root_txt.entry_type, "file");
        assert!(root_txt.children.is_none());
        
        let dir1 = tree.iter().find(|e| e.name == "dir1").unwrap();
        assert_eq!(dir1.entry_type, "directory");
        assert!(dir1.children.is_some());
        
        let dir1_children = dir1.children.as_ref().unwrap();
        assert_eq!(dir1_children.len(), 2); // file1.txt and subdir
    }
    
    #[tokio::test]
    async fn test_move_file() {
        let temp_dir = TempDir::new().unwrap();
        let config = FilesystemConfig::new(vec![temp_dir.path().to_path_buf()]).unwrap();
        let ops = FilesystemOps::new(config);
        
        let source = temp_dir.path().join("source.txt");
        let dest = temp_dir.path().join("destination.txt");
        
        ops.write_file(&source, "test content").await.unwrap();
        ops.move_file(&source, &dest).await.unwrap();
        
        assert!(!source.exists());
        assert!(dest.exists());
        
        let content = ops.read_file(&dest).await.unwrap();
        assert_eq!(content, "test content");
    }
    
    #[tokio::test]
    async fn test_move_file_to_new_directory() {
        let temp_dir = TempDir::new().unwrap();
        let config = FilesystemConfig::new(vec![temp_dir.path().to_path_buf()]).unwrap();
        let ops = FilesystemOps::new(config);
        
        let source = temp_dir.path().join("source.txt");
        let dest = temp_dir.path().join("new_dir/moved.txt");
        
        ops.write_file(&source, "test content").await.unwrap();
        ops.move_file(&source, &dest).await.unwrap();
        
        assert!(!source.exists());
        assert!(dest.exists());
        assert!(dest.parent().unwrap().exists());
    }
    
    #[tokio::test]
    async fn test_move_file_destination_exists() {
        let temp_dir = TempDir::new().unwrap();
        let config = FilesystemConfig::new(vec![temp_dir.path().to_path_buf()]).unwrap();
        let ops = FilesystemOps::new(config);
        
        let source = temp_dir.path().join("source.txt");
        let dest = temp_dir.path().join("dest.txt");
        
        ops.write_file(&source, "source content").await.unwrap();
        ops.write_file(&dest, "dest content").await.unwrap();
        
        let result = ops.move_file(&source, &dest).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("already exists"));
    }
    
    #[tokio::test]
    async fn test_get_file_info() {
        let temp_dir = TempDir::new().unwrap();
        let config = FilesystemConfig::new(vec![temp_dir.path().to_path_buf()]).unwrap();
        let ops = FilesystemOps::new(config);
        
        let test_file = temp_dir.path().join("info_test.txt");
        let content = "Test content for file info";
        ops.write_file(&test_file, content).await.unwrap();
        
        let info = ops.get_file_info(&test_file).await.unwrap();
        
        assert_eq!(info.size, content.len() as u64);
        assert!(info.is_file);
        assert!(!info.is_directory);
        assert!(!info.permissions.is_empty());
        // Modified time should be recent
        let now = Utc::now();
        let diff = now - info.modified;
        assert!(diff.num_seconds() < 60); // Should be within last minute
    }
    
    #[tokio::test]
    async fn test_get_directory_info() {
        let temp_dir = TempDir::new().unwrap();
        let config = FilesystemConfig::new(vec![temp_dir.path().to_path_buf()]).unwrap();
        let ops = FilesystemOps::new(config);
        
        let test_dir = temp_dir.path().join("test_directory");
        ops.create_directory(&test_dir).await.unwrap();
        
        let info = ops.get_file_info(&test_dir).await.unwrap();
        
        assert!(info.is_directory);
        assert!(!info.is_file);
    }
    
    #[test]
    fn test_search_files() {
        let temp_dir = TempDir::new().unwrap();
        let config = FilesystemConfig::new(vec![temp_dir.path().to_path_buf()]).unwrap();
        let ops = FilesystemOps::new(config);
        
        // Create test files
        std::fs::write(temp_dir.path().join("test1.txt"), "content").unwrap();
        std::fs::write(temp_dir.path().join("test2.txt"), "content").unwrap();
        std::fs::write(temp_dir.path().join("other.rs"), "content").unwrap();
        std::fs::create_dir(temp_dir.path().join("subdir")).unwrap();
        std::fs::write(temp_dir.path().join("subdir/test3.txt"), "content").unwrap();
        
        // Search for txt files
        let results = ops.search_files(temp_dir.path(), "txt", &[]).unwrap();
        assert_eq!(results.len(), 3);
        
        // Search with exclusion
        let results = ops.search_files(temp_dir.path(), "txt", &["subdir".to_string()]).unwrap();
        assert_eq!(results.len(), 2);
    }
    
    #[test]
    fn test_search_case_insensitive() {
        let temp_dir = TempDir::new().unwrap();
        let config = FilesystemConfig::new(vec![temp_dir.path().to_path_buf()]).unwrap();
        let ops = FilesystemOps::new(config);
        
        // Create files with different cases but different base names to avoid conflicts
        // on case-insensitive file systems
        std::fs::write(temp_dir.path().join("TEST1.txt"), "content").unwrap();
        std::fs::write(temp_dir.path().join("Test2.TXT"), "content").unwrap();
        std::fs::write(temp_dir.path().join("test3.txt"), "content").unwrap();
        std::fs::write(temp_dir.path().join("other.txt"), "content").unwrap(); // Should not match
        
        // Search should be case insensitive - searching for "test"
        let results = ops.search_files(temp_dir.path(), "test", &[]).unwrap();
        
        // Should find all files with "test" in the name (case-insensitive)
        assert_eq!(results.len(), 3);
        
        // Verify the right files were found
        let filenames: Vec<String> = results
            .iter()
            .map(|p| p.file_name().unwrap().to_string_lossy().to_lowercase())
            .collect();
        
        assert!(filenames.contains(&"test1.txt".to_string()));
        assert!(filenames.contains(&"test2.txt".to_string()));
        assert!(filenames.contains(&"test3.txt".to_string()));
        assert!(!filenames.contains(&"other.txt".to_string()));
    }
    
    #[test]
    fn test_search_with_walkdir() {
        let temp_dir = TempDir::new().unwrap();
        let config = FilesystemConfig::new(vec![temp_dir.path().to_path_buf()]).unwrap();
        let ops = FilesystemOps::new(config);
        
        // Create test files
        std::fs::write(temp_dir.path().join("findme.txt"), "content").unwrap();
        
        // Force using walkdir implementation
        let results = ops.search_with_walkdir(temp_dir.path(), "findme", &[]).unwrap();
        assert_eq!(results.len(), 1);
        assert!(results[0].ends_with("findme.txt"));
    }
    
    #[test]
    fn test_expand_home() {
        // Test tilde expansion
        let home_dir = home::home_dir().unwrap();
        
        let expanded = expand_home(Path::new("~/test")).unwrap();
        assert_eq!(expanded, home_dir.join("test"));
        
        let expanded = expand_home(Path::new("~")).unwrap();
        assert_eq!(expanded, home_dir);
        
        // Non-tilde paths should remain unchanged
        let normal_path = Path::new("/tmp/test");
        let expanded = expand_home(normal_path).unwrap();
        assert_eq!(expanded, normal_path);
    }
    
    #[test]
    fn test_normalize_line_endings() {
        assert_eq!(normalize_line_endings("test\r\nline"), "test\nline");
        assert_eq!(normalize_line_endings("test\nline"), "test\nline");
        assert_eq!(normalize_line_endings("test\r\n\r\nline"), "test\n\nline");
    }
    
    #[test]
    fn test_apply_edit() {
        // Test simple replacement
        let content = "Hello World";
        let edit = EditOperation {
            old_text: "World".to_string(),
            new_text: "Universe".to_string(),
        };
        let result = apply_edit(content, &edit).unwrap();
        assert_eq!(result, "Hello Universe");
        
        // Test multiline replacement
        let content = "Line 1\nLine 2\nLine 3";
        let edit = EditOperation {
            old_text: "Line 2".to_string(),
            new_text: "Modified Line 2".to_string(),
        };
        let result = apply_edit(content, &edit).unwrap();
        assert!(result.contains("Modified Line 2"));
        
        // Test with Windows line endings
        let content = "Line 1\r\nLine 2\r\nLine 3";
        let edit = EditOperation {
            old_text: "Line 2".to_string(),
            new_text: "Modified Line 2".to_string(),
        };
        let result = apply_edit(content, &edit).unwrap();
        assert!(result.contains("Modified Line 2"));
    }
    
    #[test]
    fn test_create_unified_diff() {
        let original = "Line 1\nLine 2\nLine 3";
        let modified = "Line 1\nModified Line 2\nLine 3";
        let path = Path::new("test.txt");
        
        let diff = create_unified_diff(original, modified, path);
        
        assert!(diff.contains("```diff"));
        assert!(diff.contains("--- test.txt"));
        assert!(diff.contains("+++ test.txt (modified)"));
        assert!(diff.contains("-Line 2"));
        assert!(diff.contains("+Modified Line 2"));
    }
    
    #[test]
    fn test_filesystem_config_validation() {
        let temp_dir = TempDir::new().unwrap();
        let config = FilesystemConfig::new(vec![temp_dir.path().to_path_buf()]).unwrap();
        
        // Test valid path
        let valid_path = temp_dir.path().join("test.txt");
        assert!(config.validate_path(&valid_path).is_ok());
        
        // Test path outside allowed directories
        let invalid_path = Path::new("/etc/passwd");
        let result = config.validate_path(invalid_path);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("outside allowed directories"));
        
        // Test non-existent file in allowed directory
        let non_existent = temp_dir.path().join("does_not_exist.txt");
        assert!(config.validate_path(&non_existent).is_ok());
    }
    
    #[test]
    fn test_list_allowed_directories() {
        let temp_dir = TempDir::new().unwrap();
        let temp_dir2 = TempDir::new().unwrap();
        let dirs = vec![temp_dir.path().to_path_buf(), temp_dir2.path().to_path_buf()];
        
        let config = FilesystemConfig::new(dirs.clone()).unwrap();
        let ops = FilesystemOps::new(config);
        
        let allowed = ops.list_allowed_directories();
        assert_eq!(allowed.len(), 2);
    }
    
    #[test]
    fn test_is_ripgrep_available() {
        // This test just ensures the function doesn't panic
        // The actual result depends on whether ripgrep is installed
        let _ = is_ripgrep_available();
    }
}
