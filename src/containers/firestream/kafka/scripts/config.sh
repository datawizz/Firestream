#!/usr/bin/env bash
# Kafka configuration generation logic - KRaft mode only (Kafka 4.0+)
# Copyright Firestream. MIT License.
# This file is sourced, not executed directly
#
# NOTE: Sub-helper function definitions (kafka_common_conf_set,
# kafka_server_conf_set, kafka_producer_consumer_conf_set,
# kafka_configure_from_environment_variables, kafka_configure_server_jaas,
# kafka_configure_consumer_producer_jaas, kafka_client_sasl_mechanism,
# kafka_configure_ssl, kafka_configure_default_truststore_locations,
# kafka_server_unify_conf) live in scripts/helpers.sh and are emitted at
# top-level of libkafka.sh.
#
# kafka_configure MUST stay nested here — its name collides with the engine's
# <name>_configure wrapper. Bash function shadowing redefines the outer
# wrapper at runtime.

info "Generating Kafka configuration..."

########################
# Main configuration function - initialize Kafka configuration
# (nested inside engine's kafka_configure wrapper — redefines via shadowing).
########################
kafka_configure() {
  info "Initializing Kafka configuration..."

  # Check for mounted configuration files
  if [[ -d "$KAFKA_MOUNTED_CONF_DIR" ]] && compgen -G "$KAFKA_MOUNTED_CONF_DIR"/* >/dev/null 2>&1; then
    debug "Copying files from $KAFKA_MOUNTED_CONF_DIR to $KAFKA_CONF_DIR"
    cp -Lr "$KAFKA_MOUNTED_CONF_DIR"/* "$KAFKA_CONF_DIR"
  fi

  # Copy truststore to cert directory
  if [[ -f "${KAFKA_TLS_TRUSTSTORE_FILE:-}" ]] && [[ ! "${KAFKA_TLS_TRUSTSTORE_FILE:-}" =~ $KAFKA_CERTS_DIR ]]; then
    info "Copying truststore ${KAFKA_TLS_TRUSTSTORE_FILE} to ${KAFKA_CERTS_DIR}"
    cp -L "${KAFKA_TLS_TRUSTSTORE_FILE}" "$KAFKA_CERTS_DIR"
  fi

  if [[ ! -f "${KAFKA_MOUNTED_CONF_DIR}/server.properties" ]]; then
    info "No injected configuration files found, creating default config files"

    # Restore original server.properties but remove conflicting settings.
    # The firestream nix-based image ships the Apache Kafka default at
    # /config/server.properties (symlink into the kafka nix-package) rather
    # than the legacy bitnami `.original` next to $KAFKA_CONF_FILE — so fall
    # back to it when the bitnami-style original isn't present.
    if [[ -f "${KAFKA_CONF_DIR}/server.properties.original" ]]; then
      cp "${KAFKA_CONF_DIR}/server.properties.original" "$KAFKA_CONF_FILE"
      kafka_server_unify_conf
    elif [[ -f /config/server.properties ]]; then
      # /config/server.properties is a symlink into the read-only nix store,
      # so we cp -L the content and force-write +w mode so the subsequent
      # in-place env-var rewrites can modify it.
      cp -L /config/server.properties "$KAFKA_CONF_FILE"
      chmod u+w "$KAFKA_CONF_FILE"
      kafka_server_unify_conf
    fi

    # Configure Kafka settings
    kafka_server_conf_set log.dirs "$KAFKA_DATA_DIR"
    kafka_configure_from_environment_variables

    # Configure Kafka producer/consumer to set up message sizes
    ! is_empty_value "${KAFKA_CFG_MAX_REQUEST_SIZE:-}" && kafka_common_conf_set "$KAFKA_CONF_DIR/producer.properties" max.request.size "$KAFKA_CFG_MAX_REQUEST_SIZE"
    ! is_empty_value "${KAFKA_CFG_MAX_PARTITION_FETCH_BYTES:-}" && kafka_common_conf_set "$KAFKA_CONF_DIR/consumer.properties" max.partition.fetch.bytes "$KAFKA_CFG_MAX_PARTITION_FETCH_BYTES"

    # If at least one listener uses SSL or SASL_SSL, ensure SSL is configured
    if kafka_has_ssl_listener; then
      kafka_configure_ssl
    fi

    # If at least one listener uses SASL_PLAINTEXT or SASL_SSL, ensure SASL is configured
    if kafka_has_sasl_listener; then
      if [[ "$KAFKA_CFG_SASL_ENABLED_MECHANISMS" =~ SCRAM ]]; then
        export KAFKA_KRAFT_BOOTSTRAP_SCRAM_USERS="true"
      fi
      kafka_server_conf_set sasl.enabled.mechanisms "$KAFKA_CFG_SASL_ENABLED_MECHANISMS"
    fi

    # Settings for each Kafka Listener are configured individually
    if ! is_empty_value "${KAFKA_CFG_LISTENER_SECURITY_PROTOCOL_MAP:-}"; then
      read -r -a protocol_maps <<<"$(tr ',' ' ' <<<"$KAFKA_CFG_LISTENER_SECURITY_PROTOCOL_MAP")"
      for protocol_map in "${protocol_maps[@]}"; do
        read -r -a map <<<"$(tr ':' ' ' <<<"$protocol_map")"
        listener="${map[0]}"
        protocol="${map[1]}"
        listener_lower="$(echo "$listener" | tr '[:upper:]' '[:lower:]')"

        if [[ "$protocol" = "SSL" || "$protocol" = "SASL_SSL" ]]; then
          listener_upper="$(echo "$listener" | tr '[:lower:]' '[:upper:]')"
          env_name="KAFKA_TLS_${listener_upper}_CLIENT_AUTH"
          [[ -n "${!env_name:-}" ]] && kafka_server_conf_set "listener.name.${listener_lower}.ssl.client.auth" "${!env_name}"
        fi

        if [[ "$protocol" = "SASL_PLAINTEXT" || "$protocol" = "SASL_SSL" ]]; then
          local role=""
          if [[ "$listener" = "${KAFKA_CFG_INTER_BROKER_LISTENER_NAME:-INTERNAL}" ]]; then
            kafka_server_conf_set sasl.mechanism.inter.broker.protocol "$KAFKA_CFG_SASL_MECHANISM_INTER_BROKER_PROTOCOL"
            role="inter-broker"
          elif [[ "${KAFKA_CFG_CONTROLLER_LISTENER_NAMES:-CONTROLLER}" =~ $listener ]]; then
            kafka_server_conf_set sasl.mechanism.controller.protocol "$KAFKA_CFG_SASL_MECHANISM_CONTROLLER_PROTOCOL"
            kafka_server_conf_set "listener.name.${listener_lower}.sasl.enabled.mechanisms" "$KAFKA_CFG_SASL_MECHANISM_CONTROLLER_PROTOCOL"
            role="controller"
          fi
          # If KAFKA_CLIENT_LISTENER_NAME is found in the listeners list, configure the producer/consumer accordingly
          if [[ "$listener" = "${KAFKA_CLIENT_LISTENER_NAME:-CLIENT}" ]]; then
            kafka_configure_consumer_producer_jaas
            kafka_producer_consumer_conf_set security.protocol "$protocol"
            kafka_producer_consumer_conf_set sasl.mechanism "${KAFKA_CLIENT_SASL_MECHANISM:-$(kafka_client_sasl_mechanism)}"
          fi
          # Configure inline listener jaas configuration, omitted if mounted JAAS conf file detected
          if [[ ! -f "${KAFKA_CONF_DIR}/kafka_jaas.conf" ]]; then
            kafka_configure_server_jaas "$listener_lower" "${role:-}"
          fi
        fi
      done
    fi

    # Configure Kafka using environment variables again (allows user overrides)
    kafka_configure_from_environment_variables
  else
    info "Detected mounted server.properties file at ${KAFKA_MOUNTED_CONF_DIR}/server.properties. Skipping configuration based on env variables"
  fi
}

info "Configuration generation complete"
