# test-sources.nix - Tests for fleet-level source archive functionality
# Copyright Firestream. MIT License.
#
# Tests source code archiving at the fleet manifest level.
# Source archives are generated alongside SBOMs in the fleet manifest,
# NOT in individual container metadata (which stays lean for runtime images).

{ pkgs, firestream }:

let
  lib = pkgs.lib;

  # Get the manifest library
  manifestLib = firestream.manifest;
  metadataLib = firestream.lib.metadata;

  # Create test packages with known sources
  testPackageA = pkgs.hello;
  testPackageB = pkgs.coreutils;

  # Create two test container-like metadata derivations
  metadataA = metadataLib.mkContainerMetadata {
    name = "test-container-a";
    version = "1.0.0";
    mainDrv = testPackageA;
  };

  metadataB = metadataLib.mkContainerMetadata {
    name = "test-container-b";
    version = "1.0.0";
    mainDrv = testPackageB;
  };

  # Simulate artifact modules with packageList and metadata
  artifactA = {
    metadata = metadataA;
    packageList = [ testPackageA ];
    isFirestreamContainer = true;
    meta = { version = "1.0.0"; };
  };

  artifactB = {
    metadata = metadataB;
    packageList = [ testPackageB ];
    isFirestreamContainer = true;
    meta = { version = "1.0.0"; };
  };

  # Build fleet manifest WITHOUT source archiving
  manifestNoSources = manifestLib.mkFleetManifest {
    artifacts = { test-a = artifactA; test-b = artifactB; };
    fleetName = "test-fleet";
    version = "1.0.0";
    archiveSources = false;
  };

  # Build fleet manifest WITH source archiving
  manifestWithSources = manifestLib.mkFleetManifest {
    artifacts = { test-a = artifactA; test-b = artifactB; };
    fleetName = "test-fleet";
    version = "1.0.0";
    archiveSources = true;
  };

  # Test 1: Verify fleet manifest without sources has no source files
  testManifestNoSources = pkgs.runCommand "test-manifest-no-sources" {} ''
    echo "Testing fleet manifest without source archiving..."

    # SBOM files should exist
    test -f ${manifestNoSources}/sbom-cyclonedx.json \
      || (echo "FAIL: sbom-cyclonedx.json not found" && exit 1)
    test -f ${manifestNoSources}/sbom-spdx.json \
      || (echo "FAIL: sbom-spdx.json not found" && exit 1)
    test -f ${manifestNoSources}/manifest.json \
      || (echo "FAIL: manifest.json not found" && exit 1)

    # Source files should NOT exist
    if [ -f ${manifestNoSources}/source_index.json ]; then
      echo "FAIL: source_index.json should not exist when archiveSources=false"
      exit 1
    fi
    if [ -d ${manifestNoSources}/sources ]; then
      echo "FAIL: sources/ should not exist when archiveSources=false"
      exit 1
    fi

    echo "PASS: Fleet manifest without sources"
    touch $out
  '';

  # Test 2: Verify source archiving generates files at fleet level
  testFleetSourceArchiving = pkgs.runCommand "test-fleet-source-archiving" {
    nativeBuildInputs = [ pkgs.jq ];
  } ''
    echo "Testing fleet-level source archive generation..."

    # SBOM files should still exist
    test -f ${manifestWithSources}/sbom-cyclonedx.json \
      || (echo "FAIL: sbom-cyclonedx.json not found" && exit 1)
    test -f ${manifestWithSources}/manifest.json \
      || (echo "FAIL: manifest.json not found" && exit 1)

    # Source index should exist
    test -f ${manifestWithSources}/source_index.json \
      || (echo "FAIL: source_index.json not found" && exit 1)

    # Validate source_index.json structure
    jq -e '.schema_version == "1.0"' ${manifestWithSources}/source_index.json \
      || (echo "FAIL: Invalid schema_version" && exit 1)
    jq -e '.coverage' ${manifestWithSources}/source_index.json \
      || (echo "FAIL: Missing coverage field" && exit 1)
    jq -e '.entries' ${manifestWithSources}/source_index.json \
      || (echo "FAIL: Missing entries field" && exit 1)

    echo "PASS: Fleet-level source archive generation"
    touch $out
  '';

  # Test 3: Verify source coverage statistics
  testCoverageStats = pkgs.runCommand "test-coverage-stats" {
    nativeBuildInputs = [ pkgs.jq ];
  } ''
    echo "Testing source coverage statistics..."

    total=$(jq -r '.coverage.total_packages' ${manifestWithSources}/source_index.json)
    has_source=$(jq -r '.coverage.has_source' ${manifestWithSources}/source_index.json)

    echo "Total packages: $total"
    echo "Has source: $has_source"

    if [ "$total" -lt 1 ]; then
      echo "FAIL: Expected at least 1 package in coverage"
      exit 1
    fi

    echo "PASS: Source coverage statistics"
    touch $out
  '';

  # Test 4: Verify source index entry structure
  testEntryStructure = pkgs.runCommand "test-entry-structure" {
    nativeBuildInputs = [ pkgs.jq ];
  } ''
    echo "Testing source index entry structure..."

    first_entry=$(jq '.entries[0]' ${manifestWithSources}/source_index.json)

    echo "$first_entry" | jq -e '.purl' \
      || (echo "FAIL: Entry missing purl field" && exit 1)
    echo "$first_entry" | jq -e '.name' \
      || (echo "FAIL: Entry missing name field" && exit 1)
    echo "$first_entry" | jq -e '.source_type' \
      || (echo "FAIL: Entry missing source_type field" && exit 1)

    echo "PASS: Source index entry structure"
    touch $out
  '';

  # Test 5: Verify individual container metadata has NO source archives
  testContainerMetadataClean = pkgs.runCommand "test-container-metadata-clean" {} ''
    echo "Testing that container metadata has no source archives..."

    # Container metadata should NOT have source_index.json
    if [ -f ${metadataA}/opt/firestream/source_index.json ]; then
      echo "FAIL: Container metadata should not contain source_index.json"
      exit 1
    fi

    # Container metadata should NOT have sources/ directory
    if [ -d ${metadataA}/opt/firestream/sources ]; then
      echo "FAIL: Container metadata should not contain sources/"
      exit 1
    fi

    # But it should have the basic metadata files
    test -f ${metadataA}/opt/firestream/metadata.json \
      || (echo "FAIL: metadata.json not found in container metadata" && exit 1)
    test -f ${metadataA}/opt/firestream/sbom-cyclonedx.json \
      || (echo "FAIL: sbom-cyclonedx.json not found in container metadata" && exit 1)

    echo "PASS: Container metadata is clean (no source archives)"
    touch $out
  '';

in pkgs.runCommand "firestream-source-tests" {
  buildInputs = [
    testManifestNoSources
    testFleetSourceArchiving
    testCoverageStats
    testEntryStructure
    testContainerMetadataClean
  ];
} ''
  echo "================================================"
  echo "All fleet source archive tests passed!"
  echo "================================================"
  echo "Manifest no sources:    PASSED"
  echo "Fleet source archiving: PASSED"
  echo "Coverage stats:         PASSED"
  echo "Entry structure:        PASSED"
  echo "Container metadata:     PASSED"
  echo "================================================"
  touch $out
''
