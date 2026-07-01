# Redis chart options: `networkPolicy.*` (top-level chart NetworkPolicy).
#
# Mostly aligns with `chartTypes.networkPolicyType` (enabled, allowExternal,
# allowExternalEgress, extraIngress, extraEgress, ingressNSMatchLabels,
# ingressNSPodMatchLabels). Adds a `metrics.*` sub-block specific to the
# redis chart (separate allowExternal / namespace match for the exporter
# endpoint), so we declare locally rather than reusing the shared type
# directly. Same shape as the shared type for the common keys.
{ lib, ... }:

let
  inherit (lib) mkOption types;
in {
  options.redis.networkPolicy = mkOption {
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

        metrics = mkOption {
          default = null;
          description = "Metrics-endpoint NetworkPolicy overrides";
          type = types.nullOr (types.submodule {
            options = {
              allowExternal = mkOption {
                type = types.nullOr types.bool;
                default = null;
                description = "Don't require client label for connections for metrics endpoint";
              };

              ingressNSMatchLabels = mkOption {
                type = types.nullOr (types.attrsOf types.str);
                default = null;
                description = "Labels to match to allow traffic from other namespaces to metrics endpoint";
              };

              ingressNSPodMatchLabels = mkOption {
                type = types.nullOr (types.attrsOf types.str);
                default = null;
                description = "Pod labels to match to allow traffic from other namespaces to metrics endpoint";
              };
            };
          });
        };
      };
    });
  };
}
