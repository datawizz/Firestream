# Nextjs chart options: `networkPolicy.*`.
#
# Re-use chartTypes.networkPolicyType: enabled / allowExternal /
# allowExternalEgress / extraIngress / extraEgress / ingressNSMatchLabels /
# ingressNSPodMatchLabels. Standard Bitnami shape.
{ lib, chartTypes, ... }:

let
  t = chartTypes;
  inherit (lib) mkOption types;
in {
  options.nextjs.networkPolicy = mkOption {
    default = null;
    description = "NetworkPolicy configuration for Nextjs";
    type = types.nullOr t.networkPolicyType;
  };
}
