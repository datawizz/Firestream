use anyhow::Result;
use tempfile::TempDir;

#[tokio::main]
async fn main() -> Result<()> {
    // Test the validate_path fix
    let temp_dir = TempDir::new()?;
    println!("Temp dir: {:?}", temp_dir.path());

    // Create a filesystem config
    let config = filesystem_manager::FilesystemConfig::new(vec![temp_dir.path().to_path_buf()])?;

    // Test validating a non-existent file path
    let test_file = temp_dir.path().join("test.txt");
    println!("Test file path: {:?}", test_file);

    let validated = config.validate_path(&test_file)?;
    println!("Validated path: {:?}", validated);

    // Check that the validated path ends with "test.txt"
    if validated.file_name().unwrap() == "test.txt" {
        println!("✓ SUCCESS: validate_path correctly preserved the filename!");
    } else {
        println!("✗ FAIL: validate_path returned {:?}", validated);
    }

    // Now test with the FilesystemOps
    let ops = filesystem_manager::FilesystemOps::new(config);

    // Try to write a file
    match ops.write_file(&test_file, "Hello, World!").await {
        Ok(_) => println!("✓ SUCCESS: File written successfully!"),
        Err(e) => println!("✗ FAIL: Error writing file: {}", e),
    }

    // Read it back
    match ops.read_file(&test_file).await {
        Ok(content) => println!("✓ SUCCESS: Read content: {}", content),
        Err(e) => println!("✗ FAIL: Error reading file: {}", e),
    }

    Ok(())
}
