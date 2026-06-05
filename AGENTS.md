# AGENTS.md

This file provides repository guidance for working on Planty.

## Architecture Overview

Planty is a full-stack Progressive Web Application with a clear separation between frontend and backend:

- **Frontend**: SolidJS-based PWA with TypeScript, Tailwind CSS, and Vite
- **Backend**: Rust/Axum REST API with SQLite database and JWT authentication
- **Database**: SQLite with SQLx for type-safe database operations
- **Authentication**: Session-based authentication with JWT tokens and user isolation

### Key Architectural Patterns

- **API-First Design**: OpenAPI spec generates TypeScript types for frontend
- **User Isolation**: All plant data is scoped to authenticated users
- **Photo Management**: File uploads with thumbnail generation
- **Progressive Web App**: Installable with offline capabilities
- **Full-Stack Production Mode**: Backend serves built frontend assets

### State Management
- Frontend uses SolidJS stores for auth and plants data
- Backend uses SQLx with embedded migrations
- Session management via tower-sessions with database storage

## Essential Commands

### Development (Recommended: use Just)
- `just dev` - Start both frontend and backend in parallel
- `just dev-hot` - Start with hot-reload (requires cargo-watch)
- `just backend` - Start only backend API server (localhost:3000)

### Building and Production
- `just build` - Build both frontend and backend for production
- `just build-frontend` - Build only frontend (outputs to frontend/dist)
- `just backend` - Runs backend in production mode serving frontend

### Testing
- `just test` - Run all tests (frontend, backend, and E2E)
- `just test-backend` - Rust unit tests via `cargo test`
- `just test-e2e` - Python E2E tests via pytest (auto-sets up venv)
- `just test-e2e-auth` - Run only authentication E2E tests
- `just test-e2e-plants` - Run only plant management E2E tests

### Code Quality
- `just check` - Type checking and linting for all code
- `just lint` - ESLint (frontend) + cargo clippy (backend)
- `just format` - Prettier (frontend) + cargo fmt (backend)
- `just typecheck` - TypeScript type checking

### Database Operations
- `just db` - Start PostgreSQL via Docker
- `just migrate` - Run database migrations
- `just migrate-new <name>` - Create new migration file

### API Types Generation
- `just generate-types` - Generate OpenAPI spec and TypeScript types
- This must be run after backend schema changes

### Useful Utilities
- `just status` - Show project environment status
- `just health` - Quick build health check
- `just urls` - Display all development URLs
- `just install` - Install all dependencies

## Important File Locations

- **API Types**: `frontend/src/types/api-generated.ts` (auto-generated)
- **Database Migrations**: `backend/migrations/`
- **OpenAPI Spec**: Generated to `frontend/src/api/openapi.json`
- **Auth State**: `frontend/src/stores/auth.ts`
- **Plant State**: `frontend/src/stores/plants.ts`
- **API Handlers**: `backend/src/handlers/`

## Development Notes

- Always run `just generate-types` after modifying backend API schemas
- E2E tests automatically set up Python virtual environment
- Backend supports both development (separate servers) and production (integrated) modes
- File uploads are limited to 10MB with custom upload directory support
- Use `cargo watch -x run` in backend/ for auto-reload during Rust development

## Database Schema

The application uses SQLite with the following key tables:
- `users` - User accounts with bcrypt password hashing
- `plants` - Plant records with care schedules and custom metrics
- `tracking_entries` - Care activities (watering, fertilizing, custom metrics)
- `photos` - Plant photos with thumbnail support
- `sessions` - Authentication session storage

## Memories

- Testing Strategy: Use `--test foo bar` approach, expanding and iterating systematically to build comprehensive test coverage
- NEVER delete planty.db. The database contains user data. If migrations are out of sync, fix it by inserting records into `_sqlx_migrations` or by making the migration idempotent — never by wiping the database.
- When migrations have been manually applied but not recorded, the fix is: `INSERT INTO _sqlx_migrations (version, description, installed_on, success, checksum, execution_time) VALUES (<version>, '<name>', datetime('now'), 1, X'00', 0);`
- Write all new migrations to be idempotent where possible (use CREATE TABLE IF NOT EXISTS, or the CREATE+INSERT+DROP+RENAME pattern for ALTER TABLE in SQLite).
- SQLx offline mode used (`.sqlx/` directory); must run `DATABASE_URL="sqlite:planty.db" cargo sqlx prepare` after changes
- When adding fields to response structs (like PlantResponse), check test files that construct those structs manually (e.g. `src/utils/calendar.rs` tests) — they will fail to compile with missing fields.
- The backend binary is `planty-api` (not default): use `cargo run --bin planty-api` or build with `SQLX_OFFLINE=true cargo build --bin planty-api`.
- To build/check without a live database, set `SQLX_OFFLINE=true`. The `.sqlx/` directory provides cached query metadata.
- After modifying SQL queries (including in tests), regenerate the offline cache: `DATABASE_URL="sqlite:planty.db" cargo sqlx prepare -- --tests`
- The coach system prompt is in `backend/src/llm/mod.rs` (COACH_SYSTEM_PROMPT). It instructs the AI to respond with JSON only (text + suggestions + input_requests + extracted_facts). No markdown in responses.
- Frontend does NOT use markdown rendering for coach messages — plain text with `whitespace-pre-wrap`. The AI is told not to use markdown formatting.
- The identification prompt is in `backend/src/handlers/identify.rs` (IDENTIFY_SYSTEM_PROMPT).
