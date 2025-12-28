# JupyterHub environment defaults
# Copyright Firestream. Apache-2.0 License.
# This file is sourced, not executed directly
#
# These defaults are applied if variables are not set.
# The module.nix envVars attribute provides the canonical defaults.

# Paths
export JUPYTERHUB_BASE_DIR="${JUPYTERHUB_BASE_DIR:-/opt/bitnami/jupyterhub}"
export JUPYTERHUB_BIN_DIR="${JUPYTERHUB_BIN_DIR:-/opt/bitnami/miniforge/bin}"
export JUPYTERHUB_CONF_DIR="${JUPYTERHUB_CONF_DIR:-${JUPYTERHUB_BASE_DIR}/etc}"
export JUPYTERHUB_CONF_FILE="${JUPYTERHUB_CONF_FILE:-${JUPYTERHUB_CONF_DIR}/jupyterhub_config.py}"
export JUPYTERHUB_DATA_DIR="${JUPYTERHUB_DATA_DIR:-${JUPYTERHUB_BASE_DIR}/data}"
export JUPYTERHUB_TMP_DIR="${JUPYTERHUB_TMP_DIR:-${JUPYTERHUB_BASE_DIR}/tmp}"
export JUPYTERHUB_PID_FILE="${JUPYTERHUB_PID_FILE:-${JUPYTERHUB_TMP_DIR}/jupyterhub.pid}"
export JUPYTERHUB_LOGS_DIR="${JUPYTERHUB_LOGS_DIR:-${JUPYTERHUB_BASE_DIR}/log}"
export JUPYTERHUB_LOG_FILE="${JUPYTERHUB_LOG_FILE:-${JUPYTERHUB_LOGS_DIR}/jupyterhub.log}"

# Volume paths
export JUPYTERHUB_VOLUME_DIR="${JUPYTERHUB_VOLUME_DIR:-/bitnami/jupyterhub}"
export JUPYTERHUB_DATA_TO_PERSIST="${JUPYTERHUB_DATA_TO_PERSIST:-${JUPYTERHUB_CONF_DIR} ${JUPYTERHUB_DATA_DIR}}"

# System users
export JUPYTERHUB_DAEMON_USER="${JUPYTERHUB_DAEMON_USER:-jupyterhub}"
export JUPYTERHUB_DAEMON_GROUP="${JUPYTERHUB_DAEMON_GROUP:-jupyterhub}"

# Port configuration
export JUPYTERHUB_PROXY_PORT_NUMBER="${JUPYTERHUB_PROXY_PORT_NUMBER:-8000}"
export JUPYTERHUB_API_PORT_NUMBER="${JUPYTERHUB_API_PORT_NUMBER:-8081}"

# Bootstrap configuration
export JUPYTERHUB_SKIP_BOOTSTRAP="${JUPYTERHUB_SKIP_BOOTSTRAP:-no}"

# Admin credentials
export JUPYTERHUB_USERNAME="${JUPYTERHUB_USERNAME:-user}"
export JUPYTERHUB_PASSWORD="${JUPYTERHUB_PASSWORD:-bitnami}"

# Database configuration
export JUPYTERHUB_DATABASE_TYPE="${JUPYTERHUB_DATABASE_TYPE:-postgresql}"
export JUPYTERHUB_DATABASE_HOST="${JUPYTERHUB_DATABASE_HOST:-postgresql}"
export JUPYTERHUB_DATABASE_PORT_NUMBER="${JUPYTERHUB_DATABASE_PORT_NUMBER:-5432}"
export JUPYTERHUB_DATABASE_NAME="${JUPYTERHUB_DATABASE_NAME:-bitnami_jupyterhub}"
export JUPYTERHUB_DATABASE_USER="${JUPYTERHUB_DATABASE_USER:-bn_jupyterhub}"
export JUPYTERHUB_DATABASE_PASSWORD="${JUPYTERHUB_DATABASE_PASSWORD:-}"

# Spawner configuration
export JUPYTERHUB_SPAWNER="${JUPYTERHUB_SPAWNER:-localprocess}"

# Authenticator configuration
export JUPYTERHUB_AUTHENTICATOR="${JUPYTERHUB_AUTHENTICATOR:-pam}"

# Timeouts
export JUPYTERHUB_DB_WAIT_TIMEOUT="${JUPYTERHUB_DB_WAIT_TIMEOUT:-120}"

# Allow empty password (development only)
export ALLOW_EMPTY_PASSWORD="${ALLOW_EMPTY_PASSWORD:-no}"

# Debug mode
export BITNAMI_DEBUG="${BITNAMI_DEBUG:-false}"
