# ---------------------------------------------------------------------------
# The container-level "make it your own" surface. Passed as the LAST module to
# `firestream.lib.<sys>.images.odoo.eval`, deep-merged on top of the Firestream
# container defaults to yield a CUSTOM Odoo image — without forking.
#
# The result is named `firestream-odoo:18.0`, which is exactly the image the
# generated docker-compose file references, so scripts/up.sh just `docker load`s
# this and `docker compose up` runs it.
#
# Option names mirror src/containers/firestream/odoo/options.nix.
# ---------------------------------------------------------------------------
{ ... }:

let
  cfg = import ../config.nix;
in
{
  # Bake the addon repos from config.nix into /opt/odoo/vendor-addons and wire
  # them into addons_path. See src/containers/firestream/odoo/vendor-addons.nix.
  config.odoo.vendoredAddons = cfg.vendoredAddons;
}
