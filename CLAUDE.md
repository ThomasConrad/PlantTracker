# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Architecture Overview

Planty is a full-stack Progressive Web Application with a clear separation between frontend and backend:

- **Frontend**: SolidJS-based PWA with TypeScript, Tailwind CSS, and Vite
- **Backend**: Rust/Axum REST API with PostgreSQL database and JWT authentication
- **Database**: PostgreSQL with SQLx for type-safe database operations
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
- `just frontend` - Start only frontend dev server (localhost:5173)
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
- CORS is configured for localhost:5173 (Vite) and localhost:3000 (backend)
- File uploads are limited to 10MB with custom upload directory support
- Use `cargo watch -x run` in backend/ for auto-reload during Rust development

## Database Schema

The application uses PostgreSQL with the following key tables:
- `users` - User accounts with bcrypt password hashing
- `plants` - Plant records with care schedules and custom metrics
- `tracking_entries` - Care activities (watering, fertilizing, custom metrics)
- `photos` - Plant photos with thumbnail support
- `sessions` - Authentication session storage