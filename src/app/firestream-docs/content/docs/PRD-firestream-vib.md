---
title: "PRD: firestream-vib"
description: "Product Requirements Document for the Rust-based container verification harness"
---

# Product Requirements Document: firestream-vib

**Rust-Based Container Verification Harness for Nix-Built Containers**

| Field | Value |
|-------|-------|
| Document Version | 1.0.0 |
| Created | 2025-12-21 |
| Status | Approved |
| Author | Firestream Team |
| Crate Name | `firestream-vib` |
| Target Location | `src/lib/rust/tools/firestream-vib/` |

---

## Table of Contents

1. [Executive Summary](#1-executive-summary)
2. [Background & Motivation](#2-background--motivation)
3. [Goals & Non-Goals](#3-goals--non-goals)
4. [Technical Architecture](#4-technical-architecture)
5. [Nix Integration Specification](#5-nix-integration-specification)
6. [Implementation Phases](#6-implementation-phases)
7. [Test Categories & Templates](#7-test-categories--templates)
8. [Security Scanning Integration](#8-security-scanning-integration)
9. [Output Formats & Reporting](#9-output-formats--reporting)
10. [CLI Interface](#10-cli-interface)
11. [Configuration Schema](#11-configuration-schema)
12. [Design Decisions](#12-design-decisions)
13. [Environment Variable Migration](#13-environment-variable-migration)
14. [Appendices](#14-appendices)

---

## 1. Executive Summary

### 1.1 Purpose

`firestream-vib` is a Rust crate that provides automated verification and security scanning for Nix-built container images. It replaces Bitnami's closed-source VIB (Verified Infrastructure for Bitnami) orchestrator while leveraging the same open-source verification tools: Goss, Trivy, and Grype.

### 1.2 Core Value Proposition

- **Nix-Native**: Extracts test configuration directly from `flake.nix` metadata and derivation information
- **Reproducible**: Leverages Nix's content-addressed hashing for deterministic test generation and caching
- **Minimal**: Only includes verification logic with independent justification; avoids porting Bitnami-specific boilerplate
- **Integrated**: All tool dependencies provided via the root `flake.nix`

### 1.3 Scope

This crate handles:
- Extraction of test metadata from Nix flakes
- Generation of Goss YAML test specifications
- Orchestration of container verification (Goss, Trivy, Grype)
- Aggregation and reporting of test results
- Execution via Docker and Kubernetes

This crate does NOT handle:
- Container image building (handled by `nix build`)
- Container registry operations

---

## 2. Background & Motivation

### 2.1 Current State

Firestream is migrating container images from Bitnami's Debian-based builds to pure Nix builds. The containers in `src/containers/firestream/` (airflow, kafka, postgresql, redis, odoo) are being rebuilt using:
- `flake.nix` for reproducible builds
- `uv2nix` for Python dependency resolution
- `dockerTools.buildLayeredImage` for OCI image generation

### 2.2 Problem Statement

Bitnami's VIB system provides robust container verification, but:
1. The orchestration layer is closed-source
2. It relies on Bitnami-specific conventions (stacksmith, hardcoded paths)
3. Configuration is duplicated between Dockerfiles and `vib-verify.json`
4. Excessive boilerplate metadata in Dockerfiles (environment variables, labels) that don't translate to Nix builds

### 2.3 Opportunity

Nix flakes provide:
- **Structured metadata via JSON export** (`nix flake show --json`, `nix derivation show`)
- **Cryptographic hashing** of all inputs and outputs
- **Explicit dependency graphs** that can generate SBOMs
- **Single source of truth** for package versions and configuration

A Rust crate can bridge Nix's metadata capabilities with the open-source verification tools, eliminating the need for Bitnami's closed-source orchestrator.

### 2.4 Design Principle: Minimal Porting

**Critical**: Do NOT blindly port Bitnami conventions. Each piece of metadata or test must have independent justification:

| Include | Exclude |
|---------|---------|
| Binary existence checks (functional requirement) | `BITNAMI_APP_NAME` env var (Bitnami-specific) |
| Version verification (correctness check) | Hardcoded paths like `/opt/bitnami` (convention) |
| Linked library validation (runtime requirement) | `OS_ARCH`, `OS_FLAVOUR` labels (Bitnami metadata) |
| Vulnerability scanning (security requirement) | Stacksmith integration hooks |
| CA certificate validation (TLS requirement) | `com.vmware.*` Docker labels |
| SPDX license compliance (legal requirement) | Bitnami-specific debug flags |

---

## 3. Goals & Non-Goals

### 3.1 Goals

| ID | Goal | Success Criteria |
|----|------|------------------|
| G1 | Extract test configuration from Nix flakes | Parse `nix flake show --json` and custom metadata outputs |
| G2 | Generate Goss YAML from Nix metadata | Produce valid Goss specs without manual duplication |
| G3 | Run Goss tests inside containers | Execute via Docker exec or Kubernetes pods |
| G4 | Integrate Trivy vulnerability scanning | Parse JSON output, apply severity thresholds |
| G5 | Integrate Grype vulnerability scanning | Parse JSON output, apply severity thresholds |
| G6 | Provide CI-friendly output | JUnit XML, SARIF, JSON formats |
| G7 | Cache test results by Nix hash | Skip unchanged containers |
| G8 | Support multi-architecture testing | Test both amd64 and arm64 images |
| G9 | Generate SBOM from Nix closures | Use `nix path-info --json --recursive` |
| G10 | Validate SPDX license compliance | Check for license files in containers |

### 3.2 Non-Goals

| ID | Non-Goal | Rationale |
|----|----------|-----------|
| NG1 | Build container images | `nix build` handles this |
| NG2 | Push to registries | Separate CI step |
| NG3 | Port Bitnami PHP/Apache tests | Not used in Firestream containers |
| NG4 | Support non-Nix containers | Focus on Nix-native workflow |
| NG5 | Interactive test debugging | CLI-first, scriptable interface |
| NG6 | Compatibility with vib-verify.json | Clean break with Nix-native format |

---

## 4. Technical Architecture

### 4.1 Crate Structure

```
src/lib/rust/tools/firestream-vib/
├── Cargo.toml
├── src/
│   ├── lib.rs                    # Public API exports
│   ├── main.rs                   # CLI binary
│   │
│   ├── nix/                      # Nix integration
│   │   ├── mod.rs
│   │   ├── flake.rs              # Parse flake show --json
│   │   ├── derivation.rs         # Parse derivation show
│   │   ├── sbom.rs               # Generate SBOM from closure
│   │   └── metadata.rs           # Custom test metadata extraction
│   │
│   ├── spec/                     # Test specifications
│   │   ├── mod.rs
│   │   ├── container.rs          # ContainerTestSpec struct
│   │   ├── goss.rs               # GossConfig struct
│   │   └── security.rs           # SecurityConfig struct
│   │
│   ├── generator/                # Test generation
│   │   ├── mod.rs
│   │   ├── goss_yaml.rs          # Goss YAML from spec
│   │   └── templates.rs          # Tera template loader
│   │
│   ├── runner/                   # Test execution
│   │   ├── mod.rs
│   │   ├── docker.rs             # Docker-based execution
│   │   ├── kubernetes.rs         # K8s pod-based execution
│   │   ├── goss.rs               # Goss runner
│   │   ├── trivy.rs              # Trivy scanner
│   │   └── grype.rs              # Grype scanner
│   │
│   ├── report/                   # Output formatting
│   │   ├── mod.rs
│   │   ├── json.rs
│   │   ├── junit.rs
│   │   └── sarif.rs
│   │
│   └── cache/                    # Result caching
│       ├── mod.rs
│       └── nix_hash.rs           # Hash-based cache keys
│
├── templates/                    # Goss YAML templates (Tera)
│   ├── check-binaries.yaml.tera
│   ├── check-directories.yaml.tera
│   ├── check-files.yaml.tera
│   ├── check-version.yaml.tera
│   ├── check-linked-libraries.yaml.tera
│   ├── check-symlinks.yaml.tera
│   ├── check-ca-certs.yaml.tera
│   └── check-spdx.yaml.tera
│
└── tests/
    ├── integration/
    └── fixtures/
```

### 4.2 Dependency Graph

```
                    ┌─────────────────────┐
                    │   firestream-vib    │
                    │    (Rust crate)     │
                    └──────────┬──────────┘
                               │
           ┌───────────────────┼───────────────────┐
           │                   │                   │
           ▼                   ▼                   ▼
    ┌─────────────┐     ┌─────────────┐     ┌─────────────┐
    │    Goss     │     │   Trivy     │     │   Grype     │
    │  (Go binary)│     │ (Go binary) │     │ (Go binary) │
    └─────────────┘     └─────────────┘     └─────────────┘
           │                   │                   │
           └───────────────────┼───────────────────┘
                               │
                               ▼
                    ┌─────────────────────┐
                    │   Root flake.nix    │
                    │ (provides all deps) │
                    └─────────────────────┘
```

### 4.3 Rust Dependencies

```toml
[package]
name = "firestream-vib"
version = "0.1.0"
edition = "2021"

[dependencies]
# CLI
clap = { version = "4", features = ["derive", "env"] }

# Async runtime
tokio = { version = "1", features = ["full", "process"] }

# Serialization
serde = { version = "1", features = ["derive"] }
serde_json = "1"
serde_yaml = "0.9"
toml = "0.8"

# Templating (Goss YAML generation)
tera = "1"

# Docker interaction
bollard = "0.18"

# Kubernetes interaction (included in default)
kube = { version = "0.99", features = ["runtime", "client"] }
k8s-openapi = { version = "0.24", features = ["v1_31"] }

# HTTP (for Goss health endpoint mode)
reqwest = { version = "0.12", features = ["json"] }

# Error handling
thiserror = "2"
anyhow = "1"

# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# Path handling
camino = "1"

# Process execution with timeout
wait-timeout = "0.2"

[dev-dependencies]
tempfile = "3"
assert_cmd = "2"
predicates = "3"
```

### 4.4 Root Flake.nix Integration

The project's root `flake.nix` MUST provide:

```nix
{
  # Tool dependencies for firestream-vib
  packages.vib-tools = pkgs.buildEnv {
    name = "firestream-vib-tools";
    paths = [
      pkgs.goss          # Server validation
      pkgs.trivy         # Vulnerability scanner
      pkgs.grype         # Container scanner
      pkgs.docker        # Container runtime
      pkgs.nix           # For flake/derivation introspection
    ];
  };

  # Development shell includes vib-tools
  devShells.default = pkgs.mkShell {
    packages = [
      packages.vib-tools
      # ... other dev dependencies
    ];
  };
}
```

---

## 5. Nix Integration Specification

### 5.1 Metadata Extraction Sources

| Source | Command | Data Available |
|--------|---------|----------------|
| Flake outputs | `nix flake show --json .` | Package names, types, descriptions |
| Flake metadata | `nix flake metadata --json .` | Inputs, locks, revisions |
| Derivation info | `nix derivation show ./result` | Build inputs, outputs, env vars |
| Custom output | `nix build .#testMetadata` | Explicit test configuration |
| Closure SBOM | `nix path-info --json --recursive ./result` | Complete dependency graph |

### 5.2 Direct Nix Invocation

**Decision**: The crate invokes `nix` commands directly rather than expecting pre-built artifacts.

```rust
pub struct NixRunner {
    nix_binary: PathBuf,
}

impl NixRunner {
    /// Build a flake output and return the store path
    pub async fn build(&self, flake_path: &Path, output: &str) -> Result<PathBuf> {
        let output = Command::new(&self.nix_binary)
            .args(["build", "--json", &format!("{}#{}", flake_path.display(), output)])
            .output()
            .await?;

        // Parse JSON output to get store path
        let builds: Vec<BuildOutput> = serde_json::from_slice(&output.stdout)?;
        Ok(builds[0].outputs["out"].clone())
    }

    /// Get flake metadata as JSON
    pub async fn flake_show(&self, flake_path: &Path) -> Result<FlakeOutputs> {
        let output = Command::new(&self.nix_binary)
            .args(["flake", "show", "--json", flake_path.to_str().unwrap()])
            .output()
            .await?;

        serde_json::from_slice(&output.stdout)
    }

    /// Generate SBOM from Nix closure
    pub async fn generate_sbom(&self, store_path: &Path) -> Result<NixSbom> {
        let output = Command::new(&self.nix_binary)
            .args(["path-info", "--json", "--recursive", store_path.to_str().unwrap()])
            .output()
            .await?;

        serde_json::from_slice(&output.stdout)
    }
}
```

### 5.3 Required Test Metadata Output

**Decision**: Container flakes MUST export a `testMetadata` package. The crate will fail with a clear error if this is missing.

```nix
# In src/containers/firestream/airflow/flake.nix
{
  packages.testMetadata = pkgs.writeTextFile {
    name = "test-metadata.json";
    text = builtins.toJSON {
      # Required fields
      name = "firestream-airflow";
      version = airflowVersion;

      # Goss test configuration
      goss = {
        # Binaries that must be in PATH
        binaries = [ "airflow" "python" "pip" ];

        # Directories that must exist
        directories = [
          { path = "/opt/airflow"; }
          { path = "/opt/airflow/dags"; mode = "0755"; }
          { path = "/opt/airflow/logs"; mode = "0755"; }
        ];

        # Version command
        version = {
          command = "airflow";
          args = [ "version" ];
          expected = airflowVersion;
          timeout_ms = 30000;
        };

        # All checks enabled by default
        check_symlinks = true;
        check_linked_libraries = {
          enabled = true;
          root_dir = "/nix/store";
          exclude_patterns = [
            ".*/\\.venv/.*"
            ".*/python.*/test/.*"
          ];
        };
        check_ca_certs = true;
        check_spdx = true;  # License compliance enabled
      };

      # Security scanning configuration
      security = {
        trivy = {
          severity_threshold = "LOW";
          vuln_types = [ "os" "library" ];
          ignore_unfixed = false;
        };
        grype = {
          fail_on_severity = "critical";
          package_types = [ "os" ];
        };
      };

      # Container-specific metadata
      exposed_ports = [ 8080 8125 8793 ];
      user = "1001:1001";
      workdir = "/opt/airflow";
    };
  };
}
```

### 5.4 Derivation Hash for Cache Keys

```rust
/// Extract content hash from Nix store path for cache keying
pub fn extract_store_hash(store_path: &str) -> Option<String> {
    // /nix/store/abc123...-name-version -> abc123...
    store_path
        .strip_prefix("/nix/store/")
        .and_then(|s| s.split('-').next())
        .map(String::from)
}
```

### 5.5 SBOM Generation from Nix Closure

**Decision**: Generate SBOMs from Nix closures using `nix path-info --json --recursive`.

```rust
#[derive(Deserialize)]
pub struct NixPathInfo {
    pub path: String,
    pub nar_hash: String,
    pub nar_size: u64,
    pub references: Vec<String>,
    pub deriver: Option<String>,
}

pub struct NixSbomGenerator;

impl NixSbomGenerator {
    /// Generate SBOM from Nix closure
    pub async fn generate(&self, store_path: &Path) -> Result<Sbom> {
        let output = Command::new("nix")
            .args(["path-info", "--json", "--recursive", store_path.to_str().unwrap()])
            .output()
            .await?;

        let path_infos: HashMap<String, NixPathInfo> = serde_json::from_slice(&output.stdout)?;

        // Convert to standard SBOM format (CycloneDX or SPDX)
        self.to_cyclonedx(path_infos)
    }

    fn to_cyclonedx(&self, paths: HashMap<String, NixPathInfo>) -> Result<Sbom> {
        // Extract package name and version from store path
        // /nix/store/abc123-python-3.12.0 -> python 3.12.0
        let components: Vec<Component> = paths.into_iter()
            .filter_map(|(path, info)| {
                let name_version = path.strip_prefix("/nix/store/")?
                    .split('-').skip(1).collect::<Vec<_>>().join("-");
                // Parse name-version
                Some(Component {
                    name: name_version.clone(),
                    version: extract_version(&name_version),
                    purl: format!("pkg:nix/{}", name_version),
                    hash: info.nar_hash,
                })
            })
            .collect();

        Ok(Sbom { components })
    }
}
```

---

## 6. Implementation Phases

### Phase 1: Foundation (MVP)

**Objective**: Complete verification pipeline with Docker and Kubernetes support.

**Deliverables**:
1. Nix metadata extraction (direct `nix` invocation)
2. All core Goss templates:
   - `check-binaries.yaml.tera`
   - `check-directories.yaml.tera`
   - `check-files.yaml.tera`
   - `check-version.yaml.tera`
   - `check-linked-libraries.yaml.tera`
   - `check-symlinks.yaml.tera`
   - `check-ca-certs.yaml.tera`
   - `check-spdx.yaml.tera`
3. Goss execution via Docker exec (copy binary into container)
4. Goss execution via Kubernetes pod
5. Trivy scanning with JSON output parsing
6. Grype scanning with JSON output parsing
7. SBOM generation from Nix closures
8. JSON report output
9. Library API (`lib.rs`) and CLI binary (`main.rs`)

**Success Criteria**:
- Verify `firestream-airflow` container end-to-end
- Generate passing Goss tests from flake metadata
- Execute tests in both Docker and Kubernetes
- Report vulnerabilities in JSON format
- Fail on missing `testMetadata` output

**Estimated Scope**: Core functionality, ~3500 LOC

---

### Phase 2: CI/CD Integration

**Objective**: Production-ready CI integration.

**Deliverables**:
1. JUnit XML output for test frameworks
2. SARIF output for GitHub/GitLab security
3. Exit code conventions (0=pass, 1=test fail, 2=vuln found)
4. GitHub Actions workflow example
5. Parallel test execution across containers

**GitHub Actions Integration**:
```yaml
- name: Verify containers
  run: |
    nix develop --command firestream-vib verify \
      --containers src/containers/firestream/* \
      --output-format sarif \
      --output-file results.sarif

- name: Upload SARIF
  uses: github/codeql-action/upload-sarif@v3
  with:
    sarif_file: results.sarif
```

---

### Phase 3: Advanced Features

**Objective**: Optimization and extended functionality.

**Deliverables**:
1. Hash-based result caching
2. Differential testing (only changed containers)
3. Multi-architecture test matrix
4. Custom test injection (user-defined Goss YAML)
5. Markdown report generation

**Cache Strategy**:
```rust
pub struct TestCache {
    cache_dir: PathBuf,
}

impl TestCache {
    pub fn get(&self, nix_hash: &str) -> Option<TestResult> {
        let cache_path = self.cache_dir.join(format!("{}.json", nix_hash));
        // Return cached result if exists and valid
    }

    pub fn set(&self, nix_hash: &str, result: &TestResult) {
        // Write result to cache
    }
}
```

---

## 7. Test Categories & Templates

### 7.1 MVP Templates (All Included in Phase 1)

| Template | Purpose | Enabled by Default |
|----------|---------|-------------------|
| `check-binaries.yaml.tera` | Verify binaries in PATH | Yes |
| `check-directories.yaml.tera` | Verify directory structure | Yes |
| `check-files.yaml.tera` | Verify file existence/content | Yes |
| `check-version.yaml.tera` | Validate application version | Yes |
| `check-linked-libraries.yaml.tera` | Verify runtime libraries load | Yes |
| `check-symlinks.yaml.tera` | Detect broken symlinks | Yes |
| `check-ca-certs.yaml.tera` | Validate TLS/CA certificates | Yes |
| `check-spdx.yaml.tera` | Verify SPDX license files | Yes |

### 7.2 Template Design

Templates use [Tera](https://keats.github.io/tera/) syntax (similar to Jinja2/Go templates):

**Example: `check-binaries.yaml.tera`**
```yaml
# Verify binaries are available in PATH
command:
{% for binary in binaries %}
  check-{{ binary }}-binary:
    exec: which {{ binary }}
    exit-status: 0
    timeout: 5000
{% endfor %}
```

**Example: `check-spdx.yaml.tera`**
```yaml
# Verify SPDX license files exist
file:
{% for spdx_path in spdx_paths %}
  {{ spdx_path }}:
    exists: true
    filetype: file
{% endfor %}

command:
  check-spdx-files-exist:
    exec: find {{ root_dir }} -name "*.spdx" -type f | head -1
    exit-status: 0
    stdout:
      - ".spdx"
```

### 7.3 Goss Execution via Docker Exec

**Decision**: Tests run by copying Goss binary into the container and executing via `docker exec`.

```rust
pub struct DockerGossRunner {
    docker: Docker,
    goss_binary: PathBuf,
}

impl DockerGossRunner {
    pub async fn run_tests(
        &self,
        container_id: &str,
        goss_yaml: &str,
    ) -> Result<GossResult> {
        // 1. Copy goss binary into container
        let goss_tar = self.create_goss_tar().await?;
        self.docker.upload_to_container(
            container_id,
            UploadToContainerOptions { path: "/tmp", .. },
            goss_tar,
        ).await?;

        // 2. Copy test YAML into container
        let yaml_tar = self.create_yaml_tar(goss_yaml).await?;
        self.docker.upload_to_container(
            container_id,
            UploadToContainerOptions { path: "/tmp", .. },
            yaml_tar,
        ).await?;

        // 3. Execute goss validate
        let exec = self.docker.create_exec(container_id, CreateExecOptions {
            cmd: Some(vec![
                "/tmp/goss",
                "validate",
                "--gossfile", "/tmp/goss.yaml",
                "--format", "json",
            ]),
            attach_stdout: Some(true),
            attach_stderr: Some(true),
            ..Default::default()
        }).await?;

        let output = self.docker.start_exec(&exec.id, None).await?;
        self.parse_goss_output(output)
    }
}
```

### 7.4 Goss Execution via Kubernetes Pod

```rust
pub struct KubernetesGossRunner {
    client: Client,
    namespace: String,
}

impl KubernetesGossRunner {
    pub async fn run_tests(
        &self,
        image: &str,
        goss_yaml: &str,
    ) -> Result<GossResult> {
        // 1. Create ConfigMap with goss.yaml
        let config_map = self.create_goss_configmap(goss_yaml).await?;

        // 2. Create Pod with goss binary and mounted configmap
        let pod = Pod {
            metadata: ObjectMeta {
                name: Some(format!("goss-test-{}", uuid::Uuid::new_v4())),
                namespace: Some(self.namespace.clone()),
                ..Default::default()
            },
            spec: Some(PodSpec {
                containers: vec![Container {
                    name: "goss".into(),
                    image: Some(image.into()),
                    command: Some(vec!["/goss".into(), "validate".into()]),
                    args: Some(vec![
                        "--gossfile".into(), "/config/goss.yaml".into(),
                        "--format".into(), "json".into(),
                    ]),
                    volume_mounts: Some(vec![VolumeMount {
                        name: "goss-config".into(),
                        mount_path: "/config".into(),
                        ..Default::default()
                    }]),
                    ..Default::default()
                }],
                volumes: Some(vec![Volume {
                    name: "goss-config".into(),
                    config_map: Some(ConfigMapVolumeSource {
                        name: Some(config_map.metadata.name.unwrap()),
                        ..Default::default()
                    }),
                    ..Default::default()
                }]),
                restart_policy: Some("Never".into()),
                ..Default::default()
            }),
            ..Default::default()
        };

        // 3. Wait for completion and get logs
        let pods: Api<Pod> = Api::namespaced(self.client.clone(), &self.namespace);
        pods.create(&PostParams::default(), &pod).await?;

        self.wait_and_get_logs(&pod).await
    }
}
```

---

## 8. Security Scanning Integration

### 8.1 Trivy Integration

**Default Threshold**: `LOW` (report all vulnerabilities)

```rust
pub struct TrivyRunner {
    binary_path: PathBuf,
}

impl TrivyRunner {
    pub async fn scan_image(&self, image: &str, config: &TrivyConfig) -> Result<TrivyReport> {
        let output = Command::new(&self.binary_path)
            .args([
                "image",
                "--format", "json",
                "--severity", &config.severity_threshold,  // Default: LOW
                "--vuln-type", &config.vuln_types.join(","),
            ])
            .arg(image)
            .output()
            .await?;

        if !output.status.success() {
            return Err(anyhow!("Trivy scan failed: {}", String::from_utf8_lossy(&output.stderr)));
        }

        serde_json::from_slice(&output.stdout)
    }
}
```

### 8.2 Grype Integration

**Default Threshold**: `critical` (fail only on critical vulnerabilities)

```rust
pub struct GrypeRunner {
    binary_path: PathBuf,
}

impl GrypeRunner {
    pub async fn scan_sbom(&self, sbom: &Sbom, config: &GrypeConfig) -> Result<GrypeReport> {
        // Write SBOM to temp file
        let sbom_file = tempfile::NamedTempFile::new()?;
        serde_json::to_writer(&sbom_file, sbom)?;

        let output = Command::new(&self.binary_path)
            .args([
                &format!("sbom:{}", sbom_file.path().display()),
                "--output", "json",
                "--fail-on", &config.fail_on_severity,  // Default: critical
            ])
            .output()
            .await?;

        // Note: Grype exits non-zero if vulnerabilities exceed threshold
        // We parse output regardless of exit code
        serde_json::from_slice(&output.stdout)
    }
}
```

### 8.3 Vulnerability Aggregation

```rust
#[derive(Serialize)]
pub struct SecurityReport {
    pub sbom_source: String,  // "nix-closure"
    pub trivy: TrivyReport,
    pub grype: GrypeReport,
    pub unique_vulnerabilities: Vec<Vulnerability>,
    pub summary: SecuritySummary,
}

impl SecurityReport {
    pub fn merge(trivy: TrivyReport, grype: GrypeReport) -> Self {
        // Deduplicate by CVE ID
        let mut seen = HashSet::new();
        let unique: Vec<_> = trivy.vulnerabilities.iter()
            .chain(grype.matches.iter().map(|m| &m.vulnerability))
            .filter(|v| seen.insert(&v.id))
            .cloned()
            .collect();

        Self {
            sbom_source: "nix-closure".into(),
            trivy,
            grype,
            unique_vulnerabilities: unique,
            summary: SecuritySummary::from(&unique),
        }
    }
}
```

---

## 9. Output Formats & Reporting

### 9.1 Supported Formats

| Format | Use Case | Standard |
|--------|----------|----------|
| JSON | Programmatic consumption | Custom schema |
| JUnit XML | CI test frameworks | JUnit 4/5 |
| SARIF | GitHub/GitLab Security | SARIF 2.1.0 |
| Markdown | Human-readable summaries | GitHub Flavored |
| TAP | Test Anything Protocol | TAP 13 |

### 9.2 Exit Codes

| Code | Meaning |
|------|---------|
| 0 | All tests passed, no critical vulnerabilities |
| 1 | Goss tests failed |
| 2 | Vulnerabilities exceed threshold |
| 3 | Both tests failed and vulnerabilities found |
| 10 | Configuration error |
| 11 | Container not found |
| 12 | Tool dependency missing (goss, trivy, grype, nix) |
| 13 | Missing testMetadata output in flake |

---

## 10. CLI Interface

### 10.1 Command Structure

```
firestream-vib <COMMAND>

Commands:
  verify      Run verification on container(s)
  generate    Generate Goss YAML without running tests
  scan        Run security scans only
  cache       Manage test result cache

Options:
  -v, --verbose    Increase verbosity (-v, -vv, -vvv)
  -q, --quiet      Suppress non-error output
  --color <WHEN>   Color output (auto, always, never)
  -h, --help       Print help
  -V, --version    Print version
```

### 10.2 Verify Command

```
firestream-vib verify [OPTIONS] <CONTAINER>...

Arguments:
  <CONTAINER>...  Path(s) to container flake directories

Options:
  --arch <ARCH>           Target architecture (amd64, arm64, or both)
  --runner <RUNNER>       Test runner (docker, kubernetes) [default: docker]
  --namespace <NS>        Kubernetes namespace [default: default]
  --skip-goss             Skip Goss tests
  --skip-security         Skip security scans
  --trivy-severity <SEV>  Trivy severity threshold [default: LOW]
  --grype-fail-on <SEV>   Grype fail-on severity [default: critical]
  --output <FORMAT>       Output format (json, junit, sarif, tap) [default: json]
  --output-file <PATH>    Write output to file instead of stdout
  --cache                 Enable result caching
  --no-cache              Disable result caching
  --parallel <N>          Run N containers in parallel [default: 4]
```

### 10.3 Usage Examples

```bash
# Verify single container with Docker
firestream-vib verify src/containers/firestream/airflow

# Verify with Kubernetes runner
firestream-vib verify src/containers/firestream/airflow \
  --runner kubernetes \
  --namespace firestream-test

# Verify all containers with SARIF output
firestream-vib verify src/containers/firestream/* \
  --output sarif \
  --output-file results.sarif

# Generate Goss YAML for inspection
firestream-vib generate src/containers/firestream/kafka > kafka-tests.yaml

# Security scan only
firestream-vib scan src/containers/firestream/postgresql \
  --trivy-severity HIGH \
  --grype-fail-on high

# Run with caching for CI
firestream-vib verify src/containers/firestream/* \
  --cache \
  --parallel 8 \
  --output junit \
  --output-file test-results.xml
```

---

## 11. Configuration Schema

### 11.1 Container Test Metadata

Full schema for `testMetadata` flake output:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "required": ["name", "version", "goss"],
  "properties": {
    "name": {
      "type": "string",
      "description": "Container name (should use FIRESTREAM_ prefix convention)"
    },
    "version": {
      "type": "string",
      "description": "Application version"
    },
    "goss": {
      "type": "object",
      "properties": {
        "binaries": {
          "type": "array",
          "items": { "type": "string" },
          "description": "Binaries that must be in PATH"
        },
        "directories": {
          "type": "array",
          "items": {
            "type": "object",
            "required": ["path"],
            "properties": {
              "path": { "type": "string" },
              "mode": { "type": "string", "pattern": "^[0-7]{3,4}$" },
              "owner": { "type": "string" },
              "group": { "type": "string" }
            }
          }
        },
        "files": {
          "type": "array",
          "items": {
            "type": "object",
            "required": ["path"],
            "properties": {
              "path": { "type": "string" },
              "exists": { "type": "boolean", "default": true },
              "mode": { "type": "string" },
              "contains": { "type": "array", "items": { "type": "string" } }
            }
          }
        },
        "version": {
          "type": "object",
          "required": ["command", "expected"],
          "properties": {
            "command": { "type": "string" },
            "args": { "type": "array", "items": { "type": "string" } },
            "expected": { "type": "string" },
            "timeout_ms": { "type": "integer", "default": 10000 }
          }
        },
        "check_symlinks": { "type": "boolean", "default": true },
        "check_linked_libraries": {
          "type": "object",
          "properties": {
            "enabled": { "type": "boolean", "default": true },
            "root_dir": { "type": "string" },
            "exclude_patterns": {
              "type": "array",
              "items": { "type": "string" }
            }
          }
        },
        "check_ca_certs": { "type": "boolean", "default": true },
        "check_spdx": { "type": "boolean", "default": true }
      }
    },
    "security": {
      "type": "object",
      "properties": {
        "trivy": {
          "type": "object",
          "properties": {
            "severity_threshold": {
              "type": "string",
              "enum": ["CRITICAL", "HIGH", "MEDIUM", "LOW"],
              "default": "LOW"
            },
            "vuln_types": {
              "type": "array",
              "items": { "type": "string", "enum": ["os", "library"] }
            },
            "ignore_unfixed": { "type": "boolean" }
          }
        },
        "grype": {
          "type": "object",
          "properties": {
            "fail_on_severity": {
              "type": "string",
              "enum": ["critical", "high", "medium", "low", "negligible"],
              "default": "critical"
            },
            "package_types": {
              "type": "array",
              "items": { "type": "string" }
            }
          }
        }
      }
    },
    "exposed_ports": {
      "type": "array",
      "items": { "type": "integer" }
    },
    "user": { "type": "string" },
    "workdir": { "type": "string" }
  }
}
```

### 11.2 Global Configuration

Optional global config file at `.firestream-vib.toml`:

```toml
[defaults]
parallel = 4
cache_enabled = true
cache_dir = ".cache/firestream-vib"
runner = "docker"  # or "kubernetes"

[kubernetes]
namespace = "firestream-test"

[security.trivy]
severity_threshold = "LOW"
vuln_types = ["os", "library"]

[security.grype]
fail_on_severity = "critical"

[output]
format = "json"

# CVEs to ignore globally (e.g., disputed, not applicable)
[ignore]
cves = [
  # "CVE-2024-XXXXX"  # Reason for ignoring
]
```

---

## 12. Design Decisions

This section documents the approved design decisions for `firestream-vib`.

### 12.1 Nix Integration

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Nix command invocation | Direct invocation | Crate runs `nix build`, `nix flake show` directly for self-contained operation |
| Missing testMetadata handling | Fail with error | Require explicit test configuration; prevents silent failures |
| SBOM generation | Nix closure (`nix path-info --json --recursive`) | More precise than container scanning; leverages Nix's dependency graph |

### 12.2 Test Execution

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Goss execution method | Copy binary, docker exec | Most reliable; doesn't require modifying container entrypoint |
| Kubernetes support | Included in MVP | Critical for production verification workflows |
| MVP template set | All core templates | Comprehensive verification from day one |

### 12.3 Security Scanning

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Trivy default threshold | LOW | Report all findings for visibility |
| Grype default threshold | critical | Only fail builds on critical issues |
| SPDX license checks | Enabled by default | Legal compliance is a core requirement |

### 12.4 Crate Design

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Crate visibility | Both lib + binary | Enables programmatic use and CLI usage |
| vib-verify.json compatibility | Clean break | Nix-native format serves different purposes |

---

## 13. Environment Variable Migration

### 13.1 Migration Strategy

When rewriting Bitnami containers to Firestream containers:

1. **Location**: Rewritten containers live in `src/containers/firestream/`
2. **Prefix Change**: All `BITNAMI_*` environment variables become `FIRESTREAM_*`
3. **Selective Migration**: Only migrate operationally relevant variables

### 13.2 Environment Variable Mapping

| Bitnami Variable | Firestream Variable | Migrate? | Rationale |
|-----------------|---------------------|----------|-----------|
| `BITNAMI_APP_NAME` | — | No | Bitnami-specific metadata |
| `BITNAMI_IMAGE_VERSION` | — | No | Use Nix derivation version |
| `BITNAMI_DEBUG` | `FIRESTREAM_DEBUG` | Yes | Operationally useful |
| `OS_ARCH` | — | No | Detected at runtime |
| `OS_FLAVOUR` | — | No | Nix handles OS layer |
| `OS_NAME` | — | No | Nix handles OS layer |
| `AIRFLOW_HOME` | `FIRESTREAM_AIRFLOW_HOME` | Yes | App configuration |
| `AIRFLOW_*` | `FIRESTREAM_AIRFLOW_*` | Selective | Only operational vars |
| `POSTGRESQL_*` | `FIRESTREAM_POSTGRESQL_*` | Selective | Only operational vars |

### 13.3 Exclusion List

The following environment variables are explicitly NOT ported:

```
# Bitnami metadata (replaced by Nix derivation info)
BITNAMI_APP_NAME
BITNAMI_IMAGE_VERSION
BITNAMI_ROOT_DIR
BITNAMI_VOLUME_DIR

# OS-level metadata (handled by Nix)
OS_ARCH
OS_FLAVOUR
OS_NAME

# Debug/internal flags (unless operationally needed)
BITNAMI_DEBUG  # Migrate as FIRESTREAM_DEBUG if needed

# Path overrides (Nix handles these)
PATH  # Set by Nix profile
LD_LIBRARY_PATH  # Set by Nix
PYTHONPATH  # Set by Nix Python environment

# VMware/Tanzu metadata
com.vmware.*
```

### 13.4 Container Naming Convention

```
# Old Bitnami naming
bitnami/airflow:3.0.3-debian-12-r5

# New Firestream naming
firestream/airflow:3.0.3-nix
```

---

## 14. Appendices

### A. Reference: Bitnami VIB Test Types

Original VIB templates and their Firestream equivalents:

| VIB Template | Port? | Firestream Equivalent | Notes |
|--------------|-------|----------------------|-------|
| check-binaries.yaml | Yes | check-binaries.yaml.tera | Direct port |
| check-directories.yaml | Yes | check-directories.yaml.tera | Direct port |
| check-files.yaml | Yes | check-files.yaml.tera | Direct port |
| check-app-version.yaml | Yes | check-version.yaml.tera | Simplified |
| check-linked-libraries.yaml | Yes | check-linked-libraries.yaml.tera | Adapt for Nix store |
| check-broken-symlinks.yaml | Yes | check-symlinks.yaml.tera | Direct port |
| check-ca-certs.yaml | Yes | check-ca-certs.yaml.tera | Direct port |
| check-spdx.yaml | Yes | check-spdx.yaml.tera | Enabled by default |
| check-php-fpm.yaml | No | N/A | PHP not used |
| check-nginx-php-fpm.yaml | No | N/A | PHP not used |
| check-apache-libphp.yaml | No | N/A | Apache/PHP not used |
| check-openssl-fips.yaml | No | N/A | Photon-specific |
| check-static.yaml | No | N/A | Bitnami paths |
| check-libgcc.yaml | No | N/A | Bitnami-specific |
| check-sed-in-place.yaml | No | N/A | Build-time check |

### B. Reference: Nix Command Outputs

**`nix flake show --json`**:
```json
{
  "packages": {
    "x86_64-linux": {
      "default": {
        "name": "firestream-airflow-3.0.3",
        "type": "derivation"
      },
      "dockerImage": {
        "name": "docker-image-firestream-airflow.tar.gz",
        "type": "derivation"
      },
      "testMetadata": {
        "name": "test-metadata.json",
        "type": "derivation"
      }
    }
  }
}
```

**`nix path-info --json --recursive ./result`** (SBOM source):
```json
{
  "/nix/store/abc123-python-3.12.0": {
    "path": "/nix/store/abc123-python-3.12.0",
    "narHash": "sha256:...",
    "narSize": 12345678,
    "references": [
      "/nix/store/def456-glibc-2.39",
      "/nix/store/ghi789-openssl-3.0.13"
    ],
    "deriver": "/nix/store/xyz-python-3.12.0.drv"
  }
}
```

### C. Glossary

| Term | Definition |
|------|------------|
| **Goss** | YAML-based server validation tool (Go) |
| **Trivy** | Comprehensive vulnerability scanner (Go) |
| **Grype** | Container/filesystem vulnerability scanner (Go) |
| **VIB** | Verified Infrastructure for Bitnami (closed source) |
| **SBOM** | Software Bill of Materials |
| **SARIF** | Static Analysis Results Interchange Format |
| **SPDX** | Software Package Data Exchange (license standard) |
| **Derivation** | Nix build recipe describing inputs/outputs |
| **Store Path** | Immutable path in /nix/store with content hash |
| **Flake** | Nix project format with locked dependencies |
| **Closure** | Complete set of dependencies for a Nix derivation |

---

## Document History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0.0-draft | 2025-12-21 | Firestream Team | Initial draft |
| 1.0.0 | 2025-12-21 | Firestream Team | Approved with all design decisions |

---

*End of Document*
