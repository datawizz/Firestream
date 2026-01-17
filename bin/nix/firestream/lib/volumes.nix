# volumes.nix - Runtime directory schema and management
# Copyright Firestream. Apache-2.0 License.
#
# This module provides a declarative schema for runtime directories in containers.
# It replaces the ad-hoc prepopulateDirs list with a structured specification that
# captures directory type, persistence mode, permissions, and ownership.
#
# Usage:
#   runtimeDirs = {
#     logs = {
#       path = "/opt/app/logs";
#       type = "logs";
#       persistence = "persistent";
#       mode = "0755";
#       owner = "app";
#       group = "app";
#       description = "Application logs";
#     };
#   };

{ pkgs, lib, logModule, fsModule }:

let
  # Valid enumeration values
  validTypes = [ "logs" "tmp" "data" "cache" "conf" "plugins" "work" "state" "custom" ];
  validPersistence = [ "ephemeral" "persistent" "tmpfs" ];

  # Default values for optional fields
  defaultDir = {
    type = "custom";
    persistence = "ephemeral";
    mode = "0755";
    owner = null;
    group = null;
    createParents = true;
    description = null;
  };

  # Normalize a directory specification (merge with defaults)
  normalizeDir = name: spec:
    defaultDir // spec // {
      # Ensure path is set (required field)
      path = spec.path or (throw "runtimeDirs.${name}: 'path' is required");
    };

  # Validate a single directory specification
  validateDir = name: spec:
    let
      normalized = normalizeDir name spec;
    in
    assert lib.assertMsg (builtins.isString normalized.path)
      "runtimeDirs.${name}: 'path' must be a string";
    assert lib.assertMsg (lib.hasPrefix "/" normalized.path)
      "runtimeDirs.${name}: 'path' must be an absolute path (got '${normalized.path}')";
    assert lib.assertMsg (lib.elem normalized.type validTypes)
      "runtimeDirs.${name}: 'type' must be one of ${builtins.toString validTypes}, got '${normalized.type}'";
    assert lib.assertMsg (lib.elem normalized.persistence validPersistence)
      "runtimeDirs.${name}: 'persistence' must be one of ${builtins.toString validPersistence}, got '${normalized.persistence}'";
    assert lib.assertMsg (builtins.match "^[0-7]{3,4}$" normalized.mode != null)
      "runtimeDirs.${name}: 'mode' must be a valid Unix permission (e.g., '0755'), got '${normalized.mode}'";
    assert lib.assertMsg (normalized.owner == null || builtins.isString normalized.owner || builtins.isInt normalized.owner)
      "runtimeDirs.${name}: 'owner' must be a string, integer (UID), or null";
    assert lib.assertMsg (normalized.group == null || builtins.isString normalized.group || builtins.isInt normalized.group)
      "runtimeDirs.${name}: 'group' must be a string, integer (GID), or null";
    normalized;

  # Convert prepopulateDirs (legacy format) to runtimeDirs
  # This provides backward compatibility with existing modules
  fromPrepopulateDirs = dirs:
    lib.listToAttrs (lib.imap0 (i: path: {
      name = "legacyDir${builtins.toString i}";
      value = {
        inherit path;
        type = "custom";
        persistence = "ephemeral";
        mode = "0755";
      };
    }) dirs);

  # Generate bash code to create a single directory at runtime
  # user: the user config { name, uid, gid } for resolving ownership to UID/GID
  mkDirCreationCode = user: name: spec:
    let
      normalized = validateDir name spec;
      hasOwner = normalized.owner != null;
      # Resolve owner to UID string:
      # - If integer, use directly (numeric UID)
      # - If string matching user.name, resolve to user.uid
      # - Otherwise use the string as-is (for other users)
      ownerUid = if hasOwner then
                   if builtins.isInt normalized.owner then builtins.toString normalized.owner
                   else if normalized.owner == user.name then builtins.toString user.uid
                   else normalized.owner
                 else "";
      # Resolve group to GID string:
      # - If integer, use directly (numeric GID)
      # - If string matching user.name or user.group, resolve to user.gid
      # - Otherwise use the string as-is
      # - If no group but has owner, use owner's value
      groupGid = if normalized.group != null then
                   if builtins.isInt normalized.group then builtins.toString normalized.group
                   else if normalized.group == user.name || normalized.group == (user.group or user.name) then builtins.toString user.gid
                   else normalized.group
                 else if hasOwner then ownerUid
                 else "";
      ownerArg = if hasOwner then ''"${ownerUid}" "${groupGid}"'' else "";
      descComment = if normalized.description != null
        then "# ${normalized.description}"
        else "";
    in ''
      # Create ${name}: ${normalized.path}
      ${descComment}
      ensure_dir_exists "${normalized.path}"${if hasOwner then " ${ownerArg}" else ""}
      ${pkgs.coreutils}/bin/chmod ${normalized.mode} "${normalized.path}"
      debug "Created directory: ${normalized.path} (${normalized.type}, ${normalized.persistence})"
    '';

  # Generate complete bash initialization code for all directories
  # user: the user config { name, uid, gid } for resolving ownership to UID/GID
  mkAllDirsCreationCode = user: runtimeDirs:
    let
      normalized = lib.mapAttrs validateDir runtimeDirs;
      # Sort by path length to create parents before children
      sortedDirs = lib.sort (a: b:
        builtins.stringLength a.value.path < builtins.stringLength b.value.path
      ) (lib.mapAttrsToList (name: spec: { inherit name; value = spec; }) normalized);
    in ''
      ########################
      # Create runtime directories
      # Generated from runtimeDirs schema
      #########################
      create_runtime_dirs() {
          info "Creating runtime directories..."

          ${lib.concatMapStringsSep "\n" ({ name, value }:
            mkDirCreationCode user name value
          ) sortedDirs}

          info "Runtime directories created successfully"
      }
    '';

  # Extract volume hints for persistent directories
  # These can be used by orchestration layers for documentation
  extractVolumeHints = runtimeDirs:
    let
      normalized = lib.mapAttrs validateDir runtimeDirs;
      persistent = lib.filterAttrs (name: spec: spec.persistence == "persistent") normalized;
    in lib.mapAttrsToList (name: spec: {
      path = spec.path;
      type = spec.type;
      name = name;
      description = spec.description;
    }) persistent;

  # Extract tmpfs hints
  extractTmpfsHints = runtimeDirs:
    let
      normalized = lib.mapAttrs validateDir runtimeDirs;
      tmpfsDirs = lib.filterAttrs (name: spec: spec.persistence == "tmpfs") normalized;
    in lib.mapAttrsToList (name: spec: {
      path = spec.path;
      type = spec.type;
      name = name;
    }) tmpfsDirs;

  # Get all directory paths from runtimeDirs (for prepopulation compatibility)
  getAllPaths = runtimeDirs:
    let
      normalized = lib.mapAttrs validateDir runtimeDirs;
    in lib.mapAttrsToList (name: spec: spec.path) normalized;

  # Shell functions for runtime directory management
  functions = ''
    ########################
    # Create a directory with proper permissions and ownership
    # Arguments:
    #   $1 - path
    #   $2 - mode (e.g., "0755")
    #   $3 - owner (optional)
    #   $4 - group (optional)
    # Returns:
    #   None
    #########################
    create_runtime_dir() {
        local path="''${1:?path is missing}"
        local mode="''${2:-0755}"
        local owner="''${3:-}"
        local group="''${4:-}"

        # Create directory if it doesn't exist
        if [[ ! -d "$path" ]]; then
            ${pkgs.coreutils}/bin/mkdir -p "$path"
            debug "Created directory: $path"
        fi

        # Set permissions
        ${pkgs.coreutils}/bin/chmod "$mode" "$path"

        # Set ownership if specified
        if [[ -n "$owner" ]]; then
            if [[ -n "$group" ]]; then
                ${pkgs.coreutils}/bin/chown "$owner":"$group" "$path"
            else
                ${pkgs.coreutils}/bin/chown "$owner":"$owner" "$path"
            fi
        fi
    }

    ########################
    # Verify directory exists and has correct permissions
    # Arguments:
    #   $1 - path
    #   $2 - expected mode (e.g., "0755") (optional)
    #   $3 - expected owner (optional)
    # Returns:
    #   0 if directory is correct, 1 otherwise
    #########################
    verify_runtime_dir() {
        local path="''${1:?path is missing}"
        local expected_mode="''${2:-}"
        local expected_owner="''${3:-}"

        if [[ ! -d "$path" ]]; then
            warn "Directory does not exist: $path"
            return 1
        fi

        if [[ -n "$expected_mode" ]]; then
            local actual_mode
            actual_mode=$(${pkgs.coreutils}/bin/stat -c "%a" "$path")
            # Remove leading zero for comparison
            expected_mode="''${expected_mode#0}"
            if [[ "$actual_mode" != "$expected_mode" ]]; then
                warn "Directory $path has mode $actual_mode, expected $expected_mode"
                return 1
            fi
        fi

        if [[ -n "$expected_owner" ]]; then
            local actual_owner
            actual_owner=$(${pkgs.coreutils}/bin/stat -c "%U" "$path")
            if [[ "$actual_owner" != "$expected_owner" ]]; then
                warn "Directory $path owned by $actual_owner, expected $expected_owner"
                return 1
            fi
        fi

        debug "Directory verified: $path"
        return 0
    }

    ########################
    # Check if a path should be treated as a volume mount point
    # Arguments:
    #   $1 - path to check
    # Returns:
    #   0 if the path appears to be a mount point, 1 otherwise
    #########################
    is_likely_mount_point() {
        local path="''${1:?path is missing}"

        # Check if it's a mount point
        if ${pkgs.util-linux}/bin/mountpoint -q "$path" 2>/dev/null; then
            return 0
        fi

        # Check if it's a symlink (often used for volume mounts)
        if [[ -L "$path" ]]; then
            return 0
        fi

        return 1
    }
  '';

in {
  meta = {
    name = "libvolumes";
    description = "Runtime directory schema and management";
    version = "1.0.0";
  };

  imports = [ logModule fsModule ];
  runtimeDeps = with pkgs; [ coreutils util-linux ];
  inherit functions;
  exports = [
    "create_runtime_dir"
    "verify_runtime_dir"
    "is_likely_mount_point"
  ];

  # Schema constants
  inherit validTypes validPersistence defaultDir;

  # Validation and normalization functions
  inherit validateDir normalizeDir;

  # Conversion from legacy format
  inherit fromPrepopulateDirs;

  # Path extraction for compatibility
  inherit getAllPaths;

  # Code generation functions
  inherit mkDirCreationCode mkAllDirsCreationCode;

  # Volume hint extraction for orchestration
  inherit extractVolumeHints extractTmpfsHints;

  script = pkgs.writeTextDir "opt/firestream/scripts/libvolumes.sh" ''
    #!/bin/bash
    # Copyright Firestream. Apache-2.0 License.
    # Generated by Nix - do not edit directly.
    ${functions}
  '';
}
