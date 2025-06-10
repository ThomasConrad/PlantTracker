# Plant Tracker - Development Commands
# Run `just --list` to see all available commands

# Default recipe - show help
default:
    @just --list

# Install all dependencies
install:
    @echo "📦 Installing all dependencies..."
    cd frontend && npm install
    @echo "✅ Dependencies installed!"

# Clean all build artifacts and dependencies
clean:
    @echo "🧹 Cleaning all build artifacts..."
    rm -rf frontend/node_modules
    rm -rf frontend/dist
    rm -rf backend/target
    rm -rf backend/uploads
    just clean-e2e
    @echo "✅ Clean complete!"

# Deep clean including lock files
clean-all: clean
    @echo "🧹 Deep cleaning with lock files..."
    rm -f frontend/package-lock.json
    rm -f backend/Cargo.lock
    @echo "✅ Deep clean complete!"

# Development - Run both frontend and backend
dev:
    @echo "🚀 Starting development servers..."
    just dev-parallel

# Run frontend and backend in parallel
dev-parallel:
    #!/usr/bin/env bash
    echo "🚀 Starting frontend and backend in parallel..."
    trap 'kill 0' SIGINT
    (cd backend && cargo run) &
    (cd frontend && npm run dev) &
    wait

# Run only the backend API server
backend:
    @echo "🦀 Starting Rust backend..."
    just build
    cd backend && cargo run --bin plant-tracker-api

# Run only the frontend development server
frontend:
    @echo "⚡ Starting frontend dev server..."
    cd frontend && npm run dev

# Build everything for production
build:
    @echo "🏗️ Building for production..."
    just generate-types
    just build-frontend
    just build-backend
    @echo "✅ Production build complete!"

# Build only the frontend
build-frontend:
    @echo "🏗️ Building frontend..."
    cd frontend && npm run build
    @echo "✅ Frontend build complete!"

# Build only the backend
build-backend:
    @echo "🏗️ Building backend..."
    cd backend && cargo build --release
    @echo "✅ Backend build complete!"

# Run all tests
test:
    @echo "🧪 Running all tests..."
    just test-frontend
    just test-backend
    just test-e2e
    @echo "✅ All tests passed!"

# Run only frontend tests
test-frontend:
    @echo "🧪 Running frontend tests..."
    cd frontend && npm run test

# Run only backend tests
test-backend:
    @echo "🧪 Running backend tests..."
    cd backend && cargo test

# Run end-to-end tests
test-e2e:
    @echo "🧪 Running end-to-end tests..."
    just setup-e2e-env
    cd backend && venv-e2e/bin/pytest test_e2e_pytest.py -v
    @echo "✅ E2E tests passed!"

# Setup Python environment for E2E tests
setup-e2e-env:
    #!/usr/bin/env bash
    echo "📦 Setting up E2E test environment..."
    cd backend
    # Create virtual environment if it doesn't exist
    if [ ! -d "venv-e2e" ]; then
        python3 -m venv venv-e2e
        echo "✅ Created Python virtual environment"
    fi
    # Activate and install dependencies
    source venv-e2e/bin/activate
    pip install -q -r requirements-e2e.txt
    echo "✅ E2E environment ready"

# Run E2E tests with specific markers
test-e2e-auth:
    @echo "🧪 Running authentication E2E tests..."
    just setup-e2e-env
    cd backend && venv-e2e/bin/pytest test_e2e_pytest.py -m auth -v

test-e2e-plants:
    @echo "🧪 Running plant management E2E tests..."
    just setup-e2e-env
    cd backend && venv-e2e/bin/pytest test_e2e_pytest.py -m plants -v

test-e2e-isolation:
    @echo "🧪 Running user isolation E2E tests..."
    just setup-e2e-env
    cd backend && venv-e2e/bin/pytest test_e2e_pytest.py -m isolation -v

# Run E2E tests in parallel (faster)
test-e2e-parallel:
    @echo "🧪 Running E2E tests in parallel..."
    just setup-e2e-env
    cd backend && venv-e2e/bin/pytest test_e2e_pytest.py -v -n auto

