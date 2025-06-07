use anyhow::Result;
use filesystem_manager::*;
use tempfile::TempDir;

#[tokio::test]
async fn test_edit_file_complex_scenarios() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let config = FilesystemConfig::new(vec![temp_dir.path().to_path_buf()])?;
    let ops = FilesystemOps::new(config);

    // Test 1: Edit with empty lines
    let test_path = temp_dir.path().join("empty_lines.txt");
    let content = "Line 1\n\nLine 3\n\nLine 5";
    ops.write_file(&test_path, content).await?;

    let edits = vec![EditOperation {
        old_text: "Line 3".to_string(),
        new_text: "Modified Line 3".to_string(),
    }];

    ops.edit_file(&test_path, &edits, false).await?;
    let result = ops.read_file(&test_path).await?;
    assert!(result.contains("Modified Line 3"));
    assert!(result.contains("\n\n")); // Empty lines preserved

    // Test 2: Edit with special characters
    let test_path = temp_dir.path().join("special_chars.txt");
    let content = "const regex = /test[\\w]+/g;\nconst str = \"test123\";";
    ops.write_file(&test_path, content).await?;

    let edits = vec![EditOperation {
        old_text: "const regex = /test[\\w]+/g;".to_string(),
        new_text: "const regex = /test[\\d]+/g;".to_string(),
    }];

    ops.edit_file(&test_path, &edits, false).await?;
    let result = ops.read_file(&test_path).await?;
    assert!(result.contains("/test[\\d]+/g"));

    // Test 3: Edit with Unicode
    let test_path = temp_dir.path().join("unicode.txt");
    let content = "Hello 世界\n🌍 Earth\n你好";
    ops.write_file(&test_path, content).await?;

    let edits = vec![EditOperation {
        old_text: "Hello 世界".to_string(),
        new_text: "你好 世界".to_string(),
    }];

    ops.edit_file(&test_path, &edits, false).await?;
    let result = ops.read_file(&test_path).await?;
    assert!(result.contains("你好 世界"));
    assert!(result.contains("🌍 Earth"));

    Ok(())
}

#[tokio::test]
async fn test_edit_file_with_tabs_and_mixed_indentation() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let config = FilesystemConfig::new(vec![temp_dir.path().to_path_buf()])?;
    let ops = FilesystemOps::new(config);

    // Test with tabs
    let test_path = temp_dir.path().join("tabs.txt");
    let content = "class Test {\n\tmethod() {\n\t\treturn true;\n\t}\n}";
    ops.write_file(&test_path, content).await?;

    let edits = vec![EditOperation {
        old_text: "\t\treturn true;".to_string(),
        new_text: "\t\tconsole.log('test');\n\t\treturn true;".to_string(),
    }];

    ops.edit_file(&test_path, &edits, false).await?;
    let result = ops.read_file(&test_path).await?;
    assert!(result.contains("\t\tconsole.log('test');"));
    assert!(result.contains("\t\treturn true;"));

    // Test with mixed indentation
    let test_path = temp_dir.path().join("mixed_indent.txt");
    let content = "function test() {\n  if (true) {\n\treturn 1;\n  }\n}";
    ops.write_file(&test_path, content).await?;

    let edits = vec![EditOperation {
        old_text: "\treturn 1;".to_string(),
        new_text: "\tconsole.log('mixed');\n\treturn 1;".to_string(),
    }];

    ops.edit_file(&test_path, &edits, false).await?;
    let result = ops.read_file(&test_path).await?;
    assert!(result.contains("\tconsole.log('mixed');"));

    Ok(())
}

