#!/usr/bin/env bash
# ---------------------------------------------------------------------------
# Full local dev loop for the Firestream Next.js app on k3s / k3d.
#
# Builds the firestream-* images (including our CUSTOM nextjs image with the
# baked app), SIDE-LOADS them into the cluster's containerd, then
# `helm upgrade --install`. Without the side-load every pod ImagePullBackOffs —
# the images are Nix-built and exist only on your machine, not a registry.
#
# Reads project values from ../config.nix. Run from examples/nextjs-k3s/.
#
# Targets (auto-detected):
#   - host k3s (default): docker save <ref> | k3s ctr -n k8s.io images import -
#   - k3d:                k3d image import <ref> -c "$K3D_CLUSTER"   (set K3D_CLUSTER)
#
# Prereqs: nix, docker, kubectl, helm, and EITHER host k3s (membership in the
# `k3s` group — no sudo) OR a k3d cluster (set K3D_CLUSTER=<name>).
#
# In-repo testing before this lands on `main` (the flake pins github):
#   NIX_OVERRIDE="--override-input firestream path:../.." bash scripts/deploy-local.sh
# ---------------------------------------------------------------------------
set -euo pipefail

cd "$(dirname "$0")/.."

# NIX_OVERRIDE (e.g. "--override-input firestream path:../..") is a SUBCOMMAND
# flag, so it must go AFTER the subcommand: `nix build --override-input ...`.
# shellcheck disable=SC2086  # NIX_OVERRIDE is intentionally word-split
nixx() { local sub="$1"; shift; nix "$sub" ${NIX_OVERRIDE:-} "$@"; }
nixval() { nixx eval --raw --impure --expr "(import ./config.nix).$1"; }

NS="$(nixval namespace)"

# Build each image (a buildLayeredImage tarball) and load it into Docker,
# echoing the loaded "repo:tag" ref.
load_image() {
  local pkg="$1"
  nixx build ".#$pkg" --out-link "result-$pkg" >&2
  docker load < "result-$pkg" | sed -n 's/^Loaded image: //p' | head -n1
}

echo "==> Building + loading images into Docker" >&2
NEXTJS_REF="$(load_image nextjs-image)"
PG_REF="$(load_image postgresql-image)"
echo "==> Loaded: $NEXTJS_REF  $PG_REF" >&2

# --- Side-load into the cluster's containerd --------------------------------
import_image() {
  local ref="$1"
  if [ -n "${K3D_CLUSTER:-}" ]; then
    echo "==> Importing $ref into k3d cluster '$K3D_CLUSTER'"
    k3d image import "$ref" -c "$K3D_CLUSTER"
  else
    echo "==> Importing $ref into host k3s containerd"
    if command -v k3s >/dev/null 2>&1; then
      docker save "$ref" | k3s ctr -n k8s.io images import -
    else
      docker save "$ref" | ctr -a /run/k3s/containerd/containerd.sock -n k8s.io images import -
    fi
  fi
}
import_image "$NEXTJS_REF"
import_image "$PG_REF"

echo "==> Building customized chart bundle"
nixx build '.#nextjs-chart' --out-link result

echo "==> Deploying Next.js (helm upgrade --install)"
./result/bin/deploy --namespace "$NS"

cat <<EOF

==> Done. Watch rollout (nextjs + bundled postgresql):
      kubectl -n $NS get pods -w

    Once Ready, expose the app:
      kubectl -n $NS port-forward svc/nextjs 3000:3000

    Then open http://127.0.0.1:3000
      (the homepage runs a live SELECT against firestream-postgresql)
EOF
