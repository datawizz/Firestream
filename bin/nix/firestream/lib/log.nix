# log.nix - Logging functions with color support and debug mode
{ pkgs, lib }:

let
  functions = ''
    # Constants
    RESET='\033[0m'
    RED='\033[38;5;1m'
    GREEN='\033[38;5;2m'
    YELLOW='\033[38;5;3m'
    MAGENTA='\033[38;5;5m'
    CYAN='\033[38;5;6m'

    ########################
    # Print to STDERR
    # Arguments:
    #   Message to print
    # Returns:
    #   None
    #########################
    stderr_print() {
        # Check FIRESTREAM_QUIET
        local bool="''${FIRESTREAM_QUIET:-false}"
        # comparison is performed without regard to the case of alphabetic characters
        shopt -s nocasematch
        if ! [[ "$bool" = 1 || "$bool" =~ ^(yes|true)$ ]]; then
            ${pkgs.coreutils}/bin/printf "%b\\n" "''${*}" >&2
        fi
    }

    ########################
    # Log message
    # Arguments:
    #   Message to log
    # Returns:
    #   None
    #########################
    log() {
        # Check FIRESTREAM_COLOR
        local color_bool="''${FIRESTREAM_COLOR:-true}"
        # comparison is performed without regard to the case of alphabetic characters
        shopt -s nocasematch
        if [[ "$color_bool" = 1 || "$color_bool" =~ ^(yes|true)$ ]]; then
            stderr_print "''${CYAN}''${MODULE:-} ''${MAGENTA}$(${pkgs.coreutils}/bin/date "+%T.%2N ")''${RESET}''${*}"
        else
            stderr_print "''${MODULE:-} $(${pkgs.coreutils}/bin/date "+%T.%2N ")''${*}"
        fi
    }

    ########################
    # Log an 'info' message
    # Arguments:
    #   Message to log
    # Returns:
    #   None
    #########################
    info() {
        local msg_color=""
        # Check FIRESTREAM_COLOR
        local color_bool="''${FIRESTREAM_COLOR:-true}"
        # comparison is performed without regard to the case of alphabetic characters
        shopt -s nocasematch
        if [[ "$color_bool" = 1 || "$color_bool" =~ ^(yes|true)$ ]]; then
            msg_color="$GREEN"
        fi
        log "''${msg_color}INFO ''${RESET} ==> ''${*}"
    }

    ########################
    # Log a 'warn' message
    # Arguments:
    #   Message to log
    # Returns:
    #   None
    #########################
    warn() {
        local msg_color=""
        # Check FIRESTREAM_COLOR
        local color_bool="''${FIRESTREAM_COLOR:-true}"
        # comparison is performed without regard to the case of alphabetic characters
        shopt -s nocasematch
        if [[ "$color_bool" = 1 || "$color_bool" =~ ^(yes|true)$ ]]; then
            msg_color="$YELLOW"
        fi
        log "''${msg_color}WARN ''${RESET} ==> ''${*}"
    }

    ########################
    # Log an 'error' message
    # Arguments:
    #   Message to log
    # Returns:
    #   None
    #########################
    error() {
        local msg_color=""
        # Check FIRESTREAM_COLOR
        local color_bool="''${FIRESTREAM_COLOR:-true}"
        # comparison is performed without regard to the case of alphabetic characters
        shopt -s nocasematch
        if [[ "$color_bool" = 1 || "$color_bool" =~ ^(yes|true)$ ]]; then
            msg_color="$RED"
        fi
        log "''${msg_color}ERROR''${RESET} ==> ''${*}"
    }

    ########################
    # Log a 'debug' message
    # Globals:
    #   FIRESTREAM_DEBUG
    # Arguments:
    #   Message to log
    # Returns:
    #   None
    #########################
    debug() {
        local msg_color=""
        # Check FIRESTREAM_COLOR
        local color_bool="''${FIRESTREAM_COLOR:-true}"
        # comparison is performed without regard to the case of alphabetic characters
        shopt -s nocasematch
        if [[ "$color_bool" = 1 || "$color_bool" =~ ^(yes|true)$ ]]; then
            msg_color="$MAGENTA"
        fi
        # Check FIRESTREAM_DEBUG
        local debug_bool="''${FIRESTREAM_DEBUG:-false}"
        if [[ "$debug_bool" = 1 || "$debug_bool" =~ ^(yes|true)$ ]]; then
            log "''${msg_color}DEBUG''${RESET} ==> ''${*}"
        fi
    }

    ########################
    # Indent a string
    # Arguments:
    #   $1 - string
    #   $2 - number of indentation characters (default: 4)
    #   $3 - indentation character (default: " ")
    # Returns:
    #   None
    #########################
    indent() {
        local string="''${1:-}"
        local num="''${2:?missing num}"
        local char="''${3:-" "}"
        # Build the indentation unit string
        local indent_unit=""
        for ((i = 0; i < num; i++)); do
            indent_unit="''${indent_unit}''${char}"
        done
        # shellcheck disable=SC2001
        # Complex regex, see https://github.com/koalaman/shellcheck/wiki/SC2001#exceptions
        ${pkgs.coreutils}/bin/echo "$string" | ${pkgs.gnused}/bin/sed "s/^/''${indent_unit}/"
    }
  '';
in
{
  meta = {
    name = "liblog";
    description = "Logging functions with color support and debug mode";
    version = "1.0.0";
  };

  imports = [];
  runtimeDeps = with pkgs; [ coreutils gnused ];
  inherit functions;
  exports = [ "stderr_print" "log" "info" "warn" "error" "debug" "indent" ];

  script = pkgs.writeTextDir "opt/firestream/scripts/liblog.sh" ''
    #!/bin/bash
    # Copyright Firestream. Apache-2.0 License.
    # Generated by Nix - do not edit directly.
    ${functions}
  '';
}
