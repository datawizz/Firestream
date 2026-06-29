# Superset chart options: `redis.*` subchart passthrough.
#
# This top-level key passes through to the in-repo Bitnami redis subchart
# (Chart.yaml v4.0.1 lists redis 21.x.x as a dep). Superset uses Redis as
# the Celery broker AND result backend. We expose the canonical knobs the
# chart's own templates read (`auth.enabled`, `auth.password`,
# `auth.existingSecret`, `architecture`, `master.service.ports.redis`,
# `enabled`) and leave `freeformType` open for the full redis values tree.
#
# Mirrors the bundled-subchart pattern from
# `jupyterhub/nix/options/postgresql.nix` (no jupyterhub redis exists; this
# is the new bundled-redis sibling for hub-and-spoke superset).
{ lib, chartTypes, ... }:

let
  t = chartTypes;
  inherit (lib) mkOption types;
in {
  options.superset.redis = mkOption {
    default = null;
    description = "Redis subchart configuration (Bitnami redis)";
    type = types.nullOr (types.submodule {
      freeformType = types.attrsOf types.anything;

      options = {
        enabled = mkOption {
          type = types.nullOr types.bool;
          default = null;
          description = "Switch to enable or disable the Redis helm subchart";
        };

        architecture = mkOption {
          type = types.nullOr (types.enum [ "standalone" "replication" ]);
          default = null;
          description = "Redis architecture";
        };

        auth = mkOption {
          default = null;
          description = "Redis auth configuration consumed by Superset";
          type = types.nullOr t.secretType;
        };

        master = mkOption {
          default = null;
          description = "Redis master overrides";
          type = types.nullOr (types.submodule {
            freeformType = types.attrsOf types.anything;
            options = {
              service = mkOption {
                default = null;
                description = "Master service overrides";
                type = types.nullOr (types.submodule {
                  freeformType = types.attrsOf types.anything;
                  options = {
                    ports = mkOption {
                      default = null;
                      description = "Service port map";
                      type = types.nullOr (types.submodule {
                        freeformType = types.attrsOf types.anything;
                        options = {
                          redis = mkOption {
                            type = types.nullOr (types.either types.str types.int);
                            default = null;
                            description = "Redis service port";
                          };
                        };
                      });
                    };
                  };
                });
              };
            };
          });
        };
      };
    });
  };
}
