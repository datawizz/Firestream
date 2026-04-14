#!/bin/bash
# Copyright Firestream. MIT License.
# Post-init script: PostgreSQL database setup
#
# This script performs any additional PostgreSQL setup after JupyterHub initialization.

# Only run if using PostgreSQL
if [[ "${JUPYTERHUB_DATABASE_TYPE:-postgresql}" != "postgresql" ]]; then
    debug "Not using PostgreSQL, skipping database post-init"
    return 0
fi

# Verify database connectivity
if [[ -n "${JUPYTERHUB_DATABASE_HOST:-}" ]]; then
    debug "Verifying PostgreSQL connection to ${JUPYTERHUB_DATABASE_HOST}:${JUPYTERHUB_DATABASE_PORT_NUMBER:-5432}"

    if wait-for-port --host "${JUPYTERHUB_DATABASE_HOST}" --timeout 5 "${JUPYTERHUB_DATABASE_PORT_NUMBER:-5432}" 2>/dev/null; then
        debug "PostgreSQL connection verified"
    else
        warn "PostgreSQL may not be reachable - service might fail to start"
    fi
fi
