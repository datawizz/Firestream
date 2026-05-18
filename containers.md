# Plan: Single Flake Container Build with Docker-Only Dependency

## Summary

Refactor Firestream's container build system to:
1. **Use a single flake approach** - all containers built from root flake
2. **Make Docker the only dependency** - use the firestream-devcontainer as a Nix builder
3. **Use volume-based Nix store caching** - avoids container bloat from commits
4. **Adopt the reentry pattern** - same script runs on host or in container

## Key Design Decisions

### Volume-Based Cache vs Container Commits

**ConceptDB's approach (avoid):**
```bash
# After build, commit container with Nix store
docker commit $(docker ps -lq) builder:latest  # Image grows each time!
```

**Better approach (use):**
```bash
# Named volume persists Nix store between builds
docker run --mount type=volume,source=firestream-nix-store,target=/nix ...
```

| Approach | Pros | Cons |
|----------|------|------|
| Container Commit | Simple, portable | Image bloat, slow pushes |
| Named Volume | Fixed image size, fast | Per-machine cache |

The volume approach keeps the builder image small (~2GB) while the Nix store cache (~5-10GB) lives in a named volume that persists across builds but doesn't bloat the image.

---

## Implementation Phases

### Phase 1: Add Missing Containers to Root Flake

**File**: `flake.nix`

Add Kafka and Spark exports using same pattern as PostgreSQL/Redis:

```nix
# After mkRedis definition (~line 363)
mkKafka = let
  firestream = import ./bin/nix/firestream { inherit pkgs; };
  mod = import ./src/containers/firestream/kafka/module.nix {
    inherit pkgs;
    lib = pkgs.lib;
    firestream = firestream;
    version = "4.0";
  };
in mod.dockerImage;

mkSpark = let
  firestream = import ./bin/nix/firestream { inherit pkgs; };
  mod = import ./src/containers/firestream/spark/module.nix {
    inherit pkgs;
    lib = pkgs.lib;
    firestream = firestream;
    sparkVersion = "4.0.0";
    jdk = pkgs.temurin-bin-17;
    python = pkgs.python312;
  };
in mod.dockerImage;
```

Add to packages (~line 393):
```nix
kafka = if isLinux then mkKafka else unavailable "kafka";
spark = if isLinux then mkSpark else unavailable "spark";
```

---

### Phase 2: Create Builder Infrastructure

#### 2.1 Create `bin/build/_common.sh`

Shared functions for build scripts:

