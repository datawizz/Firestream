# ---------------------------------------------------------------------------
# The "make it your own" surface for LOCAL k3s/k3d.
#
# Passed as the LAST module to `firestream.lib.<sys>.charts.odoo.eval`, so every
# value here deep-merges on top of (and wins over) the Firestream defaults and
# the image-injection overlay. Intentionally *sparse* — a strict subset of the
# chart's value surface (see docs/firestream-supported-app.md §9).
#
# Contrast with ../odoo-gke/firestream/odoo-overrides.nix:
#   - We DO NOT repoint `image.*` / `postgresql.image.*`. We keep the
#     default-injected `firestream-odoo:18.0` / `firestream-postgresql:17` refs
#     so the cluster runs the images scripts/deploy-local.sh side-loads.
#   - Credentials are INLINE (from config.nix), not `existingSecret`.
#   - Storage is the local-path provisioner; access is ClusterIP + port-forward
#     (no ingress controller assumed).
#
# Option names mirror src/charts/firestream/odoo/values.yaml exactly:
#   - admin keys (odooEmail/odooPassword) are FLAT (top-level)
#   - the database is the bundled `postgresql` subchart
# ---------------------------------------------------------------------------
{ ... }:

let
  cfg = import ../config.nix;
in
{
  config.odoo = {
    # --- Admin credentials (inline). With odooPassword set and no
    #     existingSecret, the chart writes this into its own odoo-password
    #     Secret; log in with odooEmail / odooPassword. ----------------------
    odooEmail = "user@example.com";
    odooPassword = cfg.odooPassword;

    # --- Database: bundled in-cluster PostgreSQL subchart -------------------
    postgresql = {
      enabled = true;
      architecture = "standalone";
      auth = {
        username = "firestream";
        database = "firestream_odoo";
        # `password` (app user) is a typed option; `postgresPassword`
        # (superuser) flows through the freeform subchart passthrough. Setting
        # both inline keeps `helm upgrade` idempotent on re-deploy.
        password = cfg.dbPassword;
        postgresPassword = cfg.dbPostgresPassword;
      };
      primary.persistence = {
        enabled = true;
        storageClass = cfg.storageClass;
        size = "10Gi";
      };
    };

    # --- Odoo filestore persistence on the local-path provisioner ----------
    persistence = {
      enabled = true;
      storageClass = cfg.storageClass;
      size = "10Gi";
    };

    # --- Access: ClusterIP + `kubectl port-forward` (Firestream convention).
    #     A port-80 LoadBalancer collides with k3d's built-in service-LB on
    #     single-node clusters; ingress needs a controller we don't assume. --
    service.type = "ClusterIP";
    ingress.enabled = false;

    replicaCount = 1;

    # Lighter footprint for a laptop. Drop this line to use the chart default
    # ("large"). Override `resources` directly for fine-grained control.
    resourcesPreset = "small";
  };
}
