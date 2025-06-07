


### Cillium ###

helm repo add cilium https://helm.cilium.io/

kubectl create namespace cilium

helm install cilium cilium/cilium --version 1.14 \
  --namespace cilium \
  --set kubeProxyReplacement=partial \
  --set hostServices.enabled=false \
  --set externalIPs.enabled=true \
  --set nodePort.enabled=true \
  --set hostPort.enabled=true \
  --set bpf.masquerade=false \
  --set image.pullPolicy=IfNotPresent \
  --set ipam.mode=kubernetes
