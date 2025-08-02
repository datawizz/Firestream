# Multi-stage Dockerfile for example_shop scraper
# Optimized for production use with Airflow

# Stage 1: Build
FROM node:18-alpine AS builder

WORKDIR /app

# Install build dependencies
RUN apk add --no-cache python3 make g++

# Copy package files
COPY package*.json ./
COPY tsconfig.json ./

# Install dependencies
RUN npm ci --only=production && \
    npm ci --only=development

# Copy source code
COPY lib/ ./lib/
COPY src/ ./src/

# Build TypeScript
RUN npm run build

# Remove dev dependencies
RUN npm prune --production

# Stage 2: Runtime
FROM node:18-alpine

# Install Chromium and dependencies
RUN apk update && \
    apk add --no-cache \
      chromium \
      nss \
      freetype \
      freetype-dev \
      harfbuzz \
      ca-certificates \
      ttf-freefont \
      dumb-init

# Tell Puppeteer to skip installing Chrome. We'll be using the installed package.
ENV PUPPETEER_SKIP_CHROMIUM_DOWNLOAD=true \
    PUPPETEER_EXECUTABLE_PATH=/usr/bin/chromium-browser

# Create app user
RUN addgroup -g 1001 -S appuser && \
    adduser -S appuser -u 1001

WORKDIR /app

# Copy built application
COPY --from=builder --chown=appuser:appuser /app/node_modules ./node_modules
COPY --from=builder --chown=appuser:appuser /app/dist ./dist
COPY --from=builder --chown=appuser:appuser /app/package*.json ./

# Create directories for screenshots and output
RUN mkdir -p /app/screenshots /app/output && \
    chown -R appuser:appuser /app

# Switch to non-root user
USER appuser

# Metadata
LABEL maintainer="" \
      description="example_shop web scraper for Airflow" \
      version="1.0.0"

# Environment variables
ENV NODE_ENV=production \
    WORKFLOW_ID=ecommerce_product_scraper \
    LOG_LEVEL=info

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
  CMD node -e "console.log('OK')" || exit 1

# Use dumb-init to handle signals properly
ENTRYPOINT ["dumb-init", "--"]

# Run the scraper
CMD ["node", "dist/index.js"]