# Container Evaluation Mechanism
# Copyright Firestream. MIT License.
#
# This module provides a uniform, options-driven entrypoint for evaluating a
# single container. It declares a standard option schema under the container's
# name, evaluates it together with the container's own options modules, then
# dispatches to the appropriate firestream container factory based on
# runtimeType.
#
# It is NOT wired into the flake outputs in Phase 1 - it is introduced here so
# later phases can migrate containers onto a shared, declarative options schema.
#
# Usage (later phases):
#   let
#     result = (import ./eval-container.nix { inherit pkgs lib firestreamLib; }) {
#       name = "airflow";
#       runtimeType = "python-workspace";
#       workspacePath = ./airflow;
#       modules = [ ./airflow/options.nix ];
#     };
#   in result.dockerImage

{ pkgs, lib, firestreamLib }:

{
  name,
  runtimeType,
  modules ? [],
  workspacePath ? null,
  modulePath ? null,
  extraFactoryArgs ? {},
}:

let
  # Standard option schema declared under the ${name} namespace.
  # The container's own options.nix supplies the concrete `version`.
  mkContainerOptions = { name }: { lib, ... }: {
    options.${name} = {
      version = lib.mkOption {
        type = lib.types.str;
        description = "Container version (supplied by the container's options.nix).";
      };

      python = lib.mkOption {
        type = lib.types.nullOr lib.types.package;
        default = null;
        description = "Python interpreter package (python-workspace runtime).";
      };

      paths = lib.mkOption {
        type = lib.types.attrsOf lib.types.str;
        default = {};
        description = "Filesystem path overrides for the container.";
      };

      env = lib.mkOption {
        type = lib.types.attrsOf lib.types.str;
        default = {};
        description = "Environment variables to bake into the container.";
      };

      envSecrets = lib.mkOption {
        type = lib.types.listOf lib.types.str;
        default = [];
        description = "Names of environment variables sourced from secrets at runtime.";
      };

      exposedPorts = lib.mkOption {
        type = lib.types.listOf lib.types.int;
        default = [];
        description = "TCP ports the container exposes.";
      };

      image = lib.mkOption {
        default = {};
        type = lib.types.submodule {
          options = {
            registry = lib.mkOption {
              type = lib.types.nullOr lib.types.str;
              default = null;
              description = "Optional image registry. When null, no registry is prepended.";
            };
            repository = lib.mkOption {
              type = lib.types.str;
              default = "firestream-${name}";
              description = "Image repository name.";
            };
            tag = lib.mkOption {
              type = lib.types.nullOr lib.types.str;
              default = null;
              description = "Optional image tag. When null, falls back to version.";
            };
          };
        };
        description = "Docker image naming configuration.";
      };

      extraModuleArgs = lib.mkOption {
        type = lib.types.attrsOf lib.types.raw;
        default = {};
        description = "Additional arguments forwarded to the container factory/module.";
      };

      # In-image health/SBOM service (firestream-healthd). Phase 3: opt-in per
      # container. When enabled, the base.nix entrypoint wrapper launches
      # firestream-healthd in the background before exec'ing the inner
      # entrypoint, exposing /healthz, /readyz, /metadata, /sbom, /closure on
      # `health.port`. Default-off: byte-identical to a non-Phase-3 image.
      health = lib.mkOption {
        default = {};
        description = "Background firestream-healthd configuration.";
        type = lib.types.submodule {
          options = {
            enable = lib.mkOption {
              type = lib.types.bool;
              default = false;
              description = ''
                If true, run firestream-healthd in the background of the
                container, exposing /healthz, /readyz, /metadata, /sbom,
                /closure. When false (the default), the container image is
                byte-identical to a pre-Phase-3 image and no health server
                is launched.
              '';
            };
            port = lib.mkOption {
              type = lib.types.int;
              default = 9180;
              description = ''
                Bind port for firestream-healthd (container-side). Use
                compose.hostPortOffset for host-side mapping. When health.enable
                is true, this port is appended to exposedPorts.
              '';
            };
            readinessCmd = lib.mkOption {
              type = lib.types.nullOr lib.types.str;
              default = null;
              description = ''
                Shell command (as a single string, passed to `sh -c`) whose
                exit-0 means ready. Used for /readyz when set. When null and
                no other readiness mode is configured, /readyz returns 200
                unconditionally.
              '';
            };
          };
        };
      };

      # Declarative docker-compose topology. Rendered to YAML by
      # nix/flake-modules/compose.nix into packages.<name>-compose and driven by
      # apps.<name>-up / apps.<name>-down. Generated from this config so the
      # image tags in the compose file always match what the flake builds.
      compose = lib.mkOption {
        default = {};
        description = "Declarative docker-compose deployment topology.";
        type = lib.types.submodule {
          options = {
            enable = lib.mkOption {
              type = lib.types.bool;
              default = true;
              description = "Whether to emit a docker-compose derivation for this container.";
            };
            projectName = lib.mkOption {
              type = lib.types.str;
              default = "firestream-${name}";
              description = "docker compose project name (-p).";
            };
            hostPortOffset = lib.mkOption {
              type = lib.types.int;
              default = 20000;
              description = ''
                Added to every published HOST port (container side unchanged) so
                concurrent stacks can run side-by-side without colliding on the
                host AND so canonical/well-known service ports on the developer
                box (e.g. a local postgres on 5432, redis on 6379, spark on 8080)
                are never grabbed by a firestream e2e stack by default. Applied to:
                  (a) the default single-service branch in compose.nix
                      (which publishes `{p}:{p}` for every exposedPort), and
                  (b) numeric host sides of any service-declared `ports`
                      entries in `compose.services.<n>.ports`.
                Non-numeric or IP-prefixed entries (e.g. "127.0.0.1:5432:5432")
                are passed through unchanged.
                Default 20000 keeps every canonical stack out of the privileged-
                feeling 0-9999 range. Individual stacks can pin a different
                offset if they need a stable host port (see airflow).
              '';
            };
            dependencies = lib.mkOption {
              type = lib.types.listOf lib.types.str;
              default = [];
              description = ''
                Image registry keys (e.g. "postgresql", "redis") whose images this
                stack needs. The `up` app builds + loads each before `docker compose up`.
              '';
            };
            sharedEnv = lib.mkOption {
              type = lib.types.attrsOf lib.types.str;
              default = {};
              description = "Environment merged into every OWN service (the container's own image).";
            };
            volumes = lib.mkOption {
              type = lib.types.attrsOf (lib.types.attrsOf lib.types.anything);
              default = {};
              description = "Named top-level docker-compose volumes (e.g. { postgresql_data = {}; }).";
            };
            services = lib.mkOption {
              default = {};
              description = ''
                Service definitions. When empty, the generator synthesizes a single
                service from imageRef + exposedPorts. An `image` of null resolves to
                this container's own image; "@<key>" resolves to a dependency image's
                ref (tag stays in sync); any other string is used literally.
              '';
              type = lib.types.attrsOf (lib.types.submodule {
                options = {
                  image = lib.mkOption {
                    type = lib.types.nullOr lib.types.str;
                    default = null;
                    description = "null ⇒ own image; \"@key\" ⇒ dependency image ref; else literal.";
                  };
                  env = lib.mkOption {
                    type = lib.types.attrsOf lib.types.str;
                    default = {};
                    description = "Per-service environment (merged over sharedEnv for own services).";
                  };
                  ports = lib.mkOption {
                    type = lib.types.listOf lib.types.str;
                    default = [];
                    description = "Published ports, e.g. [ \"8090:8080\" ].";
                  };
                  volumes = lib.mkOption {
                    type = lib.types.listOf lib.types.str;
                    default = [];
                    description = "Volume mounts, e.g. [ \"postgresql_data:/firestream/postgresql\" ].";
                  };
                  dependsOn = lib.mkOption {
                    type = lib.types.listOf lib.types.str;
                    default = [];
                    description = "Other service names this one waits on (service_healthy condition).";
                  };
                  healthcheck = lib.mkOption {
                    type = lib.types.nullOr (lib.types.attrsOf lib.types.anything);
                    default = null;
                    description = "docker-compose healthcheck block (test/interval/timeout/retries).";
                  };
                  restart = lib.mkOption {
                    type = lib.types.str;
                    default = "";
                    description = "Restart policy (e.g. \"unless-stopped\"); empty ⇒ omitted.";
                  };
                };
              });
            };
          };
        };
      };
    };
  };

  evaled = lib.evalModules {
    modules = [ (mkContainerOptions { inherit name; }) ] ++ modules;
    specialArgs = { inherit pkgs lib; };
  };

  cfg = evaled.config.${name};

  imageTag = if cfg.image.tag != null then cfg.image.tag else cfg.version;

  imageName =
    if cfg.image.registry != null
    then "${cfg.image.registry}/${cfg.image.repository}"
    else cfg.image.repository;

  # Load the container's Python wheel/source build overrides, mirroring the
  # legacy flake which passed `overrides = import .../overrides.nix { pkgs; lib; }`
  # into mkPythonWorkspaceContainer. Every python-workspace container ships an
  # overrides.nix in its workspace; without these the Python closure differs
  # (and some packages fail to build). The { pkgs, lib } call contract is
  # unchanged from the legacy flake.
  overrides =
    if workspacePath != null && builtins.pathExists (workspacePath + "/overrides.nix")
    then import (workspacePath + "/overrides.nix") { inherit pkgs lib; }
    else {};

  # When health.enable is true, the healthd port joins the container's exposed
  # ports so:
  #   (a) the docker image's ExposedPorts includes it,
  #   (b) the default single-service compose publishes it on the host,
  #   (c) compose.hostPortOffset applies to it uniformly with the app ports.
  # When health.enable is false, exposedPorts is byte-identical to today.
  effectiveExposedPorts =
    if cfg.health.enable
    then cfg.exposedPorts ++ [ cfg.health.port ]
    else cfg.exposedPorts;

  # Health configuration handed to base.nix's mkContainerModule via the
  # factory arg `health`. base.nix wraps the entrypoint only when enable=true;
  # the binary is already in imageContents from Phase 2.
  healthConfig = {
    enable = cfg.health.enable;
    port = cfg.health.port;
    readinessCmd = cfg.health.readinessCmd;
  };

  # Pass the health config to the leaf module only when enabled. This keeps
  # modules that haven't opted in (Phase 4 work) free of `health` in their
  # signature; the byte-identity guarantee for disabled containers is
  # preserved because base.nix's default `health.enable` is also false.
  healthArg = lib.optionalAttrs cfg.health.enable { health = healthConfig; };

  module =
    if runtimeType == "python-workspace" then
      firestreamLib.mkPythonWorkspaceContainer ({
        inherit workspacePath name;
        version = cfg.version;
        python = cfg.python;
        inherit overrides;
        moduleArgs = {
          envVars = cfg.env;
          envVarsWithSecrets = cfg.envSecrets;
          paths = cfg.paths;
          exposedPorts = effectiveExposedPorts;
          inherit imageName imageTag;
        } // healthArg // cfg.extraModuleArgs;
      } // extraFactoryArgs)
    else if runtimeType == "system" || runtimeType == "java" then
      import modulePath ({
        inherit pkgs lib;
        firestream = firestreamLib;
        version = cfg.version;
        envVars = cfg.env;
        envVarsWithSecrets = cfg.envSecrets;
        paths = cfg.paths;
        exposedPorts = effectiveExposedPorts;
        inherit imageName imageTag;
      } // healthArg // extraFactoryArgs // cfg.extraModuleArgs)
    else
      throw "eval-container: unknown runtimeType ${runtimeType}";

in {
  dockerImage = module.dockerImage;
  metadata = module.metadata or null;
  module = module;
  config = evaled.config;
  options = evaled.options;

  # Surfaced for the compose generator (nix/flake-modules/compose.nix) and the
  # consumer API. These are cheap strings/attrs derived from options — accessing
  # them never forces the (Linux-only) dockerImage build, so they evaluate on
  # Darwin too.
  inherit imageName imageTag;
  imageRef = "${imageName}:${imageTag}";
  compose = cfg.compose;
  exposedPorts = effectiveExposedPorts;
  env = cfg.env;
  health = cfg.health;
}