```bash
#!/usr/bin/env bash
# _common.sh - shared preamble for Firestream build scripts

set -euo pipefail

SCRIPT_DIR="${SCRIPT_DIR:-$(cd "$(dirname "${BASH_SOURCE[1]}")" && pwd -P)}"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd -P)"

# Source docker_preinit.sh for git worktree detection
source "$REPO_ROOT/docker/firestream/docker_preinit.sh"

BUILD_OUTPUT_DIR="${BUILD_OUTPUT_DIR:-$REPO_ROOT/_build}"
BUILDER_IMAGE_NAME="firestream-devcontainer"

# ── Logging ──────────────────────────────────────────────────────────
if [[ -t 1 ]]; then
    BOLD='\033[1m' DIM='\033[2m' RESET='\033[0m'
    RED='\033[31m' GREEN='\033[32m' YELLOW='\033[33m' BLUE='\033[34m' CYAN='\033[36m'
else
    BOLD='' DIM='' RESET='' RED='' GREEN='' YELLOW='' BLUE='' CYAN=''
fi

log_info()  { printf "${DIM}%s${RESET} ${CYAN}>>>${RESET} %s\n" "$(date +%H:%M:%S)" "$*"; }
log_step()  { printf "${DIM}%s${RESET} ${BLUE} ->${RESET} %s\n" "$(date +%H:%M:%S)" "$*"; }
log_ok()    { printf "${DIM}%s${RESET} ${GREEN}  ✓${RESET} %s\n" "$(date +%H:%M:%S)" "$*"; }
log_warn()  { printf "${DIM}%s${RESET} ${YELLOW}  !${RESET} %s\n" "$(date +%H:%M:%S)" "$*"; }
log_error() { printf "${DIM}%s${RESET} ${RED}  ✗${RESET} %s\n" "$(date +%H:%M:%S)" "$*" >&2; }

# ── Docker Resource Detection ────────────────────────────────────────
_detect_docker_cpus() {
    docker info --format '{{.NCPU}}' 2>/dev/null || nproc 2>/dev/null || echo 4
}

_detect_docker_memory() {
    local bytes
    bytes=$(docker info --format '{{.MemTotal}}' 2>/dev/null) || bytes=0
    if [[ "$bytes" -gt 0 ]]; then
        local gb=$(( bytes / 1073741824 ))
        local usable=$(( gb > 1 ? gb - 1 : 1 ))  # Reserve 1GB overhead
        echo "${usable}g"
    else
        echo "8g"
    fi
}

DOCKER_CPUS="${DOCKER_CPUS:-$(_detect_docker_cpus)}"
DOCKER_MEMORY="${DOCKER_MEMORY:-$(_detect_docker_memory)}"
DOCKER_SWAP="${DOCKER_SWAP:-$(( ${DOCKER_MEMORY%g} * 2 ))g}"

# ── Architecture-Specific Nix Store Volumes ──────────────────────────
# Separate volumes per architecture to avoid cache pollution
get_nix_volume() {
    local arch="${1:-$(uname -m)}"
    case "$arch" in
        x86_64|amd64)  echo "firestream-nix-store-amd64" ;;
        aarch64|arm64) echo "firestream-nix-store-arm64" ;;
        *)             echo "firestream-nix-store-$arch" ;;
    esac
}

# ── Container Registry ───────────────────────────────────────────────
declare -A CONTAINER_REGISTRY=(
    ["postgresql:16"]="postgresql-16"
    ["postgresql:17"]="postgresql-17"
    ["postgresql:"]="postgresql-17"
    ["redis:7"]="redis-7"
    ["redis:8"]="redis-8"
    ["redis:"]="redis-7"
    ["superset:4"]="superset-4"
    ["superset:5"]="superset-5"
    ["superset:"]="superset-5"
    ["airflow:"]="airflow"
    ["kafka:"]="kafka"
    ["spark:"]="spark"
    ["jupyterhub:"]="jupyterhub"
    ["odoo:"]="odoo"
)

resolve_package_name() {
    local container="$1"
    local version="${2:-}"
    local key="${container}:${version}"

    [[ -n "${CONTAINER_REGISTRY[$key]:-}" ]] && { echo "${CONTAINER_REGISTRY[$key]}"; return 0; }
    [[ -n "${CONTAINER_REGISTRY[${container}:]:-}" ]] && { echo "${CONTAINER_REGISTRY[${container}:]}"; return 0; }

    log_error "Unknown container: $container"
    return 1
}

# ── Git Worktree Mount Resolution ────────────────────────────────────
resolve_git_mounts() {
    local repo_root="$1"
    GIT_DOCKER_MOUNTS=()

    detect_git_worktree "$repo_root"
    if [[ "$GIT_WORKTREE_DETECTED" == "true" ]]; then
        log_step "Git worktree detected - mounting git dirs for Nix"
        GIT_DOCKER_MOUNTS=(-v "$GIT_WORKTREE_COMMON_DIR:$GIT_WORKTREE_COMMON_DIR:ro")
    fi
}
```

#### 2.2 Create `bin/build/container-images.sh`

Main build script with reentry pattern and volume-based caching:

