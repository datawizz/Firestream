FROM debian:bullseye-slim
ENV DEBIAN_FRONTEND=noninteractive

# Copy install scripts
COPY --from=localhost:5000/dependencies:latest /tmp/library-scripts /tmp/library-scripts

# Install Node.JS
ENV NVM_DIR="/usr/local/share/nvm"
ENV NVM_SYMLINK_CURRENT=true \
    PATH=${NVM_DIR}/current/bin:${PATH}
RUN apt-get update && bash /tmp/library-scripts/node-debian.sh "${NVM_DIR}"

# Cleanup
RUN rm -rf /tmp

# Copy App files
COPY ./services/javascript/websocket_middleware /app

WORKDIR /app

# Install dependencies
RUN npm install

EXPOSE 8080

CMD ["node", "src/server.js"]