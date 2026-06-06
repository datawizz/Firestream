# os-shell Container Module - Minimal sidecar shell image
# Copyright Firestream. MIT License.
#
# Builds the firestream-os-shell sidecar image. The container is intended to
# replace `bitnami/os-shell:*` in chart init/probe sidecar slots (jupyterhub's
# `auxiliaryImage` in particular). It is NOT a long-running daemon: charts
# replace `command:`/`args:` with their own `sh -c '...'` scripts, and probes
# `kubectl exec` into the container. The default entrypoint is therefore just a
# park-forever loop so the container survives if started bare.
#
# Image contents: a small POSIX userspace — bash, coreutils, busybox (for nc,
# basename, and other shell utilities), netcat-openbsd (for explicit `nc`
# semantics), curl (probe HTTP endpoints), gnused.

{ pkgs
, lib
, firestream
, version ? "1"

# Standard eval-container surface (defaults preserve byte-identical results
# whether invoked from the options-driven entrypoint or directly).
, paths ? {
    base = "/opt/firestream/osshell";
    conf = "/opt/firestream/osshell/config";
    data = "/firestream/osshell/data";
    logs = "/opt/firestream/osshell/logs";
  }

, envVars ? { }
, envVarsWithSecrets ? [ ]
, exposedPorts ? [ ]

# Sidecar — no in-image healthd.
, health ? { enable = false; port = 9180; readinessCmd = null; }

# Image naming passthrough.
, imageName ? "firestream-os-shell"
, imageTag ? version
}:

let
  # Runtime utilities the sidecar provides. Keep this list tight: it's the
  # promised contract for chart probe scripts.
  runtimeBinDeps = with pkgs; [
    bash
    coreutils
    busybox
    netcat-openbsd
    curl
    gnused
  ];

  # System libs needed for runtime (TLS for curl, etc.).
  systemDeps = with pkgs; [
    cacert
    openssl
  ];

in firestream.mkContainerModule {
  # Internal factory name MUST be a valid shell identifier (the factory upper-
  # cases it and uses it as a bash env var prefix in env-defaults.sh, e.g.
  # `${NAME}_BASE_DIR`). `os-shell` would produce `OS-SHELL_BASE_DIR` which is
  # not a valid identifier and fails at container startup. The external image
  # name (`firestream-os-shell`) is set via `imageName` and is unaffected; the
  # chart-side slot key is also unaffected (it's the flake-output attr name,
  # which is set in nix/flake-modules/containers/os-shell.nix).
  name = "osshell";
  inherit version;

  inherit paths envVars envVarsWithSecrets;
  inherit imageName imageTag;
  inherit exposedPorts;
  inherit health;

  # Default entrypoint: park forever so the container stays up if launched bare.
  # Charts override `command:` / `args:` (the standard k8s seam) so this is only
  # a fallback for `docker run` smoke tests.
  runCmd = ''
    exec sleep infinity
  '';

  inherit systemDeps runtimeBinDeps;

  # No persistent volumes for a sidecar shell.
  volumes = [ ];

  user = { name = "osshell"; group = "osshell"; uid = 1001; gid = 1001; };

  devShellPackages = with pkgs; [ docker ];
  devShellHook = ''
    echo "os-shell (firestream sidecar) ${version}"
    echo "Build:  nix build .#os-shell"
    echo "Load:   nix run  .#os-shell-image -- --load"
  '';
}
