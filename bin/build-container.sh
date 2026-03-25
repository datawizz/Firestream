#!/usr/bin/env bash
# Centralized container build script for Firestream
# Supports cross-platform builds (Linux on macOS via Docker)
#
# Copyright Firestream. Apache-2.0 License.

set -euo pipefail

# ── Colors & Formatting ──────────────────────────────────────────────
if [[ -t 1 ]]; then
    BOLD='\033[1m'
    DIM='\033[2m'
    RESET='\033[0m'
    RED='\033[31m'
    GREEN='\033[32m'
    YELLOW='\033[33m'
    BLUE='\033[34m'
    CYAN='\033[36m'
    WHITE='\033[37m'
else
    BOLD='' DIM='' RESET=''
    RED='' GREEN='' YELLOW='' BLUE='' CYAN='' WHITE=''
fi

_log()    { printf "${DIM}%s${RESET} %b\n" "$(date +%H:%M:%S)" "$*"; }
_info()   { _log "${CYAN}${BOLD}>>>${RESET} $*"; }
_step()   { _log "${BLUE}${BOLD} ->${RESET} $*"; }
_ok()     { _log "${GREEN}${BOLD}  ✓${RESET} $*"; }
_warn()   { _log "${YELLOW}${BOLD}  !${RESET} $*"; }
_err()    { _log "${RED}${BOLD}  ✗${RESET} $*" >&2; }

_header() {
    local name="$1"
    local arch="$2"
    local method="$3"
    echo ""
    printf "${BOLD}${CYAN}"
    echo "  ╔══════════════════════════════════════════════════╗"
    printf "  ║  %-48s║\n" "FIRESTREAM CONTAINER BUILD"
    echo "  ╠══════════════════════════════════════════════════╣"
    printf "  ║  ${WHITE}Container${CYAN}   %-38s║\n" "$name"
    printf "  ║  ${WHITE}Platform${CYAN}    %-38s║\n" "$OS_PLATFORM ($arch)"
    printf "  ║  ${WHITE}Method${CYAN}      %-38s║\n" "$method"
    echo "  ╚══════════════════════════════════════════════════╝"
    printf "${RESET}\n"
}

_footer() {
    local name="$1"
    local tag="$2"
    local elapsed="$3"
    echo ""
    printf "${BOLD}${GREEN}"
    echo "  ┌──────────────────────────────────────────────────┐"
    printf "  │  %-48s│\n" "BUILD COMPLETE"
    printf "  │  ${WHITE}Image${GREEN}   firestream-%-35s│\n" "${name}:${tag}"
    printf "  │  ${WHITE}Time${GREEN}    %-42s│\n" "${elapsed}s"
    echo "  └──────────────────────────────────────────────────┘"
    printf "${RESET}\n"
}

# ── Utilities ─────────────────────────────────────────────────────────

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
    _err "Could not find repo root"
    return 1
}

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(find_repo_root "$SCRIPT_DIR")"
CONTAINERS_DIR="$REPO_ROOT/src/containers/firestream"

# Source reusable functions from docker_preinit.sh (worktree detection, etc.)
# shellcheck source=../docker/firestream/docker_preinit.sh
source "$REPO_ROOT/docker/firestream/docker_preinit.sh"

# Detect platform
OS_PLATFORM="$(uname -s)"
CPU_ARCH="$(uname -m)"

# Track build time
BUILD_START_TIME="${EPOCHSECONDS:-$(date +%s)}"

# Usage
usage() {
    echo ""
    printf "${BOLD}Usage:${RESET} $0 ${CYAN}<container>${RESET} [options]\n"
    echo ""
    echo "  Cross-platform container build script for Firestream."
    echo "  Automatically detects platform and uses Docker for Nix builds on macOS."
    echo ""
    printf "${BOLD}Containers:${RESET}\n"
    ls -1 "$CONTAINERS_DIR" 2>/dev/null | grep -v "^\." | sed 's/^/    /' || echo "    (none found)"
    echo ""
    printf "${BOLD}Options:${RESET}\n"
    printf "  ${CYAN}--target <arch>${RESET}    Target architecture (x86_64, aarch64)\n"
    printf "  ${CYAN}--force-docker${RESET}     Force Docker-based Nix build (even on Linux)\n"
    printf "  ${CYAN}--force-nix${RESET}        Force native Nix build (fails on macOS)\n"
    printf "  ${CYAN}--dockerfile${RESET}       Use Dockerfile instead of Nix\n"
    printf "  ${CYAN}--version <ver>${RESET}    Container version (for multi-version containers)\n"
    printf "  ${CYAN}--help${RESET}             Show this help\n"
    echo ""
    printf "${BOLD}Examples:${RESET}\n"
    printf "  ${DIM}\$${RESET} $0 airflow                    ${DIM}# Auto-detect build method${RESET}\n"
    printf "  ${DIM}\$${RESET} $0 postgresql --version 17    ${DIM}# Build specific version${RESET}\n"
    printf "  ${DIM}\$${RESET} $0 kafka --dockerfile         ${DIM}# Force Dockerfile build${RESET}\n"
    printf "  ${DIM}\$${RESET} $0 airflow --target aarch64   ${DIM}# Cross-compile for ARM64${RESET}\n"
    echo ""
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

    _info "Building ${BOLD}$container${RESET} with native Nix..."
    cd "$container_dir"

    _step "Running ${BOLD}nix build${RESET}..."
    nix build .#dockerImage

    _step "Loading image into Docker..."
    docker load < result
    _ok "Image loaded"
}

