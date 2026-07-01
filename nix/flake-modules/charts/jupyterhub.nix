# JupyterHub chart flake-module (Phase 6b - Agent G5; Phase B image injection)
# Copyright Firestream. MIT License.
#
# Wires the JupyterHub Helm chart through the options-driven evalChart
# entrypoint and contributes:
#   - packages.jupyterhub-chart       (deployable chart bundle; builds on darwin)
#   - firestreamCharts.jupyterhub     (full evaluated chart result, for aggregate)
#   - firestreamChartImages.jupyterhub (consumer override API, for flake.lib.charts)
#
# Mirrors nix/flake-modules/charts/spark.nix. No isLinux gate — `helm template`
# runs in the build sandbox on every platform.
#
# The Bitnami JupyterHub chart bundles `common` and `postgresql` as vendored
# subcharts (Chart.yaml lists both). vendor-subcharts.nix consumes the
# in-repo Bitnami fork at src/charts/bitnami/.
#
# Phase B image injection: the Bitnami chart ships five image slots
#   - hub                   (bitnami/jupyterhub)
#   - proxy                 (bitnami/configurable-http-proxy)
#   - singleuser            (bitnami/jupyter-base-notebook)
#   - auxiliaryImage        (bitnami/os-shell — init sidecar)
#   - postgresql (subchart) (bitnami/postgresql)
#
# Firestream ships ONE `firestream-jupyterhub:5.3.0` image (the JupyterHub
# container is a single python-workspace artefact — see
# nix/flake-modules/containers/jupyterhub.nix). We point hub, proxy, and
# singleuser at it. That image carries the full JupyterHub python environment
# which contains the configurable-http-proxy CLI and jupyter-base-notebook
# binaries — so the same image can play all three roles (k8s pods only need
# the relevant entrypoint, which is pulled from the chart's pod spec, not
# from the image's ENTRYPOINT). The auxiliary slot points at
# `firestream-os-shell:1` (Phase A deliverable). The postgresql subchart slot
# points at `firestream-postgresql:17`.
#
# `global.security.allowInsecureImages` is flipped to bypass Bitnami's
# NOTES.txt whitelist.
{ ... }: {
  perSystem = { pkgs, lib, config, evalChart, baseChart, ... }:
    let
      chartSrc = ../../../src/charts/firestream/jupyterhub;
      optionsPath = chartSrc + "/nix/default.nix";

      # Subcharts vendored from the in-repo Bitnami fork. JupyterHub's
      # Chart.yaml lists `postgresql` (16.x.x) and `common` (2.x.x). The
      # engine's vendor-subcharts.nix handles the nested copy.
      subcharts = [
        { name = "common"; }
        { name = "postgresql"; }
      ];

      jupyterhubImg =
        let
          imgEval = config.firestreamImages.jupyterhub.eval (_: {});
          imgCfg = imgEval.config.jupyterhub.image;
        in {
          registry = imgCfg.registry;
          repository = imgCfg.repository;
          tag = imgEval.imageTag;
        };

      osShellImg =
        let
          imgEval = config.firestreamImages."os-shell".eval (_: {});
          imgCfg = imgEval.config."os-shell".image;
        in {
          registry = imgCfg.registry;
          repository = imgCfg.repository;
          tag = imgEval.imageTag;
        };

      pgImg =
        let
          imgEval = config.firestreamImages.postgresql.eval (_: {});
          imgCfg = imgEval.config.postgresql.image;
        in {
          registry = imgCfg.registry;
          repository = imgCfg.repository;
          tag = imgEval.imageTag;
        };

      # Phase F: firestream-jupyterhub bakes JUPYTERHUB_* path env vars
      # under /opt/jupyterhub (see src/containers/firestream/jupyterhub/
      # options.nix). The Bitnami chart mounts:
      #   - /etc/jupyterhub/jupyterhub_config.py (configmap subPath)
      #   - /usr/local/etc/jupyterhub/secret/    (Secret)
      #   - /tmp                                  (emptyDir)
      # No PVC for hub data; readOnlyRootFilesystem is enforced on hub +
      # proxy. The hub container runs `jupyterhub --config /etc/jupyterhub/
      # jupyterhub_config.py --upgrade-db` directly (bypassing the firestream
      # entrypoint), so most path env vars are advisory rather than load-
      # bearing — but the chart-mounted config path MUST be reflected so any
      # logging or transient state lands on /tmp (the only writable mount).
      firestreamPathOverrides = [
        { name = "JUPYTERHUB_CONF_FILE"; value = "/etc/jupyterhub/jupyterhub_config.py"; }
        { name = "JUPYTERHUB_CONF_DIR";  value = "/etc/jupyterhub"; }
        { name = "JUPYTERHUB_TMP_DIR";   value = "/tmp"; }
        { name = "JUPYTERHUB_PID_FILE";  value = "/tmp/jupyterhub.pid"; }
        { name = "JUPYTERHUB_LOGS_DIR";  value = "/tmp/logs"; }
        { name = "JUPYTERHUB_LOG_FILE";  value = "/tmp/logs/jupyterhub.log"; }
        { name = "JUPYTERHUB_DATA_DIR";  value = "/tmp/data"; }
        { name = "JUPYTERHUB_VOLUME_DIR"; value = "/tmp"; }
      ];

      # Phase 1 path-de-branding: the postgresql subchart templates + values now
      # mount/reference firestream paths directly, matching the firestream-postgresql
      # container's baked *_DIR vars. No subchart pg path-override extraEnvVars needed.
      imageInjectionModule = { ... }: {
        config.jupyterhub._meta.containerRefs = {
          hub = {
            inherit (jupyterhubImg) registry repository tag;
            componentPath = [ "hub" "image" ];
          };
          proxy = {
            inherit (jupyterhubImg) registry repository tag;
            componentPath = [ "proxy" "image" ];
          };
          singleuser = {
            inherit (jupyterhubImg) registry repository tag;
            componentPath = [ "singleuser" "image" ];
          };
          auxiliaryImage = {
            inherit (osShellImg) registry repository tag;
            componentPath = [ "auxiliaryImage" ];
          };
          postgresql = {
            inherit (pgImg) registry repository tag;
            componentPath = [ "postgresql" "image" ];
          };
        };
        config.jupyterhub.global.security.allowInsecureImages = true;
        # The bundled Bitnami postgresql subchart defaults to preloading the
        # `pgaudit` extension, which the firestream-postgresql image does not
        # ship (server start fails with `could not access file "pgaudit"`).
        # Mirror the standalone postgresql overlay and clear it for the subchart.
        config.jupyterhub.postgresql.postgresqlSharedPreloadLibraries = "";
        # Per-component extraEnvVars. singleuser pods are user-spawned but
        # also benefit from consistent env (config_files_path inheritance).
        config.jupyterhub.hub.extraEnvVars = firestreamPathOverrides;
        config.jupyterhub.proxy.extraEnvVars = firestreamPathOverrides;
        config.jupyterhub.singleuser.extraEnvVars = firestreamPathOverrides;

        # The proxy component runs the Bitnami configurable-http-proxy. Upstream
        # ships a dedicated `bitnami/configurable-http-proxy` image whose
        # entrypoint IS the CHP binary; Firestream points every jupyterhub
        # component at the single firestream-jupyterhub image, whose entrypoint
        # is the firestream lifecycle wrapper (validate/activate/config-hash).
        # For the proxy that wrapper has no jupyterhub_config.py to hash and the
        # CHP flags it receives as args are not a runnable command, so it crashes.
        # The hub already overrides command to `jupyterhub`; mirror that for the
        # proxy by running the CHP binary directly (bypassing the wrapper). The
        # firestream-jupyterhub image bundles `configurable-http-proxy` on PATH.
        config.jupyterhub.proxy.command = [ "configurable-http-proxy" ];
      };

      c = evalChart {
        name = "jupyterhub";
        inherit chartSrc subcharts;
        modules = [ optionsPath imageInjectionModule ];
      };
    in
    {
      packages.jupyterhub-chart = c.chartBundle;

      # Base (un-overlaid) chart: chart's OWN native defaults, no Firestream
      # values overlay / no image injection. Renders `bitnami/jupyterhub`.
      packages.jupyterhub-base-chart = baseChart {
        name = "jupyterhub";
        inherit chartSrc subcharts;
      };

      # Registry: full evaluated chart result (used by aggregate.nix / flake.lib).
      firestreamCharts.jupyterhub = c // { baseChart = config.packages.jupyterhub-base-chart; };

      # Registry: consumer override API exposed via flake.lib.<sys>.charts.jupyterhub.
      firestreamChartImages.jupyterhub = {
        chartBundle = c.chartBundle;
        baseChart = config.packages.jupyterhub-base-chart;
        render = c.render;
        eval = userMod: evalChart {
          name = "jupyterhub";
          inherit chartSrc subcharts;
          modules = [ optionsPath imageInjectionModule userMod ];
        };
        options = c.options;
      };
    };
}
