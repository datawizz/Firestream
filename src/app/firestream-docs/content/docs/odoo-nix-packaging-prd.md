---
title: "PRD: Odoo Nix Packaging"
description: "Product Requirements Document for migrating Odoo container packaging to Nix"
---

# Product Requirements Document: Odoo Nix Packaging Migration

**Version:** 1.0
**Date:** 2025-12-22
**Status:** Draft
**Author:** Generated from codebase analysis

---

## 1. Executive Summary

This document outlines the requirements for migrating Odoo container packaging from the current Bitnami-based approach to a fully Nix-based solution. The migration will leverage `uv2nix` for Python dependency management and Nix for all system dependencies and binary downloads.

### 1.1 Current State Analysis

| Approach | Location | Maturity | Python Deps | Reproducibility |
|----------|----------|----------|-------------|-----------------|
| Bitnami | `src/containers/firestream/odoo/` | Production-ready | Pre-built binaries | Container-level |
| nix-odoo | `_WIP/nix-odoo/` | Legacy template | pip + requirements.txt | Nix shell pinning |
| odoo-nix | `_WIP/odoo-nix/` | Modern dev env | pip + venvShellHook | Nix flake lock |

### 1.2 Key Findings

1. **Bitnami approach** downloads pre-built NAMI components (Python 3.12, Node 22, PostgreSQL client, Odoo) as tarballs with SHA256 verification
2. **nix-odoo** uses legacy `shell.nix` with tarball-based nixpkgs; Python deps via pip at runtime
3. **odoo-nix** uses modern `flake.nix` but still relies on `pip install -r` for Python dependencies
4. **Neither Nix approach** fully packages Odoo with Nix; both require manual Odoo source cloning

---

## 2. Goals & Non-Goals

### 2.1 Goals

1. **Full Reproducibility**: Entire dependency tree locked via Nix, eliminating runtime pip installs
2. **uv2nix Integration**: Use `uv2nix` to convert `uv.lock` to Nix derivations for Python dependencies
3. **Binary Caching**: All builds cacheable via Nix binary cache (Cachix or self-hosted)
4. **Multi-Architecture**: Support both x86_64-linux and aarch64-linux (matching Bitnami)
5. **Container Compatibility**: Produce both Nix development shells and OCI-compliant container images
6. **Version Flexibility**: Support Odoo 17 and 18 with templated version switching

### 2.2 Non-Goals

1. **Windows Support**: Focus on Linux (container deployment target)
2. **macOS Development**: Out of scope for container production; dev shells may work
3. **Odoo Enterprise**: Community edition only
4. **Multi-Tenant Deployment**: Infrastructure concerns handled separately by Firestream

---

## 3. Technical Requirements

### 3.1 Nix Flake Structure

```
src/containers/firestream/odoo-nix/
├── flake.nix                    # Main flake entry point
├── flake.lock                   # Locked dependencies
├── pyproject.toml               # Python project definition (PEP 621)
├── uv.lock                      # uv-generated lockfile
├── overlay.nix                  # Custom Nix overlays
├── modules/
│   ├── odoo.nix                 # Odoo derivation
│   ├── wkhtmltopdf.nix          # PDF generation binary
│   └── rtlcss.nix               # Node.js RTL CSS tool
├── packages/
│   ├── odoo-18/                 # Version-specific config
│   │   ├── default.nix
│   │   └── requirements.txt     # For reference/generation
│   └── odoo-17/
│       ├── default.nix
│       └── requirements.txt
├── container/
│   ├── docker.nix               # OCI image derivation
│   ├── entrypoint.sh            # Container entrypoint
│   └── odoo.conf.template       # Configuration template
└── tests/
    └── smoke-test.nix           # Nix-based integration tests
```

### 3.2 Python Dependency Management with uv2nix

#### 3.2.1 Conversion Workflow

```bash
# Step 1: Generate pyproject.toml from Odoo requirements.txt
# (One-time conversion, maintained going forward)

# Step 2: Create uv.lock
uv lock

# Step 3: Convert to Nix using uv2nix
# Handled automatically in flake.nix via uv2nix input
```

