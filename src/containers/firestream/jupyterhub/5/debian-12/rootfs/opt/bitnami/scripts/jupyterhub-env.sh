#!/bin/bash
# Copyright Firestream. Apache-2.0 License.
# JupyterHub environment variables
#
# This file is sourced by entrypoint scripts to set default environment variables.
# All variables can be overridden by setting them before the container starts.
#
# shellcheck disable=SC2034

# Module identification
export MODULE="jupyterhub"
export BITNAMI_DEBUG="${BITNAMI_DEBUG:-false}"

########################
# Path configuration
########################

# Base directories
export BITNAMI_ROOT_DIR="/opt/bitnami"
export JUPYTERHUB_BASE_DIR="${JUPYTERHUB_BASE_DIR:-/opt/bitnami/jupyterhub}"
export JUPYTERHUB_BIN_DIR="${JUPYTERHUB_BIN_DIR:-/opt/nix-env/bin}"

# Configuration directories
export JUPYTERHUB_CONF_DIR="${JUPYTERHUB_CONF_DIR:-${JUPYTERHUB_BASE_DIR}/etc}"
export JUPYTERHUB_CONF_FILE="${JUPYTERHUB_CONF_FILE:-${JUPYTERHUB_CONF_DIR}/jupyterhub_config.py}"

# Data directories
export JUPYTERHUB_DATA_DIR="${JUPYTERHUB_DATA_DIR:-${JUPYTERHUB_BASE_DIR}/data}"
export JUPYTERHUB_TMP_DIR="${JUPYTERHUB_TMP_DIR:-${JUPYTERHUB_BASE_DIR}/tmp}"
export JUPYTERHUB_PID_FILE="${JUPYTERHUB_PID_FILE:-${JUPYTERHUB_TMP_DIR}/jupyterhub.pid}"

# Log directories
export JUPYTERHUB_LOGS_DIR="${JUPYTERHUB_LOGS_DIR:-${JUPYTERHUB_BASE_DIR}/log}"
export JUPYTERHUB_LOG_FILE="${JUPYTERHUB_LOG_FILE:-${JUPYTERHUB_LOGS_DIR}/jupyterhub.log}"
export JUPYTERHUB_LOG_TO_FILE="${JUPYTERHUB_LOG_TO_FILE:-no}"

# Volume paths (for persistent storage)
export JUPYTERHUB_VOLUME_DIR="${JUPYTERHUB_VOLUME_DIR:-/bitnami/jupyterhub}"
export JUPYTERHUB_DATA_TO_PERSIST="${JUPYTERHUB_DATA_TO_PERSIST:-${JUPYTERHUB_CONF_DIR} ${JUPYTERHUB_DATA_DIR}}"

########################
# System users
########################

export JUPYTERHUB_DAEMON_USER="${JUPYTERHUB_DAEMON_USER:-jupyterhub}"
export JUPYTERHUB_DAEMON_GROUP="${JUPYTERHUB_DAEMON_GROUP:-jupyterhub}"

########################
# Network configuration
########################

export JUPYTERHUB_PROXY_PORT_NUMBER="${JUPYTERHUB_PROXY_PORT_NUMBER:-8000}"
export JUPYTERHUB_API_PORT_NUMBER="${JUPYTERHUB_API_PORT_NUMBER:-8081}"

########################
# Bootstrap configuration
########################

export JUPYTERHUB_SKIP_BOOTSTRAP="${JUPYTERHUB_SKIP_BOOTSTRAP:-no}"

########################
# Admin credentials
########################

export JUPYTERHUB_USERNAME="${JUPYTERHUB_USERNAME:-user}"
export JUPYTERHUB_PASSWORD="${JUPYTERHUB_PASSWORD:-}"

########################
# Database configuration
########################

