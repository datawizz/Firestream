# Odoo validation logic
# Copyright Firestream. Apache-2.0 License.
# This file is sourced, not executed directly
#
# Validates ODOO_* environment variables before initialization

debug "Validating Odoo environment variables..."

# Validate yes/no values
for var in ODOO_SKIP_BOOTSTRAP ODOO_SKIP_MODULES_UPDATE ODOO_LOAD_DEMO_DATA ODOO_LIST_DB; do
    val="${!var:-}"
    if [[ -n "$val" ]]; then
        case "${val,,}" in
            yes|no|true|false|1|0) ;;
            *) print_validation_error "Invalid value for $var: $val (expected yes/no)" ;;
        esac
    fi
done

# Validate port numbers
for port_var in ODOO_PORT_NUMBER ODOO_LONGPOLLING_PORT_NUMBER ODOO_DATABASE_PORT_NUMBER; do
    port="${!port_var:-}"
    if [[ -n "$port" ]]; then
        if ! [[ "$port" =~ ^[0-9]+$ ]] || [[ "$port" -lt 1 ]] || [[ "$port" -gt 65535 ]]; then
            print_validation_error "Invalid port for $port_var: $port"
        fi
    fi
done

# Validate database host can be resolved
if [[ -n "${ODOO_DATABASE_HOST:-}" ]]; then
    if ! getent hosts "$ODOO_DATABASE_HOST" >/dev/null 2>&1; then
        warn "Database host '$ODOO_DATABASE_HOST' could not be resolved - this may cause connection issues"
    fi
fi

# Validate credentials unless empty passwords are explicitly allowed
if is_boolean_yes "${ALLOW_EMPTY_PASSWORD:-no}"; then
    warn "ALLOW_EMPTY_PASSWORD is enabled - do not use in production"
else
    if is_empty_value "${ODOO_PASSWORD:-}"; then
        print_validation_error "ODOO_PASSWORD is required (set ALLOW_EMPTY_PASSWORD=yes to override)"
    fi
    if is_empty_value "${ODOO_DATABASE_PASSWORD:-}"; then
        print_validation_error "ODOO_DATABASE_PASSWORD is required (set ALLOW_EMPTY_PASSWORD=yes to override)"
    fi
fi

# Validate SMTP configuration if SMTP host is set
if [[ -n "${ODOO_SMTP_HOST:-}" ]]; then
    if is_empty_value "${ODOO_SMTP_PORT_NUMBER:-}"; then
        print_validation_error "ODOO_SMTP_PORT_NUMBER is required when ODOO_SMTP_HOST is set"
    fi
    if is_empty_value "${ODOO_SMTP_USER:-}"; then
        warn "ODOO_SMTP_USER is not set - SMTP authentication may fail"
    fi
    if is_empty_value "${ODOO_SMTP_PASSWORD:-}"; then
        warn "ODOO_SMTP_PASSWORD is not set - SMTP authentication may fail"
    fi
    if [[ -n "${ODOO_SMTP_PROTOCOL:-}" ]]; then
        case "${ODOO_SMTP_PROTOCOL}" in
            ssl|tls) ;;
            *) print_validation_error "Invalid ODOO_SMTP_PROTOCOL: ${ODOO_SMTP_PROTOCOL} (expected ssl or tls)" ;;
        esac
    fi
fi

# Validate email format (basic check)
if [[ -n "${ODOO_EMAIL:-}" ]]; then
    if ! [[ "${ODOO_EMAIL}" =~ ^[^@]+@[^@]+\.[^@]+$ ]]; then
        warn "ODOO_EMAIL does not appear to be a valid email address: ${ODOO_EMAIL}"
    fi
fi
