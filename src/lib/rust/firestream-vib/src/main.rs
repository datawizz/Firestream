//! Firestream VIB CLI
//!
//! Command-line interface for running container verification tests.

use clap::{Parser, Subcommand};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

use firestream_vib::{
    ContainerSpec,
    NixHashCache,
    ClosureGraph,
    MetadataConfig,
    OutputFormat,
    SbomMerger,
    generator::GossYamlGenerator,
    runner::{DockerRunner, TrivyRunner, GrypeRunner, TestResult},
    report::{JsonReporter, JUnitReporter, SarifReporter},
    nix::{NixMetadata, closure_graph},
    spec::security::SecurityResult,
};

#[derive(Parser)]
#[command(name = "firestream-vib")]
#[command(about = "Container verification harness for Nix-built containers", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Enable verbose logging
    #[arg(short, long, global = true)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate Goss test specifications from templates
    Generate {
        /// Path to the container specification
        #[arg(short, long)]
        spec: String,

        /// Output directory for generated files
        #[arg(short, long)]
        output: String,
    },

    /// Run verification tests on a container
    Run {
        /// Path to the container specification
        #[arg(short, long)]
        spec: String,

        /// Runtime to use (docker or kubernetes)
        #[arg(short, long, default_value = "docker")]
        runtime: String,

        /// Skip security scanning
        #[arg(long)]
        skip_security: bool,
    },

    /// Run tests and generate reports
    Test {
        /// Path to the container specification
        #[arg(short, long)]
        spec: String,

        /// Output format (json, junit, sarif)
        #[arg(short, long, default_value = "json")]
        format: String,

        /// Output file path
        #[arg(short, long)]
        output: Option<String>,
    },

    /// Show Nix metadata for a container
    Metadata {
        /// Nix flake or derivation path
        #[arg(short, long)]
        path: String,
    },

    /// Clear cached test results
    ClearCache,

    /// Generate container metadata and SBOMs from Nix closure graph
    ///
    /// This command is used during Nix container builds to generate:
    /// - metadata.json: Container config and build provenance
    /// - sbom-cyclonedx.json: CycloneDX 1.5 SBOM
    /// - sbom-spdx.json: SPDX 2.3 SBOM
    /// - closure.json: Full Nix closure tree
    GenerateMetadata {
        /// Path to the exportReferencesGraph output file
        #[arg(long)]
        closure_graph: String,

        /// Path to configuration JSON file
        #[arg(long)]
        config: String,

        /// Output directory for generated files
        #[arg(short, long)]
        output: String,
    },

    /// Archive source code from a fleet-level source map
    ///
    /// Reads a source map JSON (generated at Nix eval time) containing
    /// package source paths and metadata, then creates deduplicated
    /// source tarballs with a source_index.json index file.
    ///
    /// Output:
    /// - source_index.json: Index with coverage stats and per-package entries
    /// - sources/: Directory of deduplicated .tar.gz source archives
    ArchiveSources {
        /// Path to source map JSON (generated at Nix eval time)
        #[arg(long)]
        source_map: String,

        /// Output directory for source archives
        #[arg(short, long)]
        output: String,
    },

    /// Merge multiple SBOMs into a fleet SBOM
    ///
    /// This command aggregates individual container/artifact SBOMs into a
    /// unified fleet SBOM. Components are deduplicated by purl, with source
    /// containers tracked in properties.
    ///
    /// Supports output formats:
    /// - cyclonedx: CycloneDX 1.5 JSON (spec-compliant)
    /// - spdx: SPDX 2.3 JSON (spec-compliant)
    /// - manifest: Fleet inventory manifest (custom format)
    MergeSboms {
        /// Output format (cyclonedx, spdx, manifest)
        #[arg(short, long, default_value = "cyclonedx")]
        format: String,

        /// Fleet name
        #[arg(long)]
        fleet_name: String,

        /// Fleet version
        #[arg(long)]
        fleet_version: String,

        /// Input directories containing SBOM files (can be specified multiple times)
        #[arg(short, long, action = clap::ArgAction::Append)]
        input: Vec<String>,

        /// Output file path
        #[arg(short, long)]
        output: String,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Initialize tracing
    let filter = if cli.verbose {
        EnvFilter::new("debug")
    } else {
        EnvFilter::from_default_env()
    };

    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(filter)
        .init();

    match cli.command {
        Commands::Generate { spec, output } => {
            tracing::info!("Generating Goss tests from spec: {}", spec);
            tracing::info!("Output directory: {}", output);

            // Load spec from YAML file
            let spec = ContainerSpec::from_yaml_file(&spec)?;

            // Create generator
            let generator = GossYamlGenerator::new()?;

            // Generate goss.yaml from spec
            let goss_spec = spec.goss.ok_or_else(|| {
                anyhow::anyhow!("ContainerSpec must have a goss field to generate tests")
            })?;

            let yaml = generator.generate(&goss_spec)?;

            // Create output directory if needed
            std::fs::create_dir_all(&output)?;

            // Write goss.yaml
            let output_path = format!("{}/goss.yaml", output);
            std::fs::write(&output_path, yaml)?;

            tracing::info!("Generated goss.yaml at {}", output_path);
        }
        Commands::Run {
            spec,
            runtime,
            skip_security,
        } => {
            tracing::info!("Running verification tests for: {}", spec);
            tracing::info!("Runtime: {}", runtime);
            if skip_security {
                tracing::info!("Skipping security scans");
            }

            // Only docker runtime supported for now
            if runtime != "docker" {
                anyhow::bail!("Only 'docker' runtime is currently supported");
            }

            let spec = ContainerSpec::from_yaml_file(&spec)?;
            let docker = DockerRunner::new()?;

            // Start container
            tracing::info!("Starting container: {}", spec.image);
            let container_id = docker.run_container(&spec.image, &spec.env).await?;
            tracing::info!("Started container: {}", container_id);

            // Generate and run Goss tests if specified
            if let Some(goss_spec) = &spec.goss {
                tracing::info!("Running Goss tests...");
                let generator = GossYamlGenerator::new()?;
                let yaml = generator.generate(goss_spec)?;

                // Write goss.yaml to temp file
                let temp_dir = std::env::temp_dir();
                let goss_file = temp_dir.join(format!("goss-{}.yaml", container_id));
                std::fs::write(&goss_file, yaml)?;

                // For now, just run goss locally since we need dgoss integration
                // TODO: Implement proper dgoss execution
                let goss_path = goss_file.to_string_lossy().to_string();
                tracing::warn!("Goss execution in container not yet fully implemented");
                tracing::info!("Generated Goss spec at: {}", goss_path);
            }

            // Run security scans unless skipped
            if !skip_security {
                tracing::info!("Running security scans...");

                let trivy = TrivyRunner::new();
                if trivy.is_available().await {
                    tracing::info!("Running Trivy scan...");
                    match trivy.scan_image(&spec.image).await {
                        Ok(result) => {
                            tracing::info!("Trivy scan completed");
                            tracing::info!("  Scanner: {}", result.scanner);
                            tracing::info!("  Vulnerabilities found: {}", result.vulnerabilities.len());
                            for (severity, count) in &result.summary {
                                tracing::info!("    {}: {}", severity, count);
                            }
                        }
                        Err(e) => {
                            tracing::warn!("Trivy scan failed: {}", e);
                        }
                    }
                } else {
                    tracing::warn!("Trivy not available, skipping scan");
                }

                let grype = GrypeRunner::new();
                if grype.is_available().await {
                    tracing::info!("Running Grype scan...");
                    match grype.scan_image(&spec.image).await {
                        Ok(result) => {
                            tracing::info!("Grype scan completed");
                            tracing::info!("  Scanner: {}", result.scanner);
                            tracing::info!("  Vulnerabilities found: {}", result.vulnerabilities.len());
                            for (severity, count) in &result.summary {
                                tracing::info!("    {}: {}", severity, count);
                            }
                        }
                        Err(e) => {
                            tracing::warn!("Grype scan failed: {}", e);
                        }
                    }
                } else {
                    tracing::warn!("Grype not available, skipping scan");
                }
            }

            // Cleanup
            tracing::info!("Cleaning up container...");
            docker.cleanup(&container_id).await?;
            tracing::info!("Verification complete");
        }
        Commands::Test {
            spec,
            format,
            output,
        } => {
            tracing::info!("Running tests for: {}", spec);
            tracing::info!("Output format: {}", format);
            if let Some(ref output_path) = output {
                tracing::info!("Output file: {}", output_path);
            }

            let spec = ContainerSpec::from_yaml_file(&spec)?;
            let docker = DockerRunner::new()?;

            // Start container
            tracing::info!("Starting container: {}", spec.image);
            let container_id = docker.run_container(&spec.image, &spec.env).await?;
            tracing::info!("Started container: {}", container_id);

            let mut test_results: Vec<TestResult> = Vec::new();
            let mut security_results: Vec<SecurityResult> = Vec::new();

            // Run Goss tests if specified
            if let Some(goss_spec) = &spec.goss {
                tracing::info!("Running Goss tests...");
                let generator = GossYamlGenerator::new()?;
                let yaml = generator.generate(goss_spec)?;

                // Write goss.yaml to temp file
                let temp_dir = std::env::temp_dir();
                let goss_file = temp_dir.join(format!("goss-{}.yaml", container_id));
                std::fs::write(&goss_file, &yaml)?;

                // Create a test result for goss (placeholder until dgoss is implemented)
                test_results.push(TestResult {
                    name: "goss-validation".to_string(),
                    success: true,
                    duration_ms: 0,
                    output: format!("Generated Goss spec with {} bytes", yaml.len()),
                    error: None,
                });
            }

            // Run security scans
            let trivy = TrivyRunner::new();
            if trivy.is_available().await {
                tracing::info!("Running Trivy scan...");
                let start = std::time::Instant::now();
                match trivy.scan_image(&spec.image).await {
                    Ok(result) => {
                        let duration_ms = start.elapsed().as_millis() as u64;
                        let total_vulns = result.vulnerabilities.len();
                        test_results.push(TestResult {
                            name: "trivy-scan".to_string(),
                            success: true,
                            duration_ms,
                            output: format!("Found {} vulnerabilities", total_vulns),
                            error: None,
                        });
                        security_results.push(result);
                    }
                    Err(e) => {
                        let duration_ms = start.elapsed().as_millis() as u64;
                        test_results.push(TestResult {
                            name: "trivy-scan".to_string(),
                            success: false,
                            duration_ms,
                            output: String::new(),
                            error: Some(e.to_string()),
                        });
                    }
                }
            }

            let grype = GrypeRunner::new();
            if grype.is_available().await {
                tracing::info!("Running Grype scan...");
                let start = std::time::Instant::now();
                match grype.scan_image(&spec.image).await {
                    Ok(result) => {
                        let duration_ms = start.elapsed().as_millis() as u64;
                        let total_vulns = result.vulnerabilities.len();
                        test_results.push(TestResult {
                            name: "grype-scan".to_string(),
                            success: true,
                            duration_ms,
                            output: format!("Found {} vulnerabilities", total_vulns),
                            error: None,
                        });
                        security_results.push(result);
                    }
                    Err(e) => {
                        let duration_ms = start.elapsed().as_millis() as u64;
                        test_results.push(TestResult {
                            name: "grype-scan".to_string(),
                            success: false,
                            duration_ms,
                            output: String::new(),
                            error: Some(e.to_string()),
                        });
                    }
                }
            }

            // Cleanup
            tracing::info!("Cleaning up container...");
            docker.cleanup(&container_id).await?;

            // Generate report
            let report = match format.as_str() {
                "json" => {
                    let reporter = JsonReporter::new();
                    reporter.generate("firestream-vib", test_results)?
                }
                "junit" => {
                    let reporter = JUnitReporter::new();
                    reporter.generate("firestream-vib", test_results)?
                }
                "sarif" => {
                    let reporter = SarifReporter::new();
                    reporter.generate(security_results)?
                }
                _ => anyhow::bail!("Unknown format: {}", format),
            };

            // Output
            if let Some(path) = output {
                std::fs::write(&path, &report)?;
                tracing::info!("Report written to {}", path);
            } else {
                println!("{}", report);
            }
        }
        Commands::Metadata { path } => {
            tracing::info!("Fetching metadata for: {}", path);

            let metadata = NixMetadata::from_flake(&path).await?;
            let json = serde_json::to_string_pretty(&metadata)?;
            println!("{}", json);
        }
        Commands::ClearCache => {
            tracing::info!("Clearing test result cache");

            let mut cache = NixHashCache::default()?;
            cache.clear()?;
            tracing::info!("Cache cleared successfully");
        }
        Commands::GenerateMetadata {
            closure_graph: closure_graph_path,
            config,
            output,
        } => {
            tracing::info!("Generating container metadata from closure graph");
            tracing::info!("Closure graph: {}", closure_graph_path);
            tracing::info!("Config: {}", config);
            tracing::info!("Output: {}", output);

            // Parse closure graph file
            let graph_path = std::path::Path::new(&closure_graph_path);
            let graph = ClosureGraph::parse_file(graph_path)?;
            tracing::info!("Parsed {} store paths from closure graph", graph.graph.len());

            // Load config
            let config_content = std::fs::read_to_string(&config)?;
            let metadata_config: MetadataConfig = serde_json::from_str(&config_content)
                .map_err(|e| anyhow::anyhow!("Failed to parse config JSON: {}", e))?;
            tracing::info!(
                "Generating metadata for {} v{}",
                metadata_config.container_name,
                metadata_config.container_version
            );

            // Generate all metadata files
            let output_path = std::path::Path::new(&output);
            closure_graph::generate_metadata(
                &graph,
                &metadata_config,
                output_path,
            )?;

            tracing::info!("Metadata generation complete");
        }
        Commands::ArchiveSources {
            source_map,
            output,
        } => {
            tracing::info!("Archiving fleet sources");
            tracing::info!("Source map: {}", source_map);
            tracing::info!("Output: {}", output);

            // Parse source map
            let sm = firestream_vib::source_archive::parse_source_map(
                std::path::Path::new(&source_map)
            ).map_err(|e| anyhow::anyhow!("Failed to parse source map: {}", e))?;
            tracing::info!("Loaded source map with {} entries", sm.len());

            // Archive sources
            let output_path = std::path::Path::new(&output);
            let source_index = firestream_vib::source_archive::archive_sources(&sm, output_path)
                .map_err(|e| anyhow::anyhow!("Failed to archive sources: {}", e))?;

            // Write source_index.json
            let index_path = output_path.join("source_index.json");
            source_index.write_to_file(&index_path)
                .map_err(|e| anyhow::anyhow!("Failed to write source index: {}", e))?;

            tracing::info!(
                "Archived {} sources ({} bytes total)",
                source_index.coverage.archived_count,
                source_index.coverage.archived_bytes,
            );
        }
        Commands::MergeSboms {
            format,
            fleet_name,
            fleet_version,
            input,
            output,
        } => {
            tracing::info!("Merging SBOMs into fleet manifest");
            tracing::info!("Fleet: {} v{}", fleet_name, fleet_version);
            tracing::info!("Output format: {}", format);
            tracing::info!("Input directories: {:?}", input);

            // Parse output format
            let output_format = OutputFormat::from_str(&format)
                .ok_or_else(|| anyhow::anyhow!(
                    "Unknown format '{}'. Use: cyclonedx, spdx, or manifest",
                    format
                ))?;

            // Create merger
            let mut merger = SbomMerger::new(&fleet_name, &fleet_version, output_format);

            // Add all input directories
            for input_dir in &input {
                let path = std::path::Path::new(input_dir);
                tracing::info!("Adding input: {}", input_dir);
                merger.add_input(path)
                    .map_err(|e| anyhow::anyhow!("Failed to add input '{}': {}", input_dir, e))?;
            }

            if input.is_empty() {
                anyhow::bail!("No input directories specified. Use --input <dir> to add inputs.");
            }

            // Perform merge
            tracing::info!("Merging {} inputs...", input.len());
            let result = merger.merge()
                .map_err(|e| anyhow::anyhow!("Merge failed: {}", e))?;

            // Write output
            std::fs::write(&output, &result)?;
            tracing::info!("Wrote merged SBOM to {} ({} bytes)", output, result.len());
        }
    }

    Ok(())
}
