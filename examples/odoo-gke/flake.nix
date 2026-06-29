{
  description = "Firestream Odoo — a downstream GKE deployment example";

  # ---------------------------------------------------------------------------
  # This is the ONE thing that makes a repo "a Firestream deployment": a root
  # flake that imports Firestream and re-evaluates a supported app with a small
  # override of your own. Everything else in this directory (Pulumi, Cloud
  # Build, scripts) is just the GCP plumbing around that idea.
  # ---------------------------------------------------------------------------
  inputs = {
    # The Firestream monorepo. For a real downstream repo this is the only
    # input you pin. Pin to a tag/commit in production.
    firestream.url = "github:Cogent-Creation-Co/Firestream-one-flake-to-rule-them-all";

    # --- Developing against an in-repo checkout? Comment the line above and
    #     use the local path instead (this example lives at examples/odoo-gke): ---
    # firestream.url = "path:../..";

    # Re-use Firestream's pinned nixpkgs / flake-utils so versions match.
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

        # The customization seam: re-evaluate the stock Odoo chart with our
        # override module deep-merged on top of the Firestream defaults (and
        # on top of the image-injection overlay). Returns
        # { chartBundle; render; config; options; ... }.
        odoo = firestream.lib.${system}.charts.odoo.eval
          (import ./firestream/odoo-overrides.nix);

        # The deployable bundle: $out/{chart,values.yaml,chart-manifest.json}
        # plus an executable $out/bin/deploy (helm upgrade --install wrapper).
        odooChart = odoo.chartBundle;
      in
      {
        packages = {
          default = odooChart;
          # The customized Odoo chart bundle (build + inspect with `nix build`).
          odoo-chart = odooChart;

          # The Firestream-built container images, so Cloud Build can
          # `nix build .#odoo-image` / `.#postgresql-image` and push them to
          # Artifact Registry. (Linux-only; on macOS these are build stubs, but
          # evaluation is always safe.)
          #
          # odoo-image is a CUSTOM image: we re-evaluate the Firestream Odoo
          # container with our own preferences (firestream/odoo-image-overrides.nix
          # — vendored addons, env, …) deep-merged on top of the defaults. This is
          # the container analogue of charts.odoo.eval above, and the heart of the
          # Firestream value prop: inject preferences → get a custom image. Since
          # cloudbuild pushes this as <AR>/firestream-odoo:<tag> and the chart
          # override points image.repository/tag at that same ref, the customized
          # image is both what gets pushed AND what the cluster pulls.
          odoo-image = (firestream.lib.${system}.images.odoo.eval
            (import ./firestream/odoo-image-overrides.nix)).dockerImage;
          postgresql-image = firestream.packages.${system}.postgresql;
        };

        apps = {
          # `nix run .#deploy -- --namespace odoo [extra helm args]`
          deploy = {
            type = "app";
            program = "${odooChart}/bin/deploy";
          };
        };

        devShells.default = pkgs.mkShell {
          packages = with pkgs; [
            kubernetes-helm
            kubectl
            google-cloud-sdk
            pulumi-bin
          ];
          shellHook = ''
            echo "Firestream Odoo deployment example — edit ./config.nix, then see README.md" >&2
          '';
        };
      });
}
