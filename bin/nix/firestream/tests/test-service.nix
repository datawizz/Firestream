{ pkgs, firestream }:

pkgs.runCommand "test-service" {} ''
  export HOME=$TMPDIR

  cat > $TMPDIR/test.sh << 'SCRIPT'
  ${firestream.lib.service.functions}

  # Test is_service_running (negative case - no systemd in Nix build)
  ! is_service_running "fake-service" 2>/dev/null || { echo "FAIL: is_service_running should return false"; exit 1; }

  # Test generate_start_command
  export SERVICE_NAME="test-service"
  export SERVICE_EXEC="/usr/bin/test"
  export SERVICE_ARGS="--arg1 --arg2"

  cmd=$(generate_start_command)
  [[ "$cmd" == *"/usr/bin/test"* ]] || { echo "FAIL: generate_start_command should include exec"; exit 1; }
  [[ "$cmd" == *"--arg1"* ]] || { echo "FAIL: generate_start_command should include args"; exit 1; }

  # Test generate_stop_command
  stop_cmd=$(generate_stop_command)
  [[ -n "$stop_cmd" ]] || { echo "FAIL: generate_stop_command should return value"; exit 1; }

  # Test generate_reload_command
  reload_cmd=$(generate_reload_command)
  [[ -n "$reload_cmd" ]] || { echo "FAIL: generate_reload_command should return value"; exit 1; }

  # Test is_service_enabled (negative case)
  ! is_service_enabled "fake-service" 2>/dev/null || { echo "FAIL: is_service_enabled should return false"; exit 1; }

  # Test wait_for_service
  # Create a dummy PID file to simulate service start
  export SERVICE_PID_FILE="$TMPDIR/service.pid"
  (sleep 0.2 && echo "12345" > "$SERVICE_PID_FILE") &
  wait_for_service 5 || { echo "FAIL: wait_for_service should succeed"; exit 1; }

  # Test wait_for_service (timeout case)
  rm -f "$SERVICE_PID_FILE"
  ! wait_for_service 1 2>/dev/null || { echo "FAIL: wait_for_service should timeout"; exit 1; }

  # Test restart_service_if_needed
  export SERVICE_RESTART_FLAG="$TMPDIR/restart_flag"
  touch "$SERVICE_RESTART_FLAG"

  # Mock restart command
  export SERVICE_RESTART_COMMAND="echo 'Restarted' && rm -f $SERVICE_RESTART_FLAG"
  restart_service_if_needed || { echo "FAIL: restart_service_if_needed should succeed"; exit 1; }
  [[ ! -f "$SERVICE_RESTART_FLAG" ]] || { echo "FAIL: restart flag should be removed"; exit 1; }

  # Test restart_service_if_needed when no restart needed
  restart_service_if_needed || { echo "FAIL: restart_service_if_needed should succeed when no restart needed"; exit 1; }

  echo "All service tests passed!"
  SCRIPT

  ${pkgs.bash}/bin/bash $TMPDIR/test.sh
  touch $out
''
