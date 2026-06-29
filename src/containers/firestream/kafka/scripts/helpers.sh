# Kafka helper functions
# Copyright Firestream. MIT License.
# These functions are defined at top-level of libkafka.sh so chart init
# containers can `source /opt/bitnami/scripts/libkafka.sh` and invoke them
# directly. Moved verbatim from {validate,config,init}.sh — function bodies
# unchanged.

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

# NOTE: kafka_configure is NOT included here — it would collide with the
# engine's <name>_configure wrapper. Its definition lives nested inside
# scripts/config.sh (re-defined at runtime via function shadowing).

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
# Normalize a 22-char base64 Kafka Uuid (KRaft directory id) to its canonical
# form. The chart's kraft Secret generates random alphanumeric 22-char ids
# whose final character carries non-canonical low bits; Kafka rewrites them to
# canonical base64url when it formats meta.properties (e.g. a trailing 'j' ->
# 'g'). The KIP-853 --initial-controllers voter set MUST use the same canonical
# dir-id Kafka stores in each node's meta.properties — otherwise a node's vote
# requests reference a directory.id the target controller doesn't recognise,
# every peer stays UNRECORDED, and the quorum never elects a leader (clients
# then can't fetch metadata). Decode->re-encode to canonicalise; on any error
# fall back to the raw value unchanged.
# Arguments:
#   $1 - raw 22-char (base64url) id
# Returns:
#   canonical id on stdout
########################
kafka_normalize_dir_id() {
  local v="${1:-}"
  [[ -z "$v" ]] && { printf '%s' "$v"; return 0; }
  # base64url -> standard base64, then decode (16 bytes) and re-encode canonically.
  local b64="${v//-/+}"; b64="${b64//_//}"
  local canonical
  canonical="$(printf '%s==' "$b64" | base64 -d 2>/dev/null | base64 -w0 2>/dev/null)" || { printf '%s' "$v"; return 0; }
  canonical="${canonical:0:22}"
  canonical="${canonical//+/-}"; canonical="${canonical//\//_}"
  if [[ -n "$canonical" ]]; then printf '%s' "$canonical"; else printf '%s' "$v"; fi
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

  # The chart shares a single cluster id across all nodes via the kraft Secret,
  # injected as KAFKA_KRAFT_CLUSTER_ID. Every controller/broker MUST format with
  # that same id, otherwise each node invents its own and peers reject votes
  # with INCONSISTENT_CLUSTER_ID — the quorum never elects a leader. Fall back to
  # KAFKA_KRAFT_CLUSTER_ID before generating a throwaway random uuid.
  if is_empty_value "${KAFKA_CLUSTER_ID:-}" && ! is_empty_value "${KAFKA_KRAFT_CLUSTER_ID:-}"; then
    KAFKA_CLUSTER_ID="$KAFKA_KRAFT_CLUSTER_ID"
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
  # of --initial-controllers / --standalone / --no-initial-controllers (Kafka
  # 4.x dynamic-controllers / KIP-853 contract).
  #
  # Multi-controller clusters MUST be bootstrapped with --initial-controllers so
  # every node agrees on the same initial voter set (node-id@host:port:dir-id).
  # The chart's prepare-config init container materialises that list (with the
  # per-node stable directory IDs) at $KAFKA_INITIAL_CONTROLLERS_FILE and the
  # broker/controller env exports KAFKA_INITIAL_CONTROLLERS / the file path. If
  # we instead format every node with --standalone, each becomes its own
  # single-voter quorum -> split brain -> nodes CrashLoop with
  # "No readable meta.properties". Only fall back to --standalone for a genuine
  # single-voter quorum, or --no-initial-controllers when no list is available.
  if [[ "${KAFKA_CFG_PROCESS_ROLES:-}" =~ "controller" ]]; then
    args+=("--feature=kraft.version=1")
    local initial_controllers="${KAFKA_INITIAL_CONTROLLERS:-}"
    if is_empty_value "$initial_controllers" && [[ -f "${KAFKA_INITIAL_CONTROLLERS_FILE:-}" ]]; then
      initial_controllers="$(< "${KAFKA_INITIAL_CONTROLLERS_FILE}")"
    fi
    if ! is_empty_value "$initial_controllers"; then
      args+=("--initial-controllers" "$initial_controllers")
    elif [[ "${KAFKA_CFG_CONTROLLER_QUORUM_VOTERS:-}" != *,* ]]; then
      args+=("--standalone")
    else
      args+=("--no-initial-controllers")
    fi
  fi

  info "Formatting storage directories to add metadata..."
  # Do NOT swallow format errors (the previous debug_execute hid stderr): a
  # silently-failed format leaves no meta.properties, but the engine still marks
  # the app initialized, so every later restart skips initialization and Kafka
  # dies with "No readable meta.properties files found" forever. Surface the
  # output and verify meta.properties landed; returning non-zero here lets the
  # entrypoint (set -e) abort BEFORE mark_app_initialized, so a restart re-runs
  # the format instead of poisoning the data dir.
  if ! kafka-storage.sh format "${args[@]}"; then
    error "kafka-storage format failed"
    return 1
  fi
  if [[ ! -f "${KAFKA_DATA_DIR}/meta.properties" ]]; then
    error "kafka-storage format did not produce ${KAFKA_DATA_DIR}/meta.properties"
    return 1
  fi
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

# NOTE: kafka_initialize is NOT included here — it would collide with the
# engine's <name>_initialize wrapper. Its definition lives nested inside
# scripts/init.sh (re-defined at runtime via function shadowing).
