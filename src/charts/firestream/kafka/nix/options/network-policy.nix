# Kafka chart options: `networkPolicy.*`.
{ lib, ... }:

let
  inherit (lib) mkOption types;
in {
  options.kafka.networkPolicy = mkOption {
    default = null;
    description = "Network Policy configuration";
    type = types.nullOr (types.submodule {
      freeformType = types.attrsOf types.anything;

      options = {
        enabled = mkOption {
          type = types.nullOr types.bool;
          default = null;
          description = "Enable creation of NetworkPolicy resources";
        };

        allowExternal = mkOption {
          type = types.nullOr types.bool;
          default = null;
          description = "Don't require client label for connections";
        };

        allowExternalEgress = mkOption {
          type = types.nullOr types.bool;
          default = null;
          description = "Allow the pod to access any range of port and all destinations";
        };

        addExternalClientAccess = mkOption {
          type = types.nullOr types.bool;
          default = null;
          description = "Allow access from pods with client label set to true";
        };

        extraIngress = mkOption {
          type = types.nullOr (types.listOf types.attrs);
          default = null;
          description = "Add extra ingress rules to the NetworkPolicy";
        };

        extraEgress = mkOption {
          type = types.nullOr (types.listOf types.attrs);
          default = null;
          description = "Add extra egress rules to the NetworkPolicy";
        };

        ingressPodMatchLabels = mkOption {
          type = types.nullOr (types.attrsOf types.str);
          default = null;
          description = "Labels to match to allow traffic from other pods";
        };

        ingressNSMatchLabels = mkOption {
          type = types.nullOr (types.attrsOf types.str);
          default = null;
          description = "Labels to match to allow traffic from other namespaces";
        };

        ingressNSPodMatchLabels = mkOption {
          type = types.nullOr (types.attrsOf types.str);
          default = null;
          description = "Pod labels to match to allow traffic from other namespaces";
        };
      };
    });
  };
}
