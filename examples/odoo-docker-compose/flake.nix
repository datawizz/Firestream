{
  description = "Firestream Odoo — a local Docker Compose example (no Kubernetes)";

  # ---------------------------------------------------------------------------
  # The lightest possible Firestream deployment: no cluster at all. We import
  # Firestream, build a CUSTOM Odoo image (the container "make it your own"
  # seam), and run it with the docker-compose stack Firestream generates for the
  # Odoo app (postgresql + odoo, wired together). Sibling of ../odoo-k3s and
  # ../odoo-gke — same app, different runtime. See ../README.md for the pattern.
  # ---------------------------------------------------------------------------
  inputs = {
    # The Firestream monorepo — the only input a downstream repo pins. The bare
    # URL tracks the default branch (`main`); pin `?ref=<tag-or-commit>` for
    # reproducibility in real use.
    firestream.url = "github:Cogent-Creation-Co/Firestream-one-flake-to-rule-them-all";

    nixpkgs.follows = "firestream/nixpkgs";
    flake-utils.follows = "firestream/flake-utils";
  };

  outputs = { self, firestream, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
      in
      {
        packages = {
          # CUSTOM Odoo image (vendored addons from config.nix). Named
          # `firestream-odoo:18.0`, which is exactly what the generated compose
          # file references — so once scripts/up.sh `docker load`s this, the
          # stack runs it. This is the container analogue of charts.odoo.eval.
          odoo-image = (firestream.lib.${system}.images.odoo.eval
            (import ./firestream/odoo-image-overrides.nix)).dockerImage;
          postgresql-image = firestream.packages.${system}.postgresql;

          # The Firestream-generated docker-compose stack for the Odoo app:
          # $out/docker-compose.yml (services: postgresql + odoo, healthchecks,
          # named volume, host-port mapping). It references images by NAME, so
          # our custom odoo-image (same name) is picked up after loading.
          odoo-compose = firestream.lib.${system}.images.odoo.compose;
          default = self.packages.${system}.odoo-compose;
        };

        devShells.default = pkgs.mkShell {
          packages = with pkgs; [ docker-compose jq ];
          shellHook = ''
            echo "Firestream Odoo (docker-compose) example — 'make up' to start" >&2
          '';
        };
      });
}
