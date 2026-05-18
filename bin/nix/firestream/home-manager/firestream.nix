# home-manager module for Firestream CLI/TUI
{ config, lib, pkgs, ... }:

let
  cfg = config.programs.firestream;
in {
  options.programs.firestream = {
    enable = lib.mkEnableOption "Firestream CLI/TUI";

    package = lib.mkPackageOption pkgs "firestream" {
      extraDescription = ''
        Note: This package is provided by the Firestream flake overlay.
        Add `nixpkgs.overlays = [ inputs.firestream.overlays.default ];`
        to your configuration.
      '';
    };
  };

  config = lib.mkIf cfg.enable {
    home.packages = [ cfg.package ];
  };
}
