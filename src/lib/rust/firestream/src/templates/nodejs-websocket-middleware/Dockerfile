
FROM node:16-bullseye-slim as builder
RUN apt-get update && export DEBIAN_FRONTEND=noninteractive \
    && apt-get -y install --no-install-recommends \
    python3 \
    python3-pip \
    build-essential

WORKDIR /app
COPY . .
RUN npm install

FROM node:16-bullseye-slim AS runtime
# Install additional OS packages.
RUN apt update && export DEBIAN_FRONTEND=noninteractive \
    && apt -y install --no-install-recommends \
    librdkafka-dev \
    librdkafka++1 \
    librdkafka1 
WORKDIR /app
COPY --from=builder /app .
EXPOSE 8080
EXPOSE 8000
CMD [ "node", "src/server.js" ]
