#!/bin/bash
# Copyright Firestream. MIT License.
# Start JupyterHub service
#
# shellcheck disable=SC1090,SC1091

set -o errexit
set -o nounset
set -o pipefail
# set -o xtrace # Uncomment for debugging

# Load JupyterHub environment
. /opt/bitnami/scripts/jupyterhub-env.sh

# Load libraries
. /opt/bitnami/scripts/libjupyterhub.sh

declare -a args=(
    "--config=${JUPYTERHUB_CONF_FILE}"
    "--pid-file=${JUPYTERHUB_PID_FILE}"
)

# Add log file if logging to file is enabled
if [[ -n "${JUPYTERHUB_LOG_FILE:-}" ]] && [[ "${JUPYTERHUB_LOG_TO_FILE:-no}" == "yes" ]]; then
    args+=("--log-file=${JUPYTERHUB_LOG_FILE}")
fi

info "** Starting JupyterHub **"

if am_i_root; then
    exec_as_user "$JUPYTERHUB_DAEMON_USER" jupyterhub "${args[@]}" "$@"
else
    exec jupyterhub "${args[@]}" "$@"
fi
