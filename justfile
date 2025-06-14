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
    cd frontend && npm run build --mode production

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

# Start development server (builds frontend and runs backend)
dev:
    @echo "🚀 Starting development server..."
    just build-frontend
    cd backend && cargo run --bin planty-api

# === RUN COMMANDS ===

# Run backend (builds frontend first, then serves everything)
backend:
    @echo "🦀 Starting backend..."
    just build-frontend
    cd backend && cargo run --bin planty-api

run-release:
    @echo "🦀 Starting backend (release)..."
    just build-release
    cd backend && cargo run --release --bin planty-api

# === TEST COMMANDS ===

# Run all tests
test:
    @echo "🧪 Running all tests..."
    just test-backend
    just test-e2e-parallel

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