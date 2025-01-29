FROM python:3.9-slim as base

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        chromium \
        libglib2.0-0 \
        libnss3 \
        libx11-6 && \
    rm -rf /var/lib/apt/lists/*

COPY . ./app
WORKDIR /app
RUN pip install -r requirements.txt

CMD ["sleep", "infinity"]
