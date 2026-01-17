#!/usr/bin/env bash
# Kafka secrets loading logic - compatible with Bitnami _FILE pattern
# Copyright Firestream. Apache-2.0 License.
# This file is sourced, not executed directly

# List of environment variables that support _FILE suffix for secret loading
kafka_env_vars=(
  KAFKA_CLIENT_PASSWORDS
  KAFKA_INTER_BROKER_PASSWORD
  KAFKA_CONTROLLER_PASSWORD
  KAFKA_CERTIFICATE_PASSWORD
  KAFKA_KEYSTORE_PASSWORD
  KAFKA_TRUSTSTORE_PASSWORD
  KAFKA_KEY_PASSWORD
  KAFKA_CLIENT_USERS
  KAFKA_INTER_BROKER_USER
  KAFKA_CONTROLLER_USER
  KAFKA_CLUSTER_ID
)

# Load secrets from _FILE variables
# Pattern: if KAFKA_CLIENT_PASSWORDS_FILE is set and points to a readable file,
# read the content into KAFKA_CLIENT_PASSWORDS and unset the _FILE variable
for env_var in "${kafka_env_vars[@]}"; do
  file_env_var="${env_var}_FILE"
  if [[ -n "${!file_env_var:-}" ]]; then
    if [[ -r "${!file_env_var:-}" ]]; then
      debug "Loading ${env_var} from file ${!file_env_var}"
      export "${env_var}=$(< "${!file_env_var}")"
      unset "${file_env_var}"
    else
      warn "Skipping export of ${env_var}. ${!file_env_var:-} is not readable."
    fi
  fi
done

unset kafka_env_vars
