# Kafka chart options: `listeners.*`.
#
# Four named listeners (`client`, `controller`, `interbroker`, `external`) each
# describe a logical port + protocol mapping. A `freeformType` is attached so
# extraListeners and the optional `overrideListeners` / `advertisedListeners` /
# `securityProtocolMap` overrides round-trip without modification.
{ lib, ... }:

let
  inherit (lib) mkOption types;

  # Each named listener is a small typed submodule (name / containerPort /
  # protocol / sslClientAuth). Bitnami's enum for protocol is
  # PLAINTEXT|SASL_PLAINTEXT|SASL_SSL|SSL; sslClientAuth is none|requested|required
  # or empty. We keep the field types narrow but tolerant: `protocol` is open
  # `nullOr str` (a user might pass a Bitnami-internal value not in the doc).
  listenerSubmodule = types.submodule {
    freeformType = types.attrsOf types.anything;
    options = {
      name = mkOption {
        type = types.nullOr types.str;
        default = null;
        description = "Listener name";
      };

      containerPort = mkOption {
        type = types.nullOr types.int;
        default = null;
        description = "Container port for the listener";
      };

      protocol = mkOption {
        type = types.nullOr types.str;
        default = null;
        description = "Security protocol (PLAINTEXT | SASL_PLAINTEXT | SASL_SSL | SSL)";
      };

      sslClientAuth = mkOption {
        type = types.nullOr types.str;
        default = null;
        description = "mTLS client-auth mode (none | requested | required | empty)";
      };
    };
  };
in {
  options.kafka.listeners = mkOption {
    default = null;
    description = "Kafka listeners configuration";
    type = types.nullOr (types.submodule {
      freeformType = types.attrsOf types.anything;

      options = {
        client = mkOption {
          type = types.nullOr listenerSubmodule;
          default = null;
          description = "Client-facing listener";
        };

        controller = mkOption {
          type = types.nullOr listenerSubmodule;
          default = null;
          description = "KRaft controller listener";
        };

        interbroker = mkOption {
          type = types.nullOr listenerSubmodule;
          default = null;
          description = "Inter-broker listener";
        };

        external = mkOption {
          type = types.nullOr listenerSubmodule;
          default = null;
          description = "External listener";
        };

        extraListeners = mkOption {
          type = types.nullOr (types.listOf types.attrs);
          default = null;
          description = "Array of listener objects to be appended to already existing listeners";
        };

        overrideListeners = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Override Kafka 'listeners' configuration setting (string overrides all named listeners above)";
        };

        advertisedListeners = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Override Kafka 'advertised.listeners' configuration setting";
        };

        securityProtocolMap = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Override Kafka 'security.protocol.map' configuration setting";
        };
      };
    });
  };
}
