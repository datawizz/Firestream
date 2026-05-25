# manifest.nix - Fleet SBOM Manifest & Source Archive Generation
# Copyright Firestream. MIT License.
#
# This module provides functions for aggregating individual container SBOMs
# into a unified fleet manifest at Nix build time (NO Import From Derivation).
#
# Key features:
# - Dynamic artifact collection (no hardcoded container lists)
# - Build-time SBOM merging via runCommand (avoids IFD)
# - CycloneDX 1.5 and SPDX 2.3 compliant output
# - Proper deduplication by purl
# - Fleet-level source code archiving for license compliance
#
# Architecture:
# ```
# Individual containers/artifacts    Fleet Manifest
# ┌─────────────────┐
# │ airflow         │──┐
# │ .metadata       │  │
# │ .packageList    │  │
# └─────────────────┘  │
# ┌─────────────────┐  │    ┌─────────────────────┐
# │ spark           │──┼───▶│ mkFleetManifest     │
# │ .metadata       │  │    │  (runCommand)       │
# │ .packageList    │  │    │                     │
# └─────────────────┘  │    │ firestream-vib      │
# ┌─────────────────┐  │    │  merge-sboms        │
# │ firestream-tui  │──┘    │  archive-sources    │
# │ .metadata       │       └─────────────────────┘
# │ .packageList    │                │
# └─────────────────┘                ▼
#                          ┌─────────────────────┐
#                          │ $out/               │
#                          │  sbom-cyclonedx.json│
#                          │  sbom-spdx.json     │
#                          │  manifest.json      │
#                          │  source_index.json  │
#                          │  sources/           │
#                          └─────────────────────┘
# ```
#
# IMPORTANT: This module does NOT use Import From Derivation (IFD).
# All derivations are passed as build inputs to runCommand, not read during eval.
# Source maps are generated at Nix eval time by introspecting .src attributes.
#
# Usage:
#   let
#     artifacts = manifestLib.collectArtifacts {
#       inherit airflow spark jupyterhub;
#       inherit (packages) firestream wait-for-port;
#     };
#     manifest = manifestLib.mkFleetManifest {
#       inherit artifacts;
#       version = "1.0.0";
#       archiveSources = true;
#     };
#   in manifest  # Derivation containing merged SBOMs + source archives

{ pkgs, lib, firestreamVibPkg }:

