# Frontend Functionality (Phase 5 Embedded Auth Baseline)

This document maps frontend behavior to backend contracts for Phase 5.

## Architecture Contract

- Frontend keeps custom Reqstly auth UI (`/login`, `/signup`).
- Frontend talks only to Reqstly backend APIs under `/api/v1`.
- No frontend calls to Supabase SDK or `/auth/v1/*`.
- No frontend dependency on authentik/OIDC providers in baseline.
- OIDC/Entra UI/runtime paths are deferred to Phase D.

## Auth Flows

## 1) Login (`/login`)

Features:
- Email/password sign-in.
- Passkey sign-in.
- Redirect to `next` or `/` on success.

API contracts:
- `POST /api/v1/auth/login/password`
  - purpose: authenticate email/password and establish session cookie.
- `POST /api/v1/auth/passkeys/login/start`
  - purpose: start WebAuthn assertion challenge.
- `POST /api/v1/auth/passkeys/login/finish`
  - purpose: verify assertion and establish session cookie.

## 2) Signup (`/signup`)

Features:
- Email/password signup remains enabled.
- Optional post-signup passkey enrollment entry.

API contracts:
- `POST /api/v1/auth/signup`
  - purpose: create account + password identity + session.

## 3) Passkey Enrollment (`/settings` or `/profile`)

Baseline rule: enrollment requires authenticated session.

API contracts:
- `POST /api/v1/auth/passkeys/register/start`
  - purpose: start WebAuthn registration challenge.
- `POST /api/v1/auth/passkeys/register/finish`
  - purpose: verify registration and persist credential.

## 4) Logout

API contracts:
- `POST /api/v1/auth/logout`
  - purpose: destroy current session and clear cookie.

## 5) Session Bootstrap and Guard

Frontend app bootstrap:
1. Load shell
2. Call `/api/v1/me`
3. Route unauthenticated users to `/login`

API contracts:
- `GET /api/v1/me`
  - purpose: resolve authenticated `app_user` profile from session.

## 6) WebSocket Auth

Baseline target is cookie-first:
- Browser clients: rely on same-site session cookie.

Compatibility path:
- `POST /api/v1/auth/ws-token` mints short-lived bearer token.
- `/ws` accepts bearer token via `Authorization` header or `?token=`.

## App Data Flows (Unchanged `/api/v1` Prefix)

## 7) Dashboard (`/`)

- `GET /api/v1/me`
- `GET /api/v1/requests?status=<status>&page=1&limit=1`
- `GET /api/v1/requests?page=1&limit=6&sort=-updated_at`

## 8) Requests List (`/requests`)

- `GET /api/v1/requests?...`
- `GET /api/v1/meta/enums`
- `GET /api/v1/assignees/suggestions?limit=<n>&q=<term>`

## 9) Create Request (`/requests/new`)

- `GET /api/v1/meta/enums`
- `POST /api/v1/requests`

## 10) Request Detail (`/requests/[id]`)

- `GET /api/v1/requests/{id}`
- `PATCH /api/v1/requests/{id}`
- `DELETE /api/v1/requests/{id}`
- `GET /api/v1/requests/{id}/audit`

## 11) Settings (`/settings`)

- `GET /api/v1/preferences`
- `PATCH /api/v1/preferences`

## Contract Notes

- All responses use Reqstly success/error envelope shape.
- Authenticated browser mutation endpoints are CSRF-aware.
- Frontend must not store plaintext credentials or WebAuthn challenge state outside intended flow.
- Business UI identity is always app-level (`app.app_users`), not provider-internal identity.
