# Firestream Module System Tests

Comprehensive unit and integration tests for the Firestream Nix shell module system.

## Test Structure

The test suite is organized into individual test modules, each testing a specific library:

- **test-log.nix**: Tests logging functions (info, warn, error, debug, indent)
- **test-validations.nix**: Tests validation functions (is_boolean_yes, is_int, validate_port, validate_ipv4/6)
- **test-fs.nix**: Tests filesystem operations (ensure_dir_exists, configure_permissions_ownership, is_dir_empty)
- **test-os.nix**: Tests OS utilities (am_i_root, get_os_metadata, retry_while, wait_for_log_entry)
- **test-net.nix**: Tests network functions (parse_uri, validate_ip, wait_for_host, resolve_hostname_ip)
- **test-service.nix**: Tests service management (is_service_running, generate_start_command, wait_for_service)
- **test-file.nix**: Tests file operations (replace_in_file, yml_key_set, ini_file_set, json_set, xml_set)
- **test-persistence.nix**: Tests persistence layer (persist_app, persist_dir, restore_persisted_dir, backup_persisted_data)
- **test-integration.nix**: End-to-end integration test creating a complete application setup

## Running Tests

### Run All Tests

```bash
nix-build ./bin/nix/firestream/tests -A all
```

This will run all test suites and report success/failure.

### Run Individual Test Suites

```bash
# Test logging functions
nix-build ./bin/nix/firestream/tests -A logTests

# Test validations
nix-build ./bin/nix/firestream/tests -A validationsTests

# Test filesystem operations
nix-build ./bin/nix/firestream/tests -A fsTests

# Test OS utilities
nix-build ./bin/nix/firestream/tests -A osTests

# Test network functions
nix-build ./bin/nix/firestream/tests -A netTests

# Test service management
nix-build ./bin/nix/firestream/tests -A serviceTests

# Test file operations
nix-build ./bin/nix/firestream/tests -A fileTests

# Test persistence layer
nix-build ./bin/nix/firestream/tests -A persistenceTests

# Test integration scenarios
nix-build ./bin/nix/firestream/tests -A integrationTests
```

### Quick Test (from project root)

```bash
# Add to your shell or CI/CD
make test-nix-modules  # If you add this to your Makefile
```

## Test Coverage

### Log Module Tests (10 tests)
- info outputs correctly with INFO prefix
- warn outputs correctly with WARN prefix
- error outputs correctly with ERROR prefix
- debug respects FIRESTREAM_DEBUG=false
- debug outputs when FIRESTREAM_DEBUG=true
- debug works with different FIRESTREAM_DEBUG values
- indent function with custom spacing
- indent function with default spacing
- multiline indent
- color functions exist (print_welcome_page, nami_initialize)

### Validations Module Tests (15+ tests)
- is_boolean_yes: yes, YES, true, TRUE, 1
- is_boolean_yes negatives: no, false, 0, empty
- is_int: positive, negative, zero
- is_int negatives: abc, 3.14, empty
- is_positive_int: valid and invalid cases
- is_empty_value: empty and non-empty
- validate_ipv4: valid IPs (192.168.1.1, 0.0.0.0, 255.255.255.255)
- validate_ipv4 negatives: 256.1.1.1, incomplete, invalid format
- validate_ipv6: ::1, 2001:db8::1
- validate_port: valid ports (1-65535)
- validate_port negatives: 0, 99999
- validate_port with -unprivileged flag

### FS Module Tests (8 tests)
- ensure_dir_exists creates directory
- ensure_dir_exists works on existing directory
- ensure_dir_exists creates nested paths
- configure_permissions_ownership for directories
- configure_permissions_ownership for files
- is_mounted_dir negative case
- is_dir_empty positive and negative cases
- is_file_writable positive and negative cases

### OS Module Tests (8+ tests)
- am_i_root in Nix build environment
- get_os_metadata --os and --dist
- get_total_memory returns valid number
- retry_while success case
- retry_while retry case
- wait_for_log_entry success
- wait_for_log_entry timeout
- get_machine_ip returns value
- get_total_cpus returns positive number