# Run E2E tests excluding slow tests
test-e2e-fast:
    @echo "🧪 Running fast E2E tests..."
    just setup-e2e-env
    cd backend && venv-e2e/bin/pytest test_e2e_pytest.py -v -m "not slow"

# Clean E2E test artifacts
clean-e2e:
    @echo "🧹 Cleaning E2E test artifacts..."
    cd backend && rm -rf venv-e2e test_plant_tracker.db
    @echo "✅ E2E cleanup complete!"

# Run frontend tests in watch mode
test-watch:
    @echo "🧪 Running frontend tests in watch mode..."
    cd frontend && npm run test:watch

# Type check the frontend
typecheck:
    @echo "🔍 Type checking frontend..."
    cd frontend && npm run typecheck
    @echo "✅ Type check passed!"

# Lint all code
lint:
    @echo "🔍 Linting all code..."
    just lint-frontend
    just lint-backend
    @echo "✅ Linting complete!"

# Lint only frontend
lint-frontend:
    @echo "🔍 Linting frontend..."
    cd frontend && npm run lint

# Lint only backend
lint-backend:
    @echo "🔍 Linting backend..."
    cd backend && cargo clippy -- -D warnings

# Format all code
format:
    @echo "✨ Formatting all code..."
    just format-frontend
    just format-backend
    @echo "✅ Formatting complete!"

# Format only frontend
format-frontend:
    @echo "✨ Formatting frontend..."
    cd frontend && npm run format || true

# Format only backend
format-backend:
    @echo "✨ Formatting backend..."
    cd backend && cargo fmt

# Check code quality (lint + typecheck)
check:
    @echo "🔍 Checking code quality..."
    just typecheck
    just lint
    @echo "✅ Quality check passed!"

# Fix linting issues automatically
fix:
    @echo "🔧 Fixing linting issues..."
    cd frontend && npm run lint -- --fix || true
    cd backend && cargo clippy --fix --allow-dirty --allow-staged || true
    just format
    @echo "✅ Auto-fix complete!"

# Start the database using Docker
db:
    @echo "🐘 Starting PostgreSQL database..."
    docker run --name plant-tracker-db \
        -e POSTGRES_DB=plant_tracker \
        -e POSTGRES_USER=plant_user \
        -e POSTGRES_PASSWORD=plant_password \
        -p 5432:5432 -d postgres:15 || docker start plant-tracker-db
    @echo "✅ Database started on port 5432"

# Stop the database
db-stop:
    @echo "🐘 Stopping PostgreSQL database..."
    docker stop plant-tracker-db || true
    @echo "✅ Database stopped"

# Reset the database (remove and recreate)
db-reset:
    @echo "🐘 Resetting database..."
    docker stop plant-tracker-db || true
    docker rm plant-tracker-db || true
    just db
    @echo "✅ Database reset complete"

# Run database migrations
migrate:
    @echo "🔄 Running database migrations..."
    cd backend && sqlx migrate run
    @echo "✅ Migrations complete!"

# Create a new database migration
migrate-new name:
    @echo "🔄 Creating new migration: {{name}}"
    cd backend && sqlx migrate add {{name}}

# Docker commands
docker-build:
    @echo "🐳 Building Docker images..."
    docker-compose build
    @echo "✅ Docker build complete!"

# Start all services with Docker
docker-up:
    @echo "🐳 Starting Docker services..."
    docker-compose up -d
    @echo "✅ Docker services started!"
    @echo "📱 Frontend: http://localhost:5173"
    @echo "🦀 Backend: http://localhost:3000"
    @echo "🐘 Database: localhost:5432"
    @echo "🔧 Adminer: http://localhost:8080"

# Stop all Docker services
docker-down:
    @echo "🐳 Stopping Docker services..."
    docker-compose down
    @echo "✅ Docker services stopped!"

# View Docker logs
docker-logs:
    @echo "📋 Viewing Docker logs..."
    docker-compose logs -f

# Restart Docker services
docker-restart:
    @echo "🐳 Restarting Docker services..."
    docker-compose restart
    @echo "✅ Docker services restarted!"

# Production deployment preparation
deploy-prep:
    @echo "🚀 Preparing for deployment..."
    just clean
    just install
    just check
    just test-backend
    just test-e2e-fast
    just build
    @echo "✅ Ready for deployment!"

