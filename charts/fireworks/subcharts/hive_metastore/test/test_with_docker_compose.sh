

cd /workspace/charts/hive_metastore/docker && \
docker compose down && \
docker compose build && \
docker compose up -d && \
docker compose exec hive_metastore bash