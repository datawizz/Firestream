#!/usr/bin/env bash
# ---------------------------------------------------------------------------
# Full local dev loop for the Firestream Airflow app on k3s / k3d.
#
# Builds the firestream-* images (including our CUSTOM airflow image whose
# guest-DAG dependencies are baked into a SEPARATE venv at
# /opt/firestream/airflow/dags-venv via config.airflow.dagWorkspace), SIDE-LOADS
# them into the cluster's containerd, then `helm upgrade --install`. Without the
# side-load every pod ImagePullBackOffs — the images are Nix-built and exist only
# on your machine, not a registry.
#
# Reads project values from ../config.nix. Run from examples/airflow-dag-dependencies/.
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

# Freshness gate: uv2nix builds from the committed lock, so a stale lock would
# bake the wrong guest venv. Fail fast before the (slow) image build.
if command -v uv >/dev/null 2>&1; then
  echo "==> Checking dags-workspace/uv.lock is fresh (uv lock --check)" >&2
  (cd dags-workspace && uv lock --check)
else
  echo "==> WARNING: uv not on PATH; skipping uv.lock freshness check" >&2
fi

# NIX_OVERRIDE (e.g. "--override-input firestream path:../..") is a SUBCOMMAND
# flag, so it must go AFTER the subcommand: `nix build --override-input ...`.
# shellcheck disable=SC2086  # NIX_OVERRIDE is intentionally word-split
nixx() { local sub="$1"; shift; nix "$sub" ${NIX_OVERRIDE:-} "$@"; }
nixval() { nixx eval --raw --impure --expr "(import ./config.nix).$1"; }

NS="$(nixval namespace)"
AIRFLOW_PW="$(nixval airflowPassword)"

# Build each image (a buildLayeredImage tarball) and load it into Docker,
# echoing the loaded "repo:tag" ref. (The package is a tarball, not a runnable
# app, so we use `nix build` + `docker load`, not `nix run -- --load`.)
load_image() {
  local pkg="$1"
  nixx build ".#$pkg" --out-link "result-$pkg" >&2
  docker load < "result-$pkg" | sed -n 's/^Loaded image: //p' | head -n1
}

echo "==> Building + loading images into Docker" >&2
AIRFLOW_REF="$(load_image airflow-image)"
PG_REF="$(load_image postgresql-image)"
REDIS_REF="$(load_image redis-image)"
echo "==> Loaded: $AIRFLOW_REF  $PG_REF  $REDIS_REF" >&2

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
import_image "$AIRFLOW_REF"
import_image "$PG_REF"
import_image "$REDIS_REF"

echo "==> Building customized chart bundle"
nixx build '.#airflow-chart' --out-link result

echo "==> Deploying Airflow (helm upgrade --install)"
./result/bin/deploy --namespace "$NS"

cat <<EOF

==> Done. Watch rollout (CeleryExecutor → several pods):
      kubectl -n $NS get pods -w

    Once Ready, expose the Airflow web UI:
      kubectl -n $NS port-forward svc/airflow-web 8080:8080

    Then open http://127.0.0.1:8080
      login: admin
      password: $AIRFLOW_PW

    The DAG appears as 'guest_dependencies' in the DAG list. Trigger it:
      - run_guest_cli    -> BashOperator calls the guest venv's console-script
      - run_guest_python -> @task.external_python imports the guest dep (cowsay)
EOF
