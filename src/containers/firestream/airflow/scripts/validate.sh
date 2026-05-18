# Airflow validation logic
# Copyright Firestream. MIT License.
# This file is sourced, not executed directly

# Component type validation
case "$AIRFLOW_COMPONENT_TYPE" in
  api-server|dag-processor|scheduler|triggerer|webserver|worker) ;;
  *)
    print_validation_error "Invalid AIRFLOW_COMPONENT_TYPE: $AIRFLOW_COMPONENT_TYPE"
    print_validation_error "Allowed values: api-server dag-processor scheduler triggerer webserver worker"
    ;;
esac

# Executor validation
if is_empty_value "$AIRFLOW_EXECUTOR"; then
  print_validation_error "AIRFLOW_EXECUTOR is required"
else
  case "$AIRFLOW_EXECUTOR" in
    LocalExecutor|CeleryExecutor|CeleryKubernetesExecutor|KubernetesExecutor) ;;
    *)
      print_validation_error "Invalid AIRFLOW_EXECUTOR: $AIRFLOW_EXECUTOR"
      print_validation_error "Allowed values: LocalExecutor CeleryExecutor CeleryKubernetesExecutor KubernetesExecutor"
      ;;
  esac
fi

# Yes/No validation
for var in AIRFLOW_STANDALONE_DAG_PROCESSOR AIRFLOW_SKIP_DB_SETUP AIRFLOW_LOAD_EXAMPLES AIRFLOW_FORCE_OVERWRITE_CONF_FILE; do
  val="${!var:-}"
  if [[ -n "$val" ]]; then
    case "${val,,}" in
      yes|no|true|false|1|0) ;;
      *) print_validation_error "Invalid value for $var: $val (expected yes/no)" ;;
    esac
  fi
done

# Process cryptographic keys (auto-generate if missing, base64 encode)
# This must happen before config generation to ensure secrets are available
process_fernet_key || error_code=1
process_secret_key "AIRFLOW_WEBSERVER_SECRET_KEY"
process_secret_key "AIRFLOW_APISERVER_SECRET_KEY"
process_secret_key "AIRFLOW_JWT_SECRET_KEY"

# Database validation
if is_empty_value "$AIRFLOW_DATABASE_HOST"; then
  print_validation_error "AIRFLOW_DATABASE_HOST is required"
fi

# Celery executor requires Redis
if [[ "$AIRFLOW_EXECUTOR" == "CeleryExecutor" || "$AIRFLOW_EXECUTOR" == "CeleryKubernetesExecutor" ]]; then
  if is_empty_value "$REDIS_HOST"; then
    print_validation_error "REDIS_HOST is required for $AIRFLOW_EXECUTOR"
  fi
fi

# Component-specific validation
case "$AIRFLOW_COMPONENT_TYPE" in
  webserver|api-server)
    # LDAP validation
    if is_boolean_yes "${AIRFLOW_LDAP_ENABLE:-no}"; then
      for var in AIRFLOW_LDAP_URI AIRFLOW_LDAP_SEARCH AIRFLOW_LDAP_UID_FIELD \
                 AIRFLOW_LDAP_BIND_USER AIRFLOW_LDAP_BIND_PASSWORD; do
        if is_empty_value "${!var:-}"; then
          print_validation_error "$var is required when LDAP is enabled"
        fi
      done
    fi
    # Pool validation
    if [[ -n "${AIRFLOW_POOL_NAME:-}" ]]; then
      if is_empty_value "${AIRFLOW_POOL_SIZE:-}" || is_empty_value "${AIRFLOW_POOL_DESC:-}"; then
        print_validation_error "AIRFLOW_POOL_SIZE and AIRFLOW_POOL_DESC required with AIRFLOW_POOL_NAME"
      fi
    fi
    ;;
  *)
    # Non-webserver components need API server host for Airflow 3
    if is_empty_value "${AIRFLOW_APISERVER_HOST:-}"; then
      warn "AIRFLOW_APISERVER_HOST not set, using default 127.0.0.1"
    fi
    ;;
esac
