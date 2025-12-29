#!/bin/bash
# Copyright Firestream. Apache-2.0 License.
# SPDX-License-Identifier: APACHE-2.0
#
# Environment defaults for Apache Spark container
# This script sets default values for all Spark configuration variables.
#
# Variables are set using Bash parameter expansion ${VAR:-default}
# meaning existing values from environment/Docker are preserved.

# ============================================================
# Bitnami Compatibility Layer
# Maintains full backward compatibility with existing deployments
# ============================================================

export BITNAMI_ROOT_DIR="${BITNAMI_ROOT_DIR:-/opt/bitnami}"
export BITNAMI_VOLUME_DIR="${BITNAMI_VOLUME_DIR:-/bitnami}"

# Logging configuration (Bitnami pattern)
export MODULE="${MODULE:-spark}"
export BITNAMI_DEBUG="${BITNAMI_DEBUG:-false}"

# ============================================================
# Firestream Paths
# Standard paths for the Firestream container system
# ============================================================

export FIRESTREAM_ROOT_DIR="${FIRESTREAM_ROOT_DIR:-/opt/firestream}"
export FIRESTREAM_VOLUME_DIR="${FIRESTREAM_VOLUME_DIR:-/firestream}"
export FIRESTREAM_APP_NAME="spark"

# ============================================================
# Spark Paths
# Core directory structure for Apache Spark
# ============================================================

# Base installation (Nix package location)
export SPARK_HOME="${SPARK_HOME:-/opt/spark}"
export SPARK_BASE_DIR="${SPARK_BASE_DIR:-${SPARK_HOME}}"

# Configuration directories
export SPARK_CONF_DIR="${SPARK_CONF_DIR:-${SPARK_HOME}/conf}"
export SPARK_DEFAULT_CONF_DIR="${SPARK_DEFAULT_CONF_DIR:-${SPARK_HOME}/conf.default}"
export SPARK_CONF_FILE="${SPARK_CONF_FILE:-${SPARK_CONF_DIR}/spark-defaults.conf}"

# Runtime directories
export SPARK_WORK_DIR="${SPARK_WORK_DIR:-${SPARK_HOME}/work}"
export SPARK_LOG_DIR="${SPARK_LOG_DIR:-${SPARK_HOME}/logs}"
export SPARK_TMP_DIR="${SPARK_TMP_DIR:-${SPARK_HOME}/tmp}"
export SPARK_JARS_DIR="${SPARK_JARS_DIR:-${SPARK_HOME}/jars}"
export SPARK_USER_JARS_DIR="${SPARK_USER_JARS_DIR:-${FIRESTREAM_VOLUME_DIR}/spark/jars}"

# Data persistence
export SPARK_DATA_DIR="${SPARK_DATA_DIR:-${FIRESTREAM_VOLUME_DIR}/spark/data}"

# Init scripts (Docker entrypoint pattern)
export SPARK_INITSCRIPTS_DIR="${SPARK_INITSCRIPTS_DIR:-/docker-entrypoint-initdb.d}"

# ============================================================
# Spark Mode Configuration
# Determines how Spark runs: master, worker, driver, or executor
# ============================================================

# Primary mode: master, worker, driver, executor
export SPARK_MODE="${SPARK_MODE:-master}"

# Master URL for workers to connect to
export SPARK_MASTER_URL="${SPARK_MASTER_URL:-spark://spark-master:7077}"

# Run in foreground (required for containers)
export SPARK_NO_DAEMONIZE="${SPARK_NO_DAEMONIZE:-true}"

# ============================================================
# User/Group Configuration
# ============================================================

export SPARK_USER="${SPARK_USER:-spark}"
export SPARK_DAEMON_USER="${SPARK_DAEMON_USER:-spark}"
export SPARK_DAEMON_GROUP="${SPARK_DAEMON_GROUP:-spark}"

# ============================================================
# RPC Authentication and Encryption
# Security settings for Spark inter-node communication
# https://spark.apache.org/docs/latest/security.html#authentication
# ============================================================

export SPARK_RPC_AUTHENTICATION_ENABLED="${SPARK_RPC_AUTHENTICATION_ENABLED:-no}"
export SPARK_RPC_AUTHENTICATION_SECRET="${SPARK_RPC_AUTHENTICATION_SECRET:-}"
export SPARK_RPC_ENCRYPTION_ENABLED="${SPARK_RPC_ENCRYPTION_ENABLED:-no}"

# ============================================================
# Local Storage Encryption
# Encrypt shuffle data and local block data
# https://spark.apache.org/docs/latest/security.html#local-storage-encryption
# ============================================================

