{
  description = "Firestream Next.js — a downstream GKE deployment example (custom app, bundled PostgreSQL)";

  # ---------------------------------------------------------------------------
  # A Firestream deployment: a root flake that imports Firestream and
  # re-evaluates the nextjs supported app with a small override of your own.
  # Everything else here (Pulumi, Cloud Build, scripts) is the GCP plumbing
  # around that idea. The headline customization is YOUR Next.js app baked into
  # the image (./app via config.nextjs.vendoredApp).
  # ---------------------------------------------------------------------------
  inputs = {
    # The Firestream monorepo — the only input a downstream repo pins. Pin to a
    # tag/commit in production.
    firestream.url = "github:Cogent-Creation-Co/Firestream-one-flake-to-rule-them-all";

    nixpkgs.follows = "firestream/nixpkgs";
    flake-utils.follows = "firestream/flake-utils";
  };

  outputs = { self, firestream, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          config.allowUnfree = true; # google-cloud-sdk / pulumi
        };

        # The customization seam: re-evaluate the stock nextjs chart with our
        # override deep-merged on top of the Firestream defaults (and the
        # image-injection overlay).
        nextjs = firestream.lib.${system}.charts.nextjs.eval
          (import ./firestream/nextjs-overrides.nix);
        nextjsChart = nextjs.chartBundle;
      in
      {
        packages = {
          default = nextjsChart;
          nextjs-chart = nextjsChart;

          # CUSTOM Next.js image: re-evaluate the Firestream nextjs container
          # with our app baked in (firestream/nextjs-image-overrides.nix). Cloud
          # Build pushes this as <AR>/firestream-nextjs:<tag> and
          # nextjs-overrides.nix points the chart's image at that same ref, so
          # the customized image is both pushed AND run.
          nextjs-image = (firestream.lib.${system}.images.nextjs.eval
            (import ./firestream/nextjs-image-overrides.nix)).dockerImage;
          # The bundled database subchart image.
          postgresql-image = firestream.packages.${system}.postgresql;
        };

        apps.deploy = {
          # `nix run .#deploy -- --namespace nextjs [extra helm args]`
          type = "app";
          program = "${nextjsChart}/bin/deploy";
        };

        devShells.default = pkgs.mkShell {
          packages = with pkgs; [
            kubernetes-helm
            kubectl
            google-cloud-sdk
            pulumi-bin
          ];
          shellHook = ''
            echo "Firestream Next.js GKE example — edit ./config.nix, then see README.md" >&2
          '';
        };
      });
}
