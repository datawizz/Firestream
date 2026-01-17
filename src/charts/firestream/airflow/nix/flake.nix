{
  description = "Firestream Airflow Helm Chart - Nix Module System for values.yaml generation";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/release-25.11";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    let
      # Chart path relative to this flake
      chartPath = ./..;

      # Import the module system
      airflowModule = import ./modules;

      # Import environment presets
      presets = import ./presets;

    in
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        lib = pkgs.lib;

        # Evaluate module configuration with user overrides
        evalConfig = userConfig: lib.evalModules {
          modules = [
            airflowModule
            userConfig
          ];
          specialArgs = { inherit pkgs lib; };
        };

        # Default configuration using development preset
        defaultConfig = evalConfig presets.development;

        # Generate values.yaml from evaluated config
        generateValuesYaml = config:
          let
            generators = import ./modules/lib/generators.nix { inherit lib; };
          in
          generators.toValuesYaml config.config.airflow.generatedValues;

        # Default values.yaml output
        defaultValuesYaml = pkgs.writeText "values.yaml" (generateValuesYaml defaultConfig);

      in {
        # Library functions for external use
        lib = {
          inherit evalConfig generateValuesYaml;
          inherit presets;
          types = import ./modules/types { inherit lib; };
        };

        packages = {
          # Generated values.yaml file (default preset)
          values-yaml = defaultValuesYaml;

          # Helm install wrapper script
          helm-install = pkgs.writeShellScriptBin "helm-install-airflow" ''
            #!/usr/bin/env bash
            set -euo pipefail

            VALUES_FILE="''${1:-}"
            RELEASE_NAME="''${2:-airflow}"
            NAMESPACE="''${3:-airflow}"

            if [[ -z "$VALUES_FILE" ]]; then
              echo "Usage: helm-install-airflow <values-file> [release-name] [namespace]"
              exit 1
            fi

            echo "Installing Airflow Helm chart..."
            echo "  Release: $RELEASE_NAME"
            echo "  Namespace: $NAMESPACE"
            echo "  Values: $VALUES_FILE"

            ${pkgs.kubernetes-helm}/bin/helm install "$RELEASE_NAME" \
              ${chartPath} \
              --namespace "$NAMESPACE" \
              --create-namespace \
              -f "$VALUES_FILE" \
              "''${@:4}"
          '';

          # Helm upgrade wrapper script
          helm-upgrade = pkgs.writeShellScriptBin "helm-upgrade-airflow" ''
            #!/usr/bin/env bash
            set -euo pipefail

            VALUES_FILE="''${1:-}"
            RELEASE_NAME="''${2:-airflow}"
            NAMESPACE="''${3:-airflow}"

            if [[ -z "$VALUES_FILE" ]]; then
              echo "Usage: helm-upgrade-airflow <values-file> [release-name] [namespace]"
              exit 1
            fi

            echo "Upgrading Airflow Helm chart..."
            echo "  Release: $RELEASE_NAME"
            echo "  Namespace: $NAMESPACE"
            echo "  Values: $VALUES_FILE"

            ${pkgs.kubernetes-helm}/bin/helm upgrade "$RELEASE_NAME" \
              ${chartPath} \
              --namespace "$NAMESPACE" \
              -f "$VALUES_FILE" \
              "''${@:4}"
          '';

          # Helm template (dry-run) wrapper script
          helm-template = pkgs.writeShellScriptBin "helm-template-airflow" ''
            #!/usr/bin/env bash
            set -euo pipefail

            VALUES_FILE="''${1:-}"
            RELEASE_NAME="''${2:-airflow}"

            if [[ -z "$VALUES_FILE" ]]; then
              echo "Usage: helm-template-airflow <values-file> [release-name]"
              exit 1
            fi

            ${pkgs.kubernetes-helm}/bin/helm template "$RELEASE_NAME" \
              ${chartPath} \
              -f "$VALUES_FILE" \
              "''${@:3}"
          '';

          # Helm diff (requires helm-diff plugin)
          helm-diff = pkgs.writeShellScriptBin "helm-diff-airflow" ''
            #!/usr/bin/env bash
            set -euo pipefail

            VALUES_FILE="''${1:-}"
            RELEASE_NAME="''${2:-airflow}"
            NAMESPACE="''${3:-airflow}"

            if [[ -z "$VALUES_FILE" ]]; then
              echo "Usage: helm-diff-airflow <values-file> [release-name] [namespace]"
              exit 1
            fi

            ${pkgs.kubernetes-helm}/bin/helm diff upgrade "$RELEASE_NAME" \
              ${chartPath} \
              --namespace "$NAMESPACE" \
              -f "$VALUES_FILE" \
              "''${@:4}"
          '';

          default = self.packages.${system}.values-yaml;
        };

        # Apps for direct execution via `nix run`
        apps = {
          helm-install = flake-utils.lib.mkApp {
            drv = self.packages.${system}.helm-install;
          };
          helm-upgrade = flake-utils.lib.mkApp {
            drv = self.packages.${system}.helm-upgrade;
          };
          helm-template = flake-utils.lib.mkApp {
            drv = self.packages.${system}.helm-template;
          };
          helm-diff = flake-utils.lib.mkApp {
            drv = self.packages.${system}.helm-diff;
          };
        };

        # Development shell with helm and kubectl
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            kubernetes-helm
            kubectl
            yq-go
            jq
          ];

          shellHook = ''
            echo "Firestream Airflow Helm Chart Development Shell"
            echo ""
            echo "Available commands:"
            echo "  nix build .#values-yaml    - Generate values.yaml"
            echo "  nix run .#helm-install     - Install chart"
            echo "  nix run .#helm-upgrade     - Upgrade chart"
            echo "  nix run .#helm-template    - Render templates"
            echo ""
          '';
        };
      }
    );
}
