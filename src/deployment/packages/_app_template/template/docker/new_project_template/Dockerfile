# my-app/Dockerfile
FROM debian:11-slim

# Install any required packages
RUN apt-get update && \
    apt-get install -y curl && \
    rm -rf /var/lib/apt/lists/*

# Optionally copy in your application code
# COPY src/ /app/

# By default, just run a "Hello" command for demonstration
CMD ["echo", "Hello from my-app container!"]
