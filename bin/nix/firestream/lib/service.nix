# service.nix - Service management utilities
{ pkgs, lib, logModule, validationsModule }:

let
  functions = ''
    # Import dependencies
    ${logModule.functions}
    ${validationsModule.functions}

    ########################
    # Get PID from file
    # Arguments:
    #   $1 - PID file path
    # Returns:
    #   PID or empty string if invalid
    #########################
    get_pid_from_file() {
        local pid_file="''${1:?pid file is missing}"

        if [[ -f "$pid_file" ]]; then
            local pid
            pid=$(${pkgs.coreutils}/bin/cat "$pid_file" 2>/dev/null | ${pkgs.coreutils}/bin/tr -d '\n\r ')

            # Validate PID is a positive integer
            if is_positive_int "$pid" 2>/dev/null; then
                ${pkgs.coreutils}/bin/echo "$pid"
            fi
        fi
    }

    ########################
    # Check if service is running by PID
    # Arguments:
    #   $1 - PID
    # Returns:
    #   Boolean
    #########################
    is_service_running() {
        local pid="''${1:?pid is missing}"

        if ! is_positive_int "$pid" 2>/dev/null; then
            false
            return
        fi

        # Use kill -0 to check if process exists
        if ${pkgs.coreutils}/bin/kill -0 "$pid" 2>/dev/null; then
            true
        else
            false
        fi
    }

    ########################
    # Stop service gracefully using PID file
    # Arguments:
    #   $1 - PID file path
    # Flags:
    #   --signal - Signal to send (default: TERM)
    #   --timeout - Timeout in seconds to wait for termination (default: 10)
    # Returns:
    #   Boolean
    #########################
    stop_service_using_pid() {
        local pid_file=""
        local signal="TERM"
        local timeout=10

        # Parse arguments
        while [[ "$#" -gt 0 ]]; do
            case "$1" in
                --signal)
                    shift
                    signal="''${1:?missing signal value}"
                    ;;
                --timeout)
                    shift
                    timeout="''${1:?missing timeout value}"
                    ;;
                --)
                    shift
                    break
                    ;;
                -*)
                    stderr_print "unrecognized flag $1"
                    return 1
                    ;;
                *)
                    pid_file="$1"
                    ;;
            esac
            shift
        done

        if [[ -z "$pid_file" ]]; then
            stderr_print "PID file is required"
            return 1
        fi

        local pid
        pid=$(get_pid_from_file "$pid_file")

        if [[ -z "$pid" ]]; then
            warn "PID file $pid_file does not contain a valid PID"
            return 1
        fi

        if ! is_service_running "$pid"; then
            debug "Service with PID $pid is not running"
            ${pkgs.coreutils}/bin/rm -f "$pid_file"
            return 0
        fi

        info "Stopping service with PID $pid (signal: $signal)"
        ${pkgs.coreutils}/bin/kill -s "$signal" "$pid" 2>/dev/null

        # Wait for process to terminate
        local elapsed=0
        while is_service_running "$pid"; do
            if [[ "$elapsed" -ge "$timeout" ]]; then
                warn "Service did not stop within $timeout seconds, sending KILL"
                ${pkgs.coreutils}/bin/kill -9 "$pid" 2>/dev/null
                ${pkgs.coreutils}/bin/sleep 1
                break
            fi
            ${pkgs.coreutils}/bin/sleep 1
            elapsed=$((elapsed + 1))
        done

        if is_service_running "$pid"; then
            error "Failed to stop service with PID $pid"
            return 1
        else
            info "Service stopped successfully"
            ${pkgs.coreutils}/bin/rm -f "$pid_file"
            return 0
        fi
    }

    ########################
    # Start cron daemon
    # Arguments:
    #   None
    # Returns:
    #   None
    #########################
    cron_start() {
        local cron_cmd=""

        # Try to find cron executable
        if [[ -x /usr/sbin/cron ]]; then
            cron_cmd="/usr/sbin/cron"
        elif [[ -x /usr/sbin/crond ]]; then
            cron_cmd="/usr/sbin/crond"
        else
            error "Could not find cron or crond executable"
            return 1
        fi

        info "Starting cron daemon: $cron_cmd"
        "$cron_cmd"
    }

    ########################
    # Generate cron configuration file
    # Arguments:
    #   $1 - output file path
    # Flags:
    #   --run-as - User to run cron job as
    #   --schedule - Cron schedule expression (e.g., "0 * * * *")
    #   --command - Command to execute
    #   --no-clean - Do not clean existing file
    # Returns:
    #   None
    #########################
    generate_cron_conf() {
        local output_file=""
        local run_as=""
        local schedule=""
        local command=""
        local no_clean=""

        # Parse arguments
        while [[ "$#" -gt 0 ]]; do
            case "$1" in
                --run-as)
                    shift
                    run_as="''${1:?missing run-as value}"
                    ;;
                --schedule)
                    shift
                    schedule="''${1:?missing schedule value}"
                    ;;
                --command)
                    shift
                    command="''${1:?missing command value}"
                    ;;
                --no-clean)
                    no_clean="yes"
                    ;;
                --)
                    shift
                    break
                    ;;
                -*)
                    stderr_print "unrecognized flag $1"
                    return 1
                    ;;
                *)
                    output_file="$1"
                    ;;
            esac
            shift
        done

        if [[ -z "$output_file" ]]; then
            stderr_print "output file is required"
            return 1
        fi

        if [[ -z "$schedule" ]]; then
            stderr_print "schedule is required"
            return 1
        fi

        if [[ -z "$command" ]]; then
            stderr_print "command is required"
            return 1
        fi

        # Clean existing file unless --no-clean specified
        if [[ -z "$no_clean" ]] && [[ -f "$output_file" ]]; then
            ${pkgs.coreutils}/bin/rm -f "$output_file"
        fi

        debug "Generating cron configuration: $output_file"

        # Create cron entry
        local cron_entry=""
        if [[ -n "$run_as" ]]; then
            # Format: schedule user command
            cron_entry="$schedule $run_as $command"
        else
            # Format: schedule command
            cron_entry="$schedule $command"
        fi

        ${pkgs.coreutils}/bin/echo "$cron_entry" >> "$output_file"
        info "Cron configuration written to $output_file"
    }

    ########################
    # Generate logrotate configuration file
    # Arguments:
    #   $1 - log file path
    #   $2 - output config file path
    # Flags:
    #   --period - Rotation period (daily/weekly/monthly, default: daily)
    #   --rotations - Number of rotations to keep (default: 14)
    #   --extra - Extra logrotate directives
    # Returns:
    #   None
    #########################
    generate_logrotate_conf() {
        local log_file=""
        local output_file=""
        local period="daily"
        local rotations=14
        local extra=""

        # Parse arguments
        while [[ "$#" -gt 0 ]]; do
            case "$1" in
                --period)
                    shift
                    period="''${1:?missing period value}"
                    ;;
                --rotations)
                    shift
                    rotations="''${1:?missing rotations value}"
                    ;;
                --extra)
                    shift
                    extra="''${1:?missing extra value}"
                    ;;
                --)
                    shift
                    break
                    ;;
                -*)
                    stderr_print "unrecognized flag $1"
                    return 1
                    ;;
                *)
                    if [[ -z "$log_file" ]]; then
                        log_file="$1"
                    elif [[ -z "$output_file" ]]; then
                        output_file="$1"
                    else
                        stderr_print "too many arguments"
                        return 1
                    fi
                    ;;
            esac
            shift
        done

        if [[ -z "$log_file" ]]; then
            stderr_print "log file is required"
            return 1
        fi

        if [[ -z "$output_file" ]]; then
            stderr_print "output file is required"
            return 1
        fi

        # Validate period
        case "$period" in
            daily|weekly|monthly)
                ;;
            *)
                stderr_print "Invalid period: $period (must be daily/weekly/monthly)"
                return 1
                ;;
        esac

        debug "Generating logrotate configuration: $output_file"

        # Generate configuration
        ${pkgs.coreutils}/bin/cat > "$output_file" <<EOF
$log_file {
    $period
    rotate $rotations
    missingok
    notifempty
    compress
    delaycompress
    copytruncate
$(if [[ -n "$extra" ]]; then ${pkgs.coreutils}/bin/echo "    $extra"; fi)
}
EOF

        info "Logrotate configuration written to $output_file"
    }
  '';
in
{
  meta = {
    name = "libservice";
    description = "Service management utilities";
    version = "1.0.0";
  };

  imports = [ logModule validationsModule ];
  runtimeDeps = with pkgs; [ coreutils gnused ];
  inherit functions;
  exports = [
    "get_pid_from_file"
    "is_service_running"
    "stop_service_using_pid"
    "cron_start"
    "generate_cron_conf"
    "generate_logrotate_conf"
  ];

  script = pkgs.writeTextDir "opt/firestream/scripts/libservice.sh" ''
    #!/bin/bash
    # Copyright Firestream. MIT License.
    # Generated by Nix - do not edit directly.
    ${functions}
  '';
}
