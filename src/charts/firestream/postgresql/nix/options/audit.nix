# PostgreSQL chart options: `audit.*` (audit logging configuration).
{ lib, ... }:

let
  inherit (lib) mkOption types;
in {
  options.postgresql.audit = mkOption {
    default = null;
    description = "PostgreSQL audit logging configuration";
    type = types.nullOr (types.submodule {
      options = {
        logHostname = mkOption {
          type = types.nullOr types.bool;
          default = null;
          description = "Log client hostnames";
        };

        logConnections = mkOption {
          type = types.nullOr types.bool;
          default = null;
          description = "Add client log-in operations to the log file";
        };

        logDisconnections = mkOption {
          type = types.nullOr types.bool;
          default = null;
          description = "Add client log-outs operations to the log file";
        };

        pgAuditLog = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Add operations to log using the pgAudit extension";
        };

        pgAuditLogCatalog = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Log catalog using pgAudit";
        };

        clientMinMessages = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Message log level to share with the user";
        };

        logLinePrefix = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Template for log line prefix";
        };

        logTimezone = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Timezone for the log timestamps";
        };
      };
    });
  };
}
