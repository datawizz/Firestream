# Odoo container flake-module (Phase 4)
# Copyright Firestream. MIT License.
#
# Wires the odoo container (multi-version 15/16/17/18) through the options-driven
# evalContainer entrypoint and contributes:
#   - packages.odoo / packages.odoo-15 / ... / packages.odoo-18
#       (docker images; stub off-Linux). `odoo` aliases the v18 build.
#   - firestreamContainers.odoo / odoo-15 / odoo-16 / odoo-17 / odoo-18
#   - firestreamImages.odoo / odoo-15 / odoo-16 / odoo-17 / odoo-18
#
# version + python vary per build and are supplied via an inline override module;
# the per-version Odoo source derivation is forwarded via extraFactoryArgs.
{ ... }: {
  perSystem = { pkgs, lib, system, evalContainer, ... }:
    let
      isLinux = pkgs.stdenv.hostPlatform.isLinux;

      optionsPath = ../../../src/containers/firestream/odoo/options.nix;
      sourceFn = import ../../../src/containers/firestream/odoo/source.nix;

      # workspacePath is odoo/${v} (where uv.lock + the thin module.nix wrapper live).
      workspacePathFor = v: ../../../src/containers/firestream/odoo + "/${v}";

      # Python interpreter per Odoo release (matches legacy pythonForVersion map).
      pythonForVersion = {
        "15" = pkgs.python310;
        "16" = pkgs.python310;
        "17" = pkgs.python311;
        "18" = pkgs.python312;
      };

      mkOdoo = v: evalContainer {
        name = "odoo";
        runtimeType = "python-workspace";
        workspacePath = workspacePathFor v;
        modules = [
          optionsPath
          ({ ... }: {
            config.odoo = {
              version = "${v}.0";
              python = pythonForVersion.${v};
            };
          })
        ];
        extraFactoryArgs = { odooSource = sourceFn { inherit pkgs; version = v; }; };
      };

      c15 = mkOdoo "15";
      c16 = mkOdoo "16";
      c17 = mkOdoo "17";
      c18 = mkOdoo "18";

      mkImage = v: c: {
        dockerImage = c.dockerImage;
        eval = userMod: evalContainer {
          name = "odoo";
          runtimeType = "python-workspace";
          workspacePath = workspacePathFor v;
          modules = [
            optionsPath
            ({ ... }: {
              config.odoo = {
                version = "${v}.0";
                python = pythonForVersion.${v};
              };
            })
            userMod
          ];
          extraFactoryArgs = { odooSource = sourceFn { inherit pkgs; version = v; }; };
        };
        options = c.options;
      };

      unavailable = n: pkgs.runCommand "${n}-not-available" { } ''
        echo "Docker images only available on Linux systems" > $out
      '';
    in
    {
      packages.odoo = if isLinux then c18.dockerImage else unavailable "odoo";
      packages.odoo-15 = if isLinux then c15.dockerImage else unavailable "odoo-15";
      packages.odoo-16 = if isLinux then c16.dockerImage else unavailable "odoo-16";
      packages.odoo-17 = if isLinux then c17.dockerImage else unavailable "odoo-17";
      packages.odoo-18 = if isLinux then c18.dockerImage else unavailable "odoo-18";

      # Registry: `odoo` -> v18 (matches manifest membership), plus per-version keys.
      firestreamContainers.odoo = lib.optionalAttrs isLinux c18;
      firestreamContainers.odoo-15 = lib.optionalAttrs isLinux c15;
      firestreamContainers.odoo-16 = lib.optionalAttrs isLinux c16;
      firestreamContainers.odoo-17 = lib.optionalAttrs isLinux c17;
      firestreamContainers.odoo-18 = lib.optionalAttrs isLinux c18;

      firestreamImages.odoo = mkImage "18" c18;
      firestreamImages.odoo-15 = mkImage "15" c15;
      firestreamImages.odoo-16 = mkImage "16" c16;
      firestreamImages.odoo-17 = mkImage "17" c17;
      firestreamImages.odoo-18 = mkImage "18" c18;
    };
}
