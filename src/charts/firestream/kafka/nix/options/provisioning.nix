# Kafka chart options: `provisioning.*` (topic provisioning Job + ACLs).
#
# Bitnami's provisioning Job runs at install/upgrade time to bootstrap
# topics and ACLs. It bundles its own ServiceAccount, security context,
# auth submodule (TLS for the kafka-config CLI client), and exposes
# extraProvisioningCommands + preScript / postScript for custom logic.
{ lib, chartTypes, ... }:

let
  t = chartTypes;
  inherit (lib) mkOption types;
in {
  options.kafka.provisioning = mkOption {
    default = null;
    description = "Kafka topic / ACL provisioning Job";
    type = types.nullOr (types.submodule {
      freeformType = types.attrsOf types.anything;

      options = {
        enabled = mkOption {
          type = types.nullOr types.bool;
          default = null;
          description = "Enable Kafka provisioning Job";
        };

        waitForKafka = mkOption {
          type = types.nullOr types.bool;
          default = null;
          description = "Init container that waits until Kafka is ready before provisioning";
        };

        useHelmHooks = mkOption {
          type = types.nullOr types.bool;
          default = null;
          description = "Flag to indicate usage of Helm hooks";
        };

        automountServiceAccountToken = mkOption {
          type = types.nullOr types.bool;
          default = null;
          description = "Mount Service Account token in pod";
        };

        numPartitions = mkOption {
          type = types.nullOr types.int;
          default = null;
          description = "Default number of partitions for topics when unspecified";
        };

        replicationFactor = mkOption {
          type = types.nullOr types.int;
          default = null;
          description = "Default replication factor for topics when unspecified";
        };

        topics = mkOption {
          type = types.nullOr (types.listOf types.attrs);
          default = null;
          description = "Kafka topics to provision";
        };

        nodeSelector = mkOption {
          type = types.nullOr (types.attrsOf types.str);
          default = null;
          description = "Node selector";
        };

        tolerations = mkOption {
          type = types.nullOr (types.listOf t.tolerationType);
          default = null;
          description = "Tolerations";
        };

        extraProvisioningCommands = mkOption {
          type = types.nullOr (types.listOf types.str);
          default = null;
          description = "Extra commands to run to provision cluster resources (e.g. ACLs)";
        };

        parallel = mkOption {
          type = types.nullOr types.int;
          default = null;
          description = "Number of provisioning commands to run at the same time";
        };

        preScript = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Extra bash script to run before topic provisioning";
        };

        postScript = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Extra bash script to run after topic provisioning";
        };

        auth = mkOption {
          default = null;
          description = "Auth configuration for the provisioning Job";
          type = types.nullOr (types.submodule {
            freeformType = types.attrsOf types.anything;
            options = {
              tls = mkOption {
                default = null;
                description = "TLS configuration for the provisioning Job";
                type = types.nullOr (types.submodule {
                  freeformType = types.attrsOf types.anything;
                  options = {
                    type = mkOption {
                      type = types.nullOr (types.enum [ "jks" "pem" "JKS" "PEM" ]);
                      default = null;
                      description = "TLS certificate format";
                    };

                    certificatesSecret = mkOption {
                      type = types.nullOr types.str;
                      default = null;
                      description = "Existing secret containing the TLS certificates";
                    };

                    cert = mkOption {
                      type = types.nullOr types.str;
                      default = null;
                      description = "Secret key for the cert (default tls.crt)";
                    };

                    key = mkOption {
                      type = types.nullOr types.str;
                      default = null;
                      description = "Secret key for the key (default tls.key)";
                    };

                    caCert = mkOption {
                      type = types.nullOr types.str;
                      default = null;
                      description = "Secret key for the CA cert (default ca.crt)";
                    };

                    keystore = mkOption {
                      type = types.nullOr types.str;
                      default = null;
                      description = "Secret key for the keystore";
                    };

                    truststore = mkOption {
                      type = types.nullOr types.str;
                      default = null;
                      description = "Secret key for the truststore";
                    };

                    passwordsSecret = mkOption {
                      type = types.nullOr types.str;
                      default = null;
                      description = "Secret containing passwords for the JKS/PEM files";
                    };

                    keyPasswordSecretKey = mkOption {
                      type = types.nullOr types.str;
                      default = null;
                      description = "Secret key for the PEM key password";
                    };

                    keystorePasswordSecretKey = mkOption {
                      type = types.nullOr types.str;
                      default = null;
                      description = "Secret key for the keystore password";
                    };

                    truststorePasswordSecretKey = mkOption {
                      type = types.nullOr types.str;
                      default = null;
                      description = "Secret key for the truststore password";
                    };

                    keyPassword = mkOption {
                      type = types.nullOr types.str;
                      default = null;
                      description = "Password to access the password-protected PEM key";
                    };

                    keystorePassword = mkOption {
                      type = types.nullOr types.str;
                      default = null;
                      description = "Password to access the JKS keystore";
                    };

                    truststorePassword = mkOption {
                      type = types.nullOr types.str;
                      default = null;
                      description = "Password to access the JKS truststore";
                    };
                  };
                });
              };
            };
          });
        };

        command = mkOption {
          type = types.nullOr (types.listOf types.str);
          default = null;
          description = "Override provisioning container command";
        };

        args = mkOption {
          type = types.nullOr (types.listOf types.str);
          default = null;
          description = "Override provisioning container arguments";
        };

        extraEnvVars = mkOption {
          type = types.nullOr (types.listOf types.attrs);
          default = null;
          description = "Extra environment variables";
        };

        extraEnvVarsCM = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "ConfigMap with extra environment variables";
        };

        extraEnvVarsSecret = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Secret with extra environment variables";
        };

        podAnnotations = mkOption {
          type = types.nullOr (types.attrsOf types.str);
          default = null;
          description = "Extra annotations for provisioning pods";
        };

        podLabels = mkOption {
          type = types.nullOr (types.attrsOf types.str);
          default = null;
          description = "Extra labels for provisioning pods";
        };

        serviceAccount = mkOption {
          default = null;
          description = "ServiceAccount for the provisioning Job";
          type = types.nullOr (types.submodule {
            options = {
              create = mkOption {
                type = types.nullOr types.bool;
                default = null;
                description = "Enable creation of ServiceAccount";
              };

              name = mkOption {
                type = types.nullOr types.str;
                default = null;
                description = "Name of the service account to use";
              };

              automountServiceAccountToken = mkOption {
                type = types.nullOr types.bool;
                default = null;
                description = "Allows auto mount of ServiceAccountToken";
              };
            };
          });
        };

        resourcesPreset = mkOption {
          type = t.resourcesPreset;
          default = null;
          description = "Resource preset";
        };

        resources = mkOption {
          type = types.nullOr t.resourceRequirements;
          default = null;
          description = "Resource requirements";
        };

        podSecurityContext = mkOption {
          type = types.nullOr t.podSecurityContext;
          default = null;
          description = "Pod security context";
        };

        containerSecurityContext = mkOption {
          type = types.nullOr t.containerSecurityContext;
          default = null;
          description = "Container security context";
        };

        schedulerName = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Alternate scheduler name";
        };

        enableServiceLinks = mkOption {
          type = types.nullOr types.bool;
          default = null;
          description = "Inject information about services as environment variables";
        };

        extraVolumes = mkOption {
          type = types.nullOr (types.listOf types.attrs);
          default = null;
          description = "Extra volumes";
        };

        extraVolumeMounts = mkOption {
          type = types.nullOr (types.listOf types.attrs);
          default = null;
          description = "Extra volume mounts";
        };

        sidecars = mkOption {
          type = types.nullOr (types.listOf types.attrs);
          default = null;
          description = "Additional sidecar containers";
        };

        initContainers = mkOption {
          type = types.nullOr (types.listOf types.attrs);
          default = null;
          description = "Additional init containers";
        };
      };
    });
  };
}
