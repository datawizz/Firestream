{ pkgs, firestream }:

pkgs.runCommand "test-log" {} ''
  export HOME=$TMPDIR

  # Generate script with log functions
  cat > $TMPDIR/test.sh << 'SCRIPT'
  ${firestream.lib.log.functions}

  # Test 1: info outputs correctly
  output=$(info "test message" 2>&1)
  [[ "$output" == *"INFO"* ]] || { echo "FAIL: info should contain INFO"; exit 1; }
  [[ "$output" == *"test message"* ]] || { echo "FAIL: info should contain message"; exit 1; }

  # Test 2: warn outputs correctly
  output=$(warn "warning" 2>&1)
  [[ "$output" == *"WARN"* ]] || { echo "FAIL: warn should contain WARN"; exit 1; }

  # Test 3: error outputs correctly
  output=$(error "error" 2>&1)
  [[ "$output" == *"ERROR"* ]] || { echo "FAIL: error should contain ERROR"; exit 1; }

  # Test 4: debug respects FIRESTREAM_DEBUG
  export FIRESTREAM_DEBUG=false
  output=$(debug "debug msg" 2>&1)
  [[ -z "$output" ]] || { echo "FAIL: debug should be silent when FIRESTREAM_DEBUG=false"; exit 1; }

  # Test 5: debug outputs when enabled
  export FIRESTREAM_DEBUG=true
  output=$(debug "debug msg" 2>&1)
  [[ "$output" == *"DEBUG"* ]] || { echo "FAIL: debug should output when FIRESTREAM_DEBUG=true"; exit 1; }

  # Test 6: verify FIRESTREAM_DEBUG works with different values
  unset FIRESTREAM_DEBUG
  export FIRESTREAM_DEBUG=true
  output=$(debug "debug msg" 2>&1)
  [[ "$output" == *"DEBUG"* ]] || { echo "FAIL: debug should work with FIRESTREAM_DEBUG=true"; exit 1; }

  # Test 7: indent function works
  result=$(indent "test" 4)
  [[ "$result" == "    test" ]] || { echo "FAIL: indent should add 4 spaces"; exit 1; }

  # Test 8: indent with 2 spaces
  result=$(indent "test" 2)
  [[ "$result" == "  test" ]] || { echo "FAIL: indent should add 2 spaces"; exit 1; }

  # Test 9: indent with custom character
  result=$(indent "test" 3 "-")
  [[ "$result" == "---test" ]] || { echo "FAIL: indent should work with custom character"; exit 1; }

  # Test 10: all core functions exist
  declare -F stderr_print >/dev/null || { echo "FAIL: stderr_print not defined"; exit 1; }
  declare -F log >/dev/null || { echo "FAIL: log not defined"; exit 1; }

  echo "All log tests passed!"
  SCRIPT

  ${pkgs.bash}/bin/bash $TMPDIR/test.sh
  touch $out
''
