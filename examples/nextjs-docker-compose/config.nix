# ---------------------------------------------------------------------------
# YOUR configuration for the Docker Compose example. No cluster, so this is just
# the build-time image customization. Service ports, credentials, and the
# bundled PostgreSQL come from Firestream's generated compose file (see
# README.md → "Ports & credentials"). The custom app is the ./app folder, wired
# in via firestream/nextjs-image-overrides.nix.
# ---------------------------------------------------------------------------
{
  # Reserved for future build-time knobs. The app source is ./app (referenced
  # directly from nextjs-image-overrides.nix). Edit ./app/app/page.jsx to change
  # what the site renders.
}
