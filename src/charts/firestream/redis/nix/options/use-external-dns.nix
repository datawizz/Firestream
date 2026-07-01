# Redis chart options: `useExternalDNS.*` (external-dns integration).
{ lib, ... }:

let
  inherit (lib) mkOption types;
in {
  options.redis.useExternalDNS = mkOption {
    default = null;
    description = "External DNS integration (requires a working installation of `external-dns`)";
    type = types.nullOr (types.submodule {
      options = {
        enabled = mkOption {
          type = types.nullOr types.bool;
          default = null;
          description = "Enable various syntax that would enable external-dns to work";
        };

        suffix = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "The DNS suffix utilized when `external-dns` is enabled";
        };

        annotationKey = mkOption {
          # Bitnami: can be a string ("external-dns.alpha.kubernetes.io/") or
          # the literal `false` to disable annotations entirely.
          type = types.nullOr (types.either types.str types.bool);
          default = null;
          description = "The annotation key utilized when `external-dns` is enabled (set to `false` to disable annotations)";
        };

        additionalAnnotations = mkOption {
          type = types.nullOr (types.attrsOf types.anything);
          default = null;
          description = "Extra annotations to be utilized when `external-dns` is enabled";
        };
      };
    });
  };
}
