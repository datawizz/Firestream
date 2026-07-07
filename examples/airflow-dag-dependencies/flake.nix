{
  description = "Firestream Airflow — guest-DAG dependencies as a separate uv2nix venv (local k3s / k3d)";

  # ---------------------------------------------------------------------------
  # A Firestream deployment: import Firestream, re-evaluate a supported app with
  # a small override of your own. The headline customization here is guest-DAG
  # Python DEPENDENCIES resolved via uv2nix into a SEPARATE venv baked at
  # /opt/firestream/airflow/dags-venv (./dags-workspace →
  # config.airflow.dagWorkspace), isolated from Airflow's own venv. Local-cluster
  # sibling of ../airflow-k3s (which bakes a DAG via config.airflow.vendoredDags).
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
          # dagWorkspace guest venv, plus the DAG file baked via vendoredDags).
          # `config.airflow.dagWorkspace` is the seam this consumes: uv2nix →
          # /opt/firestream/airflow/dags-venv. Still named
          # `firestream-airflow:3.0.3`, so the chart's default-injected image ref
          # (which we do NOT repoint) picks it up once scripts/deploy-local.sh
          # side-loads it into the cluster.
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
          packages = with pkgs; [ kubernetes-helm kubectl k3d uv ];
          shellHook = ''
            echo "Firestream Airflow guest-DAG-deps (local k3s) example — edit ./dags-workspace or ./dags, then 'make deploy'" >&2
          '';
        };
      });
}
