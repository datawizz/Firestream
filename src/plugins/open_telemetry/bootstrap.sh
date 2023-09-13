helm repo add open-telemetry https://open-telemetry.github.io/opentelemetry-helm-charts

helm install opentelemetry-collector open-telemetry/opentelemetry-collector \
    --values src/plugins/open_telemetry/values.yaml