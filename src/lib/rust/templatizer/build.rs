//! Build script for templatizer
//!
//! This script embeds the template files at compile time using workspace-embed.
//! Templates are located at src/templates/{puppeteer,spark,superset} in the repo root.

use std::path::PathBuf;
use workspace_embed::EmbedBuilder;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=build.rs");

    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR")?);

    // Navigate to repo root (4 levels up from src/lib/rust/templatizer)
    let repo_root = manifest_dir
        .ancestors()
        .nth(4)
        .expect("Could not find repo root")
        .to_path_buf();

    println!("cargo:warning=Repo root: {}", repo_root.display());

    let result = EmbedBuilder::new()
        .source_root(&repo_root)
        .output_dir(manifest_dir.join("embedded"))
        // Include template directories
        .include_dir("src/templates/puppeteer")
        .include_dir("src/templates/spark")
        .include_dir("src/templates/superset")
        // Respect ignore files
        .respect_gitignore(true)
        // Standard exclusions
        .exclude("*.bak")
        .exclude("*.tmp")
        .exclude("*~")
        .exclude("*.pyc")
        .exclude("__pycache__")
        .exclude("node_modules")
        .exclude("target")
        .exclude(".venv")
        .build()?;

    println!(
        "cargo:warning=Embedded {} files ({} bytes) to {}",
        result.file_count,
        result.total_size,
        result.output_dir.display()
    );

    Ok(())
}
