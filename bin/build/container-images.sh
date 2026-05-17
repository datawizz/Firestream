#!/usr/bin/env bash
# container-images.sh - Build container images via Nix
#
# Builds containers inside Docker with volume-based Nix cache.
# Supports git worktrees by mounting git directories at original paths.
#
# Usage:
#   ./bin/build/container-images.sh <container1> [container2] ...
#   ./bin/build/container-images.sh postgresql --version 17
#   ./bin/build/container-images.sh airflow kafka spark
#
# Copyright Firestream. MIT License.

set -euo pipefail

echo "NOTICE: This script is deprecated. Use 'firestream build <container>' instead." >&2
echo "See: firestream build --help" >&2
echo "" >&2

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
source "$SCRIPT_DIR/_common.sh"

TARGET_ARCH="${TARGET_ARCH:-$(uname -m)}"

# ── Usage ────────────────────────────────────────────────────────────
usage() {
    echo ""
    printf "${BOLD}Usage:${RESET} $0 [options] <container1> [container2] ...\n"
    echo ""
    echo "  Build Firestream container images using Nix."
    echo "  Automatically runs builds inside Docker with persistent Nix cache."
    echo ""
    printf "${BOLD}Containers:${RESET}\n"
    list_containers | sed 's/^/    /'
    echo ""
    printf "${BOLD}Options:${RESET}\n"
    printf "  ${CYAN}--target <arch>${RESET}     Target architecture (x86_64, aarch64)\n"
    printf "  ${CYAN}--version <ver>${RESET}     Container version (applies to next container)\n"
    printf "  ${CYAN}--help${RESET}              Show this help\n"
    echo ""
    printf "${BOLD}Examples:${RESET}\n"
    printf "  ${DIM}\$${RESET} $0 airflow                    ${DIM}# Build single container${RESET}\n"
    printf "  ${DIM}\$${RESET} $0 postgresql --version 17    ${DIM}# Build specific version${RESET}\n"
    printf "  ${DIM}\$${RESET} $0 airflow kafka spark        ${DIM}# Build multiple containers${RESET}\n"
    printf "  ${DIM}\$${RESET} $0 --target x86_64 airflow    ${DIM}# Cross-compile for x86_64${RESET}\n"
    echo ""
    exit 1
}

# ── Parse Arguments ──────────────────────────────────────────────────
CONTAINERS=()
VERSIONS=()
current_version=""

