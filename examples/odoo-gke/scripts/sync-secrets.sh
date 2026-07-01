#!/usr/bin/env bash
# ---------------------------------------------------------------------------
# Materialise the Kubernetes Secrets the Odoo chart references, from values
# held in GCP Secret Manager. Idempotent: re-running updates the Secrets in
# place (create --dry-run | apply).
#
# The chart references these by name (see ../config.nix + odoo-overrides.nix):
#   odoo-credentials     key: odoo-password
#   odoo-db-credentials  keys: password, postgres-password
#
# Requires: gcloud (authenticated), kubectl (kubeconfig pointed at the target
# cluster — Cloud Build / scripts/deploy-local.sh run get-credentials first).
#
# Usage: sync-secrets.sh <namespace>
# ---------------------------------------------------------------------------
set -euo pipefail

NS="${1:-odoo}"

access() {
  gcloud secrets versions access latest --secret="$1"
}

echo "Ensuring namespace '$NS' exists..." >&2
kubectl create namespace "$NS" --dry-run=client -o yaml | kubectl apply -f -

echo "Syncing odoo-credentials..." >&2
kubectl create secret generic odoo-credentials \
  --namespace "$NS" \
  --from-literal=odoo-password="$(access odoo-password)" \
  --dry-run=client -o yaml | kubectl apply -f -

echo "Syncing odoo-db-credentials..." >&2
kubectl create secret generic odoo-db-credentials \
  --namespace "$NS" \
  --from-literal=password="$(access odoo-db-password)" \
  --from-literal=postgres-password="$(access odoo-db-postgres-password)" \
  --dry-run=client -o yaml | kubectl apply -f -

echo "Secrets synced into namespace '$NS'." >&2
