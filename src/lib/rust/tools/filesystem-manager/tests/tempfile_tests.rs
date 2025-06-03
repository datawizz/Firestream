use anyhow::Result;
use filesystem_manager::*;
use std::io::{Seek, SeekFrom, Write};
use tempfile::{NamedTempFile, TempDir};

#[tokio::test]
async fn test_tempfile_basic_operations() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let config = FilesystemConfig::new(vec![temp_dir.path().to_path_buf()])?;
    let ops = FilesystemOps::new(config);

    // Create a named tempfile
    let temp_file = NamedTempFile::new_in(temp_dir.path())?;
    let temp_path = temp_file.path().to_path_buf();

    // Write initial content
    ops.write_file(&temp_path, "Initial content").await?;

    // Edit the tempfile
    let edits = vec![EditOperation {
        old_text: "Initial".to_string(),
        new_text: "Updated".to_string(),
    }];

    ops.edit_file(&temp_path, &edits, false).await?;

    // Verify the edit
    let content = ops.read_file(&temp_path).await?;
    assert_eq!(content, "Updated content");

    // The file should still exist after our operations
    assert!(temp_path.exists());

    Ok(())
}

#[tokio::test]
async fn test_tempfile_persistence() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let config = FilesystemConfig::new(vec![temp_dir.path().to_path_buf()])?;
    let ops = FilesystemOps::new(config);

    let persisted_path = {
        // Create tempfile in a scope
        let temp_file = NamedTempFile::new_in(temp_dir.path())?;
        let temp_path = temp_file.path().to_path_buf();

        ops.write_file(&temp_path, "Temporary content").await?;

        // Persist the tempfile
        let (_file, path) = temp_file.keep()?;
        path
    };

    // File should still exist after tempfile is dropped
    assert!(persisted_path.exists());

    let content = ops.read_file(&persisted_path).await?;
    assert_eq!(content, "Temporary content");

    Ok(())
}

#[tokio::test]
async fn test_tempfile_complex_edits() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let config = FilesystemConfig::new(vec![temp_dir.path().to_path_buf()])?;
    let ops = FilesystemOps::new(config);

    let mut temp_file = NamedTempFile::new_in(temp_dir.path())?;

    // Write structured content
    writeln!(temp_file, "# Configuration File")?;
    writeln!(temp_file, "")?;
    writeln!(temp_file, "[section1]")?;
    writeln!(temp_file, "key1 = value1")?;
    writeln!(temp_file, "key2 = value2")?;
    writeln!(temp_file, "")?;
    writeln!(temp_file, "[section2]")?;
    writeln!(temp_file, "key3 = value3")?;
    temp_file.flush()?;

    let temp_path = temp_file.path();

    // Perform multiple edits
    let edits = vec![
        EditOperation {
            old_text: "# Configuration File".to_string(),
            new_text: "# Updated Configuration File".to_string(),
        },
        EditOperation {
            old_text: "key1 = value1".to_string(),
            new_text: "key1 = updated_value1".to_string(),
        },
        EditOperation {
            old_text: "[section2]".to_string(),
            new_text: "[updated_section2]".to_string(),
        },
    ];

    let diff = ops.edit_file(temp_path, &edits, false).await?;

    // Verify all edits were applied
    let content = ops.read_file(temp_path).await?;
    assert!(content.contains("# Updated Configuration File"));
    assert!(content.contains("key1 = updated_value1"));
    assert!(content.contains("[updated_section2]"));
    assert!(!content.contains("key1 = value1"));

    // Check diff contains all changes
    assert!(diff.contains("-# Configuration File"));
    assert!(diff.contains("+# Updated Configuration File"));
    assert!(diff.contains("-key1 = value1"));
    assert!(diff.contains("+key1 = updated_value1"));

    Ok(())
}

#[tokio::test]
async fn test_tempfile_with_seek_operations() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let config = FilesystemConfig::new(vec![temp_dir.path().to_path_buf()])?;
    let ops = FilesystemOps::new(config);

    let mut temp_file = NamedTempFile::new_in(temp_dir.path())?;
    let temp_path = temp_file.path().to_path_buf();

    // Write initial content
    temp_file.write_all(b"AAAA\nBBBB\nCCCC\n")?;
    temp_file.flush()?;

    // Seek and append
    temp_file.seek(SeekFrom::End(0))?;
    temp_file.write_all(b"DDDD\n")?;
    temp_file.flush()?;

    // Edit the file
    let edits = vec![EditOperation {
        old_text: "BBBB".to_string(),
        new_text: "XXXX".to_string(),
    }];

    ops.edit_file(&temp_path, &edits, false).await?;

    let content = ops.read_file(&temp_path).await?;
    assert!(content.contains("AAAA"));
    assert!(content.contains("XXXX"));
    assert!(content.contains("CCCC"));
    assert!(content.contains("DDDD"));
    assert!(!content.contains("BBBB"));

    Ok(())
}

#[tokio::test]
async fn test_tempfile_binary_safety() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let config = FilesystemConfig::new(vec![temp_dir.path().to_path_buf()])?;
    let ops = FilesystemOps::new(config);

    let temp_file = NamedTempFile::new_in(temp_dir.path())?;
    let temp_path = temp_file.path().to_path_buf();

    // Write text content (not binary)
    ops.write_file(&temp_path, "Text content with special chars: \t\n\r")
        .await?;

    // Edit should handle special characters correctly
    let edits = vec![EditOperation {
        old_text: "Text content".to_string(),
        new_text: "Modified content".to_string(),
    }];

    ops.edit_file(&temp_path, &edits, false).await?;

    let content = ops.read_file(&temp_path).await?;
    assert!(content.contains("Modified content"));
    assert!(content.contains("with special chars: \t\n"));

    Ok(())
}

