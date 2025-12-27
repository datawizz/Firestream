{ pkgs, firestream }:

pkgs.runCommand "test-fs" {} ''
  export HOME=$TMPDIR

  cat > $TMPDIR/test.sh << 'SCRIPT'
  ${firestream.lib.fs.functions}

  # Test ensure_dir_exists
  test_dir="$TMPDIR/test_dir"
  ensure_dir_exists "$test_dir"
  [[ -d "$test_dir" ]] || { echo "FAIL: ensure_dir_exists should create directory"; exit 1; }

  # Test ensure_dir_exists with existing directory
  ensure_dir_exists "$test_dir"
  [[ -d "$test_dir" ]] || { echo "FAIL: ensure_dir_exists should work on existing dir"; exit 1; }

  # Test ensure_dir_exists with nested path
  nested_dir="$TMPDIR/nested/path/to/dir"
  ensure_dir_exists "$nested_dir"
  [[ -d "$nested_dir" ]] || { echo "FAIL: ensure_dir_exists should create nested dirs"; exit 1; }

  # Test configure_permissions_ownership (with -d for directory)
  test_perms_dir="$TMPDIR/perms_test"
  mkdir -p "$test_perms_dir"
  configure_permissions_ownership "$test_perms_dir" -d "755" -f "644"
  perms=$(stat -c "%a" "$test_perms_dir" 2>/dev/null || stat -f "%Lp" "$test_perms_dir")
  [[ "$perms" == "755" ]] || { echo "FAIL: configure_permissions_ownership directory perms (got $perms)"; exit 1; }

  # Test configure_permissions_ownership on files
  test_file="$test_perms_dir/testfile"
  touch "$test_file"
  configure_permissions_ownership "$test_perms_dir" -d "755" -f "644"
  file_perms=$(stat -c "%a" "$test_file" 2>/dev/null || stat -f "%Lp" "$test_file")
  [[ "$file_perms" == "644" ]] || { echo "FAIL: configure_permissions_ownership file perms (got $file_perms)"; exit 1; }

  # Test is_mounted_dir (negative case - nothing is mounted in our test)
  ! is_mounted_dir "$test_dir" || { echo "FAIL: is_mounted_dir should return false for non-mounted dir"; exit 1; }

  # Test is_dir_empty
  empty_dir="$TMPDIR/empty"
  mkdir -p "$empty_dir"
  is_dir_empty "$empty_dir" || { echo "FAIL: is_dir_empty should return true for empty dir"; exit 1; }

  # Test is_dir_empty with file
  touch "$empty_dir/file"
  ! is_dir_empty "$empty_dir" || { echo "FAIL: is_dir_empty should return false for non-empty dir"; exit 1; }

  # Test is_file_writable
  writable_file="$TMPDIR/writable"
  touch "$writable_file"
  chmod +w "$writable_file"
  is_file_writable "$writable_file" || { echo "FAIL: is_file_writable should return true for writable file"; exit 1; }

  # Test is_file_writable (negative case)
  readonly_file="$TMPDIR/readonly"
  touch "$readonly_file"
  chmod -w "$readonly_file"
  ! is_file_writable "$readonly_file" || { echo "FAIL: is_file_writable should return false for readonly file"; exit 1; }

  echo "All fs tests passed!"
  SCRIPT

  ${pkgs.bash}/bin/bash $TMPDIR/test.sh
  touch $out
''
