# Odoo configuration generation
# Copyright Firestream. MIT License.
# This file is sourced, not executed directly
#
# Generates odoo.conf from template or defaults

info "Generating Odoo configuration..."

# Ensure config directory exists
ensure_dir_exists "$ODOO_CONF_DIR"

# Compute values
list_db_val="$(is_boolean_yes "$ODOO_LIST_DB" && echo 'True' || echo 'False')"
debug_val="$(is_boolean_yes "$BITNAMI_DEBUG" && echo 'True' || echo 'False')"

# Generate configuration file if it doesn't exist or if force overwrite is set
if [[ ! -f "$ODOO_CONF_FILE" ]] || is_boolean_yes "${ODOO_FORCE_OVERWRITE_CONF:-no}"; then

    # Check for template file first
    if [[ -f "${ODOO_CONF_FILE}.template" ]]; then
        debug "Generating config from template"
        cp "${ODOO_CONF_FILE}.template" "$ODOO_CONF_FILE"
    else
        debug "Generating default config"
        cat > "$ODOO_CONF_FILE" <<EOF
[options]
; Addons paths (Odoo built-in + baked vendored + custom)
addons_path = ${ODOO_BASE_DIR}/addons,${ODOO_BASE_DIR}/odoo/addons,${ODOO_BASE_DIR}/vendor-addons,${ODOO_ADDONS_DIR}

; Admin password for database management (master password)
admin_passwd = ${ODOO_PASSWORD}

; Data directory for filestore
data_dir = ${ODOO_DATA_DIR}

; Log file
logfile = ${ODOO_LOG_FILE}

; Database connection
db_host = ${ODOO_DATABASE_HOST}
db_name = ${ODOO_DATABASE_NAME}
db_password = ${ODOO_DATABASE_PASSWORD}
db_port = ${ODOO_DATABASE_PORT_NUMBER}
db_user = ${ODOO_DATABASE_USER}

; HTTP ports
http_port = ${ODOO_PORT_NUMBER}
gevent_port = ${ODOO_LONGPOLLING_PORT_NUMBER}

; Performance settings
limit_time_cpu = 90
limit_time_real = 150
max_cron_threads = 1

; Security
list_db = ${list_db_val}
proxy_mode = True

; Debug
log_level = $(is_boolean_yes "$BITNAMI_DEBUG" && echo 'debug' || echo 'info')
EOF
    fi

    info "Configuration file created: $ODOO_CONF_FILE"
else
    info "Using existing configuration file: $ODOO_CONF_FILE"
fi

# Configure SMTP if specified
if [[ -n "${ODOO_SMTP_HOST:-}" ]]; then
    info "Configuring SMTP settings..."
    odoo_conf_set "smtp_server" "$ODOO_SMTP_HOST"
    odoo_conf_set "smtp_port" "$ODOO_SMTP_PORT_NUMBER"

    if [[ "${ODOO_SMTP_PROTOCOL:-}" == "ssl" ]] || [[ "${ODOO_SMTP_PROTOCOL:-}" == "tls" ]]; then
        odoo_conf_set "smtp_ssl" "True"
    fi

    if [[ -n "${ODOO_SMTP_USER:-}" ]]; then
        odoo_conf_set "smtp_user" "$ODOO_SMTP_USER"
    fi
    if [[ -n "${ODOO_SMTP_PASSWORD:-}" ]]; then
        odoo_conf_set "smtp_password" "$ODOO_SMTP_PASSWORD"
    fi
fi

# Configure database filter if specified
if [[ -n "${ODOO_DATABASE_FILTER:-}" ]]; then
    odoo_conf_set "dbfilter" "$ODOO_DATABASE_FILTER"
fi

# Save config hash for change detection
save_config_hash "odoo" "$ODOO_CONF_FILE"

info "Odoo configuration complete"
