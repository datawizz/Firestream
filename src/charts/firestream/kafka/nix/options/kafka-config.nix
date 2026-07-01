# Kafka chart options: chart-wide Kafka configuration (server.properties +
# log4j2 + heap + broker rack awareness + inter-broker protocol version).
#
# These keys all live at the TOP LEVEL of the values.yaml (alongside `image`
# and `clusterId`); they're grouped into one module because they conceptually
# describe the Kafka process configuration.
{ lib, ... }:

let
  inherit (lib) mkOption types;
in {
  options.kafka = {
    config = mkOption {
      # Bitnami accepts either a string (raw YAML/properties) or an attrset
      # (the chart transforms key: value pairs into properties format).
      type = types.nullOr (types.either types.str (types.attrsOf types.anything));
      default = null;
      description = "Content for Kafka configuration (auto-generated based on other parameters otherwise)";
    };

    overrideConfiguration = mkOption {
      type = types.nullOr (types.either types.str (types.attrsOf types.anything));
      default = null;
      description = "Kafka common configuration override. Values defined here take precedence over `config`";
    };

    existingConfigmap = mkOption {
      type = types.nullOr types.str;
      default = null;
      description = "Name of an existing ConfigMap with the Kafka configuration";
    };

    secretConfig = mkOption {
      type = types.nullOr types.str;
      default = null;
      description = "Additional configuration to be appended at the end of the generated Kafka configuration (stored in a secret)";
    };

    existingSecretConfig = mkOption {
      type = types.nullOr types.str;
      default = null;
      description = "Secret with additional configuration that will be appended to the end of the generated Kafka configuration";
    };

    log4j2 = mkOption {
      type = types.nullOr types.str;
      default = null;
      description = "Content for Kafka log4j2 configuration (default one is used otherwise)";
    };

    existingLog4j2ConfigMap = mkOption {
      type = types.nullOr types.str;
      default = null;
      description = "Name of an existing ConfigMap containing the log4j2.yaml file";
    };

    heapOpts = mkOption {
      type = types.nullOr types.str;
      default = null;
      description = "Kafka Java Heap configuration";
    };

    brokerRackAwareness = mkOption {
      default = null;
      description = "Kafka broker rack awareness configuration";
      type = types.nullOr (types.submodule {
        options = {
          enabled = mkOption {
            type = types.nullOr types.bool;
            default = null;
            description = "Enable Kafka Rack Awareness";
          };

          cloudProvider = mkOption {
            type = types.nullOr types.str;
            default = null;
            description = "Cloud provider to use to set Broker Rack Awareness (allowed: aws-az, azure)";
          };

          azureApiVersion = mkOption {
            type = types.nullOr types.str;
            default = null;
            description = "Metadata API version to use when cloudProvider=azure";
          };
        };
      });
    };

    interBrokerProtocolVersion = mkOption {
      type = types.nullOr types.str;
      default = null;
      description = "Override the setting 'inter.broker.protocol.version' during the ZK migration";
    };
  };
}
