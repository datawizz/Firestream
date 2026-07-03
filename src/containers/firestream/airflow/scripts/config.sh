# Airflow configuration generation logic
# Copyright Firestream. MIT License.
# This file is sourced, not executed directly

info "Generating Airflow configuration..."

# Chart mode: airflow.cfg is fully rendered by the chart's prepare-config init
# container and mounted into this pod; the config dir itself is read-only, so the
# crudini-based ini_set calls below would fail trying to write a temp file there.
# The chart-provided cfg already contains the correct connection/executor/celery
# settings, so skip in-container re-generation when AIRFLOW_SKIP_DB_SETUP=yes.
if is_boolean_yes "${AIRFLOW_SKIP_DB_SETUP:-no}"; then
  info "AIRFLOW_SKIP_DB_SETUP=yes - using chart-provided airflow.cfg; skipping configuration generation"
  return 0
fi

major_version=$(airflow_major_version)
debug "Detected Airflow major version: $major_version"

# Generate base configuration
case "$AIRFLOW_COMPONENT_TYPE" in
  webserver|api-server)
    if [[ ! -f "$AIRFLOW_CONF_FILE" ]]; then
      info "Generating default airflow.cfg..."
      airflow config list --defaults > "$AIRFLOW_CONF_FILE"
    fi
    if [[ ! -f "$AIRFLOW_WEBSERVER_CONF_FILE" ]]; then
      info "Generating default webserver_config.py..."
      cat > "$AIRFLOW_WEBSERVER_CONF_FILE" << 'PYEOF'
# Airflow webserver configuration
from flask_appbuilder.security.manager import AUTH_DB
# from flask_appbuilder.security.manager import AUTH_LDAP

AUTH_TYPE = AUTH_DB
PYEOF
    fi
    ;;
  *)
    if [[ ! -f "$AIRFLOW_CONF_FILE" ]]; then
      info "Generating default airflow.cfg..."
      airflow config list --defaults > "$AIRFLOW_CONF_FILE"
    fi
    # Non-webserver components need API server configuration for Airflow 3
    if [[ $major_version -ne 2 ]]; then
      ini_set "api" "host" "$AIRFLOW_APISERVER_HOST" "$AIRFLOW_CONF_FILE"
      ini_set "core" "execution_api_server_url" \
        "http://${AIRFLOW_APISERVER_HOST}:${AIRFLOW_APISERVER_PORT_NUMBER}/execution/" "$AIRFLOW_CONF_FILE"
    fi
    ;;
esac

# Configure database connection
db_user=$(airflow_encode_url "$AIRFLOW_DATABASE_USERNAME")
db_pass=$(airflow_encode_url "$AIRFLOW_DATABASE_PASSWORD")
db_ssl_opt=""
is_boolean_yes "$AIRFLOW_DATABASE_USE_SSL" && db_ssl_opt="?sslmode=require"

ini_set "database" "sql_alchemy_conn" \
  "postgresql+psycopg2://${db_user}:${db_pass}@${AIRFLOW_DATABASE_HOST}:${AIRFLOW_DATABASE_PORT_NUMBER}/${AIRFLOW_DATABASE_NAME}${db_ssl_opt}" \
  "$AIRFLOW_CONF_FILE"

# Configure port
if [[ $major_version -eq 2 ]]; then
  ini_set "webserver" "web_server_port" "$AIRFLOW_APISERVER_PORT_NUMBER" "$AIRFLOW_CONF_FILE"
else
  ini_set "api" "port" "$AIRFLOW_APISERVER_PORT_NUMBER" "$AIRFLOW_CONF_FILE"
fi

# Configure base URL
scheme="http"
is_boolean_yes "$AIRFLOW_ENABLE_HTTPS" && scheme="https"
base_url="${scheme}://${AIRFLOW_APISERVER_HOST}"
[[ "$AIRFLOW_APISERVER_PORT_NUMBER" != "80" && "$AIRFLOW_APISERVER_PORT_NUMBER" != "443" ]] && \
  base_url="${base_url}:${AIRFLOW_APISERVER_PORT_NUMBER}"

