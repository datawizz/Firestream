# Kafka chart options: `sasl.*` (SASL authentication credentials & mechanism).
#
# Covers PLAIN / SCRAM-SHA-256/512 / OAUTHBEARER credentials for client,
# interbroker, and controller listeners, plus the existingSecret pointer.
{ lib, ... }:

let
  inherit (lib) mkOption types;
in {
  options.kafka.sasl = mkOption {
    default = null;
    description = "Kafka SASL settings for authentication";
    type = types.nullOr (types.submodule {
      freeformType = types.attrsOf types.anything;

      options = {
        enabledMechanisms = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Comma-separated list of allowed SASL mechanisms (PLAIN, SCRAM-SHA-256, SCRAM-SHA-512, OAUTHBEARER)";
        };

        interBrokerMechanism = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "SASL mechanism for inter-broker communication";
        };

        controllerMechanism = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "SASL mechanism for controller communications";
        };

        oauthbearer = mkOption {
          default = null;
          description = "Settings for OAuthBearer mechanism";
          type = types.nullOr (types.submodule {
            freeformType = types.attrsOf types.anything;
            options = {
              tokenEndpointUrl = mkOption {
                type = types.nullOr types.str;
                default = null;
                description = "OAuth/OIDC token endpoint URL";
              };

              jwksEndpointUrl = mkOption {
                type = types.nullOr types.str;
                default = null;
                description = "JWKS endpoint URL";
              };

              expectedAudience = mkOption {
                type = types.nullOr types.str;
                default = null;
                description = "Comma-delimited list of expected audiences for JWT verification";
              };

              subClaimName = mkOption {
                type = types.nullOr types.str;
                default = null;
                description = "OAuth claim name for the subject";
              };
            };
          });
        };

        interbroker = mkOption {
          default = null;
          description = "Credentials for inter-broker communications";
          type = types.nullOr (types.submodule {
            freeformType = types.attrsOf types.anything;
            options = {
              user = mkOption {
                type = types.nullOr types.str;
                default = null;
                description = "Username for inter-broker communications when SASL is enabled";
              };

              password = mkOption {
                type = types.nullOr types.str;
                default = null;
                description = "Password for inter-broker communications when SASL is enabled";
              };

              clientId = mkOption {
                type = types.nullOr types.str;
                default = null;
                description = "Client ID for inter-broker communications (OAUTHBEARER)";
              };

              clientSecret = mkOption {
                type = types.nullOr types.str;
                default = null;
                description = "Client Secret for inter-broker communications (OAUTHBEARER)";
              };
            };
          });
        };

        controller = mkOption {
          default = null;
          description = "Credentials for controller communications";
          type = types.nullOr (types.submodule {
            freeformType = types.attrsOf types.anything;
            options = {
              user = mkOption {
                type = types.nullOr types.str;
                default = null;
                description = "Username for controller communications when SASL is enabled";
              };

              password = mkOption {
                type = types.nullOr types.str;
                default = null;
                description = "Password for controller communications when SASL is enabled";
              };

              clientId = mkOption {
                type = types.nullOr types.str;
                default = null;
                description = "Client ID for controller communications (OAUTHBEARER)";
              };

              clientSecret = mkOption {
                type = types.nullOr types.str;
                default = null;
                description = "Client Secret for controller communications (OAUTHBEARER)";
              };
            };
          });
        };

        client = mkOption {
          default = null;
          description = "Credentials for client communications";
          type = types.nullOr (types.submodule {
            freeformType = types.attrsOf types.anything;
            options = {
              users = mkOption {
                # Bitnami: array of usernames. The chart also accepts a CSV
                # string in some legacy paths; keep tolerant.
                type = types.nullOr (types.either types.str (types.listOf types.str));
                default = null;
                description = "List of usernames for client communications when SASL is enabled";
              };

              passwords = mkOption {
                # Bitnami: comma-separated string, but can also be array.
                type = types.nullOr (types.either types.str (types.listOf types.str));
                default = null;
                description = "Comma-separated list of passwords for client communications";
              };
            };
          });
        };

        existingSecret = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Name of existing secret containing credentials for client.users, interbroker.user and controller.user";
        };
      };
    });
  };
}
