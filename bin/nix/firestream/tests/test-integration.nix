{ pkgs, firestream }:

pkgs.runCommand "test-integration" {} ''
  export HOME=$TMPDIR

  cat > $TMPDIR/test.sh << 'SCRIPT'
  # This is an end-to-end integration test that creates a sample application
  # using the Firestream module system

  # Import all modules
  ${firestream.lib.log.functions}
  ${firestream.lib.validations.functions}
  ${firestream.lib.fs.functions}
  ${firestream.lib.os.functions}
  ${firestream.lib.net.functions}
  ${firestream.lib.service.functions}
  ${firestream.lib.file.functions}
  ${firestream.lib.persistence.functions}

  info "Starting integration test..."

  # Test 1: Create application directory structure
  export APP_NAME="myapp"
  export APP_VERSION="1.0.0"
  export APP_HOME="$TMPDIR/myapp"
  export APP_CONF_DIR="$APP_HOME/conf"
  export APP_DATA_DIR="$APP_HOME/data"
  export APP_LOG_DIR="$APP_HOME/logs"

  info "Creating application directories..."
  ensure_dir_exists "$APP_HOME"
  ensure_dir_exists "$APP_CONF_DIR"
  ensure_dir_exists "$APP_DATA_DIR"
  ensure_dir_exists "$APP_LOG_DIR"

  [[ -d "$APP_HOME" ]] || { error "Failed to create APP_HOME"; exit 1; }
  [[ -d "$APP_CONF_DIR" ]] || { error "Failed to create APP_CONF_DIR"; exit 1; }

  # Test 2: Create and configure config files
  info "Creating configuration files..."
  config_file="$APP_CONF_DIR/app.conf"
  cat > "$config_file" << 'EOF'
  [server]
  port=8080
  host=localhost

  [database]
  url=postgresql://localhost/mydb
  EOF

  # Validate and update configuration
  ini_file_set "$config_file" "server" "port" "9090"
  result=$(grep -A1 "\[server\]" "$config_file" | grep "port")
  [[ "$result" == *"9090"* ]] || { error "Config update failed"; exit 1; }

  # Test 3: Validate configuration values
  info "Validating configuration..."
  validate_port 9090 || { error "Port validation failed"; exit 1; }

  # Test 4: Setup persistence
  info "Setting up persistence..."
  export PERSISTENCE_ROOT="$TMPDIR/persistence"
  persist_app || { error "Failed to persist app"; exit 1; }
  persist_dir "$APP_DATA_DIR" "data" || { error "Failed to persist data dir"; exit 1; }
  persist_file "$config_file" "config" || { error "Failed to persist config"; exit 1; }

  is_app_initialized || { error "App should be initialized"; exit 1; }

  # Test 5: Create service simulation
  info "Setting up service..."
  export SERVICE_NAME="$APP_NAME"
  export SERVICE_EXEC="$APP_HOME/bin/myapp"
  export SERVICE_ARGS="--config $config_file"
  export SERVICE_PID_FILE="$APP_HOME/myapp.pid"

  # Create dummy executable
  mkdir -p "$APP_HOME/bin"
  cat > "$SERVICE_EXEC" << 'EXEC'
  #!/usr/bin/env bash
  echo $$ > $SERVICE_PID_FILE
  while true; do sleep 1; done
  EXEC
  chmod +x "$SERVICE_EXEC"

  # Generate start command
  start_cmd=$(generate_start_command)
  [[ "$start_cmd" == *"$SERVICE_EXEC"* ]] || { error "Start command generation failed"; exit 1; }

  # Test 6: Network configuration
  info "Testing network utilities..."
  parse_uri "http://localhost:9090/api/v1" uri
  [[ "''${uri[scheme]}" == "http" ]] || { error "URI parsing failed"; exit 1; }
  [[ "''${uri[port]}" == "9090" ]] || { error "Port parsing failed"; exit 1; }

  # Test 7: File operations
  info "Testing file operations..."
  data_file="$APP_DATA_DIR/data.txt"
  echo "initial data" > "$data_file"
  replace_in_file "$data_file" "initial" "updated"
  [[ "$(cat $data_file)" == "updated data" ]] || { error "File replacement failed"; exit 1; }

  # Test 8: Logging integration
  info "Testing logging system..."
  log_file="$APP_LOG_DIR/app.log"
  echo "Application started" > "$log_file"
  echo "Initialization complete" >> "$log_file"

  # Wait for log entry
  wait_for_log_entry "Initialization complete" "$log_file" 2 || { error "Log wait failed"; exit 1; }

  # Test 9: Permission management
  info "Setting permissions..."
  configure_permissions_ownership "$APP_HOME" -d "755" -f "644"

  # Verify directory permissions
  perms=$(stat -c "%a" "$APP_HOME" 2>/dev/null || stat -f "%Lp" "$APP_HOME")
  [[ "$perms" == "755" ]] || { warn "Unexpected directory permissions: $perms"; }

  # Test 10: Backup and restore
  info "Testing backup and restore..."
  backup_dir="$TMPDIR/backup"
  backup_persisted_data "$backup_dir" || { error "Backup failed"; exit 1; }
  [[ -d "$backup_dir" ]] || { error "Backup directory not created"; exit 1; }

  # Test 11: Environment validation
  info "Validating environment..."
  export TEST_ENABLED="yes"
  is_boolean_yes "$TEST_ENABLED" || { error "Boolean validation failed"; exit 1; }

  export TEST_PORT="9090"
  is_positive_int "$TEST_PORT" || { error "Integer validation failed"; exit 1; }

  # Test 12: System information
  info "Gathering system information..."
  total_cpus=$(get_total_cpus)
  [[ "$total_cpus" -gt 0 ]] || { error "Failed to get CPU count"; exit 1; }
  info "System has $total_cpus CPUs"

  total_mem=$(get_total_memory)
  [[ "$total_mem" -gt 0 ]] || { error "Failed to get memory"; exit 1; }
  info "System has $total_mem MB memory"

  # Test 13: Cleanup test
  info "Testing cleanup operations..."
  test_temp_dir="$TMPDIR/temp_test"
  mkdir -p "$test_temp_dir"
  echo "temp" > "$test_temp_dir/file.txt"

  # Verify directory is not empty
  ! is_dir_empty "$test_temp_dir" || { error "Directory should not be empty"; exit 1; }

  # Remove content
  rm -rf "$test_temp_dir"/*

  # Verify directory is empty
  is_dir_empty "$test_temp_dir" || { error "Directory should be empty"; exit 1; }

  # Test 14: Multi-module interaction
  info "Testing multi-module interaction..."

  # Create a scenario combining log, file, and validation modules
  scenario_file="$APP_CONF_DIR/scenario.yml"
  cat > "$scenario_file" << 'YML'
  application:
    name: myapp
    port: 8080
    debug: false
  YML

  # Read and validate
  port_line=$(grep "port:" "$scenario_file")
  port_value=$(echo "$port_line" | sed 's/.*: //')
  validate_port "$port_value" || { error "Scenario port validation failed"; exit 1; }

  # Update configuration
  yml_key_set "$scenario_file" "application.debug" "true"
  debug_line=$(grep "debug:" "$scenario_file")
  [[ "$debug_line" == *"true"* ]] || { error "Scenario debug update failed"; exit 1; }

  # Log the change
  info "Updated debug setting in $scenario_file"

  # Test 15: Final verification
  info "Running final verification..."

  # Verify all created artifacts exist
  [[ -f "$config_file" ]] || { error "Config file missing"; exit 1; }
  [[ -f "$data_file" ]] || { error "Data file missing"; exit 1; }
  [[ -f "$log_file" ]] || { error "Log file missing"; exit 1; }
  [[ -f "$scenario_file" ]] || { error "Scenario file missing"; exit 1; }
  [[ -d "$backup_dir" ]] || { error "Backup directory missing"; exit 1; }

  # Verify persistence
  is_app_initialized || { error "App not initialized"; exit 1; }
  is_dir_persisted "$APP_DATA_DIR" || { error "Data dir not persisted"; exit 1; }

  info "================================================"
  info "Integration test completed successfully!"
  info "================================================"
  info "Application:    $APP_NAME v$APP_VERSION"
  info "Home:           $APP_HOME"
  info "Config:         $config_file"
  info "Data:           $APP_DATA_DIR"
  info "Logs:           $APP_LOG_DIR"
  info "Persistence:    $PERSISTENCE_ROOT"
  info "Backup:         $backup_dir"
  info "================================================"

  echo "All integration tests passed!"
  SCRIPT

  ${pkgs.bash}/bin/bash $TMPDIR/test.sh
  touch $out
''
