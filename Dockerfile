# syntax=docker/dockerfile:1
# ═══════════════════════════════════════════════════════════════════════════════
# Planty — Multi-stage Docker build
# Produces a minimal distroless image (~30MB + binary) with no shell or package manager
# Uses BuildKit cache mounts for fast rebuilds
# ═══════════════════════════════════════════════════════════════════════════════

# ─── Stage 1: Build frontend ──────────────────────────────────────────────────
FROM node:20-bookworm-slim AS frontend-builder

WORKDIR /app

# Install dependencies (cached via BuildKit mount)
COPY package.json package-lock.json ./
COPY frontend/package.json ./frontend/
RUN --mount=type=cache,target=/root/.npm \
    npm install && npm install --workspace=frontend rollup

# Copy frontend source and build
COPY frontend/ ./frontend/
RUN npm run --workspace=frontend build

# ─── Stage 2: Build backend ───────────────────────────────────────────────────
FROM rust:1.87-bookworm AS backend-builder

WORKDIR /app/backend

# Install system deps for SQLx (SQLite)
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config libssl-dev \
    && rm -rf /var/lib/apt/lists/*

ENV SQLX_OFFLINE=true

# Cache dependencies: copy manifests first
COPY backend/Cargo.toml backend/Cargo.lock ./
COPY backend/migrations/ ./migrations/
COPY backend/.sqlx/ ./.sqlx/

# Build deps with dummy source (cached unless Cargo.toml/lock changes)
RUN mkdir src && echo "fn main() {}" > src/main.rs && echo "" > src/lib.rs
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/backend/target \
    cargo build --release 2>/dev/null || true
RUN rm -rf src

# Copy actual source and build
COPY backend/src/ ./src/
RUN touch src/main.rs src/lib.rs
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/backend/target \
    cargo build --release && \
    cp target/release/planty-api /app/planty-api

# Create empty data dir for runtime stage
RUN mkdir -p /app/data && touch /app/data/.keep

# ─── Stage 3: Runtime ─────────────────────────────────────────────────────────
FROM gcr.io/distroless/cc-debian12 AS runtime

WORKDIR /app

# Copy compiled binary
COPY --from=backend-builder /app/planty-api ./planty-api

# Copy frontend assets
COPY --from=frontend-builder /app/frontend/dist ./frontend/dist

# Create data directory owned by nonroot (uid 65534)
# Distroless has no shell/mkdir, so copy empty dir from builder
COPY --from=backend-builder --chown=65534:65534 /app/data ./data

# Run as nonroot for security
USER 65534

# Environment defaults
ENV PORT=3000 \
    FRONTEND_DIR=/app/frontend/dist \
    DATABASE_URL=sqlite:/app/data/planty.db \
    RUST_LOG=info

EXPOSE 3000

ENTRYPOINT ["./planty-api"]
