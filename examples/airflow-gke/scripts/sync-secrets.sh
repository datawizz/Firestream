#!/usr/bin/env bash
# ---------------------------------------------------------------------------
# Materialise the Kubernetes Secrets the Airflow chart references, from values
# held in GCP Secret Manager. Idempotent: re-running updates the Secrets in
# place (create --dry-run | apply).
#
# The chart references these by name (see ../config.nix + airflow-overrides.nix):
#   airflow-credentials        keys: airflow-password, airflow-fernet-key,
#                                    airflow-secret-key, airflow-jwt-secret-key
#   airflow-db-credentials     keys: password, postgres-password
#   airflow-redis-credentials  key:  redis-password
#
# Requires: gcloud (authenticated), kubectl (kubeconfig pointed at the target
# cluster — Cloud Build / scripts/deploy-local.sh run get-credentials first).
#
# Usage: sync-secrets.sh <namespace>
# ---------------------------------------------------------------------------
set -euo pipefail

NS="${1:-airflow}"

access() {
  gcloud secrets versions access latest --secret="$1"
}

echo "Ensuring namespace '$NS' exists..." >&2
kubectl create namespace "$NS" --dry-run=client -o yaml | kubectl apply -f -

echo "Syncing airflow-credentials..." >&2
kubectl create secret generic airflow-credentials \
  --namespace "$NS" \
  --from-literal=airflow-password="$(access airflow-password)" \
  --from-literal=airflow-fernet-key="$(access airflow-fernet-key)" \
  --from-literal=airflow-secret-key="$(access airflow-secret-key)" \
  --from-literal=airflow-jwt-secret-key="$(access airflow-jwt-secret-key)" \
  --dry-run=client -o yaml | kubectl apply -f -

echo "Syncing airflow-db-credentials..." >&2
kubectl create secret generic airflow-db-credentials \
  --namespace "$NS" \
  --from-literal=password="$(access airflow-db-password)" \
  --from-literal=postgres-password="$(access airflow-db-postgres-password)" \
  --dry-run=client -o yaml | kubectl apply -f -

echo "Syncing airflow-redis-credentials..." >&2
kubectl create secret generic airflow-redis-credentials \
  --namespace "$NS" \
  --from-literal=redis-password="$(access airflow-redis-password)" \
  --dry-run=client -o yaml | kubectl apply -f -

echo "Secrets synced into namespace '$NS'." >&2