ini_set "webserver" "base_url" "$base_url" "$AIRFLOW_CONF_FILE"
[[ $major_version -eq 3 ]] && ini_set "api" "base_url" "$base_url" "$AIRFLOW_CONF_FILE"

# Configure secret keys
[[ -n "$AIRFLOW_FERNET_KEY" ]] && ini_set "core" "fernet_key" "$AIRFLOW_FERNET_KEY" "$AIRFLOW_CONF_FILE"

secret_section="api"
[[ $major_version -eq 2 ]] && secret_section="webserver"
ini_set "$secret_section" "secret_key" "$AIRFLOW_WEBSERVER_SECRET_KEY" "$AIRFLOW_CONF_FILE"
[[ $major_version -ne 2 ]] && ini_set "api_auth" "jwt_secret" "$AIRFLOW_APISERVER_SECRET_KEY" "$AIRFLOW_CONF_FILE"

# Configure executor
ini_set "core" "executor" "$AIRFLOW_EXECUTOR" "$AIRFLOW_CONF_FILE"
ini_set "core" "auth_manager" "airflow.providers.fab.auth_manager.fab_auth_manager.FabAuthManager" "$AIRFLOW_CONF_FILE"

# Configure Celery if needed
if [[ "$AIRFLOW_EXECUTOR" == "CeleryExecutor" || "$AIRFLOW_EXECUTOR" == "CeleryKubernetesExecutor" ]]; then
  info "Configuring Celery executor..."
  redis_user=$(airflow_encode_url "$REDIS_USER")
  redis_pass=$(airflow_encode_url "$REDIS_PASSWORD")
  redis_proto="redis"
  is_boolean_yes "$AIRFLOW_REDIS_USE_SSL" && redis_proto="rediss"

  # Build broker URL
  if [[ -n "$REDIS_USER" ]]; then
    broker_url="${redis_proto}://${redis_user}:${redis_pass}@${REDIS_HOST}:${REDIS_PORT_NUMBER}/${REDIS_DATABASE}"
  else
    broker_url="${redis_proto}://:${redis_pass}@${REDIS_HOST}:${REDIS_PORT_NUMBER}/${REDIS_DATABASE}"
  fi
  ini_set "celery" "broker_url" "$broker_url" "$AIRFLOW_CONF_FILE"

  is_boolean_yes "$AIRFLOW_REDIS_USE_SSL" && ini_set "celery" "redis_backend_use_ssl" "true" "$AIRFLOW_CONF_FILE"

  ini_set "celery" "result_backend" \
    "db+postgresql://${db_user}:${db_pass}@${AIRFLOW_DATABASE_HOST}:${AIRFLOW_DATABASE_PORT_NUMBER}/${AIRFLOW_DATABASE_NAME}${db_ssl_opt}" \
    "$AIRFLOW_CONF_FILE"
fi

# Configure examples and DAG processor
if [[ "$AIRFLOW_COMPONENT_TYPE" != "worker" ]]; then
  load_examples="False"
  is_boolean_yes "$AIRFLOW_LOAD_EXAMPLES" && load_examples="True"
  ini_set "core" "load_examples" "$load_examples" "$AIRFLOW_CONF_FILE"

  standalone_dag="False"
  is_boolean_yes "$AIRFLOW_STANDALONE_DAG_PROCESSOR" && standalone_dag="True"
  ini_set "scheduler" "standalone_dag_processor" "$standalone_dag" "$AIRFLOW_CONF_FILE"
fi

# Configure triggerer capacity
if [[ "$AIRFLOW_COMPONENT_TYPE" == "triggerer" && -n "$AIRFLOW_TRIGGERER_DEFAULT_CAPACITY" ]]; then
  capacity_key="capacity"
  [[ $major_version -eq 2 ]] && capacity_key="default_capacity"
  ini_set "triggerer" "$capacity_key" "$AIRFLOW_TRIGGERER_DEFAULT_CAPACITY" "$AIRFLOW_CONF_FILE"
fi

# Configure hostname callable if set
if [[ -n "${AIRFLOW_HOSTNAME_CALLABLE:-}" ]]; then
  ini_set "core" "hostname_callable" "$AIRFLOW_HOSTNAME_CALLABLE" "$AIRFLOW_CONF_FILE"
fi

