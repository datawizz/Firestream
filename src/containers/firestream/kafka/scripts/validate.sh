#!/usr/bin/env bash
# Kafka validation logic - KRaft mode only (Kafka 4.0+)
# Copyright Firestream. Apache-2.0 License.
# This file is sourced, not executed directly

info "Validating settings in KAFKA_* env vars..."

########################
# Check if value is in allowed list
# Arguments:
#   $1 - variable name
#   $2 - space-separated list of allowed values
########################
check_multi_value() {
  if [[ " ${2} " != *" ${!1} "* ]]; then
    print_validation_error "The allowed values for ${1} are: ${2}"
  fi
}

########################
# Check KRaft process roles are valid
########################
check_kraft_process_roles() {
  read -r -a roles_list <<<"$(tr ',;' ' ' <<<"$KAFKA_CFG_PROCESS_ROLES")"
  for role in "${roles_list[@]}"; do
    case "$role" in
    broker) ;;
    controller)
      if is_empty_value "${KAFKA_CFG_CONTROLLER_LISTENER_NAMES:-}"; then
        print_validation_error "Role 'controller' enabled but environment variable KAFKA_CFG_CONTROLLER_LISTENER_NAMES was not provided."
      fi
      if is_empty_value "${KAFKA_CFG_LISTENERS:-}" || [[ ! "$KAFKA_CFG_LISTENERS" =~ ${KAFKA_CFG_CONTROLLER_LISTENER_NAMES} ]]; then
        print_validation_error "Role 'controller' enabled but listener ${KAFKA_CFG_CONTROLLER_LISTENER_NAMES} not found in KAFKA_CFG_LISTENERS."
      fi
      if is_empty_value "${KAFKA_CFG_CONTROLLER_QUORUM_VOTERS:-}"; then
        print_validation_error "Role 'controller' enabled but environment variable KAFKA_CFG_CONTROLLER_QUORUM_VOTERS was not provided."
      fi
      ;;
    *)
      print_validation_error "Invalid KRaft process role '$role'. Supported roles are 'broker,controller'"
      ;;
    esac
  done
}

########################
# Check all listeners are using a unique and valid port
########################
check_listener_ports() {
  check_allowed_port() {
    local port="${1:?missing port variable}"
    if ! validate_port "$port" 2>/dev/null; then
      print_validation_error "An invalid port ${port} was specified in the environment variable KAFKA_CFG_LISTENERS."
    fi
  }

  read -r -a listeners <<<"$(tr ',' ' ' <<<"${KAFKA_CFG_LISTENERS:-}")"
  local -a ports=()
  for listener in "${listeners[@]}"; do
    read -r -a arr <<<"$(tr ':' ' ' <<<"$listener")"
    # Obtain the port from listener string, e.g. PLAINTEXT://:9092
    port="${arr[2]}"
    check_allowed_port "$port"
    ports+=("$port")
  done
  # Check each listener is using a unique port
  local -a unique_ports=()
  read -r -a unique_ports <<< "$(echo "${ports[@]}" | tr ' ' '\n' | sort -u | tr '\n' ' ')"
  if [[ "${#ports[@]}" != "${#unique_ports[@]}" ]]; then
    print_validation_error "There are listeners bound to the same port"
  fi
}

########################
# Check listener protocols are valid and SASL credentials are provided
########################
check_listener_protocols() {
  local -r allowed_protocols=("PLAINTEXT" "SASL_PLAINTEXT" "SASL_SSL" "SSL")

  if is_empty_value "${KAFKA_CFG_LISTENER_SECURITY_PROTOCOL_MAP:-}"; then
    return
  fi

  read -r -a protocol_maps <<<"$(tr ',' ' ' <<<"$KAFKA_CFG_LISTENER_SECURITY_PROTOCOL_MAP")"
  for protocol_map in "${protocol_maps[@]}"; do
    read -r -a map <<<"$(tr ':' ' ' <<<"$protocol_map")"
    # Obtain the listener and protocol from protocol map string, e.g. CONTROLLER:PLAINTEXT
    listener="${map[0]}"
    protocol="${map[1]}"

    # Check protocol in allowed list
    if [[ ! "${allowed_protocols[*]}" =~ $protocol ]]; then
      print_validation_error "Authentication protocol ${protocol} is not supported!"
    fi

    # If inter-broker listener configured with SASL, ensure mechanism is set
    if [[ "$listener" = "${KAFKA_CFG_INTER_BROKER_LISTENER_NAME:-INTERNAL}" ]]; then
      if [[ "$protocol" = "SASL_PLAINTEXT" ]] || [[ "$protocol" = "SASL_SSL" ]]; then
        if is_empty_value "${KAFKA_CFG_SASL_MECHANISM_INTER_BROKER_PROTOCOL:-}"; then
          print_validation_error "When using SASL for inter broker communication the mechanism should be provided using KAFKA_CFG_SASL_MECHANISM_INTER_BROKER_PROTOCOL"
        fi
        if is_empty_value "${KAFKA_INTER_BROKER_USER:-}" || is_empty_value "${KAFKA_INTER_BROKER_PASSWORD:-}"; then
          print_validation_error "In order to configure SASL authentication for Kafka inter-broker communications, you must provide the SASL credentials. Set the environment variables KAFKA_INTER_BROKER_USER and KAFKA_INTER_BROKER_PASSWORD."
        fi
      fi
    # If controller listener configured with SASL, ensure mechanism is set
    elif [[ "${KAFKA_CFG_CONTROLLER_LISTENER_NAMES:-CONTROLLER}" =~ $listener ]]; then
      if [[ "$protocol" = "SASL_PLAINTEXT" ]] || [[ "$protocol" = "SASL_SSL" ]]; then
        if is_empty_value "${KAFKA_CFG_SASL_MECHANISM_CONTROLLER_PROTOCOL:-}"; then
          print_validation_error "When using SASL for controller communication the mechanism should be provided at KAFKA_CFG_SASL_MECHANISM_CONTROLLER_PROTOCOL"
        elif [[ "$KAFKA_CFG_SASL_MECHANISM_CONTROLLER_PROTOCOL" =~ SCRAM ]]; then
          warn "KRaft controller listener may not support SCRAM-SHA-256/SCRAM-SHA-512 mechanisms. If facing any issues, we recommend switching to PLAIN mechanism."
        fi
        if is_empty_value "${KAFKA_CONTROLLER_USER:-}" || is_empty_value "${KAFKA_CONTROLLER_PASSWORD:-}"; then
          print_validation_error "In order to configure SASL authentication for Kafka control plane communications, you must provide the SASL credentials. Set the environment variables KAFKA_CONTROLLER_USER and KAFKA_CONTROLLER_PASSWORD."
        fi
      fi
    else
      # Client listener with SASL
      if [[ "$protocol" = "SASL_PLAINTEXT" ]] || [[ "$protocol" = "SASL_SSL" ]]; then
        if is_empty_value "${KAFKA_CLIENT_USERS:-}" || is_empty_value "${KAFKA_CLIENT_PASSWORDS:-}"; then
          print_validation_error "In order to configure SASL authentication for Kafka, you must provide the SASL credentials. Set the environment variables KAFKA_CLIENT_USERS and KAFKA_CLIENT_PASSWORDS."
        fi
      fi
    fi
  done
}

