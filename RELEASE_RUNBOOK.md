# Release Runbook

## 1. Preconditions
- Branch is up to date with target base.
- `backend/.env` is configured for target environment.
- Database backup exists for production/staging before deploy.

## 2. Validate Quality Gate
Run from repo root:

```bash
npm run lint && npm run test && npm run build
```

Expected: all commands pass.

## 3. Database Migration
Migrations are applied automatically by backend startup (`sqlx::migrate!`).
For manual verification:

```bash
cd backend
sqlx migrate run
```

## 4. Deploy Steps
1. Build frontend and backend artifacts.
2. Deploy backend binary and frontend dist together.
3. Start backend process with correct env vars.

Required env vars:
- `DATABASE_URL`
- `PORT`
- `FRONTEND_DIR`
- `RUST_LOG`

Optional integration env vars:
- Google Tasks OAuth settings (`GOOGLE_*` values used by app)

## 5. Post-Deploy Smoke Checks
- `GET /api/health` returns 200 and expected version.
- Login/logout works.
- Invite registration works.
- Plant create/edit/delete works.
- Tracking and photo upload flows work.
- Calendar subscription URL works with valid token.
- Calendar regenerate token invalidates previous URL.
- User settings: profile update/password change/export/delete all work.

## 6. Rollback
- Roll back to previous deployed artifact.
- Restore DB backup only if migration introduced an incompatible issue.
- Re-run smoke checks after rollback.
