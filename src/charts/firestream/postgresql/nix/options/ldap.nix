# PostgreSQL chart options: `ldap.*` (LDAP authentication).
{ lib, ... }:

let
  inherit (lib) mkOption types;
in {
  options.postgresql.ldap = mkOption {
    default = null;
    description = "LDAP authentication configuration";
    type = types.nullOr (types.submodule {
      options = {
        enabled = mkOption {
          type = types.nullOr types.bool;
          default = null;
          description = "Enable LDAP support";
        };

        server = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "IP address or name of the LDAP server";
        };

        port = mkOption {
          # Bitnami: "" by default, but can be int.
          type = types.nullOr (types.either types.str types.int);
          default = null;
          description = "Port number on the LDAP server";
        };

        prefix = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "String to prepend to the user name when forming the DN to bind";
        };

        suffix = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "String to append to the user name when forming the DN to bind";
        };

        basedn = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Root DN to begin the search for the user in";
        };

        binddn = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "DN of user to bind to LDAP";
        };

        bindpw = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Password for the user to bind to LDAP";
        };

        searchAttribute = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Attribute to match against the user name in the search";
        };

        searchFilter = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "The search filter to use when doing search+bind authentication";
        };

        scheme = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Set to 'ldaps' to use LDAPS";
        };

        tls = mkOption {
          default = null;
          description = "LDAP TLS configuration";
          type = types.nullOr (types.submodule {
            options = {
              enabled = mkOption {
                type = types.nullOr types.bool;
                default = null;
                description = "Set to true to enable TLS encryption";
              };
            };
          });
        };

        uri = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "LDAP URL beginning in the form ldap[s]://host[:port]/basedn";
        };
      };
    });
  };
}
