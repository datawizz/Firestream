#!/bin/bash
# Copyright Firestream. MIT License.
# Post-initialization script
#
# This script runs after JupyterHub initialization but before the service starts.
# It executes all scripts in /post-init.d/ and /docker-entrypoint-init.d/
#
# shellcheck disable=SC1091

set -o errexit
set -o nounset
set -o pipefail

# Load logging functions
. /opt/bitnami/scripts/jupyterhub-env.sh
. /opt/bitnami/scripts/libjupyterhub.sh

debug "Running post-initialization scripts..."

# Run scripts from /post-init.d/ (container built-in scripts)
if [[ -d /post-init.d ]]; then
    for script in /post-init.d/*.sh; do
        if [[ -f "$script" ]] && [[ -x "$script" ]]; then
            info "Running post-init script: $(basename "$script")"
            . "$script"
        elif [[ -f "$script" ]]; then
            info "Sourcing post-init script: $(basename "$script")"
            . "$script"
        fi
    done
fi

# Run scripts from /docker-entrypoint-init.d/ (user-provided scripts via volume mount)
if [[ -d /docker-entrypoint-init.d ]]; then
    for script in /docker-entrypoint-init.d/*.sh; do
        if [[ -f "$script" ]] && [[ -x "$script" ]]; then
            info "Running user init script: $(basename "$script")"
            . "$script"
        elif [[ -f "$script" ]]; then
            info "Sourcing user init script: $(basename "$script")"
            . "$script"
        fi
    done
fi

debug "Post-initialization complete"
