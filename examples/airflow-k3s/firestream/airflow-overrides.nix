# ---------------------------------------------------------------------------
# The "make it your own" surface for LOCAL k3s/k3d.
#
# Passed as the LAST module to `firestream.lib.<sys>.charts.airflow.eval`, so
# every value deep-merges on top of the Firestream defaults and the
# image-injection overlay. Intentionally sparse.
#
# Contrast with ../airflow-gke:
#   - We DO NOT repoint image refs — keep the default-injected
#     firestream-airflow:3.0.3 / firestream-postgresql:17 / firestream-redis:8
#     so the cluster runs the images scripts/deploy-local.sh side-loads
#     (including our custom airflow image with the baked DAG).
#   - Credentials are INLINE (from config.nix), not `existingSecret`.
#   - CeleryExecutor (chart default) → postgres + redis + scheduler + worker +
#     triggerer + dag-processor + api-server.
#
# Option names mirror src/charts/firestream/airflow/values.yaml.
# ---------------------------------------------------------------------------
{ ... }:

let
  cfg = import ../config.nix;
in
{
  config.airflow = {
    executor = "CeleryExecutor";

    # --- Admin login + crypto keys (inline). With these set and no
    #     existingSecret, the chart writes its own airflow Secret. ------------
    auth = {
      username = "admin";
      password = cfg.airflowPassword;
      fernetKey = cfg.fernetKey;
      secretKey = cfg.secretKey;
      jwtSecretKey = cfg.jwtSecretKey;
    };

    # --- Bundled PostgreSQL subchart (Airflow metadata DB) ------------------
    postgresql = {
      enabled = true;
      architecture = "standalone";
      auth = {
        # `password` (app user) is typed; `postgresPassword` (superuser) flows
        # through the freeform subchart passthrough. Both inline → idempotent
        # `helm upgrade`.
        password = cfg.dbPassword;
        postgresPassword = cfg.dbPostgresPassword;
      };
    };

    # --- Bundled Redis subchart (Celery broker) ----------------------------
    redis = {
      enabled = true;
      architecture = "standalone";
      auth.password = cfg.redisPassword;
    };

    # --- Storage: bind every PVC to the local-path provisioner -------------
    global.defaultStorageClass = cfg.storageClass;

    # service.type defaults to ClusterIP and ingress.enabled to false already,
    # so access is via `kubectl port-forward` (see the Makefile / README).
  };
}
