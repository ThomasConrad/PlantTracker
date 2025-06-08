# Plant Tracker API

Backend API for the Plant Tracker application, built with Rust and Axum.

## Setup

### Prerequisites
- Rust (latest stable)
- SQLite 3
- `sqlx-cli` for database migrations

Install sqlx-cli:
```bash
cargo install sqlx-cli
```

### Database Setup

1. **Environment Configuration**
   ```bash
   cp .env.example .env
   # Edit .env with your database configuration
   ```

2. **Create and Initialize Database**
   ```bash
   # Create the database
   sqlx database create
   
   # Run migrations
   sqlx migrate run
   
   # Generate query metadata for IDE support
   cargo sqlx prepare
   ```

3. **IDE Setup**
   For VS Code with rust-analyzer, the `.vscode/settings.json` file is already configured to:
   - Set the `DATABASE_URL` environment variable
   - Enable SQLite features
   - Configure SQLx extension support

### Development

```bash
# Run the application
cargo run

# Run with logging
RUST_LOG=debug cargo run

# Check code without running
cargo check

# Run tests
cargo test
```

### Database Operations

```bash
# Create a new migration
sqlx migrate add <migration_name>

# Run migrations
sqlx migrate run

# Revert last migration
sqlx migrate revert

# Generate fresh query metadata
cargo sqlx prepare
```

### API Endpoints

- `GET /api/v1/health` - Health check
- `POST /api/v1/plants` - Create a new plant
- `GET /api/v1/plants` - List plants (with pagination and search)
- `GET /api/v1/plants/:id` - Get a specific plant
- `PUT /api/v1/plants/:id` - Update a plant
- `DELETE /api/v1/plants/:id` - Delete a plant

See [DATABASE_SCHEMA.md](./DATABASE_SCHEMA.md) for complete database documentation.

### Environment Variables

- `DATABASE_URL` - Database connection string
- `PORT` - Server port (default: 3000)
- `FRONTEND_DIR` - Path to frontend build directory
- `RUST_LOG` - Logging level

### Features

- ✅ SQLite database with SQLx
- ✅ Comprehensive error handling
- ✅ Request validation
- ✅ CORS support
- ✅ Structured logging
- ✅ Database migrations
- ✅ Plant CRUD operations
- 🚧 User authentication (coming soon)
- 🚧 Photo uploads (coming soon)
- 🚧 Tracking entries (coming soon)