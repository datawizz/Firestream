# Odoo environment defaults
# Copyright Firestream. MIT License.
# This file is sourced, not executed directly
#
# These defaults are applied if variables are not set.
# The module.nix envVars attribute provides the canonical defaults.

# Paths
export ODOO_BASE_DIR="${ODOO_BASE_DIR:-/opt/bitnami/odoo}"
export ODOO_BIN_DIR="${ODOO_BIN_DIR:-${ODOO_BASE_DIR}/bin}"
export ODOO_CONF_DIR="${ODOO_CONF_DIR:-${ODOO_BASE_DIR}/conf}"
export ODOO_CONF_FILE="${ODOO_CONF_FILE:-${ODOO_CONF_DIR}/odoo.conf}"
export ODOO_DATA_DIR="${ODOO_DATA_DIR:-${ODOO_BASE_DIR}/data}"
export ODOO_ADDONS_DIR="${ODOO_ADDONS_DIR:-${ODOO_BASE_DIR}/addons}"
export ODOO_TMP_DIR="${ODOO_TMP_DIR:-${ODOO_BASE_DIR}/tmp}"
export ODOO_PID_FILE="${ODOO_PID_FILE:-${ODOO_TMP_DIR}/odoo.pid}"
export ODOO_LOGS_DIR="${ODOO_LOGS_DIR:-${ODOO_BASE_DIR}/log}"
export ODOO_LOG_FILE="${ODOO_LOG_FILE:-${ODOO_LOGS_DIR}/odoo-server.log}"

# Volume paths
export ODOO_VOLUME_DIR="${ODOO_VOLUME_DIR:-/bitnami/odoo}"
export ODOO_DATA_TO_PERSIST="${ODOO_DATA_TO_PERSIST:-${ODOO_ADDONS_DIR} ${ODOO_CONF_DIR} ${ODOO_DATA_DIR}}"

# System users
export ODOO_DAEMON_USER="${ODOO_DAEMON_USER:-odoo}"
export ODOO_DAEMON_GROUP="${ODOO_DAEMON_GROUP:-odoo}"

# Port configuration
export ODOO_PORT_NUMBER="${ODOO_PORT_NUMBER:-8069}"
export ODOO_LONGPOLLING_PORT_NUMBER="${ODOO_LONGPOLLING_PORT_NUMBER:-8072}"

# Bootstrap configuration
export ODOO_SKIP_BOOTSTRAP="${ODOO_SKIP_BOOTSTRAP:-no}"
export ODOO_SKIP_MODULES_UPDATE="${ODOO_SKIP_MODULES_UPDATE:-no}"
export ODOO_LOAD_DEMO_DATA="${ODOO_LOAD_DEMO_DATA:-no}"
export ODOO_LIST_DB="${ODOO_LIST_DB:-no}"

# Odoo credentials
export ODOO_EMAIL="${ODOO_EMAIL:-user@example.com}"
export ODOO_PASSWORD="${ODOO_PASSWORD:-bitnami}"

# SMTP configuration (optional)
export ODOO_SMTP_HOST="${ODOO_SMTP_HOST:-}"
export ODOO_SMTP_PORT_NUMBER="${ODOO_SMTP_PORT_NUMBER:-}"
export ODOO_SMTP_USER="${ODOO_SMTP_USER:-}"
export ODOO_SMTP_PASSWORD="${ODOO_SMTP_PASSWORD:-}"
export ODOO_SMTP_PROTOCOL="${ODOO_SMTP_PROTOCOL:-}"

# Database configuration
export ODOO_DATABASE_HOST="${ODOO_DATABASE_HOST:-postgresql}"
export ODOO_DATABASE_PORT_NUMBER="${ODOO_DATABASE_PORT_NUMBER:-5432}"
export ODOO_DATABASE_NAME="${ODOO_DATABASE_NAME:-bitnami_odoo}"
export ODOO_DATABASE_USER="${ODOO_DATABASE_USER:-bn_odoo}"
export ODOO_DATABASE_PASSWORD="${ODOO_DATABASE_PASSWORD:-}"
export ODOO_DATABASE_FILTER="${ODOO_DATABASE_FILTER:-}"

# Allow empty password (development only)
export ALLOW_EMPTY_PASSWORD="${ALLOW_EMPTY_PASSWORD:-no}"

# Debug mode
export BITNAMI_DEBUG="${BITNAMI_DEBUG:-false}"
