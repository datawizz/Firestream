# Wrapper for running tests with nixpkgs
let
  pkgs = import <nixpkgs> {};
in
  import ./default.nix { inherit pkgs; }
