# Odoo chart options: custom post-init scripts and env-var injection
# (FLAT — top-level keys).
#
# Odoo does NOT have a separate init Job (initialisation runs inside
# the main container on first boot). `customPostInitScripts` is a
# map of filename -> script content; the chart renders it into a
# ConfigMap (`postinit-configmap.yaml`) that the main pod mounts at
# `/docker-entrypoint-init.d/`. Supported formats are `.sh`, `.sql`,
# `.php`; the scripts are executed exclusively during the 1st boot.
#
# `extraEnvVars` / `extraEnvVarsCM` / `extraEnvVarsSecret` add
# environment variables to the main odoo container — they live at
# the top level in values.yaml, NOT nested under `init.*` or `app.*`.
{ lib, ... }:

let
  inherit (lib) mkOption types;
in {
  options.odoo = {
    customPostInitScripts = mkOption {
      type = types.nullOr (types.attrsOf types.anything);
      default = null;
      description = ''
        Custom post-init.d user scripts (map filename -> contents).
        Supported formats: .sh, .sql, .php. Executed exclusively during
        the 1st boot of the container.
      '';
    };

    extraEnvVars = mkOption {
      type = types.nullOr (types.listOf types.attrs);
      default = null;
      description = "Array with extra environment variables to add to the Odoo container";
    };

    extraEnvVarsCM = mkOption {
      type = types.nullOr types.str;
      default = null;
      description = "Name of existing ConfigMap containing extra env vars";
    };

    extraEnvVarsSecret = mkOption {
      type = types.nullOr types.str;
      default = null;
      description = "Name of existing Secret containing extra env vars";
    };
  };
}
