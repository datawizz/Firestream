{ pkgs, firestream }:

pkgs.runCommand "test-persistence" {} ''
  export HOME=$TMPDIR

  cat > $TMPDIR/test.sh << 'SCRIPT'
  ${firestream.lib.persistence.functions}

  # Setup test environment
  export PERSISTENCE_ROOT="$TMPDIR/persistence"
  export APP_NAME="testapp"
  export APP_VERSION="1.0"

  # Test is_app_initialized (negative case)
  ! is_app_initialized || { echo "FAIL: is_app_initialized should return false initially"; exit 1; }

  # Test persist_app
  persist_app || { echo "FAIL: persist_app should succeed"; exit 1; }
  [[ -d "$PERSISTENCE_ROOT/$APP_NAME" ]] || { echo "FAIL: persist_app should create app directory"; exit 1; }

  # Test is_app_initialized (positive case)
  is_app_initialized || { echo "FAIL: is_app_initialized should return true after persist_app"; exit 1; }

  # Test persist_dir
  test_data_dir="$TMPDIR/data"
  mkdir -p "$test_data_dir"
  echo "test data" > "$test_data_dir/file.txt"

  persist_dir "$test_data_dir" "data" || { echo "FAIL: persist_dir should succeed"; exit 1; }
  [[ -L "$test_data_dir" ]] || { echo "FAIL: persist_dir should create symlink"; exit 1; }
  [[ -f "$PERSISTENCE_ROOT/$APP_NAME/data/file.txt" ]] || { echo "FAIL: persist_dir should copy data"; exit 1; }

  # Test restore_persisted_dir
  rm -rf "$test_data_dir"
  restore_persisted_dir "data" "$test_data_dir" || { echo "FAIL: restore_persisted_dir should succeed"; exit 1; }
  [[ -L "$test_data_dir" ]] || { echo "FAIL: restore_persisted_dir should create symlink"; exit 1; }
  [[ -f "$test_data_dir/file.txt" ]] || { echo "FAIL: restore_persisted_dir should restore data"; exit 1; }

  # Test persist_file
  test_config="$TMPDIR/config.conf"
  echo "setting=value" > "$test_config"

  persist_file "$test_config" "config" || { echo "FAIL: persist_file should succeed"; exit 1; }
  [[ -L "$test_config" ]] || { echo "FAIL: persist_file should create symlink"; exit 1; }
  [[ -f "$PERSISTENCE_ROOT/$APP_NAME/config/config.conf" ]] || { echo "FAIL: persist_file should copy file"; exit 1; }

  # Test restore_persisted_file
  rm -f "$test_config"
  restore_persisted_file "config/config.conf" "$test_config" || { echo "FAIL: restore_persisted_file should succeed"; exit 1; }
  [[ -L "$test_config" ]] || { echo "FAIL: restore_persisted_file should create symlink"; exit 1; }
  content=$(cat "$test_config")
  [[ "$content" == "setting=value" ]] || { echo "FAIL: restore_persisted_file content (got: $content)"; exit 1; }

  # Test migrate_old_data
  old_dir="$TMPDIR/old_data"
  mkdir -p "$old_dir"
  echo "old" > "$old_dir/old.txt"

  migrate_old_data "$old_dir" "migrated" || { echo "FAIL: migrate_old_data should succeed"; exit 1; }
  [[ -f "$PERSISTENCE_ROOT/$APP_NAME/migrated/old.txt" ]] || { echo "FAIL: migrate_old_data should copy old data"; exit 1; }

  # Test list_persisted_files
  files=$(list_persisted_files)
  [[ "$files" == *"data"* ]] || { echo "FAIL: list_persisted_files should include 'data'"; exit 1; }
  [[ "$files" == *"config"* ]] || { echo "FAIL: list_persisted_files should include 'config'"; exit 1; }

  # Test is_dir_persisted
  is_dir_persisted "$test_data_dir" || { echo "FAIL: is_dir_persisted should return true for persisted dir"; exit 1; }
  ! is_dir_persisted "$TMPDIR/not_persisted" || { echo "FAIL: is_dir_persisted should return false for non-persisted"; exit 1; }

  # Test backup_persisted_data
  backup_dir="$TMPDIR/backup"
  backup_persisted_data "$backup_dir" || { echo "FAIL: backup_persisted_data should succeed"; exit 1; }
  [[ -d "$backup_dir" ]] || { echo "FAIL: backup_persisted_data should create backup dir"; exit 1; }
  [[ -f "$backup_dir/$APP_NAME/data/file.txt" ]] || { echo "FAIL: backup_persisted_data should backup files"; exit 1; }

  echo "All persistence tests passed!"
  SCRIPT

  ${pkgs.bash}/bin/bash $TMPDIR/test.sh
  touch $out
''
