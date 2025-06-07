# Filesystem Tool - Rust Implementation

A secure filesystem operations library written in Rust, providing a clean functional interface for common file and directory operations with built-in security through allowed directory restrictions.

## Features

This Rust implementation provides all the functionality of the original Node.js MCP filesystem server:

- **Secure Path Validation**: All operations are restricted to explicitly allowed directories
- **Async Operations**: Built on Tokio for high-performance async I/O
- **Home Directory Expansion**: Supports `~` expansion in paths
- **Comprehensive Error Handling**: Uses `anyhow` for rich error context
- **Performance Optimizations**:
  - Automatically uses `ripgrep` for file searches when available (10-100x faster)
  - Falls back to built-in implementation if `ripgrep` is not installed
- **File Operations**:
  - Read single or multiple files
  - Write files with automatic parent directory creation
  - Edit files with diff preview and line-based matching
  - Move/rename files and directories
- **Directory Operations**:
  - Create directories (recursive)
  - List directory contents
  - Generate directory trees as JSON
  - Search files with pattern matching and exclusions
- **File Information**: Get detailed metadata including size, timestamps, and permissions
- **Diff Generation**: Create unified diffs for file edits

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
filesystem-tool = { path = "path/to/filesystem-tool" }
```

### Optional: Install ripgrep for faster searches

The library automatically uses `ripgrep` if available for significantly faster file searches:

```bash
# macOS
brew install ripgrep

# Ubuntu/Debian
sudo apt-get install ripgrep

# Arch Linux
sudo pacman -S ripgrep

# Windows
choco install ripgrep
# or
scoop install ripgrep

# From source
cargo install ripgrep
```

## Usage

```rust
use filesystem_manager::{FilesystemConfig, FilesystemOps, EditOperation};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Configure allowed directories
    let config = FilesystemConfig::new(vec![
        PathBuf::from("/home/user/documents"),
        PathBuf::from("~/projects"),  // Supports ~ expansion
    ])?;

    let fs_ops = FilesystemOps::new(config);

    // Read a file
    let content = fs_ops.read_file(&PathBuf::from("~/projects/README.md")).await?;

    // Write a file
    fs_ops.write_file(
        &PathBuf::from("~/projects/output.txt"),
        "Hello, World!"
    ).await?;

    // Edit a file with preview
    let edits = vec![
        EditOperation {
            old_text: "old text".to_string(),
            new_text: "new text".to_string(),
        },
    ];

    // Preview changes (dry run)
    let diff = fs_ops.edit_file(&path, &edits, true).await?;
    println!("Preview:\n{}", diff);

    // Apply changes
    fs_ops.edit_file(&path, &edits, false).await?;

    Ok(())
}
```

## API Reference

### Core Types

#### `FilesystemConfig`
Configuration for allowed directories. All paths are canonicalized during initialization.

```rust
let config = FilesystemConfig::new(vec![PathBuf::from("/allowed/path")])?;
```

#### `FilesystemOps`
Main struct providing all filesystem operations.

```rust
let fs_ops = FilesystemOps::new(config);
```

#### `FileInfo`
Detailed file metadata:
```rust
pub struct FileInfo {
    pub size: u64,
    pub created: Option<DateTime<Utc>>,
    pub modified: DateTime<Utc>,
    pub accessed: DateTime<Utc>,
    pub is_directory: bool,
    pub is_file: bool,
    pub permissions: String,  // Octal format, e.g., "755"
}
```

#### `EditOperation`
Represents a text replacement operation:
```rust
pub struct EditOperation {
    pub old_text: String,
    pub new_text: String,
}
```

#### `TreeEntry`
Directory tree node:
```rust
pub struct TreeEntry {
    pub name: String,
    pub entry_type: String,  // "file" or "directory"
    pub children: Option<Vec<TreeEntry>>,
}
```

### Methods

#### File Operations

- `read_file(path: &Path) -> Result<String>`
  - Read complete file contents as UTF-8

- `read_multiple_files(paths: &[PathBuf]) -> Vec<Result<(PathBuf, String)>>`
  - Read multiple files, returning results for each

- `write_file(path: &Path, content: &str) -> Result<()>`
  - Write content to file, creating parent directories if needed

- `edit_file(path: &Path, edits: &[EditOperation], dry_run: bool) -> Result<String>`
  - Apply edits to a file with smart line matching
  - Returns unified diff showing changes
  - Set `dry_run = true` to preview without applying

#### Directory Operations

- `create_directory(path: &Path) -> Result<()>`
  - Create directory recursively

- `list_directory(path: &Path) -> Result<Vec<(String, bool)>>`
  - List entries with (name, is_directory) tuples

- `directory_tree(path: &Path) -> Result<Vec<TreeEntry>>`
  - Get recursive directory structure as JSON-serializable tree

#### File Management

- `move_file(source: &Path, destination: &Path) -> Result<()>`
  - Move or rename files/directories
  - Fails if destination exists

- `search_files(path: &Path, pattern: &str, exclude_patterns: &[String]) -> Result<Vec<PathBuf>>`
  - Recursively search for files matching pattern
  - Case-insensitive partial name matching
  - Supports glob patterns for exclusions
  - **Performance**: Automatically uses `ripgrep` if available for 10-100x faster searches

- `get_file_info(path: &Path) -> Result<FileInfo>`
  - Get detailed file metadata

#### Utility

- `list_allowed_directories() -> Vec<&Path>`
  - Get list of configured allowed directories

## Differences from Node.js Implementation

### Improvements
1. **Type Safety**: Leverages Rust's type system for compile-time guarantees
2. **Memory Safety**: No null/undefined issues, guaranteed memory safety
3. **Performance**: Zero-cost abstractions and efficient async I/O
4. **Error Handling**: Rich error context with `anyhow`
5. **Testing**: Built-in test framework with `#[tokio::test]`