#[tokio::test]
async fn test_edit_file_edge_cases() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let config = FilesystemConfig::new(vec![temp_dir.path().to_path_buf()])?;
    let ops = FilesystemOps::new(config);

    // Test 1: Edit at beginning of file
    let test_path = temp_dir.path().join("beginning.txt");
    let content = "First line\nSecond line";
    ops.write_file(&test_path, content).await?;

    let edits = vec![EditOperation {
        old_text: "First line".to_string(),
        new_text: "Modified first line".to_string(),
    }];

    ops.edit_file(&test_path, &edits, false).await?;
    let result = ops.read_file(&test_path).await?;
    assert!(result.starts_with("Modified first line"));

    // Test 2: Edit at end of file
    let test_path = temp_dir.path().join("end.txt");
    let content = "First line\nLast line";
    ops.write_file(&test_path, content).await?;

    let edits = vec![EditOperation {
        old_text: "Last line".to_string(),
        new_text: "Modified last line".to_string(),
    }];

    ops.edit_file(&test_path, &edits, false).await?;
    let result = ops.read_file(&test_path).await?;
    assert!(result.ends_with("Modified last line"));

    // Test 3: Edit entire file content
    let test_path = temp_dir.path().join("entire.txt");
    let content = "Complete content";
    ops.write_file(&test_path, content).await?;

    let edits = vec![EditOperation {
        old_text: "Complete content".to_string(),
        new_text: "Entirely new content".to_string(),
    }];

    ops.edit_file(&test_path, &edits, false).await?;
    let result = ops.read_file(&test_path).await?;
    assert_eq!(result, "Entirely new content");

    // Test 4: Edit non-existent text (should not modify file)
    let test_path = temp_dir.path().join("no_match.txt");
    let content = "Original content";
    ops.write_file(&test_path, content).await?;

    let edits = vec![EditOperation {
        old_text: "Non-existent text".to_string(),
        new_text: "New text".to_string(),
    }];

    ops.edit_file(&test_path, &edits, false).await?;
    let result = ops.read_file(&test_path).await?;
    assert_eq!(result, "Original content");

    Ok(())
}

#[tokio::test]
async fn test_concurrent_operations() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let config = FilesystemConfig::new(vec![temp_dir.path().to_path_buf()])?;
    let ops = FilesystemOps::new(config);

    // Create multiple files concurrently
    let mut handles = vec![];

    for i in 0..10 {
        let temp_dir_path = temp_dir.path().to_path_buf();
        let path = temp_dir.path().join(format!("concurrent_{}.txt", i));
        let content = format!("Content {}", i);
        let handle = tokio::spawn(async move {
            // Create a new ops instance for each concurrent operation
            let config = FilesystemConfig::new(vec![temp_dir_path])?;
            let ops = FilesystemOps::new(config);
            ops.write_file(&path, &content).await
        });
        handles.push(handle);
    }

    // Wait for all writes to complete
    for handle in handles {
        handle.await??;
    }

    // Verify all files exist
    for i in 0..10 {
        let path = temp_dir.path().join(format!("concurrent_{}.txt", i));
        let content = ops.read_file(&path).await?;
        assert_eq!(content, format!("Content {}", i));
    }

    Ok(())
}

#[tokio::test]
async fn test_large_file_operations() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let config = FilesystemConfig::new(vec![temp_dir.path().to_path_buf()])?;
    let ops = FilesystemOps::new(config);

    // Create a large file (1MB)
    let large_content = "a".repeat(1024 * 1024);
    let test_path = temp_dir.path().join("large_file.txt");
    ops.write_file(&test_path, &large_content).await?;

    // Read it back
    let read_content = ops.read_file(&test_path).await?;
    assert_eq!(read_content.len(), 1024 * 1024);

    // Edit a portion of it
    let edits = vec![EditOperation {
        old_text: "a".repeat(100),
        new_text: "b".repeat(100),
    }];

    ops.edit_file(&test_path, &edits, false).await?;
    let edited_content = ops.read_file(&test_path).await?;
    assert!(edited_content.contains(&"b".repeat(100)));

    Ok(())
}

#[tokio::test]
async fn test_symbolic_links_handling() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let config = FilesystemConfig::new(vec![temp_dir.path().to_path_buf()])?;
    let ops = FilesystemOps::new(config);

    // Create a file and a symlink to it
    let original = temp_dir.path().join("original.txt");
    let symlink = temp_dir.path().join("symlink.txt");

    ops.write_file(&original, "Original content").await?;

    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(&original, &symlink)?;

        // Should be able to read through symlink
        let content = ops.read_file(&symlink).await?;
        assert_eq!(content, "Original content");

        // Edit through symlink
        let edits = vec![EditOperation {
            old_text: "Original".to_string(),
            new_text: "Modified".to_string(),
        }];

        ops.edit_file(&symlink, &edits, false).await?;

        // Verify both files show the change
        let original_content = ops.read_file(&original).await?;
        let symlink_content = ops.read_file(&symlink).await?;
        assert_eq!(original_content, "Modified content");
        assert_eq!(symlink_content, "Modified content");
    }

    Ok(())
}

