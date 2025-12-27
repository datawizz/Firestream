# state.nix - Application state tracking and generation management
# Copyright Firestream. Apache-2.0 License.
#
# This module provides functions for tracking application state:
# - Config hash calculation and comparison
# - Generation counter management
# - Activation time recording
# - Build-time prepopulation marker checking
#
# Usage:
#   state_dir=$(get_state_dir "airflow")
#   hash=$(get_config_hash "/path/to/config.ini")
#   save_config_hash "airflow" "/path/to/config.ini"
#   if has_config_changed "airflow" "/path/to/config.ini"; then echo "changed"; fi
#   gen=$(get_generation "airflow")
#   new_gen=$(increment_generation "airflow")
#   record_activation "airflow"
#   if check_prepopulated "airflow"; then echo "prepopulated"; fi

{ pkgs, lib, logModule, fsModule, configModule }:

let
  functions = ''
    # Import dependencies
    ${logModule.functions}
    ${fsModule.functions}

    ########################
    # Get the state directory for an application
    # Arguments:
    #   $1 - app - Application name
    #   $2 - base_dir - Base directory (default: /firestream)
    # Returns:
    #   State directory path (stdout)
    #########################
    get_state_dir() {
        local app="''${1:?app name is missing}"
        local base_dir="''${2:-/firestream}"
        local state_dir="''${base_dir}/''${app}/.state"

        ${pkgs.coreutils}/bin/echo "$state_dir"
    }

    ########################
    # Calculate SHA256 hash of config file(s)
    # Arguments:
    #   $1+ - file path(s) to hash (can be multiple files)
    # Returns:
    #   SHA256 hash string (stdout)
    #########################
    get_config_hash() {
        local files=("$@")
        local combined_hash=""

        if [[ ''${#files[@]} -eq 0 ]]; then
            error "No config files specified for hashing"
            return 1
        fi

        # Hash each file and combine
        for file in "''${files[@]}"; do
            if [[ ! -f "$file" ]]; then
                warn "Config file does not exist: $file"
                continue
            fi
            debug "Hashing config file: $file"
            local file_hash
            file_hash="$(${pkgs.coreutils}/bin/sha256sum "$file" | ${pkgs.coreutils}/bin/cut -d' ' -f1)"
            combined_hash="''${combined_hash}''${file_hash}"
        done

        if [[ -z "$combined_hash" ]]; then
            error "No valid config files to hash"
            return 1
        fi

        # If multiple files, hash the combined hashes for a single result
        if [[ ''${#files[@]} -gt 1 ]]; then
            combined_hash="$(${pkgs.coreutils}/bin/echo -n "$combined_hash" | ${pkgs.coreutils}/bin/sha256sum | ${pkgs.coreutils}/bin/cut -d' ' -f1)"
        fi

        ${pkgs.coreutils}/bin/echo "$combined_hash"
    }

    ########################
    # Save config hash to state directory
    # Arguments:
    #   $1 - app - Application name
    #   $2+ - config file path(s)
    # Creates:
    #   /firestream/{app}/.state/config-hash
    #########################
    save_config_hash() {
        local app="''${1:?app name is missing}"
        shift
        local files=("$@")
        local base_dir="/firestream"
        local state_dir
        state_dir="$(get_state_dir "$app" "$base_dir")"
        local hash_file="''${state_dir}/config-hash"

        if [[ ''${#files[@]} -eq 0 ]]; then
            error "No config files specified"
            return 1
        fi

        debug "Saving config hash for $app"
        ensure_dir_exists "$state_dir"

        local hash
        hash="$(get_config_hash "''${files[@]}")"
        if [[ $? -ne 0 ]]; then
            error "Failed to calculate config hash for $app"
            return 1
        fi

        ${pkgs.coreutils}/bin/echo "$hash" > "$hash_file"
        info "Saved config hash for $app: $hash"
    }

    ########################
    # Check if config has changed since last save
    # Arguments:
    #   $1 - app - Application name
    #   $2+ - config file path(s)
    # Returns:
    #   0 if changed (or no saved hash), 1 if same
    #########################
    has_config_changed() {
        local app="''${1:?app name is missing}"
        shift
        local files=("$@")
        local base_dir="/firestream"
        local state_dir
        state_dir="$(get_state_dir "$app" "$base_dir")"
        local hash_file="''${state_dir}/config-hash"

        if [[ ''${#files[@]} -eq 0 ]]; then
            error "No config files specified"
            return 0  # Consider it changed if we can't compare
        fi

        # If no saved hash, consider it changed
        if [[ ! -f "$hash_file" ]]; then
            debug "No saved config hash for $app - considering changed"
            return 0
        fi

        local saved_hash
        saved_hash="$(${pkgs.coreutils}/bin/cat "$hash_file")"

        local current_hash
        current_hash="$(get_config_hash "''${files[@]}")"
        if [[ $? -ne 0 ]]; then
            warn "Failed to calculate current config hash for $app"
            return 0  # Consider it changed if we can't calculate
        fi

        if [[ "$saved_hash" == "$current_hash" ]]; then
            debug "Config hash unchanged for $app"
            return 1  # Not changed
        else
            debug "Config hash changed for $app: $saved_hash -> $current_hash"
            return 0  # Changed
        fi
    }

    ########################
    # Get current generation number
    # Arguments:
    #   $1 - app - Application name
    #   $2 - base_dir - Base directory (default: /firestream)
    # Returns:
    #   Integer generation number (stdout), 0 if not set
    #########################
    get_generation() {
        local app="''${1:?app name is missing}"
        local base_dir="''${2:-/firestream}"
        local state_dir
        state_dir="$(get_state_dir "$app" "$base_dir")"
        local gen_file="''${state_dir}/generation"

        if [[ -f "$gen_file" ]]; then
            local gen
            gen="$(${pkgs.coreutils}/bin/cat "$gen_file")"
            # Validate it's a number
            if [[ "$gen" =~ ^[0-9]+$ ]]; then
                ${pkgs.coreutils}/bin/echo "$gen"
            else
                warn "Invalid generation value for $app: $gen"
                ${pkgs.coreutils}/bin/echo "0"
            fi
        else
            debug "No generation file for $app, returning 0"
            ${pkgs.coreutils}/bin/echo "0"
        fi
    }

    ########################
    # Increment generation counter
    # Arguments:
    #   $1 - app - Application name
    #   $2 - base_dir - Base directory (default: /firestream)
    # Returns:
    #   New generation number (stdout)
    #########################
    increment_generation() {
        local app="''${1:?app name is missing}"
        local base_dir="''${2:-/firestream}"
        local state_dir
        state_dir="$(get_state_dir "$app" "$base_dir")"
        local gen_file="''${state_dir}/generation"

        ensure_dir_exists "$state_dir"

        local current_gen
        current_gen="$(get_generation "$app" "$base_dir")"
        local new_gen=$((current_gen + 1))

        ${pkgs.coreutils}/bin/echo "$new_gen" > "$gen_file"
        debug "Incremented generation for $app: $current_gen -> $new_gen"
        ${pkgs.coreutils}/bin/echo "$new_gen"
    }

    ########################
    # Record successful activation
    # Arguments:
    #   $1 - app - Application name
    #   $2 - base_dir - Base directory (default: /firestream)
    # Creates:
    #   activation-time file with ISO timestamp
    # Side effects:
    #   Increments generation counter
    #########################
    record_activation() {
        local app="''${1:?app name is missing}"
        local base_dir="''${2:-/firestream}"
        local state_dir
        state_dir="$(get_state_dir "$app" "$base_dir")"
        local activation_file="''${state_dir}/activation-time"

        ensure_dir_exists "$state_dir"

        # Record ISO timestamp
        local timestamp
        timestamp="$(${pkgs.coreutils}/bin/date -Iseconds)"
        ${pkgs.coreutils}/bin/echo "$timestamp" > "$activation_file"

        # Increment generation
        local new_gen
        new_gen="$(increment_generation "$app" "$base_dir")"

        info "Recorded activation for $app at $timestamp (generation $new_gen)"
    }

    ########################
    # Check if build-time prepopulated marker exists
    # Arguments:
    #   $1 - app - Application name
    #   $2 - base_dir - Base directory (default: /firestream)
    # Returns:
    #   0 if prepopulated marker exists, 1 if not
    #########################
    check_prepopulated() {
        local app="''${1:?app name is missing}"
        local base_dir="''${2:-/firestream}"
        local state_dir
        state_dir="$(get_state_dir "$app" "$base_dir")"
        local marker_file="''${state_dir}/prepopulated"

        if [[ -f "$marker_file" ]]; then
            debug "Prepopulated marker exists for $app"
            return 0
        else
            debug "No prepopulated marker for $app"
            return 1
        fi
    }

    ########################
    # Create prepopulated marker (typically called during build phase)
    # Arguments:
    #   $1 - app - Application name
    #   $2 - base_dir - Base directory (default: /firestream)
    # Creates:
    #   prepopulated marker file
    #########################
    mark_prepopulated() {
        local app="''${1:?app name is missing}"
        local base_dir="''${2:-/firestream}"
        local state_dir
        state_dir="$(get_state_dir "$app" "$base_dir")"
        local marker_file="''${state_dir}/prepopulated"

        ensure_dir_exists "$state_dir"
        ${pkgs.coreutils}/bin/touch "$marker_file"
        info "Marked $app as prepopulated"
    }

    ########################
    # Get last activation time
    # Arguments:
    #   $1 - app - Application name
    #   $2 - base_dir - Base directory (default: /firestream)
    # Returns:
    #   ISO timestamp (stdout) or empty string if never activated
    #########################
    get_activation_time() {
        local app="''${1:?app name is missing}"
        local base_dir="''${2:-/firestream}"
        local state_dir
        state_dir="$(get_state_dir "$app" "$base_dir")"
        local activation_file="''${state_dir}/activation-time"

        if [[ -f "$activation_file" ]]; then
            ${pkgs.coreutils}/bin/cat "$activation_file"
        else
            debug "No activation time recorded for $app"
            ${pkgs.coreutils}/bin/echo ""
        fi
    }

    ########################
    # Clear all state for an application
    # Arguments:
    #   $1 - app - Application name
    #   $2 - base_dir - Base directory (default: /firestream)
    # Side effects:
    #   Removes entire .state directory
    #########################
    clear_state() {
        local app="''${1:?app name is missing}"
        local base_dir="''${2:-/firestream}"
        local state_dir
        state_dir="$(get_state_dir "$app" "$base_dir")"

        if [[ -d "$state_dir" ]]; then
            warn "Clearing all state for $app"
            ${pkgs.coreutils}/bin/rm -rf "$state_dir"
            info "State cleared for $app"
        else
            debug "No state directory to clear for $app"
        fi
    }
  '';
in
{
  meta = {
    name = "libstate";
    description = "Application state tracking and generation management";
    version = "1.0.0";
  };

  imports = [ logModule fsModule configModule ];
  runtimeDeps = with pkgs; [ coreutils ];
  inherit functions;
  exports = [
    "get_state_dir"
    "get_config_hash"
    "save_config_hash"
    "has_config_changed"
    "get_generation"
    "increment_generation"
    "record_activation"
    "check_prepopulated"
    "mark_prepopulated"
    "get_activation_time"
    "clear_state"
  ];

  script = pkgs.writeTextDir "opt/firestream/scripts/libstate.sh" ''
    #!/bin/bash
    # Copyright Firestream. Apache-2.0 License.
    # Generated by Nix - do not edit directly.
    ${functions}
  '';
}
