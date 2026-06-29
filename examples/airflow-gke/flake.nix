{
  description = "Firestream Airflow — a downstream GKE deployment example (custom baked DAG)";

  # ---------------------------------------------------------------------------
  # A Firestream deployment: a root flake that imports Firestream and
  # re-evaluates a supported app with a small override of your own. Everything
  # else here (Pulumi, Cloud Build, scripts) is the GCP plumbing around that
  # idea. The headline customization is a CUSTOM DAG baked into the Airflow image
  # (./dags via config.airflow.vendoredDags).
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

        # The customization seam: re-evaluate the stock Airflow chart with our
        # override deep-merged on top of the Firestream defaults (and the
        # image-injection overlay).
        airflow = firestream.lib.${system}.charts.airflow.eval
          (import ./firestream/airflow-overrides.nix);
        airflowChart = airflow.chartBundle;
      in
      {
        packages = {
          default = airflowChart;
          airflow-chart = airflowChart;

          # CUSTOM Airflow image: re-evaluate the Firestream Airflow container
          # with our preferences (firestream/airflow-image-overrides.nix — the
          # baked DAG). Cloud Build pushes this as <AR>/firestream-airflow:<tag>
          # and airflow-overrides.nix points the chart's image at that same ref,
          # so the customized image is both pushed AND run.
          airflow-image = (firestream.lib.${system}.images.airflow.eval
            (import ./firestream/airflow-image-overrides.nix)).dockerImage;
          # Subchart images (CeleryExecutor needs both).
          postgresql-image = firestream.packages.${system}.postgresql;
          redis-image = firestream.packages.${system}.redis;
        };

        apps.deploy = {
          # `nix run .#deploy -- --namespace airflow [extra helm args]`
          type = "app";
          program = "${airflowChart}/bin/deploy";
        };

        devShells.default = pkgs.mkShell {
          packages = with pkgs; [
            kubernetes-helm
            kubectl
            google-cloud-sdk
            pulumi-bin
          ];
          shellHook = ''
            echo "Firestream Airflow GKE example — edit ./config.nix, then see README.md" >&2
          '';
        };
      });
}
