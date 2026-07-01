# Docker-based Linux image builder flake-module
# Copyright Firestream. MIT License.
#
# Makes "build a Linux container image from a Darwin host via Docker" a
# first-class flake capability. Firestream's container images are built with
# dockerTools and are gated behind `isLinux`; on macOS `nix build .#airflow`
# yields a stub. This module ships a self-contained shell app that runs the Nix
# build *inside* a `nixos/nix` Linux container (selecting `--platform`), then
# loads the resulting image tarball into the local Docker daemon.
#
# It contributes (on EVERY system, Darwin and Linux):
#   - apps.build-image            generic: `nix run .#build-image -- <pkg> [opts]`
#   - apps.<name>-image           one per image registry key (airflow-image, ...)
#   - _module.args.firestreamBuildImage   the builder derivation (used by compose.nix)
#
# The logic mirrors the proven bin/build/container-images.sh: a persistent
# per-arch Nix store volume for caching, git-worktree dir mounts, and `docker
# load` of the dereferenced tarball.
{ inputs, ... }: {
  perSystem = { pkgs, lib, config, system, ... }:
    let
      # Default target arch = host arch (aarch64-darwin -> aarch64 -> linux/arm64).
      hostArch = lib.head (lib.splitString "-" system);

      builder = pkgs.writeShellScriptBin "firestream-build-image" ''
        set -euo pipefail
        export PATH=${lib.makeBinPath [ pkgs.coreutils pkgs.gnused pkgs.gnugrep pkgs.gawk pkgs.git ]}:"$PATH"

        # Baked at app-build time: the flake's own source (store path) is the
        # fallback when not invoked from inside a Firestream working tree.
        SELF_STORE_PATH=${inputs.self}
        DEFAULT_ARCH=${lib.escapeShellArg hostArch}

        usage() {
          cat <<'EOF'
        firestream-build-image — build a Firestream container image via Docker.

        Usage: firestream-build-image <package> [options]

          <package>            Flake package to build (e.g. airflow, postgresql, redis, kafka)

        Options:
          --target <arch>      Target arch: x86_64 | aarch64  (default: host arch)
          --load               Load the image into the local Docker daemon (default)
          --no-load            Build the tarball but do not load it
          --out <dir>          Directory for the output tarball (default: ./.firestream-build)
          --flake <ref>        Flake directory to build from (default: enclosing
                               Firestream repo, else the pinned firestream source)
          -h, --help           Show this help
        EOF
        }

        PKG=""
        TARGET_ARCH="$DEFAULT_ARCH"
        DO_LOAD=1
        OUT_DIR=""
        FLAKE_DIR_OVERRIDE=""

        while [ $# -gt 0 ]; do
          case "$1" in
            --target|--arch) TARGET_ARCH="$2"; shift 2 ;;
            --load)          DO_LOAD=1; shift ;;
            --no-load)       DO_LOAD=0; shift ;;
            --out)           OUT_DIR="$2"; shift 2 ;;
            --flake)         FLAKE_DIR_OVERRIDE="$2"; shift 2 ;;
            -h|--help)       usage; exit 0 ;;
            -*)              echo "Unknown option: $1" >&2; usage; exit 1 ;;
            *)               if [ -z "$PKG" ]; then PKG="$1"; else echo "Unexpected argument: $1" >&2; exit 1; fi; shift ;;
          esac
        done

        [ -n "$PKG" ] || { usage; exit 1; }
        command -v docker >/dev/null 2>&1 || { echo "ERROR: docker not found on PATH" >&2; exit 1; }

        # Map arch -> docker platform + per-arch persistent Nix store volume.
        case "$TARGET_ARCH" in
          x86_64|amd64)  PLATFORM="linux/amd64"; NIX_VOLUME="firestream-nix-store-amd64" ;;
          aarch64|arm64) PLATFORM="linux/arm64"; NIX_VOLUME="firestream-nix-store-arm64" ;;
          *)             PLATFORM="linux/$TARGET_ARCH"; NIX_VOLUME="firestream-nix-store-$TARGET_ARCH" ;;
        esac

        # Find the enclosing Firestream working tree (flake.nix + container dir).
        find_repo_root() {
          local dir
          dir="$(cd "$1" 2>/dev/null && pwd -P)" || return 1
          while [ "$dir" != "/" ]; do
            if [ -f "$dir/flake.nix" ] && [ -d "$dir/src/containers/firestream" ]; then
              echo "$dir"; return 0
            fi
            dir="$(dirname "$dir")"
          done
          return 1
        }

        if [ -n "$FLAKE_DIR_OVERRIDE" ]; then
          FLAKE_DIR="$FLAKE_DIR_OVERRIDE"
        elif FLAKE_DIR="$(find_repo_root "$PWD")"; then
          :
        else
          FLAKE_DIR="$SELF_STORE_PATH"
        fi

        OUT_DIR="''${OUT_DIR:-$PWD/.firestream-build}"
        mkdir -p "$OUT_DIR"

        BUILDER_IMAGE="nixos/nix:latest"
        echo ">>> Pulling builder ($PLATFORM)..." >&2
        docker pull --platform "$PLATFORM" "$BUILDER_IMAGE" >/dev/null 2>&1 || true

        MOUNTS=( --mount "type=volume,source=$NIX_VOLUME,target=/nix"
                 -v "$OUT_DIR:/out" )

        case "$FLAKE_DIR" in
          /nix/store/*)
            # Clean source snapshot (e.g. external consumer): mount at /flake.
            MOUNTS+=( -v "$FLAKE_DIR:/flake:ro" )
            WORKDIR="/flake"
            FLAKE_REF="/flake#$PKG"
            ;;
          *)
            # Live working tree: mount at its original path so worktree .git
            # pointers resolve, and add the worktree's git dirs (read-only).
            FLAKE_DIR="$(cd "$FLAKE_DIR" && pwd -P)"
            MOUNTS+=( -v "$FLAKE_DIR:$FLAKE_DIR:ro" )
            WORKDIR="$FLAKE_DIR"
            FLAKE_REF=".#$PKG"
            if [ -f "$FLAKE_DIR/.git" ]; then
              gitdir="$(sed 's/^gitdir: //' "$FLAKE_DIR/.git" | tr -d '\n\r')"
              case "$gitdir" in /*) : ;; *) gitdir="$FLAKE_DIR/$gitdir" ;; esac
              gitdir="$(cd "$gitdir" && pwd -P)"
              if [ -f "$gitdir/commondir" ]; then
                maindir="$(cd "$gitdir/$(tr -d '\n\r' < "$gitdir/commondir")" && pwd -P)"
              else
                maindir="$(cd "$gitdir/../.." && pwd -P)"
              fi
              MOUNTS+=( -v "$gitdir:$gitdir:ro" -v "$maindir:$maindir:ro" )
              echo ">>> Worktree detected; mounting git dirs" >&2
            fi
            ;;
        esac

        echo ">>> Building $PKG for $PLATFORM (flake: $FLAKE_DIR)..." >&2
        docker run --rm --platform "$PLATFORM" "''${MOUNTS[@]}" -w "$WORKDIR" \
          "$BUILDER_IMAGE" \
          sh -c '
            set -eu
            echo "experimental-features = nix-command flakes" >> /etc/nix/nix.conf
            git config --global --add safe.directory "*" 2>/dev/null || true
            nix build "'"$FLAKE_REF"'" -o /tmp/result -L --no-update-lock-file
            cp -L /tmp/result "/out/'"$PKG"'.tar.gz"
          '

        TARBALL="$OUT_DIR/$PKG.tar.gz"
        [ -s "$TARBALL" ] || { echo "ERROR: build produced no tarball: $TARBALL" >&2; exit 1; }
        echo ">>> Built: $TARBALL" >&2

        if [ "$DO_LOAD" -eq 1 ]; then
          echo ">>> Loading into Docker daemon..." >&2
          docker load < "$TARBALL"
        fi
      '';

      imageNames = builtins.attrNames config.firestreamImages;

      mkImageApp = name: {
        type = "app";
        program = "${pkgs.writeShellScriptBin "firestream-image-${name}" ''
          exec ${builder}/bin/firestream-build-image ${lib.escapeShellArg name} "$@"
        ''}/bin/firestream-image-${name}";
      };
    in
    {
      _module.args.firestreamBuildImage = builder;

      apps = {
        build-image = { type = "app"; program = "${builder}/bin/firestream-build-image"; };
      } // lib.listToAttrs (map (n: { name = "${n}-image"; value = mkImageApp n; }) imageNames);
    };
}
