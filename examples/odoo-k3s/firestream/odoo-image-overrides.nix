# ---------------------------------------------------------------------------
# The container-level "make it your own" surface — sibling of odoo-overrides.nix
# (which customizes the Helm CHART). Passed as the LAST module to
# `firestream.lib.<sys>.images.odoo.eval`, so it deep-merges on top of the
# Firestream container defaults and yields a CUSTOM Odoo image — without forking.
#
# Identical in spirit to ../odoo-gke: the resulting image is still named
# `firestream-odoo:18.0`, and because odoo-overrides.nix does NOT repoint the
# chart's image ref, the cluster runs THIS image once scripts/deploy-local.sh
# side-loads it into containerd.
#
# Option names mirror src/containers/firestream/odoo/options.nix. Other bakeable
# knobs you could set here: config.odoo.env.* , config.odoo.exposedPorts , etc.
# ---------------------------------------------------------------------------
{ ... }:

let
  cfg = import ../config.nix;
in
{
  # Bake the addon repos declared in config.nix into /opt/odoo/vendor-addons and
  # wire them into addons_path. See src/containers/firestream/odoo/vendor-addons.nix.
  config.odoo.vendoredAddons = cfg.vendoredAddons;
}
