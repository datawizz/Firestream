# ---------------------------------------------------------------------------
# YOUR deployment configuration. This is the file you edit to "make it your
# own" — it is the single source of project-specific values for both the Nix
# chart override (firestream/odoo-overrides.nix) and the deploy scripts.
#
# After `pulumi up` in ./pulumi, copy the stack outputs (clusterName, region,
# arHost, arRepo) into the matching fields below.
# ---------------------------------------------------------------------------
{
  # --- Google Cloud project + location -------------------------------------
  projectId = "my-gcp-project";
  region = "us-central1";

  # --- GKE cluster (created by ./pulumi) + target namespace ----------------
  clusterName = "firestream-odoo";
  namespace = "odoo";

  # --- Artifact Registry (created by ./pulumi) -----------------------------
  # arHost is always "<region>-docker.pkg.dev".
  arHost = "us-central1-docker.pkg.dev";
  # arRepo is "<projectId>/<repositoryId>".
  arRepo = "my-gcp-project/firestream";
  # Tag the firestream-* images are pushed under by Cloud Build. Use an
  # immutable tag (e.g. the git SHA) in real pipelines.
  imageTag = "latest";

  # --- Ingress hostname (point DNS at the GKE ingress IP once created) -----
  domain = "odoo.example.com";

  # --- GKE StorageClass ----------------------------------------------------
  # Autopilot ships "standard-rwo" (balanced PD) and "premium-rwo" (SSD).
  storageClass = "standard-rwo";

  # --- Build-time vendored Odoo addons -------------------------------------
  # Third-party addon repos fetched at IMAGE BUILD time and baked into
  # /opt/odoo/vendor-addons (wired into addons_path). This is the headline
  # "inject your preferences into Firestream's Nix system and get a custom image"
  # capability — consumed by firestream/odoo-image-overrides.nix via
  # firestream.lib.<sys>.images.odoo.eval. Pin `rev` to a commit (not a branch)
  # for reproducibility; `hash` is the fetchFromGitHub SRI hash.
  vendoredAddons = [
    {
      name = "oca-server-tools";
      owner = "OCA";
      repo = "server-tools";
      rev = "13acf53ba602809d64efdc634dd87dd3439e1564"; # 18.0 @ 2026-06-27
      hash = "sha256-dAntlcVk42jme8Vo4ds7sK1OCjyKyZw2n/mKyO+/j3o=";
      # Vendor a single dependency-light module for the demo; drop `modules` to
      # auto-discover and bake every module in the repo.
      modules = [ "base_fontawesome" ];
    }
  ];

  # --- Kubernetes Secret names ---------------------------------------------
  # These Secrets are NOT created by Helm. Cloud Build (scripts/sync-secrets.sh)
  # materialises them from GCP Secret Manager at deploy time; the chart only
  # ever references them by name.
  odooSecretName = "odoo-credentials"; # must contain key: odoo-password
  dbSecretName = "odoo-db-credentials"; # must contain keys: password, postgres-password
  # smtpSecretName = "odoo-smtp-credentials"; # must contain key: smtp-password
}
