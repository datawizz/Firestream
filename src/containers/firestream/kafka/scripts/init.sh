#!/usr/bin/env bash
# Kafka initialization logic - KRaft mode only (Kafka 4.0+)
# Copyright Firestream. MIT License.
# This file is sourced, not executed directly
#
# NOTE: Sub-helper function definitions (kafka_dynamic_environment_variables,
# kafka_kraft_storage_initialize, kafka_custom_init_scripts,
# kafka_clean_from_restart, kafka_create_directories, is_kafka_running,
# is_kafka_not_running, kafka_stop) live in scripts/helpers.sh and are emitted
# at top-level of libkafka.sh.
#
# kafka_initialize MUST stay nested here — its name collides with the engine's
# <name>_initialize wrapper. Bash function shadowing redefines the outer
# wrapper at runtime.

info "Initializing Kafka..."

########################
# Main initialization function (nested inside engine's kafka_initialize wrapper —
# redefines via shadowing).
########################
kafka_initialize() {
  info "Running Kafka initialization..."

  # Clean stale files from previous run
  kafka_clean_from_restart

  # Create required directories
  kafka_create_directories

  # Set dynamic environment variables
  kafka_dynamic_environment_variables

  # Configure default truststore locations
  kafka_configure_default_truststore_locations

  # Generate configuration
  kafka_configure

  # Check if this is a fresh install or existing data
  if [[ ! -f "${KAFKA_DATA_DIR}/meta.properties" ]]; then
    info "First run detected, initializing KRaft storage..."
    kafka_kraft_storage_initialize
  else
    info "Deploying Kafka with persisted data..."
    # For existing data, just verify cluster ID matches
    local existing_cluster_id
    existing_cluster_id=$(grep "^cluster.id=" "${KAFKA_DATA_DIR}/meta.properties" | sed -E 's/^cluster\.id=(\S+)$/\1/')
    if [[ -n "${KAFKA_CLUSTER_ID:-}" ]] && [[ "$existing_cluster_id" != "$KAFKA_CLUSTER_ID" ]]; then
      warn "Existing cluster ID ($existing_cluster_id) differs from KAFKA_CLUSTER_ID ($KAFKA_CLUSTER_ID). Using existing cluster ID."
    fi
    export KAFKA_CLUSTER_ID="$existing_cluster_id"
  fi
}

# Run initialization
kafka_initialize
kafka_custom_init_scripts

info "Kafka initialization complete"
