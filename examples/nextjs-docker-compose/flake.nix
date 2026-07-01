{
  description = "Firestream Next.js — a local Docker Compose example (custom app, bundled PostgreSQL)";

  # ---------------------------------------------------------------------------
  # The lightest Firestream Next.js deployment: no cluster. Import Firestream,
  # build a CUSTOM Next.js image with YOUR app baked in (the `vendoredApp` seam),
  # and run it with the docker-compose stack Firestream generates for nextjs
  # (firestream-nextjs + firestream-postgresql). Sibling of ../nextjs-k3s and
  # ../nextjs-gke.
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
          # CUSTOM Next.js image (app from ./app baked in). Named
          # `firestream-nextjs:1.0.0`, exactly what the generated compose
          # references — so scripts/up.sh `docker load`s it and the stack runs it.
          nextjs-image = (firestream.lib.${system}.images.nextjs.eval
            (import ./firestream/nextjs-image-overrides.nix)).dockerImage;
          postgresql-image = firestream.packages.${system}.postgresql;

          # The Firestream-generated docker-compose stack for nextjs:
          # $out/docker-compose.yml (nextjs + postgresql, healthchecks, volumes,
          # host-port mapping). References images by name.
          nextjs-compose = firestream.lib.${system}.images.nextjs.compose;
          default = self.packages.${system}.nextjs-compose;
        };

        devShells.default = pkgs.mkShell {
          packages = with pkgs; [ docker-compose jq ];
          shellHook = ''
            echo "Firestream Next.js (docker-compose) example — 'make up' to start" >&2
          '';
        };
      });
}
