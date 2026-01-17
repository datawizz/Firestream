#!/usr/bin/env bash
# Centralized container build script for Firestream
# Supports cross-platform builds (Linux on macOS via Docker)
#
# Copyright Firestream. Apache-2.0 License.

set -euo pipefail

# Find repo root (works in container at /workspace or on host)
find_repo_root() {
    local dir="${1:-$(pwd)}"
    while [[ "$dir" != "/" ]]; do
        if [[ -f "$dir/CLAUDE.md" && -d "$dir/src/containers/firestream" ]]; then
            echo "$dir"
            return 0
        fi
        dir="$(dirname "$dir")"
    done
    # Fallback: check common container mount point
    if [[ -d "/workspace/src/containers/firestream" ]]; then
        echo "/workspace"
        return 0
    fi
    echo "Error: Could not find repo root" >&2
    return 1
}

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(find_repo_root "$SCRIPT_DIR")"
CONTAINERS_DIR="$REPO_ROOT/src/containers/firestream"

# Detect platform
OS_PLATFORM="$(uname -s)"
CPU_ARCH="$(uname -m)"

# Usage
usage() {
    echo "Usage: $0 <container> [options]"
    echo ""
    echo "Cross-platform container build script for Firestream."
    echo "Automatically detects platform and uses Docker for Nix builds on macOS."
    echo ""
    echo "Containers:"
    ls -1 "$CONTAINERS_DIR" 2>/dev/null | grep -v "^\." || echo "  (none found)"
    echo ""
    echo "Options:"
    echo "  --target <arch>    Target architecture (x86_64, aarch64)"
    echo "  --force-docker     Force Docker-based Nix build (even on Linux)"
    echo "  --force-nix        Force native Nix build (fails on macOS)"
    echo "  --dockerfile       Use Dockerfile instead of Nix"
    echo "  --version <ver>    Container version (for multi-version containers)"
    echo "  --help             Show this help"
    echo ""
    echo "Examples:"
    echo "  $0 airflow                    # Auto-detect build method"
    echo "  $0 postgresql --version 17    # Build specific version"
    echo "  $0 kafka --dockerfile         # Force Dockerfile build"
    echo "  $0 airflow --target aarch64   # Cross-compile for ARM64"
    exit 1
}

# Check if container uses Nix
is_nix_container() {
    local container="$1"
    [[ -f "$CONTAINERS_DIR/$container/flake.nix" ]]
}

# Extract version from flake.nix
get_container_version() {
    local container="$1"
    local flake_file="$CONTAINERS_DIR/$container/flake.nix"
    if [[ -f "$flake_file" ]]; then
        # Try different patterns for version extraction (POSIX-compatible for macOS)
        sed -n 's/.*airflowVersion[[:space:]]*=[[:space:]]*"\([^"]*\)".*/\1/p' "$flake_file" | head -1 || \
        sed -n 's/.*version[[:space:]]*=[[:space:]]*"\([^"]*\)".*/\1/p' "$flake_file" | head -1 || \
        echo "latest"
    else
        echo "latest"
    fi
}

# Build with Nix (native Linux)
build_nix_native() {
    local container="$1"
    local container_dir="$CONTAINERS_DIR/$container"

    echo "==> Building $container with native Nix..."
    cd "$container_dir"
    nix build .#dockerImage

    echo "==> Loading image into Docker..."
    docker load < result

    echo "==> Build complete (image tag set by Nix)"
}

