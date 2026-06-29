{
  description = "Firestream Next.js — a local k3s / k3d deployment example (custom app, bundled PostgreSQL)";

  # ---------------------------------------------------------------------------
  # A Firestream deployment: import Firestream, re-evaluate the nextjs supported
  # app with a small override of your own. The headline customization is YOUR
  # Next.js app baked into the image (./app → standalone build via the
  # `vendoredApp` seam) plus an inline-credentialed bundled PostgreSQL. Local-
  # cluster sibling of ../nextjs-gke and ../nextjs-docker-compose.
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

        # Re-evaluate the stock nextjs chart with our local override.
        nextjs = firestream.lib.${system}.charts.nextjs.eval
          (import ./firestream/nextjs-overrides.nix);
        nextjsChart = nextjs.chartBundle;
      in
      {
        packages = {
          default = nextjsChart;
          nextjs-chart = nextjsChart;

          # CUSTOM Next.js image: re-evaluate the Firestream nextjs container
          # with our app baked in (firestream/nextjs-image-overrides.nix). Still
          # named `firestream-nextjs:1.0.0`, so the chart's default-injected
          # image ref (which we do NOT repoint) picks it up once
          # scripts/deploy-local.sh side-loads it into the cluster.
          nextjs-image = (firestream.lib.${system}.images.nextjs.eval
            (import ./firestream/nextjs-image-overrides.nix)).dockerImage;
          # The bundled database subchart image.
          postgresql-image = firestream.packages.${system}.postgresql;
        };

        apps.deploy = {
          # `nix run .#deploy -- --namespace nextjs`. NOTE: helm only — does NOT
          # side-load images. Use `make deploy` for the full local loop.
          type = "app";
          program = "${nextjsChart}/bin/deploy";
        };

        devShells.default = pkgs.mkShell {
          packages = with pkgs; [ kubernetes-helm kubectl k3d ];
          shellHook = ''
            echo "Firestream Next.js (local k3s) example — edit ./config.nix or ./app, then 'make deploy'" >&2
          '';
        };
      });
}
