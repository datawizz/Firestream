# Odoo chart options: authentication / credentials (FLAT — top-level keys).
#
# Bitnami odoo's auth shape is FLAT in values.yaml — there is NO `auth.*`
# section. The web-UI admin credentials, SMTP credentials, and the
# existingSecret pointer are all top-level keys. We mirror that exactly
# so the generated values.yaml is a faithful sparse override.
#
# secrets.yaml writes:
#   odoo-password    <- providedValues = list "odooPassword", randomised
#                       per render via common.secrets.passwords.manage
#                       (length 10) when both `odooPassword` and
#                       `existingSecret` are empty.
#   smtp-password    <- base64-encoded from `smtpPassword` (NOT random)
#                       only when `smtpPassword` is set and
#                       `smtpExistingSecret` is empty.
#
# `existingSecret` skips the in-chart `odoo-password` secret; the key
# `odoo-password` is then read from the external secret.
# `smtpExistingSecret` is the analogous override for `smtp-password`.
{ lib, ... }:

let
  inherit (lib) mkOption types;
in {
  options.odoo = {
    # ----- Odoo admin user credentials -----
    odooEmail = mkOption {
      type = types.nullOr types.str;
      default = null;
      description = "Odoo admin user email";
    };

    odooPassword = mkOption {
      type = types.nullOr types.str;
      default = null;
      description = "Odoo admin user password (defaults to a random 10-character alphanumeric string if empty)";
    };

    odooSkipInstall = mkOption {
      type = types.nullOr types.bool;
      default = null;
      description = "Skip Odoo installation wizard";
    };

    odooDatabaseFilter = mkOption {
      type = types.nullOr types.str;
      default = null;
      description = "Filter odoo database by using a regex";
    };

    loadDemoData = mkOption {
      type = types.nullOr types.bool;
      default = null;
      description = "Whether to load demo data for all modules during initialisation";
    };

    # ----- SMTP credentials -----
    smtpHost = mkOption {
      type = types.nullOr types.str;
      default = null;
      description = "SMTP server host";
    };

    smtpPort = mkOption {
      type = types.nullOr (types.either types.str types.int);
      default = null;
      description = "SMTP server port";
    };

    smtpUser = mkOption {
      type = types.nullOr types.str;
      default = null;
      description = "SMTP username";
    };

    smtpPassword = mkOption {
      type = types.nullOr types.str;
      default = null;
      description = "SMTP user password";
    };

    smtpProtocol = mkOption {
      type = types.nullOr types.str;
      default = null;
      description = "SMTP protocol";
    };

    # ----- existingSecret pointers -----
    existingSecret = mkOption {
      type = types.nullOr types.str;
      default = null;
      description = "Name of existing secret containing Odoo credentials (must contain key `odoo-password`)";
    };

    smtpExistingSecret = mkOption {
      type = types.nullOr types.str;
      default = null;
      description = "Name of an existing secret with SMTP credentials (must contain key `smtp-password`)";
    };

    allowEmptyPassword = mkOption {
      type = types.nullOr types.bool;
      default = null;
      description = "Allow the container to be started with blank passwords";
    };
  };
}
