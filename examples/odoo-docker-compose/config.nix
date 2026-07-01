# ---------------------------------------------------------------------------
# YOUR configuration for the Docker Compose example. There's no cluster, so this
# is mostly the build-time image customization. Service ports, DB credentials,
# and the data volume are defined by Firestream's generated compose file (see
# README.md → "Ports & credentials").
# ---------------------------------------------------------------------------
{
  # --- Build-time vendored Odoo addons -------------------------------------
  # The headline "inject preferences → custom image, no fork" demo. Fetched at
  # IMAGE BUILD time and baked into /opt/odoo/vendor-addons (wired into
  # addons_path). Pin `rev` to a commit; `hash` is the fetchFromGitHub SRI hash.
  # Set to [ ] to skip and run the stock image.
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