export SPARK_LOCAL_STORAGE_ENCRYPTION_ENABLED="${SPARK_LOCAL_STORAGE_ENCRYPTION_ENABLED:-no}"

# ============================================================
# SSL/TLS Configuration
# HTTPS for Spark UI and inter-node encryption
# https://spark.apache.org/docs/latest/security.html#ssl-configuration
# ============================================================

export SPARK_SSL_ENABLED="${SPARK_SSL_ENABLED:-no}"
export SPARK_SSL_KEY_PASSWORD="${SPARK_SSL_KEY_PASSWORD:-}"
export SPARK_SSL_KEYSTORE_PASSWORD="${SPARK_SSL_KEYSTORE_PASSWORD:-}"
export SPARK_SSL_KEYSTORE_FILE="${SPARK_SSL_KEYSTORE_FILE:-${SPARK_CONF_DIR}/certs/spark-keystore.jks}"
export SPARK_SSL_TRUSTSTORE_PASSWORD="${SPARK_SSL_TRUSTSTORE_PASSWORD:-}"
export SPARK_SSL_TRUSTSTORE_FILE="${SPARK_SSL_TRUSTSTORE_FILE:-${SPARK_CONF_DIR}/certs/spark-truststore.jks}"
export SPARK_SSL_NEED_CLIENT_AUTH="${SPARK_SSL_NEED_CLIENT_AUTH:-yes}"
export SPARK_SSL_PROTOCOL="${SPARK_SSL_PROTOCOL:-TLSv1.2}"
export SPARK_WEBUI_SSL_PORT="${SPARK_WEBUI_SSL_PORT:-}"

# ============================================================
# Metrics Configuration
# Prometheus metrics endpoint
# ============================================================

export SPARK_METRICS_ENABLED="${SPARK_METRICS_ENABLED:-false}"

# ============================================================
# Kubernetes Mode Configuration
# Settings for running Spark on Kubernetes
# https://spark.apache.org/docs/latest/running-on-kubernetes.html
# ============================================================

# Driver configuration
export SPARK_DRIVER_BIND_ADDRESS="${SPARK_DRIVER_BIND_ADDRESS:-}"
export SPARK_DRIVER_MEMORY="${SPARK_DRIVER_MEMORY:-1g}"
export SPARK_DRIVER_CORES="${SPARK_DRIVER_CORES:-1}"
export SPARK_DRIVER_URL="${SPARK_DRIVER_URL:-}"
export SPARK_DRIVER_POD_IP="${SPARK_DRIVER_POD_IP:-}"

# Executor configuration
export SPARK_EXECUTOR_MEMORY="${SPARK_EXECUTOR_MEMORY:-1g}"
export SPARK_EXECUTOR_CORES="${SPARK_EXECUTOR_CORES:-1}"
export SPARK_EXECUTOR_ID="${SPARK_EXECUTOR_ID:-}"
export SPARK_EXECUTOR_POD_IP="${SPARK_EXECUTOR_POD_IP:-}"
export SPARK_EXECUTOR_POD_NAME="${SPARK_EXECUTOR_POD_NAME:-}"

# Application configuration
export SPARK_APPLICATION_ID="${SPARK_APPLICATION_ID:-}"
export SPARK_RESOURCE_PROFILE_ID="${SPARK_RESOURCE_PROFILE_ID:-0}"

# ============================================================
# Java Configuration
# JVM settings for Spark
# ============================================================

export JAVA_HOME="${JAVA_HOME:-/opt/java}"
export SPARK_JAVA_OPTS="${SPARK_JAVA_OPTS:-}"

# ============================================================
# Python Configuration
# PySpark settings
# ============================================================

export PYTHONPATH="${PYTHONPATH:-${SPARK_HOME}/python:${SPARK_HOME}/python/lib/py4j-0.10.9.7-src.zip}"
export PYSPARK_PYTHON="${PYSPARK_PYTHON:-python3}"
export PYSPARK_DRIVER_PYTHON="${PYSPARK_DRIVER_PYTHON:-python3}"

# ============================================================
# NSS Wrapper Configuration
# For running as arbitrary UID in containers
# ============================================================

export LIBNSS_WRAPPER_PATH="${LIBNSS_WRAPPER_PATH:-/opt/bitnami/common/lib/libnss_wrapper.so}"
export NSS_WRAPPER_PASSWD="${NSS_WRAPPER_PASSWD:-/tmp/passwd}"
export NSS_WRAPPER_GROUP="${NSS_WRAPPER_GROUP:-/tmp/group}"

# Custom environment variables may be defined below
