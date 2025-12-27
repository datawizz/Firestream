{ pkgs, firestream }:

pkgs.runCommand "test-os" {} ''
  export HOME=$TMPDIR

  cat > $TMPDIR/test.sh << 'SCRIPT'
  ${firestream.lib.os.functions}

  # Test am_i_root (in Nix build, we're not root)
  ! am_i_root || { echo "FAIL: am_i_root should return false in Nix build"; exit 1; }

  # Test get_os_metadata
  os=$(get_os_metadata --os)
  [[ -n "$os" ]] || { echo "FAIL: get_os_metadata --os should return value"; exit 1; }

  dist=$(get_os_metadata --dist)
  [[ -n "$dist" ]] || { echo "FAIL: get_os_metadata --dist should return value"; exit 1; }

  # Test get_total_memory
  mem=$(get_total_memory)
  [[ "$mem" =~ ^[0-9]+$ ]] || { echo "FAIL: get_total_memory should return number (got $mem)"; exit 1; }

  # Test retry_while (success case)
  counter=0
  retry_while "[ \$counter -lt 3 ]" && counter=$((counter+1))
  [[ "$counter" -eq 1 ]] || { echo "FAIL: retry_while should execute once when condition succeeds"; exit 1; }

  # Test retry_while (retry case)
  counter=0
  retry_while "counter=\$((counter+1)); [ \$counter -lt 3 ]" --tries 5
  [[ "$counter" -ge 2 ]] || { echo "FAIL: retry_while should retry (counter=$counter)"; exit 1; }

  # Test dns_lookup
  # Note: In Nix sandbox, DNS might not work, so we'll test function exists
  declare -F dns_lookup >/dev/null || { echo "FAIL: dns_lookup not defined"; exit 1; }

  # Test wait_for_log_entry
  test_log="$TMPDIR/test.log"
  echo "Starting..." > "$test_log"
  (sleep 0.2 && echo "SUCCESS_MARKER" >> "$test_log") &
  wait_for_log_entry "SUCCESS_MARKER" "$test_log" 2 || { echo "FAIL: wait_for_log_entry should find marker"; exit 1; }

  # Test wait_for_log_entry (timeout case)
  test_log2="$TMPDIR/test2.log"
  echo "Starting..." > "$test_log2"
  ! wait_for_log_entry "NEVER_APPEARS" "$test_log2" 1 2>/dev/null || { echo "FAIL: wait_for_log_entry should timeout"; exit 1; }

  # Test get_machine_ip
  ip=$(get_machine_ip)
  [[ -n "$ip" ]] || { echo "FAIL: get_machine_ip should return value"; exit 1; }

  # Test get_total_cpus
  cpus=$(get_total_cpus)
  [[ "$cpus" =~ ^[0-9]+$ ]] || { echo "FAIL: get_total_cpus should return number (got $cpus)"; exit 1; }
  [[ "$cpus" -gt 0 ]] || { echo "FAIL: get_total_cpus should return positive number"; exit 1; }

  echo "All os tests passed!"
  SCRIPT

  ${pkgs.bash}/bin/bash $TMPDIR/test.sh
  touch $out
''
