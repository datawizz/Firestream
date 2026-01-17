#!/usr/bin/env bash
# Load _FILE variables - compatible with Bitnami pattern
# Copyright Firestream. Apache-2.0 License.

postgresql_env_vars=(
  POSTGRESQL_PASSWORD
  POSTGRESQL_USERNAME
  POSTGRESQL_DATABASE
  POSTGRESQL_POSTGRES_PASSWORD
  POSTGRESQL_REPLICATION_USER
  POSTGRESQL_REPLICATION_PASSWORD
  POSTGRESQL_MASTER_HOST
  POSTGRESQL_MASTER_PORT_NUMBER
  POSTGRESQL_TLS_CERT_FILE
  POSTGRESQL_TLS_KEY_FILE
  POSTGRESQL_TLS_CA_FILE
  POSTGRESQL_TLS_CRL_FILE
  POSTGRESQL_LDAP_URL
  POSTGRESQL_LDAP_SERVER
  POSTGRESQL_LDAP_PORT
  POSTGRESQL_LDAP_PREFIX
  POSTGRESQL_LDAP_SUFFIX
  POSTGRESQL_LDAP_BASE_DN
  POSTGRESQL_LDAP_BIND_DN
  POSTGRESQL_LDAP_BIND_PASSWORD
  POSTGRESQL_LDAP_SEARCH_ATTR
  POSTGRESQL_LDAP_SEARCH_FILTER
  POSTGRESQL_INITDB_ARGS
  POSTGRESQL_SHARED_PRELOAD_LIBRARIES
)

for env_var in "${postgresql_env_vars[@]}"; do
  file_env_var="${env_var}_FILE"
  if [[ -n "${!file_env_var:-}" ]]; then
    if [[ -r "${!file_env_var:-}" ]]; then
      export "${env_var}=$(< "${!file_env_var}")"
      unset "${file_env_var}"
    else
      warn "Skipping export of ${env_var}. ${!file_env_var:-} is not readable."
    fi
  fi
done
unset postgresql_env_vars