```bash
#!/usr/bin/env bash
# container-images.sh - Build container images via Nix
#
# Usage:
#   ./bin/build/container-images.sh <container1> [container2] ...
#   ./bin/build/container-images.sh postgresql --version 17
#   ./bin/build/container-images.sh airflow kafka spark
#
# Uses volume-based Nix store caching (no container bloat)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
source "$SCRIPT_DIR/_common.sh"

TARGET_ARCH="${TARGET_ARCH:-$(uname -m)}"

# Parse arguments
CONTAINERS=()
VERSIONS=()
current_version=""

while [[ $# -gt 0 ]]; do
    case "$1" in
        --version) current_version="$2"; shift 2 ;;
        --target)  TARGET_ARCH="$2"; shift 2 ;;
        --help|-h)
            echo "Usage: $0 [--target arch] [--version ver] <container1> [container2] ..."
            exit 0 ;;
        -*) log_error "Unknown option: $1"; exit 1 ;;
        *)
            CONTAINERS+=("$1")
            VERSIONS+=("$current_version")
            current_version=""
            shift
            ;;
    esac
done

[[ ${#CONTAINERS[@]} -eq 0 ]] && { echo "Usage: $0 <container1> [container2] ..."; exit 1; }

# Resolve all package names
PACKAGES=()
for i in "${!CONTAINERS[@]}"; do
    pkg=$(resolve_package_name "${CONTAINERS[$i]}" "${VERSIONS[$i]}")
    PACKAGES+=("$pkg")
done

log_info "Building: ${PACKAGES[*]} (arch: $TARGET_ARCH)"
mkdir -p "$BUILD_OUTPUT_DIR"

# ============= HOST MODE: DOCKER REENTRY =============

if [[ -z "${FIRESTREAM_IN_CONTAINER:-}" ]]; then
    # Acquire batch lock to prevent concurrent builds
    BATCH_LOCK="$BUILD_OUTPUT_DIR/.build-batch.lock"
    cleanup_lock() { rm -rf "$BATCH_LOCK"; }
    if ! mkdir "$BATCH_LOCK" 2>/dev/null; then
        lock_pid=$(cat "$BATCH_LOCK/pid" 2>/dev/null || echo "")
        if [[ -n "$lock_pid" ]] && kill -0 "$lock_pid" 2>/dev/null; then
            log_error "Another build is running (PID $lock_pid)"
            log_error "If stale, remove: rm -rf $BATCH_LOCK"
            exit 1
        fi
        log_warn "Removing stale lock (PID ${lock_pid:-unknown})"
        rm -rf "$BATCH_LOCK"
        mkdir "$BATCH_LOCK"
    fi
    echo $$ > "$BATCH_LOCK/pid"
    trap cleanup_lock EXIT

    # Ensure Docker is available
    if ! command -v docker &>/dev/null; then
        log_error "Docker is required but not found"
        exit 1
    fi

    # Ensure builder image exists (use devcontainer)
    BUILDER_TAG="${BUILDER_IMAGE_NAME}:latest"
    if ! docker image inspect "$BUILDER_TAG" >/dev/null 2>&1; then
        log_info "Building builder image from devcontainer..."
        docker build -t "$BUILDER_TAG" \
            --target devcontainer \
            -f "$REPO_ROOT/docker/firestream/Dockerfile" \
            "$REPO_ROOT"
    fi

    # Map architecture to Docker platform
    local docker_platform
    case "$TARGET_ARCH" in
        x86_64|amd64)  docker_platform="linux/amd64" ;;
        aarch64|arm64) docker_platform="linux/arm64" ;;
        *)             docker_platform="linux/$TARGET_ARCH" ;;
    esac

    # Get architecture-specific Nix store volume
    NIX_VOLUME="$(get_nix_volume "$TARGET_ARCH")"

    resolve_git_mounts "$REPO_ROOT"

    log_info "Docker reentry: nix build inside $BUILDER_TAG"
    log_step "Platform: $docker_platform"
    log_step "Nix store volume: $NIX_VOLUME (persistent cache)"
    log_step "Resources: ${DOCKER_CPUS} CPUs, ${DOCKER_MEMORY} RAM"

    docker run --rm \
        --cpus "$DOCKER_CPUS" \
        --memory "$DOCKER_MEMORY" \
        --memory-swap "$DOCKER_SWAP" \
        --platform "$docker_platform" \
        -v "$REPO_ROOT:$REPO_ROOT:ro" \
        -v "$BUILD_OUTPUT_DIR:$BUILD_OUTPUT_DIR" \
        -v /var/run/docker.sock:/var/run/docker.sock \
        --mount type=volume,source="$NIX_VOLUME",target=/nix \
        ${GIT_DOCKER_MOUNTS[@]+"${GIT_DOCKER_MOUNTS[@]}"} \
        -w "$REPO_ROOT" \
        -e FIRESTREAM_IN_CONTAINER=1 \
        -e BUILD_OUTPUT_DIR="$BUILD_OUTPUT_DIR" \
        "$BUILDER_TAG" \
        bash "$REPO_ROOT/bin/build/container-images.sh" "${CONTAINERS[@]}"
    exit $?
fi

# ============= CONTAINER MODE: SERIAL NIX BUILDS =============

git config --global --add safe.directory "$REPO_ROOT" 2>/dev/null || true

CONTAINER_CPUS=$(nproc 2>/dev/null || echo 4)

export NIX_CONFIG="experimental-features = nix-command flakes
max-jobs = 1
cores = $CONTAINER_CPUS
max-substitution-jobs = 8
http-connections = 8"

log_step "Nix config: max-jobs=1, cores=$CONTAINER_CPUS (serial builds)"

BUILD_START=$(date +%s)
SUCCEEDED=0
FAILED=0

for i in "${!PACKAGES[@]}"; do
    pkg="${PACKAGES[$i]}"
    container="${CONTAINERS[$i]}"
    OUT_DIR="$BUILD_OUTPUT_DIR/$pkg"
    mkdir -p "$OUT_DIR"

    log_info "Building $pkg ($((i+1))/${#PACKAGES[@]})..."

    if nix build ".#$pkg" -o /tmp/result -L --no-update-lock-file 2>&1 | tee "$OUT_DIR/build.log"; then
        cp /tmp/result "$OUT_DIR/${pkg}.tar.gz"
        rm -f "$OUT_DIR/build.log"

        # Load into Docker (via socket)
        log_step "Loading $pkg into Docker..."
        docker load < "$OUT_DIR/${pkg}.tar.gz"

        # Get image tag from loaded output
        IMAGE_TAG=$(docker images --format '{{.Repository}}:{{.Tag}}' | grep "firestream-$container" | head -1)
        log_ok "$pkg → $IMAGE_TAG"

        SUCCEEDED=$((SUCCEEDED + 1))
    else
        log_error "FAILED: $pkg"
        FAILED=$((FAILED + 1))
    fi
done

BUILD_END=$(date +%s)
BUILD_DURATION=$((BUILD_END - BUILD_START))

echo ""
log_info "════════════════════════════════════════════════════════════════"
log_info "  Build Summary"
log_info "════════════════════════════════════════════════════════════════"
printf "  Succeeded: %d\n" "$SUCCEEDED"
printf "  Failed:    %d\n" "$FAILED"
printf "  Duration:  %ds\n" "$BUILD_DURATION"
log_info "════════════════════════════════════════════════════════════════"

[[ "$FAILED" -gt 0 ]] && exit 1
exit 0
```