# Configure LDAP for webserver/api-server
if [[ "$AIRFLOW_COMPONENT_TYPE" == "webserver" || "$AIRFLOW_COMPONENT_TYPE" == "api-server" ]]; then
  ini_set "webserver" "rbac" "true" "$AIRFLOW_CONF_FILE"

  if is_boolean_yes "${AIRFLOW_LDAP_ENABLE:-no}"; then
    info "Configuring LDAP authentication..."
    sed -i 's/AUTH_TYPE = AUTH_DB/AUTH_TYPE = AUTH_LDAP/' "$AIRFLOW_WEBSERVER_CONF_FILE"
    sed -i 's/from flask_appbuilder.security.manager import AUTH_DB/from flask_appbuilder.security.manager import AUTH_LDAP/' "$AIRFLOW_WEBSERVER_CONF_FILE"

    # String values must be Python-quoted (is_literal="no"); bool/dict literals
    # (True/False, {...}) must be inserted raw (is_literal="yes").
    python_conf_set "AUTH_LDAP_SERVER" "$AIRFLOW_LDAP_URI" "$AIRFLOW_WEBSERVER_CONF_FILE" "no"
    python_conf_set "AUTH_LDAP_SEARCH" "$AIRFLOW_LDAP_SEARCH" "$AIRFLOW_WEBSERVER_CONF_FILE" "no"
    python_conf_set "AUTH_LDAP_UID_FIELD" "$AIRFLOW_LDAP_UID_FIELD" "$AIRFLOW_WEBSERVER_CONF_FILE" "no"
    python_conf_set "AUTH_LDAP_BIND_USER" "$AIRFLOW_LDAP_BIND_USER" "$AIRFLOW_WEBSERVER_CONF_FILE" "no"
    python_conf_set "AUTH_LDAP_BIND_PASSWORD" "$AIRFLOW_LDAP_BIND_PASSWORD" "$AIRFLOW_WEBSERVER_CONF_FILE" "no"
    python_conf_set "AUTH_USER_REGISTRATION" "$AIRFLOW_LDAP_USER_REGISTRATION" "$AIRFLOW_WEBSERVER_CONF_FILE" "yes"
    [[ -n "${AIRFLOW_LDAP_USER_REGISTRATION_ROLE:-}" ]] && \
      python_conf_set "AUTH_USER_REGISTRATION_ROLE" "$AIRFLOW_LDAP_USER_REGISTRATION_ROLE" "$AIRFLOW_WEBSERVER_CONF_FILE" "no"
    [[ -n "${AIRFLOW_LDAP_ROLES_MAPPING:-}" ]] && \
      python_conf_set "AUTH_ROLES_MAPPING" "$AIRFLOW_LDAP_ROLES_MAPPING" "$AIRFLOW_WEBSERVER_CONF_FILE" "yes"
    python_conf_set "AUTH_ROLES_SYNC_AT_LOGIN" "$AIRFLOW_LDAP_ROLES_SYNC_AT_LOGIN" "$AIRFLOW_WEBSERVER_CONF_FILE" "yes"
    python_conf_set "AUTH_LDAP_ALLOW_SELF_SIGNED" "$AIRFLOW_LDAP_ALLOW_SELF_SIGNED" "$AIRFLOW_WEBSERVER_CONF_FILE" "yes"

    if [[ "$AIRFLOW_LDAP_USE_TLS" == "True" && -n "${AIRFLOW_LDAP_TLS_CA_CERTIFICATE:-}" ]]; then
      python_conf_set "AUTH_LDAP_TLS_CACERTFILE" "$AIRFLOW_LDAP_TLS_CA_CERTIFICATE" "$AIRFLOW_WEBSERVER_CONF_FILE" "no"
    fi
  fi

  # Configure hashing for Airflow 3
  if [[ $major_version -eq 3 ]]; then
    python_conf_set "FAB_PASSWORD_HASH_METHOD" "pbkdf2:sha256" "$AIRFLOW_WEBSERVER_CONF_FILE" "no"
  fi
fi

# Debug logging
is_boolean_yes "${BITNAMI_DEBUG:-false}" && ini_set "logging" "logging_level" "DEBUG" "$AIRFLOW_CONF_FILE"

info "Configuration generation complete"
