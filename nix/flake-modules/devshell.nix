# Dev shell flake-module
# Copyright Firestream. MIT License.
#
# Ports the legacy flake's devShells.default 1:1. On Darwin uses mkShellNoCC
# (avoids stdenv's automatic SDK setup); shellHook comes from modules/darwin.nix.
{ inputs, ... }: {
  perSystem = { pkgs, system, ... }:
    let
      isDarwin = pkgs.stdenv.isDarwin;

      shellEnv = import ./lib/shell-env.nix {
        inherit pkgs system;
        inherit (inputs) fenix;
      };

      darwin = import ../../bin/nix/firestream/modules/darwin.nix {
        inherit pkgs;
        lib = pkgs.lib;
      };
    in
    {
      devShells.default = (if isDarwin then pkgs.mkShellNoCC else pkgs.mkShell) {
        packages = shellEnv.shellPackages;
        shellHook = darwin.shellHook;
      };
    };
}
