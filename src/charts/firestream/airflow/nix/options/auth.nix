# Airflow chart options: `auth.*` (web UI credentials & app secrets).
{ lib, ... }:

let
  inherit (lib) mkOption types;
in {
  options.airflow.auth = mkOption {
    default = null;
    description = "Airflow authentication / secret-key parameters";
    type = types.nullOr (types.submodule {
      options = {
        username = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Username to access web UI";
        };

        password = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Password to access web UI";
        };

        fernetKey = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Fernet key to secure connections";
        };

        secretKey = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Secret key to run web server";
        };

        jwtSecretKey = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "JWT secret key to secure connections between components";
        };

        existingSecret = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Name of an existing secret to use for Airflow credentials";
        };
      };
    });
  };
}
