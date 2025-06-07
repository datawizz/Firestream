#!/bin/bash

helm repo add ngrok https://ngrok.github.io/kubernetes-ingress-controller

helm upgrade --install ngrok-ingress-controller ngrok/kubernetes-ingress-controller \
  --namespace default \
  --create-namespace \
  --set credentials.apiKey=$NGROK_API_KEY \
  --set credentials.authtoken=$NGROK_AUTHTOKEN 

# kubectl apply -f /workspace/k8s/ingress/ingress.ngrok.yaml

# Apply the environment variables to the kustomization template, without writing to a file
# export SUBDOMAIN="your-unique-subdomain"
# export MAIN_DOMAIN="ngrok.app"
# cat overlays/production/kustomization.template.yaml | envsubst | kustomize build - | kubectl apply -f -
