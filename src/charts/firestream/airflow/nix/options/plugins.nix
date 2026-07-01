# Airflow chart options: `plugins.*` (load custom plugins from Git repositories).
{ lib, chartTypes, ... }:

let
  t = chartTypes;
  inherit (lib) mkOption types;
in {
  options.airflow.plugins = mkOption {
    default = null;
    description = "Load custom plugins from Git repositories";
    type = types.nullOr (types.submodule {
      options = {
        enabled = mkOption {
          type = types.nullOr types.bool;
          default = null;
          description = "Enable loading plugins from Git repositories";
        };

        repositories = mkOption {
          type = types.nullOr (types.listOf t.gitRepositoryType);
          default = null;
          description = "Array of repositories from which to download plugins";
        };

        sshKey = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "SSH private key used to clone/sync plugins (ignored if existingSshKeySecret is set)";
        };

        existingSshKeySecret = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Name of a secret containing the SSH private key used to clone/sync plugins";
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
