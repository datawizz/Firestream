# Convenience targets for the Firestream Odoo local-k3s example.
# Project-specific values come from config.nix (single source of truth).

NS := $(shell nix eval --raw --impure --expr '(import ./config.nix).namespace')

.PHONY: help chart render deploy port-forward status destroy clean

help:
	@echo "Targets:"
	@echo "  chart        Build the customized Odoo chart bundle (.#odoo-chart)"
	@echo "  render       Print the rendered Kubernetes manifests"
	@echo "  deploy       Build + side-load images, then helm upgrade --install (scripts/deploy-local.sh)"
	@echo "  port-forward Forward localhost:8069 -> svc/odoo (Ctrl-C to stop)"
	@echo "  status       Show pods / services / PVCs in the '$(NS)' namespace"
	@echo "  destroy      Delete the '$(NS)' namespace (release + PVCs) for a clean reinstall"
	@echo "  clean        Remove local nix result symlinks"

# --- Chart (no cluster needed) ---------------------------------------------
chart:
	nix build .#odoo-chart --out-link result

render: chart
	cat result/rendered.yaml

# --- Deploy ----------------------------------------------------------------
deploy:
	bash scripts/deploy-local.sh

port-forward:
	kubectl -n $(NS) port-forward svc/odoo 8069:80

status:
	kubectl -n $(NS) get pods,svc,pvc

# --- Teardown --------------------------------------------------------------
destroy:
	kubectl delete namespace $(NS)

clean:
	rm -f result result-*
