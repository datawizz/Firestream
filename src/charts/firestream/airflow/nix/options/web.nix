# Airflow chart options: `web.*` (Airflow API server / web UI).
#
# Composition (list-form submodule): the shared component base type
# (mkComponentType, ~43 leaves) merged with web-only extra keys that are not
# part of the generic component shape.
{ lib, chartTypes, ... }:

let
  t = chartTypes;
  inherit (lib) mkOption types;
in {
  options.airflow.web = mkOption {
    default = { };
    description = "Airflow web (API server) parameters";
    # mkComponentType returns a submodule *type*; splice its underlying modules
    # (via getSubModules) together with the web-only extra option module.
    type = types.submodule ((t.mkComponentType { componentName = "web"; }).getSubModules ++ [
      {
        options = {
          baseUrl = mkOption {
            type = types.nullOr types.str;
            default = null;
            description = "Set the base url for the Airflow webserver";
          };

          configuration = mkOption {
            type = types.nullOr types.str;
            default = null;
            description = "Specify content for webserver_config.py";
          };

          extraConfiguration = mkOption {
            type = types.nullOr types.str;
            default = null;
            description = "Add extra content to the webserver_config.py file";
          };

          existingConfigmap = mkOption {
            type = types.nullOr types.str;
            default = null;
            description = "Name of an existing ConfigMap with the webserver_config.py";
          };

          tls = mkOption {
            type = types.nullOr t.tlsType;
            default = null;
            description = "TLS configuration for the Airflow web server";
          };
        };
      }
    ]);
  };
}
