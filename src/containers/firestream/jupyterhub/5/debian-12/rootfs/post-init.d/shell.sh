#!/bin/bash
# Copyright Firestream. MIT License.
# Post-init script: Shell environment setup
#
# This script sets up shell-related configuration after JupyterHub initialization.

# Set up history file location if needed
export HISTFILE="${JUPYTERHUB_DATA_DIR:-/opt/bitnami/jupyterhub/data}/.bash_history"

# Ensure the history file directory exists
mkdir -p "$(dirname "$HISTFILE")" 2>/dev/null || true

# Log environment summary if debug is enabled
if [[ "${BITNAMI_DEBUG:-false}" == "true" ]]; then
    debug "Environment summary:"
    debug "  JUPYTERHUB_BASE_DIR: ${JUPYTERHUB_BASE_DIR:-not set}"
    debug "  JUPYTERHUB_CONF_FILE: ${JUPYTERHUB_CONF_FILE:-not set}"
    debug "  JUPYTERHUB_DATA_DIR: ${JUPYTERHUB_DATA_DIR:-not set}"
    debug "  JUPYTERHUB_SPAWNER: ${JUPYTERHUB_SPAWNER:-not set}"
    debug "  JUPYTERHUB_AUTHENTICATOR: ${JUPYTERHUB_AUTHENTICATOR:-not set}"
    debug "  PATH: ${PATH}"
fi
