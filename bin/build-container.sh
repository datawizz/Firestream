#!/usr/bin/env bash
# build-container.sh - Wrapper for container-images.sh
#
# Maintains backward compatibility with existing usage.
# Delegates to the new build infrastructure with proper git worktree support.
#
# Copyright Firestream. MIT License.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"

# Forward to new build system
exec "$SCRIPT_DIR/build/container-images.sh" "$@"
