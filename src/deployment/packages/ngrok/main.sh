#!/bin/bash

# Set required environment variables

export SUBDOMAIN="fire-11-configure-ngrok-for-local-development"


# # Add ngrok helm repository
helm repo add ngrok https://ngrok.github.io/kubernetes-ingress-controller

# # Install the ngrok ingress controller in its own namespace
helm install ngrok-ingress-controller ngrok/kubernetes-ingress-controller --version 0.8.0 \
  --set image.tag=0.4.0 \
  --namespace ngrok-ingress-controller \
  --create-namespace \
  --set credentials.apiKey=$NGROK_API_KEY \
  --set credentials.authtoken=$NGROK_AUTHTOKEN

# Wait and verify that the controller is running
echo "Waiting for the ngrok ingress controller to run..."
sleep 60
kubectl get pods -n ngrok-ingress-controller

cat <<EOF | kubectl apply -f -
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: metrics-server
  namespace: kube-system
spec:
  ingressClassName: ngrok
  rules:
  - host: metrics-server.$SUBDOMAIN.$NGROK_DOMAIN
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: metrics-server
            port:
              number: 443
EOF



# Deploy the 2048 game service
cat <<EOF | kubectl apply -f -
apiVersion: v1
kind: Service
metadata:
  name: game-2048
spec:
  ports:
    - name: http
      port: 80
      targetPort: 80
  selector:
    app: game-2048
---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: game-2048
spec:
  replicas: 1
  selector:
    matchLabels:
      app: game-2048
  template:
    metadata:
      labels:
        app: game-2048
    spec:
      containers:
        - name: backend
          image: alexwhen/docker-2048
          ports:
            - name: http
              containerPort: 80
EOF

# Verify the game deployment
echo "Verifying the 2048 game deployment..."
sleep 30
kubectl get pods

# Create an Ingress resource for the 2048 game service
cat <<EOF | kubectl apply -f -
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: game-2048
spec:
  ingressClassName: ngrok
  rules:
    - host: $SUBDOMAIN.$NGROK_DOMAIN
      http:
        paths:
          - path: /
            pathType: Prefix
            backend:
              service:
                name: game-2048
                port:
                  number: 80
EOF

# Provide instructions to open the game in a browser
echo "Open your game at: https://$SUBDOMAIN.$NGROK_DOMAIN"
