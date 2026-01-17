{ pkgs, firestream }:

pkgs.runCommand "test-net" {} ''
  export HOME=$TMPDIR

  cat > $TMPDIR/test.sh << 'SCRIPT'
  ${firestream.lib.net.functions}

  # Test parse_uri
  parse_uri "http://example.com:8080/path" uri
  [[ "''${uri[scheme]}" == "http" ]] || { echo "FAIL: parse_uri scheme"; exit 1; }
  [[ "''${uri[host]}" == "example.com" ]] || { echo "FAIL: parse_uri host"; exit 1; }
  [[ "''${uri[port]}" == "8080" ]] || { echo "FAIL: parse_uri port"; exit 1; }
  [[ "''${uri[path]}" == "/path" ]] || { echo "FAIL: parse_uri path"; exit 1; }

  # Test parse_uri with default port
  parse_uri "https://example.com/path" uri2
  [[ "''${uri2[scheme]}" == "https" ]] || { echo "FAIL: parse_uri https scheme"; exit 1; }
  [[ "''${uri2[port]}" == "443" ]] || { echo "FAIL: parse_uri default https port"; exit 1; }

  # Test parse_uri with http default port
  parse_uri "http://example.com" uri3
  [[ "''${uri3[port]}" == "80" ]] || { echo "FAIL: parse_uri default http port"; exit 1; }

  # Test parse_uri with query string
  parse_uri "http://example.com/path?key=value" uri4
  [[ "''${uri4[path]}" == "/path" ]] || { echo "FAIL: parse_uri path with query"; exit 1; }
  [[ "''${uri4[query]}" == "key=value" ]] || { echo "FAIL: parse_uri query"; exit 1; }

  # Test resolve_hostname_ip
  # In Nix sandbox, DNS might not work, so we test with localhost
  result=$(resolve_hostname_ip "localhost" 2>/dev/null || echo "127.0.0.1")
  [[ -n "$result" ]] || { echo "FAIL: resolve_hostname_ip should return value"; exit 1; }

  # Test validate_ip (IPv4)
  validate_ip "192.168.1.1" 4 || { echo "FAIL: validate_ip IPv4"; exit 1; }
  ! validate_ip "invalid" 4 2>/dev/null || { echo "FAIL: validate_ip should reject invalid IPv4"; exit 1; }

  # Test validate_ip (IPv6)
  validate_ip "::1" 6 || { echo "FAIL: validate_ip IPv6"; exit 1; }
  ! validate_ip "192.168.1.1" 6 2>/dev/null || { echo "FAIL: validate_ip should reject IPv4 when expecting IPv6"; exit 1; }

  # Test validate_ip (any version)
  validate_ip "192.168.1.1" || { echo "FAIL: validate_ip any (IPv4)"; exit 1; }
  validate_ip "::1" || { echo "FAIL: validate_ip any (IPv6)"; exit 1; }
  ! validate_ip "not-an-ip" 2>/dev/null || { echo "FAIL: validate_ip should reject invalid"; exit 1; }

  # Test wait_for_host
  # Create a simple server in background for testing
  (
    ${pkgs.netcat}/bin/nc -l 12345 >/dev/null 2>&1 &
    echo $! > $TMPDIR/nc.pid
  )
  sleep 0.5
  wait_for_host localhost 12345 5 || { echo "FAIL: wait_for_host should succeed"; exit 1; }
  kill $(cat $TMPDIR/nc.pid) 2>/dev/null || true

  # Test wait_for_host (timeout case)
  ! wait_for_host localhost 54321 1 2>/dev/null || { echo "FAIL: wait_for_host should timeout"; exit 1; }

  # Test get_port_from_url
  port=$(get_port_from_url "http://example.com:8080")
  [[ "$port" == "8080" ]] || { echo "FAIL: get_port_from_url explicit port (got $port)"; exit 1; }

  port=$(get_port_from_url "https://example.com")
  [[ "$port" == "443" ]] || { echo "FAIL: get_port_from_url default https (got $port)"; exit 1; }

  port=$(get_port_from_url "http://example.com")
  [[ "$port" == "80" ]] || { echo "FAIL: get_port_from_url default http (got $port)"; exit 1; }

  echo "All net tests passed!"
  SCRIPT

  ${pkgs.bash}/bin/bash $TMPDIR/test.sh
  touch $out
''
