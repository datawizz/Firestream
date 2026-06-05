#!/usr/bin/env bash
# Kafka configuration generation logic - KRaft mode only (Kafka 4.0+)
# Copyright Firestream. MIT License.
# This file is sourced, not executed directly

info "Generating Kafka configuration..."

########################
# Set a configuration setting value to a file
# Arguments:
#   $1 - file
#   $2 - key
#   $3 - value
########################
kafka_common_conf_set() {
  local file="${1:?missing file}"
  local key="${2:?missing key}"
  shift 2
  local value="$*"

  if [[ -z "$value" ]]; then
    warn "Missing value for key $key"
    return 1
  fi

  # Check if the value was set before
  if grep -q "^[#\\s]*$key\s*=.*" "$file" 2>/dev/null; then
    # Update the existing key
    replace_in_file "$file" "^[#\\s]*${key}\s*=.*" "${key}=${value}"
  else
    # Add a new key
    printf '\n%s=%s' "$key" "$value" >>"$file"
  fi
}

########################
# Set a configuration setting value to server.properties
# Arguments:
#   $1 - key
#   $2 - value
########################
kafka_server_conf_set() {
  kafka_common_conf_set "$KAFKA_CONF_FILE" "$@"
}

########################
# Set a configuration setting value to producer.properties and consumer.properties
# Arguments:
#   $1 - key
#   $2 - value
########################
kafka_producer_consumer_conf_set() {
  kafka_common_conf_set "$KAFKA_CONF_DIR/producer.properties" "$@"
  kafka_common_conf_set "$KAFKA_CONF_DIR/consumer.properties" "$@"
}

########################
# Configure Kafka from KAFKA_CFG_* environment variables
########################
kafka_configure_from_environment_variables() {
  info "Configuring Kafka from environment variables..."
  # List of special cases to apply to the variables
  local -r exception_regexps=(
    "s/sasl\.ssl/sasl_ssl/g"
    "s/sasl\.plaintext/sasl_plaintext/g"
  )
  # Map environment variables to config properties
  for var in "${!KAFKA_CFG_@}"; do
    value="${!var}"
    # Skip empty values to avoid errors under set -e
    if is_empty_value "$value"; then
      continue
    fi

    key="$(echo "$var" | sed -e 's/^KAFKA_CFG_//g' -e 's/_/\./g' | tr '[:upper:]' '[:lower:]')"

    # Apply exception regexps
    for regex in "${exception_regexps[@]}"; do
      key="$(echo "$key" | sed "$regex")"
    done

    kafka_server_conf_set "$key" "$value"
  done
}