# Build with Nix via Docker (for macOS)
# Uses a named Docker volume to persist Nix store between builds.
# The volume preserves compiled derivations for faster subsequent builds.
# See: https://kevincox.ca/2022/01/02/nix-in-docker-caching/
build_nix_docker() {
    local container="$1"
    local target_arch="${2:-$CPU_ARCH}"

    _info "Building ${BOLD}$container${RESET} with Nix inside Docker"

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
        *)
            _err "Unknown architecture: $target_arch"
            exit 1
            ;;
    esac

    # ── Git worktree support ──────────────────────────────────────────
    # When .git is a file (worktree pointer), libgit2 can't follow the
    # absolute host path inside the container. Fix: mount the repo and
    # the main .git dir at their original host paths so every absolute
    # gitdir pointer resolves unchanged.
    detect_git_worktree "$REPO_ROOT"
    local repo_mount work_dir
    if [[ "$GIT_WORKTREE_DETECTED" == "true" ]]; then
        repo_mount=(-v "${REPO_ROOT}:${REPO_ROOT}" -v "${GIT_WORKTREE_COMMON_DIR}:${GIT_WORKTREE_COMMON_DIR}:ro")
        work_dir="${REPO_ROOT}/src/containers/firestream/$container"
        _step "Worktree detected — mounting at original host paths"
    else
        repo_mount=(-v "$REPO_ROOT:/workspace")
        work_dir="/workspace/src/containers/firestream/$container"
    fi

    local tmp_image="/tmp/nix-image-$container-$$.tar.gz"

    _step "Platform: ${BOLD}$docker_platform${RESET}"
    _step "Nix store: ${BOLD}$nix_volume${RESET}"
    _step "Running ${BOLD}nix build${RESET} in container..."

    # Run Nix build inside Docker with persistent Nix store
    # The named volume persists compiled derivations between builds
    docker run --rm \
        "${repo_mount[@]}" \
        --mount type=volume,source="$nix_volume",target=/nix \
        -w "$work_dir" \
        --platform "$docker_platform" \
        nixos/nix:latest \
        sh -c '
            echo "experimental-features = nix-command flakes" >> /etc/nix/nix.conf
            image_path=$(nix build .#dockerImage --no-link --print-out-paths)
            echo ">>> $image_path" >&2
            cat "$image_path"
        ' > "$tmp_image"

    _step "Loading image into Docker..."
    docker load < "$tmp_image"
    rm -f "$tmp_image"
    _ok "Image loaded"
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
        _err "No Dockerfile found for $container"
        _step "Searched in: $container_dir"
        exit 1
    fi

    local tag="${version:-latest}"
    _info "Building ${BOLD}$container${RESET} with Dockerfile"
    _step "Dockerfile: ${DIM}$dockerfile${RESET}"

    docker build -t "firestream-$container:$tag" -f "$dockerfile" "$container_dir"
    _ok "Image built: firestream-$container:$tag"
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
        _err "Container '${BOLD}$container${RESET}${RED}' not found in $CONTAINERS_DIR"
        echo ""
        usage
    fi

    # Determine build method label
    local method="auto"
    if $use_dockerfile; then
        method="Dockerfile"
    elif is_nix_container "$container"; then
        if [[ "$OS_PLATFORM" == "Linux" ]] && ! $force_docker; then
            method="Nix (native)"
        elif $force_nix; then
            _err "Native Nix builds only work on Linux"
            _step "Remove ${BOLD}--force-nix${RESET} to use Docker-based build on macOS"
            exit 1
        else
            method="Nix (via Docker)"
        fi
    else
        method="Dockerfile"
    fi

    _header "$container" "$target_arch" "$method"

    # Run the build
    if $use_dockerfile; then
        build_dockerfile "$container" "$version"
    elif is_nix_container "$container"; then
        if [[ "$OS_PLATFORM" == "Linux" ]] && ! $force_docker; then
            build_nix_native "$container"
        else
            build_nix_docker "$container" "$target_arch"
        fi
    else
        build_dockerfile "$container" "$version"
    fi

    # Calculate elapsed time
    local end_time="${EPOCHSECONDS:-$(date +%s)}"
    local elapsed=$(( end_time - BUILD_START_TIME ))

    # Get image tag
    local image_tag
    image_tag=$(docker images "firestream-$container" --format '{{.Tag}}' | head -1)
    image_tag="${image_tag:-latest}"

    _footer "$container" "$image_tag" "$elapsed"
}

main "$@"
