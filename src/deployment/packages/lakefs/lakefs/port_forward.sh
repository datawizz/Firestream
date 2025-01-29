


# Wait for lakeFS to be ready, then port-forward to the lakeFS UI
export POD_NAME=$(kubectl get pods --namespace default -l "app.kubernetes.io/instance=lakefs" -o jsonpath="{.items[0].metadata.name}")
kubectl wait --for=condition=ready pod $POD_NAME
echo "Visit http://127.0.0.1:8000/setup to use your application"
kubectl port-forward $POD_NAME 8000:8000 --namespace default


# export POD_NAME=$(kubectl get pods --namespace default -l "app.kubernetes.io/instance=lakefs" -o jsonpath="{.items[0].metadata.name}")
# kubectl wait --for=condition=ready pod $POD_NAME
# echo "Visit http://127.0.0.1:8000/setup to use your application"
# kubectl port-forward $POD_NAME 8000:8000 --namespace default