### Net Module Tests (10+ tests)
- parse_uri with explicit port
- parse_uri with default HTTPS port (443)
- parse_uri with default HTTP port (80)
- parse_uri with query string
- resolve_hostname_ip
- validate_ip IPv4 version
- validate_ip IPv6 version
- validate_ip any version
- wait_for_host success
- wait_for_host timeout
- get_port_from_url explicit and default ports

### Service Module Tests (8 tests)
- is_service_running negative case
- generate_start_command includes exec and args
- generate_stop_command returns value
- generate_reload_command returns value
- is_service_enabled negative case
- wait_for_service success
- wait_for_service timeout
- restart_service_if_needed with and without flag

### File Module Tests (10 tests)
- replace_in_file basic replacement
- replace_in_file with special characters
- replace_in_file with regex
- yml_key_set updates YAML values
- ini_file_set updates INI values
- xml_set updates XML values
- json_set updates JSON values
- append_file_if_not_exists appends
- append_file_if_not_exists avoids duplicates
- remove_in_file removes lines

### Persistence Module Tests (12 tests)
- is_app_initialized negative case
- persist_app creates structure
- is_app_initialized positive case
- persist_dir creates symlink and copies data
- restore_persisted_dir restores from persistence
- persist_file creates symlink for file
- restore_persisted_file restores file
- migrate_old_data copies old data
- list_persisted_files lists all persisted items
- is_dir_persisted positive and negative cases
- backup_persisted_data creates backup

### Integration Tests (15 scenarios)
1. Create application directory structure
2. Create and configure config files
3. Validate configuration values
4. Setup persistence layer
5. Create service simulation
6. Network configuration parsing
7. File operations
8. Logging integration
9. Permission management
10. Backup and restore
11. Environment validation
12. System information gathering
13. Cleanup operations
14. Multi-module interaction
15. Final verification

## Test Design Principles

1. **Deterministic**: No network calls, no random behavior
2. **Fast**: All tests run in seconds
3. **Isolated**: Each test runs in isolated TMPDIR
4. **Comprehensive**: Cover success AND failure cases
5. **Self-contained**: Tests use generated shell functions from modules

## Understanding Test Failures

When a test fails, you'll see output like:

```
FAIL: <test description>
```

The test will exit with code 1 and Nix will report the build failure.

### Common Issues

- **Permission errors**: Check if test is trying to write outside TMPDIR
- **Missing dependencies**: Ensure all required tools are in pkgs
- **Sandbox restrictions**: Some operations (like DNS) may not work in Nix sandbox

## Adding New Tests

To add tests for a new module:

1. Create `test-<module>.nix` in this directory
2. Follow the pattern of existing tests
3. Import and export in `default.nix`
4. Run to verify

Example structure:

```nix
{ pkgs, firestream }:

pkgs.runCommand "test-mymodule" {} ''
  export HOME=$TMPDIR

  cat > $TMPDIR/test.sh << 'SCRIPT'
  ${firestream.lib.mymodule.functions}

  # Test 1: Description
  my_function "arg" || { echo "FAIL: test description"; exit 1; }

  echo "All mymodule tests passed!"
  SCRIPT

  ${pkgs.bash}/bin/bash $TMPDIR/test.sh
  touch $out
''
```

## Continuous Integration

These tests are designed to run in CI/CD environments:

```yaml
# .github/workflows/test.yml
- name: Test Nix Modules
  run: nix-build ./bin/nix/firestream/tests -A all
```

## Test Results

Successful test run creates a derivation in `/nix/store` with a marker file.
Failed tests exit with error code and descriptive message.

## Performance

Expected test execution times (on modern hardware):

- Individual test suite: < 5 seconds
- All tests combined: < 30 seconds
- Integration test: < 10 seconds

## Debugging Tests

To see verbose output:

```bash
# Run test with nix-build and check output
nix-build ./bin/nix/firestream/tests -A logTests 2>&1 | less

# Or run the generated script directly for debugging
nix-shell -p bash --run "bash /path/to/test.sh"
```

## Future Enhancements

Potential improvements to the test suite:

- [ ] Add property-based testing with QuickCheck-style generators
- [ ] Add performance benchmarks
- [ ] Add code coverage reporting
- [ ] Add mutation testing
- [ ] Add stress tests for concurrent operations
- [ ] Add tests for error recovery scenarios
