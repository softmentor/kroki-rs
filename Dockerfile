ARG BASE_IMAGE=base

# Stage 1: Base - System dependencies
# This stage contains essential diagram tools and the headless browser environment.
FROM debian:bookworm-slim AS base
LABEL org.opencontainers.image.name="softmentor/kroki-rs-base"
LABEL org.opencontainers.image.description="Consolidated base image for kroki-rs (Graphviz, D2, Ditaa, and Headless Browser for Mermaid/BPMN)"

# Install core tools + Chromium for headless rendering (git for GHA checkout in container jobs)
RUN apt-get update && apt-get install -y \
    curl \
    wget \
    git \
    graphviz \
    ditaa \
    make \
    build-essential \
    fontconfig \
    fonts-liberation \
    chromium \
    libnss3 \
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
    libpangocairo-1.0-0 \
    libpango-1.0-0 \
    libcairo2 \
    --no-install-recommends && \
    rm -rf /var/lib/apt/lists/*

# Install D2
RUN curl -fsSL https://d2lang.com/install.sh | sh

WORKDIR /app

# Stage 2: CI - Development & Testing environment
FROM base AS ci
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"
# Install nextest for faster CI tests and cargo-chef for build optimization
RUN cargo install --locked cargo-nextest cargo-chef

# Stage 3: Planner (cargo-chef)
FROM rust:slim-bookworm AS chef
RUN cargo install cargo-chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# Stage 4: Builder
FROM chef AS builder
ARG FEATURES="native-browser"
COPY --from=planner /app/recipe.json recipe.json

# Build dependencies - leveraging BuildKit cache mounts
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/app/target \
    cargo chef cook --release --features "$FEATURES" --recipe-path recipe.json

# Now copy source and build the app
COPY . .
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/app/target \
    cargo build --release --features "$FEATURES" && \
    mkdir -p dist && cp target/release/kroki-rs dist/kroki-rs

# Stage 5: Final runtime image
FROM ${BASE_IMAGE}

ENV KROKI_PORT=8000
ENV KROKI_ADMIN_PORT=8081
ENV CHROME_BIN=/usr/bin/chromium
ENV JAVA_AWT_HEADLESS=true

WORKDIR /app

COPY --from=builder /app/dist/kroki-rs /usr/local/bin/kroki-rs

# Healthcheck to verify the binary is functional
HEALTHCHECK --interval=30s --timeout=5s --start-period=5s --retries=3 \
  CMD kroki-rs --version || exit 1

EXPOSE 8000 8081

CMD ["kroki-rs", "serve"]
