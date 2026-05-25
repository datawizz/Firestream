# test-volumes.nix - Test suite for the volumes module
# Copyright Firestream. MIT License.
#
# Run tests with: nix-build test-volumes.nix
# Or evaluate: nix eval -f test-volumes.nix

{ pkgs ? import <nixpkgs> {} }:

let
  lib = pkgs.lib;

  # Import the modules with dependencies
  logModule = import ../lib/log.nix { inherit pkgs lib; };
  fsModule = import ../lib/fs.nix { inherit pkgs lib logModule; };
  volumesModule = import ../lib/volumes.nix { inherit pkgs lib logModule fsModule; };

  # Test helpers
  assertEq = name: actual: expected:
    if actual == expected
    then { inherit name; passed = true; }
    else {
      inherit name;
      passed = false;
      message = "Expected: ${builtins.toJSON expected}, Got: ${builtins.toJSON actual}";
    };

  assertThrows = name: expr:
    let
      result = builtins.tryEval expr;
    in
    if !result.success
    then { inherit name; passed = true; }
    else { inherit name; passed = false; message = "Expected expression to throw, but it succeeded"; };

  assertContains = name: str: substring:
    if lib.hasInfix substring str
    then { inherit name; passed = true; }
    else { inherit name; passed = false; message = "Expected '${str}' to contain '${substring}'"; };

  # Run all tests and collect results
  runTests = tests:
    let
      results = map (test: test) tests;
      passed = lib.filter (r: r.passed) results;
      failed = lib.filter (r: !r.passed) results;
    in {
      total = builtins.length results;
      passedCount = builtins.length passed;
      failedCount = builtins.length failed;
      inherit passed failed;
      allPassed = builtins.length failed == 0;
    };

  # ============================================================================
  # Test Cases
  # ============================================================================

  tests = [
    # --- Schema Constants Tests ---

    (assertEq "validTypes contains expected types"
      volumesModule.validTypes
      [ "logs" "tmp" "data" "cache" "conf" "plugins" "work" "state" "custom" ])

    (assertEq "validPersistence contains expected modes"
      volumesModule.validPersistence
      [ "ephemeral" "persistent" "tmpfs" ])

    (assertEq "defaultDir has correct default type"
      volumesModule.defaultDir.type
      "custom")

    (assertEq "defaultDir has correct default persistence"
      volumesModule.defaultDir.persistence
      "ephemeral")

    (assertEq "defaultDir has correct default mode"
      volumesModule.defaultDir.mode
      "0755")

    # --- normalizeDir Tests ---

    (assertEq "normalizeDir merges with defaults"
      (volumesModule.normalizeDir "test" { path = "/test"; }).type
      "custom")

    (assertEq "normalizeDir preserves explicit type"
      (volumesModule.normalizeDir "test" { path = "/test"; type = "logs"; }).type
      "logs")

    (assertEq "normalizeDir preserves explicit mode"
      (volumesModule.normalizeDir "test" { path = "/test"; mode = "0700"; }).mode
      "0700")

    (assertEq "normalizeDir preserves owner"
      (volumesModule.normalizeDir "test" { path = "/test"; owner = "myuser"; }).owner
      "myuser")

    # --- validateDir Tests ---

    (assertEq "validateDir accepts valid spec"
      (volumesModule.validateDir "test" {
        path = "/opt/test";
        type = "data";
        persistence = "persistent";
        mode = "0755";
      }).path
      "/opt/test")

    (assertEq "validateDir accepts minimal spec"
      (volumesModule.validateDir "test" { path = "/test"; }).persistence
      "ephemeral")

    (assertThrows "validateDir rejects missing path"
      (volumesModule.validateDir "test" { type = "logs"; }))

    (assertThrows "validateDir rejects relative path"
      (volumesModule.validateDir "test" { path = "relative/path"; }))

    (assertThrows "validateDir rejects invalid type"
      (volumesModule.validateDir "test" { path = "/test"; type = "invalid"; }))

    (assertThrows "validateDir rejects invalid persistence"
      (volumesModule.validateDir "test" { path = "/test"; persistence = "invalid"; }))

    (assertThrows "validateDir rejects invalid mode format"
      (volumesModule.validateDir "test" { path = "/test"; mode = "abc"; }))

    (assertThrows "validateDir rejects mode with wrong length"
      (volumesModule.validateDir "test" { path = "/test"; mode = "75"; }))

    # --- fromPrepopulateDirs Tests ---

    (assertEq "fromPrepopulateDirs converts empty list"
      (volumesModule.fromPrepopulateDirs [])
      {})

    (assertEq "fromPrepopulateDirs converts single path"
      (volumesModule.fromPrepopulateDirs [ "/opt/app" ]).legacyDir0.path
      "/opt/app")

    (assertEq "fromPrepopulateDirs sets type to custom"
      (volumesModule.fromPrepopulateDirs [ "/opt/app" ]).legacyDir0.type
      "custom")

    (assertEq "fromPrepopulateDirs sets persistence to ephemeral"
      (volumesModule.fromPrepopulateDirs [ "/opt/app" ]).legacyDir0.persistence
      "ephemeral")

    (assertEq "fromPrepopulateDirs converts multiple paths"
      (builtins.length (builtins.attrNames (volumesModule.fromPrepopulateDirs [ "/a" "/b" "/c" ])))
      3)

    # --- getAllPaths Tests ---

    (assertEq "getAllPaths extracts all paths"
      (lib.sort builtins.lessThan (volumesModule.getAllPaths {
        logs = { path = "/opt/logs"; };
        data = { path = "/opt/data"; };
      }))
      [ "/opt/data" "/opt/logs" ])

    (assertEq "getAllPaths handles empty runtimeDirs"
      (volumesModule.getAllPaths {})
      [])

    # --- extractVolumeHints Tests ---

    (assertEq "extractVolumeHints filters persistent dirs"
      (builtins.length (volumesModule.extractVolumeHints {
        logs = { path = "/opt/logs"; persistence = "persistent"; };
        tmp = { path = "/opt/tmp"; persistence = "ephemeral"; };
      }))
      1)

    (assertEq "extractVolumeHints returns path and type"
      (builtins.head (volumesModule.extractVolumeHints {
        data = { path = "/data"; type = "data"; persistence = "persistent"; };
      })).path
      "/data")

    (assertEq "extractVolumeHints returns empty for all ephemeral"
      (volumesModule.extractVolumeHints {
        tmp = { path = "/tmp"; persistence = "ephemeral"; };
      })
      [])

    # --- extractTmpfsHints Tests ---

    (assertEq "extractTmpfsHints filters tmpfs dirs"
      (builtins.length (volumesModule.extractTmpfsHints {
        cache = { path = "/cache"; persistence = "tmpfs"; };
        data = { path = "/data"; persistence = "persistent"; };
      }))
      1)

    # --- mkDirCreationCode Tests ---

    (let
      code = volumesModule.mkDirCreationCode "logs" {
        path = "/opt/logs";
        type = "logs";
        persistence = "persistent";
        mode = "0755";
        owner = "app";
        group = "app";
        description = "Application logs";
      };
    in assertContains "mkDirCreationCode includes path"
      code
      "/opt/logs")

    (let
      code = volumesModule.mkDirCreationCode "logs" {
        path = "/opt/logs";
        owner = "app";
        group = "app";
      };
    in assertContains "mkDirCreationCode includes owner"
      code
      "\"app\"")

    (let
      code = volumesModule.mkDirCreationCode "logs" {
        path = "/opt/logs";
        mode = "0700";
      };
    in assertContains "mkDirCreationCode includes chmod"
      code
      "chmod 0700")

    (let
      code = volumesModule.mkDirCreationCode "logs" {
        path = "/opt/logs";
        description = "Test description";
      };
    in assertContains "mkDirCreationCode includes description comment"
      code
      "Test description")

    # --- mkAllDirsCreationCode Tests ---

    (let
      code = volumesModule.mkAllDirsCreationCode {
        logs = { path = "/opt/logs"; };
        tmp = { path = "/opt/tmp"; };
      };
    in assertContains "mkAllDirsCreationCode generates function"
      code
      "create_runtime_dirs()")

    (let
      code = volumesModule.mkAllDirsCreationCode {
        logs = { path = "/opt/logs"; };
      };
    in assertContains "mkAllDirsCreationCode includes info message"
      code
      "Creating runtime directories")

    # --- Module Metadata Tests ---

    (assertEq "module meta name is correct"
      volumesModule.meta.name
      "libvolumes")

    (assertEq "module exports expected functions"
      (lib.elem "create_runtime_dir" volumesModule.exports)
      true)

    (assertEq "module exports verify_runtime_dir"
      (lib.elem "verify_runtime_dir" volumesModule.exports)
      true)

    # --- Functions Tests ---

    (assertContains "functions include create_runtime_dir"
      volumesModule.functions
      "create_runtime_dir()")

    (assertContains "functions include verify_runtime_dir"
      volumesModule.functions
      "verify_runtime_dir()")

    (assertContains "functions include is_likely_mount_point"
      volumesModule.functions
      "is_likely_mount_point()")
  ];

  # Run all tests
  testResults = runTests tests;

in {
  # Export test results
  inherit testResults;

  # Export individual test results for debugging
  allTests = tests;

  # Summary for nix-build output
  summary = pkgs.writeText "test-volumes-summary.txt" ''
    Volumes Module Test Results
    ===========================
    Total:  ${toString testResults.total}
    Passed: ${toString testResults.passedCount}
    Failed: ${toString testResults.failedCount}

    ${if testResults.allPassed then "All tests passed!" else ''
    Failed tests:
    ${lib.concatMapStringsSep "\n" (t: "  - ${t.name}: ${t.message or ""}") testResults.failed}
    ''}
  '';

  # For CI: derivation that fails if tests fail
  check = pkgs.runCommand "test-volumes-check" {} ''
    ${if testResults.allPassed
      then ''
        echo "All ${toString testResults.total} tests passed"
        touch $out
      ''
      else ''
        echo "Test failures:"
        ${lib.concatMapStringsSep "\n" (t: ''echo "  FAIL: ${t.name} - ${t.message or ""}"'') testResults.failed}
        exit 1
      ''
    }
  '';
}
