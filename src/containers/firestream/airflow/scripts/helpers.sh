# Airflow helper functions
# Copyright Firestream. MIT License.
# These functions are defined at top-level of libairflow.sh so chart init
# containers can `source /opt/bitnami/scripts/libairflow.sh` and invoke them
# directly. The two retry-loop helpers were moved verbatim from init.sh;
# the airflow_conf_set / airflow_wait_for_* helpers are new minimal
# implementations modeled on Bitnami's libairflow.sh, sufficient to satisfy
# the init-container call sites in
# src/charts/firestream/airflow/templates/_init_containers_sidecars.tpl.

########################
# Set a key/value in an Airflow .cfg (INI) file.
# Bitnami signature: airflow_conf_set <section> <key> <value> [<file>]
# Default file is $AIRFLOW_CONF_FILE (typically /opt/airflow/airflow.cfg).
# Implementation uses crudini if available, falling back to a sed-based
# write that handles "missing section" / "missing key" / "update existing
# key" cases.
########################
airflow_conf_set() {
    local -r section="${1:?missing section}"
    local -r key="${2:?missing key}"
    local -r value="${3:-}"
    local -r file="${4:-${AIRFLOW_CONF_FILE:-/opt/airflow/airflow.cfg}}"

    if [[ ! -f "$file" ]]; then
        warn "airflow_conf_set: config file $file does not exist; creating"
        mkdir -p "$(dirname "$file")"
        : > "$file"
    fi

    if command -v crudini >/dev/null 2>&1; then
        crudini --set "$file" "$section" "$key" "$value"
        return $?
    fi

    # crudini-less fallback: parse-and-rewrite. Bitnami ships crudini in the
    # image so this branch is mostly defensive.
    if ! grep -qE "^\[${section}\]" "$file"; then
        printf '\n[%s]\n' "$section" >> "$file"
    fi
    # Detect existing key inside the section
    local in_section=0
    local matched=0
    local tmp
    tmp="$(mktemp)"
    while IFS= read -r line || [[ -n "$line" ]]; do
        if [[ "$line" =~ ^\[${section}\]$ ]]; then
            in_section=1
            printf '%s\n' "$line" >> "$tmp"
            continue
        fi
        if [[ $in_section -eq 1 && "$line" =~ ^\[ ]]; then
            if [[ $matched -eq 0 ]]; then
                printf '%s = %s\n' "$key" "$value" >> "$tmp"
                matched=1
            fi
            in_section=0
        fi
        if [[ $in_section -eq 1 && "$line" =~ ^[[:space:]]*${key}[[:space:]]*= ]]; then
            printf '%s = %s\n' "$key" "$value" >> "$tmp"
            matched=1
            continue
        fi
        printf '%s\n' "$line" >> "$tmp"
    done < "$file"
    if [[ $matched -eq 0 ]]; then
        printf '%s = %s\n' "$key" "$value" >> "$tmp"
    fi
    mv "$tmp" "$file"
}

########################
# Set a key in the Airflow webserver_config.py (Python file).
# Bitnami signature: airflow_webserver_conf_set <key> <value> <quote_value>
# When quote_value is "yes"/"true" the value is wrapped in double-quotes
# (Python string literal); otherwise it's written verbatim.
########################
airflow_webserver_conf_set() {
    local -r key="${1:?missing key}"
    local -r value="${2:-}"
    local -r quote_value="${3:-no}"
    local -r file="${AIRFLOW_WEBSERVER_CONF_FILE:-/opt/airflow/webserver_config.py}"

    if [[ ! -f "$file" ]]; then
        warn "airflow_webserver_conf_set: $file does not exist; creating"
        mkdir -p "$(dirname "$file")"
        : > "$file"
    fi

    local replacement
    if [[ "$quote_value" =~ ^(yes|true|1)$ ]]; then
        replacement="${key} = \"${value}\""
    else
        replacement="${key} = ${value}"
    fi

    if grep -qE "^${key}[[:space:]]*=" "$file"; then
        sed -i -E "s|^${key}[[:space:]]*=.*|${replacement}|" "$file"
    else
        printf '\n%s\n' "$replacement" >> "$file"
    fi
}

########################
# Block until the Airflow metadata DB is reachable on
# $AIRFLOW_DATABASE_HOST:$AIRFLOW_DATABASE_PORT_NUMBER. Bitnami's
# implementation polls TCP with retry_while; we use the wait-for-port
# helper that's already in PATH (provided by firestream.waitForPortPkg).
########################
airflow_wait_for_db_connection() {
    local -r host="${AIRFLOW_DATABASE_HOST:-localhost}"
    local -r port="${AIRFLOW_DATABASE_PORT_NUMBER:-5432}"
    local -r timeout="${AIRFLOW_DB_CONNECTION_TIMEOUT:-120}"

    info "Waiting for Airflow database at ${host}:${port} (timeout: ${timeout}s)..."
    if command -v wait-for-port >/dev/null 2>&1; then
        wait-for-port --host "$host" --timeout "$timeout" "$port"
    else
        # Fallback: bash /dev/tcp loop
        local elapsed=0
        while ! (echo > "/dev/tcp/${host}/${port}") >/dev/null 2>&1; do
            if (( elapsed >= timeout )); then
                error "Timed out waiting for ${host}:${port}"
                return 1
            fi
            sleep 1
            elapsed=$((elapsed + 1))
        done
    fi
    info "Airflow database is reachable"
}

########################
# Block until `airflow db check-migrations` reports the metadata DB is
# at the latest revision. Used by sidecar containers that depend on the
# initial migration having completed.
########################
airflow_wait_for_db_migrations() {
    local -r tries="${AIRFLOW_DB_MIGRATE_TRIES:-60}"
    local -r sleep_secs="${AIRFLOW_DB_MIGRATE_SLEEP:-10}"
    local attempt=0

    info "Waiting for Airflow DB migrations to complete (up to ${tries} attempts)..."
    while (( attempt < tries )); do
        if airflow db check-migrations --migration-wait-timeout=0 >/dev/null 2>&1; then
            info "Airflow DB migrations complete"
            return 0
        fi
        attempt=$((attempt + 1))
        sleep "$sleep_secs"
    done
    error "Timed out waiting for Airflow DB migrations"
    return 1
}

########################
# Block until the configured admin user appears in `airflow users list`.
# Airflow 3+ requires the FAB admin row to exist before workers can fetch
# task instances (the wait-for-db-migrations sidecar gates on this).
########################
airflow_wait_for_admin_user() {
    local -r username="${AIRFLOW_USERNAME:-admin}"
    local -r tries="${AIRFLOW_ADMIN_WAIT_TRIES:-60}"
    local -r sleep_secs="${AIRFLOW_ADMIN_WAIT_SLEEP:-5}"
    local attempt=0

    info "Waiting for Airflow admin user '${username}' to exist..."
    while (( attempt < tries )); do
        if airflow users list --output plain 2>&1 | grep -v DEBUG | grep -q "$username"; then
            info "Airflow admin user '${username}' is present"
            return 0
        fi
        attempt=$((attempt + 1))
        sleep "$sleep_secs"
    done
    error "Timed out waiting for Airflow admin user '${username}'"
    return 1
}

# Helper function for retry_while - checks if admin user exists
# Must handle set -e properly by not letting the command trigger immediate exit
is_airflow_admin_created() {
    local airflow_users
    airflow_users="$(airflow users list --output plain 2>&1 | grep -v DEBUG)" || true
    if echo "${airflow_users}" | grep -q "$AIRFLOW_USERNAME"; then
        return 0
    fi
    return 1
}

# Helper function for retry_while - checks if DB migrations are complete
# Must handle set -e properly by not letting the command trigger immediate exit
is_db_migrated() {
    local result=0
    airflow db check-migrations --migration-wait-timeout=0 >/dev/null 2>&1 || result=$?
    return $result
}
