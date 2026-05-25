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

    # All other arguments are passed through to moduleArgs
    ...
  }@args:
  let
    # Load workspace from container's uv.lock
    workspace = uv2nix.lib.workspace.loadWorkspace {
      workspaceRoot = workspacePath;
    };

    # Create overlay preferring binary wheels
    overlay = workspace.mkPyprojectOverlay {
      sourcePreference = "wheel";
    };

    # Get container-specific overrides (default to empty)
    wheelOverrides = overrides.wheelOverrides or (final: prev: {});
    sourceOverrides = overrides.sourceOverrides or (final: prev: {});

    # Create base Python package set
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

    # Create virtual environment with all dependencies
    pythonEnv = pythonSet.mkVirtualEnv "${name}-env" workspace.deps.default;

    # ── Source archiving support ──────────────────────────────────────
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

    # Filter out factory-specific args, keep rest for moduleArgs passthrough
    extraArgs = builtins.removeAttrs args [
      "workspacePath" "name" "version" "python" "overrides" "moduleArgs"
    ];

    # Build base module arguments
    baseModuleArgs = {
      inherit pkgs lib pythonEnv python;
      firestream = firestreamLib;
      # Pass version with container-specific name (airflowVersion, odooVersion, etc.)
      "${name}Version" = version;
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
    # so the fleet manifest can introspect their .src attributes
    packageList = (module.packageList or []) ++ pythonSourcePkgs;

    config = (module.config or {}) // {
      inherit overrides;
    };
  };
}
