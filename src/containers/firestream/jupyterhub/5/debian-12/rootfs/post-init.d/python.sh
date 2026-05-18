#!/bin/bash
# Copyright Firestream. MIT License.
# Post-init script: Python environment setup
#
# This script sets up Python-related configuration after JupyterHub initialization.

# Ensure Python cache directory is writable
if [[ -n "${PYTHONPYCACHEPREFIX:-}" ]]; then
    mkdir -p "$PYTHONPYCACHEPREFIX" 2>/dev/null || true
fi

# Verify JupyterHub is available
if command -v jupyterhub >/dev/null 2>&1; then
    debug "JupyterHub version: $(jupyterhub --version 2>/dev/null || echo 'unknown')"
else
    warn "JupyterHub command not found in PATH"
fi

# Verify Jupyter Notebook is available (optional dependency)
if command -v jupyter >/dev/null 2>&1; then
    debug "Jupyter version: $(jupyter --version 2>/dev/null | head -1 || echo 'unknown')"
fi
