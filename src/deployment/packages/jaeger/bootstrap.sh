

# ### Jaeger All-in-One ###
# helm repo add jaegertracing https://jaegertracing.github.io/helm-charts
# helm upgrade --install jaeger jaegertracing/jaeger \
#   --set provisionDataStore.cassandra=false \
#   --set storage.type=elasticsearch \
#   --set storage.elasticsearch.host="$OPENSEARCH_HOST" \
#   --set storage.elasticsearch.port="$OPENSEARCH_PORT" \
#   --set storage.elasticsearch.user="$OPENSEARCH_USERNAME" \
#   --set storage.elasticsearch.password="$OPENSEARCH_PASSWORD" \
#   --set ingester.enabled=true \
#   --set storage.kafka.brokers[0]="$KAFKA_BOOTSTRAP_SERVERS" \
#   --set storage.kafka.topic="jaeger-spans"
  # --debug --dry-run


  # --set provisionDataStore.cassandra=false \
  # --set provisionDataStore.elasticsearch=true \
  # --set storage.type=elasticsearch

# helm install jaeger jaegertracing/jaeger \
#   --set provisionDataStore.cassandra=false \
#   --set storage.type=elasticsearch \
#   --set storage.elasticsearch.host=<HOST> \
#   --set storage.elasticsearch.port=<PORT> \
#   --set storage.elasticsearch.user=<USER> \
#   --set storage.elasticsearch.password=<password>
