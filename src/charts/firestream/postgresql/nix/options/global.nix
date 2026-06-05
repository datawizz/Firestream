# PostgreSQL chart options: `global.*`
#
# Model A: every leaf is `nullOr`-wrapped and defaults to null so the generated
# values.yaml stays a sparse override file. Mirrors values.yaml `global:` 1:1.
{ lib, ... }:

let
  inherit (lib) mkOption types;
in {
  options.postgresql.global = mkOption {
    default = null;
    description = "Global Docker image parameters and cross-subchart settings";
    type = types.nullOr (types.submodule {
      freeformType = types.attrsOf types.anything;

      options = {
        imageRegistry = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Global Docker image registry";
        };

        imagePullSecrets = mkOption {
          type = types.nullOr (types.listOf types.str);
          default = null;
          description = "Global Docker registry secret names as an array";
        };

        defaultStorageClass = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Global default StorageClass for Persistent Volume(s)";
        };

        storageClass = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "DEPRECATED: use global.defaultStorageClass instead";
        };

        security = mkOption {
          default = null;
          description = "Global security settings";
          type = types.nullOr (types.submodule {
            options = {
              allowInsecureImages = mkOption {
                type = types.nullOr types.bool;
                default = null;
                description = "Allows skipping image verification";
              };
            };
          });
        };

        postgresql = mkOption {
          default = null;
          description = "Global PostgreSQL chart overrides (apply to both bundled and external PostgreSQL).";
          type = types.nullOr (types.submodule {
            freeformType = types.attrsOf types.anything;
            options = {
              fullnameOverride = mkOption {
                type = types.nullOr types.str;
                default = null;
                description = "Full chart name (overrides `fullnameOverride`)";
              };

              auth = mkOption {
                default = null;
                description = "Global PostgreSQL auth overrides";
                type = types.nullOr (types.submodule {
                  freeformType = types.attrsOf types.anything;
                  options = {
                    postgresPassword = mkOption {
                      type = types.nullOr types.str;
                      default = null;
                      description = "Password for the 'postgres' admin user (overrides `auth.postgresPassword`)";
                    };

                    username = mkOption {
                      type = types.nullOr types.str;
                      default = null;
                      description = "Name for a custom user to create (overrides `auth.username`)";
                    };

                    password = mkOption {
                      type = types.nullOr types.str;
                      default = null;
                      description = "Password for the custom user (overrides `auth.password`)";
                    };

                    database = mkOption {
                      type = types.nullOr types.str;
                      default = null;
                      description = "Name for a custom database to create (overrides `auth.database`)";
                    };

                    existingSecret = mkOption {
                      type = types.nullOr types.str;
                      default = null;
                      description = "Name of existing secret to use for PostgreSQL credentials (overrides `auth.existingSecret`)";
                    };

                    secretKeys = mkOption {
                      default = null;
                      description = "Names of keys in the existing secret to use for PostgreSQL credentials";
                      type = types.nullOr (types.submodule {
                        options = {
                          adminPasswordKey = mkOption {
                            type = types.nullOr types.str;
                            default = null;
                            description = "Name of key for admin password";
                          };

                          userPasswordKey = mkOption {
                            type = types.nullOr types.str;
                            default = null;
                            description = "Name of key for user password";
                          };

                          replicationPasswordKey = mkOption {
                            type = types.nullOr types.str;
                            default = null;
                            description = "Name of key for replication password";
                          };
                        };
                      });
                    };
                  };
                });
              };

              service = mkOption {
                default = null;
                description = "Global PostgreSQL service overrides";
                type = types.nullOr (types.submodule {
                  freeformType = types.attrsOf types.anything;
                  options = {
                    ports = mkOption {
                      default = null;
                      description = "Ports config";
                      type = types.nullOr (types.submodule {
                        freeformType = types.attrsOf types.anything;
                        options = {
                          postgresql = mkOption {
                            # Bitnami accepts string or int here (overrides primary.service.ports.postgresql).
                            type = types.nullOr (types.either types.str types.int);
                            default = null;
                            description = "PostgreSQL service port (overrides `service.ports.postgresql`)";
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

        compatibility = mkOption {
          default = null;
          description = "Compatibility adaptations for Kubernetes platforms";
          type = types.nullOr (types.submodule {
            options = {
              openshift = mkOption {
                default = null;
                description = "Adaptations for OpenShift";
                type = types.nullOr (types.submodule {
                  options = {
                    adaptSecurityContext = mkOption {
                      type = types.nullOr (types.enum [ "auto" "force" "disabled" ]);
                      default = null;
                      description = "Adapt the securityContext sections of the deployment to make them work on OpenShift";
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