#[tokio::test]
async fn test_tempfile_concurrent_edits() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let config = FilesystemConfig::new(vec![temp_dir.path().to_path_buf()])?;
    let ops = FilesystemOps::new(config);

    // Create multiple tempfiles
    let mut temp_files = Vec::new();
    for i in 0..5 {
        let mut temp_file = NamedTempFile::new_in(temp_dir.path())?;
        writeln!(temp_file, "File {} content", i)?;
        writeln!(temp_file, "Line 2")?;
        writeln!(temp_file, "Line 3")?;
        temp_file.flush()?;
        temp_files.push(temp_file);
    }

    // Edit all files concurrently
    let mut handles = vec![];

    for (i, temp_file) in temp_files.iter().enumerate() {
        let temp_dir_path = temp_dir.path().to_path_buf();
        let path = temp_file.path().to_path_buf();

        let handle = tokio::spawn(async move {
            // Create a new ops instance for each concurrent operation
            let config = FilesystemConfig::new(vec![temp_dir_path])?;
            let ops = FilesystemOps::new(config);
            let edits = vec![EditOperation {
                old_text: format!("File {} content", i),
                new_text: format!("Updated file {} content", i),
            }];
            ops.edit_file(&path, &edits, false).await
        });

        handles.push(handle);
    }

    // Wait for all edits to complete
    for handle in handles {
        handle.await??;
    }

    // Verify all edits
    for (i, temp_file) in temp_files.iter().enumerate() {
        let content = ops.read_file(temp_file.path()).await?;
        assert!(content.contains(&format!("Updated file {} content", i)));
        assert!(!content.contains(&format!("File {} content", i)));
    }

    Ok(())
}

#[tokio::test]
async fn test_tempfile_move_operations() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let config = FilesystemConfig::new(vec![temp_dir.path().to_path_buf()])?;
    let ops = FilesystemOps::new(config);

    let temp_file = NamedTempFile::new_in(temp_dir.path())?;
    let temp_path = temp_file.path().to_path_buf();

    ops.write_file(&temp_path, "Content to move").await?;

    // Move tempfile to a permanent location
    let new_path = temp_dir.path().join("permanent.txt");
    ops.move_file(&temp_path, &new_path).await?;

    // Original tempfile path should not exist
    assert!(!temp_path.exists());

    // New path should exist with content
    assert!(new_path.exists());
    let content = ops.read_file(&new_path).await?;
    assert_eq!(content, "Content to move");

    Ok(())
}

#[tokio::test]
async fn test_tempfile_cleanup_behavior() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let config = FilesystemConfig::new(vec![temp_dir.path().to_path_buf()])?;
    let ops = FilesystemOps::new(config);

    let temp_path = {
        let temp_file = NamedTempFile::new_in(temp_dir.path())?;
        let path = temp_file.path().to_path_buf();

        ops.write_file(&path, "Temporary data").await?;

        // Edit the file
        let edits = vec![EditOperation {
            old_text: "Temporary".to_string(),
            new_text: "Modified".to_string(),
        }];

        ops.edit_file(&path, &edits, false).await?;

        path
        // temp_file is dropped here
    };

    // By default, NamedTempFile deletes the file when dropped
    assert!(!temp_path.exists());

    Ok(())
}

#[tokio::test]
async fn test_tempfile_with_directory_operations() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let config = FilesystemConfig::new(vec![temp_dir.path().to_path_buf()])?;
    let ops = FilesystemOps::new(config);

    // Create a subdirectory for tempfiles
    let sub_dir = temp_dir.path().join("tempfiles");
    ops.create_directory(&sub_dir).await?;

    // Create multiple tempfiles in subdirectory
    let mut files = Vec::new();
    for i in 0..3 {
        let temp_file = NamedTempFile::new_in(&sub_dir)?;
        ops.write_file(temp_file.path(), &format!("Temp content {}", i))
            .await?;
        files.push(temp_file);
    }

    // List directory should show all tempfiles
    let entries = ops.list_directory(&sub_dir).await?;
    assert_eq!(entries.len(), 3);

    // Search should find tempfiles
    let search_results = ops.search_files(&sub_dir, "tmp", &[])?;
    assert_eq!(search_results.len(), 3);

    Ok(())
}

#[tokio::test]
async fn test_tempfile_error_handling() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let config = FilesystemConfig::new(vec![temp_dir.path().to_path_buf()])?;
    let ops = FilesystemOps::new(config);

    let temp_file = NamedTempFile::new_in(temp_dir.path())?;
    let temp_path = temp_file.path().to_path_buf();

    ops.write_file(&temp_path, "Original content").await?;

    // Test editing with non-matching text
    let edits = vec![EditOperation {
        old_text: "This text doesn't exist".to_string(),
        new_text: "New text".to_string(),
    }];

    // This should succeed but not modify the file
    let _diff = ops.edit_file(&temp_path, &edits, false).await?;
    let content = ops.read_file(&temp_path).await?;
    assert_eq!(content, "Original content");

    // Test dry run on tempfile
    let edits = vec![EditOperation {
        old_text: "Original".to_string(),
        new_text: "Modified".to_string(),
    }];

    let diff = ops.edit_file(&temp_path, &edits, true).await?;
    let content = ops.read_file(&temp_path).await?;
    assert_eq!(content, "Original content"); // Should not be modified
    assert!(diff.contains("-Original"));
    assert!(diff.contains("+Modified"));

    Ok(())
}
