# Release Checklist

## Quality
- [ ] `npm run lint` passed
- [ ] `npm run test` passed
- [ ] `npm run build` passed
- [ ] CI workflow green on target commit

## Backend / DB
- [ ] Migrations apply successfully on clean DB
- [ ] Existing DB upgrade verified
- [ ] Health endpoint responds correctly

## Core Flows
- [ ] Invite validation and registration
- [ ] Login/logout/session persistence
- [ ] Plant CRUD
- [ ] Tracking entries CRUD
- [ ] Photo upload/list/delete
- [ ] Calendar subscription URL generation
- [ ] Calendar token rotation invalidates old token

## Settings / Account
- [ ] Profile update works
- [ ] Password change works
- [ ] Data export downloads JSON payload
- [ ] Account deletion removes account and logs out

## Integrations / Admin
- [ ] Google Tasks connect/status/disconnect/sync (when configured)
- [ ] Admin dashboard/user list/invite controls load

## UX / Platform
- [ ] Desktop smoke check
- [ ] Mobile viewport smoke check
- [ ] PWA install/offline shell basic verification

## Release
- [ ] Version/tag decided (`vMAJOR.MINOR.PATCH`)
- [ ] Rollback plan confirmed
- [ ] Final sign-off recorded
