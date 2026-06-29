# JupyterHub chart options: `postgresql.*` subchart passthrough.
#
# This top-level key passes through to the in-repo Bitnami postgresql
# subchart. We expose the canonical knobs the chart's own jupyterhub_config
# template reads (`auth.username`, `auth.password`, `auth.database`,
# `auth.existingSecret`, `architecture`, `service.ports.postgresql`,
# `enabled`) and leave `freeformType` open for the full postgresql values
# tree (any field the upstream Bitnami postgresql chart accepts).
{ lib, chartTypes, ... }:

let
  t = chartTypes;
  inherit (lib) mkOption types;
in {
  options.jupyterhub.postgresql = mkOption {
    default = null;
    description = "PostgreSQL subchart configuration (Bitnami postgresql)";
    type = types.nullOr (types.submodule {
      freeformType = types.attrsOf types.anything;

      options = {
        enabled = mkOption {
          type = types.nullOr types.bool;
          default = null;
          description = "Switch to enable or disable the PostgreSQL helm subchart";
        };

        architecture = mkOption {
          type = types.nullOr (types.enum [ "standalone" "replication" ]);
          default = null;
          description = "PostgreSQL architecture";
        };

        auth = mkOption {
          default = null;
          description = "PostgreSQL auth configuration consumed by the hub";
          type = types.nullOr t.secretType;
        };

        service = mkOption {
          default = null;
          description = "PostgreSQL subchart service overrides";
          type = types.nullOr (types.submodule {
            freeformType = types.attrsOf types.anything;
            options = {
              ports = mkOption {
                default = null;
                description = "Service port map";
                type = types.nullOr (types.submodule {
                  freeformType = types.attrsOf types.anything;
                  options = {
                    postgresql = mkOption {
                      type = types.nullOr (types.either types.str types.int);
                      default = null;
                      description = "PostgreSQL service port";
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