#### 3.2.2 pyproject.toml Requirements

The `pyproject.toml` must include:

```toml
[project]
name = "odoo-firestream"
version = "18.0.0"
requires-python = ">=3.12,<3.13"

dependencies = [
    # Core Odoo dependencies (from Odoo 18 requirements.txt)
    "babel>=2.9.1",
    "chardet>=4.0.0",
    "cryptography>=40.0.0",
    "decorator>=5.1.1",
    "docutils>=0.17",
    "gevent>=21.8.0",
    "greenlet>=1.1.0",
    "idna>=3.3",
    "Jinja2>=3.0.3",
    "lxml>=4.9.0",
    "lxml-html-clean>=0.1.0",
    "MarkupSafe>=2.1.0",
    "num2words>=0.5.10",
    "ofxparse>=0.21",
    "passlib>=1.7.4",
    "Pillow>=9.0.0",
    "polib>=1.1.1",
    "psutil>=5.9.0",
    "psycopg2-binary>=2.9.3",
    "pydot>=1.4.2",
    "pyopenssl>=21.0.0",
    "PyPDF2>=2.0.0",
    "pyserial>=3.5",
    "python-dateutil>=2.8.2",
    "python-ldap>=3.4.0",
    "python-stdnum>=1.17",
    "pytz>=2022.1",
    "pyusb>=1.2.1",
    "qrcode>=7.3.1",
    "reportlab>=3.6.0",
    "requests>=2.27.0",
    "rjsmin>=1.2.0",
    "urllib3>=1.26.0",
    "vobject>=0.9.6.1",
    "Werkzeug>=2.2.0",
    "xlrd>=2.0.1",
    "XlsxWriter>=3.0.0",
    "xlwt>=1.3.0",
    "zeep>=4.1.0",
]

[project.optional-dependencies]
dev = [
    "debugpy>=1.6.0",
    "ipdb>=0.13.0",
    "pylint-odoo>=8.0.0",
]
```

#### 3.2.3 uv2nix Flake Integration

```nix
{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.11";
    flake-utils.url = "github:numtide/flake-utils";
    uv2nix = {
      url = "github:pyproject-nix/uv2nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    pyproject-nix = {
      url = "github:pyproject-nix/pyproject.nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, uv2nix, pyproject-nix, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };

        # Parse uv.lock and generate Python environment
        workspace = uv2nix.lib.workspace.loadWorkspace {
          workspaceRoot = ./.;
        };

        overlay = workspace.mkPyprojectOverlay {
          sourcePreference = "wheel";
        };

        python = pkgs.python312;
        pythonSet = (pkgs.callPackage pyproject-nix.lib.mkPythonSet {
          inherit python;
        }).overrideScope overlay;

      in {
        packages.default = pythonSet.mkVirtualEnv "odoo-env" workspace.deps.all;

        devShells.default = pkgs.mkShell {
          packages = [
            self.packages.${system}.default
            pkgs.postgresql_16
            pkgs.nodejs_22
            # ... additional tools
          ];
        };
      }
    );
}
```

### 3.3 Non-Python Dependencies

All system dependencies must be fetched via Nix, not downloaded at build time:

#### 3.3.1 wkhtmltopdf

```nix
# modules/wkhtmltopdf.nix
{ lib, stdenv, fetchurl, autoPatchelfHook, dpkg
, fontconfig, freetype, libjpeg, libpng, libX11, libXrender, openssl, zlib }:

let
  version = "0.12.6.1-3";

  src = {
    x86_64-linux = fetchurl {
      url = "https://github.com/wkhtmltopdf/packaging/releases/download/${version}/wkhtmltox_${version}.bookworm_amd64.deb";
      sha256 = "sha256-XXXX...";  # Pin exact hash
    };
    aarch64-linux = fetchurl {
      url = "https://github.com/wkhtmltopdf/packaging/releases/download/${version}/wkhtmltox_${version}.bookworm_arm64.deb";
      sha256 = "sha256-YYYY...";  # Pin exact hash
    };
  }.${stdenv.hostPlatform.system} or (throw "Unsupported platform");

in stdenv.mkDerivation {
  pname = "wkhtmltopdf";
  inherit version src;

  nativeBuildInputs = [ autoPatchelfHook dpkg ];
  buildInputs = [ fontconfig freetype libjpeg libpng libX11 libXrender openssl zlib ];

  unpackPhase = "dpkg-deb -x $src .";

  installPhase = ''
    mkdir -p $out
    cp -r usr/local/* $out/
  '';
}
```