# Watch for changes and rebuild frontend
watch:
    @echo "👀 Watching for frontend changes..."
    cd frontend && npm run dev

# Generate OpenAPI spec and TypeScript types
generate-types:
    @echo "🔄 Generating OpenAPI spec and TypeScript types..."
    cd backend && cargo build --bin generate-openapi
    cd backend && ./target/debug/generate-openapi > ../frontend/src/api/openapi.json
    cd frontend && npm run generate-types
    @echo "✅ OpenAPI spec and TypeScript types generated!"

# Run security audit
audit:
    @echo "🔒 Running security audit..."
    cd frontend && npm audit
    cd backend && cargo audit || echo "Install cargo-audit with: cargo install cargo-audit"

# Update dependencies
update:
    @echo "⬆️  Updating dependencies..."
    cd frontend && npm update
    cd backend && cargo update
    @echo "✅ Dependencies updated!"

# Show project status
status:
    @echo "📊 Project Status"
    @echo "=================="
    @echo ""
    @echo "📁 Frontend:"
    @cd frontend && echo "   Node: $(node --version)" && echo "   NPM: $(npm --version)"
    @echo ""
    @echo "🦀 Backend:"
    @cd backend && echo "   Rust: $(rustc --version)" && echo "   Cargo: $(cargo --version)"
    @echo ""
    @echo "🐳 Docker:"
    @docker --version 2>/dev/null || echo "   Docker: Not installed"
    @echo ""
    @echo "📦 Dependencies:"
    @test -d frontend/node_modules && echo "   ✅ Frontend dependencies installed" || echo "   ❌ Frontend dependencies missing (run: just install)"
    @test -f backend/Cargo.lock && echo "   ✅ Backend dependencies resolved" || echo "   ❌ Backend dependencies not resolved (run: just backend)"

# Development server with hot reload for both frontend and backend
dev-hot:
    @echo "🔥 Starting hot-reload development..."
    #!/usr/bin/env bash
    trap 'kill 0' SIGINT
    echo "🦀 Starting backend with cargo-watch..."
    (cd backend && cargo watch -x run) &
    echo "⚡ Starting frontend with Vite..."
    (cd frontend && npm run dev) &
    wait

# Install development tools
install-tools:
    @echo "🛠️ Installing development tools..."
    cargo install cargo-watch sqlx-cli cargo-audit 2>/dev/null || true
    @echo "✅ Development tools installed!"
    @echo "   - cargo-watch: Auto-rebuild on file changes"
    @echo "   - sqlx-cli: Database migration tool"
    @echo "   - cargo-audit: Security vulnerability scanner"

# Quick health check
health:
    @echo "🏥 Running health checks..."
    @echo "Frontend build..." && cd frontend && npm run build > /dev/null && echo "✅ Frontend OK" || echo "❌ Frontend Failed"
    @echo "Backend build..." && cd backend && cargo check > /dev/null && echo "✅ Backend OK" || echo "❌ Backend Failed"
    @echo "TypeScript..." && cd frontend && npm run typecheck > /dev/null && echo "✅ TypeScript OK" || echo "❌ TypeScript Failed"

# Create a new release
release version:
    @echo "🚀 Creating release {{version}}..."
    git tag -a v{{version}} -m "Release v{{version}}"
    @echo "✅ Release v{{version}} tagged!"
    @echo "📝 Push with: git push origin v{{version}}"

# Show useful URLs for development
urls:
    @echo "🔗 Development URLs"
    @echo "==================="
    @echo "📱 Frontend (dev):     http://localhost:5173"
    @echo "🦀 Backend API:        http://localhost:3000"
    @echo "📚 API Health:         http://localhost:3000/"
    @echo "🐘 Database:           localhost:5432"
    @echo "🔧 DB Admin (Docker):  http://localhost:8080"
    @echo ""
    @echo "🐳 Docker URLs (when running docker-compose):"
    @echo "📱 Frontend:           http://localhost:5173"
    @echo "🦀 Backend:            http://localhost:3000"
    @echo "🔧 Adminer:            http://localhost:8080"