########################
# Check if at least one listener uses SSL
########################
kafka_has_ssl_listener() {
  if ! is_empty_value "${KAFKA_CFG_LISTENERS:-}"; then
    if is_empty_value "${KAFKA_CFG_LISTENER_SECURITY_PROTOCOL_MAP:-}"; then
      if [[ "$KAFKA_CFG_LISTENERS" =~ SSL: || "$KAFKA_CFG_LISTENERS" =~ SASL_SSL: ]]; then
        return 0
      fi
    else
      read -r -a protocol_maps <<<"$(tr ',' ' ' <<<"$KAFKA_CFG_LISTENER_SECURITY_PROTOCOL_MAP")"
      for protocol_map in "${protocol_maps[@]}"; do
        read -r -a map <<<"$(tr ':' ' ' <<<"$protocol_map")"
        listener="${map[0]}"
        protocol="${map[1]}"
        if [[ "$protocol" = "SSL" || "$protocol" = "SASL_SSL" ]]; then
          if [[ "$KAFKA_CFG_LISTENERS" =~ $listener ]]; then
            return 0
          fi
        fi
      done
    fi
  fi
  return 1
}

########################
# Check if at least one listener uses SASL
########################
kafka_has_sasl_listener() {
  if ! is_empty_value "${KAFKA_CFG_LISTENERS:-}"; then
    if is_empty_value "${KAFKA_CFG_LISTENER_SECURITY_PROTOCOL_MAP:-}"; then
      if [[ "$KAFKA_CFG_LISTENERS" =~ SASL_PLAINTEXT: ]] || [[ "$KAFKA_CFG_LISTENERS" =~ SASL_SSL: ]]; then
        return 0
      fi
    else
      read -r -a protocol_maps <<<"$(tr ',' ' ' <<<"$KAFKA_CFG_LISTENER_SECURITY_PROTOCOL_MAP")"
      for protocol_map in "${protocol_maps[@]}"; do
        read -r -a map <<<"$(tr ':' ' ' <<<"$protocol_map")"
        listener="${map[0]}"
        protocol="${map[1]}"
        if [[ "$protocol" = "SASL_PLAINTEXT" || "$protocol" = "SASL_SSL" ]]; then
          if [[ "$KAFKA_CFG_LISTENERS" =~ $listener ]]; then
            return 0
          fi
        fi
      done
    fi
  fi
  return 1
}

########################
# Check if at least one listener uses plaintext
########################
kafka_has_plaintext_listener() {
  if ! is_empty_value "${KAFKA_CFG_LISTENER_SECURITY_PROTOCOL_MAP:-}"; then
    read -r -a protocol_maps <<<"$(tr ',' ' ' <<<"$KAFKA_CFG_LISTENER_SECURITY_PROTOCOL_MAP")"
    for protocol_map in "${protocol_maps[@]}"; do
      read -r -a map <<<"$(tr ':' ' ' <<<"$protocol_map")"
      listener="${map[0]}"
      protocol="${map[1]}"
      if [[ "$protocol" = "PLAINTEXT" ]]; then
        if is_empty_value "${KAFKA_CFG_LISTENERS:-}" || [[ "$KAFKA_CFG_LISTENERS" =~ $listener ]]; then
          return 0
        fi
      fi
    done
  else
    if is_empty_value "${KAFKA_CFG_LISTENERS:-}" || [[ "$KAFKA_CFG_LISTENERS" =~ PLAINTEXT: ]]; then
      return 0
    fi
  fi
  return 1
}

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
