# Test Coverage Documentation

This document describes the comprehensive test coverage for the `filesystem-tool` crate.

## Test Organization

The tests are organized in three locations:

1. **Unit Tests** (`src/lib.rs` - `mod tests`)
   - Basic functionality tests
   - Core operation tests
   - Error handling tests

2. **Integration Tests** (`tests/integration_tests.rs`)
   - Complex scenarios
   - Edge cases
   - Concurrent operations
   - Large file handling
   - Permission handling
   - Security tests

3. **Tempfile Tests** (`tests/tempfile_tests.rs`)
   - Comprehensive tempfile operations
   - Tempfile persistence
   - Concurrent tempfile edits
   - Tempfile cleanup behavior

## Coverage Areas

### File Operations
- ✅ Read single file
- ✅ Read multiple files (with error handling)
- ✅ Write file (with parent directory creation)
- ✅ Move/rename files
- ✅ File info retrieval

### Edit Operations (Special Focus)
- ✅ Basic single-line edits
- ✅ Multi-line edits
- ✅ Edits with special characters
- ✅ Unicode support
- ✅ Tab and mixed indentation handling
- ✅ Edits at file beginning/end
- ✅ Entire file replacement
- ✅ Non-matching text handling
- ✅ Dry run mode
- ✅ Windows line ending normalization
- ✅ Diff generation

### Tempfile Operations
- ✅ Basic tempfile editing
- ✅ Tempfile persistence with `keep()`
- ✅ Complex multi-edit operations
- ✅ Seek and append operations
- ✅ Concurrent tempfile editing
- ✅ Tempfile move operations
- ✅ Cleanup behavior testing
- ✅ Error handling with tempfiles

### Directory Operations
- ✅ Create directory (with nested paths)
- ✅ List directory contents
- ✅ Directory tree generation
- ✅ Directory info retrieval

### Search Operations
- ✅ Basic file search
- ✅ Case-insensitive search
- ✅ Search with exclusions
- ✅ Ripgrep integration (when available)
- ✅ Walkdir fallback
- ✅ Complex pattern matching

### Security & Validation
- ✅ Path validation within allowed directories
- ✅ Directory traversal protection
- ✅ Symbolic link handling
- ✅ Permission checking

### Edge Cases & Error Handling
- ✅ Empty file handling
- ✅ Large file operations (1MB+)
- ✅ Non-existent file handling
- ✅ Concurrent operations
- ✅ Mixed success/failure scenarios
- ✅ Invalid path rejection

## Running the Tests

To run all tests with full coverage:

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test suites
cargo test --test integration_tests
cargo test --test tempfile_tests

# Run with coverage (requires cargo-tarpaulin)
cargo tarpaulin --out Html
```

## Key Test Insights

1. **Edit Operations**: The edit functionality handles various edge cases including:
   - Preserving indentation
   - Handling different line endings
   - Supporting Unicode
   - Generating proper diffs

2. **Tempfile Safety**: Tempfiles are properly handled with:
   - Automatic cleanup
   - Persistence options
   - Concurrent access safety

3. **Security**: The crate properly validates paths and prevents:
   - Directory traversal attacks
   - Access outside allowed directories
   - Unauthorized file access

4. **Performance**: The crate uses ripgrep when available for fast searches, falling back to walkdir when necessary.
