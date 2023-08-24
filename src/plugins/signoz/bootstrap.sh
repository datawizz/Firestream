helm repo add signoz https://charts.signoz.io


# kubectl create ns platform

helm --namespace default install signoz signoz/signoz


export POD_NAME=$(kubectl get pods --namespace default -l "app.kubernetes.io/name=signoz,app.kubernetes.io/instance=signoz,app.kubernetes.io/component=frontend" -o jsonpath="{.items[0].metadata.name}")
echo "Visit http://127.0.0.1:3301 to use your application"
kubectl --namespace default port-forward $POD_NAME 3301:3301