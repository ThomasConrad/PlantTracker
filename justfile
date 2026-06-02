# Planty - Development Commands

# Default recipe - show help
default:
    @just --list

# === BUILD COMMANDS ===

# Build everything for production
build:
    @echo "🏗️ Building for production..."
    just generate
    just build-frontend
    just build-backend
    @echo "✅ Production build complete!"

build-release:
    @echo "🏗️ Building for release..."
    just generate
    just build-frontend-release
    just build-backend-release
    @echo "✅ Release build complete!"

# Build frontend
build-frontend:
    @echo "🏗️ Building frontend..."
    cd frontend && npm run build

# Build frontend for release (optimized)
build-frontend-release:
    @echo "🏗️ Building frontend for release..."
    cd frontend && npm run build

# Build backend
build-backend-release:
    just build
    @echo "🏗️ Building backend..."
    cd backend && cargo build --release

# Build backend for development
build-backend:
    @echo "🏗️ Building backend (dev)..."
    cd backend && cargo build

# === DEVELOPMENT COMMANDS ===

# Start development server with hot reload:
# - Vite (HMR) on :3000
# - Axum API (cargo-watch) on :3001
dev:
    @echo "🚀 Starting development server..."
    npm run dev

# Start frontend dev server only (Vite HMR)
frontend:
    @echo "⚡ Starting frontend (Vite HMR) on :3000..."
    cd frontend && npm run dev -- --port 3000 --strictPort

# Start production-like local preview:
# - Builds frontend (including PWA artifacts)
# - Runs Axum API on :3001
# - Runs Vite preview on :3000 with /api proxy
preview:
    @echo "🔎 Starting production-like preview..."
    cd frontend && npm run build
    npx concurrently \
        "just preview-backend" \
        "just preview-frontend"

# Run backend API for preview mode
preview-backend:
    @echo "🦀 Starting backend API on :3001..."
    cd backend && PORT=3001 FRONTEND_DIR=../frontend/dist cargo run --bin planty-api

# Run frontend preview server
preview-frontend:
    @echo "⚡ Starting frontend preview on :3000..."
    cd frontend && npm run preview -- --port 3000 --strictPort

# === RUN COMMANDS ===

# Run backend dev API only with auto-reload (cargo-watch)
backend:
    @echo "🦀 Starting backend API with cargo-watch on :3001..."
    cd backend && PORT=3001 FRONTEND_DIR=../frontend/.vite-dev cargo watch -x "run --bin planty-api"

run-release:
    @echo "🦀 Starting backend (release)..."
    just build-release
    cd backend && cargo run --release --bin planty-api

# === TEST COMMANDS ===

# Run all tests
test:
    @echo "🧪 Running all tests..."
    just test-backend
    just test-e2e

# Run frontend tests
test-frontend:
    @echo "🧪 Running frontend tests..."
    cd frontend && npm run test

# Run backend tests
test-backend:
    @echo "🧪 Running backend tests..."
    cd backend && cargo test

test-e2e:
    @echo "🧪 Running E2E tests in parallel..."
    just setup-e2e-env
    cd backend && venv-e2e/bin/pytest test_e2e_pytest.py -v -n auto

# Run E2E authentication tests
test-e2e-auth:
    @echo "🧪 Running authentication E2E tests..."
    just setup-e2e-env
    cd backend && venv-e2e/bin/pytest test_e2e_pytest.py -m auth -v  -n auto

# Run E2E plant management tests
test-e2e-plants:
    @echo "🧪 Running plant management E2E tests..."
    just setup-e2e-env
    cd backend && venv-e2e/bin/pytest test_e2e_pytest.py -m plants -v -n auto

# Run E2E calendar tests
test-e2e-calendar:
    @echo "🧪 Running calendar E2E tests..."
    just setup-e2e-env
    cd backend && venv-e2e/bin/pytest test_e2e_pytest.py -k "calendar" -v -n auto

# Run E2E tests with custom filter
test-e2e-filter filter:
    @echo "🧪 Running E2E tests with filter: {{filter}}"
    just setup-e2e-env
    cd backend && venv-e2e/bin/pytest test_e2e_pytest.py -k "{{filter}}" -v -n auto

# Run E2E isolation tests
test-e2e-isolation:
    @echo "🧪 Running user isolation E2E tests..."
    just setup-e2e-env
    cd backend && venv-e2e/bin/pytest test_e2e_pytest.py -m isolation -v -n auto

# === GENERATE COMMANDS ===

# Generate OpenAPI spec and TypeScript types
generate:
    @echo "🔄 Generating OpenAPI spec and TypeScript types..."
    cd backend && cargo build --bin generate-openapi
    cd backend && ./target/debug/generate-openapi > ../frontend/src/api/openapi.json
    cd frontend && npm run generate-types

# Generate types only
generate-types:
    @echo "🔄 Generating TypeScript types from OpenAPI spec..."
    cd frontend && npm run generate-types

# Generate OpenAPI spec only
generate-openapi:
    @echo "🔄 Generating OpenAPI spec..."
    cd backend && cargo build --bin generate-openapi
    cd backend && ./target/debug/generate-openapi > ../frontend/src/api/openapi.json

# === DATABASE COMMANDS ===

# Run database migrations
migrate:
    @echo "🗃️ Running database migrations..."
    cd backend && sqlx migrate run

# Create a new database migration
migrate-new name:
    @echo "🗃️ Creating new migration: {{name}}"
    cd backend && sqlx migrate add {{name}}

# Reset database (drop and recreate)
db-reset:
    @echo "🗃️ Resetting database..."
    cd backend && rm -f planty.db
    just migrate

# === UTILITY COMMANDS ===

# Setup Python environment for E2E tests
setup-e2e-env:
    #!/usr/bin/env bash
    echo "📦 Setting up E2E test environment..."
    cd backend
    if [ ! -d "venv-e2e" ]; then
        python3 -m venv venv-e2e
    fi
    source venv-e2e/bin/activate
    pip install -q -r requirements-e2e.txt
    echo "✅ E2E environment ready"

# Install all dependencies
install:
    @echo "📦 Installing all dependencies..."
    cd frontend && npm install

# Check code quality (type checking and linting)
check:
    @echo "🔍 Running code quality checks..."
    just typecheck
    just lint
    @echo "✅ All checks passed!"

# Type check the frontend
typecheck:
    @echo "🔍 Type checking frontend..."
    cd frontend && npm run typecheck

# Lint all code
lint:
    @echo "🔍 Linting all code..."
    cd frontend && npm run lint
    cd backend && cargo clippy -- -D warnings

# Format all code
format:
    @echo "✨ Formatting all code..."
    cd frontend && npm run format || true
    cd backend && cargo fmt

# Clean all build artifacts
clean:
    @echo "🧹 Cleaning all build artifacts..."
    rm -rf frontend/node_modules
    rm -rf frontend/dist
    rm -rf backend/target
    rm -rf backend/venv-e2e

# === DOCKER COMMANDS ===

# Build Docker image
docker-build:
    @echo "🐳 Building Docker image..."
    docker build -t planty .

# Run with Docker Compose
docker-up:
    @echo "🐳 Starting Planty via Docker Compose..."
    docker compose up -d

# Stop Docker Compose
docker-down:
    docker compose down

# View Docker logs
docker-logs:
    docker compose logs -f planty

# Rebuild and restart
docker-restart:
    docker compose down
    docker compose up -d --build

# Build, run, and smoke-test the Docker image
docker-test:
    @echo "🧪 Running Docker smoke tests..."
    ./test-docker.sh
