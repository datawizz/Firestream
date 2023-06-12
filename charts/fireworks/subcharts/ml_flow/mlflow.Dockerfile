



FROM python:3.8-slim-buster

# Build dependencies
RUN apt-get update -y \
    && apt-get install -y --no-install-recommends \
    build-essential libpq-dev

# Pip dependencies
RUN pip install psycopg2-binary==2.8.5 && \
    pip install mlflow[extras]==2.0.1 \
    pip install boto3

ENV BACKEND_STORE_URI=""
ENV DEFAULT_ARTIFACT_ROOT="/opt/artifact"

# Port
EXPOSE 5555

CMD mlflow server --host 0.0.0.0 --port 5555 
# --backend-store-uri $BACKEND_STORE_URI --default-artifact-root $DEFAULT_ARTIFACT_ROOT