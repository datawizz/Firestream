# file.nix - File manipulation utilities (ConfigMap-safe)
{ pkgs, lib, logModule }:

let
  functions = ''
    # Import log functions
    ${logModule.functions}

    ########################
    # Replace a string in a file (ConfigMap-safe - no sed -i)
    # Arguments:
    #   $1 - match - String to search for
    #   $2 - substitution - String to replace with
    #   $3 - filename - File to modify
    # Returns:
    #   None
    #########################
    replace_in_file() {
        local match="''${1:?match pattern is missing}"
        local substitution="''${2:?substitution string is missing}"
        local filename="''${3:?filename is missing}"

        # Find a delimiter character not used in match or substitution
        local del="/"
        for d in "/" "|" "," ":" ";" "@" "#" "%" "^" "~"; do
            if [[ ! "$match" =~ $d ]] && [[ ! "$substitution" =~ $d ]]; then
                del="$d"
                break
            fi
        done

        # ConfigMap-safe: read, process, write (no sed -i)
        local result
        result="$(${pkgs.gnused}/bin/sed "s''${del}''${match}''${del}''${substitution}''${del}g" "$filename")"
        ${pkgs.coreutils}/bin/echo "$result" > "$filename"
    }

    ########################
    # Replace a multiline pattern in a file (ConfigMap-safe)
    # Arguments:
    #   $1 - match - Regex pattern to search for (multiline)
    #   $2 - substitution - String to replace with
    #   $3 - filename - File to modify
    # Returns:
    #   None
    #########################
    replace_in_file_multiline() {
        local match="''${1:?match pattern is missing}"
        local substitution="''${2:?substitution string is missing}"
        local filename="''${3:?filename is missing}"

        # ConfigMap-safe: read entire file, process with perl, write back
        local result
        result="$(${pkgs.perl}/bin/perl -0pe "s''${match}''${substitution}g" "$filename")"
        ${pkgs.coreutils}/bin/echo "$result" > "$filename"
    }

    ########################
    # Remove lines matching a pattern from a file (ConfigMap-safe)
    # Arguments:
    #   $1 - pattern - Pattern to match for deletion
    #   $2 - filename - File to modify
    # Returns:
    #   None
    #########################
    remove_in_file() {
        local pattern="''${1:?pattern is missing}"
        local filename="''${2:?filename is missing}"

        # Find a delimiter character not used in pattern
        local del="/"
        for d in "/" "|" "," ":" ";" "@" "#" "%" "^" "~"; do
            if [[ ! "$pattern" =~ $d ]]; then
                del="$d"
                break
            fi
        done

        # ConfigMap-safe: read, delete matching lines, write
        local result
        result="$(${pkgs.gnused}/bin/sed "''${del}''${pattern}''${del}d" "$filename")"
        ${pkgs.coreutils}/bin/echo "$result" > "$filename"
    }

    ########################
    # Append content after the last line matching a pattern
    # Arguments:
    #   $1 - pattern - Pattern to match
    #   $2 - content - Content to append after last match
    #   $3 - filename - File to modify
    # Returns:
    #   None
    #########################
    append_file_after_last_match() {
        local pattern="''${1:?pattern is missing}"
        local content="''${2:?content is missing}"
        local filename="''${3:?filename is missing}"

        # Find the line number of the last match
        local last_match_line
        last_match_line=$(${pkgs.gnugrep}/bin/grep -n "$pattern" "$filename" | ${pkgs.coreutils}/bin/tail -1 | ${pkgs.coreutils}/bin/cut -d: -f1)

        if [[ -z "$last_match_line" ]]; then
            warn "Pattern '$pattern' not found in $filename, appending to end of file"
            ${pkgs.coreutils}/bin/echo "$content" >> "$filename"
        else
            # ConfigMap-safe: read, insert after last match, write
            local result
            result="$(${pkgs.gnused}/bin/sed "''${last_match_line}a\\
$content" "$filename")"
            ${pkgs.coreutils}/bin/echo "$result" > "$filename"
        fi
    }

    ########################
    # Wait for a log entry to appear in a file
    # Arguments:
    #   $1 - log_pattern - Pattern to search for in logs
    #   $2 - log_file - Log file to monitor
    #   $3 - timeout - Timeout in seconds (default: 60)
    # Returns:
    #   0 if pattern found, 1 if timeout
    #########################
    wait_for_log_entry() {
        local log_pattern="''${1:?log pattern is missing}"
        local log_file="''${2:?log file is missing}"
        local timeout="''${3:-60}"

        debug "Waiting for log entry matching '$log_pattern' in $log_file (timeout: ''${timeout}s)"

        # Create a unique temporary file for the tail process
        local tail_pid
        local found=0

        # Use timeout command with tail and grep
        if ${pkgs.coreutils}/bin/timeout "$timeout" ${pkgs.coreutils}/bin/tail -f "$log_file" 2>/dev/null | ${pkgs.gnugrep}/bin/grep -q -m 1 "$log_pattern"; then
            info "Log entry found: $log_pattern"
            return 0
        else
            error "Timeout waiting for log entry: $log_pattern"
            return 1
        fi
    }
  '';
in
{
  meta = {
    name = "libfile";
    description = "File manipulation utilities (ConfigMap-safe)";
    version = "1.0.0";
  };

  imports = [ logModule ];
  runtimeDeps = with pkgs; [ coreutils gnused gnugrep perl ];
  inherit functions;
  exports = [
    "replace_in_file"
    "replace_in_file_multiline"
    "remove_in_file"
    "append_file_after_last_match"
    "wait_for_log_entry"
  ];

  script = pkgs.writeTextDir "opt/firestream/scripts/libfile.sh" ''
    #!/bin/bash
    # Copyright Firestream. Apache-2.0 License.
    # Generated by Nix - do not edit directly.
    ${functions}
  '';
}