#### 3.3.2 Node.js Tools (rtlcss, less)

```nix
# modules/rtlcss.nix
{ buildNpmPackage, fetchFromGitHub }:

buildNpmPackage rec {
  pname = "rtlcss";
  version = "4.1.1";

  src = fetchFromGitHub {
    owner = "MohammadYounes";
    repo = "rtlcss";
    rev = "v${version}";
    sha256 = "sha256-XXXX...";
  };

  npmDepsHash = "sha256-YYYY...";
}
```

#### 3.3.3 Fonts

```nix
# Include in devShell/container
fonts = with pkgs; [
  dejavu_fonts
  liberation_ttf
  noto-fonts
  roboto
  font-awesome
];
```

### 3.4 Odoo Source Fetching

```nix
# modules/odoo.nix
{ lib, fetchFromGitHub, python312Packages, wkhtmltopdf, rtlcss, less, postgresql }:

let
  version = "18.0";

  odooSrc = fetchFromGitHub {
    owner = "odoo";
    repo = "odoo";
    rev = "${version}";  # Or specific commit hash for reproducibility
    sha256 = "sha256-XXXX...";
  };

in python312Packages.buildPythonApplication {
  pname = "odoo";
  inherit version;
  src = odooSrc;

  propagatedBuildInputs = [
    # Python dependencies from uv2nix
  ];

  nativeBuildInputs = [ wkhtmltopdf rtlcss less ];

  # Odoo doesn't follow standard Python packaging
  format = "other";

  installPhase = ''
    mkdir -p $out/lib/odoo
    cp -r . $out/lib/odoo/

    mkdir -p $out/bin
    cat > $out/bin/odoo <<EOF
    #!/bin/sh
    exec ${python312Packages.python}/bin/python $out/lib/odoo/odoo-bin "\$@"
    EOF
    chmod +x $out/bin/odoo
  '';
}
```

### 3.5 Container Image Generation

```nix
# container/docker.nix
{ pkgs, odoo, postgresql }:

pkgs.dockerTools.buildLayeredImage {
  name = "firestream/odoo";
  tag = "18-nix";

  contents = [
    pkgs.bashInteractive
    pkgs.coreutils
    pkgs.procps
    odoo
    postgresql
    # Include fonts, SSL certs, etc.
    pkgs.cacert
    pkgs.dejavu_fonts
  ];

  config = {
    Cmd = [ "/bin/odoo" "--config=/etc/odoo/odoo.conf" ];
    ExposedPorts = {
      "8069/tcp" = {};
      "8072/tcp" = {};
    };
    Env = [
      "ODOO_RC=/etc/odoo/odoo.conf"
      "PATH=/bin"
    ];
    User = "odoo";
  };

  extraCommands = ''
    # Create odoo user
    mkdir -p etc
    echo 'odoo:x:1001:1001:Odoo:/var/lib/odoo:/bin/sh' >> etc/passwd
    echo 'odoo:x:1001:' >> etc/group

    # Create directories
    mkdir -p var/lib/odoo
    mkdir -p etc/odoo
  '';
}
```

---

## 4. Migration Path

### Phase 1: Foundation (Week 1-2)
1. Create `pyproject.toml` from Odoo 18 `requirements.txt`
2. Generate initial `uv.lock` file
3. Implement basic `flake.nix` with uv2nix integration
4. Verify Python environment builds correctly

