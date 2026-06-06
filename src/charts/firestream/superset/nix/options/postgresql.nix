# Superset chart options: `postgresql.*` subchart passthrough.
#
# This top-level key passes through to the in-repo Bitnami postgresql
# subchart. We expose the canonical knobs the chart's own templates read
# (`auth.username`, `auth.password`, `auth.database`, `auth.existingSecret`,
# `auth.enablePostgresUser`, `architecture`, `primary.service.ports.postgresql`,
# `enabled`) and leave `freeformType` open for the full postgresql values
# tree (any field the upstream Bitnami postgresql chart accepts).
#
# Mirrors `jupyterhub/nix/options/postgresql.nix`.
{ lib, ... }:

let
  inherit (lib) mkOption types;
in {
  options.superset.postgresql = mkOption {
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
          description = "PostgreSQL auth configuration consumed by Superset";
          type = types.nullOr (types.submodule {
            freeformType = types.attrsOf types.anything;
            options = {
              enablePostgresUser = mkOption {
                type = types.nullOr types.bool;
                default = null;
                description = "Assign a password to the 'postgres' admin user";
              };
              username = mkOption {
                type = types.nullOr types.str;
                default = null;
                description = "Name for a custom user to create";
              };
              password = mkOption {
                type = types.nullOr types.str;
                default = null;
                description = "Password for the custom user (random if empty)";
              };
              database = mkOption {
                type = types.nullOr types.str;
                default = null;
                description = "Name for a custom database to create";
              };
              existingSecret = mkOption {
                type = types.nullOr types.str;
                default = null;
                description = "Name of an existing secret to use for PostgreSQL credentials";
              };
            };
          });
        };

        primary = mkOption {
          default = null;
          description = "PostgreSQL primary node overrides";
          type = types.nullOr (types.submodule {
            freeformType = types.attrsOf types.anything;
            options = {
              service = mkOption {
                default = null;
                description = "Primary service overrides";
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
      };
    });
  };
}
