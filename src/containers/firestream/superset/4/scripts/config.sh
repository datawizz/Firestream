# Superset runtime configuration adjustments
# This file is sourced, not executed directly
# Copyright Firestream. MIT License.

info "Configuring Superset runtime settings..."

# The main configuration is generated during the activation phase (activateFn)
# via template substitution. This script handles any runtime adjustments.

# Verify config file exists
local config_file="${SUPERSET_CONFIG_PATH:-/opt/superset/superset_config.py}"
if [[ ! -f "$config_file" ]]; then
  warn "Configuration file not found at $config_file"
  warn "This may indicate an issue with the activation phase"
fi

# Set up PYTHONPATH for custom configurations
export PYTHONPATH="${PYTHONPATH:-}:/app/pythonpath"

# Create superset_config_docker.py hook if custom config dir has content
if [[ -d "/app/pythonpath" ]] && [[ -n "$(ls -A /app/pythonpath 2>/dev/null)" ]]; then
  info "Custom Python modules found in /app/pythonpath"
fi

debug "Runtime configuration complete"
