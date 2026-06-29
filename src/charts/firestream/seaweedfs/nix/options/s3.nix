# SeaweedFS chart options: `s3.*` (top-level S3 gateway settings).
#
# The all-in-one pod reads several values from BOTH `allInOne.s3.*` and the
# top-level `s3.*` (the latter as the inheritance fallback). Critically, the
# s3-secret.yaml template only reads `.Values.s3.enableAuth` and
# `.Values.s3.credentials.admin.{accessKey,secretKey}` — so the credential
# Secret (`<fullname>-s3-secret`) is generated/populated from THIS block.
#
# Firestream defaults: S3 on, auth on, deterministic dev credentials
# (firestream / firestream-secret) so Phase E consumers can read them.
{ lib, ... }:

let
  inherit (lib) mkOption types;
in {
  options.seaweedfs.s3 = mkOption {
    default = null;
    description = "Top-level S3 gateway settings (also the inheritance fallback for allInOne.s3).";
    type = types.nullOr (types.submodule {
      freeformType = types.attrsOf types.anything;
      options = {
        enabled = mkOption {
          type = types.nullOr types.bool;
          default = null;
          description = "Enable the S3 gateway.";
        };
        port = mkOption {
          type = types.nullOr types.int;
          default = null;
          description = "S3 gateway port.";
        };
        enableAuth = mkOption {
          type = types.nullOr types.bool;
          default = null;
          description = "Enable S3 user/permission auth. Drives generation of the s3 credential Secret.";
        };
        domainName = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Suffix of the host name ({bucket}.{domainName}).";
        };
        credentials = mkOption {
          default = null;
          description = "Explicit S3 credentials written into the generated s3 secret.";
          type = types.nullOr (types.submodule {
            freeformType = types.attrsOf types.anything;
            options = {
              admin = mkOption {
                default = null;
                description = "Admin (full Read/Write) credentials.";
                type = types.nullOr (types.submodule {
                  freeformType = types.attrsOf types.anything;
                  options = {
                    accessKey = mkOption {
                      type = types.nullOr types.str;
                      default = null;
                      description = "Admin access key id.";
                    };
                    secretKey = mkOption {
                      type = types.nullOr types.str;
                      default = null;
                      description = "Admin secret access key.";
                    };
                  };
                });
              };
            };
          });
        };
      };
    });
  };
}
