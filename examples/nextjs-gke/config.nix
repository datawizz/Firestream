# ---------------------------------------------------------------------------
# YOUR deployment configuration — the single source of project-specific values
# for both the Nix chart override (firestream/nextjs-overrides.nix) and the
# deploy scripts.
#
# After `pulumi up` in ./pulumi, copy the stack outputs (clusterName, region,
# arHost, arRepo) into the matching fields below.
#
# The custom app is NOT here; it's the ./app folder, baked into the image via
# firestream/nextjs-image-overrides.nix.
# ---------------------------------------------------------------------------
{
  # --- Google Cloud project + location -------------------------------------
  projectId = "my-gcp-project";
  region = "us-central1";

  # --- GKE cluster (created by ./pulumi) + target namespace ----------------
  clusterName = "firestream-nextjs";
  namespace = "nextjs";

  # --- Artifact Registry (created by ./pulumi) -----------------------------
  arHost = "us-central1-docker.pkg.dev"; # always "<region>-docker.pkg.dev"
  arRepo = "my-gcp-project/firestream"; # "<projectId>/<repositoryId>"
  imageTag = "latest"; # use an immutable tag (git SHA) in real pipelines

  # --- Ingress hostname (point DNS at the GKE ingress IP once created) -----
  domain = "nextjs.example.com";

  # --- GKE StorageClass ----------------------------------------------------
  # Autopilot ships "standard-rwo" (balanced PD) and "premium-rwo" (SSD).
  storageClass = "standard-rwo";

  # --- Kubernetes Secret name ----------------------------------------------
  # NOT created by Helm. Cloud Build (scripts/sync-secrets.sh) materialises it
  # from GCP Secret Manager at deploy time; the chart references it by name.
  # keys: password (app user), postgres-password (superuser).
  dbSecretName = "nextjs-db-credentials";
}
