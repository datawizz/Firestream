# Redis Environment Variable Defaults
# Copyright Firestream. MIT License.
# This file is sourced, not executed directly
#
# Documents all environment variables with their default values.
# These defaults are also set in module.nix envVars.

# Paths (set by Nix, documented here for reference)
export REDIS_BASE_DIR="${REDIS_BASE_DIR:-/opt/firestream/redis}"
export REDIS_CONF_DIR="${REDIS_CONF_DIR:-${REDIS_BASE_DIR}/etc}"
export REDIS_DEFAULT_CONF_DIR="${REDIS_DEFAULT_CONF_DIR:-${REDIS_BASE_DIR}/etc.default}"
export REDIS_DATA_DIR="${REDIS_DATA_DIR:-/firestream/redis/data}"
export REDIS_MOUNTED_CONF_DIR="${REDIS_MOUNTED_CONF_DIR:-${REDIS_BASE_DIR}/mounted-etc}"
export REDIS_OVERRIDES_FILE="${REDIS_OVERRIDES_FILE:-${REDIS_MOUNTED_CONF_DIR}/overrides.conf}"
export REDIS_CONF_FILE="${REDIS_CONF_FILE:-${REDIS_CONF_DIR}/redis.conf}"
export REDIS_LOG_DIR="${REDIS_LOG_DIR:-${REDIS_BASE_DIR}/logs}"
export REDIS_LOG_FILE="${REDIS_LOG_FILE:-${REDIS_LOG_DIR}/redis.log}"
export REDIS_TMP_DIR="${REDIS_TMP_DIR:-${REDIS_BASE_DIR}/tmp}"
export REDIS_PID_FILE="${REDIS_PID_FILE:-${REDIS_TMP_DIR}/redis.pid}"
export REDIS_BIN_DIR="${REDIS_BIN_DIR:-${REDIS_BASE_DIR}/bin}"
export REDIS_VOLUME_DIR="${REDIS_VOLUME_DIR:-/firestream/redis}"

# System users (when running with a privileged user)
export REDIS_DAEMON_USER="${REDIS_DAEMON_USER:-redis}"
export REDIS_DAEMON_GROUP="${REDIS_DAEMON_GROUP:-redis}"

# Redis settings
export REDIS_PORT_NUMBER="${REDIS_PORT_NUMBER:-6379}"
export REDIS_PASSWORD="${REDIS_PASSWORD:-}"
export REDIS_DATABASE="${REDIS_DATABASE:-redis}"
export REDIS_DISABLE_COMMANDS="${REDIS_DISABLE_COMMANDS:-}"
export REDIS_EXTRA_FLAGS="${REDIS_EXTRA_FLAGS:-}"
export ALLOW_EMPTY_PASSWORD="${ALLOW_EMPTY_PASSWORD:-no}"

# Persistence settings
export REDIS_AOF_ENABLED="${REDIS_AOF_ENABLED:-yes}"
export REDIS_RDB_POLICY="${REDIS_RDB_POLICY:-}"
export REDIS_RDB_POLICY_DISABLED="${REDIS_RDB_POLICY_DISABLED:-no}"

# Replication settings
export REDIS_REPLICATION_MODE="${REDIS_REPLICATION_MODE:-}"
export REDIS_MASTER_HOST="${REDIS_MASTER_HOST:-}"
export REDIS_MASTER_PORT_NUMBER="${REDIS_MASTER_PORT_NUMBER:-6379}"
export REDIS_MASTER_PASSWORD="${REDIS_MASTER_PASSWORD:-}"
export REDIS_REPLICA_IP="${REDIS_REPLICA_IP:-}"
export REDIS_REPLICA_PORT="${REDIS_REPLICA_PORT:-}"

# Multi-threading settings
export REDIS_IO_THREADS="${REDIS_IO_THREADS:-}"
export REDIS_IO_THREADS_DO_READS="${REDIS_IO_THREADS_DO_READS:-}"

# ACL settings
export REDIS_ACLFILE="${REDIS_ACLFILE:-}"

# TLS settings
export REDIS_TLS_ENABLED="${REDIS_TLS_ENABLED:-no}"
export REDIS_TLS_PORT_NUMBER="${REDIS_TLS_PORT_NUMBER:-6379}"
export REDIS_TLS_CERT_FILE="${REDIS_TLS_CERT_FILE:-}"
export REDIS_TLS_KEY_FILE="${REDIS_TLS_KEY_FILE:-}"
export REDIS_TLS_KEY_FILE_PASS="${REDIS_TLS_KEY_FILE_PASS:-}"
export REDIS_TLS_CA_FILE="${REDIS_TLS_CA_FILE:-}"
export REDIS_TLS_CA_DIR="${REDIS_TLS_CA_DIR:-}"
export REDIS_TLS_DH_PARAMS_FILE="${REDIS_TLS_DH_PARAMS_FILE:-}"
export REDIS_TLS_AUTH_CLIENTS="${REDIS_TLS_AUTH_CLIENTS:-yes}"

# Sentinel settings (for replica discovery)
export REDIS_SENTINEL_HOST="${REDIS_SENTINEL_HOST:-}"
export REDIS_SENTINEL_PORT_NUMBER="${REDIS_SENTINEL_PORT_NUMBER:-26379}"
export REDIS_SENTINEL_MASTER_NAME="${REDIS_SENTINEL_MASTER_NAME:-}"

# Remote connections
export REDIS_ALLOW_REMOTE_CONNECTIONS="${REDIS_ALLOW_REMOTE_CONNECTIONS:-yes}"

# Debug
export BITNAMI_DEBUG="${BITNAMI_DEBUG:-false}"
