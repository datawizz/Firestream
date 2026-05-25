#!/usr/bin/env bash
# manifest.sh - Build fleet SBOM manifest via Nix in container
#
# Builds the fleet manifest inside Docker with volume-based Nix cache.
# Supports git worktrees by mounting git directories at original paths.
#
# Usage:
#   ./bin/build/manifest.sh              # Build fleet manifest
#   ./bin/build/manifest.sh airflow      # Build individual SBOM
#
# Copyright Firestream. MIT License.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
source "$SCRIPT_DIR/_common.sh"

TARGET_ARCH="${TARGET_ARCH:-$(uname -m)}"

# ── Usage ────────────────────────────────────────────────────────────
usage() {
    echo ""
    printf "${BOLD}Usage:${RESET} $0 [container]\n"
    echo ""
    echo "  Build fleet SBOM manifest using Nix."
    echo "  Automatically runs builds inside Docker with persistent Nix cache."
    echo ""
    printf "${BOLD}Commands:${RESET}\n"
    printf "  ${CYAN}(no args)${RESET}     Build complete fleet manifest\n"
    printf "  ${CYAN}<container>${RESET}   Build SBOM for specific container\n"
    echo ""
    printf "${BOLD}Examples:${RESET}\n"
    printf "  ${DIM}\$${RESET} $0                    ${DIM}# Build fleet manifest${RESET}\n"
    printf "  ${DIM}\$${RESET} $0 airflow            ${DIM}# Build airflow SBOM only${RESET}\n"
    printf "  ${DIM}\$${RESET} $0 spark              ${DIM}# Build spark SBOM only${RESET}\n"
    echo ""
    exit 1
}

# ── Parse Arguments ──────────────────────────────────────────────────
CONTAINER=""
while [[ $# -gt 0 ]]; do
    case "$1" in
        --help|-h)
            usage
            ;;
        -*)
            log_error "Unknown option: $1"
            exit 1
            ;;
        *)
            CONTAINER="$1"
            shift
            ;;
    esac
done

# Determine what to build
if [[ -z "$CONTAINER" ]]; then
    NIX_TARGET=".#manifest"
    OUTPUT_NAME="manifest"
    log_info "Building fleet manifest"
else
    NIX_TARGET=".#sbom.$CONTAINER"
    OUTPUT_NAME="sbom-$CONTAINER"
    log_info "Building SBOM for: $CONTAINER"
fi

# ── Setup ────────────────────────────────────────────────────────────
OUTPUT_DIR="$BUILD_OUTPUT_DIR/$OUTPUT_NAME"
CONTAINER_OUT_DIR="/build/$OUTPUT_NAME"
mkdir -p "$OUTPUT_DIR"

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
log_step "Output: $OUTPUT_DIR"

if [[ "$GIT_WORKTREE_DETECTED" == "true" ]]; then
    log_step "Worktree detected - mounting git dirs at original paths"
fi

# Build docker run base arguments
docker_args=(
    --rm
    --cpus "$DOCKER_CPUS"
    --memory "$DOCKER_MEMORY"
    --memory-swap "$DOCKER_SWAP"
    --platform "$docker_platform"
    -v "$REPO_ROOT:$REPO_ROOT:ro"
    -v "$BUILD_OUTPUT_DIR:/build"
    --mount "type=volume,source=$NIX_VOLUME,target=/nix"
    -w "$REPO_ROOT"
)

# Add git worktree mounts if detected
if [[ ${#GIT_DOCKER_MOUNTS[@]} -gt 0 ]]; then
    docker_args+=("${GIT_DOCKER_MOUNTS[@]}")
fi

# ── Build ────────────────────────────────────────────────────────────
BUILD_START=$(date +%s)

docker run "${docker_args[@]}" "$BUILDER_TAG" \
    sh -c '
        set -euo pipefail

        echo "experimental-features = nix-command flakes" >> /etc/nix/nix.conf
        git config --global --add safe.directory "'"$REPO_ROOT"'" 2>/dev/null || true

        echo ">>> Building '"$NIX_TARGET"'..."
        nix build "'"$NIX_TARGET"'" -o /tmp/result -L --no-update-lock-file

        # Copy to container-local temp first (Nix store output is read-only;
        # chmod fails on macOS Docker volume mounts, so fix perms locally first)
        rm -rf "'"$CONTAINER_OUT_DIR"'"
        mkdir -p "'"$CONTAINER_OUT_DIR"'"
        cp -rL /tmp/result /tmp/result-writable
        chmod -R u+w /tmp/result-writable
        cp -r /tmp/result-writable/* "'"$CONTAINER_OUT_DIR"'/"
        rm -rf /tmp/result-writable

        echo ">>> Build successful"
    '

build_status=$?
BUILD_END=$(date +%s)
BUILD_DURATION=$((BUILD_END - BUILD_START))

if [[ $build_status -eq 0 ]]; then
    echo ""
    log_ok "Build completed in ${BUILD_DURATION}s"
    log_step "Output: $OUTPUT_DIR"
    echo ""
    ls -la "$OUTPUT_DIR/"
else
    log_error "Build failed after ${BUILD_DURATION}s"
    exit 1
fi
