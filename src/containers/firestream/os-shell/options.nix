# os-shell Container Options
# Copyright Firestream. MIT License.
#
# Externalized, declarative configuration for the os-shell container, consumed by
# bin/nix/firestream/containers/eval-container.nix. The os-shell container is a
# minimal busybox/bash sidecar image intended to replace `bitnami/os-shell:*` in
# chart init/probe sidecar slots (e.g. jupyterhub's `auxiliaryImage`). It does
# NOT run a long-lived daemon — charts replace its command/args with their own
# `sh -c '...'` scripts.
#
# Single-version: `version = "1"` is a semantic Firestream tag and intentionally
# does NOT mirror bitnami's `12-debian-12-r48`.

{ lib, ... }:

{
  config.os-shell = {
    version = lib.mkDefault "1";

    # Paths configuration. The os-shell sidecar has no persistent state of its
    # own; these paths only satisfy the firestream factory's mkAppModule
    # contract (it always exports `${APP}_BASE_DIR` etc. from envDefaults).
    # NOTE: paths use `osshell` (no hyphen) because the factory's internal
    # `name` is `osshell` (a valid bash identifier — see module.nix comment).
    # The external image name remains `firestream-os-shell:1`.
    paths = {
      base = lib.mkDefault "/opt/firestream/osshell";
      conf = lib.mkDefault "/opt/firestream/osshell/config";
      data = lib.mkDefault "/firestream/osshell/data";
      logs = lib.mkDefault "/opt/firestream/osshell/logs";
    };

    # No service-specific env. Empty default is fine — mkAppModule still
    # synthesises the standard `*_BASE_DIR` exports from `paths` above.
    env = builtins.mapAttrs (_: lib.mkDefault) { };
    envSecrets = lib.mkDefault [ ];

    # Image naming overrides; defaults align with the standard typed-options
    # pattern (eval-container.nix:85 -> repository "firestream-os-shell").
    image = {
      registry = lib.mkDefault null;
      repository = lib.mkDefault "firestream-os-shell";
      tag = lib.mkDefault null; # null -> falls back to version (i.e. "1")
    };

    # Sidecar: no ports, no long-lived service.
    exposedPorts = lib.mkDefault [ ];

    # Healthd is for long-running services — this container is a sidecar that
    # is `kubectl exec`'d into by chart probes. Explicitly off.
    health = {
      enable = lib.mkDefault false;
    };
  };
}
