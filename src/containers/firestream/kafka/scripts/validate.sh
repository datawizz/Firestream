#!/usr/bin/env bash
# Kafka validation logic - KRaft mode only (Kafka 4.0+)
# Copyright Firestream. MIT License.
# This file is sourced, not executed directly
#
# NOTE: Helper function definitions (check_multi_value, check_kraft_process_roles,
# check_listener_ports, check_listener_protocols, kafka_has_ssl_listener,
# kafka_has_sasl_listener, kafka_has_plaintext_listener) live in scripts/helpers.sh
# and are emitted at top-level of libkafka.sh.

info "Validating settings in KAFKA_* env vars..."

# Main validation logic

# KRaft mode is required for Kafka 4.0+
if is_empty_value "${KAFKA_CFG_PROCESS_ROLES:-}"; then
  print_validation_error "Kafka requires at least one process role to be set. Set the environment variable KAFKA_CFG_PROCESS_ROLES to configure the process roles for Kafka."
fi

# Check KRaft process roles
check_kraft_process_roles

# Check node ID is set
if is_empty_value "${KAFKA_CFG_NODE_ID:-}"; then
  print_validation_error "KRaft mode requires a unique node.id, please set the environment variable KAFKA_CFG_NODE_ID"
fi

# Check listener ports are unique and allowed
check_listener_ports

# Check listeners are mapped to a valid security protocol
check_listener_protocols

# Warn users if plaintext listeners are configured
if kafka_has_plaintext_listener; then
  warn "Kafka has been configured with a PLAINTEXT listener, this setting is not recommended for production environments."
fi

# If SSL/SASL_SSL listeners configured, check certificates are provided
if kafka_has_ssl_listener; then
  if [[ "$KAFKA_TLS_TYPE" = "JKS" ]]; then
    if { [[ ! -f "${KAFKA_CERTS_DIR}/kafka.keystore.jks" ]] || [[ ! -f "${KAFKA_TLS_TRUSTSTORE_FILE:-}" ]]; } &&
       { [[ ! -f "${KAFKA_MOUNTED_CONF_DIR}/certs/kafka.keystore.jks" ]] || [[ ! -f "${KAFKA_TLS_TRUSTSTORE_FILE:-}" ]]; }; then
      print_validation_error "In order to configure the TLS encryption for Kafka with JKS certs you must mount your kafka.keystore.jks and kafka.truststore.jks certs to the ${KAFKA_MOUNTED_CONF_DIR}/certs directory."
    fi
  elif [[ "$KAFKA_TLS_TYPE" = "PEM" ]]; then
    if { [[ ! -f "${KAFKA_CERTS_DIR}/kafka.keystore.pem" ]] || [[ ! -f "${KAFKA_CERTS_DIR}/kafka.keystore.key" ]] || [[ ! -f "${KAFKA_TLS_TRUSTSTORE_FILE:-}" ]]; } &&
       { [[ ! -f "${KAFKA_MOUNTED_CONF_DIR}/certs/kafka.keystore.pem" ]] || [[ ! -f "${KAFKA_MOUNTED_CONF_DIR}/certs/kafka.keystore.key" ]] || [[ ! -f "${KAFKA_TLS_TRUSTSTORE_FILE:-}" ]]; }; then
      print_validation_error "In order to configure the TLS encryption for Kafka with PEM certs you must mount your kafka.keystore.pem, kafka.keystore.key and kafka.truststore.pem certs to the ${KAFKA_MOUNTED_CONF_DIR}/certs directory."
    fi
  fi
fi

# If SASL/SASL_SSL listeners configured, check SASL mechanisms are provided
if kafka_has_sasl_listener; then
  if is_empty_value "${KAFKA_CFG_SASL_ENABLED_MECHANISMS:-}"; then
    print_validation_error "Specified SASL protocol but no SASL mechanisms provided in KAFKA_CFG_SASL_ENABLED_MECHANISMS"
  fi
fi

# Check users and passwords lists are the same size
read -r -a users <<<"$(tr ',;' ' ' <<<"${KAFKA_CLIENT_USERS:-}")"
read -r -a passwords <<<"$(tr ',;' ' ' <<<"${KAFKA_CLIENT_PASSWORDS:-}")"
if [[ "${#users[@]}" -ne "${#passwords[@]}" ]]; then
  print_validation_error "Specify the same number of passwords on KAFKA_CLIENT_PASSWORDS as the number of users on KAFKA_CLIENT_USERS!"
fi

# Validate enum values
check_multi_value "KAFKA_TLS_TYPE" "JKS PEM"
check_multi_value "KAFKA_TLS_CLIENT_AUTH" "none requested required"

info "Validation complete."
