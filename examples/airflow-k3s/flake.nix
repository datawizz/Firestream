{
  description = "Firestream Airflow — a local k3s / k3d deployment example (custom baked DAG)";

  # ---------------------------------------------------------------------------
  # A Firestream deployment: import Firestream, re-evaluate a supported app with
  # a small override of your own. The headline customization here is a CUSTOM
  # DAG baked into the Airflow image (./dags → /opt/firestream/airflow/dags via
  # config.airflow.vendoredDags) — the Airflow analogue of odoo's vendored
  # addons. Local-cluster sibling of ../airflow-gke and ../airflow-docker-compose.
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

        # Re-evaluate the stock Airflow chart with our local override.
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
          # baked DAG). Still named `firestream-airflow:3.0.3`, so the chart's
          # default-injected image ref (which we do NOT repoint) picks it up once
          # scripts/deploy-local.sh side-loads it into the cluster.
          airflow-image = (firestream.lib.${system}.images.airflow.eval
            (import ./firestream/airflow-image-overrides.nix)).dockerImage;
          # The two subchart images (Celery needs both).
          postgresql-image = firestream.packages.${system}.postgresql;
          redis-image = firestream.packages.${system}.redis;
        };

        apps.deploy = {
          # `nix run .#deploy -- --namespace airflow`. NOTE: helm only — does NOT
          # side-load images. Use `make deploy` for the full local loop.
          type = "app";
          program = "${airflowChart}/bin/deploy";
        };

        devShells.default = pkgs.mkShell {
          packages = with pkgs; [ kubernetes-helm kubectl k3d ];
          shellHook = ''
            echo "Firestream Airflow (local k3s) example — edit ./config.nix or ./dags, then 'make deploy'" >&2
          '';
        };
      });
}
