# ---------------------------------------------------------------------------
# YOUR local deployment configuration. The single source of truth for both the
# Nix chart override (firestream/odoo-overrides.nix) and the deploy script
# (scripts/deploy-local.sh). Much smaller than the GKE example's config — there
# is no cloud project, registry, or Secret Manager to wire.
# ---------------------------------------------------------------------------
{
  # --- Target namespace ----------------------------------------------------
  namespace = "odoo";

  # --- Storage -------------------------------------------------------------
  # k3s and k3d both ship the rancher.io/local-path provisioner as the default
  # StorageClass. Leave as "local-path" unless your cluster uses something else.
  storageClass = "local-path";

  # --- Credentials (inline) ------------------------------------------------
  # Local example: passwords live here, NOT in Secret Manager. Setting them
  # explicitly (rather than letting the chart randomise) gives a KNOWN admin
  # login AND lets `helm upgrade` re-deploy without Bitnami's "you must provide
  # your current passwords when upgrading" guard tripping. Change them freely.
  odooPassword = "admin1234"; # Odoo admin (login: see odooEmail in the override)
  dbPassword = "odoo"; # PostgreSQL application user (firestream)
  dbPostgresPassword = "odoo-admin"; # PostgreSQL superuser (postgres)

  # --- Build-time vendored Odoo addons -------------------------------------
  # Same headline "inject preferences → custom image, no fork" demo as the GKE
  # example. Fetched at IMAGE BUILD time and baked into /opt/odoo/vendor-addons
  # (wired into addons_path). Pin `rev` to a commit; `hash` is the
  # fetchFromGitHub SRI hash. Set to [ ] to skip and use the stock image.
  vendoredAddons = [
    {
      name = "oca-server-tools";
      owner = "OCA";
      repo = "server-tools";
      rev = "13acf53ba602809d64efdc634dd87dd3439e1564"; # 18.0 @ 2026-06-27
      hash = "sha256-dAntlcVk42jme8Vo4ds7sK1OCjyKyZw2n/mKyO+/j3o=";
      modules = [ "base_fontawesome" ];
    }
  ];
}