while [[ $# -gt 0 ]]; do
    case "$1" in
        --version)
            [[ $# -lt 2 ]] && { log_error "--version requires argument"; exit 1; }
            current_version="$2"
            shift 2
            ;;
        --target)
            [[ $# -lt 2 ]] && { log_error "--target requires argument"; exit 1; }
            TARGET_ARCH="$2"
            shift 2
            ;;
        --help|-h)
            usage
            ;;
        -*)
            log_error "Unknown option: $1"
            exit 1
            ;;
        *)
            CONTAINERS+=("$1")
            VERSIONS+=("$current_version")
            current_version=""
            shift
            ;;
    esac
done

[[ ${#CONTAINERS[@]} -eq 0 ]] && usage

# Validate containers exist
for container in "${CONTAINERS[@]}"; do
    if [[ ! -d "$CONTAINERS_DIR/$container" ]]; then
        log_error "Container '$container' not found in $CONTAINERS_DIR"
        echo ""
        echo "Available containers:"
        list_containers | sed 's/^/  /'
        exit 1
    fi
done

# Resolve all package names
PACKAGES=()
for i in "${!CONTAINERS[@]}"; do
    pkg=$(resolve_package_name "${CONTAINERS[$i]}" "${VERSIONS[$i]}")
    PACKAGES+=("$pkg")
done

log_info "Building: ${PACKAGES[*]} (arch: $TARGET_ARCH)"

# ── Setup ────────────────────────────────────────────────────────────
mkdir -p "$BUILD_OUTPUT_DIR"

# Acquire batch lock to prevent concurrent builds
BATCH_LOCK="$BUILD_OUTPUT_DIR/.build-batch.lock"
INTERRUPTED=false
DOCKER_PID=""

cleanup() {
    if [[ -n "$DOCKER_PID" ]] && kill -0 "$DOCKER_PID" 2>/dev/null; then
        kill "$DOCKER_PID" 2>/dev/null || true
        wait "$DOCKER_PID" 2>/dev/null || true
    fi
    rm -rf "$BATCH_LOCK"
}

handle_interrupt() {
    INTERRUPTED=true
    echo ""
    log_warn "Interrupted - stopping builds..."
    cleanup
    exit 130
}

trap cleanup EXIT
trap handle_interrupt INT TERM

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

# Ensure Docker is available
if ! command -v docker &>/dev/null; then
    log_error "Docker is required but not found"
    exit 1
fi

# Use nixos/nix as the builder image
BUILDER_TAG="nixos/nix:latest"
if ! docker image inspect "$BUILDER_TAG" >/dev/null 2>&1; then
    log_info "Pulling builder image..."
    docker pull "$BUILDER_TAG"
fi

# Map architecture to Docker platform
docker_platform=""
case "$TARGET_ARCH" in
    x86_64|amd64)  docker_platform="linux/amd64" ;;
    aarch64|arm64) docker_platform="linux/arm64" ;;
    *)             docker_platform="linux/$TARGET_ARCH" ;;
esac

# Get architecture-specific Nix store volume
NIX_VOLUME="$(get_nix_volume "$TARGET_ARCH")"

# Resolve git mounts (critical for worktree support)
resolve_git_mounts "$REPO_ROOT"

log_info "Building with Nix inside Docker"
log_step "Platform: $docker_platform"
log_step "Nix store volume: $NIX_VOLUME (persistent cache)"
log_step "Resources: ${DOCKER_CPUS} CPUs, ${DOCKER_MEMORY} RAM"

if [[ "$GIT_WORKTREE_DETECTED" == "true" ]]; then
    log_step "Worktree detected - mounting git dirs at original paths"
fi

# Build docker run base arguments
# NOTE: Mount _build at /build (not nested under repo) to avoid permission issues
# with nested bind mounts. The repo is mounted read-only, but we need write access
# to the output directory.
docker_args=(
    --rm
    --cpus "$DOCKER_CPUS"
    --memory "$DOCKER_MEMORY"
    --memory-swap "$DOCKER_SWAP"
    --platform "$docker_platform"
    -v "$REPO_ROOT:$REPO_ROOT:ro"
    -v "$BUILD_OUTPUT_DIR:/build"
    -v /var/run/docker.sock:/var/run/docker.sock
    --mount "type=volume,source=$NIX_VOLUME,target=/nix"
    -w "$REPO_ROOT"
)

# Add git worktree mounts if detected
if [[ ${#GIT_DOCKER_MOUNTS[@]} -gt 0 ]]; then
    docker_args+=("${GIT_DOCKER_MOUNTS[@]}")
fi

# ── Build each container ─────────────────────────────────────────────
BUILD_START=$(date +%s)
SUCCEEDED=0
FAILED=0
FAILED_CONTAINERS=()

for i in "${!PACKAGES[@]}"; do
    # Check for interrupt before starting next build
    if $INTERRUPTED; then
        log_warn "Skipping remaining builds due to interrupt"
        break
    fi

    pkg="${PACKAGES[$i]}"
    container="${CONTAINERS[$i]}"
    OUT_DIR="$BUILD_OUTPUT_DIR/$pkg"
    # Container sees output at /build/$pkg (not nested under repo mount)
    CONTAINER_OUT_DIR="/build/$pkg"
    mkdir -p "$OUT_DIR"

    echo ""
    log_info "Building $pkg ($((i+1))/${#PACKAGES[@]})..."

    # Run nix build inside container
    # Use process substitution to capture docker exit code while still teeing output.
    # The inner script uses set -euo pipefail for strict error handling.
    # cp -L dereferences the nix symlink to copy the actual file.
    docker run "${docker_args[@]}" "$BUILDER_TAG" \
        sh -c '
            set -euo pipefail

            echo "experimental-features = nix-command flakes" >> /etc/nix/nix.conf
            git config --global --add safe.directory "'"$REPO_ROOT"'" 2>/dev/null || true

            echo ">>> Building '"$pkg"'..."
            nix build ".#'"$pkg"'" -o /tmp/result -L --no-update-lock-file

            # Dereference symlink (-L) to copy actual file, not symlink
            cp -L /tmp/result "'"$CONTAINER_OUT_DIR/${pkg}.tar.gz"'"

            # Verify file exists and has content
            if [ ! -s "'"$CONTAINER_OUT_DIR/${pkg}.tar.gz"'" ]; then
                echo "ERROR: Output file empty or missing" >&2
                exit 1
            fi

            echo ">>> Build successful: '"$pkg"'"
        ' > >(tee "$OUT_DIR/build.log") 2>&1 &
    DOCKER_PID=$!
    wait $DOCKER_PID
    build_status=$?
    DOCKER_PID=""

    if $INTERRUPTED; then
        break
    fi

    # Verify output file exists on host (defense in depth)
    if [[ $build_status -eq 0 ]] && [[ ! -s "$OUT_DIR/${pkg}.tar.gz" ]]; then
        log_error "Build reported success but output file missing or empty: $OUT_DIR/${pkg}.tar.gz"
        build_status=1
    fi

    if [[ $build_status -eq 0 ]]; then
        rm -f "$OUT_DIR/build.log"  # Remove log on success

        # Load into Docker
        log_step "Loading $pkg into Docker..."
        LOAD_OUTPUT=$(docker load < "$OUT_DIR/${pkg}.tar.gz" 2>&1)
        if [[ $? -eq 0 ]]; then
            IMAGE_TAG=$(echo "$LOAD_OUTPUT" | sed -n 's/.*Loaded image: //p' | awk '{print $1}' | tail -1)
            [[ -z "$IMAGE_TAG" ]] && IMAGE_TAG="(unknown)"
            log_ok "$pkg -> $IMAGE_TAG"
            SUCCEEDED=$((SUCCEEDED + 1))
        else
            log_error "Failed to load $pkg into Docker"
            FAILED=$((FAILED + 1))
            FAILED_CONTAINERS+=("$pkg")
        fi
    else
        log_error "FAILED: $pkg (see $OUT_DIR/build.log)"
        FAILED=$((FAILED + 1))
        FAILED_CONTAINERS+=("$pkg")
    fi
done

BUILD_END=$(date +%s)
BUILD_DURATION=$((BUILD_END - BUILD_START))

# ── Summary ──────────────────────────────────────────────────────────
echo ""
printf "${BOLD}${CYAN}"
echo "  ╔══════════════════════════════════════════════════╗"
printf "  ║  %-48s║\n" "BUILD SUMMARY"
echo "  ╠══════════════════════════════════════════════════╣"
if [[ "$FAILED" -eq 0 ]]; then
    printf "  ║  ${GREEN}Succeeded${CYAN}   %-38s║\n" "$SUCCEEDED"
else
    printf "  ║  ${GREEN}Succeeded${CYAN}   %-38s║\n" "$SUCCEEDED"
    printf "  ║  ${RED}Failed${CYAN}      %-38s║\n" "$FAILED"
fi
printf "  ║  Duration    %-38s║\n" "${BUILD_DURATION}s"
echo "  ╚══════════════════════════════════════════════════╝"
printf "${RESET}\n"

if [[ ${#FAILED_CONTAINERS[@]} -gt 0 ]]; then
    log_error "Failed containers: ${FAILED_CONTAINERS[*]}"
    exit 1
fi

exit 0