########################
# Configure JAAS for a given listener
# Arguments:
#   $1 - Name of the listener
#   $2 - Role (controller, inter-broker, or empty for client)
########################
kafka_configure_server_jaas() {
  local listener="${1:?missing listener name}"
  local role="${2:-}"

  if [[ "$role" = "controller" ]]; then
    local jaas_content=()
    if [[ "$KAFKA_CFG_SASL_MECHANISM_CONTROLLER_PROTOCOL" = "PLAIN" ]]; then
      jaas_content=(
        "org.apache.kafka.common.security.plain.PlainLoginModule required"
        "username=\"${KAFKA_CONTROLLER_USER}\""
        "password=\"${KAFKA_CONTROLLER_PASSWORD}\""
        "user_${KAFKA_CONTROLLER_USER}=\"${KAFKA_CONTROLLER_PASSWORD}\";"
      )
    elif [[ "$KAFKA_CFG_SASL_MECHANISM_CONTROLLER_PROTOCOL" =~ SCRAM ]]; then
      jaas_content=(
        "org.apache.kafka.common.security.scram.ScramLoginModule required"
        "username=\"${KAFKA_CONTROLLER_USER}\""
        "password=\"${KAFKA_CONTROLLER_PASSWORD}\";"
      )
    fi
    listener_lower="$(echo "$listener" | tr '[:upper:]' '[:lower:]')"
    sasl_mechanism_lower="$(echo "$KAFKA_CFG_SASL_MECHANISM_CONTROLLER_PROTOCOL" | tr '[:upper:]' '[:lower:]')"
    kafka_server_conf_set "listener.name.${listener_lower}.${sasl_mechanism_lower}.sasl.jaas.config" "${jaas_content[*]}"
  else
    read -r -a sasl_mechanisms_arr <<<"$(tr ',' ' ' <<<"$KAFKA_CFG_SASL_ENABLED_MECHANISMS")"
    read -r -a users <<<"$(tr ',;' ' ' <<<"$KAFKA_CLIENT_USERS")"
    read -r -a passwords <<<"$(tr ',;' ' ' <<<"$KAFKA_CLIENT_PASSWORDS")"

    # Configure JAAS for each SASL mechanism
    for sasl_mechanism in "${sasl_mechanisms_arr[@]}"; do
      local jaas_content=()
      # For PLAIN mechanism, only the first username will be used
      if [[ "$sasl_mechanism" = "PLAIN" ]]; then
        jaas_content=("org.apache.kafka.common.security.plain.PlainLoginModule required")
        if [[ "$role" = "inter-broker" ]]; then
          jaas_content+=(
            "username=\"${KAFKA_INTER_BROKER_USER}\""
            "password=\"${KAFKA_INTER_BROKER_PASSWORD}\""
          )
          users+=("$KAFKA_INTER_BROKER_USER")
          passwords+=("$KAFKA_INTER_BROKER_PASSWORD")
        fi
        for ((i = 0; i < ${#users[@]}; i++)); do
          jaas_content+=("user_${users[i]}=\"${passwords[i]}\"")
        done
        # Add semi-colon to the last element of the array
        jaas_content[${#jaas_content[@]} - 1]="${jaas_content[${#jaas_content[@]} - 1]};"
      elif [[ "$sasl_mechanism" =~ SCRAM ]]; then
        if [[ "$role" = "inter-broker" ]]; then
          jaas_content=(
            "org.apache.kafka.common.security.scram.ScramLoginModule required"
            "username=\"${KAFKA_INTER_BROKER_USER}\""
            "password=\"${KAFKA_INTER_BROKER_PASSWORD}\";"
          )
        else
          jaas_content=("org.apache.kafka.common.security.scram.ScramLoginModule required;")
        fi
      fi
      listener_lower="$(echo "$listener" | tr '[:upper:]' '[:lower:]')"
      sasl_mechanism_lower="$(echo "$sasl_mechanism" | tr '[:upper:]' '[:lower:]')"
      kafka_server_conf_set "listener.name.${listener_lower}.${sasl_mechanism_lower}.sasl.jaas.config" "${jaas_content[*]}"
    done
  fi
}

########################
# Configure JAAS for producer/consumer
########################
kafka_configure_consumer_producer_jaas() {
  local jaas_content=()
  read -r -a users <<<"$(tr ',;' ' ' <<<"${KAFKA_CLIENT_USERS}")"
  read -r -a passwords <<<"$(tr ',;' ' ' <<<"${KAFKA_CLIENT_PASSWORDS}")"

  if [[ "${KAFKA_CFG_SASL_ENABLED_MECHANISMS}" =~ SCRAM ]]; then
    jaas_content=("org.apache.kafka.common.security.scram.ScramLoginModule required")
  elif [[ "${KAFKA_CFG_SASL_ENABLED_MECHANISMS}" =~ PLAIN ]]; then
    jaas_content=("org.apache.kafka.common.security.plain.PlainLoginModule required")
  else
    error "Couldn't configure a supported SASL mechanism for Kafka consumer/producer properties"
    return 1
  fi

  jaas_content+=(
    "username=\"${users[0]}\""
    "password=\"${passwords[0]}\";"
  )

  kafka_producer_consumer_conf_set "sasl.jaas.config" "${jaas_content[*]}"
}

########################
# Returns the most secure SASL mechanism available for Kafka clients
########################
kafka_client_sasl_mechanism() {
  local sasl_mechanism=""

  if [[ "$KAFKA_CFG_SASL_ENABLED_MECHANISMS" =~ SCRAM-SHA-512 ]]; then
    sasl_mechanism="SCRAM-SHA-512"
  elif [[ "$KAFKA_CFG_SASL_ENABLED_MECHANISMS" =~ SCRAM-SHA-256 ]]; then
    sasl_mechanism="SCRAM-SHA-256"
  elif [[ "$KAFKA_CFG_SASL_ENABLED_MECHANISMS" =~ PLAIN ]]; then
    sasl_mechanism="PLAIN"
  fi
  echo "$sasl_mechanism"
}

########################
# Configure Kafka SSL settings
########################
kafka_configure_ssl() {
  info "Configuring SSL..."
  # Configures both Kafka server and producers/consumers
  configure_both() {
    kafka_server_conf_set "${1:?missing key}" "${2:?missing value}"
    kafka_producer_consumer_conf_set "${1:?missing key}" "${2:?missing value}"
  }
  kafka_server_conf_set "ssl.client.auth" "${KAFKA_TLS_CLIENT_AUTH}"
  configure_both ssl.keystore.type "${KAFKA_TLS_TYPE}"
  configure_both ssl.truststore.type "${KAFKA_TLS_TYPE}"
  local -r kafka_truststore_location="${KAFKA_CERTS_DIR}/$(basename "${KAFKA_TLS_TRUSTSTORE_FILE}")"
  ! is_empty_value "${KAFKA_CERTIFICATE_PASSWORD:-}" && configure_both ssl.key.password "$KAFKA_CERTIFICATE_PASSWORD"

  if [[ "$KAFKA_TLS_TYPE" = "PEM" ]]; then
    file_to_multiline_property() {
      awk 'NR > 1{print line"\\n\\"}{line=$0;}END{print $0" "}' <"${1:?missing file}"
    }
    configure_both ssl.keystore.key "$(file_to_multiline_property "${KAFKA_CERTS_DIR}/kafka.keystore.key")"
    configure_both ssl.keystore.certificate.chain "$(file_to_multiline_property "${KAFKA_CERTS_DIR}/kafka.keystore.pem")"
    configure_both ssl.truststore.certificates "$(file_to_multiline_property "${kafka_truststore_location}")"
  elif [[ "$KAFKA_TLS_TYPE" = "JKS" ]]; then
    configure_both ssl.keystore.location "$KAFKA_CERTS_DIR"/kafka.keystore.jks
    configure_both ssl.truststore.location "$kafka_truststore_location"
    ! is_empty_value "${KAFKA_CERTIFICATE_PASSWORD:-}" && configure_both ssl.keystore.password "$KAFKA_CERTIFICATE_PASSWORD"
    ! is_empty_value "${KAFKA_CERTIFICATE_PASSWORD:-}" && configure_both ssl.truststore.password "$KAFKA_CERTIFICATE_PASSWORD"
  fi
}

########################
# Configure default truststore locations
########################
kafka_configure_default_truststore_locations() {
  # Set KAFKA_TLS_TRUSTSTORE_FILE to default location if not set
  if kafka_has_ssl_listener && is_empty_value "${KAFKA_TLS_TRUSTSTORE_FILE:-}"; then
    local kafka_truststore_filename="kafka.truststore.jks"
    [[ "$KAFKA_TLS_TYPE" = "PEM" ]] && kafka_truststore_filename="kafka.truststore.pem"
    if [[ -f "${KAFKA_CERTS_DIR}/${kafka_truststore_filename}" ]]; then
      export KAFKA_TLS_TRUSTSTORE_FILE="${KAFKA_CERTS_DIR}/${kafka_truststore_filename}"
    else
      export KAFKA_TLS_TRUSTSTORE_FILE="${KAFKA_MOUNTED_CONF_DIR}/certs/${kafka_truststore_filename}"
    fi
  fi
}

########################
# Remove default KRaft/Zookeeper settings from server.properties
########################
kafka_server_unify_conf() {
  # Setting patterns to be commented
  local -r remove_settings=(
    "^zookeeper\."
    "^group\.initial"
    "^broker\."
    "^node\."
    "^process\."
    "^listeners="
    "^listener\."
    "^controller\."
    "^inter\.broker"
    "^advertised\.listeners"
  )

  for match_pattern in "${remove_settings[@]}"; do
    # Replace beginning of line with comment
    sub_pattern="${match_pattern/^/#}"
    # Replace \. with .
    sub_pattern="${sub_pattern/\\./.}"
    replace_in_file "$KAFKA_CONF_FILE" "$match_pattern" "$sub_pattern"
  done
}

########################
# Main configuration function - initialize Kafka configuration
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