### Phase 2: System Dependencies (Week 2-3)
1. Package wkhtmltopdf as Nix derivation
2. Package rtlcss/less via buildNpmPackage
3. Add font packages
4. Test PDF generation and SCSS compilation

### Phase 3: Odoo Derivation (Week 3-4)
1. Create Odoo fetchFromGitHub derivation
2. Implement odoo-bin wrapper script
3. Add configuration template system
4. Verify Odoo starts and serves web interface

### Phase 4: Container Image (Week 4-5)
1. Implement dockerTools.buildLayeredImage
2. Add entrypoint script with Bitnami-style initialization
3. Support environment variable configuration
4. Test container locally with docker-compose

### Phase 5: Integration (Week 5-6)
1. Integrate with Firestream K3D deployment
2. Create Helm chart values for Nix-based image
3. Document upgrade path from Bitnami
4. Add CI/CD for automated builds

---

## 5. Comparison: Bitnami vs Nix Approach

| Aspect | Bitnami | Proposed Nix |
|--------|---------|--------------|
| **Reproducibility** | Container image SHA | Nix flake.lock + content-addressed store |
| **Python Deps** | Pre-built in NAMI tarball | uv2nix from uv.lock |
| **System Deps** | apt-get + download | Nix derivations |
| **Customization** | Environment variables only | Full source modification |
| **Build Time** | ~5 min (download) | ~15 min first build, cached after |
| **Image Size** | ~1.5GB | ~800MB (with proper layering) |
| **Update Strategy** | Pull new Bitnami tag | Update flake.lock + rebuild |
| **Offline Builds** | Impossible | Possible with Nix cache |
| **Multi-arch** | Separate images | Single flake, per-system outputs |

---

## 6. Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| uv2nix doesn't support all Python deps | Medium | High | Fall back to buildPythonPackage overrides |
| wkhtmltopdf binary incompatibility | Low | Medium | Pin exact version, test PDF output |
| Odoo updates break packaging | Medium | Medium | Pin Odoo commit hash, not branch |
| Build cache invalidation | Low | Low | Use Cachix for shared caching |
| PostgreSQL client version mismatch | Low | Low | Pin both server and client versions |

---

## 7. Success Criteria

1. **Functional**: Odoo web interface accessible, can create/edit records
2. **PDF Generation**: Reports generate correctly with fonts
3. **Reproducible**: `nix build` produces identical output across machines
4. **Cacheable**: Cachix can cache and restore builds
5. **Configurable**: Environment variables work like Bitnami
6. **Deployable**: Container runs in K3D with Firestream stack
7. **Maintainable**: Odoo version upgrades require minimal changes

---

## 8. Open Questions

1. **PostgreSQL Bundling**: Should PostgreSQL be in the same container (like dev) or separate (like Bitnami production)?
2. **Custom Addons**: How should custom Odoo addons be integrated into the Nix build?
3. **Secrets Management**: Should we support `_FILE` suffix pattern like Bitnami?
4. **Upgrade Scripts**: Should Odoo module upgrades be part of container init?
5. **Enterprise Compatibility**: If needed later, how would enterprise modules integrate?

---

## 9. References

