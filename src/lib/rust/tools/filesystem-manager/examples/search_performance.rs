use anyhow::Result;
use filesystem_manager::{FilesystemConfig, FilesystemOps};
use std::path::PathBuf;
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<()> {
    // Test search performance on a larger directory
    let test_path = PathBuf::from(".");  // Current directory

    let config = FilesystemConfig::new(vec![test_path.clone()])?;
    let fs_ops = FilesystemOps::new(config);

    println!("Testing file search performance...\n");

    // Search for Rust files
    let start = Instant::now();
    let results = fs_ops.search_files(&test_path, "rs", &["target".to_string()])?;
    let duration = start.elapsed();

    println!("Found {} Rust files in {:.2?}", results.len(), duration);

    // Check if ripgrep was used
    if std::process::Command::new("rg")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        println!("✅ Using ripgrep for fast searches");
    } else {
        println!("ℹ️  Using built-in search (install ripgrep for faster searches)");
    }

    // Show first few results
    println!("\nFirst 5 results:");
    for (i, path) in results.iter().take(5).enumerate() {
        println!("  {}. {:?}", i + 1, path);
    }

    Ok(())
}
