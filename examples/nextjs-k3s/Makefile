# Convenience targets for the Firestream Next.js local-k3s example.
# Project-specific values come from config.nix (single source of truth).

NS := $(shell nix eval --raw --impure --expr '(import ./config.nix).namespace')

.PHONY: help chart render deploy port-forward status destroy clean

help:
	@echo "Targets:"
	@echo "  chart        Build the customized nextjs chart bundle (.#nextjs-chart)"
	@echo "  render       Print the rendered Kubernetes manifests"
	@echo "  deploy       Build + side-load images (incl. baked app), then helm upgrade --install"
	@echo "  port-forward Forward localhost:3000 -> svc/nextjs (Ctrl-C to stop)"
	@echo "  status       Show pods / services / PVCs in the '$(NS)' namespace"
	@echo "  destroy      Delete the '$(NS)' namespace (release + PVCs) for a clean reinstall"
	@echo "  clean        Remove local nix result symlinks"

chart:
	nix build .#nextjs-chart --out-link result

render: chart
	cat result/rendered.yaml

deploy:
	bash scripts/deploy-local.sh

port-forward:
	kubectl -n $(NS) port-forward svc/nextjs 3000:3000

status:
	kubectl -n $(NS) get pods,svc,pvc

destroy:
	kubectl delete namespace $(NS)

clean:
	rm -f result result-*