- [uv2nix Documentation](https://github.com/pyproject-nix/uv2nix)
- [pyproject.nix](https://github.com/pyproject-nix/pyproject.nix)
- [Nix dockerTools](https://nixos.org/manual/nixpkgs/stable/#sec-pkgs-dockerTools)
- [Odoo 18 Requirements](https://github.com/odoo/odoo/blob/18.0/requirements.txt)
- [Bitnami Odoo Container](https://github.com/bitnami/containers/tree/main/bitnami/odoo)

---

## Appendix A: Current Implementation Analysis

### A.1 Bitnami Approach Strengths

1. **Mature initialization logic**: `libodoo.sh` handles database connection, migrations, and configuration
2. **Non-root security**: Runs as UID 1001
3. **Secrets support**: `_FILE` suffix pattern for Kubernetes secrets
4. **PostgreSQL integration**: Built-in connection retries and validation
5. **Custom init hooks**: Shell, Python, and SQL scripts in `/docker-entrypoint-init.d/`

### A.2 nix-odoo Approach Insights

1. **Multi-version templates**: Pattern for supporting Odoo 13-18 with version-pinned nixpkgs
2. **dev-server.sh orchestration**: 288-line script handling PostgreSQL, virtualenv, and Odoo lifecycle
3. **Socket-based PostgreSQL**: Avoids port conflicts with host
4. **Pre-commit integration**: Code quality enforcement baked in

### A.3 odoo-nix Approach Insights

1. **Modern flake structure**: Uses `flake-utils` for multi-system support
2. **postVenvCreation hook**: Handles pip install after venv creation
3. **Just command runner**: Simplified developer commands
4. **Comprehensive fonts**: Proper typography for PDF generation

---

## Appendix B: Proposed flake.nix Template

```nix
{
  description = "Firestream Odoo - Nix-based Odoo packaging";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.11";
    flake-utils.url = "github:numtide/flake-utils";

    uv2nix = {
      url = "github:pyproject-nix/uv2nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    pyproject-nix = {
      url = "github:pyproject-nix/pyproject.nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    pyproject-build-systems = {
      url = "github:pyproject-nix/build-system-pkgs";
      inputs.pyproject-nix.follows = "pyproject-nix";
      inputs.uv2nix.follows = "uv2nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, uv2nix, pyproject-nix, pyproject-build-systems }:
    flake-utils.lib.eachSystem [ "x86_64-linux" "aarch64-linux" ] (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          config.allowUnfree = true;  # For wkhtmltopdf if needed
        };

        # Load Python workspace from uv.lock
        workspace = uv2nix.lib.workspace.loadWorkspace {
          workspaceRoot = ./.;
        };

        # Create overlay from uv.lock
        overlay = workspace.mkPyprojectOverlay {
          sourcePreference = "wheel";
        };

        # Build system overlay
        pyprojectOverrides = pyproject-build-systems.lib.mkOverlay { };

        # Create Python set with all dependencies
        python = pkgs.python312;
        pythonSet = (pkgs.callPackage pyproject-nix.lib.mkPythonSet {
          inherit python;
        }).overrideScope (
          pkgs.lib.composeManyExtensions [ pyprojectOverrides overlay ]
        );

        # Python virtual environment with all deps
        odooVenv = pythonSet.mkVirtualEnv "odoo-venv" workspace.deps.all;

        # wkhtmltopdf derivation
        wkhtmltopdf = pkgs.callPackage ./modules/wkhtmltopdf.nix { };

        # Odoo source
        odooSrc = pkgs.fetchFromGitHub {
          owner = "odoo";
          repo = "odoo";
          rev = "18.0";
          sha256 = "sha256-FIXME";
        };

        # Full Odoo package
        odoo = pkgs.callPackage ./modules/odoo.nix {
          inherit odooVenv wkhtmltopdf odooSrc;
        };

        # Container image
        odooImage = pkgs.callPackage ./container/docker.nix {
          inherit odoo;
        };

      in {
        packages = {
          default = odoo;
          inherit odooVenv odoo odooImage wkhtmltopdf;
        };

        devShells.default = pkgs.mkShell {
          packages = [
            odooVenv
            pkgs.postgresql_16
            pkgs.nodejs_22
            pkgs.less
            wkhtmltopdf
            pkgs.just
            pkgs.uv
          ];

          shellHook = ''
            export PGDATA="$PWD/.pgdata"
            export PGHOST="$PWD/.pgdata"
            export PGPORT=5433
            export PGDATABASE=odoo-dev

            if [ ! -d "$PGDATA" ]; then
              initdb --auth=trust --no-locale --encoding=UTF8
            fi

            if ! pg_isready -q; then
              pg_ctl start -l "$PGDATA/postgres.log"
            fi

            echo "Odoo Nix development environment ready"
            echo "Run 'just init' to initialize database"
            echo "Run 'just run' to start Odoo"
          '';
        };

        checks = {
          # Add smoke tests
        };
      }
    );
}
```

---

*End of Document*
