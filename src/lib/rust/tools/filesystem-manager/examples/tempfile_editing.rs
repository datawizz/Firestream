use anyhow::Result;
use filesystem_manager::{EditOperation, FilesystemConfig, FilesystemOps};
use std::io::Write;
use tempfile::{NamedTempFile, TempDir};

#[tokio::main]
async fn main() -> Result<()> {
    // Create a temporary directory for our operations
    let temp_dir = TempDir::new()?;
    println!("Working in temporary directory: {:?}", temp_dir.path());

    // Configure filesystem operations with the temp directory as allowed
    let config = FilesystemConfig::new(vec![temp_dir.path().to_path_buf()])?;
    let ops = FilesystemOps::new(config);

    // Example 1: Basic tempfile editing
    println!("\n=== Example 1: Basic Tempfile Editing ===");
    {
        let mut temp_file = NamedTempFile::new_in(temp_dir.path())?;
        writeln!(temp_file, "Hello, World!")?;
        writeln!(temp_file, "This is a temporary file.")?;
        writeln!(temp_file, "It will be edited soon.")?;
        temp_file.flush()?;

        println!("Created tempfile at: {:?}", temp_file.path());

        // Read the original content
        let original = ops.read_file(temp_file.path()).await?;
        println!("Original content:\n{}", original);

        // Edit the tempfile
        let edits = vec![
            EditOperation {
                old_text: "Hello, World!".to_string(),
                new_text: "Greetings, Universe!".to_string(),
            },
            EditOperation {
                old_text: "temporary file".to_string(),
                new_text: "tempfile example".to_string(),
            },
        ];

        let diff = ops.edit_file(temp_file.path(), &edits, false).await?;
        println!("\nGenerated diff:\n{}", diff);

        let edited = ops.read_file(temp_file.path()).await?;
        println!("\nEdited content:\n{}", edited);
    }

    // Example 2: Dry run mode
    println!("\n=== Example 2: Dry Run Mode ===");
    {
        let mut temp_file = NamedTempFile::new_in(temp_dir.path())?;
        writeln!(temp_file, "Line 1: Do not modify")?;
        writeln!(temp_file, "Line 2: This will be changed")?;
        writeln!(temp_file, "Line 3: Keep this line")?;
        temp_file.flush()?;

        let edits = vec![EditOperation {
            old_text: "This will be changed".to_string(),
            new_text: "This would be changed (but dry run)".to_string(),
        }];

        // Dry run - generates diff but doesn't modify file
        let diff = ops.edit_file(temp_file.path(), &edits, true).await?;
        println!("Dry run diff:\n{}", diff);

        let content = ops.read_file(temp_file.path()).await?;
        println!("\nFile content (unchanged due to dry run):\n{}", content);
    }

    // Example 3: Persisting a tempfile
    println!("\n=== Example 3: Persisting a Tempfile ===");
    {
        let temp_file = NamedTempFile::new_in(temp_dir.path())?;
        let temp_path = temp_file.path().to_path_buf();

        // Write and edit content
        ops.write_file(&temp_path, "This file will be persisted")
            .await?;

        let edits = vec![EditOperation {
            old_text: "persisted".to_string(),
            new_text: "kept permanently".to_string(),
        }];

        ops.edit_file(&temp_path, &edits, false).await?;

        // Persist the file
        let (_, persisted_path) = temp_file.keep()?;
        println!("File persisted at: {:?}", persisted_path);

        let content = ops.read_file(&persisted_path).await?;
        println!("Persisted content: {}", content);
    }

    // Example 4: Complex multi-line editing
    println!("\n=== Example 4: Complex Multi-line Editing ===");
    {
        let mut temp_file = NamedTempFile::new_in(temp_dir.path())?;
        writeln!(temp_file, "fn main() {{")?;
        writeln!(temp_file, "    println!(\"Hello\");")?;
        writeln!(temp_file, "    // TODO: Add more code")?;
        writeln!(temp_file, "}}")?;
        temp_file.flush()?;

        let edits = vec![EditOperation {
            old_text: "    // TODO: Add more code".to_string(),
            new_text: "    let name = \"Rust\";\n    println!(\"Hello, {}!\", name);".to_string(),
        }];

        let diff = ops.edit_file(temp_file.path(), &edits, false).await?;
        println!("Code diff:\n{}", diff);

        let final_code = ops.read_file(temp_file.path()).await?;
        println!("\nFinal code:\n{}", final_code);
    }

    // Example 5: Error handling
    println!("\n=== Example 5: Error Handling ===");
    {
        let temp_file = NamedTempFile::new_in(temp_dir.path())?;
        ops.write_file(temp_file.path(), "Original text").await?;

        // Try to edit non-existent text
        let edits = vec![EditOperation {
            old_text: "This text doesn't exist".to_string(),
            new_text: "New text".to_string(),
        }];

        ops.edit_file(temp_file.path(), &edits, false).await?;

        let content = ops.read_file(temp_file.path()).await?;
        println!("Content after failed edit (unchanged): {}", content);
    }

    println!("\n=== All examples completed successfully! ===");
    Ok(())
}
