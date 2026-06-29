# ---------------------------------------------------------------------------
# YOUR local Next.js deployment configuration. Single source of truth for the
# chart override (firestream/nextjs-overrides.nix) and the deploy script.
#
# The custom app is NOT configured here; it's the ./app folder, baked into the
# image via firestream/nextjs-image-overrides.nix.
# ---------------------------------------------------------------------------
{
  namespace = "nextjs";

  # k3s/k3d default provisioner.
  storageClass = "local-path";

  # --- Credentials (inline) ------------------------------------------------
  # Local example: secrets live here, not in Secret Manager. Explicit values
  # keep `helm upgrade` idempotent (Bitnami's postgres refuses upgrades with an
  # empty password).
  dbPassword = "nextjs"; # PostgreSQL application user (firestream)
  dbPostgresPassword = "nextjs-admin"; # PostgreSQL superuser
}
