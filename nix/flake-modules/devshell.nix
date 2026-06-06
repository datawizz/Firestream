# Dev shell flake-module
# Copyright Firestream. MIT License.
#
# Ports the legacy flake's devShells.default 1:1. On Darwin uses mkShellNoCC
# (avoids stdenv's automatic SDK setup); shellHook comes from modules/darwin.nix.
#
# Phase 5: also wires `FIRESTREAM_CHARTS_DIR` to `$PWD/.firestream/charts`
# and (best-effort) builds `packages.firestream-charts-bundle` into that
# path so the Rust CLI's `helm deploy` subcommand finds an index.json
# without operators having to set the env var manually.
{ inputs, ... }: {
  perSystem = { pkgs, system, self', ... }:
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

      # Best-effort: if the chart bundle build succeeds, point
      # FIRESTREAM_CHARTS_DIR at a per-shell symlink so `firestream helm
      # deploy ...` works without explicit --charts-dir. If the build fails
      # (charts wave not yet landed, network issues during fetch, etc.)
      # we still load the shell — `firestream helm` will surface a clear
      # error pointing operators at the build command.
      chartsBundleAttr = "firestream-charts-bundle";
      chartsHook = ''
        # Resolve to an absolute path so tools that change cwd (cargo run
        # in a workspace member, IDEs) still see the right location.
        export FIRESTREAM_CHARTS_DIR="$PWD/.firestream/charts"
        mkdir -p "$PWD/.firestream"
        if [ ! -e "$FIRESTREAM_CHARTS_DIR/index.json" ]; then
          echo "[firestream devShell] Building chart bundle (best-effort)..." >&2
          if nix build ".#${chartsBundleAttr}" \
              --out-link "$FIRESTREAM_CHARTS_DIR" 2>/dev/null; then
            echo "[firestream devShell] FIRESTREAM_CHARTS_DIR=$FIRESTREAM_CHARTS_DIR" >&2
          else
            echo "[firestream devShell] Chart bundle build skipped; " \
                 "run 'nix build .#${chartsBundleAttr} --out-link $FIRESTREAM_CHARTS_DIR' manually if needed." >&2
          fi
        fi
      '';
    in
    {
      devShells.default = (if isDarwin then pkgs.mkShellNoCC else pkgs.mkShell) {
        packages = shellEnv.shellPackages;
        shellHook = darwin.shellHook + chartsHook;
      };
    };
}
