#!/usr/bin/env bash
# ---------------------------------------------------------------------------
# Local equivalent of cloudbuild.yaml, for the dev loop. Builds + pushes the
# images, builds the customized chart bundle, authenticates kubectl to the GKE
# cluster, syncs Secrets from Secret Manager, and deploys.
#
# Reads project-specific values straight out of ../config.nix so there is a
# single source of truth. Run from examples/odoo/ (or anywhere — it cd's to its
# own dir).
#
# Prereqs: nix, docker, gcloud (authenticated), kubectl, helm.
# ---------------------------------------------------------------------------
set -euo pipefail

cd "$(dirname "$0")/.."

# Pull scalars out of config.nix via nix eval so we never duplicate them here.
nixval() { nix eval --raw --impure --expr "(import ./config.nix).$1"; }

PROJECT_ID="$(nixval projectId)"
REGION="$(nixval region)"
CLUSTER="$(nixval clusterName)"
NAMESPACE="$(nixval namespace)"
AR_HOST="$(nixval arHost)"
AR_REPO="$(nixval arRepo)"
IMAGE_TAG="$(nixval imageTag)"

echo "==> Authenticating Docker to Artifact Registry ($AR_HOST)"
gcloud auth configure-docker "$AR_HOST" --quiet

push_image() {
  local pkg="$1" name="$2"
  echo "==> Building $name image (nix build .#$pkg)"
  nix build ".#$pkg" --out-link "result-$pkg"
  local loaded
  loaded="$(docker load < "result-$pkg" | sed -n 's/^Loaded image: //p' | head -n1)"
  echo "==> Loaded $loaded; tagging + pushing"
  docker tag "$loaded" "$AR_HOST/$AR_REPO/$name:$IMAGE_TAG"
  docker push "$AR_HOST/$AR_REPO/$name:$IMAGE_TAG"
}

push_image odoo-image firestream-odoo
push_image postgresql-image firestream-postgresql

echo "==> Building customized chart bundle (nix build .#odoo-chart)"
nix build ".#odoo-chart" --out-link result

echo "==> Fetching GKE credentials for $CLUSTER ($REGION)"
gcloud container clusters get-credentials "$CLUSTER" \
  --region "$REGION" --project "$PROJECT_ID"

echo "==> Syncing Secrets from Secret Manager"
scripts/sync-secrets.sh "$NAMESPACE"

echo "==> Deploying Odoo (helm upgrade --install)"
./result/bin/deploy --namespace "$NAMESPACE"

echo "==> Done. Watch rollout with: kubectl -n $NAMESPACE get pods -w"
