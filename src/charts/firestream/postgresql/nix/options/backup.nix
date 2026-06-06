# PostgreSQL chart options: `backup.*` (logical-dump CronJob).
#
# The Bitnami `backup.cronjob.*` block is very deep and bespoke; we expose
# the on/off switch and the cronjob attrset as a free-form passthrough so users
# can pass arbitrary cron tuning without us re-declaring it here.
{ lib, ... }:

let
  inherit (lib) mkOption types;
in {
  options.postgresql.backup = mkOption {
    default = null;
    description = "Logical-dump backup CronJob configuration";
    type = types.nullOr (types.submodule {
      freeformType = types.attrsOf types.anything;

      options = {
        enabled = mkOption {
          type = types.nullOr types.bool;
          default = null;
          description = "Enable the logical dump of the database";
        };

        cronjob = mkOption {
          type = types.nullOr (types.attrsOf types.anything);
          default = null;
          description = "CronJob configuration (free-form passthrough; see values.yaml)";
        };
      };
    });
  };
}
