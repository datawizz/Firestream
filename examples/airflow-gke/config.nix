# ---------------------------------------------------------------------------
# YOUR deployment configuration — the single source of project-specific values
# for both the Nix chart override (firestream/airflow-overrides.nix) and the
# deploy scripts.
#
# After `pulumi up` in ./pulumi, copy the stack outputs (clusterName, region,
# arHost, arRepo) into the matching fields below.
#
# The custom DAG is NOT here; it's the ./dags folder, baked into the image via
# firestream/airflow-image-overrides.nix.
# ---------------------------------------------------------------------------
{
  # --- Google Cloud project + location -------------------------------------
  projectId = "my-gcp-project";
  region = "us-central1";

  # --- GKE cluster (created by ./pulumi) + target namespace ----------------
  clusterName = "firestream-airflow";
  namespace = "airflow";

  # --- Artifact Registry (created by ./pulumi) -----------------------------
  arHost = "us-central1-docker.pkg.dev"; # always "<region>-docker.pkg.dev"
  arRepo = "my-gcp-project/firestream"; # "<projectId>/<repositoryId>"
  imageTag = "latest"; # use an immutable tag (git SHA) in real pipelines

  # --- Ingress hostname (point DNS at the GKE ingress IP once created) -----
  domain = "airflow.example.com";

  # --- GKE StorageClass ----------------------------------------------------
  # Autopilot ships "standard-rwo" (balanced PD) and "premium-rwo" (SSD).
  storageClass = "standard-rwo";

  # --- Kubernetes Secret names ---------------------------------------------
  # NOT created by Helm. Cloud Build (scripts/sync-secrets.sh) materialises them
  # from GCP Secret Manager at deploy time; the chart references them by name.
  airflowSecretName = "airflow-credentials"; # keys: airflow-password, airflow-fernet-key, airflow-secret-key, airflow-jwt-secret-key
  dbSecretName = "airflow-db-credentials"; # keys: password, postgres-password
  redisSecretName = "airflow-redis-credentials"; # key: redis-password
}
