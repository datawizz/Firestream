# Airflow chart options: `triggerer.*` (Airflow triggerer StatefulSet).
#
# Composition: shared component base type (mkComponentType, incl. `enabled`)
# plus triggerer-only extras (defaultCapacity, persistence,
# persistentVolumeClaimRetentionPolicy, service).
{ lib, chartTypes, ... }:

let
  t = chartTypes;
  inherit (lib) mkOption types;
in {
  options.airflow.triggerer = mkOption {
    default = { };
    description = "Airflow triggerer parameters";
    # Splice the shared component modules (via getSubModules) with triggerer extras.
    type = types.submodule ((t.mkComponentType { componentName = "triggerer"; }).getSubModules ++ [
      {
        options = {
          defaultCapacity = mkOption {
            type = types.nullOr types.int;
            default = null;
            description = "Maximum number of triggers each triggerer will run at once";
          };

          persistence = mkOption {
            default = null;
            description = "Persistence configuration for the triggerer";
            type = types.nullOr (types.submodule {
              options = {
                enabled = mkOption {
                  type = types.nullOr types.bool;
                  default = null;
                  description = "Enable persistence using Persistent Volume Claims";
                };

                storageClass = mkOption {
                  type = types.nullOr types.str;
                  default = null;
                  description = "PVC Storage Class for the triggerer data volume";
                };

                annotations = mkOption {
                  type = types.nullOr (types.attrsOf types.str);
                  default = null;
                  description = "Annotations for the PVC";
                };

                accessModes = mkOption {
                  type = types.nullOr (types.listOf types.str);
                  default = null;
                  description = "PVC Access Modes for the triggerer data volume";
                };

                size = mkOption {
                  type = types.nullOr types.str;
                  default = null;
                  description = "PVC Storage Request for the triggerer data volume";
                };

                selector = mkOption {
                  type = types.nullOr (types.attrsOf types.anything);
                  default = null;
                  description = "Selector to match an existing Persistent Volume for the data PVC";
                };

                dataSource = mkOption {
                  type = types.nullOr (types.attrsOf types.anything);
                  default = null;
                  description = "Custom PVC data source";
                };

                existingClaim = mkOption {
                  type = types.nullOr types.str;
                  default = null;
                  description = "Name of an existing PVC to use";
                };
              };
            });
          };

          persistentVolumeClaimRetentionPolicy = mkOption {
            default = null;
            description = "PVC retention policy for the triggerer StatefulSet";
            type = types.nullOr (types.submodule {
              options = {
                enabled = mkOption {
                  type = types.nullOr types.bool;
                  default = null;
                  description = "Enable Persistent volume retention policy for the StatefulSet";
                };

                whenScaled = mkOption {
                  type = types.nullOr (types.enum [ "Retain" "Delete" ]);
                  default = null;
                  description = "Volume retention behavior when the replica count is reduced";
                };

                whenDeleted = mkOption {
                  type = types.nullOr (types.enum [ "Retain" "Delete" ]);
                  default = null;
                  description = "Volume retention behavior that applies when the StatefulSet is deleted";
                };
              };
            });
          };

          service = mkOption {
            type = types.nullOr t.serviceType;
            default = null;
            description = "Triggerer headless/logs service parameters";
          };
        };
      }
    ]);
  };
}
