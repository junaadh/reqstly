# Frontend Functionality (Embedded Auth Baseline)

This document maps frontend behavior to backend contracts in the current Phase 5 baseline.

## Architecture Contract

- Frontend keeps custom Reqstly auth UI (`/login`, `/signup`).
- Frontend talks only to Reqstly backend APIs under `/api/v1` (via SvelteKit proxy routes).
- No frontend calls to Supabase SDK or `/auth/v1/*`.
- No frontend dependency on authentik/OIDC providers in baseline.
- OIDC/Entra UI/runtime remains deferred to Phase D.

## Auth Flows

## 1) Login (`/login`)

Features:
- Email/password sign-in.
- Passkey sign-in.
- Redirect to `next` or `/` on success.
- No unauthenticated `/api/me` probe on mount.

API contracts:
- `POST /api/v1/auth/login/password`
- `POST /api/v1/auth/passkeys/login/start`
- `POST /api/v1/auth/passkeys/login/finish`
- `GET /api/v1/auth/csrf` (post-auth bootstrap for mutation flows)

## 2) Signup (`/signup`)

Features:
- Email/password signup.
- Passkey signup flow (email + display name, passwordless).

API contracts:
- `POST /api/v1/auth/signup`
- `POST /api/v1/auth/passkeys/signup/start`
- `POST /api/v1/auth/passkeys/signup/finish`
- `GET /api/v1/auth/csrf` (post-auth bootstrap for mutation flows)

## 3) Passkey Enrollment (Authenticated)

Features:
- Existing signed-in users can add passkeys from profile/settings.

API contracts:
- `GET /api/v1/auth/passkeys`
- `POST /api/v1/auth/passkeys/register/start`
- `POST /api/v1/auth/passkeys/register/finish`

## 4) Logout and Session Revocation

API contracts:
- `POST /api/v1/auth/logout`
- `POST /api/v1/auth/sessions/revoke` (invalidate all active sessions/ws tokens)

## 5) Session Bootstrap and Guards

Frontend app bootstrap:
1. Load shell
2. Call `/api/v1/me`
3. Route unauthenticated users to `/login`

API contract:
- `GET /api/v1/me`

## 6) WebSocket Auth

Primary path:
- Browser clients use same-site session cookie.

Compatibility path:
- `POST /api/v1/auth/ws-token` mints short-lived bearer token.
- `/ws` accepts bearer token via `Authorization` header or `?token=`.
- Bearer tokens must be minted by Reqstly (`/auth/ws-token`), not ad-hoc signed.

## 7) CSRF and Origin

- Authenticated browser mutation endpoints require CSRF token header.
- Frontend retrieves token from `GET /api/v1/auth/csrf`.
- Mutation requests include `X-CSRF-Token`.
- Server-side proxy forwards `Origin` for mutating calls to satisfy backend origin checks.

## App Data Flows (`/api/v1`)

## 8) Dashboard (`/`)

- `GET /api/v1/me`
- `GET /api/v1/requests?status=<status>&page=1&limit=1`
- `GET /api/v1/requests?page=1&limit=6&sort=-updated_at`

## 9) Requests List (`/requests`)

- `GET /api/v1/requests?...`
- `GET /api/v1/meta/enums`
- `GET /api/v1/assignees/suggestions?limit=<n>&q=<term>`

## 10) Create Request (`/requests/new`)

- `GET /api/v1/meta/enums`
- `POST /api/v1/requests`

## 11) Request Detail (`/requests/[id]`)

- `GET /api/v1/requests/{id}`
- `PATCH /api/v1/requests/{id}`
- `DELETE /api/v1/requests/{id}`
- `GET /api/v1/requests/{id}/audit`

## 12) Settings (`/settings`)

- `GET /api/v1/preferences`
- `PATCH /api/v1/preferences`

## Contract Notes

- Responses use Reqstly success/error envelope shape.
- Authenticated browser mutation endpoints are CSRF-aware.
- Business UI identity is always app-level (`app.app_users`), never provider-internal identity.
