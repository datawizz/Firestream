#!/usr/bin/env bash
# ---------------------------------------------------------------------------
# Materialise the Kubernetes Secret the nextjs chart references, from values
# held in GCP Secret Manager. Idempotent: re-running updates the Secret in
# place (create --dry-run | apply).
#
# The chart references this by name (see ../config.nix + nextjs-overrides.nix):
#   nextjs-db-credentials   keys: password (app user), postgres-password (admin)
#
# Requires: gcloud (authenticated), kubectl (kubeconfig pointed at the target
# cluster — Cloud Build / scripts/deploy-local.sh run get-credentials first).
#
# Usage: sync-secrets.sh <namespace>
# ---------------------------------------------------------------------------
set -euo pipefail

NS="${1:-nextjs}"

access() {
  gcloud secrets versions access latest --secret="$1"
}

echo "Ensuring namespace '$NS' exists..." >&2
kubectl create namespace "$NS" --dry-run=client -o yaml | kubectl apply -f -

echo "Syncing nextjs-db-credentials..." >&2
kubectl create secret generic nextjs-db-credentials \
  --namespace "$NS" \
  --from-literal=password="$(access nextjs-db-password)" \
  --from-literal=postgres-password="$(access nextjs-db-postgres-password)" \
  --dry-run=client -o yaml | kubectl apply -f -

echo "Secret synced into namespace '$NS'." >&2
