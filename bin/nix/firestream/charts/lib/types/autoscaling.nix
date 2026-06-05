# Autoscaling type definitions (HPA and VPA)
#
# Model A: every leaf is nullOr-wrapped and defaults to null so unset fields
# are stripped from the generated values.yaml.

{ lib, ... }:

let
  inherit (lib) mkOption types;

in {
  # Vertical Pod Autoscaler type
  vpaType = types.submodule {
    options = {
      enabled = mkOption {
        type = types.nullOr types.bool;
        default = null;
        description = "Enable Vertical Pod Autoscaler";
      };

      annotations = mkOption {
        type = types.nullOr (types.attrsOf types.str);
        default = null;
        description = "VPA annotations";
      };

      controlledResources = mkOption {
        type = types.nullOr (types.listOf types.str);
        default = null;
        description = "Resources to be controlled by VPA";
        example = [ "cpu" "memory" ];
      };

      maxAllowed = mkOption {
        type = types.nullOr (types.attrsOf types.str);
        default = null;
        description = "Maximum allowed resources";
        example = { cpu = "2"; memory = "4Gi"; };
      };

      minAllowed = mkOption {
        type = types.nullOr (types.attrsOf types.str);
        default = null;
        description = "Minimum allowed resources";
        example = { cpu = "100m"; memory = "128Mi"; };
      };

      updatePolicy = mkOption {
        type = types.nullOr (types.submodule {
          options = {
            updateMode = mkOption {
              type = types.nullOr (types.enum [ "Off" "Initial" "Recreate" "Auto" ]);
              default = null;
              description = "VPA update mode";
            };
          };
        });
        default = null;
        description = "VPA update policy";
      };
    };
  };

  # Horizontal Pod Autoscaler type
  hpaType = types.submodule {
    options = {
      enabled = mkOption {
        type = types.nullOr types.bool;
        default = null;
        description = "Enable Horizontal Pod Autoscaler";
      };

      minReplicas = mkOption {
        type = types.nullOr types.int;
        default = null;
        description = "Minimum number of replicas";
      };

      maxReplicas = mkOption {
        type = types.nullOr types.int;
        default = null;
        description = "Maximum number of replicas";
      };

      targetCPU = mkOption {
        type = types.nullOr types.int;
        default = null;
        description = "Target CPU utilization percentage";
        example = 80;
      };

      targetMemory = mkOption {
        type = types.nullOr types.int;
        default = null;
        description = "Target memory utilization percentage";
        example = 80;
      };

      behavior = mkOption {
        type = types.nullOr (types.submodule {
          options = {
            scaleDown = mkOption {
              type = types.nullOr (types.attrsOf types.anything);
              default = null;
              description = "Scale down behavior";
            };

            scaleUp = mkOption {
              type = types.nullOr (types.attrsOf types.anything);
              default = null;
              description = "Scale up behavior";
            };
          };
        });
        default = null;
        description = "HPA scaling behavior";
      };
    };
  };

  # Combined autoscaling type
  autoscalingType = types.submodule {
    options = {
      vpa = mkOption {
        type = types.nullOr (types.submodule {
          options = {
            enabled = mkOption {
              type = types.nullOr types.bool;
              default = null;
              description = "Enable Vertical Pod Autoscaler";
            };

            annotations = mkOption {
              type = types.nullOr (types.attrsOf types.str);
              default = null;
              description = "VPA annotations";
            };

            controlledResources = mkOption {
              type = types.nullOr (types.listOf types.str);
              default = null;
              description = "Resources to be controlled by VPA";
            };

            maxAllowed = mkOption {
              type = types.nullOr (types.attrsOf types.str);
              default = null;
              description = "Maximum allowed resources";
            };

            minAllowed = mkOption {
              type = types.nullOr (types.attrsOf types.str);
              default = null;
              description = "Minimum allowed resources";
            };

            updatePolicy = mkOption {
              type = types.nullOr (types.submodule {
                options = {
                  updateMode = mkOption {
                    type = types.nullOr (types.enum [ "Off" "Initial" "Recreate" "Auto" ]);
                    default = null;
                    description = "VPA update mode";
                  };
                };
              });
              default = null;
              description = "VPA update policy";
            };
          };
        });
        default = null;
        description = "Vertical Pod Autoscaler configuration";
      };

      hpa = mkOption {
        type = types.nullOr (types.submodule {
          options = {
            enabled = mkOption {
              type = types.nullOr types.bool;
              default = null;
              description = "Enable Horizontal Pod Autoscaler";
            };

            minReplicas = mkOption {
              type = types.nullOr types.int;
              default = null;
              description = "Minimum number of replicas";
            };

            maxReplicas = mkOption {
              type = types.nullOr types.int;
              default = null;
              description = "Maximum number of replicas";
            };

            targetCPU = mkOption {
              type = types.nullOr types.int;
              default = null;
              description = "Target CPU utilization percentage";
            };

            targetMemory = mkOption {
              type = types.nullOr types.int;
              default = null;
              description = "Target memory utilization percentage";
            };

            behavior = mkOption {
              type = types.nullOr (types.submodule {
                options = {
                  scaleDown = mkOption {
                    type = types.nullOr (types.attrsOf types.anything);
                    default = null;
                    description = "Scale down behavior";
                  };

                  scaleUp = mkOption {
                    type = types.nullOr (types.attrsOf types.anything);
                    default = null;
                    description = "Scale up behavior";
                  };
                };
              });
              default = null;
              description = "HPA scaling behavior";
            };
          };
        });
        default = null;
        description = "Horizontal Pod Autoscaler configuration";
      };
    };
  };
}
