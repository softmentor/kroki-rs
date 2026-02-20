ARG BASE_IMAGE=base

# Stage 1: Base - System and Node.js dependencies
# This stage can be pre-built and pushed to GHCR to accelerate CI
FROM debian:bookworm-slim AS base
LABEL org.opencontainers.image.description="Base image for kroki-rs containing all diagram tool dependencies"

RUN apt-get update && apt-get install -y \
    curl \
    wget \
    graphviz \
    ditaa \
    plantuml \
    make \
    build-essential \
    python3 \
    default-jre \
    libcairo2-dev \
    libpango1.0-dev \
    libjpeg-dev \
    libgif-dev \
    librsvg2-dev \
    chromium \
    libnss3 \
    libnspr4 \
    libatk1.0-0 \
    libatk-bridge2.0-0 \
    libcups2 \
    libdrm2 \
    libxcomposite1 \
    libxdamage1 \
    libxext6 \
    libxfixes3 \
    libxrandr2 \
    libgbm1 \
    libasound2 \
    libxshmfence1 \
    fontconfig \
    fonts-liberation \
    --no-install-recommends && \
    rm -rf /var/lib/apt/lists/*

# Install modern Node.js 22.x and D2
RUN curl -fsSL https://deb.nodesource.com/setup_22.x | bash - && \
    apt-get install -y nodejs --no-install-recommends && \
    rm -rf /var/lib/apt/lists/* && \
    ln -s /usr/bin/chromium /usr/bin/chromium-browser || true && \
    curl -fsSL https://d2lang.com/install.sh | sh

WORKDIR /app

# Install isolated Node.js dependencies (Puppeteer/Playwright tools)
# This is also part of the base as it changes infrequently
COPY package.json package-lock.json* ./
RUN npm install --omit=dev --no-audit --no-fund && npm cache clean --force

# Link NPM binaries to /usr/local/bin for global reach
RUN ln -s /app/node_modules/.bin/* /usr/local/bin/ || true

# Stage 2: Builder (selective binary injection or full build)
FROM rust:slim-bookworm AS builder
ARG TARGETARCH
WORKDIR /app

# Copy all files (respecting .dockerignore)
COPY . .

# 1. First, check if there's an arch-specific binary in dist-amd64/ or dist-arm64/
# 2. Second, check if there's a generic binary in dist/
# 3. Finally, build from source if nothing found
RUN if [ "$TARGETARCH" = "amd64" ] && [ -f ./dist-amd64/kroki-rs-linux-amd64 ]; then \
      echo "Injecting pre-built Linux AMD64 binary..."; \
      mkdir -p dist && cp ./dist-amd64/kroki-rs-linux-amd64 dist/kroki-rs; \
    elif [ "$TARGETARCH" = "arm64" ] && [ -f ./dist-arm64/kroki-rs-linux-arm64 ]; then \
      echo "Injecting pre-built Linux ARM64 binary..."; \
      mkdir -p dist && cp ./dist-arm64/kroki-rs-linux-arm64 dist/kroki-rs; \
    elif [ -f ./dist/kroki-rs ]; then \
      echo "Using generic pre-built binary in ./dist/kroki-rs"; \
    else \
      echo "No pre-built binary found for $TARGETARCH. Building from source..."; \
      apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/* && \
      cargo build --release && \
      mkdir -p dist && cp target/release/kroki-rs dist/kroki-rs; \
    fi

# Stage 3: Final runtime image
FROM ${BASE_IMAGE}

# Set up environment variables
ENV PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD=1
ENV PLAYWRIGHT_EXECUTABLE_PATH=/usr/bin/chromium
ENV KROKI_PORT=8000
ENV KROKI_ADMIN_PORT=8081

WORKDIR /app

# Copy binary from builder stage
COPY --from=builder /app/dist/kroki-rs /usr/local/bin/kroki-rs

# Copy the Playwright worker script
# The server expects it at ./src/browser/worker.js relative to the binary or WORKDIR
COPY src/browser ./src/browser

# Expose ports for Server API and Admin API
EXPOSE 8000 8081

# Run the server on 0.0.0.0 (internal binding)
CMD ["kroki-rs", "serve"]
