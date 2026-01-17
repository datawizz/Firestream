{ pkgs, firestream }:

pkgs.runCommand "test-validations" {} ''
  export HOME=$TMPDIR

  cat > $TMPDIR/test.sh << 'SCRIPT'
  ${firestream.lib.validations.functions}

  # Test is_boolean_yes
  is_boolean_yes "yes" || { echo "FAIL: is_boolean_yes yes"; exit 1; }
  is_boolean_yes "YES" || { echo "FAIL: is_boolean_yes YES"; exit 1; }
  is_boolean_yes "true" || { echo "FAIL: is_boolean_yes true"; exit 1; }
  is_boolean_yes "TRUE" || { echo "FAIL: is_boolean_yes TRUE"; exit 1; }
  is_boolean_yes "1" || { echo "FAIL: is_boolean_yes 1"; exit 1; }
  ! is_boolean_yes "no" || { echo "FAIL: is_boolean_yes no should fail"; exit 1; }
  ! is_boolean_yes "false" || { echo "FAIL: is_boolean_yes false should fail"; exit 1; }
  ! is_boolean_yes "0" || { echo "FAIL: is_boolean_yes 0 should fail"; exit 1; }
  ! is_boolean_yes "" || { echo "FAIL: is_boolean_yes empty should fail"; exit 1; }

  # Test is_int
  is_int "42" || { echo "FAIL: is_int 42"; exit 1; }
  is_int "-5" || { echo "FAIL: is_int -5"; exit 1; }
  is_int "0" || { echo "FAIL: is_int 0"; exit 1; }
  ! is_int "abc" 2>/dev/null || { echo "FAIL: is_int abc should fail"; exit 1; }
  ! is_int "3.14" 2>/dev/null || { echo "FAIL: is_int 3.14 should fail"; exit 1; }
  ! is_int "" 2>/dev/null || { echo "FAIL: is_int empty should fail"; exit 1; }

  # Test is_positive_int
  is_positive_int "42" || { echo "FAIL: is_positive_int 42"; exit 1; }
  is_positive_int "0" || { echo "FAIL: is_positive_int 0"; exit 1; }
  ! is_positive_int "-5" 2>/dev/null || { echo "FAIL: is_positive_int -5 should fail"; exit 1; }
  ! is_positive_int "abc" 2>/dev/null || { echo "FAIL: is_positive_int abc should fail"; exit 1; }

  # Test is_empty_value
  is_empty_value "" || { echo "FAIL: is_empty_value empty"; exit 1; }
  ! is_empty_value "something" || { echo "FAIL: is_empty_value something should fail"; exit 1; }
  ! is_empty_value " " || { echo "FAIL: is_empty_value space should fail"; exit 1; }

  # Test validate_ipv4
  validate_ipv4 "192.168.1.1" || { echo "FAIL: validate_ipv4 192.168.1.1"; exit 1; }
  validate_ipv4 "0.0.0.0" || { echo "FAIL: validate_ipv4 0.0.0.0"; exit 1; }
  validate_ipv4 "255.255.255.255" || { echo "FAIL: validate_ipv4 255.255.255.255"; exit 1; }
  ! validate_ipv4 "256.1.1.1" 2>/dev/null || { echo "FAIL: validate_ipv4 256.1.1.1 should fail"; exit 1; }
  ! validate_ipv4 "192.168.1" 2>/dev/null || { echo "FAIL: validate_ipv4 192.168.1 should fail"; exit 1; }
  ! validate_ipv4 "not.an.ip.addr" 2>/dev/null || { echo "FAIL: validate_ipv4 not.an.ip.addr should fail"; exit 1; }

  # Test validate_ipv6
  validate_ipv6 "::1" || { echo "FAIL: validate_ipv6 ::1"; exit 1; }
  validate_ipv6 "2001:db8::1" || { echo "FAIL: validate_ipv6 2001:db8::1"; exit 1; }
  ! validate_ipv6 "192.168.1.1" 2>/dev/null || { echo "FAIL: validate_ipv6 should reject IPv4"; exit 1; }

  # Test validate_port
  validate_port 8080 || { echo "FAIL: validate_port 8080"; exit 1; }
  validate_port 80 || { echo "FAIL: validate_port 80"; exit 1; }
  validate_port 1 || { echo "FAIL: validate_port 1"; exit 1; }
  validate_port 65535 || { echo "FAIL: validate_port 65535"; exit 1; }
  ! validate_port 0 2>/dev/null || { echo "FAIL: validate_port 0 should fail"; exit 1; }
  ! validate_port 99999 2>/dev/null || { echo "FAIL: validate_port 99999 should fail"; exit 1; }
  ! validate_port -unprivileged 80 2>/dev/null || { echo "FAIL: validate_port -unprivileged 80 should fail"; exit 1; }
  validate_port -unprivileged 8080 || { echo "FAIL: validate_port -unprivileged 8080"; exit 1; }
  validate_port -unprivileged 1024 || { echo "FAIL: validate_port -unprivileged 1024"; exit 1; }

  echo "All validation tests passed!"
  SCRIPT

  ${pkgs.bash}/bin/bash $TMPDIR/test.sh
  touch $out
''
