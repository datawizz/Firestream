# Airflow chart options: `ldap.*` (LDAP authentication).
{ lib, ... }:

let
  inherit (lib) mkOption types;
in {
  options.airflow.ldap = mkOption {
    default = null;
    description = "LDAP authentication configuration";
    type = types.nullOr (types.submodule {
      options = {
        enabled = mkOption {
          type = types.nullOr types.bool;
          default = null;
          description = "Enable LDAP authentication";
        };

        uri = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Server URI, eg. ldap://ldap_server:389";
        };

        basedn = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Base of the search, eg. ou=people,dc=example,dc=org";
        };

        searchAttribute = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "LDAP search attribute to detect the user";
        };

        firstnameField = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Field used to retrieve the firstname";
        };

        lastnameField = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Field used to retrieve the lastname";
        };

        emailField = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Field used to retrieve the email";
        };

        binddn = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "DN of the account used to search in the LDAP server";
        };

        bindpw = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Bind password";
        };

        existingSecret = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Name of an existing secret to use for LDAP credentials";
        };

        userRegistration = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Set to 'True' to enable user self registration";
        };

        userRegistrationRole = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Sets the default role assigned to user (eg. Public, Admin)";
        };

        rolesMapping = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Mapping from LDAP DN to a list of Airflow roles";
        };

        rolesSyncAtLogin = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Replace ALL the user's roles each login, or only on registration";
        };

        tls = mkOption {
          default = null;
          description = "LDAP TLS configuration";
          type = types.nullOr (types.submodule {
            options = {
              enabled = mkOption {
                type = types.nullOr types.bool;
                default = null;
                description = "Enable LDAP over TLS (LDAPS)";
              };

              allowSelfSigned = mkOption {
                type = types.nullOr types.bool;
                default = null;
                description = "Allow self-signed certificates";
              };

              certificatesSecret = mkOption {
                type = types.nullOr types.str;
                default = null;
                description = "Name of the existing secret containing the certificates";
              };

              certificatesMountPath = mkOption {
                type = types.nullOr types.str;
                default = null;
                description = "Path where the certificates will be mounted";
              };

              CAFilename = mkOption {
                type = types.nullOr types.str;
                default = null;
                description = "CA certificate filename";
              };
            };
          });
        };
      };
    });
  };
}
