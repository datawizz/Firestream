
### Minio ###
helm install minio bitnami/minio \
  --set auth.rootUser="$S3_LOCAL_ACCESS_KEY_ID" \
  --set auth.rootPassword="$S3_LOCAL_SECRET_ACCESS_KEY" \
  --set persistence.storageClass=local-path \
  --set persistence.accessMode=ReadWriteOnce \
  --set persistence.size=10Gi
  # --set defaultBuckets="$S3_LOCAL_BUCKET_NAME"
