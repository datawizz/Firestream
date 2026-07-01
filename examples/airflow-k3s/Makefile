# Convenience targets for the Firestream Airflow local-k3s example.
# Project-specific values come from config.nix (single source of truth).

NS := $(shell nix eval --raw --impure --expr '(import ./config.nix).namespace')

.PHONY: help chart render deploy port-forward status destroy clean

help:
	@echo "Targets:"
	@echo "  chart        Build the customized Airflow chart bundle (.#airflow-chart)"
	@echo "  render       Print the rendered Kubernetes manifests"
	@echo "  deploy       Build + side-load images (incl. baked DAG), then helm upgrade --install"
	@echo "  port-forward Forward localhost:8080 -> svc/airflow-web (Ctrl-C to stop)"
	@echo "  status       Show pods / services / PVCs in the '$(NS)' namespace"
	@echo "  destroy      Delete the '$(NS)' namespace (release + PVCs) for a clean reinstall"
	@echo "  clean        Remove local nix result symlinks"

chart:
	nix build .#airflow-chart --out-link result

render: chart
	cat result/rendered.yaml

deploy:
	bash scripts/deploy-local.sh

port-forward:
	kubectl -n $(NS) port-forward svc/airflow-web 8080:8080

status:
	kubectl -n $(NS) get pods,svc,pvc

destroy:
	kubectl delete namespace $(NS)

clean:
	rm -f result result-*
