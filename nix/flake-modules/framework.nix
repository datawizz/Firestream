# Framework flake-module
# Copyright Firestream. MIT License.
#
# Establishes the shared perSystem module args used by every other flake-module:
#   - pkgs:            nixpkgs configured with allowUnfree (matching the legacy
#                      pkgsForSystem exactly; NOT flake-parts' default pkgs).
#   - firestreamLib:   the Firestream Nix module system (bin/nix/firestream).
#   - evalContainer:   the options-driven container evaluation entrypoint.
#   - evalChart:       the options-driven Helm chart evaluation entrypoint.
#   - toChartManifest: pure function from a resolved chart `cfg` to a
#                      chart-manifest.json derivation. Surfaced so downstream
#                      flake-modules (Agent C's aggregate index, debug builds,
#                      etc.) can produce a manifest WITHOUT going through the
#                      full chart-bundle build.
#   - injectContainerImages: pure function merging container-registry image
#                      overrides into a chart's values tree. Surfaced for
#                      flake-modules that need to compose values from
#                      `firestreamImages.<name>` outside the bundle build.
{ inputs, ... }: {
  perSystem = { system, lib, ... }:
    let
      # CRITICAL for parity: identical to the legacy flake's pkgsForSystem.
      pkgs = import inputs.nixpkgs {
        inherit system;
        config.allowUnfree = true;
      };

      firestreamLib = import ../../bin/nix/firestream {
        inherit pkgs system;
        inherit (inputs) fenix crane pyproject-nix uv2nix pyproject-build-systems;
      };

      evalContainer = import ../../bin/nix/firestream/containers/eval-container.nix {
        inherit pkgs lib firestreamLib;
      };

      evalChart = import ../../bin/nix/firestream/charts/eval-chart.nix {
        inherit pkgs lib firestreamLib;
      };

      baseChart = import ../../bin/nix/firestream/charts/lib/base-chart.nix {
        inherit pkgs lib;
      };

      toChartManifest = import ../../bin/nix/firestream/charts/lib/to-chart-manifest.nix {
        inherit pkgs lib;
      };

      injectContainerImages = import ../../bin/nix/firestream/charts/lib/inject-container-images.nix {
        inherit lib;
      };
    in
    {
      _module.args.pkgs = pkgs;
      _module.args.firestreamLib = firestreamLib;
      _module.args.evalContainer = evalContainer;
      _module.args.evalChart = evalChart;
      _module.args.baseChart = baseChart;
      _module.args.toChartManifest = toChartManifest;
      _module.args.injectContainerImages = injectContainerImages;
    };
}
