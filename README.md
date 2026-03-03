# Planty MVP

Planty is a full-stack app for tracking plant care schedules, activity logs, photos, and reminders.

## MVP Features
- Invite-based registration and session authentication
- Invite quotas for users (no unlimited user creation)
- Plant CRUD with watering/fertilizing schedule tracking
- Tracking entries (watering, fertilizing, notes, custom metrics, photos)
- Photo uploads and gallery/preview management
- Calendar subscription feed (`.ics`) with persistent token + rotation
- Google Tasks integration (optional, when configured)
- Admin dashboard and user/invite management
- User settings: profile update, password change, data export, account deletion

## Feature Flags
- `WAITLIST_ENABLED` (backend): defaults to `false`. Enables waitlist API routes when set to `true`.
- `VITE_WAITLIST_ENABLED` (frontend): defaults to `false`. Shows waitlist UI when set to `true`.

## Stack
- Frontend: SolidJS + Vite + TypeScript
- Backend: Rust + Axum + SQLx
- Database: SQLite

## Versioning
- Root package: `1.0.0`
- Frontend package: `1.0.0`
- Backend crate: `1.0.0`
- Tag format: `vMAJOR.MINOR.PATCH`

## Local Development

```bash
npm run install:frontend
npm run dev
```

Dev mode defaults:
- Frontend (Vite + HMR): `http://localhost:3000`
- Backend API (Axum via cargo-watch): `http://localhost:3001`
- Vite proxies `/api/*` to backend automatically.

If you do not have `cargo-watch` yet:
```bash
cargo install cargo-watch
```

## Quality Gate

```bash
npm run lint && npm run test && npm run build
```

## CI
GitHub Actions workflow: `.github/workflows/ci.yml`
- Runs on pull requests and pushes to `main`
- Enforces `lint`, `test`, and `build`

## Ops Docs
- [Release Runbook](./RELEASE_RUNBOOK.md)
- [Release Checklist](./RELEASE_CHECKLIST.md)
- [Backend README](./backend/README.md)
