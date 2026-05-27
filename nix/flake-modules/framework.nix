# Framework flake-module
# Copyright Firestream. MIT License.
#
# Establishes the shared perSystem module args used by every other flake-module:
#   - pkgs:          nixpkgs configured with allowUnfree (matching the legacy
#                    pkgsForSystem exactly; NOT flake-parts' default pkgs).
#   - firestreamLib: the Firestream Nix module system (bin/nix/firestream).
#   - evalContainer: the options-driven container evaluation entrypoint.
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
    in
    {
      _module.args.pkgs = pkgs;
      _module.args.firestreamLib = firestreamLib;
      _module.args.evalContainer = evalContainer;
    };
}
