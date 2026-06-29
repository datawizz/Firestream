# ---------------------------------------------------------------------------
# The "make it your own" surface for GKE.
#
# Passed as the LAST module to `firestream.lib.<sys>.charts.airflow.eval`, so
# every value deep-merges on top of the Firestream defaults and the
# image-injection overlay. Intentionally sparse.
#
# Contrast with ../airflow-k3s: here we REPOINT the image refs at Artifact
# Registry (Cloud Build pushes them there) and read credentials from K8s
# Secrets that Cloud Build materialises from Secret Manager (`existingSecret`),
# so no passwords land in git or values.yaml.
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

    # --- Admin + crypto keys from a K8s Secret (Cloud Build → Secret Manager).
    #     Keys: airflow-password, airflow-fernet-key, airflow-secret-key,
    #     airflow-jwt-secret-key. ----------------------------------------------
    auth = {
      username = "admin";
      existingSecret = cfg.airflowSecretName;
    };

    # --- Airflow image from Artifact Registry (Cloud Build pushes it). This
    #     overlaps the slot Firestream injects by default; an explicit user
    #     value wins. ----------------------------------------------------------
    image = {
      registry = cfg.arHost;
      repository = "${cfg.arRepo}/firestream-airflow";
      tag = cfg.imageTag;
    };

    # --- Bundled PostgreSQL subchart (metadata DB) -------------------------
    postgresql = {
      enabled = true;
      architecture = "standalone";
      # Bitnami pg reads `password` (user) + `postgres-password` (admin).
      auth.existingSecret = cfg.dbSecretName;
      image = {
        registry = cfg.arHost;
        repository = "${cfg.arRepo}/firestream-postgresql";
        tag = cfg.imageTag;
      };
      primary.persistence = {
        enabled = true;
        storageClass = cfg.storageClass;
        size = "8Gi";
      };
    };

    # --- Bundled Redis subchart (Celery broker) ----------------------------
    redis = {
      enabled = true;
      architecture = "standalone";
      auth.existingSecret = cfg.redisSecretName; # key: redis-password
      image = {
        registry = cfg.arHost;
        repository = "${cfg.arRepo}/firestream-redis";
        tag = cfg.imageTag;
      };
      master.persistence = {
        enabled = true;
        storageClass = cfg.storageClass;
        size = "8Gi";
      };
    };

    # --- Storage default for component PVCs (worker/triggerer) -------------
    global.defaultStorageClass = cfg.storageClass;

    # --- Expose via a GKE L7 ingress (GCE). On Autopilot/VPC-native clusters
    #     a ClusterIP service is fronted by container-native (NEG) LB. ---------
    service.type = "ClusterIP";
    ingress = {
      enabled = true;
      ingressClassName = "gce";
      hostname = cfg.domain;
      path = "/";
      pathType = "Prefix";
    };
  };
}
