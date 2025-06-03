use anyhow::Result;
use filesystem_manager::{EditOperation, FilesystemConfig, FilesystemOps};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize with allowed directories
    let allowed_dirs = vec![PathBuf::from("./test_files"), PathBuf::from("/tmp/fs_demo")];

    let config = FilesystemConfig::new(allowed_dirs)?;
    let fs_ops = FilesystemOps::new(config);

    // Create a test directory
    println!("Creating directory...");
    fs_ops
        .create_directory(&PathBuf::from("./test_files"))
        .await?;

    // Write a file
    println!("Writing file...");
    let test_file = PathBuf::from("./test_files/hello.txt");
    fs_ops
        .write_file(&test_file, "Hello, World!\nThis is a test file.\n")
        .await?;

    // Read the file
    println!("Reading file...");
    let content = fs_ops.read_file(&test_file).await?;
    println!("File content:\n{}", content);

    // Edit the file
    println!("\nEditing file (dry run)...");
    let edits = vec![EditOperation {
        old_text: "Hello, World!".to_string(),
        new_text: "Hello, Rust!".to_string(),
    }];

    let diff = fs_ops.edit_file(&test_file, &edits, true).await?;
    println!("Diff preview:\n{}", diff);

    // Apply the edit
    println!("\nApplying edit...");
    fs_ops.edit_file(&test_file, &edits, false).await?;

    // Read again to confirm
    let new_content = fs_ops.read_file(&test_file).await?;
    println!("Updated content:\n{}", new_content);

    // List directory
    println!("\nListing directory:");
    let entries = fs_ops
        .list_directory(&PathBuf::from("./test_files"))
        .await?;
    for (name, is_dir) in entries {
        let type_str = if is_dir { "[DIR]" } else { "[FILE]" };
        println!("{} {}", type_str, name);
    }

    // Get file info
    println!("\nFile information:");
    let info = fs_ops.get_file_info(&test_file).await?;
    println!("{:#?}", info);

    // Search for files
    println!("\nSearching for .txt files...");
    let search_results = fs_ops.search_files(&PathBuf::from("./test_files"), "txt", &[])?;
    for result in &search_results {
        println!("  Found: {:?}", result);
    }

    // List allowed directories
    println!("\nAllowed directories:");
    for dir in fs_ops.list_allowed_directories() {
        println!("  - {:?}", dir);
    }

    Ok(())
}
