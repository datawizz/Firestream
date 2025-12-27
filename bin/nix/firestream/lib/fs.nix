# fs.nix - File system operations and utilities
{ pkgs, lib, logModule }:

let
  functions = ''
    # Import log functions
    ${logModule.functions}

    ########################
    # Ensure a file/directory is owned (user and group) by the given user
    # Arguments:
    #   $1 - filepath
    #   $2 - owner
    #   $3 - group (optional, defaults to owner)
    # Returns:
    #   None
    #########################
    owned_by() {
        local path="''${1:?path is missing}"
        local owner="''${2:?owner is missing}"
        local group="''${3:-}"

        if [[ -n $group ]]; then
            ${pkgs.coreutils}/bin/chown "$owner":"$group" "$path"
        else
            ${pkgs.coreutils}/bin/chown "$owner":"$owner" "$path"
        fi
    }

    ########################
    # Ensure a directory exists and, optionally, is owned by the given user
    # Arguments:
    #   $1 - directory
    #   $2 - owner user (optional)
    #   $3 - owner group (optional)
    # Returns:
    #   None
    #########################
    ensure_dir_exists() {
        local dir="''${1:?directory is missing}"
        local owner_user="''${2:-}"
        local owner_group="''${3:-}"

        [ -d "''${dir}" ] || ${pkgs.coreutils}/bin/mkdir -p "''${dir}"
        if [[ -n $owner_user ]]; then
            owned_by "$dir" "$owner_user" "$owner_group"
        fi
    }

    ########################
    # Checks whether a directory is empty or not
    # Arguments:
    #   $1 - directory
    # Returns:
    #   Boolean
    #########################
    is_dir_empty() {
        local -r path="''${1:?missing directory}"
        # Calculate real path in order to avoid issues with symlinks
        local -r dir="$(${pkgs.coreutils}/bin/realpath "$path")"
        if [[ ! -e "$dir" ]] || [[ -z "$(${pkgs.coreutils}/bin/ls -A "$dir")" ]]; then
            true
        else
            false
        fi
    }

    ########################
    # Checks whether a mounted directory is empty or not
    # Arguments:
    #   $1 - directory
    # Returns:
    #   Boolean
    #########################
    is_mounted_dir_empty() {
        local dir="''${1:?missing directory}"

        if is_dir_empty "$dir" || ${pkgs.findutils}/bin/find "$dir" -mindepth 1 -maxdepth 1 -not -name ".snapshot" -not -name "lost+found" -exec false {} +; then
            true
        else
            false
        fi
    }

    ########################
    # Checks whether a file can be written to or not
    # Arguments:
    #   $1 - file
    # Returns:
    #   Boolean
    #########################
    is_file_writable() {
        local file="''${1:?missing file}"
        local dir
        dir="$(${pkgs.coreutils}/bin/dirname "$file")"

        if [[ (-f "$file" && -w "$file") || (! -f "$file" && -d "$dir" && -w "$dir") ]]; then
            true
        else
            false
        fi
    }

    ########################
    # Configure permissions and ownership recursively
    # Globals:
    #   None
    # Arguments:
    #   $1 - paths (as a string)
    # Flags:
    #   -f|--file-mode - mode for files
    #   -d|--dir-mode - mode for directories
    #   -u|--user - user
    #   -g|--group - group
    # Returns:
    #   None
    #########################
    configure_permissions_ownership() {
        local -r paths="''${1:?paths is missing}"
        local dir_mode=""
        local file_mode=""
        local user=""
        local group=""

        # Validate arguments
        shift 1
        while [ "$#" -gt 0 ]; do
            case "$1" in
            -f | --file-mode)
                shift
                file_mode="''${1:?missing mode for files}"
                ;;
            -d | --dir-mode)
                shift
                dir_mode="''${1:?missing mode for directories}"
                ;;
            -u | --user)
                shift
                user="''${1:?missing user}"
                ;;
            -g | --group)
                shift
                group="''${1:?missing group}"
                ;;
            *)
                ${pkgs.coreutils}/bin/echo "Invalid command line flag $1" >&2
                return 1
                ;;
            esac
            shift
        done

        read -r -a filepaths <<<"$paths"
        for p in "''${filepaths[@]}"; do
            if [[ -e "$p" ]]; then
                ${pkgs.findutils}/bin/find -L "$p" -printf ""
                if [[ -n $dir_mode ]]; then
                    ${pkgs.findutils}/bin/find -L "$p" -type d ! -perm "$dir_mode" -print0 | ${pkgs.findutils}/bin/xargs -r -0 ${pkgs.coreutils}/bin/chmod "$dir_mode"
                fi
                if [[ -n $file_mode ]]; then
                    ${pkgs.findutils}/bin/find -L "$p" -type f ! -perm "$file_mode" -print0 | ${pkgs.findutils}/bin/xargs -r -0 ${pkgs.coreutils}/bin/chmod "$file_mode"
                fi
                if [[ -n $user ]] && [[ -n $group ]]; then
                    ${pkgs.findutils}/bin/find -L "$p" -print0 | ${pkgs.findutils}/bin/xargs -r -0 ${pkgs.coreutils}/bin/chown "''${user}:''${group}"
                elif [[ -n $user ]] && [[ -z $group ]]; then
                    ${pkgs.findutils}/bin/find -L "$p" -print0 | ${pkgs.findutils}/bin/xargs -r -0 ${pkgs.coreutils}/bin/chown "''${user}"
                elif [[ -z $user ]] && [[ -n $group ]]; then
                    ${pkgs.findutils}/bin/find -L "$p" -print0 | ${pkgs.findutils}/bin/xargs -r -0 ${pkgs.coreutils}/bin/chgrp "''${group}"
                fi
            else
                stderr_print "$p does not exist"
            fi
        done
    }
  '';
in
{
  meta = {
    name = "libfs";
    description = "File system operations and utilities";
    version = "1.0.0";
  };

  imports = [ logModule ];
  runtimeDeps = with pkgs; [ coreutils findutils ];
  inherit functions;
  exports = [
    "owned_by"
    "ensure_dir_exists"
    "is_dir_empty"
    "is_mounted_dir_empty"
    "is_file_writable"
    "configure_permissions_ownership"
  ];

  script = pkgs.writeTextDir "opt/firestream/scripts/libfs.sh" ''
    #!/bin/bash
    # Copyright Firestream. Apache-2.0 License.
    # Generated by Nix - do not edit directly.
    ${functions}
  '';
}