let
  # ── Source introspection helpers ────────────────────────────────────
  # These run at Nix evaluation time to extract source metadata from packages.

  # Safely get SPDX license identifier from a package
  getLicenseId = pkg:
    let
      meta = pkg.meta or {};
      license = meta.license or null;
    in
    if license == null then null
    else if builtins.isList license then
      let first = builtins.head license; in
      first.spdxId or first.shortName or null
    else
      license.spdxId or license.shortName or null;

  # Classify a package's source availability
  getSourceType = pkg:
    let
      pname = pkg.pname or pkg.name or "";
    in
    if pkg ? src || pkg ? srcs then "has-source"
    else if lib.hasPrefix "bootstrap" pname then "bootstrap"
    else "binary";

  # ── Artifact collection ────────────────────────────────────────────

  # Collect all artifacts tagged for fleet manifest inclusion
  # Filters packages that have either isFirestreamContainer or isFirestreamArtifact set
  #
  # Arguments:
  #   packages - attribute set of packages to filter
  #
  # Returns:
  #   attribute set containing only packages with fleet manifest tags
  collectArtifacts = packages:
    lib.filterAttrs (_name: pkg:
      # Accept packages that are either containers or Rust artifacts
      # and have a metadata derivation
      (pkg.isFirestreamContainer or false || pkg.isFirestreamArtifact or false)
      && (pkg.metadata or null) != null
    ) packages;

  # ── Fleet source map ───────────────────────────────────────────────

  # Build a unified source map across all fleet artifacts at Nix eval time.
  # Introspects .src/.srcs attributes on every package across all containers.
  # Deduplicates by Nix store path (listToAttrs keeps last occurrence).
  #
  # Arguments:
  #   artifacts - attribute set of artifacts with .packageList lists
  #
  # Returns:
  #   attribute set mapping store paths to source metadata
  # Strip Nix string context from store paths so that the resulting JSON
  # can be interpolated into runCommand without creating forbidden references.
  # The source paths are informational (consumed by firestream-vib at build time
  # by reading from /nix/store directly), not Nix dependency edges.
  strip = builtins.unsafeDiscardStringContext;

  buildFleetSourceMap = artifacts:
    let
      # Collect all packages from all artifacts
      allPackages = lib.concatMap (pkg:
        pkg.packageList or []
      ) (lib.attrValues artifacts);

      # Build source map entries
      entries = lib.filter (x: x != null) (map (pkg:
        let
          storePathStr =
            if builtins.isString pkg then pkg
            else if pkg ? outPath then builtins.toString pkg
            else null;
        in
        if storePathStr == null then null
        else {
          name = strip storePathStr;
          value = {
            srcPath =
              if pkg ? src then strip (builtins.toString pkg.src)
              else if pkg ? srcs then map (s: strip (builtins.toString s)) pkg.srcs
              else null;
            type = getSourceType pkg;
            license = getLicenseId pkg;
            pname = pkg.pname or pkg.name or null;
            version = pkg.version or null;
          };
        }
      ) allPackages);
    in
    # listToAttrs deduplicates by store path automatically
    lib.listToAttrs entries;

  # Collect all source derivations that need to be in the build sandbox.
  # These are passed as explicit build inputs so Nix fetches them,
  # while the source map JSON uses stripped paths (no string context).
  collectSourceDeps = artifacts:
    let
      allPackages = lib.concatMap (pkg:
        pkg.packageList or []
      ) (lib.attrValues artifacts);
    in
    lib.unique (lib.concatMap (pkg:
      (if pkg ? src then [ pkg.src ] else [])
      ++ (if pkg ? srcs then pkg.srcs else [])
    ) allPackages);

  # ── Fleet manifest builder ─────────────────────────────────────────

  # Build a fleet manifest from collected artifacts
  # This aggregates all individual SBOMs into unified CycloneDX and SPDX documents
  # and optionally archives source code for license compliance.
  #
  # Arguments:
  #   artifacts      - attribute set of artifacts with .metadata derivations
  #   fleetName      - name of the fleet (default: "firestream")
  #   version        - version string for the fleet manifest
  #   archiveSources - enable fleet-level source code archiving (default: false)
  #
  # Returns:
  #   derivation containing:
  #   - sbom-cyclonedx.json: Merged CycloneDX 1.5 SBOM
  #   - sbom-spdx.json: Merged SPDX 2.3 SBOM
  #   - manifest.json: Fleet inventory and summary
  #   - source_index.json: Source archive index (if archiveSources)
  #   - sources/: Deduplicated source tarballs (if archiveSources)
  mkFleetManifest = {
    artifacts,
    fleetName ? "firestream",
    version ? "dev",
    archiveSources ? false,
  }:
  let
    # Convert artifacts to a list with names for processing
    artifactList = lib.mapAttrsToList (name: pkg: {
      inherit name;
      metadata = pkg.metadata;
      isContainer = pkg.isFirestreamContainer or false;
      isArtifact = pkg.isFirestreamArtifact or false;
      pkgVersion = pkg.meta.version or "unknown";
    }) artifacts;

    # Generate input arguments for firestream-vib
    # Each input is passed as: --input <metadata-path>
    inputArgs = lib.concatMapStringsSep " " (artifact:
      "--input ${artifact.metadata}/opt/firestream"
    ) artifactList;

    # Generate artifact manifest entries for the summary JSON
    artifactManifestJson = builtins.toJSON (map (a: {
      name = a.name;
      version = a.pkgVersion;
      type = if a.isContainer then "container" else "artifact";
      metadata_path = "${a.metadata}/opt/firestream";
    }) artifactList);

    # Build fleet-level source map at Nix eval time (NOT build time)
    # This introspects .src attributes on all packages across all containers
    fleetSourceMap = buildFleetSourceMap artifacts;
    sourceMapJson = pkgs.writeText "fleet-source-map.json" (builtins.toJSON fleetSourceMap);

    # Collect actual source derivations so Nix fetches them into the sandbox.
    # The source map JSON has stripped string context (needed for runCommand),
    # so we pass sources as explicit build inputs to ensure they exist.
    sourceDeps = if archiveSources then collectSourceDeps artifacts else [];

  in pkgs.runCommand "firestream-fleet-manifest" {
    nativeBuildInputs = [ firestreamVibPkg pkgs.jq ]
      ++ lib.optionals archiveSources [ pkgs.gnutar pkgs.gzip ];

    # Pass metadata derivations as build inputs (NOT IFD - these are just paths)
    # Nix will build all metadata derivations first, then this runCommand
    metadataPaths = map (a: a.metadata) artifactList;

    # Pass source derivations so they're fetched into the build sandbox
    # (the source map JSON references these paths but without string context)
    inherit sourceDeps;

    # For reproducible timestamps, use SOURCE_DATE_EPOCH
    SOURCE_DATE_EPOCH = "0";

    # Pass configuration for the merge command
    inherit fleetName version;

    # Artifact manifest for summary generation
    artifactManifest = artifactManifestJson;

    passthru = {
      # Expose artifacts for introspection
      inherit artifacts artifactList;
      # Expose source map for debugging
      inherit fleetSourceMap;
    };
  } ''
    mkdir -p $out

    echo "Generating fleet manifest for ${fleetName} v${version}"
    echo "Artifacts: ${toString (map (a: a.name) artifactList)}"

    # Generate the merged CycloneDX SBOM
    echo "Merging SBOMs into CycloneDX format..."
    firestream-vib merge-sboms \
      --format cyclonedx \
      --fleet-name "${fleetName}" \
      --fleet-version "${version}" \
      ${inputArgs} \
      --output $out/sbom-cyclonedx.json

    # Generate the merged SPDX SBOM
    echo "Merging SBOMs into SPDX format..."
    firestream-vib merge-sboms \
      --format spdx \
      --fleet-name "${fleetName}" \
      --fleet-version "${version}" \
      ${inputArgs} \
      --output $out/sbom-spdx.json

    # Generate the fleet inventory manifest
    echo "Generating fleet inventory manifest..."
    firestream-vib merge-sboms \
      --format manifest \
      --fleet-name "${fleetName}" \
      --fleet-version "${version}" \
      ${inputArgs} \
      --output $out/manifest.json

    # Verify SBOM files were created
    for f in sbom-cyclonedx.json sbom-spdx.json manifest.json; do
      if [ ! -f "$out/$f" ]; then
        echo "ERROR: Failed to generate $f"
        exit 1
      fi
      echo "Generated: $out/$f ($(wc -c < "$out/$f") bytes)"
    done

    ${lib.optionalString archiveSources ''
      # Archive fleet source code for license compliance
      echo "Archiving fleet source code..."
      echo "Source map: ${sourceMapJson}"
      firestream-vib archive-sources \
        --source-map ${sourceMapJson} \
        --output $out

      if [ -f "$out/source_index.json" ]; then
        echo "Generated: $out/source_index.json ($(wc -c < "$out/source_index.json") bytes)"
      fi
      if [ -d "$out/sources" ]; then
        echo "Generated: $out/sources/ ($(ls "$out/sources" | wc -l) archives)"
      fi
    ''}

    echo "Fleet manifest generation complete"
  '';

in {
  meta = {
    name = "libmanifest";
    description = "Fleet SBOM manifest and source archive generation (no IFD)";
    version = "2.0.0";
  };

  # Export the collection function
  inherit collectArtifacts;

  # Export the manifest builder
  inherit mkFleetManifest;

  # Export for testing
  inherit buildFleetSourceMap;
}
