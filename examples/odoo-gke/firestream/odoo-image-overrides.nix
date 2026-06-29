# ---------------------------------------------------------------------------
# The container-level "make it your own" surface — the sibling of
# odoo-overrides.nix (which customizes the Helm CHART). This module is passed as
# the LAST module to `firestream.lib.<sys>.images.odoo.eval`, so every value here
# deep-merges on top of the Firestream container defaults and yields a CUSTOM
# Odoo image — without forking Firestream.
#
# This is the core Firestream value prop: inject your preferences into
# Firestream's Nix system, get a custom image back. The headline preference is
# `vendoredAddons` (third-party addon repos baked into the image at build time),
# but ANY bakeable container option works here, e.g.:
#
#   config.odoo.env.ODOO_LOAD_DEMO_DATA = "yes";   # extra/overridden env vars
#   config.odoo.exposedPorts = [ 8069 8072 ];      # ports
#   config.odoo.paths.data = "/opt/odoo/data";     # path layout
#   config.odoo.health.enable = true;              # in-image healthd
#
# Option names mirror src/containers/firestream/odoo/options.nix.
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
