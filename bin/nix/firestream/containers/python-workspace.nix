# Python Workspace Container Factory
# Copyright Firestream. MIT License.
#
# This module combines uv2nix workspace loading with mkPythonContainerModule.
# It allows building Python containers from a workspace directory containing
# pyproject.toml and uv.lock without needing a separate flake.nix per container.
#
# Usage:
#   let
#     container = mkPythonWorkspaceContainer {
#       workspacePath = ./src/containers/firestream/airflow;
#       name = "airflow";
#       version = "3.0.3";
#       python = pkgs.python312;
#       overrides = import ./overrides.nix { inherit pkgs lib; };
#     };
#   in container.dockerImage

{ pkgs
, lib
, mkPythonContainerModule
, firestreamLib
, uv2nix
, pyproject-nix
, pyproject-build-systems
}:

{
  # Factory function for Python workspace containers
  mkPythonWorkspaceContainer = {
    # Required: path to directory with pyproject.toml/uv.lock/module.nix
    workspacePath,

    # Required: container name
    name,

    # Required: container version
    version,

    # Optional: Python interpreter (default: python312)
    python ? pkgs.python312,

    # Optional: wheel and source overrides
    # Expected structure: { wheelOverrides = final: prev: { ... }; sourceOverrides = final: prev: { ... }; }
    overrides ? {},

    # Optional: additional arguments to pass to module.nix
    # These are merged directly into the module call
    moduleArgs ? {},

    # Optional: additional, separate uv2nix venvs to build alongside the
    # primary workspace (e.g. guest Airflow DAG deps). Each entry:
    #   { name; src; overrides ? {}; python ? <primary python>; }
    # Each becomes its own baked venv at /opt/firestream/${name}/${w.name}-venv.
    extraWorkspaces ? [],

    # All other arguments are passed through to moduleArgs
    ...
  }@args:
  let
    # ── Workspace builder ─────────────────────────────────────────────
    # Loads a uv2nix workspace and resolves it TWICE:
    #   - wheel overlay  → runtime `pythonEnv` (mkVirtualEnv)
    #   - sdist overlay  → `pythonSourcePkgs` for license-compliance source
    #                      archiving / SBOM (mkVirtualEnv exposes no per-pkg .src)
    # It builds its OWN `pythonBase` so each workspace honors its own interpreter.
    buildWorkspace = { workspacePath, overrides ? {}, python, envName }:
    let
      # Load workspace from container's uv.lock
      workspace = uv2nix.lib.workspace.loadWorkspace {
        workspaceRoot = workspacePath;
      };

      # ── requires-python guardrail (loud) ────────────────────────────
      # Fail fast with a legible eval error if the effective interpreter does
      # not satisfy the workspace's declared project.requires-python, instead
      # of surfacing later as a cryptic wheel-tag / silently-mismatched venv.
      requiresPython =
        (builtins.fromTOML (builtins.readFile (workspacePath + "/pyproject.toml")))
          .project.requires-python or null;
      pythonGuard =
        if requiresPython == null then null
        else let
          conds = pyproject-nix.lib.pep440.parseVersionConds requiresPython;
          ver = pyproject-nix.lib.pep440.parseVersion python.version;
          ok = builtins.all
            (c: pyproject-nix.lib.pep440.comparators.${c.op} ver c.version)
            conds;
        in if ok then null
           else throw ''
             python-workspace: interpreter python-${python.version} does not satisfy requires-python "${requiresPython}" declared in ${toString workspacePath}/pyproject.toml'';

      # Create overlay preferring binary wheels
      overlay = workspace.mkPyprojectOverlay {
        sourcePreference = "wheel";
      };

      # Get container-specific overrides (default to empty)
      wheelOverrides = overrides.wheelOverrides or (final: prev: {});
      sourceOverrides = overrides.sourceOverrides or (final: prev: {});

      # Create base Python package set (per-workspace, honors its own interpreter)
      pythonBase = pkgs.callPackage pyproject-nix.build.packages {
        inherit python;
      };

      # Compose all overlays - ORDER MATTERS
      # 1. Build systems FIRST (provides build backends like setuptools, hatchling, etc.)
      # 2. Workspace overlay second (provides package definitions from uv.lock)
      # 3. Source build overrides third (fixes for packages built from source)
      # 4. Wheel runtime overrides last (runtime library dependencies)
      pythonSet = pythonBase.overrideScope (
        lib.composeManyExtensions [
          pyproject-build-systems.overlays.default
          overlay
          sourceOverrides
          wheelOverrides
        ]
      );

      # Create virtual environment with all dependencies.
      # `seq` on the guard forces the requires-python check whenever the runtime
      # env is realised, without altering the derivation (drvPath unchanged).
      pythonEnv = builtins.seq pythonGuard
        (pythonSet.mkVirtualEnv envName workspace.deps.default);

      # ── Source archiving support ────────────────────────────────────
      # Parallel resolution with sdist preference for source code archiving.
      # Only used for .src introspection — the runtime container uses wheel-based pythonEnv.
      sdistOverlay = workspace.mkPyprojectOverlay {
        sourcePreference = "sdist";
      };

      pythonSetSdist = pythonBase.overrideScope (
        lib.composeManyExtensions [
          pyproject-build-systems.overlays.default
          sdistOverlay
          sourceOverrides
        ]
      );

      # Extract source-resolved packages for fleet-level source archiving.
      # workspace.deps.default has the dependency names; look each up in the sdist set.
      pythonSourcePkgs = let
        depNames = builtins.attrNames (workspace.deps.default);
      in lib.filter (p: p != null) (map (name:
        let pkg = builtins.tryEval (
          if pythonSetSdist ? ${name} then pythonSetSdist.${name} else null
        );
        in if pkg.success then pkg.value else null
      ) depNames);
    in {
      inherit pythonEnv pythonSet workspace pythonSourcePkgs;
    };

    # Build the PRIMARY workspace. Outputs are byte-identical to the pre-factory
    # inline form when extraWorkspaces == [].
    primary = buildWorkspace {
      inherit workspacePath overrides python;
      envName = "${name}-env";
    };
    inherit (primary) pythonEnv pythonSet workspace pythonSourcePkgs;

    # Build each additional workspace as its own baked venv, mounted as a sibling
    # of the primary venv at /opt/firestream/${name}/${w.name}-venv.
    extraBuilt = map (w:
      (buildWorkspace {
        workspacePath = w.src;
        overrides = w.overrides or {};
        python = w.python or python;
        envName = "${w.name}-env";
      }) // {
        inherit (w) name;
        mountPath = "/opt/firestream/${name}/${w.name}-venv";
      }
    ) extraWorkspaces;

    # Filter out factory-specific args, keep rest for moduleArgs passthrough
    extraArgs = builtins.removeAttrs args [
      "workspacePath" "name" "version" "python" "overrides" "moduleArgs"
      "extraWorkspaces"
    ];

    # Guest venvs for module.nix to materialize. Passed ONLY when there is at
    # least one extra workspace: module.nix has no `...` catch-all, so an
    # unconditional key would break every container whose module.nix predates
    # the `extraPythonEnvs ? []` arg — and would perturb the primary closure.
    extraPythonEnvs = map (b: { inherit (b) name pythonEnv mountPath; }) extraBuilt;

    # Build base module arguments
    baseModuleArgs = {
      inherit pkgs lib pythonEnv python;
      firestream = firestreamLib;
      # Pass version with container-specific name (airflowVersion, odooVersion, etc.)
      "${name}Version" = version;
    } // lib.optionalAttrs (extraBuilt != []) {
      inherit extraPythonEnvs;
    };

    # Merge all module arguments:
    # 1. Base args (pkgs, lib, pythonEnv, etc.)
    # 2. Extra args passed to factory (e.g., odooSource)
    # 3. Explicit moduleArgs
    allModuleArgs = baseModuleArgs // extraArgs // moduleArgs;

    # Import container module with computed arguments
    module = import (workspacePath + "/module.nix") allModuleArgs;

  in module // {
    # Add additional exports for debugging/introspection
    inherit pythonEnv pythonSet workspace pythonSourcePkgs;

    # Append Python source packages to the container's packageList
    # so the fleet manifest can introspect their .src attributes.
    # Extra-workspace source pkgs are folded in for SBOM completeness; the
    # manifest layer dedups by store path, so no pname dedup is needed here.
    packageList = (module.packageList or [])
      ++ pythonSourcePkgs
      ++ lib.concatMap (b: b.pythonSourcePkgs) extraBuilt;

    config = (module.config or {}) // {
      inherit overrides;
    };
  };
}