export JUPYTERHUB_DATABASE_TYPE="${JUPYTERHUB_DATABASE_TYPE:-postgresql}"
export JUPYTERHUB_DATABASE_HOST="${JUPYTERHUB_DATABASE_HOST:-postgresql}"
export JUPYTERHUB_DATABASE_PORT_NUMBER="${JUPYTERHUB_DATABASE_PORT_NUMBER:-5432}"
export JUPYTERHUB_DATABASE_NAME="${JUPYTERHUB_DATABASE_NAME:-bitnami_jupyterhub}"
export JUPYTERHUB_DATABASE_USER="${JUPYTERHUB_DATABASE_USER:-bn_jupyterhub}"
export JUPYTERHUB_DATABASE_PASSWORD="${JUPYTERHUB_DATABASE_PASSWORD:-}"

########################
# Spawner configuration
########################

# Options: localprocess, simple, docker, kubernetes
export JUPYTERHUB_SPAWNER="${JUPYTERHUB_SPAWNER:-localprocess}"

########################
# Authenticator configuration
########################

# Options: pam, dummy, oauth, ldap
export JUPYTERHUB_AUTHENTICATOR="${JUPYTERHUB_AUTHENTICATOR:-pam}"

########################
# Timeouts and limits
########################

export JUPYTERHUB_DB_WAIT_TIMEOUT="${JUPYTERHUB_DB_WAIT_TIMEOUT:-120}"

########################
# Security
########################

export ALLOW_EMPTY_PASSWORD="${ALLOW_EMPTY_PASSWORD:-no}"

########################
# OAuth configuration (when JUPYTERHUB_AUTHENTICATOR=oauth)
########################

export JUPYTERHUB_OAUTH_CLIENT_ID="${JUPYTERHUB_OAUTH_CLIENT_ID:-}"
export JUPYTERHUB_OAUTH_CLIENT_SECRET="${JUPYTERHUB_OAUTH_CLIENT_SECRET:-}"
export JUPYTERHUB_OAUTH_CALLBACK_URL="${JUPYTERHUB_OAUTH_CALLBACK_URL:-}"
export JUPYTERHUB_OAUTH_TOKEN_URL="${JUPYTERHUB_OAUTH_TOKEN_URL:-}"
export JUPYTERHUB_OAUTH_AUTHORIZE_URL="${JUPYTERHUB_OAUTH_AUTHORIZE_URL:-}"
export JUPYTERHUB_OAUTH_USERDATA_URL="${JUPYTERHUB_OAUTH_USERDATA_URL:-}"

########################
# LDAP configuration (when JUPYTERHUB_AUTHENTICATOR=ldap)
########################

export JUPYTERHUB_LDAP_SERVER="${JUPYTERHUB_LDAP_SERVER:-}"
export JUPYTERHUB_LDAP_BIND_DN="${JUPYTERHUB_LDAP_BIND_DN:-}"
export JUPYTERHUB_LDAP_BIND_PASSWORD="${JUPYTERHUB_LDAP_BIND_PASSWORD:-}"
export JUPYTERHUB_LDAP_USER_SEARCH_BASE="${JUPYTERHUB_LDAP_USER_SEARCH_BASE:-}"

########################
# Kubernetes spawner configuration (when JUPYTERHUB_SPAWNER=kubernetes)
########################

export JUPYTERHUB_K8S_NAMESPACE="${JUPYTERHUB_K8S_NAMESPACE:-default}"
export JUPYTERHUB_K8S_IMAGE="${JUPYTERHUB_K8S_IMAGE:-jupyter/base-notebook:latest}"
export JUPYTERHUB_K8S_STORAGE_CLASS="${JUPYTERHUB_K8S_STORAGE_CLASS:-}"
export JUPYTERHUB_K8S_STORAGE_SIZE="${JUPYTERHUB_K8S_STORAGE_SIZE:-1Gi}"

########################
# Docker spawner configuration (when JUPYTERHUB_SPAWNER=docker)
########################

export JUPYTERHUB_DOCKER_IMAGE="${JUPYTERHUB_DOCKER_IMAGE:-jupyter/base-notebook:latest}"
export JUPYTERHUB_DOCKER_NETWORK="${JUPYTERHUB_DOCKER_NETWORK:-bridge}"
