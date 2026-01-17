//! Build script for nix-container-builder
//!
//! This script embeds the Nix workspace files at compile time using workspace-embed.

use std::path::PathBuf;
use workspace_embed::EmbedBuilder;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=build.rs");

    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR")?);

    // Navigate to repo root (4 levels up from src/lib/rust/nix-container-builder)
    let repo_root = manifest_dir
        .ancestors()
        .nth(4)
        .expect("Could not find repo root")
        .to_path_buf();

    println!("cargo:warning=Repo root: {}", repo_root.display());

    let result = EmbedBuilder::new()
        .source_root(&repo_root)
        .output_dir(manifest_dir.join("embedded"))
        // Include Nix module system
        .include_dir("bin/nix/firestream")
        // Include container definitions
        .include_dir("src/containers/firestream")
        // Include root flake files
        .include_file("flake.nix")
        .include_file("flake.lock")
        // Respect ignore files
        .respect_gitignore(true)
        .respect_dockerignore(true)
        // Include minimal .git for Nix flake resolution
        .git_minimal()
        // Standard exclusions (in addition to gitignore)
        .exclude("*.pyc")
        .exclude("__pycache__")
        .exclude("node_modules")
        .exclude("target")
        .exclude(".venv")
        .exclude("result")
        .exclude("result-*")
        .build()?;

    println!(
        "cargo:warning=Embedded {} files ({} bytes) to {}",
        result.file_count,
        result.total_size,
        result.output_dir.display()
    );

    Ok(())
}