---

### Phase 3: Refactor `build-container.sh` as Wrapper

**File**: `bin/build-container.sh`

Simplify to delegate to new infrastructure (maintains backward compatibility):

```bash
#!/usr/bin/env bash
# build-container.sh - Wrapper for container-images.sh
# Maintains backward compatibility with existing Makefile targets

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"

# Forward to new build system
exec "$SCRIPT_DIR/build/container-images.sh" "$@"
```

---

### Phase 4: Update Makefile

**File**: `Makefile`

```makefile
# ── Container Build Infrastructure ───────────────────────────────────

BUILD_CONTAINER := bin/build/container-images.sh

# Single container builds
container-build-%:
	@$(BUILD_CONTAINER) $*

# Version-specific builds
postgres-build-16:
	@$(BUILD_CONTAINER) postgresql --version 16

postgres-build-17:
	@$(BUILD_CONTAINER) postgresql --version 17

redis-build-7:
	@$(BUILD_CONTAINER) redis --version 7

redis-build-8:
	@$(BUILD_CONTAINER) redis --version 8

superset-build-4:
	@$(BUILD_CONTAINER) superset --version 4

superset-build-5:
	@$(BUILD_CONTAINER) superset --version 5

# Batch builds
containers-build-all:
	@$(BUILD_CONTAINER) postgresql redis airflow kafka spark jupyterhub odoo superset

# ── Builder Management ───────────────────────────────────────────────

# Build the builder image (devcontainer)
builder-build:
	docker build -t firestream-devcontainer:latest \
		--target devcontainer \
		-f docker/firestream/Dockerfile .

# Show Nix store cache usage
builder-cache-stats:
	@echo "=== Nix Store Cache Volumes ==="
	@docker volume ls --filter name=firestream-nix-store
	@echo ""
	@for vol in $$(docker volume ls -q --filter name=firestream-nix-store); do \
		size=$$(docker system df -v 2>/dev/null | grep "$$vol" | awk '{print $$3}' || echo "unknown"); \
		echo "  $$vol: $$size"; \
	done

# Clean Nix store cache (reclaim disk space)
builder-cache-clean:
	@echo "Removing Nix store cache volumes..."
	docker volume rm firestream-nix-store-amd64 2>/dev/null || true
	docker volume rm firestream-nix-store-arm64 2>/dev/null || true
	@echo "Cache cleared. Next build will be slower (cold cache)."
```

