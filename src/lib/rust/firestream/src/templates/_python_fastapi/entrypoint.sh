#!/bin/bash
set -e

# Default values
PORT=${PORT:-8080}
HOST=${HOST:-0.0.0.0}
LOG_LEVEL=${LOG_LEVEL:-info}

echo "Starting FastAPI application..."
echo "Host: $HOST"
echo "Port: $PORT"
echo "Log Level: $LOG_LEVEL"

# Run the FastAPI application
exec python -m uvicorn src.main:app \
    --host "$HOST" \
    --port "$PORT" \
    --log-level "$LOG_LEVEL"
