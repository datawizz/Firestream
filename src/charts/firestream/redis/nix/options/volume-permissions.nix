# Redis chart options: `volumePermissions.*` (init container that fixes
# ownership on the persistent volume).
{ lib, chartTypes, ... }:

let
  t = chartTypes;
  inherit (lib) mkOption types;
in {
  options.redis.volumePermissions = mkOption {
    default = null;
    description = "Init container for fixing volume permissions";
    type = types.nullOr (types.submodule {
      freeformType = types.attrsOf types.anything;
      options = {
        enabled = mkOption {
          type = types.nullOr types.bool;
          default = null;
          description = "Enable the init container that changes the owner and group of the persistent volume";
        };

        image = mkOption {
          type = types.nullOr t.imageType;
          default = null;
          description = "Init container volume-permissions image";
        };

        resourcesPreset = mkOption {
          type = t.resourcesPreset;
          default = null;
          description = "Resource preset for the init container";
        };

        resources = mkOption {
          type = types.nullOr t.resourceRequirements;
          default = null;
          description = "Resource requirements for the init container";
        };

        containerSecurityContext = mkOption {
          # Bitnami's shape here is unusual: only seLinuxOptions + runAsUser are
          # specified by default (no `enabled` flag). Use a freeform submodule
          # so callers can supply runAsUser: "auto" (OpenShift) or any other
          # security-context field they need.
          default = null;
          description = "Init container security context";
          type = types.nullOr (types.submodule {
            freeformType = types.attrsOf types.anything;
            options = {
              seLinuxOptions = mkOption {
                type = types.nullOr (types.attrsOf types.anything);
                default = null;
                description = "Set SELinux options in container";
              };

              runAsUser = mkOption {
                # Bitnami accepts the special string "auto" OR an int UID.
                type = types.nullOr (types.either types.str types.int);
                default = null;
                description = "Set init container's Security Context runAsUser (use 'auto' for OpenShift)";
              };
            };
          });
        };

        extraEnvVars = mkOption {
          type = types.nullOr (types.listOf types.attrs);
          default = null;
          description = "Extra environment variables for the volume-permissions init container";
        };
      };
    });
  };
}