---

## Critical Files

| File | Action | Purpose |
|------|--------|---------|
| `flake.nix` | Modify | Add Kafka/Spark exports |
| `bin/build/_common.sh` | **Create** | Shared functions (registry, logging, volumes) |
| `bin/build/container-images.sh` | **Create** | Main build script with reentry |
| `bin/build-container.sh` | Modify | Simplify as wrapper |
| `Makefile` | Modify | Update targets, add cache management |

---

## Build Flow

```
┌──────────────────────────────────────────────────────────────────────┐
│  Darwin/Linux Host                                                    │
│                                                                       │
│  $ make airflow-build                                                │
│      ↓                                                               │
│  bin/build/container-images.sh airflow                               │
│      ↓                                                               │
│  FIRESTREAM_IN_CONTAINER not set → HOST MODE                         │
│      ↓                                                               │
│  docker run firestream-devcontainer:latest                           │
│    --mount type=volume,source=firestream-nix-store-amd64,target=/nix │
│    -v /var/run/docker.sock:/var/run/docker.sock                      │
│    -e FIRESTREAM_IN_CONTAINER=1                                      │
│      ↓                                                               │
│  ┌────────────────────────────────────────────────────────────────┐  │
│  │  Inside Container                                               │  │
│  │                                                                 │  │
│  │  FIRESTREAM_IN_CONTAINER=1 → CONTAINER MODE                     │  │
│  │      ↓                                                          │  │
│  │  nix build .#airflow -o /tmp/result                            │  │
│  │      ↓                                                          │  │
│  │  cp /tmp/result _build/airflow/airflow.tar.gz                  │  │
│  │      ↓                                                          │  │
│  │  docker load < _build/airflow/airflow.tar.gz                   │  │
│  │      ↓                                                          │  │
│  │  firestream-airflow:3.0.3 ✓                                    │  │
│  └────────────────────────────────────────────────────────────────┘  │
│                                                                       │
│  Image available: firestream-airflow:3.0.3                           │
└──────────────────────────────────────────────────────────────────────┘

Volume: firestream-nix-store-amd64
  └─ Persists /nix store between builds
  └─ ~5-10GB cache, fixed size
  └─ Builder image stays ~2GB
```

---

## Benefits Over ConceptDB Approach

| Aspect | ConceptDB (Commit) | Firestream (Volume) |
|--------|-------------------|---------------------|
| Image Size | Grows with each build | Fixed (~2GB) |
| Cache Location | Inside image | Named volume |
| CI/CD | Must push large images | Only push small builder |
| Disk Usage | Image + cache duplicated | Single cache volume |
| Cache Invalidation | Rebuild image | `docker volume rm` |

---

## Verification

```bash
# Test single container build
make airflow-build
docker images | grep firestream-airflow

# Test versioned container
bin/build/container-images.sh postgresql --version 17
docker images | grep firestream-postgresql

# Test batch build
bin/build/container-images.sh airflow kafka spark

# Check cache usage
make builder-cache-stats

# Clean cache if needed
make builder-cache-clean

# Verify cross-platform (on macOS)
bin/build/container-images.sh --target x86_64 airflow
# Should use firestream-nix-store-amd64 volume
```
