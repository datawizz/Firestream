#!/bin/bash



# SUPABASE_ANON_KEY=""
# SUPABASE_SERVICE_KEY=""
# SUPABASE_SECRET=""
# SUPABASE_SERVICE_ROLE_KEY=""


SUPABASE_URL="demo.firestream.ngrok.io"
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
  --set kong.ingress.tls='false' 