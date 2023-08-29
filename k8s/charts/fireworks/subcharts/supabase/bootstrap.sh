#!/bin/bash

bash /workspace/k8s/ingress/bootstrap.sh

# SUPABASE_ANON_KEY=""
# SUPABASE_SERVICE_KEY=""
# SUPABASE_SECRET=""
# SUPABASE_SERVICE_ROLE_KEY=""
SUPABASE_ANON_KEY="eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.ewogICAgInJvbGUiOiAiYW5vbiIsCiAgICAiaXNzIjogInN1cGFiYXNlIiwKICAgICJpYXQiOiAxNjc1NDAwNDAwLAogICAgImV4cCI6IDE4MzMxNjY4MDAKfQ.ztuiBzjaVoFHmoljUXWmnuDN6QU2WgJICeqwyzyZO88"
SUPABASE_SERVICE_KEY="eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.ewogICAgInJvbGUiOiAic2VydmljZV9yb2xlIiwKICAgICJpc3MiOiAic3VwYWJhc2UiLAogICAgImlhdCI6IDE2NzU0MDA0MDAsCiAgICAiZXhwIjogMTgzMzE2NjgwMAp9.qNsmXzz4tG7eqJPh1Y58DbtIlJBauwpqx39UF-MwM8k"
SUPABASE_SECRET="abcdefghijklmnopqrstuvwxyz123456"

SUPABASE_URL="demo.fireworks.ngrok.io"
INGRESS_CLASS="ngrok"


# Create JWT secret
kubectl -n default create secret generic demo-supabase-jwt \
  --from-literal=anonKey=$SUPABASE_ANON_KEY \
  --from-literal=serviceKey=$SUPABASE_SERVICE_KEY \
  --from-literal=secret=$SUPABASE_SECRET

# Create SMTP secret
kubectl -n default create secret generic demo-supabase-smtp \
  --from-literal=username='your-mail@example.com' \
  --from-literal=password='example123456'

# Create DB secret
kubectl -n default create secret generic demo-supabase-db \
  --from-literal=username='postgres' \
  --from-literal=password='example123456' 



helm upgrade --install supabase oci://registry-1.docker.io/bitnamicharts/supabase \
  --set global.jwt.existingSecret=demo-supabase-jwt \
  --set global.jwt.existingSecretKey=secret \
  --set global.jwt.existingSecretAnonKey=anonKey \
  --set global.jwt.existingSecretServiceKey=serviceKey \
  --set studio.ingress.enabled="true" \
  --set studio.ingress.ingressClassName=$INGRESS_CLASS \
  --set studio.ingress.hostname=$SUPABASE_URL \
  --set studio.ingress.tls="false" \
  --set studio.ingress.selfSigned="false" \
  --set kong.ingressController.enabled="false" \
  --set kong.ingress.enabled="true" \
  --set kong.ingress.hostname=$SUPABASE_URL \
  --set kong.ingress.tls="false" \
  --set kong.ingress.ingressClassName=$INGRESS_CLASS 
  # --set publicURL=$SUPABASE_URL \
  # --set studio.publicURL=$SUPABASE_URL \
  # --set kong.ingress.annotations="nginx.ingress.kubernetes.io/rewrite-target: /"