#[tokio::test]
async fn test_permission_handling() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let config = FilesystemConfig::new(vec![temp_dir.path().to_path_buf()])?;
    let ops = FilesystemOps::new(config);

    let test_path = temp_dir.path().join("permissions.txt");
    ops.write_file(&test_path, "Test content").await?;

    // Get initial permissions
    let info = ops.get_file_info(&test_path).await?;
    assert!(!info.permissions.is_empty());

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        // Make file read-only
        let metadata = std::fs::metadata(&test_path)?;
        let mut perms = metadata.permissions();
        perms.set_mode(0o444);
        std::fs::set_permissions(&test_path, perms)?;

        // Try to write (should fail)
        let result = ops.write_file(&test_path, "New content").await;
        assert!(result.is_err());

        // Restore permissions
        let mut perms = std::fs::metadata(&test_path)?.permissions();
        perms.set_mode(0o644);
        std::fs::set_permissions(&test_path, perms)?;
    }

    Ok(())
}

#[tokio::test]
async fn test_directory_traversal_protection() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let config = FilesystemConfig::new(vec![temp_dir.path().to_path_buf()])?;
    let ops = FilesystemOps::new(config);

    // Test various directory traversal attempts
    let traversal_attempts = vec![
        "../outside.txt",
        "subdir/../../outside.txt",
        "/etc/passwd",
        "~/../../../etc/passwd",
        "./././../outside.txt",
    ];

    for attempt in traversal_attempts {
        let path = temp_dir.path().join(attempt);
        let result = ops.read_file(&path).await;
        assert!(result.is_err(), "Path {} should be rejected", attempt);
    }

    Ok(())
}

#[tokio::test]
async fn test_search_with_complex_patterns() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let config = FilesystemConfig::new(vec![temp_dir.path().to_path_buf()])?;
    let ops = FilesystemOps::new(config);

    // Create a complex directory structure
    let files = vec![
        "project/src/main.rs",
        "project/src/lib.rs",
        "project/src/utils/helper.rs",
        "project/tests/test_main.rs",
        "project/target/debug/main",
        "project/.git/config",
        "project/README.md",
        "project/Cargo.toml",
        "project/node_modules/package/index.js",
    ];

    for file in &files {
        let path = temp_dir.path().join(file);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, "content")?;
    }

    // Test 1: Search for Rust files
    let results = ops.search_files(temp_dir.path(), "rs", &[])?;
    assert_eq!(results.len(), 4);

    // Test 2: Search with exclusions
    let exclusions = vec!["target".to_string(), "node_modules".to_string()];
    let results = ops.search_files(temp_dir.path(), "", &exclusions)?;
    assert!(!results
        .iter()
        .any(|p| p.to_string_lossy().contains("target")));
    assert!(!results
        .iter()
        .any(|p| p.to_string_lossy().contains("node_modules")));

    // Test 3: Search for partial matches
    let results = ops.search_files(temp_dir.path(), "main", &[])?;
    assert_eq!(results.len(), 3); // main.rs, test_main.rs, and target/debug/main

    Ok(())
}

#[tokio::test]
async fn test_error_recovery() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let config = FilesystemConfig::new(vec![temp_dir.path().to_path_buf()])?;
    let ops = FilesystemOps::new(config);

    // Test reading multiple files with some failures
    let paths = vec![
        temp_dir.path().join("exists1.txt"),
        temp_dir.path().join("does_not_exist.txt"),
        temp_dir.path().join("exists2.txt"),
    ];

    ops.write_file(&paths[0], "Content 1").await?;
    ops.write_file(&paths[2], "Content 2").await?;

    let results = ops.read_multiple_files(&paths).await;
    assert!(results[0].is_ok());
    assert!(results[1].is_err());
    assert!(results[2].is_ok());

    Ok(())
}

#[tokio::test]
async fn test_empty_file_handling() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let config = FilesystemConfig::new(vec![temp_dir.path().to_path_buf()])?;
    let ops = FilesystemOps::new(config);

    // Test empty file
    let empty_path = temp_dir.path().join("empty.txt");
    ops.write_file(&empty_path, "").await?;

    let content = ops.read_file(&empty_path).await?;
    assert_eq!(content, "");

    let info = ops.get_file_info(&empty_path).await?;
    assert_eq!(info.size, 0);

    // Test editing empty file
    let edits = vec![EditOperation {
        old_text: "".to_string(),
        new_text: "New content".to_string(),
    }];

    ops.edit_file(&empty_path, &edits, false).await?;
    let content = ops.read_file(&empty_path).await?;
    assert_eq!(content, "New content");

    Ok(())
}
