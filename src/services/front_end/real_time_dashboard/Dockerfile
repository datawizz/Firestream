FROM debian:bullseye-slim
ENV DEBIAN_FRONTEND=noninteractive

# Copy install scripts
COPY --from=localhost:5000/dependencies:latest /tmp/workspace/bin /tmp/library-scripts

# Install Node.JS
# TODO install from a common location in a upstream Docker stage
ARG NODE_VERSION=14
ENV NVM_DIR="/usr/local/share/nvm"
ENV NVM_SYMLINK_CURRENT=true \
    PATH=${NVM_DIR}/current/bin:${PATH}
RUN apt-get update && bash /tmp/library-scripts/node-debian.sh "${NVM_DIR}" "$NODE_VERSION"

# Cleanup
RUN rm -rf /tmp

# Copy App files
# TODO install from a common location in a upstream Docker stage
COPY services/javascript/dashboard /app
WORKDIR /app

# Install dependencies
# TODO install from a common location in a upstream Docker stage
RUN npm install

# Build the Next.js application
RUN npx next build

ENV PORT=8080

EXPOSE 8080
CMD ["npm", "start"]