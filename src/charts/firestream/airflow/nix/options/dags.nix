# Airflow chart options: `dags.*` (load DAGs from Git repositories).
{ lib, chartTypes, ... }:

let
  t = chartTypes;
  inherit (lib) mkOption types;
in {
  options.airflow.dags = mkOption {
    default = null;
    description = "Load DAGs from Git repositories";
    type = types.nullOr (types.submodule {
      options = {
        enabled = mkOption {
          type = types.nullOr types.bool;
          default = null;
          description = "Enable loading DAGs from Git repositories";
        };

        existingConfigmap = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Name of an existing ConfigMap with the DAGs";
        };

        repositories = mkOption {
          type = types.nullOr (types.listOf t.gitRepositoryType);
          default = null;
          description = "Array of repositories from which to download DAGs";
        };

        sshKey = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "SSH private key used to clone/sync DAGs (ignored if existingSshKeySecret is set)";
        };

        existingSshKeySecret = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Name of a secret containing the SSH private key used to clone/sync DAGs";
        };

        existingSshKeySecretKey = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Key in the existing secret containing the SSH private key";
        };
      };
    });
  };
}
