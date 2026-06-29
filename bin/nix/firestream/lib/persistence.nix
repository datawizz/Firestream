# persistence.nix - Application persistence management
{ pkgs, lib, logModule, fsModule }:

let
  functions = ''
    # Import dependencies
    ${logModule.functions}
    ${fsModule.functions}

    ########################
    # Persist application data to volume
    # Creates symlinks from source to destination for persistence
    # Arguments:
    #   $1 - app - Application name
    #   $2 - source_dir - Source directory to persist
    #   $3 - volume_dir - Volume directory for persistence
    # Returns:
    #   None
    #########################
    persist_app() {
        local app="''${1:?app name is missing}"
        local source_dir="''${2:?source directory is missing}"
        local volume_dir="''${3:?volume directory is missing}"

        info "Persisting ''${app} data from ''${source_dir} to ''${volume_dir}"

        # Ensure volume directory exists
        ensure_dir_exists "$volume_dir"

        # Check if this is first run (source exists but volume is empty)
        if [[ -e "$source_dir" ]] && is_dir_empty "$volume_dir"; then
            debug "First run detected - copying ''${source_dir} to ''${volume_dir}"
            # Copy with archive mode to preserve ACLs and permissions
            ${pkgs.coreutils}/bin/cp -a "$source_dir/." "$volume_dir/"
        fi

        # If source exists and is not a symlink, back it up and create symlink
        if [[ -e "$source_dir" ]] && [[ ! -L "$source_dir" ]]; then
            debug "Backing up original ''${source_dir}"
            ${pkgs.coreutils}/bin/mv "$source_dir" "''${source_dir}.bak"
        fi

        # Create symlink from source to volume
        if [[ ! -e "$source_dir" ]]; then
            debug "Creating symlink: ''${source_dir} -> ''${volume_dir}"
            ${pkgs.coreutils}/bin/ln -sfn "$volume_dir" "$source_dir"
        fi

        info "Successfully persisted ''${app} data"
    }

    ########################
    # Restore application from persisted data
    # Recreates symlinks from volume back to application directories
    # Arguments:
    #   $1 - app - Application name
    #   $2 - source_dir - Source directory to restore to
    #   $3 - volume_dir - Volume directory containing persisted data
    # Returns:
    #   None
    #########################
    restore_persisted_app() {
        local app="''${1:?app name is missing}"
        local source_dir="''${2:?source directory is missing}"
        local volume_dir="''${3:?volume directory is missing}"

        info "Restoring ''${app} data from ''${volume_dir} to ''${source_dir}"

        # Check if volume directory exists and has data
        if [[ ! -e "$volume_dir" ]] || is_dir_empty "$volume_dir"; then
            warn "No persisted data found in ''${volume_dir}"
            return 1
        fi

        # Remove existing source if it's not a symlink
        if [[ -e "$source_dir" ]] && [[ ! -L "$source_dir" ]]; then
            debug "Removing non-symlinked ''${source_dir}"
            ${pkgs.coreutils}/bin/rm -rf "$source_dir"
        fi

        # Create parent directory if it doesn't exist
        local parent_dir
        parent_dir="$(${pkgs.coreutils}/bin/dirname "$source_dir")"
        ensure_dir_exists "$parent_dir"

        # Create symlink from source to volume
        debug "Creating symlink: ''${source_dir} -> ''${volume_dir}"
        ${pkgs.coreutils}/bin/ln -sfn "$volume_dir" "$source_dir"

        info "Successfully restored ''${app} data"
    }

    ########################
    # Check if application has been initialized
    # Arguments:
    #   $1 - app - Application name
    #   $2 - base_dir - Base directory (default: /firestream)
    # Returns:
    #   0 if initialized, 1 if not
    #########################
    is_app_initialized() {
        local app="''${1:?app name is missing}"
        local base_dir="''${2:-/firestream}"
        local marker_file="''${base_dir}/''${app}/.app_initialized"

        if [[ -f "$marker_file" ]]; then
            debug "Application ''${app} is already initialized"
            return 0
        else
            debug "Application ''${app} is not initialized"
            return 1
        fi
    }

    ########################
    # Mark application as initialized
    # Arguments:
    #   $1 - app - Application name
    #   $2 - base_dir - Base directory (default: /firestream)
    # Returns:
    #   None
    #########################
    mark_app_initialized() {
        local app="''${1:?app name is missing}"
        local base_dir="''${2:-/firestream}"
        local marker_file="''${base_dir}/''${app}/.app_initialized"
        local marker_dir
        marker_dir="$(${pkgs.coreutils}/bin/dirname "$marker_file")"

        debug "Marking ''${app} as initialized"
        # Tolerate read-only base_dir (Bitnami chart pods often have no PVC at
        # /firestream and run with readOnlyRootFilesystem=true). The
        # is_app_initialized check on next boot will still return false, which
        # is OK — re-running initialization is idempotent. ensure_dir_exists
        # and touch failures are non-fatal.
        ensure_dir_exists "$marker_dir" 2>/dev/null || {
            debug "Skipped mark_app_initialized: base_dir not writable"
            return 0
        }
        ${pkgs.coreutils}/bin/touch "$marker_file" 2>/dev/null || {
            debug "Skipped mark_app_initialized: marker_file not writable"
            return 0
        }
        info "Application ''${app} marked as initialized"
    }

    ########################
    # Get list of directories that should be persisted for an app
    # Arguments:
    #   $1 - app - Application name
    #   $2 - base_dir - Base directory (default: /firestream)
    # Returns:
    #   Array of directory paths (printed to stdout)
    #########################
    get_persisted_dirs() {
        local app="''${1:?app name is missing}"
        local base_dir="''${2:-/firestream}"

        # Common persistence patterns
        local dirs=(
            "''${base_dir}/''${app}/data"
            "''${base_dir}/''${app}/conf"
            "''${base_dir}/''${app}/logs"
        )

        # App-specific persistence directories
        case "$app" in
            postgresql)
                dirs+=(
                    "''${base_dir}/postgresql/data"
                    "''${base_dir}/postgresql/conf"
                )
                ;;
            airflow)
                dirs+=(
                    "''${base_dir}/airflow/dags"
                    "''${base_dir}/airflow/logs"
                    "''${base_dir}/airflow/plugins"
                )
                ;;
            kafka)
                dirs+=(
                    "''${base_dir}/kafka/data"
                    "''${base_dir}/kafka/logs"
                )
                ;;
            redis)
                dirs+=(
                    "''${base_dir}/redis/data"
                )
                ;;
            spark)
                dirs+=(
                    "''${base_dir}/spark/work"
                    "''${base_dir}/spark/logs"
                )
                ;;
            odoo)
                dirs+=(
                    "''${base_dir}/odoo/data"
                    "''${base_dir}/odoo/conf"
                    "''${base_dir}/odoo/addons"
                )
                ;;
            superset)
                dirs+=(
                    "''${base_dir}/superset/data"
                    "''${base_dir}/superset/conf"
                )
                ;;
            gitea)
                dirs+=(
                    "''${base_dir}/gitea/data"
                    "''${base_dir}/gitea/custom"
                )
                ;;
        esac

        # Print directories that exist
        for dir in "''${dirs[@]}"; do
            if [[ -e "$dir" ]]; then
                ${pkgs.coreutils}/bin/echo "$dir"
            fi
        done
    }
  '';
in
{
  meta = {
    name = "libpersistence";
    description = "Application persistence management";
    version = "1.0.0";
  };

  imports = [ logModule fsModule ];
  runtimeDeps = with pkgs; [ coreutils findutils ];
  inherit functions;
  exports = [
    "persist_app"
    "restore_persisted_app"
    "is_app_initialized"
    "mark_app_initialized"
    "get_persisted_dirs"
  ];

  script = pkgs.writeTextDir "opt/firestream/scripts/libpersistence.sh" ''
    #!/bin/bash
    # Copyright Firestream. MIT License.
    # Generated by Nix - do not edit directly.
    ${functions}
  '';
}
