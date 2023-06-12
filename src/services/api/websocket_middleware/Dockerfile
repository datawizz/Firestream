
ARG BASE_IMAGE
FROM ${BASE_IMAGE} as builder


FROM debian:bullseye-slim
ENV DEBIAN_FRONTEND=noninteractive
WORKDIR /app

# Copy install scripts
COPY --from=builder /tmp/workspace/bin/install_scripts /tmp/install_scripts

# Install Node.JS
ARG NODE_VERSION=14
ENV NVM_DIR="/usr/local/share/nvm"
ENV NVM_SYMLINK_CURRENT=true \
    PATH=${NVM_DIR}/current/bin:${PATH}
RUN apt-get update && bash /tmp/install_scripts/node-debian.sh "${NVM_DIR}" "$NODE_VERSION"

# Cleanup
RUN rm -rf /tmp

# Copy App files
COPY . .

# Install dependencies
# TODO pull these from builder
RUN npm install

EXPOSE 8080

CMD ["node", "src/server.js"]