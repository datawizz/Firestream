# Network policy type definition
#
# Model A: every leaf is nullOr-wrapped and defaults to null so unset fields
# are stripped from the generated values.yaml.

{ lib, ... }:

let
  inherit (lib) mkOption types;

in {
  # NetworkPolicy configuration type
  networkPolicyType = types.submodule {
    options = {
      enabled = mkOption {
        type = types.nullOr types.bool;
        default = null;
        description = "Enable NetworkPolicy";
      };

      allowExternal = mkOption {
        type = types.nullOr types.bool;
        default = null;
        description = "Allow external connections";
      };

      allowExternalEgress = mkOption {
        type = types.nullOr types.bool;
        default = null;
        description = "Allow external egress traffic";
      };

      extraIngress = mkOption {
        type = types.nullOr (types.listOf types.attrs);
        default = null;
        description = "Additional ingress rules";
        example = [
          {
            from = [
              { namespaceSelector.matchLabels."kubernetes.io/metadata.name" = "monitoring"; }
            ];
            ports = [
              { port = 8080; protocol = "TCP"; }
            ];
          }
        ];
      };

      extraEgress = mkOption {
        type = types.nullOr (types.listOf types.attrs);
        default = null;
        description = "Additional egress rules";
        example = [
          {
            to = [
              { ipBlock.cidr = "10.0.0.0/8"; }
            ];
            ports = [
              { port = 5432; protocol = "TCP"; }
            ];
          }
        ];
      };

      ingressNSMatchLabels = mkOption {
        type = types.nullOr (types.attrsOf types.str);
        default = null;
        description = "Namespace labels for ingress rules";
      };

      ingressNSPodMatchLabels = mkOption {
        type = types.nullOr (types.attrsOf types.str);
        default = null;
        description = "Pod labels within namespace for ingress rules";
      };
    };
  };
}
