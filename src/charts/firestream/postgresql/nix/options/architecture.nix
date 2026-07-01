# PostgreSQL chart options: `architecture`, `replication.*`, `containerPorts.*`.
{ lib, ... }:

let
  inherit (lib) mkOption types;
in {
  options.postgresql = {
    architecture = mkOption {
      type = types.nullOr (types.enum [ "standalone" "replication" ]);
      default = null;
      description = "PostgreSQL architecture (standalone or replication)";
    };

    replication = mkOption {
      default = null;
      description = "Replication configuration (ignored if architecture is standalone)";
      type = types.nullOr (types.submodule {
        options = {
          synchronousCommit = mkOption {
            type = types.nullOr (types.enum [ "on" "remote_apply" "remote_write" "local" "off" ]);
            default = null;
            description = "Set synchronous commit mode";
          };

          numSynchronousReplicas = mkOption {
            type = types.nullOr types.int;
            default = null;
            description = "Number of replicas with synchronous replication";
          };

          applicationName = mkOption {
            type = types.nullOr types.str;
            default = null;
            description = "Cluster application name";
          };
        };
      });
    };

    containerPorts = mkOption {
      default = null;
      description = "Container ports";
      type = types.nullOr (types.submodule {
        freeformType = types.attrsOf types.anything;
        options = {
          postgresql = mkOption {
            type = types.nullOr types.int;
            default = null;
            description = "PostgreSQL container port";
          };
        };
      });
    };
  };
}
