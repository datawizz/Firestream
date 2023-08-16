#!/bin/bash

helm repo add ngrok https://ngrok.github.io/kubernetes-ingress-controller

helm upgrade --install ngrok-ingress-controller ngrok/kubernetes-ingress-controller \
  --namespace default \
  --create-namespace \
  --set credentials.apiKey=$NGROK_API_KEY \
  --set credentials.authtoken=$NGROK_AUTHTOKEN 

# kubectl apply -f /workspace/k8s/ingress/ingress.ngrok.yaml