### Architecture Changes
1. **No MCP Protocol**: This is a pure library implementation without the Model Context Protocol layer
2. **Async by Default**: All I/O operations are async using Tokio
3. **Builder Pattern**: Configuration uses a builder-style approach
4. **Native Types**: Uses Rust's `PathBuf` and standard library types

### Security Features
- Path validation happens at the configuration level
- Symlink resolution with security checks
- All paths are canonicalized to prevent directory traversal
- Explicit allowed directory configuration

## Example: Building an MCP Server

To create an MCP server using this library, you would:

1. Add MCP SDK dependencies
2. Implement the tool handlers using `FilesystemOps`
3. Map the JSON schemas to Rust types
4. Handle the stdio transport

Example structure:
```rust
// In your MCP server implementation
async fn handle_read_file(args: ReadFileArgs) -> Result<String> {
    let fs_ops = /* your configured FilesystemOps */;
    fs_ops.read_file(&PathBuf::from(args.path)).await
}
```

## Testing

The library includes comprehensive test coverage with over 50 tests covering all functionality:

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test suites
cargo test --test integration_tests
cargo test --test tempfile_tests

# Run tests with the enhanced script
./test.sh
```

### Test Coverage

- **Unit Tests** (`src/lib.rs`): Core functionality, error handling, and helper functions
- **Integration Tests** (`tests/integration_tests.rs`): Complex scenarios, edge cases, concurrent operations
- **Tempfile Tests** (`tests/tempfile_tests.rs`): Comprehensive tempfile editing and manipulation

The test suite covers:
- ✅ All public API methods
- ✅ Edge cases and error conditions
- ✅ Security validation (path traversal protection)
- ✅ Concurrent operations
- ✅ Large file handling
- ✅ Unicode and special character support
- ✅ Tempfile operations with `NamedTempFile`

See `tests/README.md` for detailed coverage documentation.

## System Dependencies

- **Optional**: `ripgrep` - When installed, provides 10-100x faster file searches. The library automatically detects and uses it when available, falling back to a built-in implementation otherwise.

## Building

```bash
# Build the library
cargo build --release

# Run tests
cargo test

# Run examples
cargo run --example demo
cargo run --example search_performance
cargo run --example tempfile_editing  # Comprehensive tempfile editing examples
```

## License

This implementation maintains API compatibility with the original Node.js version while providing Rust's safety and performance benefits.
