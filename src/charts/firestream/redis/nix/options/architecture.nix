# Redis chart options: `architecture` (standalone vs. replication).
{ lib, ... }:

let
  inherit (lib) mkOption types;
in {
  options.redis = {
    architecture = mkOption {
      type = types.nullOr (types.enum [ "standalone" "replication" ]);
      default = null;
      description = "Redis architecture (standalone or replication)";
    };
  };
}
