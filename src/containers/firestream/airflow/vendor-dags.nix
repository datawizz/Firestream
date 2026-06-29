# Airflow Vendored-DAGs Builder
# Copyright Firestream. MIT License.
#
# Build-time vendoring of DAG files into the Airflow image. Takes a list of
# DAG-source specs and produces a SINGLE derivation that lays every file out
# under `$out/opt/firestream/airflow/dags/...`. Because the container factory
# adds this derivation to `extraDeps` and the image builder links `/opt` into
# the rootfs (see bin/nix/firestream/containers/base.nix), the files land at
# `/opt/firestream/airflow/dags/` in the image — exactly Airflow's
# `dags_folder` (AIRFLOW_DAGS_DIR), so the scheduler/dag-processor pick them up
# with no chart or git-sync configuration.
#
# This is the Airflow analogue of ./vendor-addons.nix in the odoo container: a
# downstream flake supplies `config.airflow.vendoredDags = [ ... ]` and gets an
# Airflow image with those DAGs baked in, without forking Firestream.
#
# Layout invariant: `${src}/${sourceRoot}/<file>` -> image
# `/opt/firestream/airflow/dags/<file>`.
#
# Usage (from module.nix):
#   vendoredDagsDrv = import ./vendor-dags.nix { inherit pkgs lib; } {
#     version = airflowVersion;          # e.g. "3.0.3"
#     specs   = vendoredDags;            # list of resolved option submodules
#   };
#
# Spec shape (each list entry — see options.nix `vendoredDags` type):
#   { name; src ? null; owner ? null; repo ? null; rev ? null; hash ? null; sourceRoot ? "."; }
#   - `src` (a derivation/path), when non-null, WINS over owner/repo/rev/hash so
#     a local `dags/` folder, fetchgit, or flake input all work. This is the
#     common case for "bake my DAG".
#   - `sourceRoot` is the subdir inside the source holding the DAG files.
#   - All files/dirs under `${src}/${sourceRoot}` are copied recursively (so a
#     folder of DAGs plus shared helper modules all land together).

{ pkgs, lib }:

{ version, specs }:

let
  # Resolve each spec's source tree: explicit `src` wins, else fetchFromGitHub.
  resolveSrc = spec:
    if (spec.src or null) != null
    then spec.src
    else pkgs.fetchFromGitHub {
      owner = spec.owner;
      repo = spec.repo;
      rev = spec.rev;
      sha256 = spec.hash;
    };

  # Per-spec shell snippet that copies the source tree into the dags dir.
  copySnippet = spec:
    let
      src = resolveSrc spec;
      srcRoot = spec.sourceRoot or ".";
      label = spec.name or "${spec.owner or "?"}/${spec.repo or "?"}";
    in ''
      echo "[vendor-dags] ${label}: ${src}/${srcRoot}"
      srcdir="${src}/${srcRoot}"
      if [[ ! -d "$srcdir" ]]; then
        echo "[vendor-dags] ERROR: sourceRoot '${srcRoot}' not found in ${label}" >&2
        exit 1
      fi
      found=0
      for entry in "$srcdir"/*; do
        [[ -e "$entry" ]] || continue
        base="$(basename "$entry")"
        if [[ -e "$out/opt/firestream/airflow/dags/$base" ]]; then
          echo "[vendor-dags] ERROR: duplicate dag entry '$base' (from ${label})" >&2
          exit 1
        fi
        cp -r "$entry" "$out/opt/firestream/airflow/dags/$base"
        found=1
      done
      if [[ "$found" -eq 0 ]]; then
        echo "[vendor-dags] ERROR: no files found in ${label} (${srcRoot})" >&2
        exit 1
      fi
    '';
in
pkgs.runCommand "airflow-vendored-dags-${version}"
  {
    meta.description = "Vendored Airflow DAGs baked into /opt/firestream/airflow/dags";
  }
  ''
    mkdir -p "$out/opt/firestream/airflow/dags"
    ${lib.concatMapStrings copySnippet specs}
    # Make copies writable: store sources are read-only, but the runtime may
    # stat/touch DAG files. Bytecode writes go to a separate pycache dir.
    chmod -R u+w "$out/opt/firestream/airflow/dags"
  ''
