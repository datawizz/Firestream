#!/usr/bin/env bash
# Kafka initialization logic - KRaft mode only (Kafka 4.0+)
# Copyright Firestream. MIT License.
# This file is sourced, not executed directly

info "Initializing Kafka..."

########################
# Dynamically set node.id/controller.quorum.voters if _COMMAND env vars are set
########################
kafka_dynamic_environment_variables() {
  if ! is_empty_value "${KAFKA_NODE_ID_COMMAND:-}"; then
    KAFKA_CFG_NODE_ID="$(eval "${KAFKA_NODE_ID_COMMAND}")"
    export KAFKA_CFG_NODE_ID
  fi
  if ! is_empty_value "${KAFKA_CONTROLLER_QUORUM_VOTERS_COMMAND:-}"; then
    KAFKA_CFG_CONTROLLER_QUORUM_VOTERS="$(eval "${KAFKA_CONTROLLER_QUORUM_VOTERS_COMMAND}")"
    export KAFKA_CFG_CONTROLLER_QUORUM_VOTERS
  fi
}

########################
# Initialize KRaft storage
########################
kafka_kraft_storage_initialize() {
  local args=("--config" "$KAFKA_CONF_FILE" "--ignore-formatted")
  info "Initializing KRaft storage metadata"

  # If cluster.id found in meta.properties, use it
  if [[ -f "${KAFKA_DATA_DIR}/meta.properties" ]]; then
    KAFKA_CLUSTER_ID=$(grep "^cluster.id=" "${KAFKA_DATA_DIR}/meta.properties" | sed -E 's/^cluster\.id=(\S+)$/\1/')
    export KAFKA_CLUSTER_ID
  fi

  if is_empty_value "${KAFKA_CLUSTER_ID:-}"; then
    warn "KAFKA_CLUSTER_ID not set - If using multiple nodes then you must use the same Cluster ID for each one"
    KAFKA_CLUSTER_ID="$(kafka-storage.sh random-uuid)"
    export KAFKA_CLUSTER_ID
    info "Generated Kafka cluster ID '${KAFKA_CLUSTER_ID}'"
  fi
  args+=("--cluster-id=$KAFKA_CLUSTER_ID")

  # SCRAM users are configured during the cluster bootstrapping process
  if is_boolean_yes "${KAFKA_KRAFT_BOOTSTRAP_SCRAM_USERS:-}"; then
    info "Adding KRaft SCRAM users at storage bootstrap"
    read -r -a users <<<"$(tr ',;' ' ' <<<"${KAFKA_CLIENT_USERS}")"
    read -r -a passwords <<<"$(tr ',;' ' ' <<<"${KAFKA_CLIENT_PASSWORDS}")"

    # Configure SCRAM-SHA-256 if enabled
    if grep -Eq "^sasl.enabled.mechanisms=.*SCRAM-SHA-256" "$KAFKA_CONF_FILE"; then
      for ((i = 0; i < ${#users[@]}; i++)); do
        args+=("--add-scram" "SCRAM-SHA-256=[name=${users[i]},password=${passwords[i]}]")
      done
    fi

    # Configure SCRAM-SHA-512 if enabled
    if grep -Eq "^sasl.enabled.mechanisms=.*SCRAM-SHA-512" "$KAFKA_CONF_FILE"; then
      for ((i = 0; i < ${#users[@]}; i++)); do
        args+=("--add-scram" "SCRAM-SHA-512=[name=${users[i]},password=${passwords[i]}]")
      done
    fi

    # Add interbroker credentials
    if grep -Eq "^sasl.mechanism.inter.broker.protocol=SCRAM-SHA-256" "$KAFKA_CONF_FILE"; then
      args+=("--add-scram" "SCRAM-SHA-256=[name=${KAFKA_INTER_BROKER_USER},password=${KAFKA_INTER_BROKER_PASSWORD}]")
    elif grep -Eq "^sasl.mechanism.inter.broker.protocol=SCRAM-SHA-512" "$KAFKA_CONF_FILE"; then
      args+=("--add-scram" "SCRAM-SHA-512=[name=${KAFKA_INTER_BROKER_USER},password=${KAFKA_INTER_BROKER_PASSWORD}]")
    fi

    # Add controller credentials
    if grep -Eq "^sasl.mechanism.controller.protocol=SCRAM-SHA-256" "$KAFKA_CONF_FILE"; then
      args+=("--add-scram" "SCRAM-SHA-256=[name=${KAFKA_CONTROLLER_USER},password=${KAFKA_CONTROLLER_PASSWORD}]")
    elif grep -Eq "^sasl.mechanism.controller.protocol=SCRAM-SHA-512" "$KAFKA_CONF_FILE"; then
      args+=("--add-scram" "SCRAM-SHA-512=[name=${KAFKA_CONTROLLER_USER},password=${KAFKA_CONTROLLER_PASSWORD}]")
    fi
  fi

  # For Kafka 4.0+, controller role requires kraft.version feature, and
  # `kafka-storage format --feature=kraft.version=1` additionally requires one
  # of --standalone / --initial-controllers / --no-initial-controllers (Kafka
  # 4.x dynamic-controllers contract). Pick --standalone when the quorum has a
  # single voter, otherwise --no-initial-controllers (multi-node bootstrap).
  if [[ "${KAFKA_CFG_PROCESS_ROLES:-}" =~ "controller" ]]; then
    args+=("--feature=kraft.version=1")
    if [[ "${KAFKA_CFG_CONTROLLER_QUORUM_VOTERS:-}" != *,* ]]; then
      args+=("--standalone")
    else
      args+=("--no-initial-controllers")
    fi
  fi

  info "Formatting storage directories to add metadata..."
  debug_execute kafka-storage.sh format "${args[@]}"
}

########################
# Run custom initialization scripts
########################
kafka_custom_init_scripts() {
  if [[ -n $(find "${KAFKA_INITSCRIPTS_DIR}/" -type f -regex ".*\.\(sh\)" 2>/dev/null) ]] && [[ ! -f "${KAFKA_VOLUME_DIR}/.user_scripts_initialized" ]]; then
    info "Loading user's custom files from $KAFKA_INITSCRIPTS_DIR"
    for f in "${KAFKA_INITSCRIPTS_DIR}"/*; do
      case "$f" in
      *.sh)
        if [[ -x "$f" ]]; then
          debug "Executing $f"
          if ! "$f"; then
            error "Failed executing $f"
            return 1
          fi
        else
          warn "Sourcing $f as it is not executable by the current user, any error may cause initialization to fail"
          . "$f"
        fi
        ;;
      *)
        warn "Skipping $f, supported formats are: .sh"
        ;;
      esac
    done
    touch "$KAFKA_VOLUME_DIR"/.user_scripts_initialized
  fi
}

########################
# Clean stale files from restart
########################
kafka_clean_from_restart() {
  if [[ -f "$KAFKA_PID_FILE" ]]; then
    info "Cleaning stale PID file"
    rm -f "$KAFKA_PID_FILE"
  fi
}

########################
# Create required directories
########################
kafka_create_directories() {
  for dir in "$KAFKA_TMP_DIR" "$KAFKA_LOG_DIR" "$KAFKA_DATA_DIR" "$KAFKA_CONF_DIR" "$KAFKA_CERTS_DIR"; do
    ensure_dir_exists "$dir"
  done
  chmod u+rwx "$KAFKA_DATA_DIR"
}

########################
# Check if Kafka is running
########################
is_kafka_running() {
  local pid
  pid="$(get_pid_from_file "$KAFKA_PID_FILE" 2>/dev/null)"
  if [[ -n "$pid" ]]; then
    is_service_running "$pid"
  else
    false
  fi
}

########################
# Check if Kafka is not running
########################
is_kafka_not_running() {
  ! is_kafka_running
}

########################
# Stop Kafka
########################
kafka_stop() {
  ! is_kafka_running && return
  stop_service_using_pid "$KAFKA_PID_FILE" TERM
}

########################
# Main initialization function
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
