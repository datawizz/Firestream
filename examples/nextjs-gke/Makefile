# Convenience targets for the Firestream Next.js GKE example.
# Project-specific values come from config.nix (single source of truth).

NAMESPACE := $(shell nix eval --raw --impure --expr '(import ./config.nix).namespace')

.PHONY: help chart render pulumi-preview pulumi-up images deploy destroy clean

help:
	@echo "Targets:"
	@echo "  chart           Build the customized nextjs chart bundle (.#nextjs-chart)"
	@echo "  render          Print the rendered Kubernetes manifests"
	@echo "  pulumi-preview  Preview the GKE/AR/Secret-Manager infrastructure"
	@echo "  pulumi-up       Provision the infrastructure (billable)"
	@echo "  images          Build + push firestream-* images + deploy (scripts/deploy-local.sh)"
	@echo "  deploy          Alias for 'images' (full local deploy)"
	@echo "  destroy         Tear down the Pulumi infrastructure"
	@echo "  clean           Remove local nix result symlinks"

# --- Chart (no cluster needed) ---------------------------------------------
chart:
	nix build .#nextjs-chart --out-link result

render: chart
	cat result/rendered.yaml

# --- Infrastructure (Pulumi) -----------------------------------------------
pulumi-preview:
	cd pulumi && pulumi preview

pulumi-up:
	cd pulumi && pulumi up

destroy:
	cd pulumi && pulumi destroy

# --- Deploy ----------------------------------------------------------------
images deploy:
	bash scripts/deploy-local.sh

clean:
	rm -f result result-* *.tar.gz
