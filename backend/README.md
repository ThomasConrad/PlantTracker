# Planty API

Rust + Axum backend for Planty MVP.

## Setup

### Prerequisites
- Rust stable
- SQLite 3
- `sqlx-cli` (optional for manual migration commands)

### Environment

```bash
cp .env.example .env
```

Set at least:
- `DATABASE_URL`
- `PORT`
- `FRONTEND_DIR`
- `RUST_LOG`
- `WAITLIST_ENABLED` (`false` by default; set `true` to expose waitlist APIs)

## Run

```bash
cargo run
```

## Test

```bash
cargo test
```

## Migrations

Migrations are applied automatically on startup via `sqlx::migrate!`.
Manual migration command:

```bash
sqlx migrate run
```

## Key API Areas
- Auth: login/register/logout/me + profile/password/export/delete account
- Invites (waitlist is feature-flagged)
- Plants/tracking/photos
- Calendar feed and token rotation
- Google Tasks integration
- Admin dashboard and user controls

## Notes
- Uses SQLite by default.
- Session auth is cookie-based with DB-backed sessions.
