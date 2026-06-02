# ═══════════════════════════════════════════════════════════════════════════════
# Planty — Multi-stage Docker build
# Produces a minimal image (~100MB) with the compiled Rust binary + frontend assets
# ═══════════════════════════════════════════════════════════════════════════════

# ─── Stage 1: Build frontend ──────────────────────────────────────────────────
FROM node:20-alpine AS frontend-builder

WORKDIR /app

# Install dependencies first (cacheable layer) — monorepo workspace layout
COPY package.json package-lock.json ./
COPY frontend/package.json ./frontend/
RUN npm ci --ignore-scripts

# Copy frontend source and build
COPY frontend/ ./frontend/
RUN npm run --workspace=frontend build

# ─── Stage 2: Build backend ───────────────────────────────────────────────────
FROM rust:1.82-bookworm AS backend-builder

WORKDIR /app/backend

# Install system deps for SQLx (SQLite)
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Cache dependencies: copy manifests first
COPY backend/Cargo.toml backend/Cargo.lock ./
COPY backend/migrations/ ./migrations/
COPY backend/.sqlx/ ./.sqlx/

# Create a dummy main.rs to build deps
RUN mkdir src && echo "fn main() {}" > src/main.rs && echo "" > src/lib.rs
ENV SQLX_OFFLINE=true
RUN cargo build --release 2>/dev/null || true
RUN rm -rf src

# Copy actual source and rebuild
COPY backend/src/ ./src/
# Touch main.rs so cargo knows it changed
RUN touch src/main.rs src/lib.rs
RUN cargo build --release

# ─── Stage 3: Runtime ─────────────────────────────────────────────────────────
FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates curl \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -ms /bin/sh planty

WORKDIR /app

# Copy compiled binary
COPY --from=backend-builder /app/backend/target/release/planty-api ./planty-api

# Copy frontend assets
COPY --from=frontend-builder /app/frontend/dist ./frontend/dist

# Create data directory for SQLite
RUN mkdir -p /app/data && chown planty:planty /app/data

USER planty

# Environment defaults
ENV PORT=3000 \
    FRONTEND_DIR=/app/frontend/dist \
    DATABASE_URL=sqlite:/app/data/planty.db \
    RUST_LOG=info

EXPOSE 3000

# Health check
HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD curl -f http://localhost:3000/api/health || exit 1

ENTRYPOINT ["./planty-api"]
