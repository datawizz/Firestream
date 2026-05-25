# PostgreSQL configuration generation logic
# Copyright Firestream. MIT License.
# This file is sourced, not executed directly
#
# NOTE: All configuration functions (postgresql_create_config, postgresql_set_property, etc.)
# are defined in postgresqlHelpers in module.nix to ensure they're available in both
# initFn and configFn phases.

info "Generating PostgreSQL configuration..."

info "Configuration generation complete"