# Build with Nix via Docker (for macOS)
# Uses a named Docker volume to persist Nix store between builds.
# The volume preserves compiled derivations for faster subsequent builds.
# See: https://kevincox.ca/2022/01/02/nix-in-docker-caching/
build_nix_docker() {
    local container="$1"
    local target_arch="${2:-$CPU_ARCH}"

    echo "==> Building $container with Nix inside Docker (target: $target_arch)..."

    # Map architecture to Docker platform and volume name
    local docker_platform
    local nix_volume
    case "$target_arch" in
        x86_64|amd64)
            docker_platform="linux/amd64"
            nix_volume="firestream-nix-store-amd64"
            ;;
        aarch64|arm64)
            docker_platform="linux/arm64"
            nix_volume="firestream-nix-store-arm64"
            ;;
        *) echo "Error: Unknown architecture: $target_arch"; exit 1 ;;
    esac

    local tmp_image="/tmp/nix-image-$container-$$.tar.gz"

    echo "==> Running Nix build in Docker container..."
    echo "    Platform: $docker_platform"
    echo "    Nix store volume: $nix_volume"

    # Run Nix build inside Docker with persistent Nix store
    # The named volume persists compiled derivations between builds
    docker run --rm \
        -v "$REPO_ROOT:/workspace" \
        --mount type=volume,source="$nix_volume",target=/nix \
        -w "/workspace/src/containers/firestream/$container" \
        --platform "$docker_platform" \
        nixos/nix:latest \
        sh -c '
            echo "experimental-features = nix-command flakes" >> /etc/nix/nix.conf
            echo "==> Running nix build..." >&2
            image_path=$(nix build .#dockerImage --no-link --print-out-paths)
            echo "==> Outputting image from: $image_path" >&2
            cat "$image_path"
        ' > "$tmp_image"

    # Load the image
    echo "==> Loading image into Docker..."
    docker load < "$tmp_image"
    rm -f "$tmp_image"

    echo "==> Build complete (image tag set by Nix)"
}

# Build with Dockerfile
build_dockerfile() {
    local container="$1"
    local version="${2:-}"
    local container_dir="$CONTAINERS_DIR/$container"

    # Find Dockerfile
    local dockerfile=""
    if [[ -n "$version" && -f "$container_dir/$version/debian-12/Dockerfile" ]]; then
        dockerfile="$container_dir/$version/debian-12/Dockerfile"
    elif [[ -f "$container_dir/Dockerfile" ]]; then
        dockerfile="$container_dir/Dockerfile"
    else
        # Find first version directory with Dockerfile
        dockerfile=$(find "$container_dir" -name "Dockerfile" -path "*/debian-12/*" 2>/dev/null | sort -V | tail -1)
    fi

    if [[ -z "$dockerfile" || ! -f "$dockerfile" ]]; then
        echo "Error: No Dockerfile found for $container"
        echo "Searched in: $container_dir"
        exit 1
    fi

    local tag="${version:-latest}"
    echo "==> Building $container with Dockerfile: $dockerfile"
    docker build -t "firestream-$container:$tag" -f "$dockerfile" "$container_dir"
}

# Main
main() {
    [[ $# -lt 1 ]] && usage

    local container=""
    local target_arch="$CPU_ARCH"
    local force_docker=false
    local force_nix=false
    local use_dockerfile=false
    local version=""

    # Parse arguments
    while [[ $# -gt 0 ]]; do
        case "$1" in
            --target) target_arch="$2"; shift 2 ;;
            --force-docker) force_docker=true; shift ;;
            --force-nix) force_nix=true; shift ;;
            --dockerfile) use_dockerfile=true; shift ;;
            --version) version="$2"; shift 2 ;;
            --help|-h) usage ;;
            -*) echo "Unknown option: $1"; usage ;;
            *) container="$1"; shift ;;
        esac
    done

    [[ -z "$container" ]] && usage

    # Validate container exists
    if [[ ! -d "$CONTAINERS_DIR/$container" ]]; then
        echo "Error: Container '$container' not found in $CONTAINERS_DIR"
        echo ""
        usage
    fi

    echo "==> Firestream Container Builder"
    echo "    Container: $container"
    echo "    Platform: $OS_PLATFORM ($CPU_ARCH)"
    echo "    Repo root: $REPO_ROOT"
    echo ""

    # Determine build method
    if $use_dockerfile; then
        build_dockerfile "$container" "$version"
    elif is_nix_container "$container"; then
        if [[ "$OS_PLATFORM" == "Linux" ]] && ! $force_docker; then
            build_nix_native "$container"
        elif $force_nix; then
            echo "Error: Native Nix builds only work on Linux"
            echo "Hint: Remove --force-nix to use Docker-based build on macOS"
            exit 1
        else
            build_nix_docker "$container" "$target_arch"
        fi
    else
        build_dockerfile "$container" "$version"
    fi

    echo ""
    echo "==> Build complete: firestream-$container"
}

main "$@"
