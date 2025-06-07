### Kafka ###
helm upgrade --install kafka bitnami/kafka --version 24.0.10  \
  --set controller.replicaCount=5 \
  --set controller.heapOpts="-Xmx1024m -Xms1024m" \
  --set controller.persistence.size=20Gi \
  --set listeners.client.protocol=PLAINTEXT \
  --set listeners.controller.protocol=PLAINTEXT \
  --set listeners.interbroker.protocol=PLAINTEXT \
  --set listeners.external.protocol=PLAINTEXT
