# Kafka chart options: KRaft cluster identity (clusterId, existingKraftSecret,
# kraftVersion). These are top-level values keys (not nested under a `kraft:`
# section in the upstream chart) but live in their own option module here so
# default.nix's `imports` reads as a list of cohesive concerns.
{ lib, ... }:

let
  inherit (lib) mkOption types;
in {
  options.kafka = {
    clusterId = mkOption {
      type = types.nullOr types.str;
      default = null;
      description = ''
        Kafka KRaft cluster ID (ignored if existingKraftSecret is set). A random
        cluster ID will be generated the 1st time KRaft is initialised if not set.
      '';
    };

    existingKraftSecret = mkOption {
      type = types.nullOr types.str;
      default = null;
      description = "Name of the secret containing the Kafka KRaft Cluster ID and one directory ID per controller replica";
    };

    kraftVersion = mkOption {
      # Bitnami: integer (0 for static quorum, 1 for dynamic quorum). Some
      # users pass it as a string in YAML; accept either to avoid breaking
      # render-fidelity on the rare config that quotes it.
      type = types.nullOr (types.either types.str types.int);
      default = null;
      description = "KRaft version (kraftVersion=0 => static quorum, kraftVersion=1 => dynamic quorum)";
    };
  };
}
