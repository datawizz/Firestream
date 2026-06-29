# ---------------------------------------------------------------------------
# The "make it your own" surface.
#
# This is a NixOS module passed as the LAST module to
# `firestream.lib.<sys>.charts.odoo.eval`, so every value here deep-merges on
# top of (and wins over) the Firestream defaults and the image-injection
# overlay. It is intentionally a *sparse* override — a strict subset of the
# chart's value surface (see docs/firestream-supported-app.md §9). Anything you
# don't set keeps the Firestream default.
#
# Option names mirror src/charts/firestream/odoo/values.yaml exactly:
#   - admin/SMTP/existingSecret keys are FLAT (top-level), not under `auth.*`
#   - the database is the bundled `postgresql` subchart
# ---------------------------------------------------------------------------
{ config, lib, ... }:

let
  cfg = import ../config.nix;
in
{
  config.odoo = {
    # --- Admin credentials live in a K8s Secret that Cloud Build creates from
    #     GCP Secret Manager. With existingSecret set, odooPassword is ignored
    #     and the chart reads key `odoo-password` from this Secret. ----------
    existingSecret = cfg.odooSecretName;

    # --- Database: bundled in-cluster PostgreSQL subchart (no Cloud SQL) -----
    postgresql = {
      enabled = true;
      architecture = "standalone";
      auth = {
        username = "bn_odoo";
        database = "bitnami_odoo";
        # Bitnami postgres reads `password` (user) + `postgres-password`
        # (admin) from this Secret. Odoo's deployment references the same
        # Secret for its DB connection.
        existingSecret = cfg.dbSecretName;
      };
      # Pull the firestream-postgresql image from Artifact Registry.
      image = {
        registry = cfg.arHost;
        repository = "${cfg.arRepo}/firestream-postgresql";
        tag = cfg.imageTag;
      };
      primary.persistence = {
        enabled = true;
        storageClass = cfg.storageClass;
        size = "20Gi";
      };
    };

    # --- Odoo image from Artifact Registry (Cloud Build pushes it there).
    #     This overlaps the slot Firestream injects by default; per
    #     inject-container-images.nix, an explicit user value wins. -----------
    image = {
      registry = cfg.arHost;
      repository = "${cfg.arRepo}/firestream-odoo";
      tag = cfg.imageTag;
    };

    # --- Odoo filestore persistence on a GKE StorageClass -------------------
    persistence = {
      enabled = true;
      storageClass = cfg.storageClass;
      size = "20Gi";
    };

    # --- Expose via a GKE L7 ingress (GCE). On Autopilot/VPC-native clusters
    #     a ClusterIP service is fronted by container-native (NEG) load
    #     balancing automatically. ---------------------------------------------
    service.type = "ClusterIP";
    ingress = {
      enabled = true;
      ingressClassName = "gce";
      hostname = cfg.domain;
      path = "/";
      pathType = "Prefix";
    };

    replicaCount = 1;

    # --- Optional: outbound email. Create cfg.smtpSecretName (key
    #     `smtp-password`) and uncomment. ------------------------------------
    # smtpHost = "smtp.example.com";
    # smtpPort = 587;
    # smtpUser = "odoo@example.com";
    # smtpProtocol = "tls";
    # smtpExistingSecret = cfg.smtpSecretName;
  };
}
