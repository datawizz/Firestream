{
  description = "Firestream Airflow — a local Docker Compose example (custom baked DAG)";

  # ---------------------------------------------------------------------------
  # The lightest Firestream deployment: no cluster. Import Firestream, build a
  # CUSTOM Airflow image with a DAG baked in (the container "make it your own"
  # seam), and run it with the multi-service docker-compose stack Firestream
  # generates for Airflow (postgresql + redis + scheduler/triggerer/dag-processor/
  # worker + api-server). Sibling of ../airflow-k3s and ../airflow-gke.
  # ---------------------------------------------------------------------------
  inputs = {
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
          # CUSTOM Airflow image (DAG from ./dags baked in). Named
          # `firestream-airflow:3.0.3`, exactly what the generated compose
          # references — so scripts/up.sh `docker load`s it and the stack runs it.
          airflow-image = (firestream.lib.${system}.images.airflow.eval
            (import ./firestream/airflow-image-overrides.nix)).dockerImage;
          postgresql-image = firestream.packages.${system}.postgresql;
          redis-image = firestream.packages.${system}.redis;

          # The Firestream-generated docker-compose stack for Airflow:
          # $out/docker-compose.yml (5 airflow services + postgresql + redis,
          # healthchecks, volumes, host-port mapping). References images by name.
          airflow-compose = firestream.lib.${system}.images.airflow.compose;
          default = self.packages.${system}.airflow-compose;
        };

        devShells.default = pkgs.mkShell {
          packages = with pkgs; [ docker-compose jq ];
          shellHook = ''
            echo "Firestream Airflow (docker-compose) example — 'make up' to start" >&2
          '';
        };
      });
}
