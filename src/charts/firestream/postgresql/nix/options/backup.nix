# PostgreSQL chart options: `backup.*` (logical-dump CronJob).
#
# The Bitnami `backup.cronjob.*` block is very deep and bespoke; we expose
# the on/off switch and the cronjob attrset as a free-form passthrough so users
# can pass arbitrary cron tuning without us re-declaring it here.
#
# In addition to the Bitnami passthrough, Firestream adds a typed `backup.s3`
# block. It is the END-USER OVERRIDE SURFACE for the backup target: the chart
# flake-module (nix/flake-modules/charts/postgresql.nix) reads `backup.s3` and
# from it builds the CronJob's `extraEnvVars` + streaming `command`. Defaults
# point at the in-cluster SeaweedFS dev store; override the fields to target a
# real cloud S3 bucket (set `existingSecret` for creds, clear `endpoint`, and
# switch `addressingStyle` to "auto"/"virtual"). `backup.s3` is inert to the
# upstream Bitnami templates — it is purely a Firestream input.
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

        schedule = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Cron schedule for the backup CronJob (default '@daily' applied by the flake-module).";
        };

        cronjob = mkOption {
          type = types.nullOr (types.attrsOf types.anything);
          default = null;
          description = "CronJob configuration (free-form passthrough; see values.yaml)";
        };

        # Firestream end-user override surface for the backup destination.
        s3 = mkOption {
          description = ''
            S3 backup target. Consumed by the chart flake-module to build the
            backup CronJob env + command. Defaults to the in-cluster SeaweedFS
            store. To target real cloud S3: set `existingSecret`, clear
            `endpoint` (null), and set `addressingStyle = "auto"`.
          '';
          default = {};
          type = types.submodule {
            options = {
              endpoint = mkOption {
                type = types.nullOr types.str;
                default = "http://seaweedfs-all-in-one.seaweedfs.svc.cluster.local:8333";
                description = ''
                  S3 endpoint URL. Passed to `aws --endpoint-url` only when
                  non-null/non-empty. Set to null for AWS S3 (the SDK uses the
                  default regional endpoint).
                '';
              };
              bucket = mkOption {
                type = types.str;
                default = "firestream";
                description = "Destination bucket for dumps.";
              };
              prefix = mkOption {
                type = types.str;
                default = "pg-backups";
                description = "Key prefix within the bucket.";
              };
              region = mkOption {
                type = types.str;
                default = "us-east-1";
                description = "AWS_DEFAULT_REGION for the aws CLI.";
              };
              addressingStyle = mkOption {
                type = types.nullOr (types.enum [ "path" "virtual" "auto" ]);
                default = "path";
                description = ''
                  S3 addressing style. "path" for SeaweedFS/MinIO; "auto" or
                  "virtual" for AWS S3. Applied via `aws configure set
                  default.s3.addressing_style` when non-null.
                '';
              };
              existingSecret = mkOption {
                type = types.nullOr types.str;
                default = null;
                description = ''
                  Name of a Secret IN THE RELEASE NAMESPACE holding the S3
                  credentials. When set, creds are injected via secretKeyRef
                  (the secure path for real cloud S3) instead of the literal
                  accessKeyId/secretAccessKey below.
                '';
              };
              accessKeyIdKey = mkOption {
                type = types.str;
                default = "AWS_ACCESS_KEY_ID";
                description = "Key within existingSecret holding the access key id.";
              };
              secretAccessKeyKey = mkOption {
                type = types.str;
                default = "AWS_SECRET_ACCESS_KEY";
                description = "Key within existingSecret holding the secret access key.";
              };
              accessKeyId = mkOption {
                type = types.nullOr types.str;
                default = "firestream";
                description = "Literal access key id (dev only). Ignored when existingSecret is set.";
              };
              secretAccessKey = mkOption {
                type = types.nullOr types.str;
                default = "firestream-secret";
                description = "Literal secret access key (dev only). Ignored when existingSecret is set.";
              };
            };
          };
        };
      };
    });
  };
}
